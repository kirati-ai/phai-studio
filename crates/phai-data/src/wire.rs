//! MySQL's client/server protocol, as much as a data explorer needs: the handshake (TLS when asked),
//! the three ways a password is proven - `mysql_native_password`, `caching_sha2_password` and
//! `sha256_password` - a switch from one to another, text queries with several results, OK and
//! error packets. Hashing, RSA and TLS come from crates and from Windows; none is written here.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use sha1::{Digest, Sha1};
use sha2::Sha256;

const CLIENT_LONG_PASSWORD: u32 = 0x1;
const CLIENT_LONG_FLAG: u32 = 0x4;
const CLIENT_CONNECT_WITH_DB: u32 = 0x8;
const CLIENT_PROTOCOL_41: u32 = 0x200;
const CLIENT_SSL: u32 = 0x800;
const CLIENT_TRANSACTIONS: u32 = 0x2000;
const CLIENT_SECURE_CONNECTION: u32 = 0x8000;
const CLIENT_MULTI_STATEMENTS: u32 = 0x1_0000;
const CLIENT_MULTI_RESULTS: u32 = 0x2_0000;
const CLIENT_PLUGIN_AUTH: u32 = 0x8_0000;
const CLIENT_PLUGIN_AUTH_LENENC: u32 = 0x20_0000;
const SERVER_MORE_RESULTS: u16 = 0x0008;
/// utf8mb4, so Thai and every other script come and go whole.
const UTF8MB4: u8 = 45;

/// Where to connect, and as whom.
pub struct Login<'a> {
    pub host: &'a str,
    pub port: u16,
    pub user: &'a str,
    pub password: &'a str,
    pub database: Option<&'a str>,
    pub ssl: bool,
    pub trust_cert: bool,
}

enum Stream {
    Plain(TcpStream),
    Tls(Box<native_tls::TlsStream<TcpStream>>),
    /// For the moment the plain stream is handed to TLS.
    Gone,
}

impl Read for Stream {
    fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Stream::Plain(s) => s.read(b),
            Stream::Tls(s) => s.read(b),
            Stream::Gone => Err(std::io::ErrorKind::NotConnected.into()),
        }
    }
}

impl Write for Stream {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        match self {
            Stream::Plain(s) => s.write(b),
            Stream::Tls(s) => s.write(b),
            Stream::Gone => Err(std::io::ErrorKind::NotConnected.into()),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Stream::Plain(s) => s.flush(),
            Stream::Tls(s) => s.flush(),
            Stream::Gone => Ok(()),
        }
    }
}

/// One result of a query: columns and rows (a NULL as `None`), or the rows a statement changed.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Result1 {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Option<String>>>,
    pub affected: u64,
}

pub struct Conn {
    s: Stream,
    seq: u8,
    pub server_version: String,
    tls: bool,
}

fn err(s: impl Into<String>) -> String {
    s.into()
}

fn io(e: std::io::Error) -> String {
    e.to_string()
}

// --- the little readers -------------------------------------------------------------------------

struct Cur<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Cur<'a> {
    fn new(b: &'a [u8]) -> Cur<'a> {
        Cur { b, at: 0 }
    }
    fn u8(&mut self) -> u8 {
        let v = self.b.get(self.at).copied().unwrap_or(0);
        self.at += 1;
        v
    }
    fn u16(&mut self) -> u16 {
        self.u8() as u16 | (self.u8() as u16) << 8
    }
    fn u32(&mut self) -> u32 {
        self.u16() as u32 | (self.u16() as u32) << 16
    }
    fn take(&mut self, n: usize) -> &'a [u8] {
        let end = (self.at + n).min(self.b.len());
        let s = &self.b[self.at.min(end)..end];
        self.at = end;
        s
    }
    fn nul(&mut self) -> &'a [u8] {
        let rest = &self.b[self.at.min(self.b.len())..];
        let n = rest.iter().position(|c| *c == 0).unwrap_or(rest.len());
        self.at += n + 1;
        &rest[..n]
    }
    fn rest(&mut self) -> &'a [u8] {
        let s = &self.b[self.at.min(self.b.len())..];
        self.at = self.b.len();
        s
    }
    /// A length-encoded integer; `None` for 0xFB, a NULL.
    fn lenenc(&mut self) -> Option<u64> {
        match self.u8() {
            0xFB => None,
            0xFC => Some(self.u16() as u64),
            0xFD => Some(self.u16() as u64 | (self.u8() as u64) << 16),
            0xFE => Some(self.u32() as u64 | (self.u32() as u64) << 32),
            n => Some(n as u64),
        }
    }
    fn lenenc_str(&mut self) -> Option<&'a [u8]> {
        let n = self.lenenc()? as usize;
        Some(self.take(n))
    }
    fn done(&self) -> bool {
        self.at >= self.b.len()
    }
}

