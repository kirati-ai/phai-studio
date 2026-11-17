//! PostgreSQL's frontend/backend protocol (version 3), as much as a data explorer needs: TLS when
//! asked, the password proven by SCRAM-SHA-256, MD5 or in clear (over TLS), simple queries with
//! several results, and the extended protocol for a statement with `$1` parameters. Hashing,
//! HMAC, PBKDF2 and TLS come from crates and from Windows; none is written here.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use base64::Engine as _;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

use crate::wire::Result1;

pub struct Login<'a> {
    pub host: &'a str,
    pub port: u16,
    pub user: &'a str,
    pub password: &'a str,
    pub database: &'a str,
    pub ssl: bool,
    pub trust_cert: bool,
}

enum Stream {
    Plain(TcpStream),
    Tls(Box<native_tls::TlsStream<TcpStream>>),
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

pub struct Conn {
    s: Stream,
    pub server_version: String,
}

fn io(e: std::io::Error) -> String {
    e.to_string()
}

/// An ErrorResponse's words: `28P01: password authentication failed for user ...`.
fn error_text(body: &[u8]) -> String {
    let mut code = String::new();
    let mut msg = String::new();
    for f in body.split(|b| *b == 0) {
        if let Some((&t, v)) = f.split_first() {
            let v = String::from_utf8_lossy(v).to_string();
            match t {
                b'C' => code = v,
                b'M' => msg = v,
                _ => {}
            }
        }
    }
    if code.is_empty() { msg } else { format!("{code}: {msg}") }
}

fn be32(b: &[u8]) -> i32 {
    i32::from_be_bytes([b.first().copied().unwrap_or(0), b.get(1).copied().unwrap_or(0), b.get(2).copied().unwrap_or(0), b.get(3).copied().unwrap_or(0)])
}

fn cstr(b: &[u8]) -> (String, &[u8]) {
    let n = b.iter().position(|c| *c == 0).unwrap_or(b.len());
    (String::from_utf8_lossy(&b[..n]).to_string(), &b[(n + 1).min(b.len())..])
}

/// `md5` + md5(md5(password + user) + salt), as hex - PostgreSQL's MD5 proof.
pub fn md5_password(user: &str, password: &str, salt: &[u8]) -> String {
    let inner = format!("{:x}", md5::Md5::digest(format!("{password}{user}").as_bytes()));
    let mut h = md5::Md5::new();
    h.update(inner.as_bytes());
    h.update(salt);
    format!("md5{:x}", h.finalize())
}

/// SCRAM-SHA-256 (RFC 5802 / 7677): the client's proof for `server_first`, and the server
/// signature it must answer with. `client_first_bare` is `n=,r=<nonce>`.
pub fn scram_proof(password: &str, client_first_bare: &str, server_first: &str, client_nonce: &str) -> Result<(String, Vec<u8>), String> {
    let mut nonce = "";
    let mut salt = Vec::new();
    let mut iters = 0u32;
    for part in server_first.split(',') {
        if let Some(v) = part.strip_prefix("r=") {
            nonce = v;
        } else if let Some(v) = part.strip_prefix("s=") {
            salt = base64::engine::general_purpose::STANDARD.decode(v).map_err(|e| e.to_string())?;
        } else if let Some(v) = part.strip_prefix("i=") {
            iters = v.parse().map_err(|_| "bad iteration count".to_string())?;
        }
    }
    if !nonce.starts_with(client_nonce) || iters == 0 {
        return Err("the server's SCRAM answer does not fit".into());
    }
    let mut salted = [0u8; 32];
    pbkdf2::pbkdf2::<Hmac<Sha256>>(password.as_bytes(), &salt, iters, &mut salted).map_err(|e| e.to_string())?;
    let hmac = |key: &[u8], data: &[u8]| -> Vec<u8> {
        let mut m = <Hmac<Sha256> as Mac>::new_from_slice(key).expect("any key length");
        m.update(data);
        m.finalize().into_bytes().to_vec()
    };
    let client_key = hmac(&salted, b"Client Key");
    let stored = Sha256::digest(&client_key);
    let without_proof = format!("c=biws,r={nonce}");
    let auth = format!("{client_first_bare},{server_first},{without_proof}");
    let sig = hmac(&stored, auth.as_bytes());
    let proof: Vec<u8> = client_key.iter().zip(sig.iter()).map(|(a, b)| a ^ b).collect();
    let server_key = hmac(&salted, b"Server Key");
    let server_sig = hmac(&server_key, auth.as_bytes());
    Ok((format!("{without_proof},p={}", base64::engine::general_purpose::STANDARD.encode(proof)), server_sig))
}

impl Conn {
    fn send(&mut self, tag: Option<u8>, body: &[u8]) -> Result<(), String> {
        let mut m = Vec::with_capacity(body.len() + 5);
        if let Some(t) = tag {
            m.push(t);
        }
        m.extend_from_slice(&((body.len() + 4) as i32).to_be_bytes());
        m.extend_from_slice(body);
        self.s.write_all(&m).map_err(io)?;
        self.s.flush().map_err(io)
    }

