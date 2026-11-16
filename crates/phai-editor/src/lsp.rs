//! A client of the Language Server Protocol, for rust-analyzer: JSON-RPC over the server's stdin
//! and stdout, framed by `Content-Length` headers. A thread reads what the server says and posts
//! each message to a window as [`WM_LSP`]; the window's thread does everything else, so no state
//! is shared between threads but the pipe.
//!
//! Positions in LSP count UTF-16 units within a line; [`to_lsp`] and [`from_lsp`] convert.

use std::io::{BufRead, BufReader, Read, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Mutex;

use serde_json::{json, Value};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging as wm;

/// Posted for each message from the server; `LPARAM` is a `Box<serde_json::Value>` to take back.
pub const WM_LSP: u32 = wm::WM_APP + 0x0C01;

pub struct Lsp {
    child: Child,
    stdin: Mutex<ChildStdin>,
    next: i64,
    pub root: PathBuf,
}

/// Where rust-analyzer is: beside Cargo, on the PATH, or in `D:\Tools\Rust`.
pub fn find_server() -> Option<PathBuf> {
    let mut places: Vec<PathBuf> = Vec::new();
    if let Some(path) = std::env::var_os("PATH") {
        places.extend(std::env::split_paths(&path));
    }
    if let Some(home) = std::env::var_os("CARGO_HOME") {
        places.push(PathBuf::from(home).join("bin"));
    }
    if let Some(user) = std::env::var_os("USERPROFILE") {
        places.push(PathBuf::from(user).join(".cargo").join("bin"));
    }
    places.push(PathBuf::from(r"D:\Tools\Rust\cargo\bin"));
    places.into_iter().map(|d| d.join("rust-analyzer.exe")).find(|p| p.is_file())
}

/// `file:///d%3A/Project/x.rs` for `D:\Project\x.rs` - the drive and anything unusual escaped.
pub fn uri_of(path: &Path) -> String {
    let p = path.to_string_lossy().replace('\\', "/");
    let p = p.trim_start_matches("//?/");
    let mut out = String::from("file:///");
    for (i, c) in p.chars().enumerate() {
        match c {
            ':' if i == 1 => out.push_str("%3A"),
            'A'..='Z' if i == 0 && p.chars().nth(1) == Some(':') => out.push(c.to_ascii_lowercase()),
            'a'..='z' | 'A'..='Z' | '0'..='9' | '/' | '-' | '_' | '.' | '~' => out.push(c),
            c => {
                let mut b = [0u8; 4];
                for byte in c.encode_utf8(&mut b).bytes() {
                    out.push_str(&format!("%{byte:02X}"));
                }
            }
        }
    }
    out
}

/// The path a `file:` URI names.
pub fn path_of(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file:///")?;
    let mut bytes = Vec::new();
    let b = rest.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&rest[i + 1..i + 3], 16) {
                bytes.push(v);
                i += 3;
                continue;
            }
        }
        bytes.push(b[i]);
        i += 1;
    }
    Some(PathBuf::from(String::from_utf8_lossy(&bytes).replace('/', "\\")))
}