fn put_lenenc(out: &mut Vec<u8>, n: u64) {
    if n < 251 {
        out.push(n as u8);
    } else if n < 0x1_0000 {
        out.push(0xFC);
        out.extend_from_slice(&(n as u16).to_le_bytes());
    } else if n < 0x100_0000 {
        out.push(0xFD);
        out.extend_from_slice(&(n as u32).to_le_bytes()[..3]);
    } else {
        out.push(0xFE);
        out.extend_from_slice(&n.to_le_bytes());
    }
}

/// An error packet's words: `(1045) Access denied for user ...`.
fn server_error(p: &[u8]) -> String {
    let mut c = Cur::new(p);
    c.u8();
    let code = c.u16();
    let mut rest = c.rest();
    if rest.first() == Some(&b'#') {
        rest = &rest[6.min(rest.len())..];
    }
    format!("({code}) {}", String::from_utf8_lossy(rest))
}

// --- proving the password ------------------------------------------------------------------------

fn xor(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b.iter().cycle()).map(|(x, y)| x ^ y).collect()
}

/// `mysql_native_password`: SHA1(password) XOR SHA1(scramble + SHA1(SHA1(password))).
pub fn native_password(password: &str, scramble: &[u8]) -> Vec<u8> {
    if password.is_empty() {
        return vec![];
    }
    let h1 = Sha1::digest(password.as_bytes());
    let h2 = Sha1::digest(h1);
    let mut h = Sha1::new();
    h.update(scramble);
    h.update(h2);
    xor(&h1, &h.finalize())
}

/// `caching_sha2_password`'s fast proof: SHA256(password) XOR SHA256(SHA256(SHA256(password)) + scramble).
pub fn caching_sha2(password: &str, scramble: &[u8]) -> Vec<u8> {
    if password.is_empty() {
        return vec![];
    }
    let h1 = Sha256::digest(password.as_bytes());
    let h2 = Sha256::digest(h1);
    let mut h = Sha256::new();
    h.update(h2);
    h.update(scramble);
    xor(&h1, &h.finalize())
}

/// The password for a server that wants it whole without TLS: XORed with the scramble and
/// encrypted to the server's own RSA key (OAEP), so only it can read it.
fn rsa_password(password: &str, scramble: &[u8], pem: &[u8]) -> Result<Vec<u8>, String> {
    use rsa::pkcs8::DecodePublicKey;
    let text = String::from_utf8_lossy(pem);
    let key = rsa::RsaPublicKey::from_public_key_pem(text.trim()).or_else(|_| {
        use rsa::pkcs1::DecodeRsaPublicKey;
        rsa::RsaPublicKey::from_pkcs1_pem(text.trim())
    });
    let key = key.map_err(|e| format!("server key: {e}"))?;
    let mut plain = password.as_bytes().to_vec();
    plain.push(0);
    let mixed = xor(&plain, scramble);
    key.encrypt(&mut rand::thread_rng(), rsa::Oaep::new::<Sha1>(), &mixed).map_err(|e| e.to_string())
}