    fn recv(&mut self) -> Result<(u8, Vec<u8>), String> {
        let mut head = [0u8; 5];
        self.s.read_exact(&mut head).map_err(io)?;
        let len = be32(&head[1..]) as usize;
        let mut body = vec![0u8; len.saturating_sub(4)];
        self.s.read_exact(&mut body).map_err(io)?;
        Ok((head[0], body))
    }

    pub fn connect(l: &Login) -> Result<Conn, String> {
        let addr = (l.host, l.port).to_socket_addrs().map_err(io)?.next().ok_or_else(|| format!("{} not found", l.host))?;
        let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(6)).map_err(io)?;
        tcp.set_read_timeout(Some(Duration::from_secs(60))).map_err(io)?;
        let _ = tcp.set_nodelay(true);
        let mut c = Conn { s: Stream::Plain(tcp), server_version: String::new() };
        if l.ssl {
            // SSLRequest: 'S', and TLS from here.
            c.send(None, &80877103i32.to_be_bytes())?;
            let mut a = [0u8; 1];
            c.s.read_exact(&mut a).map_err(io)?;
            if a[0] != b'S' {
                return Err("the server does not offer TLS".into());
            }
            let connector = native_tls::TlsConnector::builder().danger_accept_invalid_certs(l.trust_cert).danger_accept_invalid_hostnames(l.trust_cert).build().map_err(|e| e.to_string())?;
            let Stream::Plain(tcp) = std::mem::replace(&mut c.s, Stream::Gone) else { return Err("TLS twice".into()) };
            c.s = Stream::Tls(Box::new(connector.connect(l.host, tcp).map_err(|e| e.to_string())?));
        }
        let mut start = Vec::new();
        start.extend_from_slice(&196608i32.to_be_bytes());
        let db = if l.database.is_empty() { "postgres" } else { l.database };
        for (k, v) in [("user", l.user), ("database", db), ("client_encoding", "UTF8"), ("application_name", "Phai Studio")] {
            start.extend_from_slice(k.as_bytes());
            start.push(0);
            start.extend_from_slice(v.as_bytes());
            start.push(0);
        }
        start.push(0);
        c.send(None, &start)?;
        let tls = l.ssl;
        let mut scram: Option<(String, String, Vec<u8>)> = None;
        loop {
            let (t, b) = c.recv()?;
            match t {
                b'R' => match be32(&b) {
                    0 => {}
                    3 => {
                        if !tls {
                            return Err("the server wants the password in clear without TLS: tick Encrypt the connection".into());
                        }
                        c.send(Some(b'p'), &[l.password.as_bytes(), &[0]].concat())?;
                    }
                    5 => {
                        let p = md5_password(l.user, l.password, &b[4..8.min(b.len())]);
                        c.send(Some(b'p'), &[p.as_bytes(), &[0]].concat())?;
                    }
                    10 => {
                        let mechs: Vec<String> = b[4..].split(|x| *x == 0).filter(|m| !m.is_empty()).map(|m| String::from_utf8_lossy(m).to_string()).collect();
                        if !mechs.iter().any(|m| m == "SCRAM-SHA-256") {
                            return Err(format!("no way to log in that Phai knows: {}", mechs.join(", ")));
                        }
                        let nonce: String = {
                            use rand::RngCore;
                            let mut raw = [0u8; 18];
                            rand::thread_rng().fill_bytes(&mut raw);
                            base64::engine::general_purpose::STANDARD.encode(raw)
                        };
                        let bare = format!("n=,r={nonce}");
                        let first = format!("n,,{bare}");
                        let mut m = Vec::new();
                        m.extend_from_slice(b"SCRAM-SHA-256\0");
                        m.extend_from_slice(&(first.len() as i32).to_be_bytes());
                        m.extend_from_slice(first.as_bytes());
                        c.send(Some(b'p'), &m)?;
                        scram = Some((bare, nonce, vec![]));
                    }
                    11 => {
                        let Some((bare, nonce, _)) = scram.take() else { return Err("SCRAM out of turn".into()) };
                        let server_first = String::from_utf8_lossy(&b[4..]).to_string();
                        let (fin, sig) = scram_proof(l.password, &bare, &server_first, &nonce)?;
                        c.send(Some(b'p'), fin.as_bytes())?;
                        scram = Some((bare, nonce, sig));
                    }
                    12 => {
                        // The server proves it knew the password too.
                        let fin = String::from_utf8_lossy(&b[4..]).to_string();
                        let want = scram.as_ref().map(|s| base64::engine::general_purpose::STANDARD.encode(&s.2)).unwrap_or_default();
                        if fin.strip_prefix("v=").map(str::trim) != Some(want.as_str()) {
                            return Err("the server's SCRAM signature is wrong".into());
                        }
                    }
                    n => return Err(format!("a way to log in that Phai does not know ({n})")),
                },
                b'S' => {
                    let (k, rest) = cstr(&b);
                    if k == "server_version" {
                        c.server_version = cstr(rest).0;
                    }
                }
                b'E' => return Err(error_text(&b)),
                b'Z' => return Ok(c),
                _ => {}
            }
        }
    }

