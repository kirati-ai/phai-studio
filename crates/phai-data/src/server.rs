//! The tables of a program's bound grids served over HTTP, for its web app (the PWA): a page in a
//! browser cannot talk to a database server itself, so a small program beside the database does -
//! `data-server/` in the project, which Phai Studio writes, calls [`run`].
//!
//! Only the tables the forms bind, with only their bound columns: a request names a binding
//! (`main_window/customers`), never SQL. Each request carries the access key
//! (`Authorization: Bearer ...`, `PHAI_DATA_TOKEN`) - always: "only this computer" protects
//! nothing, as any site open in a browser here can ask `127.0.0.1` too, and a web server passing
//! requests on makes them all come from this computer. Without `PHAI_DATA_TOKEN` the server makes
//! a key and says it. A wrong key waits a second, one at a time, so keys cannot be guessed quickly.
//! Put it behind the web server's HTTPS at the same address as the app (`/phai-data/`),
//! as nginx's `location /phai-data/ { proxy_pass http://127.0.0.1:8791; }`, or give the app its
//! address and the server answers other sites too (CORS).
//!
//! `POST .../phai-data/load {"table": name}` - `{"rows": [[...], ...]}`;
//! `POST .../phai-data/save {"table": name, "loaded": [[...]], "rows": [[tag, [...]], ...]}` -
//! `{"changes": n}`, as [`Binding::save`] does it with the rows the page loaded.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

use crate::{Binding, Connection};

/// A value of JSON - only what the requests need.
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(o) => o.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    pub fn str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn arr(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(a) => Some(a),
            _ => None,
        }
    }
    /// A row: an array of strings (a number or null taken as its text, `NULL`).
    pub fn row(&self) -> Option<Vec<String>> {
        self.arr()?
            .iter()
            .map(|v| match v {
                Json::Str(s) => Some(s.clone()),
                Json::Num(n) => Some(n.to_string()),
                Json::Null => Some("NULL".into()),
                Json::Bool(b) => Some(b.to_string()),
                _ => None,
            })
            .collect()
    }

    pub fn parse(text: &str) -> Result<Json, String> {
        let mut p = Parser { s: text.as_bytes(), i: 0 };
        let v = p.value(0)?;
        p.ws();
        if p.i != p.s.len() {
            return Err("JSON: something after the value".into());
        }
        Ok(v)
    }
}