impl Conn {
    pub fn connect(l: &Login) -> Result<Conn, String> {
        let addr = (l.host, l.port).to_socket_addrs().map_err(io)?.next().ok_or_else(|| err(format!("{} not found", l.host)))?;
        let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(6)).map_err(io)?;
        tcp.set_read_timeout(Some(Duration::from_secs(60))).map_err(io)?;
        tcp.set_write_timeout(Some(Duration::from_secs(60))).map_err(io)?;
        let _ = tcp.set_nodelay(true);
        let mut c = Conn { s: Stream::Plain(tcp), seq: 0, server_version: String::new(), tls: false };
        // The server speaks first.
        let hello = c.read_packet()?;
        if hello.first() == Some(&0xFF) {
            return Err(server_error(&hello));
        }
        let mut h = Cur::new(&hello);
        if h.u8() != 10 {
            return Err(err("not a MySQL server (protocol 10)"));
        }
        c.server_version = String::from_utf8_lossy(h.nul()).to_string();
        h.u32();
        let mut scramble = h.take(8).to_vec();
        h.u8();
        let mut caps = h.u16() as u32;
        h.u8();
        h.u16();
        caps |= (h.u16() as u32) << 16;
        let data_len = h.u8() as usize;
        h.take(10);
        if caps & CLIENT_SECURE_CONNECTION != 0 {
            let n = data_len.saturating_sub(8).max(13);
            let part = h.take(n);
            scramble.extend_from_slice(part.strip_suffix(&[0]).unwrap_or(part));
        }
        let mut plugin = if caps & CLIENT_PLUGIN_AUTH != 0 { String::from_utf8_lossy(h.nul()).to_string() } else { "mysql_native_password".into() };
        if plugin.is_empty() {
            plugin = "mysql_native_password".into();
        }

