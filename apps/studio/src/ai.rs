//! The AI assistant: a pane beside Project and Properties where Claude answers questions about the
//! project and - in Act mode - changes it: reads and searches its files, edits code and forms
//! (each change shown side by side to accept or not), builds and runs its tests until they pass,
//! and photographs the running program to see what it made. In Ask mode it changes nothing.
//!
//! The conversation is kept as the API gave it and only ever added to, so Claude's thinking stays
//! valid from one turn to the next and the prompt cache keeps hitting. The tools are the same in
//! both modes - which mode is in force is said in each message - for the same reason.

use std::path::{Component, Path, PathBuf};
use std::sync::mpsc;

use phai_ui::text::{t, tf};
use phai_ui::win;
use serde_json::{json, Value};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::HFONT;
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::claude;
use crate::explorer;
use crate::studio;

const ID_TRANSCRIPT: i32 = 10;
const ID_INPUT: i32 = 11;
const ID_SEND: i32 = 12;
const ID_MODE: i32 = 13;
const ID_MODEL: i32 = 14;
const ID_NEW: i32 = 15;
const ID_KEY: i32 = 16;
const ID_STATUS: i32 = 17;

/// What the worker tells the pane: its LPARAM is a boxed [`Note`].
const WM_AI: u32 = wm::WM_APP + 0x0E40;

/// Effort: medium is Claude Opus 5.5's own default, and the level its makers start agentic coding at.
const EFFORT: &str = "medium";
/// Room for the thinking as well as the answer of a long coding turn.
const MAX_TOKENS: u32 = 64_000;
/// Turns of tool use in one answer before the assistant stops to let the person look.
const MAX_STEPS: usize = 60;

/// What the worker thread sends the pane.
enum Note {
    Text(String),
    Thinking(String),
    Block(String),
    Line(String),
    Status(String),
    /// Something to do on the window's thread (the open editors, the designer, a dialog), and
    /// where its answer goes.
    Run(Box<dyn FnOnce() -> Value + Send>, mpsc::Sender<Value>),
    /// The answer is over: the conversation as it now stands, and why it ended early, if it did.
    Done(Vec<Value>, Option<String>),
}

struct Pane {
    transcript: HWND,
    input: HWND,
    send: HWND,
    mode: HWND,
    model: HWND,
    new_chat: HWND,
    key: HWND,
    status: HWND,
    /// The conversation, as sent and received - only added to.
    messages: Vec<Value>,
    /// This conversation's system prompt and project, fixed when it began.
    system: String,
    root: Option<PathBuf>,
    busy: Option<claude::Cancel>,
    /// Accept All was chosen: the rest of this conversation's changes go in without asking.
    accept_all: bool,
    /// The last block shown: a new kind starts a new paragraph.
    last_block: String,
}

/// The pane, made inside `parent` (the side), hidden until the View menu shows it.
pub fn create(parent: HWND, font: HFONT) -> HWND {
    let class = win::register("PhaiStudioAI", Some(proc_), None);
    unsafe {
        let h = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_CLIPCHILDREN, 0, 0, 10, 10, Some(parent), None, Some(win::instance()), None).unwrap_or_default();
        let transcript = win::textarea_with(h, "", ID_TRANSCRIPT, font, true, "vertical");
        let _ = wm::SendMessageW(transcript, 0x00CF /* EM_SETREADONLY */, Some(WPARAM(1)), None);
        let input = win::textarea_with(h, "", ID_INPUT, font, true, "vertical");
        let send = win::button(h, t("Send"), ID_SEND, font, true);
        let mode = win::combo(h, ID_MODE, font, &[t("Ask"), t("Act")]);
        let names: Vec<&str> = claude::MODELS.iter().map(|(_, n)| *n).collect();
        let model = win::combo(h, ID_MODEL, font, &names);
        let chosen = crate::setting_pub("ai_model").and_then(|m| claude::MODELS.iter().position(|(id, _)| *id == m)).unwrap_or(0);
        let _ = wm::SendMessageW(model, 0x014E /* CB_SETCURSEL */, Some(WPARAM(chosen)), None);
        let act = crate::setting_pub("ai_mode").as_deref() == Some("act");
        let _ = wm::SendMessageW(mode, 0x014E, Some(WPARAM(act as usize)), None);
        let new_chat = win::button(h, t("New chat"), ID_NEW, font, false);
        let key = win::button(h, t("API key..."), ID_KEY, font, false);
        let status = win::label(h, "", ID_STATUS, font);
        win::set_state(h, Box::new(Pane { transcript, input, send, mode, model, new_chat, key, status, messages: vec![], system: String::new(), root: None, busy: None, accept_all: false, last_block: String::new() }));
        let p = state(h).unwrap();
        p.greet();
        // `PHAI_AI_ASK=<words>`: asked by itself a moment after Studio opens - for tests and the
        // guide's picture, with `PHAI_CLAUDE_URL` pointing at a test's own server.
        if std::env::var_os("PHAI_AI_ASK").is_some() {
            wm::SetTimer(Some(h), 1, 1500, None);
        }
        h
    }
}

fn state(h: HWND) -> Option<&'static mut Pane> {
    unsafe { win::state::<Pane>(h) }
}

impl Pane {
    fn greet(&mut self) {
        let words = if claude::api_key().is_some() {
            t("Ask about your project, or switch to Act and say what to change. Claude reads the files it needs; in Act mode each change is shown to you first.")
        } else {
            t("Claude needs an API key from console.anthropic.com - click API key... The key stays in Windows' Credential Manager; what Claude costs, Anthropic bills you.")
        };
        win::set_text(self.transcript, &format!("{words}\r\n\r\n"));
    }

    fn append(&mut self, text: &str) {
        let text = text.replace("\r\n", "\n").replace('\n', "\r\n");
        let w = win::wide(&text);
        unsafe {
            let n = wm::GetWindowTextLengthW(self.transcript) as usize;
            let _ = wm::SendMessageW(self.transcript, 0x00B1 /* EM_SETSEL */, Some(WPARAM(n)), Some(LPARAM(n as isize)));
            let _ = wm::SendMessageW(self.transcript, 0x00C2 /* EM_REPLACESEL */, Some(WPARAM(0)), Some(LPARAM(w.as_ptr() as isize)));
            let _ = wm::SendMessageW(self.transcript, 0x00B7 /* EM_SCROLLCARET */, None, None);
        }
    }

