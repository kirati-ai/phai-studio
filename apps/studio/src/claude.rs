//! Talking to Claude: Anthropic's Messages API over HTTPS (WinHTTP, no other library), its answer
//! read as it streams (server-sent events) into the content blocks it is made of - text, thinking,
//! tool calls - and the API key kept in Windows' Credential Manager, never in a file of Phai
//! Studio's. What the person pays Anthropic for is theirs: no key, no assistant.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use windows::core::{w, PCWSTR};
use windows::Win32::Networking::WinHttp as http;
use windows::Win32::Security::Credentials as cred;

/// The models the assistant offers, the first the default: (id, name).
pub const MODELS: [(&str, &str); 3] = [("claude-opus-5-5", "Claude Opus 5.5"), ("claude-sonnet-5-5", "Claude Sonnet 5.5"), ("claude-haiku-5-5", "Claude Haiku 5.5")];

/// The API version every request names.
pub const API_VERSION: &str = "2026-10-21";

/// Where the key is kept in Credential Manager (Windows Credentials > Generic).
const TARGET: PCWSTR = w!("Phai Studio/Claude API key");

/// The API key: the one kept in Credential Manager, else `ANTHROPIC_API_KEY`.
pub fn api_key() -> Option<String> {
    stored_key().or_else(|| std::env::var("ANTHROPIC_API_KEY").ok()).map(|k| k.trim().to_string()).filter(|k| !k.is_empty())
}

/// Whether a key is kept in Credential Manager (not only in the environment).
pub fn stored_key() -> Option<String> {
    unsafe {
        let mut c: *mut cred::CREDENTIALW = std::ptr::null_mut();
        cred::CredReadW(TARGET, cred::CRED_TYPE_GENERIC, None, &mut c).ok()?;
        let r = &*c;
        let bytes = std::slice::from_raw_parts(r.CredentialBlob, r.CredentialBlobSize as usize).to_vec();
        cred::CredFree(c as *const _);
        String::from_utf8(bytes).ok()
    }
}

/// Keeps `key` in Credential Manager for this Windows user, or takes it out with `None`.
pub fn save_key(key: Option<&str>) -> Result<(), String> {
    unsafe {
        match key.map(str::trim).filter(|k| !k.is_empty()) {
            None => {
                let _ = cred::CredDeleteW(TARGET, cred::CRED_TYPE_GENERIC, None);
                Ok(())
            }
            Some(k) => {
                let mut blob = k.as_bytes().to_vec();
                let user = w!("Phai Studio");
                let c = cred::CREDENTIALW {
                    Type: cred::CRED_TYPE_GENERIC,
                    TargetName: windows::core::PWSTR(TARGET.as_ptr() as *mut u16),
                    CredentialBlobSize: blob.len() as u32,
                    CredentialBlob: blob.as_mut_ptr(),
                    Persist: cred::CRED_PERSIST_LOCAL_MACHINE,
                    UserName: windows::core::PWSTR(user.as_ptr() as *mut u16),
                    ..Default::default()
                };
                cred::CredWriteW(&c, 0).map_err(|e| e.message())
            }
        }
    }
}

/// What arrives while an answer streams in.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Words of the answer.
    Text(String),
    /// Words of a thinking block: a summary of the reasoning, or a note of progress between tools.
    Thinking(String),
    /// A new block began: `text`, `thinking`, `tool_use` (with the tool's name)...
    Block(String, Option<String>),
}

/// An answer, whole: its content blocks as the API gave them (to send back unchanged), why it
/// stopped, and the tool calls whose input was not JSON (by id, with what came).
#[derive(Debug, Default)]
pub struct Reply {
    pub content: Vec<Value>,
    pub stop_reason: String,
    pub stop_details: Value,
    pub usage: Value,
    pub bad_inputs: Vec<(String, String)>,
}

/// Reads server-sent events into a [`Reply`], a piece at a time as they arrive.
#[derive(Default)]
pub struct Assembler {
    pending: Vec<u8>,
    reply: Reply,
    partial: Vec<String>,
    error: Option<String>,
}