    fn results(&mut self, max: usize) -> Result<Vec<(Result1, bool)>, String> {
        let mut out = Vec::new();
        let mut cur: Option<(Result1, bool)> = None;
        let mut failed: Option<String> = None;
        loop {
            let (t, b) = self.recv()?;
            match t {
                b'T' => {
                    let n = u16::from_be_bytes([b[0], b[1]]) as usize;
                    let mut rest = &b[2..];
                    let mut cols = Vec::with_capacity(n);
                    for _ in 0..n {
                        let (name, r) = cstr(rest);
                        cols.push(name);
                        rest = &r[18.min(r.len())..];
                    }
                    cur = Some((Result1 { columns: cols, ..Default::default() }, false));
                }
                b'D' => {
                    if let Some((r, cut)) = cur.as_mut() {
                        if r.rows.len() >= max {
                            *cut = true;
                            continue;
                        }
                        let n = u16::from_be_bytes([b[0], b[1]]) as usize;
                        let mut at = 2;
                        let mut row = Vec::with_capacity(n);
                        for _ in 0..n {
                            let len = be32(&b[at..]);
                            at += 4;
                            if len < 0 {
                                row.push(None);
                            } else {
                                let end = (at + len as usize).min(b.len());
                                row.push(Some(String::from_utf8_lossy(&b[at..end]).to_string()));
                                at = end;
                            }
                        }
                        r.rows.push(row);
                    }
                }
                b'C' => {
                    let tag = cstr(&b).0;
                    match cur.take() {
                        Some(r) => out.push(r),
                        None => {
                            let affected = tag.rsplit(' ').next().and_then(|n| n.parse().ok()).unwrap_or(0);
                            out.push((Result1 { affected, ..Default::default() }, false));
                        }
                    }
                }
                b'E' => failed = Some(error_text(&b)),
                b'Z' => {
                    return match failed {
                        Some(e) => Err(e),
                        None => Ok(out),
                    }
                }
                _ => {}
            }
        }
    }

    /// Runs `sql` (several statements allowed): every result, in order.
    pub fn query(&mut self, sql: &str, max: usize) -> Result<Vec<(Result1, bool)>, String> {
        self.send(Some(b'Q'), &[sql.as_bytes(), &[0]].concat())?;
        self.results(max)
    }

    /// One statement with `$1`, `$2`... and `params` in their places, sent apart from the SQL.
    pub fn execute(&mut self, sql: &str, params: &[Option<&str>], max: usize) -> Result<(Result1, bool), String> {
        // Parse, Bind (all text), Describe the portal, Execute, Sync.
        let mut p = vec![0];
        p.extend_from_slice(sql.as_bytes());
        p.push(0);
        p.extend_from_slice(&0i16.to_be_bytes());
        self.send(Some(b'P'), &p)?;
        let mut b = vec![0, 0];
        b.extend_from_slice(&0i16.to_be_bytes());
        b.extend_from_slice(&(params.len() as i16).to_be_bytes());
        for v in params {
            match v {
                Some(s) => {
                    b.extend_from_slice(&(s.len() as i32).to_be_bytes());
                    b.extend_from_slice(s.as_bytes());
                }
                None => b.extend_from_slice(&(-1i32).to_be_bytes()),
            }
        }
        b.extend_from_slice(&0i16.to_be_bytes());
        self.send(Some(b'B'), &b)?;
        self.send(Some(b'D'), b"P\0")?;
        self.send(Some(b'E'), &[0, 0, 0, 0, 0])?;
        self.send(Some(b'S'), &[])?;
        let mut all = self.results(max)?;
        Ok(all.pop().unwrap_or_default())
    }
}

impl Drop for Conn {
    fn drop(&mut self) {
        let _ = self.send(Some(b'X'), &[]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 7677's example exchange: user `user`, password `pencil`.
    #[test]
    fn scram_matches_the_rfc() {
        let bare = "n=user,r=rOprNGfwEbeRWgbNEkqO";
        let server_first = "r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096";
        let (fin, sig) = scram_proof("pencil", bare, server_first, "rOprNGfwEbeRWgbNEkqO").unwrap();
        assert_eq!(fin, "c=biws,r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,p=dHzbZapWIk4jUhN+Ute9ytag9zjfMHgsqmmiz7AndVQ=");
        assert_eq!(base64::engine::general_purpose::STANDARD.encode(sig), "6rriTRBi23WpRR/wtup+mMhUZUn/dB5nLTJRsjl95G4=");
        assert_eq!(md5_password("u", "p", &[1, 2, 3, 4]).len(), 35);
        assert_eq!(error_text(b"SFATAL\0C28P01\0Mpassword authentication failed\0\0"), "28P01: password authentication failed");
    }
}
