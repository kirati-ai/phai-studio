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
                        "documentSymbol": { "hierarchicalDocumentSymbolSupport": true },
                        "codeLens": {},
                        "completion": { "completionItem": { "snippetSupport": false } },
                        "publishDiagnostics": {},
                        "definition": { "linkSupport": true },
                        "references": {},
                        "rename": { "prepareSupport": true },
                        "codeAction": { "codeActionLiteralSupport": { "codeActionKind": { "valueSet": ["", "quickfix", "refactor", "refactor.extract", "refactor.inline", "refactor.rewrite", "source"] } } },
                        "signatureHelp": { "signatureInformation": { "parameterInformation": { "labelOffsetSupport": true } } }
                    },
                    "workspace": { "workspaceEdit": { "documentChanges": true } },
                    "general": { "positionEncodings": ["utf-16"] }
                },
                "workspaceFolders": [{ "uri": uri_of(root), "name": "project" }],
                // Type mistakes as you type, not only after `cargo check`.
                "initializationOptions": { "diagnostics": { "experimental": { "enable": true } }, "checkOnSave": true, "workspace": { "symbol": { "search": { "kind": "all_symbols" } } } }
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

impl Lsp {
    fn at(&mut self, method: &str, path: &Path, line: u32, character: u32, extra: Value) -> i64 {
        let mut params = json!({ "textDocument": { "uri": uri_of(path) }, "position": { "line": line, "character": character } });
        if let (Value::Object(p), Value::Object(e)) = (&mut params, extra) {
            p.extend(e);
        }
        self.request(method, params)
    }

    /// Where the thing at a place is defined (F12).
    pub fn definition(&mut self, path: &Path, line: u32, character: u32) -> i64 {
        self.at("textDocument/definition", path, line, character, json!({}))
    }

    /// What the call the caret is in takes, and which of it the caret is at (Parameter Info).
    pub fn signature_help(&mut self, path: &Path, line: u32, character: u32) -> i64 {
        self.at("textDocument/signatureHelp", path, line, character, json!({}))
    }

    /// The fixes and refactorings offered from `start` to `end` (Quick Actions, Ctrl+.).
    pub fn code_actions(&mut self, path: &Path, start: (u32, u32), end: (u32, u32)) -> i64 {
        let params = json!({
            "textDocument": { "uri": uri_of(path) },
            "range": { "start": { "line": start.0, "character": start.1 }, "end": { "line": end.0, "character": end.1 } },
            "context": { "diagnostics": [] }
        });
        self.request("textDocument/codeAction", params)
    }

    /// The function at a place, as Call Hierarchy starts from it.
    pub fn prepare_calls(&mut self, path: &Path, line: u32, character: u32) -> i64 {
        self.at("textDocument/prepareCallHierarchy", path, line, character, json!({}))
    }

    /// Who calls the function `item` (from a prepareCallHierarchy answer).
    pub fn incoming_calls(&mut self, item: Value) -> i64 {
        self.request("callHierarchy/incomingCalls", json!({ "item": item }))
    }

    /// A file's symbols, nested as they are written (Document Outline for code).
    pub fn document_symbols(&mut self, path: &Path) -> i64 {
        self.request("textDocument/documentSymbol", json!({ "textDocument": { "uri": uri_of(path) } }))
    }

    /// The solution's symbols whose names match `query` (Go to All).
    pub fn workspace_symbols(&mut self, query: &str) -> i64 {
        self.request("workspace/symbol", json!({ "query": query }))
    }

    /// Every place the thing at a place is used, its definition too (Shift+F12).
    pub fn references(&mut self, path: &Path, line: u32, character: u32) -> i64 {
        self.at("textDocument/references", path, line, character, json!({ "context": { "includeDeclaration": true } }))
    }

    /// Whether the thing at a place can be renamed, and the name it has now.
    pub fn prepare_rename(&mut self, path: &Path, line: u32, character: u32) -> i64 {
        self.at("textDocument/prepareRename", path, line, character, json!({}))
    }