/// A string as JSON writes it.
pub fn quote(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// Rows as JSON: `[["a","b"],...]`.
pub fn rows_json(rows: &[Vec<String>]) -> String {
    let r: Vec<String> = rows.iter().map(|r| format!("[{}]", r.iter().map(|c| quote(c)).collect::<Vec<_>>().join(","))).collect();
    format!("[{}]", r.join(","))
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while self.i < self.s.len() && matches!(self.s[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }
    fn eat(&mut self, b: u8) -> Result<(), String> {
        self.ws();
        if self.s.get(self.i) == Some(&b) {
            self.i += 1;
            Ok(())
        } else {
            Err(format!("JSON: {} expected at {}", b as char, self.i))
        }
    }
    fn value(&mut self, depth: usize) -> Result<Json, String> {
        if depth > 32 {
            return Err("JSON: too deep".into());
        }
        self.ws();
        match self.s.get(self.i) {
            Some(b'{') => {
                self.i += 1;
                let mut o = Vec::new();
                self.ws();
                if self.s.get(self.i) == Some(&b'}') {
                    self.i += 1;
                    return Ok(Json::Obj(o));
                }
                loop {
                    self.ws();
                    let k = self.string()?;
                    self.eat(b':')?;
                    o.push((k, self.value(depth + 1)?));
                    self.ws();
                    match self.s.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b'}') => {
                            self.i += 1;
                            return Ok(Json::Obj(o));
                        }
                        _ => return Err("JSON: , or } expected".into()),
                    }
                }
            }
            Some(b'[') => {
                self.i += 1;
                let mut a = Vec::new();
                self.ws();
                if self.s.get(self.i) == Some(&b']') {
                    self.i += 1;
                    return Ok(Json::Arr(a));
                }
                loop {
                    a.push(self.value(depth + 1)?);
                    self.ws();
                    match self.s.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b']') => {
                            self.i += 1;
                            return Ok(Json::Arr(a));
                        }
                        _ => return Err("JSON: , or ] expected".into()),
                    }
                }
            }
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(b't') if self.s[self.i..].starts_with(b"true") => {
                self.i += 4;
                Ok(Json::Bool(true))
            }
            Some(b'f') if self.s[self.i..].starts_with(b"false") => {
                self.i += 5;
                Ok(Json::Bool(false))
            }
            Some(b'n') if self.s[self.i..].starts_with(b"null") => {
                self.i += 4;
                Ok(Json::Null)
            }
            Some(c) if c.is_ascii_digit() || *c == b'-' => {
                let start = self.i;
                while self.i < self.s.len() && matches!(self.s[self.i], b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E') {
                    self.i += 1;
                }
                std::str::from_utf8(&self.s[start..self.i]).ok().and_then(|t| t.parse().ok()).map(Json::Num).ok_or_else(|| "JSON: a bad number".into())
            }
            _ => Err(format!("JSON: a value expected at {}", self.i)),
        }
    }
    fn string(&mut self) -> Result<String, String> {
        if self.s.get(self.i) != Some(&b'"') {
            return Err("JSON: a string expected".into());
        }
        self.i += 1;
        let mut out: Vec<u8> = Vec::new();
        loop {
            let Some(&c) = self.s.get(self.i) else { return Err("JSON: a string never ended".into()) };
            self.i += 1;
            match c {
                b'"' => return String::from_utf8(out).map_err(|_| "JSON: not UTF-8".into()),
                b'\\' => {
                    let Some(&e) = self.s.get(self.i) else { return Err("JSON: a bad escape".into()) };
                    self.i += 1;
                    let ch = match e {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{8}',
                        b'f' => '\u{c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => {
                            let mut code = self.hex4()?;
                            // A pair of surrogates: one character.
                            if (0xD800..0xDC00).contains(&code) && self.s[self.i..].starts_with(b"\\u") {
                                self.i += 2;
                                let low = self.hex4()?;
                                code = 0x10000 + ((code - 0xD800) << 10) + (low.wrapping_sub(0xDC00) & 0x3FF);
                            }
                            char::from_u32(code).unwrap_or('\u{FFFD}')
                        }
                        _ => return Err("JSON: a bad escape".into()),
                    };
                    let mut buf = [0u8; 4];
                    out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                }
                c => out.push(c),
            }
        }
    }
    fn hex4(&mut self) -> Result<u32, String> {
        let h = self.s.get(self.i..self.i + 4).and_then(|h| std::str::from_utf8(h).ok()).and_then(|h| u32::from_str_radix(h, 16).ok()).ok_or("JSON: a bad \\u")?;
        self.i += 4;
        Ok(h)
    }
}

/// A binding the server answers for: its name in requests (`main_window/customers`) and its table.
pub struct Served {
    pub name: String,
    pub binding: Binding,
}

/// What the server knows: its tables and its access key (empty: none, on this computer only).
pub struct Server {
    pub tables: Vec<Served>,
    pub token: String,
}

/// One response: its status, its JSON.
pub struct Answer {
    pub status: u16,
    pub body: String,
}

fn error(status: u16, message: &str) -> Answer {
    Answer { status, body: format!("{{\"error\":{}}}", quote(message)) }
}

/// A key no one can guess: `digits` decimal digits (at most 19) from the system's randomness.
pub fn random_key(digits: u32) -> String {
    use std::hash::{BuildHasher, Hasher};
    let mut h = std::collections::hash_map::RandomState::new().build_hasher();
    h.write_u128(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0));
    let digits = digits.clamp(1, 19);
    format!("{:0width$}", h.finish() % 10u64.pow(digits), width = digits as usize)
}

/// Wrong keys wait their turn here.
static WRONG: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Whether two keys are the same, taking as long whatever they hold.
fn same(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut diff = (a.len() ^ b.len()) as u8;
    for i in 0..a.len().max(b.len()) {
        diff |= a.get(i).copied().unwrap_or(0) ^ b.get(i).copied().unwrap_or(1);
    }
    diff == 0
}