impl Assembler {
    /// Bytes of the stream - a line may be split between two pieces, or a letter between bytes.
    pub fn feed(&mut self, bytes: &[u8], on: &mut dyn FnMut(Event)) {
        self.pending.extend_from_slice(bytes);
        while let Some(i) = self.pending.iter().position(|b| *b == b'\n') {
            let line: Vec<u8> = self.pending.drain(..=i).collect();
            let line = String::from_utf8_lossy(&line);
            let line = line.trim_end_matches(['\r', '\n']);
            if let Some(data) = line.strip_prefix("data:") {
                if let Ok(v) = serde_json::from_str::<Value>(data.trim_start()) {
                    self.event(&v, on);
                }
            }
        }
    }

    fn event(&mut self, v: &Value, on: &mut dyn FnMut(Event)) {
        let r = &mut self.reply;
        let index = v["index"].as_u64().unwrap_or(0) as usize;
        match v["type"].as_str().unwrap_or("") {
            "message_start" => r.usage = v["message"]["usage"].clone(),
            "content_block_start" => {
                let mut block = v["content_block"].clone();
                let kind = block["type"].as_str().unwrap_or("").to_string();
                if kind == "tool_use" {
                    block["input"] = json!({});
                }
                while r.content.len() <= index {
                    r.content.push(Value::Null);
                    self.partial.push(String::new());
                }
                on(Event::Block(kind.clone(), block["name"].as_str().map(str::to_string)));
                if let Some(t) = block["text"].as_str().filter(|t| !t.is_empty()) {
                    on(Event::Text(t.to_string()));
                }
                r.content[index] = block;
            }
            "content_block_delta" => {
                let Some(block) = r.content.get_mut(index) else { return };
                let d = &v["delta"];
                let mut append = |field: &str, add: &str| {
                    let was = block[field].as_str().unwrap_or("").to_string();
                    block[field] = Value::String(was + add);
                };
                match d["type"].as_str().unwrap_or("") {
                    "text_delta" => {
                        let t = d["text"].as_str().unwrap_or("");
                        append("text", t);
                        on(Event::Text(t.to_string()));
                    }
                    "thinking_delta" => {
                        let t = d["thinking"].as_str().unwrap_or("");
                        append("thinking", t);
                        if !t.is_empty() {
                            on(Event::Thinking(t.to_string()));
                        }
                    }
                    "signature_delta" => append("signature", d["signature"].as_str().unwrap_or("")),
                    "input_json_delta" => {
                        if let Some(p) = self.partial.get_mut(index) {
                            p.push_str(d["partial_json"].as_str().unwrap_or(""));
                        }
                    }
                    _ => {}
                }
            }
            "content_block_stop" => {
                let Some(block) = r.content.get_mut(index) else { return };
                if block["type"] == "tool_use" {
                    let raw = self.partial.get(index).cloned().unwrap_or_default();
                    // Strictly: a cut-off or broken input is not run, it goes back as an error.
                    match if raw.trim().is_empty() { Ok(json!({})) } else { serde_json::from_str::<Value>(&raw) } {
                        Ok(input) if input.is_object() => block["input"] = input,
                        _ => r.bad_inputs.push((block["id"].as_str().unwrap_or("").to_string(), raw)),
                    }
                }
            }
            "message_delta" => {
                if let Some(s) = v["delta"]["stop_reason"].as_str() {
                    r.stop_reason = s.to_string();
                }
                if !v["delta"]["stop_details"].is_null() {
                    r.stop_details = v["delta"]["stop_details"].clone();
                }
                if let Some(u) = v["usage"].as_object() {
                    if !r.usage.is_object() {
                        r.usage = json!({});
                    }
                    for (k, x) in u {
                        r.usage[k] = x.clone();
                    }
                }
            }
            "error" => self.error = Some(v["error"]["message"].as_str().unwrap_or("the API reported an error").to_string()),
            _ => {}
        }
    }

    /// The answer, once the stream has ended.
    pub fn finish(mut self) -> Result<Reply, String> {
        if let Some(e) = self.error {
            return Err(e);
        }
        self.reply.content.retain(|b| !b.is_null());
        Ok(self.reply)
    }
}