    /// Every change that gives the thing at a place a new name, in every file.
    pub fn rename(&mut self, path: &Path, line: u32, character: u32, new_name: &str) -> i64 {
        self.at("textDocument/rename", path, line, character, json!({ "newName": new_name }))
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

/// Parameter Info's words: the signature, the parameter the caret is at marked with `«»`, and
/// under it what that parameter or the function is, when the server says.
pub fn signature_text(result: &Value) -> Option<String> {
    let sigs = result.get("signatures")?.as_array()?;
    let k = result.get("activeSignature").and_then(Value::as_u64).unwrap_or(0) as usize;
    let sig = sigs.get(k).or(sigs.first())?;
    let label = sig.get("label")?.as_str()?.to_string();
    let active = sig.get("activeParameter").or(result.get("activeParameter")).and_then(Value::as_u64).map(|n| n as usize);
    let params = sig.get("parameters").and_then(Value::as_array).cloned().unwrap_or_default();
    let mut shown = label.clone();
    if let Some(p) = active.and_then(|a| params.get(a)) {
        // A parameter's label: offsets into the signature (UTF-16), or its words.
        let span = match p.get("label") {
            Some(Value::Array(a)) if a.len() == 2 => {
                let u: Vec<u16> = label.encode_utf16().collect();
                let (a, b) = (a[0].as_u64().unwrap_or(0) as usize, a[1].as_u64().unwrap_or(0) as usize);
                (b <= u.len() && a <= b).then(|| (String::from_utf16_lossy(&u[..a]).len(), String::from_utf16_lossy(&u[..b]).len()))
            }
            Some(Value::String(s)) => label.find(s.as_str()).map(|i| (i, i + s.len())),
            _ => None,
        };
        if let Some((a, b)) = span {
            shown = format!("{}«{}»{}", &label[..a], &label[a..b], &label[b..]);
        }
    }
    let doc = |v: Option<&Value>| match v {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Object(o)) => o.get("value").and_then(Value::as_str).map(str::to_string),
        _ => None,
    };
    let more = active.and_then(|a| params.get(a)).and_then(|p| doc(p.get("documentation"))).or_else(|| doc(sig.get("documentation")));
    if let Some(m) = more.map(|m| m.lines().filter(|l| !l.trim_start().starts_with("```")).take(6).collect::<Vec<_>>().join("\n")).filter(|m| !m.trim().is_empty()) {
        shown.push('\n');
        shown.push_str(m.trim());
    }
    Some(shown)
}

/// A Quick Action: its title and the changes it makes (a workspace edit).
#[derive(Clone, Debug)]
pub struct Action {
    pub title: String,
    pub edit: Value,
}

/// The actions of a codeAction answer that carry their changes, fixes first.
pub fn actions_of(result: &Value) -> Vec<Action> {
    let mut v: Vec<(bool, Action)> = result
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| {
                    let edit = x.get("edit")?.clone();
                    let fix = x.get("kind").and_then(Value::as_str).is_some_and(|k| k.starts_with("quickfix"));
                    Some((!fix, Action { title: x.get("title")?.as_str()?.to_string(), edit }))
                })
                .collect()
        })
        .unwrap_or_default();
    v.sort_by_key(|(later, _)| *later);
    v.into_iter().map(|(_, a)| a).collect()
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

/// A place in a file: LSP lines and characters from the start to the end of it.
#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    pub path: PathBuf,
    pub start: (u32, u32),
    pub end: (u32, u32),
}

fn position(v: &Value) -> (u32, u32) {
    (v["line"].as_u64().unwrap_or(0) as u32, v["character"].as_u64().unwrap_or(0) as u32)
}

/// The places of a definition answer: one `Location`, a list of them, or `LocationLink`s.
pub fn locations_of(result: &Value) -> Vec<Location> {
    let one = |v: &Value| -> Option<Location> {
        let uri = v.get("targetUri").or(v.get("uri"))?.as_str()?;
        let range = v.get("targetSelectionRange").or(v.get("range"))?;
        Some(Location { path: path_of(uri)?, start: position(&range["start"]), end: position(&range["end"]) })
    };
    match result {
        Value::Array(a) => a.iter().filter_map(one).collect(),
        Value::Object(_) => one(result).into_iter().collect(),
        _ => vec![],
    }
}

/// The places that call a function, from an incomingCalls answer: each call's place, and the
/// name of the function it is in.
pub fn calls_of(result: &Value) -> Vec<(Location, String)> {
    let mut out = Vec::new();
    for c in result.as_array().into_iter().flatten() {
        let from = &c["from"];
        let Some(path) = from["uri"].as_str().and_then(path_of) else { continue };
        let name = from["name"].as_str().unwrap_or("").to_string();
        for r in c["fromRanges"].as_array().into_iter().flatten() {
            out.push((Location { path: path.clone(), start: position(&r["start"]), end: position(&r["end"]) }, name.clone()));
        }
    }
    out
}