/// Whether two paths are the same file, Windows' way: case does not matter.
pub fn same_path(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| p.to_string_lossy().replace('/', "\\").trim_start_matches(r"\\?\").to_ascii_lowercase();
    norm(a) == norm(b)
}

/// LSP's `(line, character)` for byte `column` of `line_text`: characters counted in UTF-16 units.
pub fn to_lsp(line_text: &str, column: usize) -> u32 {
    line_text[..column.min(line_text.len())].encode_utf16().count() as u32
}

/// The byte column in `line_text` of LSP's `character`.
pub fn from_lsp(line_text: &str, character: u32) -> usize {
    let mut units = 0u32;
    for (i, c) in line_text.char_indices() {
        if units >= character {
            return i;
        }
        units += c.len_utf16() as u32;
    }
    line_text.len()
}

fn send(stdin: &Mutex<ChildStdin>, msg: &Value) -> bool {
    let body = msg.to_string();
    let mut w = stdin.lock().unwrap_or_else(|e| e.into_inner());
    write!(w, "Content-Length: {}\r\n\r\n{}", body.len(), body).is_ok() && w.flush().is_ok()
}

impl Lsp {
    /// Starts rust-analyzer for the project at `root`; what it says goes to `window`.
    pub fn start(root: &Path, window: HWND) -> Option<Lsp> {
        let server = find_server()?;
        let mut child = Command::new(server).current_dir(root).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).creation_flags(0x0800_0000).spawn().ok()?;
        let stdin = child.stdin.take()?;
        let stdout = child.stdout.take()?;
        let target = window.0 as isize;
        std::thread::spawn(move || {
            let mut r = BufReader::new(stdout);
            loop {
                let mut len = 0usize;
                loop {
                    let mut header = String::new();
                    if r.read_line(&mut header).unwrap_or(0) == 0 {
                        return;
                    }
                    let h = header.trim();
                    if h.is_empty() {
                        break;
                    }
                    if let Some(v) = h.strip_prefix("Content-Length:") {
                        len = v.trim().parse().unwrap_or(0);
                    }
                }
                let mut body = vec![0u8; len];
                if r.read_exact(&mut body).is_err() {
                    return;
                }
                let Ok(value) = serde_json::from_slice::<Value>(&body) else { continue };
                // `PHAI_LSP_LOG=1`: what the server says, one line each, for finding out why.
                if std::env::var_os("PHAI_LSP_LOG").is_some() {
                    let what = value.get("method").and_then(|m| m.as_str()).map(str::to_string).unwrap_or_else(|| format!("answer {}", value.get("id").cloned().unwrap_or_default()));
                    let extra = if what == "textDocument/publishDiagnostics" { format!(" {} {}", value["params"]["uri"], value["params"]["diagnostics"].as_array().map(|a| a.len()).unwrap_or(0)) } else { String::new() };
                    eprintln!("lsp: {what}{extra}");
                }
                let boxed = Box::into_raw(Box::new(value));
                unsafe {
                    if wm::PostMessageW(Some(HWND(target as *mut _)), WM_LSP, WPARAM(0), LPARAM(boxed as isize)).is_err() {
                        drop(Box::from_raw(boxed));
                        return;
                    }
                }
            }
        });
        let mut lsp = Lsp { child, stdin: Mutex::new(stdin), next: 1, root: root.to_path_buf() };
        lsp.request(
            "initialize",
            json!({
                "processId": std::process::id(),
                "rootUri": uri_of(root),
                "capabilities": {
                    "textDocument": {
                        "hover": { "contentFormat": ["plaintext", "markdown"] },
                        "completion": { "completionItem": { "snippetSupport": false } },
                        "publishDiagnostics": {}
                    },
                    "general": { "positionEncodings": ["utf-16"] }
                },
                "workspaceFolders": [{ "uri": uri_of(root), "name": "project" }],
                // Type mistakes as you type, not only after `cargo check`.
                "initializationOptions": { "diagnostics": { "experimental": { "enable": true } }, "checkOnSave": true }
            }),
        );
        // rust-analyzer takes the notification as soon as it has answered `initialize`.
        lsp.notify("initialized", json!({}));
        Some(lsp)
    }

    /// Sends a request; returns its id, which the answer will carry.
    pub fn request(&mut self, method: &str, params: Value) -> i64 {
        let id = self.next;
        self.next += 1;
        send(&self.stdin, &json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        id
    }

    /// Answers a request the server made of us - with nothing, which is all it needs here.
    pub fn respond(&self, id: &Value) {
        send(&self.stdin, &json!({ "jsonrpc": "2.0", "id": id, "result": Value::Null }));
    }

    pub fn notify(&self, method: &str, params: Value) {
        send(&self.stdin, &json!({ "jsonrpc": "2.0", "method": method, "params": params }));
    }

    /// Opens a file with the server - and says it was saved, so `cargo check` runs and its errors
    /// come at once rather than on the first save.
    pub fn did_open(&self, path: &Path, text: &str) {
        self.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri_of(path), "languageId": "rust", "version": 1, "text": text } }));
        self.did_save(path);
    }

    pub fn did_change(&self, path: &Path, version: i64, text: &str) {
        self.notify("textDocument/didChange", json!({ "textDocument": { "uri": uri_of(path), "version": version }, "contentChanges": [{ "text": text }] }));
    }

    pub fn did_save(&self, path: &Path) {
        self.notify("textDocument/didSave", json!({ "textDocument": { "uri": uri_of(path) } }));
    }

    pub fn did_close(&self, path: &Path) {
        self.notify("textDocument/didClose", json!({ "textDocument": { "uri": uri_of(path) } }));
    }

    pub fn hover(&mut self, path: &Path, line: u32, character: u32) -> i64 {
        self.request("textDocument/hover", json!({ "textDocument": { "uri": uri_of(path) }, "position": { "line": line, "character": character } }))
    }

    pub fn completion(&mut self, path: &Path, line: u32, character: u32) -> i64 {
        self.request("textDocument/completion", json!({ "textDocument": { "uri": uri_of(path) }, "position": { "line": line, "character": character } }))
    }
}

impl Drop for Lsp {
    fn drop(&mut self) {
        let _ = self.request("shutdown", Value::Null);
        self.notify("exit", Value::Null);
        let _ = self.child.kill();
    }
}

/// A diagnostic from `textDocument/publishDiagnostics`: where (LSP lines and characters), how bad,
/// and what it says.
#[derive(Clone, Debug, PartialEq)]
pub struct Diagnostic {
    pub start: (u32, u32),
    pub end: (u32, u32),
    pub error: bool,
    pub message: String,
}

/// The file and diagnostics of a `publishDiagnostics` notification.
pub fn diagnostics_of(msg: &Value) -> Option<(PathBuf, Vec<Diagnostic>)> {
    if msg.get("method")?.as_str()? != "textDocument/publishDiagnostics" {
        return None;
    }
    let p = msg.get("params")?;
    let path = path_of(p.get("uri")?.as_str()?)?;
    let list = p.get("diagnostics")?.as_array()?;
    let pos = |v: &Value| (v["line"].as_u64().unwrap_or(0) as u32, v["character"].as_u64().unwrap_or(0) as u32);
    let out = list
        .iter()
        .map(|d| Diagnostic {
            start: pos(&d["range"]["start"]),
            end: pos(&d["range"]["end"]),
            error: d["severity"].as_u64().unwrap_or(1) == 1,
            message: d["message"].as_str().unwrap_or_default().to_string(),
        })
        .collect();
    Some((path, out))
}