        let mut ours = CLIENT_LONG_PASSWORD | CLIENT_LONG_FLAG | CLIENT_PROTOCOL_41 | CLIENT_TRANSACTIONS | CLIENT_SECURE_CONNECTION | CLIENT_MULTI_STATEMENTS | CLIENT_MULTI_RESULTS | CLIENT_PLUGIN_AUTH | CLIENT_PLUGIN_AUTH_LENENC;
        if l.database.is_some() {
            ours |= CLIENT_CONNECT_WITH_DB;
        }
        if l.ssl {
            if caps & CLIENT_SSL == 0 {
                return Err(err("the server does not offer TLS"));
            }
            ours |= CLIENT_SSL;
            // Ask for TLS, then talk through it.
            let mut req = Vec::new();
            req.extend_from_slice(&ours.to_le_bytes());
            req.extend_from_slice(&(16u32 << 20).to_le_bytes());
            req.push(UTF8MB4);
            req.extend_from_slice(&[0u8; 23]);
            c.write_packet(&req)?;
            let connector = native_tls::TlsConnector::builder().danger_accept_invalid_certs(l.trust_cert).danger_accept_invalid_hostnames(l.trust_cert).build().map_err(|e| e.to_string())?;
            let Stream::Plain(tcp) = std::mem::replace(&mut c.s, Stream::Gone) else { return Err(err("TLS twice")) };
            let tls = connector.connect(l.host, tcp).map_err(|e| e.to_string())?;
            c.s = Stream::Tls(Box::new(tls));
            c.tls = true;
        }
        let auth = c.first_proof(&plugin, l.password, &scramble);
        let mut resp = Vec::new();
        resp.extend_from_slice(&ours.to_le_bytes());
        resp.extend_from_slice(&(16u32 << 20).to_le_bytes());
        resp.push(UTF8MB4);
        resp.extend_from_slice(&[0u8; 23]);
        resp.extend_from_slice(l.user.as_bytes());
        resp.push(0);
        put_lenenc(&mut resp, auth.len() as u64);
        resp.extend_from_slice(&auth);
        if let Some(db) = l.database {
            resp.extend_from_slice(db.as_bytes());
            resp.push(0);
        }
        resp.extend_from_slice(plugin.as_bytes());
        resp.push(0);
        c.write_packet(&resp)?;
        c.finish_auth(plugin, l.password, scramble)?;
        Ok(c)
    }

    fn first_proof(&self, plugin: &str, password: &str, scramble: &[u8]) -> Vec<u8> {
        match plugin {
            "caching_sha2_password" => caching_sha2(password, scramble),
            "sha256_password" if password.is_empty() => vec![],
            // Over TLS the password itself; without, ask for the server's key (1).
            "sha256_password" if self.tls => [password.as_bytes(), &[0]].concat(),
            "sha256_password" => vec![1],
            "mysql_clear_password" if self.tls => [password.as_bytes(), &[0]].concat(),
            _ => native_password(password, scramble),
        }
    }

    /// The server's answers until it says OK: a switch to another plugin, more data asked for,
    /// its public key.
    fn finish_auth(&mut self, mut plugin: String, password: &str, mut scramble: Vec<u8>) -> Result<(), String> {
        for _ in 0..6 {
            let p = self.read_packet()?;
            match p.first() {
                Some(0x00) => return Ok(()),
                Some(0xFF) => return Err(server_error(&p)),
                Some(0xFE) => {
                    // Auth switch: another plugin, a new scramble.
                    let mut c = Cur::new(&p);
                    c.u8();
                    plugin = String::from_utf8_lossy(c.nul()).to_string();
                    let data = c.rest();
                    scramble = data.strip_suffix(&[0]).unwrap_or(data).to_vec();
                    let proof = self.first_proof(&plugin, password, &scramble);
                    self.write_packet(&proof)?;
                }
                Some(0x01) => {
                    let data = &p[1..];
                    match (plugin.as_str(), data) {
                        ("caching_sha2_password", [3]) => {} // fast auth done; OK follows
                        ("caching_sha2_password", [4]) => {
                            // Full auth: the password itself over TLS, else to the server's key.
                            if self.tls {
                                self.write_packet(&[password.as_bytes(), &[0]].concat())?;
                            } else {
                                self.write_packet(&[2])?;
                            }
                        }
                        (_, pem) if pem.starts_with(b"-----BEGIN") => {
                            let enc = rsa_password(password, &scramble, pem)?;
                            self.write_packet(&enc)?;
                        }
                        _ => return Err(err(format!("unexpected answer from {plugin}"))),
                    }
                }
                _ if plugin == "sha256_password" && p.starts_with(b"-----BEGIN") => {
                    let enc = rsa_password(password, &scramble, &p)?;
                    self.write_packet(&enc)?;
                }
                _ => return Err(err("unexpected packet while logging in")),
            }
        }
        Err(err("logging in did not finish"))
    }

    // --- packets -------------------------------------------------------------------------------

    fn read_packet(&mut self) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        loop {
            let mut head = [0u8; 4];
            self.s.read_exact(&mut head).map_err(io)?;
            let len = head[0] as usize | (head[1] as usize) << 8 | (head[2] as usize) << 16;
            self.seq = head[3].wrapping_add(1);
            let start = out.len();
            out.resize(start + len, 0);
            self.s.read_exact(&mut out[start..]).map_err(io)?;
            // A packet of 16 MB exactly goes on in the next.
            if len < 0xFF_FFFF {
                return Ok(out);
            }
        }
    }

    fn write_packet(&mut self, payload: &[u8]) -> Result<(), String> {
        let mut chunks = payload.chunks(0xFF_FFFF).peekable();
        if payload.is_empty() {
            let head = [0, 0, 0, self.seq];
            self.seq = self.seq.wrapping_add(1);
            self.s.write_all(&head).map_err(io)?;
        }
        while let Some(c) = chunks.next() {
            let n = c.len();
            let head = [n as u8, (n >> 8) as u8, (n >> 16) as u8, self.seq];
            self.seq = self.seq.wrapping_add(1);
            self.s.write_all(&head).map_err(io)?;
            self.s.write_all(c).map_err(io)?;
            if n == 0xFF_FFFF && chunks.peek().is_none() {
                let head = [0, 0, 0, self.seq];
                self.seq = self.seq.wrapping_add(1);
                self.s.write_all(&head).map_err(io)?;
            }
        }
        self.s.flush().map_err(io)
    }

    // --- queries -------------------------------------------------------------------------------

    /// Runs `sql` (several statements allowed); every result, in order. Rows past `max` in a
    /// result are read and dropped.
    pub fn query(&mut self, sql: &str, max: usize) -> Result<Vec<(Result1, bool)>, String> {
        self.seq = 0;
        let mut p = vec![0x03];
        p.extend_from_slice(sql.as_bytes());
        self.write_packet(&p)?;
        let mut out = Vec::new();
        loop {
            let first = self.read_packet()?;
            let more = match first.first() {
                Some(0x00) => {
                    let mut c = Cur::new(&first);
                    c.u8();
                    let affected = c.lenenc().unwrap_or(0);
                    c.lenenc();
                    let status = c.u16();
                    out.push((Result1 { affected, ..Default::default() }, false));
                    status & SERVER_MORE_RESULTS != 0
                }
                Some(0xFF) => return Err(server_error(&first)),
                Some(0xFB) => return Err(err("the server asked for a local file; that is not done")),
                _ => {
                    let n = Cur::new(&first).lenenc().unwrap_or(0) as usize;
                    let mut columns = Vec::with_capacity(n);
                    for _ in 0..n {
                        let d = self.read_packet()?;
                        let mut c = Cur::new(&d);
                        for _ in 0..4 {
                            c.lenenc_str();
                        }
                        let name = c.lenenc_str().unwrap_or_default();
                        columns.push(String::from_utf8_lossy(name).to_string());
                    }
                    // EOF after the columns.
                    let eof = self.read_packet()?;
                    if eof.first() == Some(&0xFF) {
                        return Err(server_error(&eof));
                    }
                    let mut rows = Vec::new();
                    let mut cut = false;
                    let status;
                    loop {
                        let r = self.read_packet()?;
                        if r.first() == Some(&0xFE) && r.len() < 9 {
                            let mut c = Cur::new(&r);
                            c.u8();
                            c.u16();
                            status = c.u16();
                            break;
                        }
                        if r.first() == Some(&0xFF) {
                            return Err(server_error(&r));
                        }
                        if rows.len() >= max {
                            cut = true;
                            continue;
                        }
                        let mut c = Cur::new(&r);
                        let mut row = Vec::with_capacity(n);
                        for _ in 0..n {
                            row.push(if c.done() { None } else { c.lenenc_str().map(|b| String::from_utf8_lossy(b).to_string()) });
                        }
                        rows.push(row);
                    }
                    out.push((Result1 { columns, rows, affected: 0 }, cut));
                    status & SERVER_MORE_RESULTS != 0
                }
            };
            if !more {
                return Ok(out);
            }
        }
    }
}