impl Server {
    /// The answer to a request: its method, its path, its `Authorization` header, its body.
    pub fn answer(&self, method: &str, path: &str, auth: Option<&str>, body: &str) -> Answer {
        let path = path.split('?').next().unwrap_or("");
        // No key set: nothing is served (see the top of this file).
        if self.token.is_empty() || !auth.and_then(|a| a.strip_prefix("Bearer ")).is_some_and(|t| same(t.trim(), &self.token)) {
            return error(401, "the access key is missing or wrong");
        }
        if method == "GET" && path.trim_end_matches('/').ends_with("/phai-data") {
            let names: Vec<String> = self.tables.iter().map(|t| quote(&t.name)).collect();
            return Answer { status: 200, body: format!("{{\"tables\":[{}]}}", names.join(",")) };
        }
        if method != "POST" {
            return error(405, "POST only");
        }
        let what = if path.ends_with("/phai-data/load") {
            "load"
        } else if path.ends_with("/phai-data/save") {
            "save"
        } else {
            return error(404, "no such request");
        };
        let req = match Json::parse(body) {
            Ok(j) => j,
            Err(e) => return error(400, &e),
        };
        let Some(name) = req.get("table").and_then(Json::str) else { return error(400, "which table?") };
        let Some(t) = self.tables.iter().find(|t| t.name == name) else { return error(404, &format!("{name}: not a table this server serves")) };
        if what == "load" {
            return match t.binding.load() {
                Ok(rows) => Answer { status: 200, body: format!("{{\"rows\":{}}}", rows_json(&rows)) },
                Err(e) => error(500, &e),
            };
        }
        let Some(loaded) = req.get("loaded").and_then(Json::arr).and_then(|a| a.iter().map(Json::row).collect::<Option<Vec<_>>>()) else {
            return error(400, "loaded: the rows the page read");
        };
        let rows = req.get("rows").and_then(Json::arr).and_then(|a| {
            a.iter()
                .map(|r| {
                    let pair = r.arr()?;
                    let tag = match pair.first()? {
                        Json::Num(n) if *n >= 0.0 => *n as usize,
                        _ => return None,
                    };
                    Some((tag, pair.get(1)?.row()?))
                })
                .collect::<Option<Vec<_>>>()
        });
        let Some(rows) = rows else { return error(400, "rows: [tag, [cells]] for each row the grid holds") };
        match t.binding.save_against(&loaded, &rows) {
            Ok(n) => Answer { status: 200, body: format!("{{\"changes\":{n}}}") },
            Err(e) => error(500, &e),
        }
    }

    /// Serves on `listener` until the program ends: each connection on a thread of its own.
    pub fn serve(self, listener: TcpListener) {
        let me = Arc::new(self);
        for stream in listener.incoming().flatten() {
            let me = me.clone();
            std::thread::spawn(move || {
                let _ = me.connection(stream);
            });
        }
    }

    fn connection(&self, mut stream: TcpStream) -> std::io::Result<()> {
        stream.set_read_timeout(Some(std::time::Duration::from_secs(30)))?;
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut first = String::new();
        reader.read_line(&mut first)?;
        let mut parts = first.split_whitespace();
        let (method, path) = (parts.next().unwrap_or("").to_string(), parts.next().unwrap_or("").to_string());
        let (mut length, mut auth, mut origin) = (0usize, None, None);
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line)? == 0 || line.trim().is_empty() {
                break;
            }
            if let Some((k, v)) = line.split_once(':') {
                match k.trim().to_ascii_lowercase().as_str() {
                    "content-length" => length = v.trim().parse().unwrap_or(0),
                    "authorization" => auth = Some(v.trim().to_string()),
                    "origin" => origin = Some(v.trim().to_string()),
                    _ => {}
                }
            }
        }
        const MAX: usize = 64 << 20;
        let answer = if length > MAX {
            error(413, "too much at once")
        } else if method == "OPTIONS" {
            Answer { status: 204, body: String::new() }
        } else {
            let mut body = vec![0u8; length];
            reader.read_exact(&mut body)?;
            self.answer(&method, &path, auth.as_deref(), &String::from_utf8_lossy(&body))
        };
        if answer.status == 401 {
            // A second for each wrong key, one after another: a million keys take eleven days.
            let _turn = WRONG.lock().unwrap_or_else(|e| e.into_inner());
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        let reason = match answer.status {
            200 => "OK",
            204 => "No Content",
            400 => "Bad Request",
            401 => "Unauthorized",
            404 => "Not Found",
            405 => "Method Not Allowed",
            413 => "Payload Too Large",
            _ => "Internal Server Error",
        };
        // Another site's page may ask too (the app served elsewhere); the key still decides.
        let cors = format!("Access-Control-Allow-Origin: {}\r\nAccess-Control-Allow-Headers: Authorization, Content-Type\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nVary: Origin\r\n", origin.as_deref().unwrap_or("*"));
        let head = format!("HTTP/1.1 {} {reason}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\n{cors}Connection: close\r\n\r\n", answer.status, answer.body.len());
        stream.write_all(head.as_bytes())?;
        stream.write_all(answer.body.as_bytes())?;
        stream.flush()
    }
}

/// A bound grid's table, as the data server's `main` names it: (name, database, table, columns,
/// keys).
pub type Table<'a> = (&'a str, &'a str, &'a str, &'a [&'a str], &'a [&'a str]);