/// Stops a request that is under way, from another thread.
#[derive(Clone, Default)]
pub struct Cancel {
    stopped: Arc<AtomicBool>,
    request: Arc<Mutex<usize>>,
}

impl Cancel {
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
        let mut r = self.request.lock().unwrap_or_else(|e| e.into_inner());
        if *r != 0 {
            unsafe {
                let _ = http::WinHttpCloseHandle(*r as *mut _);
            }
            *r = 0;
        }
    }
    pub fn stopped(&self) -> bool {
        self.stopped.load(Ordering::SeqCst)
    }
}

/// Why a request failed: the HTTP status (0 when it never got one) and what the API or Windows said.
#[derive(Debug)]
pub struct Failure {
    pub status: u32,
    pub message: String,
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.status == 0 { write!(f, "{}", self.message) } else { write!(f, "{} ({})", self.message, self.status) }
    }
}

struct Handle(*mut core::ffi::c_void);

impl Drop for Handle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                let _ = http::WinHttpCloseHandle(self.0);
            }
        }
    }
}

/// Where requests go: Anthropic's API, or `PHAI_CLAUDE_URL` (`http://127.0.0.1:8123`) for a test's
/// own server. (scheme is https, host, port.)
fn endpoint() -> (bool, String, u16) {
    let url = std::env::var("PHAI_CLAUDE_URL").unwrap_or_default();
    let (secure, rest) = match (url.strip_prefix("http://"), url.strip_prefix("https://")) {
        (Some(r), _) => (false, r),
        (_, Some(r)) => (true, r),
        _ => return (true, "api.anthropic.com".into(), 443),
    };
    let rest = rest.trim_end_matches('/');
    match rest.rsplit_once(':').and_then(|(h, p)| Some((h, p.parse::<u16>().ok()?))) {
        Some((h, p)) => (secure, h.to_string(), p),
        None => (secure, rest.to_string(), if secure { 443 } else { 80 }),
    }
}

/// POSTs `body` to the Messages API and reads its stream, telling `on` as it comes.
pub fn post(key: &str, betas: &[&str], body: &str, cancel: &Cancel, on: &mut dyn FnMut(Event)) -> Result<Reply, Failure> {
    let fail = |m: String| Failure { status: 0, message: m };
    let last = || windows::core::Error::from_thread().message();
    unsafe {
        let agent = windows::core::HSTRING::from(format!("phai-studio/{}", env!("CARGO_PKG_VERSION")));
        let session = Handle(http::WinHttpOpen(&agent, http::WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, PCWSTR::null(), PCWSTR::null(), 0));
        if session.0.is_null() {
            return Err(fail(last()));
        }
        // A long answer streams for minutes: wait up to ten between pieces.
        let _ = http::WinHttpSetTimeouts(session.0, 0, 30_000, 60_000, 600_000);
        let (secure, host, port) = endpoint();
        let connect = Handle(http::WinHttpConnect(session.0, &windows::core::HSTRING::from(host), port, 0));
        if connect.0.is_null() {
            return Err(fail(last()));
        }
        let req = http::WinHttpOpenRequest(connect.0, w!("POST"), w!("/v1/messages"), PCWSTR::null(), PCWSTR::null(), std::ptr::null(), if secure { http::WINHTTP_FLAG_SECURE } else { Default::default() });
        if req.is_null() {
            return Err(fail(last()));
        }
        *cancel.request.lock().unwrap_or_else(|e| e.into_inner()) = req as usize;
        // Whoever closes it - Stop, or this when done - takes it from the shared place first.
        let close = || {
            let mut r = cancel.request.lock().unwrap_or_else(|e| e.into_inner());
            if *r != 0 {
                let _ = http::WinHttpCloseHandle(*r as *mut _);
                *r = 0;
            }
        };
        let mut headers = format!("Content-Type: application/json\r\nx-api-key: {key}\r\nanthropic-version: {API_VERSION}\r\n");
        if !betas.is_empty() {
            headers.push_str(&format!("anthropic-beta: {}\r\n", betas.join(",")));
        }
        let headers: Vec<u16> = headers.encode_utf16().collect();
        let sent = http::WinHttpSendRequest(req, Some(&headers), Some(body.as_ptr() as *const _), body.len() as u32, body.len() as u32, 0).and_then(|_| http::WinHttpReceiveResponse(req, std::ptr::null_mut()));
        if let Err(e) = sent {
            close();
            return Err(fail(if cancel.stopped() { "stopped".into() } else { e.message() }));
        }
        let mut status: u32 = 0;
        let mut size = 4u32;
        let _ = http::WinHttpQueryHeaders(req, http::WINHTTP_QUERY_STATUS_CODE | http::WINHTTP_QUERY_FLAG_NUMBER, PCWSTR::null(), Some(&mut status as *mut u32 as *mut _), &mut size, std::ptr::null_mut());
        let mut asm = Assembler::default();
        let mut whole = Vec::new();
        let mut buf = vec![0u8; 16 * 1024];
        loop {
            let mut read = 0u32;
            if http::WinHttpReadData(req, buf.as_mut_ptr() as *mut _, buf.len() as u32, &mut read).is_err() {
                close();
                return Err(fail(if cancel.stopped() { "stopped".into() } else { last() }));
            }
            if read == 0 {
                break;
            }
            let piece = &buf[..read as usize];
            if status == 200 {
                asm.feed(piece, on);
            } else {
                whole.extend_from_slice(piece);
            }
        }
        close();
        if status != 200 {
            let text = String::from_utf8_lossy(&whole).to_string();
            let message = serde_json::from_str::<Value>(&text).ok().and_then(|v| v["error"]["message"].as_str().map(str::to_string)).unwrap_or(text);
            return Err(Failure { status, message });
        }
        asm.finish().map_err(fail)
    }
}