// --- prepared statements -------------------------------------------------------------------------

/// A column as a prepared statement's binary rows need it: its type and whether it is unsigned.
#[derive(Clone, Copy, Debug)]
struct ColType {
    ty: u8,
    unsigned: bool,
}

fn column_def(d: &[u8]) -> (String, ColType) {
    let mut c = Cur::new(d);
    for _ in 0..4 {
        c.lenenc_str();
    }
    let name = String::from_utf8_lossy(c.lenenc_str().unwrap_or_default()).to_string();
    c.lenenc_str();
    c.lenenc();
    c.u16();
    c.u32();
    let ty = c.u8();
    let flags = c.u16();
    (name, ColType { ty, unsigned: flags & 0x20 != 0 })
}

/// The packet that runs statement `id` with `params`, each sent as text (the server converts it)
/// or NULL.
pub fn execute_packet(id: u32, params: &[Option<&str>]) -> Vec<u8> {
    let mut p = vec![0x17];
    p.extend_from_slice(&id.to_le_bytes());
    p.push(0); // no cursor
    p.extend_from_slice(&1u32.to_le_bytes());
    if !params.is_empty() {
        let mut nulls = vec![0u8; params.len().div_ceil(8)];
        for (i, v) in params.iter().enumerate() {
            if v.is_none() {
                nulls[i / 8] |= 1 << (i % 8);
            }
        }
        p.extend_from_slice(&nulls);
        p.push(1); // types follow
        for _ in params {
            p.extend_from_slice(&[0xFD, 0x00]); // VAR_STRING
        }
        for v in params.iter().flatten() {
            put_lenenc(&mut p, v.len() as u64);
            p.extend_from_slice(v.as_bytes());
        }
    }
    p
}