/// The data server's `main`: these tables of `conn` served at `PHAI_DATA_ADDR` (127.0.0.1:8791
/// when unset), the access key `PHAI_DATA_TOKEN` - one made and said when it is not set.
pub fn run(conn: Connection, tables: &[Table]) -> Result<(), String> {
    let mut token = std::env::var("PHAI_DATA_TOKEN").unwrap_or_default().trim().to_string();
    let addr = std::env::var("PHAI_DATA_ADDR").unwrap_or_else(|_| "127.0.0.1:8791".into());
    let listener = TcpListener::bind(&addr).map_err(|e| format!("{addr}: {e}"))?;
    let served = tables.iter().map(|(name, db, table, cols, keys)| Served { name: name.to_string(), binding: Binding::new(conn.clone(), db, table, cols, keys) }).collect();
    println!("Phai data server: http://{addr}/phai-data/ - {} table(s)", tables.len());
    if token.is_empty() {
        token = random_key(12);
        println!("The access key the app asks for: {token} (PHAI_DATA_TOKEN chooses one that stays)");
    }
    Server { tables: served, token }.serve(listener);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Kind;

    #[test]
    fn json_reads_and_writes_what_the_pages_send() {
        let j = Json::parse(r#"{"table":"main_window/customers","rows":[[1,["1","สมชาย \"ใจดี\"",null]],[0,["","ไ😀"]]]}"#).unwrap();
        assert_eq!(j.get("table").and_then(Json::str), Some("main_window/customers"));
        let rows = j.get("rows").and_then(Json::arr).unwrap();
        assert_eq!(rows[0].arr().unwrap()[1].row().unwrap(), vec!["1", "สมชาย \"ใจดี\"", "NULL"]);
        assert_eq!(rows[1].arr().unwrap()[1].row().unwrap(), vec!["", "ไ😀"]);
        assert_eq!(rows_json(&[vec!["a\"b".into(), "ไทย\n".into()]]), "[[\"a\\\"b\",\"ไทย\\n\"]]");
        assert!(Json::parse("[1,]").is_err() && Json::parse("{\"a\":1} x").is_err());
    }

    #[test]
    fn the_server_loads_and_saves_a_bound_table_with_its_key() {
        let path = std::env::temp_dir().join(format!("phai-serve-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let c = Connection { host: path.to_string_lossy().to_string(), kind: Kind::Sqlite, ..Default::default() };
        c.query(None, "CREATE TABLE customers (id INTEGER PRIMARY KEY, name TEXT, secret TEXT); INSERT INTO customers (name, secret) VALUES ('A', 'x'), ('B', 'y');").unwrap();
        let s = Server { tables: vec![Served { name: "w/customers".into(), binding: Binding::new(c.clone(), "main", "customers", &["id", "name"], &["id"]) }], token: "k3y".into() };
        assert_eq!(s.answer("POST", "/phai-data/load", None, "{\"table\":\"w/customers\"}").status, 401);
        assert_eq!(s.answer("POST", "/phai-data/load", Some("Bearer nope"), "{\"table\":\"w/customers\"}").status, 401);
        let a = s.answer("POST", "/app/phai-data/load", Some("Bearer k3y"), "{\"table\":\"w/customers\"}");
        assert_eq!((a.status, a.body.as_str()), (200, "{\"rows\":[[\"1\",\"A\"],[\"2\",\"B\"]]}"), "only the bound columns");
        assert_eq!(s.answer("POST", "/phai-data/load", Some("Bearer k3y"), "{\"table\":\"customers\"}").status, 404, "a name the forms gave, not a table");
        let save = "{\"table\":\"w/customers\",\"loaded\":[[\"1\",\"A\"],[\"2\",\"B\"]],\"rows\":[[1,[\"1\",\"เอ\"]],[0,[\"\",\"C\"]]]}";
        let a = s.answer("POST", "/phai-data/save", Some("Bearer k3y"), save);
        assert_eq!((a.status, a.body.as_str()), (200, "{\"changes\":3}"), "one changed, one added, one deleted");
        let names: Vec<String> = c.query(None, "SELECT name FROM customers ORDER BY id").unwrap().rows.into_iter().map(|r| r[0].clone()).collect();
        assert_eq!(names, ["เอ", "C"]);
        let open = Server { tables: s.tables, token: String::new() };
        assert_eq!(open.answer("POST", "/phai-data/load", None, "{\"table\":\"w/customers\"}").status, 401, "no key set: nothing served");
        assert_eq!(open.answer("POST", "/phai-data/load", Some("Bearer "), "{\"table\":\"w/customers\"}").status, 401);
        let k = random_key(6);
        assert!(k.len() == 6 && k.bytes().all(|b| b.is_ascii_digit()), "{k}");
        let _ = std::fs::remove_file(&path);
    }
}