    fn layout(&self, h: HWND) {
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(h, &mut r);
        }
        let px = |v| win::px(h, v);
        let (w, ht) = (r.right, r.bottom);
        let row = px(26);
        let m = px(6);
        // The bar: the mode and the model.
        win::move_to(self.mode, m, m, px(76), px(240));
        win::move_to(self.model, m * 2 + px(76), m, (w - m * 3 - px(76)).max(px(80)), px(240));
        // Below: the box to write in, Send, New chat and the key beside it.
        let bw = px(84);
        let input_h = row * 3 + px(8);
        let top = m * 2 + row;
        let status_h = px(20);
        win::move_to(self.transcript, m, top, w - m * 2, (ht - top - input_h - status_h - m * 3).max(px(40)));
        win::move_to(self.status, m, ht - input_h - status_h - m * 2, w - m * 2, status_h);
        let y = ht - input_h - m;
        win::move_to(self.input, m, y, (w - m * 3 - bw).max(px(60)), input_h);
        win::move_to(self.send, w - m - bw, y, bw, row);
        win::move_to(self.new_chat, w - m - bw, y + row + px(4), bw, row);
        win::move_to(self.key, w - m - bw, y + (row + px(4)) * 2, bw, row);
    }

    fn act(&self) -> bool {
        win::combo_selected(self.mode) == 1
    }

    fn model_id(&self) -> &'static str {
        claude::MODELS.get(win::combo_selected(self.model).max(0) as usize).map(|(id, _)| *id).unwrap_or(claude::MODELS[0].0)
    }

    fn new_chat(&mut self) {
        if let Some(c) = self.busy.take() {
            c.stop();
        }
        self.messages.clear();
        self.system.clear();
        self.root = None;
        self.accept_all = false;
        self.last_block.clear();
        win::set_text(self.send, t("Send"));
        win::set_text(self.status, "");
        self.greet();
    }

    fn ask_key(&mut self, h: HWND) {
        let owner = studio().map(|s| s.hwnd).unwrap_or(h);
        let Some(a) = phai_ui::dialogs::ask_fields(Some(owner), t("Claude API key"), &[phai_ui::dialogs::Field::Password(t("API key from console.anthropic.com (empty: forget it)"), "")]) else { return };
        match claude::save_key(Some(&a[0]).filter(|k| !k.trim().is_empty()).map(String::as_str)) {
            Ok(()) => {
                let words = if a[0].trim().is_empty() { t("The API key is forgotten.") } else { t("The API key is kept in Windows' Credential Manager.") };
                win::set_text(self.status, words);
            }
            Err(e) => {
                win::message(Some(owner), &e, t("Claude API key"), wm::MB_ICONWARNING);
            }
        }
    }

    fn send(&mut self, h: HWND) {
        if let Some(c) = self.busy.take() {
            c.stop();
            win::set_text(self.status, t("Stopping..."));
            return;
        }
        let words = win::text_of(self.input).trim().to_string();
        if words.is_empty() {
            return;
        }
        if claude::api_key().is_none() {
            self.ask_key(h);
        }
        let Some(key) = claude::api_key() else { return };
        let Some(root) = studio().and_then(|s| s.project_dir_pub()) else {
            win::set_text(self.status, t("Open a project first: Claude works in the project of the open file."));
            return;
        };
        if self.root.as_deref() != Some(root.as_path()) {
            if !self.messages.is_empty() {
                self.new_chat();
            }
            self.root = Some(root.clone());
            self.system = system_prompt(&root);
        }
        let act = self.act();
        crate::remember_setting_pub("ai_mode", if act { "act" } else { "ask" });
        crate::remember_setting_pub("ai_model", self.model_id());
        let mode = if act { "Act: you may change the project's files, each change is shown to the person to accept." } else { "Ask: answer only - do not change any file (write_file and edit_file will refuse)." };
        self.messages.push(json!({"role": "user", "content": [
            {"type": "text", "text": words},
            {"type": "text", "text": format!("<studio>Mode: {mode}</studio>")}
        ]}));
        win::set_text(self.input, "");
        self.append(&format!("{} {words}\n\n", t("You:")));
        self.last_block.clear();
        let cancel = claude::Cancel::default();
        self.busy = Some(cancel.clone());
        win::set_text(self.send, t("Stop"));
        win::set_text(self.status, t("Claude is thinking..."));
        let job = Job { key, model: self.model_id().to_string(), system: self.system.clone(), messages: self.messages.clone(), root, act, cancel, pane: h.0 as isize };
        std::thread::spawn(move || job.run());
    }

    fn note(&mut self, h: HWND, n: Note) {
        match n {
            Note::Block(kind) => {
                if kind != self.last_block && (kind == "text" || kind == "thinking") {
                    if self.last_block.is_empty() {
                        self.append(&format!("{}\n", t("Claude:")));
                    } else {
                        self.append("\n");
                    }
                    if kind == "thinking" {
                        self.append("› ");
                    }
                }
                self.last_block = kind;
            }
            Note::Text(s) => self.append(&s),
            Note::Thinking(s) => self.append(&s),
            Note::Line(s) => {
                self.append(&format!("\n⚙ {s}\n"));
                self.last_block = "tool".into();
            }
            Note::Status(s) => win::set_text(self.status, &s),
            Note::Run(f, back) => {
                let _ = back.send(f());
            }
            Note::Done(messages, err) => {
                // Only what this answer added to the conversation it was given.
                if messages.len() >= self.messages.len() {
                    self.messages = messages;
                }
                self.busy = None;
                win::set_text(self.send, t("Send"));
                win::set_text(self.status, &err.clone().unwrap_or_default());
                if let Some(e) = err {
                    self.append(&format!("\n[{e}]\n"));
                }
                self.append("\n\n");
                self.last_block.clear();
                let _ = h;
            }
        }
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_SIZE => {
            if let Some(p) = state(h) {
                p.layout(h);
            }
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            let Some(p) = state(h) else { return LRESULT(0) };
            match win::loword(w.0) as i32 {
                ID_SEND => p.send(h),
                ID_NEW => p.new_chat(),
                ID_KEY => p.ask_key(h),
                _ => {}
            }
            LRESULT(0)
        }
        wm::WM_TIMER if w.0 == 1 => {
            unsafe {
                let _ = wm::KillTimer(Some(h), 1);
            }
            if let (Some(p), Ok(words)) = (state(h), std::env::var("PHAI_AI_ASK")) {
                if let Some(s) = studio() {
                    explorer::show_ai(s.side, true);
                }
                // `act:` before the words: in Act mode.
                let (act, words) = match words.strip_prefix("act:") {
                    Some(w) => (true, w.to_string()),
                    None => (false, words),
                };
                let _ = unsafe { wm::SendMessageW(p.mode, 0x014E /* CB_SETCURSEL */, Some(WPARAM(act as usize)), None) };
                win::set_text(p.input, &words);
                p.send(h);
            }
            LRESULT(0)
        }
        WM_AI => {
            let n = unsafe { Box::from_raw(l.0 as *mut Note) };
            if let Some(p) = state(h) {
                p.note(h, *n);
            }
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            drop(unsafe { win::take_state::<Pane>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::ctl_color(msg, w, l).unwrap_or_else(|| win::def(h, msg, w, l)),
    }
}

/// Ctrl+Enter in the box below sends, as in a chat.
pub fn is_send_key(focus: HWND, vk: usize, ctrl: bool) -> Option<HWND> {
    if vk != 0x0D || !ctrl {
        return None;
    }
    let pane = unsafe { wm::GetParent(focus) }.ok()?;
    state(pane).filter(|p| p.input == focus).map(|_| pane)
}

/// Sends what is in the box, from the keyboard.
pub fn send_from_key(pane: HWND) {
    if let Some(p) = state(pane) {
        p.send(pane);
    }
}

/// What this conversation tells Claude about where it is - the same for every turn of it.
fn system_prompt(root: &Path) -> String {
    let name = root.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    format!(
        "You are the AI assistant inside Phai Studio, a RAD IDE for Windows programs written in Rust. \
The person is working on the project \"{name}\"; your tools work inside its folder, with paths relative to it.\n\n\
A Phai Studio project: Cargo.toml; src/main.rs; each window is a form - `<name>.phf` (TOML that the visual designer \
reads and writes: controls, their bounds, properties, events), `<name>_ui.rs` (Rust that Phai Studio generates from \
the .phf - never edit it, it is written again on every save and build), and `<name>.rs` (the person's code: \
`impl <Name>Events for <Name>` with a method for each event the .phf names). The controls are phai-ui's \
(a WinForms-like library on Win32); `ui.<id>` reaches a control. src/text.rs holds the Thai for every English \
sentence the program shows; theme.css styles the controls.\n\n\
Before you write or change a .phf, read the reference with phai_reference: Phai Studio refuses a form file it cannot read. \
After changing code or forms, build; fix what the compiler says and build again until it builds. Run the tests \
when there are tests for what you changed. A screenshot of the running program shows what a form looks like.\n\n\
Each message says the mode. In Ask mode, answer and explain without changing files. In Act mode, change what the \
request needs and nothing more; the person sees each change side by side and accepts it or not - a declined \
change is their decision, so do not try it again unasked.\n\n\
Before your first tool call, say in one line what you are going to do. When you are done, say in a few lines \
what you changed and what you checked. Answer in the language the person writes in - often Thai."
    )
}

/// The tools: the same in both modes, so the conversation's beginning never changes.
fn tools() -> Value {
    let path = json!({"type": "string", "description": "A path relative to the project folder, with / between folders"});
    json!([
        {"name": "list_files", "description": "Lists the project's files and folders (not target/ or .git/), folders ending in /. Give a folder to list only it.",
         "eager_input_streaming": true,
         "input_schema": {"type": "object", "properties": {"path": path}, "required": []}},
        {"name": "read_file", "description": "Reads a text file of the project, each line numbered. An open file is read as the editor has it, unsaved changes included. Long files come in parts: give offset (the first line, from 1) and limit (how many lines).",
         "eager_input_streaming": true,
         "input_schema": {"type": "object", "properties": {"path": path, "offset": {"type": "integer"}, "limit": {"type": "integer"}}, "required": ["path"]}},
        {"name": "search", "description": "Finds the lines of the project's text files (.rs .phf .toml .css .md ...) that contain the words, ignoring case. Up to 200 lines, as path:line: text.",
         "eager_input_streaming": true,
         "input_schema": {"type": "object", "properties": {"text": {"type": "string"}, "path": {"type": "string", "description": "Only in this folder"}}, "required": ["text"]}},
        {"name": "write_file", "description": "Writes a whole file, new or replacing what it has. Act mode only; the person accepts or declines the change. For a small change to a big file, edit_file is better.",
         "eager_input_streaming": true,
         "input_schema": {"type": "object", "properties": {"path": path, "contents": {"type": "string"}}, "required": ["path", "contents"]}},
        {"name": "edit_file", "description": "Replaces old_text with new_text in a file - old_text exactly as the file has it (see read_file, without the line numbers) and found once, unless replace_all. Act mode only; the person accepts or declines the change.",
         "eager_input_streaming": true,
         "input_schema": {"type": "object", "properties": {"path": path, "old_text": {"type": "string"}, "new_text": {"type": "string"}, "replace_all": {"type": "boolean"}}, "required": ["path", "old_text", "new_text"]}},
        {"name": "build", "description": "Saves the open files, writes the forms' generated code, and builds the project with cargo. Returns whether it built, with the compiler's errors and warnings.",
         "eager_input_streaming": true,
         "input_schema": {"type": "object", "properties": {}, "required": []}},
        {"name": "run_tests", "description": "Runs the project's tests (cargo test), only those whose names have filter when given. Returns the end of what cargo said.",
         "eager_input_streaming": true,
         "input_schema": {"type": "object", "properties": {"filter": {"type": "string"}}, "required": []}},
        {"name": "screenshot", "description": "Builds the project, runs it and photographs its first window as it opens (the program is closed again at once, nothing is clicked). Returns the picture.",
         "eager_input_streaming": true,
         "input_schema": {"type": "object", "properties": {}, "required": []}},
        {"name": "phai_reference", "description": "The reference of Phai Studio's form file (.phf): every field of a form and its controls, the WinForms-like properties and events, examples. Read it before writing or changing a .phf.",
         "eager_input_streaming": true,
         "input_schema": {"type": "object", "properties": {}, "required": []}}
    ])
}

/// One answer of Claude's, tools and all, on a thread of its own.
struct Job {
    key: String,
    model: String,
    system: String,
    messages: Vec<Value>,
    root: PathBuf,
    act: bool,
    cancel: claude::Cancel,
    pane: isize,
}

impl Job {
    fn tell(&self, n: Note) {
        let b = Box::into_raw(Box::new(n));
        unsafe {
            if wm::PostMessageW(Some(HWND(self.pane as *mut _)), WM_AI, WPARAM(0), LPARAM(b as isize)).is_err() {
                drop(Box::from_raw(b));
            }
        }
    }

    /// Runs `f` on the window's thread and waits for what it returns.
    fn on_ui(&self, f: impl FnOnce() -> Value + Send + 'static) -> Value {
        let (tx, rx) = mpsc::channel();
        self.tell(Note::Run(Box::new(f), tx));
        rx.recv().unwrap_or(Value::Null)
    }

    fn run(mut self) {
        let tools = tools();
        let mut fallbacks = true;
        let mut tries = 0;
        let mut steps = 0;
        let ended = loop {
            if self.cancel.stopped() {
                break Some(t("Stopped").to_string());
            }
            let mut body = json!({
                "model": self.model,
                "max_tokens": MAX_TOKENS,
                "stream": true,
                "system": [{"type": "text", "text": self.system}],
                "tools": tools,
                "messages": self.messages,
                "thinking": {"type": "adaptive", "display": "summarized"},
                "output_config": {"effort": EFFORT},
                "cache_control": {"type": "ephemeral"}
            });
            // A safety classifier's false alarm answered by the model Anthropic recommends,
            // rather than a stop - where the account and model allow it.
            let mut betas = vec![];
            if fallbacks && self.model != "claude-haiku-5-5" {
                body["fallbacks"] = json!("default");
                betas.push("server-side-fallback-2026-07-01");
            }
            let pane = self.pane;
            let tell = |n: Note| {
                let b = Box::into_raw(Box::new(n));
                unsafe {
                    if wm::PostMessageW(Some(HWND(pane as *mut _)), WM_AI, WPARAM(0), LPARAM(b as isize)).is_err() {
                        drop(Box::from_raw(b));
                    }
                }
            };
            let mut on = |e: claude::Event| match e {
                claude::Event::Text(s) => tell(Note::Text(s)),
                claude::Event::Thinking(s) => tell(Note::Thinking(s)),
                claude::Event::Block(kind, name) => {
                    if let Some(n) = name {
                        tell(Note::Status(tf("Claude is preparing {0}...", &[&n])));
                    } else if kind == "text" {
                        tell(Note::Status(t("Claude is writing...").into()));
                    }
                    tell(Note::Block(kind));
                }
            };
            let reply = match claude::post(&self.key, &betas, &body.to_string(), &self.cancel, &mut on) {
                Ok(r) => r,
                Err(f) => {
                    if self.cancel.stopped() {
                        break Some(t("Stopped").to_string());
                    }
                    if f.status == 400 && fallbacks && f.message.to_lowercase().contains("fallback") {
                        fallbacks = false;
                        continue;
                    }
                    // Busy or overloaded: wait and try again, a few times.
                    if (f.status == 429 || f.status == 529 || f.status >= 500 || f.status == 0) && tries < 3 {
                        tries += 1;
                        self.tell(Note::Status(tf("{0} - trying again in {1} s", &[&f.to_string(), &(2u64 << tries)])));
                        std::thread::sleep(std::time::Duration::from_secs(2 << tries));
                        continue;
                    }
                    break Some(match f.status {
                        401 => t("The API key was refused - click API key... to give another.").to_string(),
                        _ => f.to_string(),
                    });
                }
            };
            tries = 0;
            if !reply.content.is_empty() {
                self.messages.push(json!({"role": "assistant", "content": reply.content}));
            }
            match reply.stop_reason.as_str() {
                "refusal" => break Some(t("Claude declined this request.").to_string()),
                "pause_turn" => continue,
                "tool_use" | "max_tokens" => {
                    let calls: Vec<Value> = reply.content.iter().filter(|b| b["type"] == "tool_use").cloned().collect();
                    if calls.is_empty() {
                        break (reply.stop_reason == "max_tokens").then(|| t("The answer was cut off - it was too long.").to_string());
                    }
                    let mut results = Vec::new();
                    for c in &calls {
                        let id = c["id"].as_str().unwrap_or("").to_string();
                        let result = if self.cancel.stopped() {
                            error_result(&id, "The person stopped the assistant.")
                        } else if reply.stop_reason == "max_tokens" {
                            error_result(&id, "The answer was cut off at max_tokens before this call was complete - it was not run. Write smaller pieces (edit_file rather than a whole file).")
                        } else if let Some((_, raw)) = reply.bad_inputs.iter().find(|(b, _)| *b == id) {
                            error_result(&id, &json!({"INVALID_JSON": raw}).to_string())
                        } else {
                            self.call(&id, c["name"].as_str().unwrap_or(""), &c["input"])
                        };
                        results.push(result);
                    }
                    self.messages.push(json!({"role": "user", "content": results}));
                    steps += 1;
                    if steps >= MAX_STEPS {
                        break Some(tf("Stopped after {0} steps - say go on to continue.", &[&MAX_STEPS]));
                    }
                }
                _ => break None,
            }
        };
        let messages = std::mem::take(&mut self.messages);
        self.tell(Note::Done(messages, ended));
    }

    /// Runs one tool; its result block.
    fn call(&self, id: &str, name: &str, input: &Value) -> Value {
        let path = input["path"].as_str().unwrap_or("").to_string();
        self.tell(Note::Line(match name {
            "list_files" => format!("{} {}", t("List files"), if path.is_empty() { "." } else { &path }),
            "read_file" => format!("{} {path}", t("Read")),
            "search" => format!("{} \"{}\"", t("Search"), input["text"].as_str().unwrap_or("")),
            "write_file" => format!("{} {path}", t("Write")),
            "edit_file" => format!("{} {path}", t("Edit")),
            "build" => t("Build").to_string(),
            "run_tests" => t("Run tests").to_string(),
            "screenshot" => t("Screenshot of the program").to_string(),
            "phai_reference" => t("Read the form file reference").to_string(),
            other => other.to_string(),
        }));
        let root = self.root.clone();
        let input = input.clone();
        let done = |r: Result<String, String>| match r {
            Ok(s) => json!({"type": "tool_result", "tool_use_id": id, "content": s}),
            Err(e) => error_result(id, &e),
        };
        match name {
            "list_files" => done(list_files(&root, &path)),
            "search" => done(search(&root, input["text"].as_str().unwrap_or(""), input["path"].as_str().unwrap_or(""))),
            "phai_reference" => done(Ok(include_str!("../../../docs/phaiform.md").to_string())),
            "read_file" => {
                let r = self.on_ui(move || json!(read_file(&root, &input).map_err(|e| format!("\u{1}{e}")).unwrap_or_else(|e| e)));
                let s = r.as_str().unwrap_or("").to_string();
                done(match s.strip_prefix('\u{1}') {
                    Some(e) => Err(e.to_string()),
                    None => Ok(s),
                })
            }
            "write_file" | "edit_file" => {
                if !self.act {
                    return done(Err("Ask mode: no file is changed. Explain the change instead; the person can switch to Act to have it made.".into()));
                }
                let edit = name == "edit_file";
                let pane = self.pane;
                let r = self.on_ui(move || json!(change_file(HWND(pane as *mut _), &root, &input, edit).map_err(|e| format!("\u{1}{e}")).unwrap_or_else(|e| e)));
                let s = r.as_str().unwrap_or("").to_string();
                done(match s.strip_prefix('\u{1}') {
                    Some(e) => Err(e.to_string()),
                    None => Ok(s),
                })
            }
            "build" => {
                self.tell(Note::Status(t("Building...").into()));
                self.on_ui(prepare_build);
                done(cargo(&root, &["build", "--color", "never", "--message-format", "short"], &self.cancel))
            }
            "run_tests" => {
                self.tell(Note::Status(t("Running the tests...").into()));
                self.on_ui(prepare_build);
                let filter = input["filter"].as_str().unwrap_or("").to_string();
                let mut args = vec!["test", "--color", "never"];
                if !filter.is_empty() {
                    args.push(&filter);
                }
                done(cargo(&root, &args, &self.cancel))
            }
            "screenshot" => {
                self.tell(Note::Status(t("Building and running the program...").into()));
                self.on_ui(prepare_build);
                match screenshot(&root, &self.cancel) {
                    Ok(png) => json!({"type": "tool_result", "tool_use_id": id, "content": [
                        {"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": claude::base64(&png)}},
                        {"type": "text", "text": "The program's first window as it opened."}
                    ]}),
                    Err(e) => error_result(id, &e),
                }
            }
            other => error_result(id, &format!("There is no tool called {other}.")),
        }
    }
}

fn error_result(id: &str, message: &str) -> Value {
    json!({"type": "tool_result", "tool_use_id": id, "is_error": true, "content": message})
}

/// A path the model gave, inside the project - never outside it, never into target/ or .git/
/// when it is to be written.
pub fn inside(root: &Path, rel: &str, writing: bool) -> Result<PathBuf, String> {
    let rel = rel.trim().replace('\\', "/");
    if rel.is_empty() {
        return Err("No path given.".into());
    }
    let given = Path::new(&rel);
    let rest = if given.is_absolute() {
        given.strip_prefix(root).map_err(|_| format!("{rel} is outside the project."))?.to_path_buf()
    } else {
        given.to_path_buf()
    };
    let mut parts: Vec<String> = Vec::new();
    for c in rest.components() {
        match c {
            Component::Normal(p) => parts.push(p.to_string_lossy().to_string()),
            Component::ParentDir => {
                if parts.pop().is_none() {
                    return Err(format!("{rel} is outside the project."));
                }
            }
            Component::CurDir => {}
            _ => return Err(format!("{rel} is outside the project.")),
        }
    }
    if parts.is_empty() {
        return Err("That is the project folder itself, not a file.".into());
    }
    if writing && matches!(parts[0].to_lowercase().as_str(), "target" | ".git") {
        return Err(format!("{rel}: target/ and .git/ are not the project's own files."));
    }
    if writing && parts.last().is_some_and(|f| f.ends_with("_ui.rs")) {
        return Err(format!("{rel} is generated from its .phf - change the form file instead."));
    }
    Ok(parts.iter().fold(root.to_path_buf(), |p, s| p.join(s)))
}

fn skipped(name: &str) -> bool {
    matches!(name, "target" | ".git" | "node_modules" | "publish")
}

/// The project's files under `rel` ("" all), folders with a trailing `/`.
pub fn list_files(root: &Path, rel: &str) -> Result<String, String> {
    let start = if rel.trim().is_empty() || rel.trim() == "." { root.to_path_buf() } else { inside(root, rel, false)? };
    let mut out = Vec::new();
    let mut stack = vec![start.clone()];
    while let Some(d) = stack.pop() {
        let Ok(list) = std::fs::read_dir(&d) else { continue };
        let mut entries: Vec<_> = list.flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries.into_iter().rev() {
            let name = e.file_name().to_string_lossy().to_string();
            if skipped(&name) {
                continue;
            }
            let p = e.path();
            let shown = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().replace('\\', "/");
            if p.is_dir() {
                out.push(format!("{shown}/"));
                stack.push(p);
            } else {
                out.push(shown);
            }
        }
        if out.len() > 2000 {
            break;
        }
    }
    out.sort();
    if out.is_empty() {
        return Ok("(nothing there)".into());
    }
    Ok(out.join("\n"))
}

fn texty(p: &Path) -> bool {
    p.extension().and_then(|e| e.to_str()).is_some_and(|e| matches!(e.to_lowercase().as_str(), "rs" | "phf" | "phaiform" | "toml" | "css" | "md" | "txt" | "json" | "html" | "js" | "sql" | "csv" | "ron" | "yaml" | "yml"))
}

/// Lines of the project's text files that have `text`, ignoring case.
pub fn search(root: &Path, text: &str, rel: &str) -> Result<String, String> {
    if text.trim().is_empty() {
        return Err("Nothing to search for.".into());
    }
    let start = if rel.trim().is_empty() { root.to_path_buf() } else { inside(root, rel, false)? };
    let needle = text.to_lowercase();
    let mut out = Vec::new();
    let mut stack = vec![start];
    while let Some(d) = stack.pop() {
        let Ok(list) = std::fs::read_dir(&d) else { continue };
        for e in list.flatten() {
            let p = e.path();
            if skipped(&e.file_name().to_string_lossy()) {
                continue;
            }
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            if !texty(&p) || e.metadata().map(|m| m.len()).unwrap_or(0) > 2_000_000 {
                continue;
            }
            let Ok(body) = std::fs::read_to_string(&p) else { continue };
            let shown = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().replace('\\', "/");
            for (i, line) in body.lines().enumerate() {
                if line.to_lowercase().contains(&needle) {
                    out.push(format!("{shown}:{}: {}", i + 1, line.trim_end().chars().take(240).collect::<String>()));
                    if out.len() >= 200 {
                        out.sort();
                        return Ok(out.join("\n") + "\n(more not shown)");
                    }
                }
            }
        }
    }
    out.sort();
    Ok(if out.is_empty() { format!("No line has \"{text}\".") } else { out.join("\n") })
}

/// A file's text as the person sees it: the open editor's, unsaved changes and all, else the disk's.
fn current_text(path: &Path) -> Option<String> {
    studio().and_then(|s| s.open_text_of(path)).or_else(|| std::fs::read_to_string(path).ok()).map(|t| t.replace("\r\n", "\n"))
}

/// Lines of a file, numbered from 1.
pub fn numbered(text: &str, offset: usize, limit: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let from = offset.max(1) - 1;
    let to = (from + limit).min(lines.len());
    let mut out: String = (from..to).map(|i| format!("{:>5}\t{}\n", i + 1, lines[i])).collect();
    if to < lines.len() {
        out.push_str(&format!("(lines {}-{} of {}; more with offset {})\n", from + 1, to, lines.len(), to + 1));
    }
    if lines.is_empty() {
        out.push_str("(the file is empty)\n");
    }
    out
}

fn read_file(root: &Path, input: &Value) -> Result<String, String> {
    let p = inside(root, input["path"].as_str().unwrap_or(""), false)?;
    let text = current_text(&p).ok_or_else(|| format!("{} cannot be read - is it there? list_files shows what is.", input["path"].as_str().unwrap_or("")))?;
    let offset = input["offset"].as_u64().unwrap_or(1) as usize;
    let limit = input["limit"].as_u64().unwrap_or(1500).clamp(1, 5000) as usize;
    Ok(numbered(&text, offset, limit))
}

/// What a write or edit makes of a file's text, or why it cannot.
pub fn changed_text(old: Option<&str>, input: &Value, edit: bool) -> Result<String, String> {
    if !edit {
        return input["contents"].as_str().map(str::to_string).ok_or_else(|| "contents is missing.".into());
    }
    let old = old.ok_or("The file is not there - write_file makes a new one.")?;
    let (from, to) = (input["old_text"].as_str().unwrap_or(""), input["new_text"].as_str().unwrap_or(""));
    if from.is_empty() {
        return Err("old_text is empty.".into());
    }
    let n = old.matches(from).count();
    match n {
        0 => Err("old_text is not in the file as written - read_file it again and copy the lines exactly, without their numbers.".into()),
        1 => Ok(old.replacen(from, to, 1)),
        _ if input["replace_all"].as_bool() == Some(true) => Ok(old.replace(from, to)),
        _ => Err(format!("old_text is in the file {n} times - give more of the lines around it, or replace_all.")),
    }
}

/// A write or an edit, shown to the person and made if they accept - on the window's thread.
fn change_file(pane: HWND, root: &Path, input: &Value, edit: bool) -> Result<String, String> {
    let rel = input["path"].as_str().unwrap_or("").to_string();
    let p = inside(root, &rel, true)?;
    let old = current_text(&p);
    let new = changed_text(old.as_deref(), input, edit)?;
    if old.as_deref() == Some(new.as_str()) {
        return Ok(format!("{rel} already has that - nothing changed."));
    }
    // A form file Phai Studio could not open is not written at all.
    if phai_form::is_form(&p) {
        phai_form::FormFile::parse(&new).map_err(|e| format!("{rel} would not be a valid form file: {e}. Nothing was written; read phai_reference and fix it."))?;
    }
    let accept_all = state(pane).is_some_and(|s| s.accept_all);
    if !accept_all {
        match review(pane, &rel, old.as_deref().unwrap_or(""), &new) {
            Review::Accept => {}
            Review::AcceptAll => {
                if let Some(s) = state(pane) {
                    s.accept_all = true;
                }
            }
            Review::Decline => return Err(format!("The person declined this change to {rel}. Nothing was written.")),
        }
    }
    let crlf = std::fs::read(&p).map(|b| b.windows(2).any(|w| w == b"\r\n")).unwrap_or(false);
    let on_disk = if crlf { new.replace('\n', "\r\n") } else { new.clone() };
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(&p, on_disk).map_err(|e| format!("{rel} cannot be written: {e}"))?;
    if let Some(s) = studio() {
        s.file_changed_by_assistant(&p, &new);
    }
    let lines = new.lines().count();
    Ok(if old.is_none() { format!("Made {rel} ({lines} lines).") } else { format!("Changed {rel} ({lines} lines now).") })
}

/// Before cargo runs: the open files saved, the forms' code written.
fn prepare_build() -> Value {
    if let Some(s) = studio() {
        s.save_all_pub();
    }
    Value::Null
}

/// Runs cargo in the project; what it said, cut to the end that matters.
fn cargo(root: &Path, args: &[&str], cancel: &claude::Cancel) -> Result<String, String> {
    use std::os::windows::process::CommandExt;
    let exe = crate::cargo::find_cargo().ok_or("cargo was not found - is Rust installed?")?;
    let child = std::process::Command::new(exe)
        .args(args)
        .current_dir(root)
        .env("CARGO_TERM_COLOR", "never")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .creation_flags(0x0800_0000)
        .spawn()
        .map_err(|e| e.to_string())?;
    let pid = child.id();
    let out = std::thread::scope(|sc| {
        let waiter = sc.spawn(|| child.wait_with_output());
        while !waiter.is_finished() {
            if cancel.stopped() {
                let _ = std::process::Command::new("taskkill").args(["/T", "/F", "/PID", &pid.to_string()]).creation_flags(0x0800_0000).output();
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        waiter.join().ok()
    });
    let out = out.and_then(|r| r.ok()).ok_or("cargo did not finish")?;
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    // The compiler's own lines; the list of crates it compiled says nothing.
    let kept: Vec<&str> = text.lines().filter(|l| !l.trim_start().starts_with("Compiling ") && !l.trim_start().starts_with("Downloaded ") && !l.trim_start().starts_with("Downloading ")).collect();
    let tail: Vec<&str> = kept.iter().rev().take(200).rev().copied().collect();
    let said = tail.join("\n");
    if out.status.success() {
        Ok(format!("cargo {} succeeded.\n{said}", args[0]))
    } else {
        Err(format!("cargo {} failed.\n{said}", args[0]))
    }
}

/// The project's package name from its Cargo.toml.
fn package_name(root: &Path) -> Option<String> {
    let toml = std::fs::read_to_string(root.join("Cargo.toml")).ok()?;
    let mut in_package = false;
    for line in toml.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            in_package = l == "[package]";
        } else if in_package {
            if let Some(v) = l.strip_prefix("name").map(str::trim).and_then(|r| r.strip_prefix('=')) {
                return Some(v.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

/// Builds and runs the program, photographed as its first window opens (`PHAI_SHOT`).
fn screenshot(root: &Path, cancel: &claude::Cancel) -> Result<Vec<u8>, String> {
    use std::os::windows::process::CommandExt;
    cargo(root, &["build", "--color", "never", "--message-format", "short"], cancel)?;
    let name = package_name(root).ok_or("Cargo.toml has no package name")?;
    let exe = root.join("target").join("debug").join(format!("{name}.exe"));
    let shot = std::env::temp_dir().join(format!("phai-ai-shot-{}.png", std::process::id()));
    let _ = std::fs::remove_file(&shot);
    let mut child = std::process::Command::new(&exe).current_dir(root).env("PHAI_SHOT", &shot).creation_flags(0x0800_0000).spawn().map_err(|e| format!("{} cannot run: {e}", exe.display()))?;
    for _ in 0..300 {
        if child.try_wait().ok().flatten().is_some() || cancel.stopped() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let _ = child.kill();
    let png = std::fs::read(&shot).map_err(|_| "The program did not open a window to photograph within 30 seconds.".to_string())?;
    let _ = std::fs::remove_file(&shot);
    Ok(png)
}

/// What the person said to a change.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Review {
    Accept,
    AcceptAll,
    Decline,
}

struct ReviewWin {
    left: HWND,
    right: HWND,
    accept: HWND,
    all: HWND,
    decline: HWND,
    answer: Option<Review>,
}

const ID_ACCEPT: i32 = 1;
const ID_DECLINE: i32 = 2;
const ID_ALL: i32 = 3;

/// The change side by side - the file as it is, as it would be, what differs marked - with
/// Accept, Accept All and Decline. Waits for the answer.
fn review(pane: HWND, rel: &str, old: &str, new: &str) -> Review {
    if let Ok(a) = std::env::var("PHAI_AI_REVIEW") {
        return match a.as_str() {
            "accept" => Review::Accept,
            "all" => Review::AcceptAll,
            _ => Review::Decline,
        };
    }
    let owner = studio().map(|s| s.hwnd).unwrap_or(pane);
    win::init();
    let class = win::register("PhaiAIReview", Some(review_proc), None);
    let title = win::wide(&tf("Claude wants to change {0}", &[&rel]));
    unsafe {
        let hwnd = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_OVERLAPPEDWINDOW | wm::WS_CLIPCHILDREN, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let left = phai_editor::view::create(hwnd, 10, old, rel);
        let right = phai_editor::view::create(hwnd, 11, new, rel);
        let (l, r) = phai_editor::tools::diff_lines(old, new);
        phai_editor::view::set_changes(left, &l);
        phai_editor::view::set_changes(right, &r);
        let accept = win::button(hwnd, t("Accept"), ID_ACCEPT, font, true);
        let all = win::button(hwnd, t("Accept all in this chat"), ID_ALL, font, false);
        let decline = win::button(hwnd, t("Decline"), ID_DECLINE, font, false);
        win::set_state(hwnd, Box::new(ReviewWin { left, right, accept, all, decline, answer: None }));
        let mut o = RECT::default();
        let _ = wm::GetWindowRect(owner, &mut o);
        let (w, h) = (((o.right - o.left) * 8 / 10).max(win::px(hwnd, 800)), ((o.bottom - o.top) * 8 / 10).max(win::px(hwnd, 500)));
        let _ = wm::SetWindowPos(hwnd, None, o.left + (o.right - o.left - w) / 2, o.top + (o.bottom - o.top - h) / 2, w, h, wm::SWP_NOZORDER);
        win::theme_frame(hwnd);
        win::enable(owner, false);
        win::show_top(hwnd);
        // `PHAI_AI_REVIEW_SHOT=<png>`: photographed and declined, for the guide.
        if let Some(to) = std::env::var_os("PHAI_AI_REVIEW_SHOT") {
            let mut msg = wm::MSG::default();
            for _ in 0..30 {
                while wm::PeekMessageW(&mut msg, None, 0, 0, wm::PM_REMOVE).as_bool() {
                    let _ = wm::TranslateMessage(&msg);
                    wm::DispatchMessageW(&msg);
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            let _ = phai_ui::shot::save(hwnd, Path::new(&to));
            if let Some(s) = win::state::<ReviewWin>(hwnd) {
                s.answer = Some(Review::Decline);
            }
        }
        let mut msg = wm::MSG::default();
        while win::state::<ReviewWin>(hwnd).is_some_and(|s| s.answer.is_none()) {
            if !wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
                wm::PostQuitMessage(msg.wParam.0 as i32);
                break;
            }
            let _ = wm::TranslateMessage(&msg);
            wm::DispatchMessageW(&msg);
        }
        let answer = win::state::<ReviewWin>(hwnd).and_then(|s| s.answer).unwrap_or(Review::Decline);
        win::enable(owner, true);
        let _ = wm::SetForegroundWindow(owner);
        let _ = wm::DestroyWindow(hwnd);
        answer
    }
}

extern "system" fn review_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(s) = (unsafe { win::state::<ReviewWin>(h) }) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            let (cw, ch) = (win::loword(l.0 as usize) as i32, win::hiword(l.0 as usize) as i32);
            let bar = win::px(h, 44);
            let half = cw / 2;
            win::move_to(s.left, 0, 0, half - 2, ch - bar);
            win::move_to(s.right, half + 2, 0, cw - half - 2, ch - bar);
            let (bw, bh, m) = (win::px(h, 110), win::px(h, 30), win::px(h, 8));
            let wide = win::px(h, 190);
            win::move_to(s.decline, cw - m - bw, ch - bar + m - 1, bw, bh);
            win::move_to(s.all, cw - m * 2 - bw - wide, ch - bar + m - 1, wide, bh);
            win::move_to(s.accept, cw - m * 3 - bw * 2 - wide, ch - bar + m - 1, bw, bh);
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            s.answer = match win::loword(w.0) as i32 {
                ID_ACCEPT => Some(Review::Accept),
                ID_ALL => Some(Review::AcceptAll),
                ID_DECLINE => Some(Review::Decline),
                _ => s.answer,
            };
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            s.answer = Some(Review::Decline);
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            drop(unsafe { win::take_state::<ReviewWin>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::ctl_color(msg, w, l).unwrap_or_else(|| win::def(h, msg, w, l)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_stay_inside_the_project() {
        let root = Path::new(r"C:\work\shop");
        assert_eq!(inside(root, "src/main.rs", true).unwrap(), root.join("src").join("main.rs"));
        assert_eq!(inside(root, "./src/../Cargo.toml", false).unwrap(), root.join("Cargo.toml"));
        assert_eq!(inside(root, r"C:\work\shop\src\a.rs", true).unwrap(), root.join("src").join("a.rs"));
        assert!(inside(root, "../other/x.rs", false).is_err());
        assert!(inside(root, r"C:\Windows\win.ini", false).is_err());
        assert!(inside(root, "target/debug/x", true).is_err());
        assert!(inside(root, "target/debug/x", false).is_ok(), "reading is fine");
        assert!(inside(root, ".git/config", true).is_err());
        assert!(inside(root, "src/main_window_ui.rs", true).is_err(), "generated");
        assert!(inside(root, "", false).is_err());
    }

    #[test]
    fn edits_replace_exactly_once_or_say_why_not() {
        let e = |from: &str, to: &str| json!({"old_text": from, "new_text": to});
        assert_eq!(changed_text(Some("a b a"), &e("b", "c"), true).unwrap(), "a c a");
        assert!(changed_text(Some("a b a"), &e("a", "c"), true).unwrap_err().contains("2 times"));
        assert_eq!(changed_text(Some("a b a"), &json!({"old_text": "a", "new_text": "c", "replace_all": true}), true).unwrap(), "c b c");
        assert!(changed_text(Some("a b"), &e("x", "c"), true).is_err());
        assert!(changed_text(None, &e("x", "c"), true).is_err());
        assert_eq!(changed_text(None, &json!({"contents": "new"}), false).unwrap(), "new");
    }

    #[test]
    fn files_are_numbered_listed_and_searched() {
        assert_eq!(numbered("a\nb\nc", 2, 1), "    2\tb\n(lines 2-2 of 3; more with offset 3)\n");
        assert_eq!(numbered("a\nb", 1, 10), "    1\ta\n    2\tb\n");
        let dir = std::env::temp_dir().join(format!("phai-ai-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::create_dir_all(dir.join("target")).unwrap();
        std::fs::write(dir.join("src").join("main.rs"), "fn main() {\n    println!(\"Hello\");\n}\n").unwrap();
        std::fs::write(dir.join("target").join("junk.rs"), "Hello").unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"shop\"\n").unwrap();
        assert_eq!(list_files(&dir, "").unwrap(), "Cargo.toml\nsrc/\nsrc/main.rs");
        assert_eq!(search(&dir, "hello", "").unwrap(), "src/main.rs:2:     println!(\"Hello\");");
        assert_eq!(package_name(&dir).as_deref(), Some("shop"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_tools_are_well_formed() {
        let tools = tools();
        let names: Vec<&str> = tools.as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert_eq!(names, ["list_files", "read_file", "search", "write_file", "edit_file", "build", "run_tests", "screenshot", "phai_reference"]);
        for t in tools.as_array().unwrap() {
            assert_eq!(t["input_schema"]["type"], "object");
            assert!(t["description"].as_str().unwrap().len() > 30);
        }
        assert!(system_prompt(Path::new(r"C:\x\shop")).contains("\"shop\""));
    }
}