/// A symbol of a file as written: its name, kind in words, where its name is, what is in it.
#[derive(Clone, Debug, PartialEq)]
pub struct DocSymbol {
    pub name: String,
    pub kind: &'static str,
    pub start: (u32, u32),
    pub children: Vec<DocSymbol>,
}

/// The LSP's number for a kind of symbol, in words.
pub fn kind_name(k: u64) -> &'static str {
    match k {
        2 => "Module",
        5 => "Class",
        6 | 9 => "Method",
        8 => "Field",
        10 => "Enum",
        11 => "Trait",
        12 => "Function",
        13 => "Variable",
        14 => "Constant",
        19 => "Impl",
        22 => "Enum member",
        23 => "Struct",
        26 => "Type",
        _ => "Symbol",
    }
}

/// A documentSymbol answer, nested; local variables left out, as an outline has no room for them.
pub fn doc_symbols_of(result: &Value) -> Vec<DocSymbol> {
    fn one(v: &Value) -> Option<DocSymbol> {
        let kind = v.get("kind").and_then(Value::as_u64).unwrap_or(0);
        if kind == 13 {
            return None;
        }
        let range = v.get("selectionRange").or(v.get("range")).or(v.get("location").and_then(|l| l.get("range")))?;
        Some(DocSymbol {
            name: v.get("name")?.as_str()?.to_string(),
            kind: kind_name(kind),
            start: position(&range["start"]),
            children: v.get("children").and_then(Value::as_array).map(|c| c.iter().filter_map(one).collect()).unwrap_or_default(),
        })
    }
    result.as_array().map(|a| a.iter().filter_map(one).collect()).unwrap_or_default()
}

/// A symbol of a `workspace/symbol` answer: its name, its kind in words, where it is.
#[derive(Clone, Debug, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub kind: &'static str,
    pub path: PathBuf,
    pub start: (u32, u32),
}