/// The words of a hover answer, as plain text: code fences dropped, the rest kept.
pub fn hover_text(result: &Value) -> Option<String> {
    let c = result.get("contents")?;
    let raw = match c {
        Value::String(s) => s.clone(),
        Value::Object(o) => o.get("value")?.as_str()?.to_string(),
        Value::Array(a) => a.iter().filter_map(|x| x.as_str().map(str::to_string).or_else(|| x.get("value").and_then(|v| v.as_str()).map(str::to_string))).collect::<Vec<_>>().join("\n"),
        _ => return None,
    };
    let text: Vec<&str> = raw.lines().filter(|l| !l.trim_start().starts_with("```")).collect();
    let t = text.join("\n").trim().to_string();
    (!t.is_empty()).then_some(t)
}

/// A completion's label and the text it puts in.
#[derive(Clone, Debug, PartialEq)]
pub struct Completion {
    pub label: String,
    pub insert: String,
    pub detail: String,
}

/// The items of a completion answer, best first, at most `max`.
pub fn completions_of(result: &Value, max: usize) -> Vec<Completion> {
    let items = match result {
        Value::Array(a) => a.clone(),
        Value::Object(o) => o.get("items").and_then(|i| i.as_array()).cloned().unwrap_or_default(),
        _ => vec![],
    };
    let mut v: Vec<(String, Completion)> = items
        .iter()
        .map(|i| {
            let label = i["label"].as_str().unwrap_or_default().trim().to_string();
            let insert = i["textEdit"]["newText"].as_str().or(i["insertText"].as_str()).unwrap_or(&label).to_string();
            // Snippet marks (`$0`, `${1:x}`) out: the plain text only.
            let mut plain = String::new();
            let mut chars = insert.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '$' {
                    if chars.peek() == Some(&'{') {
                        chars.next();
                        while let Some(d) = chars.next() {
                            if d == ':' {
                                for e in chars.by_ref() {
                                    if e == '}' {
                                        break;
                                    }
                                    plain.push(e);
                                }
                                break;
                            }
                            if d == '}' {
                                break;
                            }
                        }
                    } else {
                        while chars.peek().is_some_and(|d| d.is_ascii_digit()) {
                            chars.next();
                        }
                    }
                } else {
                    plain.push(c);
                }
            }
            let sort = i["sortText"].as_str().unwrap_or(&label).to_string();
            (sort, Completion { label, insert: plain, detail: i["detail"].as_str().unwrap_or_default().to_string() })
        })
        .collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v.into_iter().map(|(_, c)| c).take(max).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_and_uris_go_both_ways() {
        let p = Path::new(r"D:\Project.NET\phai studio\src\main.rs");
        let u = uri_of(p);
        assert_eq!(u, "file:///d%3A/Project.NET/phai%20studio/src/main.rs");
        assert!(same_path(&path_of(&u).unwrap(), p));
        assert!(same_path(&path_of("file:///D:/x/ไทย.rs").unwrap(), Path::new(r"d:\x\ไทย.rs")));
    }

    #[test]
    fn positions_count_utf16_units() {
        let line = "let ชื่อ = \"😀\"; x";
        let col = line.find('x').unwrap();
        let ch = to_lsp(line, col);
        assert_eq!(from_lsp(line, ch), col);
        assert_eq!(ch as usize, line[..col].encode_utf16().count());
    }

    #[test]
    fn diagnostics_hover_and_completions_are_read() {
        let msg = json!({"method": "textDocument/publishDiagnostics", "params": {"uri": "file:///d%3A/p/main.rs", "diagnostics": [
            {"range": {"start": {"line": 3, "character": 4}, "end": {"line": 3, "character": 9}}, "severity": 1, "message": "mismatched types"},
            {"range": {"start": {"line": 1, "character": 0}, "end": {"line": 1, "character": 2}}, "severity": 2, "message": "unused"}]}});
        let (path, d) = diagnostics_of(&msg).unwrap();
        assert!(same_path(&path, Path::new(r"D:\p\main.rs")));
        assert_eq!(d.len(), 2);
        assert!(d[0].error && !d[1].error);
        assert_eq!(hover_text(&json!({"contents": {"kind": "markdown", "value": "```rust\nfn go() -> u32\n```\nGoes."}})).as_deref(), Some("fn go() -> u32\nGoes."));
        let c = completions_of(&json!({"items": [
            {"label": "push", "sortText": "b", "insertText": "push(${1:value})$0", "detail": "fn(&mut self, T)"},
            {"label": "pop", "sortText": "a"}]}), 10);
        assert_eq!(c[0].label, "pop");
        assert_eq!(c[1].insert, "push(value)");
    }
}