/// A binary row's values as text, by the columns' types.
fn binary_row(r: &[u8], cols: &[ColType]) -> Vec<Option<String>> {
    let mut c = Cur::new(r);
    c.u8(); // 0x00
    let bitmap = c.take((cols.len() + 7 + 2) / 8).to_vec();
    let is_null = |i: usize| bitmap.get((i + 2) / 8).is_some_and(|b| b & (1 << ((i + 2) % 8)) != 0);
    let mut out = Vec::with_capacity(cols.len());
    for (i, col) in cols.iter().enumerate() {
        if is_null(i) {
            out.push(None);
            continue;
        }
        let v = match col.ty {
            0x01 => Some(if col.unsigned { (c.u8()).to_string() } else { (c.u8() as i8).to_string() }),
            0x02 | 0x0D => {
                let v = c.u16();
                Some(if col.unsigned { v.to_string() } else { (v as i16).to_string() })
            }
            0x03 | 0x09 => {
                let v = c.u32();
                Some(if col.unsigned { v.to_string() } else { (v as i32).to_string() })
            }
            0x08 => {
                let v = c.u32() as u64 | (c.u32() as u64) << 32;
                Some(if col.unsigned { v.to_string() } else { (v as i64).to_string() })
            }
            0x04 => Some(f32::from_bits(c.u32()).to_string()),
            0x05 => Some(f64::from_bits(c.u32() as u64 | (c.u32() as u64) << 32).to_string()),
            0x0A | 0x07 | 0x0C => {
                // DATE, TIMESTAMP, DATETIME: a length, then as much of the date as there is.
                let n = c.u8();
                let (mut y, mut mo, mut d, mut h, mut mi, mut se, mut us) = (0u16, 0u8, 0u8, 0u8, 0u8, 0u8, 0u32);
                if n >= 4 {
                    y = c.u16();
                    mo = c.u8();
                    d = c.u8();
                }
                if n >= 7 {
                    h = c.u8();
                    mi = c.u8();
                    se = c.u8();
                }
                if n >= 11 {
                    us = c.u32();
                }
                Some(if col.ty == 0x0A || n < 7 { format!("{y:04}-{mo:02}-{d:02}") } else if us == 0 { format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{se:02}") } else { format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{se:02}.{us:06}") })
            }
            0x0B => {
                let n = c.u8();
                let (mut neg, mut days, mut h, mut mi, mut se, mut us) = (false, 0u32, 0u8, 0u8, 0u8, 0u32);
                if n >= 8 {
                    neg = c.u8() == 1;
                    days = c.u32();
                    h = c.u8();
                    mi = c.u8();
                    se = c.u8();
                }
                if n >= 12 {
                    us = c.u32();
                }
                let hours = days as u64 * 24 + h as u64;
                Some(format!("{}{hours:02}:{mi:02}:{se:02}{}", if neg { "-" } else { "" }, if us == 0 { String::new() } else { format!(".{us:06}") }))
            }
            0x06 => None,
            // Strings, decimals, blobs, JSON, enums, sets, bits: length-encoded bytes.
            _ => c.lenenc_str().map(|b| String::from_utf8_lossy(b).to_string()),
        };
        out.push(v);
    }
    out
}