/// Base64, for a picture sent to the model.
pub fn base64(data: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        out.push(A[(n >> 18) as usize & 63] as char);
        out.push(A[(n >> 12) as usize & 63] as char);
        out.push(if c.len() > 1 { A[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if c.len() > 2 { A[n as usize & 63] as char } else { '=' });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_streamed_answer_comes_together_in_its_blocks() {
        let stream = concat!(
            "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"usage\":{\"input_tokens\":10}}}\n\n",
            "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"thinking\",\"thinking\":\"\",\"signature\":\"\"}}\n\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"Looking at the form.\"}}\n\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"signature_delta\",\"signature\":\"sig==\"}}\n\n",
            "data: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
            "data: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
            "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"text_delta\",\"text\":\"สวัสดี \"}}\n\n",
            "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"text_delta\",\"text\":\"ครับ\"}}\n\n",
            "data: {\"type\":\"content_block_stop\",\"index\":1}\n\n",
            "data: {\"type\":\"content_block_start\",\"index\":2,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"read_file\",\"input\":{}}}\n\n",
            "data: {\"type\":\"content_block_delta\",\"index\":2,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"path\\\": \\\"src/\"}}\n\n",
            "data: {\"type\":\"content_block_delta\",\"index\":2,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"main.rs\\\"}\"}}\n\n",
            "data: {\"type\":\"content_block_stop\",\"index\":2}\n\n",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":42}}\n\n",
            "data: {\"type\":\"message_stop\"}\n\n",
        );
        // In pieces that cut lines and a Thai letter's bytes apart.
        let bytes = stream.as_bytes();
        let mut asm = Assembler::default();
        let mut seen = Vec::new();
        for piece in bytes.chunks(7) {
            asm.feed(piece, &mut |e| seen.push(e));
        }
        let r = asm.finish().unwrap();
        assert_eq!(r.stop_reason, "tool_use");
        assert_eq!(r.usage["input_tokens"], 10);
        assert_eq!(r.usage["output_tokens"], 42);
        assert_eq!(r.content.len(), 3);
        assert_eq!(r.content[0], json!({"type": "thinking", "thinking": "Looking at the form.", "signature": "sig=="}));
        assert_eq!(r.content[1], json!({"type": "text", "text": "สวัสดี ครับ"}));
        assert_eq!(r.content[2], json!({"type": "tool_use", "id": "toolu_1", "name": "read_file", "input": {"path": "src/main.rs"}}));
        assert!(r.bad_inputs.is_empty());
        let text: String = seen.iter().filter_map(|e| if let Event::Text(t) = e { Some(t.as_str()) } else { None }).collect();
        assert_eq!(text, "สวัสดี ครับ");
        assert!(seen.contains(&Event::Block("tool_use".into(), Some("read_file".into()))));
    }

    #[test]
    fn a_broken_tool_input_is_kept_aside_and_an_error_is_an_error() {
        let mut asm = Assembler::default();
        let s = "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"t1\",\"name\":\"write_file\",\"input\":{}}}\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"path\\\": \\\"a\"}}\ndata: {\"type\":\"content_block_stop\",\"index\":0}\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"max_tokens\"}}\n";
        asm.feed(s.as_bytes(), &mut |_| {});
        let r = asm.finish().unwrap();
        assert_eq!(r.bad_inputs, vec![("t1".to_string(), "{\"path\": \"a".to_string())]);
        assert_eq!(r.stop_reason, "max_tokens");
        let mut asm = Assembler::default();
        asm.feed(b"data: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\",\"message\":\"Overloaded\"}}\n", &mut |_| {});
        assert_eq!(asm.finish().unwrap_err(), "Overloaded");
    }

    /// A request and its stream through WinHTTP, to a server of the test's own on this computer -
    /// the headers Anthropic needs sent, the answer read; and an error status read as an error.
    #[test]
    fn a_request_goes_out_and_its_stream_comes_back() {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let mut seen = Vec::new();
            for answer in [200, 401] {
                let (mut s, _) = listener.accept().unwrap();
                let mut req = Vec::new();
                let mut buf = [0u8; 4096];
                // Headers, then the body as long as Content-Length says.
                loop {
                    let n = s.read(&mut buf).unwrap();
                    req.extend_from_slice(&buf[..n]);
                    let text = String::from_utf8_lossy(&req).to_string();
                    if let Some(end) = text.find("\r\n\r\n") {
                        let len = text.lines().find_map(|l| l.to_lowercase().strip_prefix("content-length:").map(|v| v.trim().parse::<usize>().unwrap())).unwrap_or(0);
                        if req.len() >= end + 4 + len {
                            break;
                        }
                    }
                }
                seen.push(String::from_utf8_lossy(&req).to_string());
                let body = if answer == 200 {
                    "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hi\"}}\n\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"}}\n\n".to_string()
                } else {
                    "{\"type\":\"error\",\"error\":{\"type\":\"authentication_error\",\"message\":\"invalid x-api-key\"}}".to_string()
                };
                let status = if answer == 200 { "200 OK" } else { "401 Unauthorized" };
                let _ = write!(s, "HTTP/1.1 {status}\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            }
            seen
        });
        // SAFETY of the test: only this test sets it, and every request here goes to the server above.
        unsafe { std::env::set_var("PHAI_CLAUDE_URL", format!("http://127.0.0.1:{port}")) };
        let mut text = String::new();
        let r = post("sk-test", &["server-side-fallback-2026-07-01"], "{\"model\":\"claude-opus-5-5\"}", &Cancel::default(), &mut |e| {
            if let Event::Text(t) = e {
                text.push_str(&t);
            }
        })
        .unwrap();
        assert_eq!(text, "Hi");
        assert_eq!(r.content, vec![json!({"type": "text", "text": "Hi"})]);
        assert_eq!(r.stop_reason, "end_turn");
        let e = post("bad", &[], "{}", &Cancel::default(), &mut |_| {}).unwrap_err();
        assert_eq!((e.status, e.message.as_str()), (401, "invalid x-api-key"));
        let seen = server.join().unwrap();
        let first = seen[0].to_lowercase();
        assert!(first.starts_with("post /v1/messages http/1.1"), "{first}");
        for h in ["x-api-key: sk-test", "anthropic-version: 2026-10-21", "anthropic-beta: server-side-fallback-2026-07-01", "content-type: application/json"] {
            assert!(first.contains(h), "{h} in {first}");
        }
        assert!(seen[0].ends_with("{\"model\":\"claude-opus-5-5\"}"));
    }

    #[test]
    fn base64_is_standard() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }
}