/// The symbols of a `workspace/symbol` answer - the standard library's and fetched crates' left
/// to the caller to drop.
pub fn symbols_of(result: &Value) -> Vec<Symbol> {
    let kind = |k: u64| match k {
        2 => "Module",
        5 => "Class",
        6 | 9 => "Method",
        8 => "Field",
        10 => "Enum",
        11 => "Trait",
        12 => "Function",
        13 => "Variable",
        14 => "Constant",
        22 => "Enum member",
        23 => "Struct",
        26 => "Type",
        _ => "Symbol",
    };
    result
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|s| {
                    let loc = s.get("location")?;
                    let path = path_of(loc.get("uri")?.as_str()?)?;
                    let start = loc.get("range").map(|r| position(&r["start"])).unwrap_or((0, 0));
                    Some(Symbol { name: s.get("name")?.as_str()?.to_string(), kind: kind(s.get("kind").and_then(Value::as_u64).unwrap_or(0)), path, start })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// One change in a file: the LSP range it replaces and what goes there.
#[derive(Clone, Debug, PartialEq)]
pub struct TextEdit {
    pub start: (u32, u32),
    pub end: (u32, u32),
    pub text: String,
}

/// The changes of a `WorkspaceEdit`, by file - from `documentChanges` or `changes`. A file
/// created, renamed or deleted is not a text change and is left out.
pub fn workspace_edit_of(result: &Value) -> Vec<(PathBuf, Vec<TextEdit>)> {
    let edits = |list: &Value| -> Vec<TextEdit> {
        list.as_array()
            .map(|a| a.iter().map(|e| TextEdit { start: position(&e["range"]["start"]), end: position(&e["range"]["end"]), text: e["newText"].as_str().unwrap_or_default().to_string() }).collect())
            .unwrap_or_default()
    };
    let mut out: Vec<(PathBuf, Vec<TextEdit>)> = Vec::new();
    let mut add = |uri: &str, list: Vec<TextEdit>| {
        let Some(path) = path_of(uri) else { return };
        match out.iter_mut().find(|(p, _)| same_path(p, &path)) {
            Some((_, l)) => l.extend(list),
            None => out.push((path, list)),
        }
    };
    if let Some(changes) = result.get("documentChanges").and_then(|d| d.as_array()) {
        for c in changes {
            if let Some(uri) = c["textDocument"]["uri"].as_str() {
                add(uri, edits(&c["edits"]));
            }
        }
    } else if let Some(map) = result.get("changes").and_then(|c| c.as_object()) {
        for (uri, list) in map {
            add(uri, edits(list));
        }
    }
    out
}

/// The name a `prepareRename` answer offers: its placeholder, or `None` when it cannot be renamed.
/// `text` is the file, for an answer that gives only a range.
pub fn rename_placeholder(result: &Value, text: &str) -> Option<String> {
    if let Some(p) = result.get("placeholder").and_then(|p| p.as_str()) {
        return Some(p.to_string());
    }
    let range = result.get("range").unwrap_or(result);
    let (start, end) = (range.get("start")?, range.get("end")?);
    let (sl, sc) = position(start);
    let (el, ec) = position(end);
    if sl != el {
        return None;
    }
    let line = text.lines().nth(sl as usize)?;
    Some(line[from_lsp(line, sc)..from_lsp(line, ec)].to_string())
}

/// Byte offsets in `text` of LSP edits: `(from, to, new text)`, last first, so each can be made in
/// turn with the places still as the server saw them.
pub fn edit_spans<'a>(text: &str, edits: &'a [TextEdit]) -> Vec<(usize, usize, &'a str)> {
    let starts: Vec<usize> = std::iter::once(0).chain(text.match_indices('\n').map(|(i, _)| i + 1)).collect();
    let offset = |(line, ch): (u32, u32)| -> usize {
        let Some(&s) = starts.get(line as usize) else { return text.len() };
        let end = text[s..].find('\n').map(|e| s + e).unwrap_or(text.len());
        s + from_lsp(&text[s..end], ch)
    };
    let mut spans: Vec<(usize, usize, &str)> = edits.iter().map(|e| (offset(e.start), offset(e.end), e.text.as_str())).filter(|(a, b, _)| a <= b).collect();
    spans.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
    spans
}

/// `text` with `edits` made.
pub fn apply_edits(text: &str, edits: &[TextEdit]) -> String {
    let mut out = text.to_string();
    for (a, b, t) in edit_spans(text, edits) {
        out.replace_range(a..b, t);
    }
    out
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

    #[test]
    fn definitions_and_renames_are_read_and_made() {
        let link = json!([{"targetUri": "file:///d%3A/p/a.rs", "targetRange": {"start": {"line": 0, "character": 0}, "end": {"line": 9, "character": 1}},
            "targetSelectionRange": {"start": {"line": 2, "character": 3}, "end": {"line": 2, "character": 7}}}]);
        let l = locations_of(&link);
        assert_eq!(l.len(), 1);
        assert_eq!((l[0].start, l[0].end), ((2, 3), (2, 7)));
        let plain = json!({"uri": "file:///d%3A/p/b.rs", "range": {"start": {"line": 1, "character": 0}, "end": {"line": 1, "character": 2}}});
        assert!(same_path(&locations_of(&plain)[0].path, Path::new(r"D:\p\b.rs")));
        assert!(locations_of(&Value::Null).is_empty());

        let text = "fn go() {}\nfn main() { go(); go() }\n";
        let r = |l, a, b| json!({"range": {"start": {"line": l, "character": a}, "end": {"line": l, "character": b}}, "newText": "run"});
        let we = json!({"documentChanges": [{"textDocument": {"uri": "file:///d%3A/p/a.rs", "version": 3}, "edits": [r(1, 12, 14), r(0, 3, 5), r(1, 18, 20)]},
            {"kind": "rename", "oldUri": "file:///x", "newUri": "file:///y"}]});
        let files = workspace_edit_of(&we);
        assert_eq!(files.len(), 1);
        assert_eq!(apply_edits(text, &files[0].1), "fn run() {}\nfn main() { run(); run() }\n");
        let old = json!({"changes": {"file:///d%3A/p/a.rs": [r(0, 3, 5)]}});
        assert_eq!(workspace_edit_of(&old)[0].1.len(), 1);

        assert_eq!(rename_placeholder(&json!({"range": {"start": {"line": 0, "character": 3}, "end": {"line": 0, "character": 5}}, "placeholder": "go"}), text).as_deref(), Some("go"));
        assert_eq!(rename_placeholder(&json!({"start": {"line": 1, "character": 3}, "end": {"line": 1, "character": 7}}), text).as_deref(), Some("main"));
        assert_eq!(rename_placeholder(&Value::Null, text), None);
        // Thai counts in UTF-16 units, as the server does.
        let thai = "let ชื่อ = 1;";
        let e = [TextEdit { start: (0, 4), end: (0, 8), text: "name".into() }];
        assert_eq!(apply_edits(thai, &e), "let name = 1;");
    }
}