impl Conn {
    /// Runs one statement with `?` marks and `params` in their places - the values go apart from
    /// the SQL, so nothing in them can change what the SQL does.
    pub fn execute(&mut self, sql: &str, params: &[Option<&str>], max: usize) -> Result<(Result1, bool), String> {
        self.seq = 0;
        let mut p = vec![0x16];
        p.extend_from_slice(sql.as_bytes());
        self.write_packet(&p)?;
        let ok = self.read_packet()?;
        if ok.first() == Some(&0xFF) {
            return Err(server_error(&ok));
        }
        let mut c = Cur::new(&ok);
        c.u8();
        let id = c.u32();
        let ncols = c.u16() as usize;
        let nparams = c.u16() as usize;
        // The parameters' and the columns' descriptions, each followed by EOF.
        if nparams > 0 {
            for _ in 0..nparams {
                self.read_packet()?;
            }
            self.read_packet()?;
        }
        if ncols > 0 {
            for _ in 0..ncols {
                self.read_packet()?;
            }
            self.read_packet()?;
        }
        if nparams != params.len() {
            self.close_statement(id);
            return Err(format!("the statement has {nparams} ? marks, {} values were given", params.len()));
        }
        self.seq = 0;
        self.write_packet(&execute_packet(id, params))?;
        let first = self.read_packet()?;
        let result = match first.first() {
            Some(0xFF) => Err(server_error(&first)),
            Some(0x00) => {
                let mut c = Cur::new(&first);
                c.u8();
                Ok((Result1 { affected: c.lenenc().unwrap_or(0), ..Default::default() }, false))
            }
            _ => {
                let n = Cur::new(&first).lenenc().unwrap_or(0) as usize;
                let mut names = Vec::with_capacity(n);
                let mut types = Vec::with_capacity(n);
                for _ in 0..n {
                    let (name, ty) = column_def(&self.read_packet()?);
                    names.push(name);
                    types.push(ty);
                }
                self.read_packet()?;
                let mut rows = Vec::new();
                let mut cut = false;
                loop {
                    let r = self.read_packet()?;
                    if r.first() == Some(&0xFE) && r.len() < 9 {
                        break;
                    }
                    if r.first() == Some(&0xFF) {
                        return Err(server_error(&r));
                    }
                    if rows.len() >= max {
                        cut = true;
                        continue;
                    }
                    rows.push(binary_row(&r, &types));
                }
                Ok((Result1 { columns: names, rows, affected: 0 }, cut))
            }
        };
        self.close_statement(id);
        result
    }

    fn close_statement(&mut self, id: u32) {
        self.seq = 0;
        let mut p = vec![0x19];
        p.extend_from_slice(&id.to_le_bytes());
        let _ = self.write_packet(&p);
    }
}

impl Drop for Conn {
    fn drop(&mut self) {
        // COM_QUIT, then the socket goes.
        self.seq = 0;
        let _ = self.write_packet(&[0x01]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proofs_match_mysqls_own() {
        // From MySQL's documentation of the native scramble: an empty password sends nothing.
        assert!(native_password("", b"12345678901234567890").is_empty());
        assert_eq!(native_password("pw", b"abcdefghijklmnopqrst").len(), 20);
        assert_eq!(caching_sha2("pw", b"abcdefghijklmnopqrst").len(), 32);
        let mut v = Vec::new();
        put_lenenc(&mut v, 300);
        assert_eq!(v, [0xFC, 0x2C, 0x01]);
        assert_eq!(Cur::new(&v).lenenc(), Some(300));
        assert_eq!(server_error(b"\xff\x15\x04#28000Access denied"), "(1045) Access denied");
    }

    #[test]
    fn prepared_statements_encode_and_decode() {
        let p = execute_packet(7, &[Some("ไผ่"), None]);
        assert_eq!(&p[..10], &[0x17, 7, 0, 0, 0, 0, 1, 0, 0, 0]);
        assert_eq!(p[10], 0b10, "the second value is NULL");
        assert_eq!(p[11], 1);
        assert_eq!(&p[12..16], &[0xFD, 0, 0xFD, 0]);
        assert_eq!(p[16] as usize, "ไผ่".len());
        // A row of (-5i32, NULL, "ab", 2026-11-17).
        let cols = [ColType { ty: 0x03, unsigned: false }, ColType { ty: 0x0F, unsigned: false }, ColType { ty: 0xFD, unsigned: false }, ColType { ty: 0x0A, unsigned: false }];
        let mut r = vec![0x00, 0b0000_1000];
        r.extend_from_slice(&(-5i32).to_le_bytes());
        r.extend_from_slice(&[2, b'a', b'b']);
        r.extend_from_slice(&[4, 0xEA, 0x07, 11, 17]);
        assert_eq!(binary_row(&r, &cols), vec![Some("-5".into()), None, Some("ab".into()), Some("2026-11-17".into())]);
    }
}
