//! The AI assistant: a pane beside Project and Properties where GitHub Copilot answers questions
//! about the project and - in Act mode - changes it. It is the agent of GitHub's Copilot Language
//! Server, spoken to over ACP (`acp.rs`) with the person's GitHub sign-in; its models (GPT, Claude,
//! Gemini... whatever the account offers) are in the model box. The agent reads the project and
//! uses its own tools; each change it wants to make to a file is shown side by side to accept or
//! not, anything else it wants to run is asked first, and in Ask mode it changes nothing.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use phai_ui::text::{t, tf};
use phai_ui::win;
use serde_json::{json, Value};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::HFONT;
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::acp;
use crate::copilot;
use crate::explorer;
use crate::studio;

const ID_TRANSCRIPT: i32 = 10;
const ID_INPUT: i32 = 11;
const ID_SEND: i32 = 12;
const ID_MODE: i32 = 13;
const ID_MODEL: i32 = 14;
const ID_NEW: i32 = 15;
const ID_SIGN_IN: i32 = 16;
const ID_STATUS: i32 = 17;

/// A change of Copilot's shown side by side (its window waits for the answer, messages still
/// coming): the agent's next requests wait for it, here - outside the pane, which the window's
/// wait reaches again.
static REVIEWING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static QUEUED: std::sync::Mutex<Vec<Value>> = std::sync::Mutex::new(Vec::new());

fn reviewing() -> bool {
    REVIEWING.load(std::sync::atomic::Ordering::SeqCst)
}

fn set_reviewing(on: bool) {
    REVIEWING.store(on, std::sync::atomic::Ordering::SeqCst);
}

fn take_queued() -> Vec<Value> {
    std::mem::take(&mut *QUEUED.lock().unwrap_or_else(|e| e.into_inner()))
}

/// What an answer of Copilot's agent is to.
#[derive(Clone, Debug)]
enum AcpAsk {
    Init,
    NewSession,
    SetModel(String),
    Prompt,
}

struct Pane {
    transcript: HWND,
    input: HWND,
    send: HWND,
    mode: HWND,
    model: HWND,
    new_chat: HWND,
    sign_in: HWND,
    status: HWND,
    /// The project the conversation is about.
    root: Option<PathBuf>,
    /// An answer is coming: Send is Stop.
    busy: bool,
    /// Accept All was chosen: the rest of this conversation's changes go in without asking.
    accept_all: bool,
    /// The last block shown: a new kind starts a new paragraph.
    last_block: String,
    /// The models the model box offers: (id, name).
    cmodels: Vec<(String, String)>,
    acp: Option<acp::Acp>,
    acp_ready: bool,
    acp_asked: HashMap<i64, AcpAsk>,
    session: Option<String>,
    /// The model the session was last set to.
    session_model: Option<String>,
    /// What is to be sent once the agent and its session are ready.
    waiting: Option<Value>,
    /// The agent's tool calls, by id: their kind and the files they name.
    calls: HashMap<String, (String, Vec<String>)>,
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
        let model = win::combo(h, ID_MODEL, font, &[]);
        let act = crate::setting_pub("ai_mode").as_deref() == Some("act");
        let _ = wm::SendMessageW(mode, 0x014E /* CB_SETCURSEL */, Some(WPARAM(act as usize)), None);
        let new_chat = win::button(h, t("New chat"), ID_NEW, font, false);
        let sign_in = win::button(h, t("Sign in..."), ID_SIGN_IN, font, false);
        let status = win::label(h, "", ID_STATUS, font);
        win::set_state(
            h,
            Box::new(Pane {
                transcript,
                input,
                send,
                mode,
                model,
                new_chat,
                sign_in,
                status,
                root: None,
                busy: false,
                accept_all: false,
                last_block: String::new(),
                cmodels: vec![],
                acp: None,
                acp_ready: false,
                acp_asked: HashMap::new(),
                session: None,
                session_model: None,
                waiting: None,
                calls: HashMap::new(),
            }),
        );
        let p = state(h).unwrap();
        p.fill_models(h);
        p.greet();
        // `PHAI_AI_ASK=<words>`: asked by itself a moment after Studio opens - for tests and the
        // guide's picture, with `PHAI_COPILOT_SERVER=fake` standing in for Copilot.
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
        let words = if !crate::copilot_on_pub() {
            t("GitHub Copilot is off: turn it on in Tools > Options > GitHub Copilot, then sign in with your GitHub account (Copilot Free is enough).")
        } else {
            t("Ask GitHub Copilot about your project, or switch to Act and say what to change. Copilot reads the files it needs; in Act mode each change is shown to you first. What it uses counts against your Copilot plan.")
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
        // Below: the box to write in, Send, New chat and Sign in beside it.
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
        win::move_to(self.sign_in, w - m - bw, y + (row + px(4)) * 2, bw, row);
    }

    fn act(&self) -> bool {
        win::combo_selected(self.mode) == 1
    }

    /// The model chosen in the model box, if the account has offered any.
    fn copilot_model(&self) -> Option<String> {
        self.cmodels.get(win::combo_selected(self.model).max(0) as usize).map(|(id, _)| id.clone())
    }

    /// The model box filled with the models Copilot offers - asked of the server when not yet known.
    fn fill_models(&mut self, h: HWND) {
        let saved = crate::setting_pub("copilot_model");
        let chosen = saved.and_then(|m| self.cmodels.iter().position(|(id, _)| *id == m)).unwrap_or(0);
        unsafe {
            let _ = wm::SendMessageW(self.model, 0x014B /* CB_RESETCONTENT */, None, None);
            for (_, n) in &self.cmodels {
                let w = win::wide(n);
                let _ = wm::SendMessageW(self.model, 0x0143 /* CB_ADDSTRING */, None, Some(LPARAM(w.as_ptr() as isize)));
            }
            let _ = wm::SendMessageW(self.model, 0x014E /* CB_SETCURSEL */, Some(WPARAM(chosen)), None);
        }
        if self.cmodels.is_empty() && crate::copilot_on_pub() {
            if let Some(s) = studio() {
                s.copilot_models_for(h);
            }
        }
    }

    fn new_chat(&mut self) {
        if std::mem::take(&mut self.busy) {
            if let (Some(a), Some(sid)) = (self.acp.as_ref(), self.session.as_ref()) {
                a.notify("session/cancel", json!({ "sessionId": sid }));
            }
        }
        // The next words start a session of their own.
        self.session = None;
        self.session_model = None;
        self.waiting = None;
        self.calls.clear();
        self.root = None;
        self.accept_all = false;
        self.last_block.clear();
        win::set_text(self.send, t("Send"));
        win::set_text(self.status, "");
        self.greet();
    }

    fn send(&mut self, h: HWND) {
        if self.busy {
            win::set_text(self.status, t("Stopping..."));
            match (self.acp.as_ref(), self.session.as_ref()) {
                (Some(a), Some(sid)) => a.notify("session/cancel", json!({ "sessionId": sid })),
                _ => self.copilot_done(Some(t("Stopped").to_string())),
            }
            return;
        }
        let words = win::text_of(self.input).trim().to_string();
        if !words.is_empty() {
            self.send_copilot(h, words);
        }
    }
}

// --- GitHub Copilot's agent ---------------------------------------------------------------------

impl Pane {
    /// Words for Copilot's agent: sent once it is running and has a session for the project.
    fn send_copilot(&mut self, h: HWND, words: String) {
        if !crate::copilot_on_pub() {
            win::set_text(self.status, t("Turn on GitHub Copilot in Tools > Options first."));
            return;
        }
        let Some(root) = studio().and_then(|s| s.project_dir_pub()) else {
            win::set_text(self.status, t("Open a project first: Copilot works in the project of the open file."));
            return;
        };
        if self.root.as_deref() != Some(root.as_path()) {
            if self.session.is_some() {
                self.new_chat();
            }
            self.root = Some(root.clone());
            // Another folder: an agent started there.
            self.acp = None;
        }
        let act = self.act();
        crate::remember_setting_pub("ai_mode", if act { "act" } else { "ask" });
        if let Some(m) = self.copilot_model() {
            crate::remember_setting_pub("copilot_model", &m);
        }
        // The agent reads the files as they are on disk: the open ones saved first in Act mode,
        // so what it changes is what is shown here.
        if act {
            if let Some(s) = studio() {
                s.save_all_pub();
            }
        }
        let mode = if act { "Act: you may change the project's files; each change is shown to the person to accept." } else { "Ask: answer only - do not change, create or delete any file, and run no commands." };
        let mut prompt = vec![json!({"type": "text", "text": words}), json!({"type": "text", "text": format!("<studio>Mode: {mode}</studio>")})];
        // The first words of a session: where it is, and the form files' reference.
        if self.session.is_none() {
            prompt.insert(0, json!({"type": "text", "text": format!("<studio>{}</studio>", copilot_context(&root))}));
            prompt.push(json!({"type": "resource", "resource": {"uri": "phai://reference/phaiform.md", "mimeType": "text/markdown", "text": include_str!("../../../docs/phaiform.md")}}));
        }
        self.waiting = Some(Value::Array(prompt));
        win::set_text(self.input, "");
        self.append(&format!("{} {words}\n\n", t("You:")));
        self.last_block.clear();
        self.busy = true;
        win::set_text(self.send, t("Stop"));
        win::set_text(self.status, t("Copilot is thinking..."));
        self.acp_go(h);
    }

    /// The next step towards sending what waits: the agent started, initialized, a session made,
    /// its model set - then the words sent.
    fn acp_go(&mut self, h: HWND) {
        let Some(root) = self.root.clone() else { return };
        if !self.acp.as_mut().is_some_and(|a| a.alive()) {
            self.acp_ready = false;
            self.session = None;
            let Some(launch) = copilot::find() else {
                return self.copilot_done(Some(t("GitHub Copilot's server is not here yet - turn Copilot on in Tools > Options to get it.").to_string()));
            };
            let Some(mut a) = acp::Acp::start(copilot::command(&launch, true, &root), h) else {
                return self.copilot_done(Some(t("Copilot's agent could not be started.").to_string()));
            };
            let id = a.request("initialize", json!({
                "protocolVersion": 1,
                "clientCapabilities": { "fs": { "readTextFile": true, "writeTextFile": true }, "terminal": false },
                "clientInfo": { "name": "phai-studio", "title": "Phai Studio", "version": env!("CARGO_PKG_VERSION") }
            }));
            self.acp_asked.insert(id, AcpAsk::Init);
            self.acp = Some(a);
            return;
        }
        if !self.acp_ready {
            return;
        }
        let model = self.copilot_model().filter(|m| self.session_model.as_deref() != Some(m.as_str()));
        let Some(a) = self.acp.as_mut() else { return };
        let Some(sid) = self.session.clone() else {
            let id = a.request("session/new", json!({ "cwd": root.to_string_lossy(), "mcpServers": [] }));
            self.acp_asked.insert(id, AcpAsk::NewSession);
            return;
        };
        if let Some(m) = model {
            let id = a.request("session/set_model", json!({ "sessionId": sid, "modelId": m }));
            self.acp_asked.insert(id, AcpAsk::SetModel(m.clone()));
            self.session_model = Some(m);
        }
        if let Some(prompt) = self.waiting.take() {
            let id = a.request("session/prompt", json!({ "sessionId": sid, "prompt": prompt }));
            self.acp_asked.insert(id, AcpAsk::Prompt);
        }
    }

    /// The answer is over.
    fn copilot_done(&mut self, err: Option<String>) {
        self.busy = false;
        self.waiting = None;
        win::set_text(self.send, t("Send"));
        win::set_text(self.status, &err.clone().unwrap_or_default());
        if let Some(e) = err {
            self.append(&format!("\n[{e}]\n"));
        }
        self.append("\n\n");
        self.last_block.clear();
    }

    /// A paragraph of `kind` (text, thinking) begins, under "Copilot:" if it is the first.
    fn block(&mut self, kind: &str) {
        if kind != self.last_block {
            if self.last_block.is_empty() {
                self.append(&format!("{}\n", t("Copilot:")));
            } else {
                self.append("\n");
            }
            if kind == "thinking" {
                self.append("› ");
            }
            self.last_block = kind.to_string();
        }
    }

    /// A message from Copilot's agent.
    fn acp_message(&mut self, h: HWND, msg: Value) {
        if msg["phaiGone"] == true {
            self.acp = None;
            self.acp_ready = false;
            self.session = None;
            if self.busy {
                self.copilot_done(Some(t("Copilot's agent ended.").to_string()));
            }
            return;
        }
        // An answer of ours.
        if let (Some(id), None) = (msg.get("id").and_then(Value::as_i64), msg.get("method")) {
            let Some(ask) = self.acp_asked.remove(&id) else { return };
            let error = msg["error"]["message"].as_str().map(str::to_string);
            let result = msg.get("result").cloned().unwrap_or(Value::Null);
            match ask {
                AcpAsk::Init => match error {
                    Some(e) => self.copilot_done(Some(e)),
                    None => {
                        self.acp_ready = true;
                        self.acp_go(h);
                    }
                },
                AcpAsk::NewSession => match error {
                    Some(e) => self.copilot_done(Some(plain_agent_error(&e))),
                    None => {
                        self.session = result["sessionId"].as_str().map(str::to_string);
                        self.session_model = result["models"]["currentModelId"].as_str().map(str::to_string);
                        // The agent's own list of models, where it gives one, is the one to choose from.
                        let list: Vec<(String, String)> = result["models"]["availableModels"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .filter_map(|m| Some((m["modelId"].as_str()?.to_string(), m["name"].as_str().or(m["modelId"].as_str())?.to_string())))
                            .collect();
                        if !list.is_empty() && list != self.cmodels {
                            let chosen = self.copilot_model();
                            self.cmodels = list;
                            self.fill_models(h);
                            if let Some(i) = chosen.and_then(|c| self.cmodels.iter().position(|(id, _)| *id == c)) {
                                unsafe {
                                    let _ = wm::SendMessageW(self.model, 0x014E, Some(WPARAM(i)), None);
                                }
                            }
                        }
                        if self.session.is_none() {
                            return self.copilot_done(Some(t("Copilot's agent gave no session.").to_string()));
                        }
                        self.acp_go(h);
                    }
                },
                // Not taken: asked again with the next words.
                AcpAsk::SetModel(m) => {
                    if error.is_some() && self.session_model.as_deref() == Some(m.as_str()) {
                        self.session_model = None;
                    }
                }
                AcpAsk::Prompt => {
                    let ended = match (error, result["stopReason"].as_str()) {
                        (Some(e), _) => Some(plain_agent_error(&e)),
                        (None, Some("cancelled")) => Some(t("Stopped").to_string()),
                        (None, Some("refusal")) => Some(t("Copilot declined this request.").to_string()),
                        (None, Some("max_tokens")) => Some(t("The answer was cut off - it was too long.").to_string()),
                        (None, Some("max_turn_requests")) => Some(t("Copilot stopped after many steps - say go on to continue.").to_string()),
                        _ => None,
                    };
                    self.copilot_done(ended);
                }
            }
            return;
        }
        let method = msg["method"].as_str().unwrap_or("").to_string();
        if msg.get("id").is_some() {
            return self.acp_request(h, msg);
        }
        if method == "session/update" && msg["params"]["sessionId"].as_str() == self.session.as_deref() {
            self.acp_update(&msg["params"]["update"]);
        }
    }

    /// What the agent says as it works: its words, its thinking, the tools it uses, its plan.
    fn acp_update(&mut self, u: &Value) {
        let text = || u["content"]["text"].as_str().unwrap_or("").to_string();
        match u["sessionUpdate"].as_str().unwrap_or("") {
            "agent_message_chunk" => {
                self.block("text");
                win::set_text(self.status, t("Copilot is writing..."));
                self.append(&text());
            }
            "agent_thought_chunk" => {
                self.block("thinking");
                self.append(&text());
            }
            "tool_call" => {
                let id = u["toolCallId"].as_str().unwrap_or("").to_string();
                let kind = u["kind"].as_str().unwrap_or("other").to_string();
                let files: Vec<String> = u["locations"].as_array().into_iter().flatten().filter_map(|l| l["path"].as_str().map(str::to_string)).collect();
                let title = u["title"].as_str().unwrap_or(&kind).to_string();
                if self.last_block.is_empty() {
                    self.append(&format!("{}\n", t("Copilot:")));
                } else if self.last_block != "tool" {
                    self.append("\n");
                }
                self.append(&format!("⚙ {title}\n"));
                self.last_block = "tool".into();
                win::set_text(self.status, &format!("Copilot: {title}..."));
                self.calls.insert(id, (kind, files));
            }
            "tool_call_update" => {
                let id = u["toolCallId"].as_str().unwrap_or("").to_string();
                if u["status"] == "failed" {
                    self.append(&format!("⚙ {}\n", t("(did not go)")));
                    self.last_block = "tool".into();
                }
                if u["status"] != "completed" {
                    return;
                }
                // A file it changed: the editor showing it takes the file as it now is on disk -
                // the agent writes it itself, and its diff's `newText` may be only the part changed.
                let mut changed: Vec<PathBuf> = u["content"].as_array().into_iter().flatten().filter(|c| c["type"] == "diff").filter_map(|c| Some(PathBuf::from(c["path"].as_str()?))).collect();
                if let Some((_, files)) = self.calls.get(&id).filter(|(k, _)| k == "edit") {
                    changed.extend(files.iter().map(PathBuf::from));
                }
                changed.sort();
                changed.dedup();
                let root = self.root.clone().unwrap_or_default();
                for path in changed {
                    if !lsp_same_start(&path, &root) {
                        continue;
                    }
                    let Some(text) = std::fs::read_to_string(&path).ok().map(|t| t.replace("\r\n", "\n")) else { continue };
                    if let Some(s) = studio() {
                        s.file_changed_by_assistant(&path, &text);
                    }
                }
            }
            "plan" => {
                let items: Vec<String> = u["entries"].as_array().into_iter().flatten().map(|e| format!("{} {}", if e["status"] == "completed" { "☑" } else { "☐" }, e["content"].as_str().unwrap_or(""))).collect();
                if !items.is_empty() {
                    self.append(&format!("\n{}\n", items.join("\n")));
                    self.last_block = "tool".into();
                }
            }
            _ => {}
        }
    }

    /// The agent asking something of Studio: leave to use a tool, a file's text, a file written.
    fn acp_request(&mut self, h: HWND, msg: Value) {
        let id = msg["id"].clone();
        let p = msg["params"].clone();
        let act = self.act();
        let root = self.root.clone().unwrap_or_default();
        match msg["method"].as_str().unwrap_or("") {
            "session/request_permission" => {
                // One change shown at a time: the next waits for it.
                if reviewing() {
                    QUEUED.lock().unwrap_or_else(|e| e.into_inner()).push(msg);
                    return;
                }
                let tc = &p["toolCall"];
                let kind = tc["kind"].as_str().unwrap_or("other");
                let title = tc["title"].as_str().unwrap_or(kind).to_string();
                let allow = match kind {
                    "read" | "search" | "think" | "fetch" => true,
                    "edit" | "delete" | "move" if !act => {
                        win::set_text(self.status, t("Ask mode: Copilot's change was not made - switch to Act to have it made."));
                        false
                    }
                    "edit" => self.review_edit(h, tc),
                    _ if !act => false,
                    _ => {
                        let owner = studio().map(|s| s.hwnd).unwrap_or(h);
                        let what = tc["rawInput"]["command"].as_str().map(|c| format!("\n\n{c}")).unwrap_or_default();
                        std::env::var_os("PHAI_AI_REVIEW").is_some_and(|a| a == "accept" || a == "all")
                            || win::message(Some(owner), &format!("{}{what}", tf("Copilot wants to: {0}", &[&title])), "GitHub Copilot", wm::MB_YESNO | wm::MB_ICONQUESTION) == wm::IDYES
                    }
                };
                if let Some(a) = self.acp.as_ref() {
                    a.answer(&id, acp::choose(&p["options"], allow));
                }
                // What waited for this one.
                if !reviewing() {
                    for q in take_queued() {
                        self.acp_request(h, q);
                    }
                }
            }
            "fs/read_text_file" => {
                let path = p["path"].as_str().unwrap_or("");
                let r = inside(&root, path, false).and_then(|f| current_text(&f).ok_or_else(|| format!("{path} cannot be read")));
                if let Some(a) = self.acp.as_ref() {
                    match r {
                        Ok(text) => {
                            let from = p["line"].as_u64().unwrap_or(1).max(1) as usize - 1;
                            let limit = p["limit"].as_u64().map(|n| n as usize).unwrap_or(usize::MAX);
                            let content = if from == 0 && limit == usize::MAX { text } else { text.split('\n').skip(from).take(limit).collect::<Vec<_>>().join("\n") };
                            a.answer(&id, json!({ "content": content }));
                        }
                        Err(e) => a.refuse(&id, -32002, &e),
                    }
                }
            }
            "fs/write_text_file" => {
                let path = p["path"].as_str().unwrap_or("").to_string();
                let r = if !act {
                    Err("Ask mode: no file is changed.".to_string())
                } else {
                    let rel = inside(&root, &path, true).map(|f| f.strip_prefix(&root).map(|r| r.to_string_lossy().replace('\\', "/")).unwrap_or(path.clone()));
                    rel.and_then(|rel| {
                        set_reviewing(true);
                        let r = change_file(h, &root, &json!({ "path": rel, "contents": p["content"] }), false);
                        set_reviewing(false);
                        r
                    })
                };
                if let Some(a) = self.acp.as_ref() {
                    match r {
                        Ok(_) => a.answer(&id, json!({})),
                        Err(e) => a.refuse(&id, -32003, &e),
                    }
                }
                for q in take_queued() {
                    self.acp_request(h, q);
                }
            }
            _ => {
                if let Some(a) = self.acp.as_ref() {
                    a.refuse(&id, -32601, "Phai Studio does not offer that.");
                }
            }
        }
    }

    /// A change the agent wants to make, shown side by side: whether the person let it.
    fn review_edit(&mut self, h: HWND, tc: &Value) -> bool {
        let root = self.root.clone().unwrap_or_default();
        let file = tc["rawInput"]["fileName"].as_str().or(tc["rawInput"]["path"].as_str()).or(tc["locations"][0]["path"].as_str()).unwrap_or("").to_string();
        let path = match inside(&root, &file, true) {
            Ok(p) => p,
            Err(e) => {
                win::set_text(self.status, &tf("Copilot's change was refused: {0}", &[&e]));
                return false;
            }
        };
        if self.accept_all {
            return true;
        }
        let rel = path.strip_prefix(&root).map(|r| r.to_string_lossy().replace('\\', "/")).unwrap_or(file.clone());
        let old = std::fs::read_to_string(&path).unwrap_or_default().replace("\r\n", "\n");
        let new = match tc["rawInput"]["diff"].as_str() {
            Some(d) => acp::apply_unified(&old, d).unwrap_or_else(|| d.to_string()),
            None => tc["rawInput"]["content"].as_str().or(tc["rawInput"]["newText"].as_str()).map(str::to_string).unwrap_or_else(|| old.clone()),
        };
        set_reviewing(true);
        let r = review(h, &tf("Copilot wants to change {0}", &[&rel]), &rel, &old, &new);
        set_reviewing(false);
        match r {
            Review::Accept => true,
            Review::AcceptAll => {
                self.accept_all = true;
                true
            }
            Review::Decline => false,
        }
    }
}

/// Whether `path` is under `root`, Windows' way (case does not matter).
fn lsp_same_start(path: &Path, root: &Path) -> bool {
    let low = |p: &Path| p.to_string_lossy().replace('/', "\\").to_lowercase();
    low(path).starts_with(&low(root))
}

/// An error of the agent's, in words a person can act on.
fn plain_agent_error(e: &str) -> String {
    let l = e.to_lowercase();
    if l.contains("auth") || l.contains("sign in") || l.contains("not signed") || l.contains("token") {
        t("Sign in to GitHub Copilot first: Sign in... below, or Copilot on the status bar.").to_string()
    } else if l.contains("quota") || l.contains("limit") || l.contains("exceeded") {
        t("This month's Copilot use is spent - it comes back next month, or with a paid plan.").to_string()
    } else {
        e.to_string()
    }
}

/// What Copilot's agent is told of where it is, once a session.
fn copilot_context(root: &Path) -> String {
    let name = root.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    format!(
        "You are working inside Phai Studio, a RAD IDE for Windows programs written in Rust, on the project \"{name}\". \
A Phai Studio project: Cargo.toml; src/main.rs; each window is a form - `<name>.phf` (TOML the visual designer reads and writes: \
controls, their bounds, properties, events - see the attached reference before changing one), `<name>_ui.rs` (generated from the .phf - \
never edit it), and `<name>.rs` (the person's code: `impl <Name>Events for <Name>` with a method per event). The controls are phai-ui's \
(a WinForms-like library on Win32); `ui.<id>` reaches a control. src/text.rs holds the Thai for every English sentence the program shows. \
Each change you make is shown to the person to accept; a declined change is their decision. Answer in the language the person writes in - often Thai."
    )
}

/// The models Copilot's server offers, for the pane `pane`; `note` said when there are none.
pub fn copilot_models(pane: HWND, list: Vec<(String, String)>, note: Option<&str>) {
    let Some(p) = state(pane) else { return };
    if !list.is_empty() {
        p.cmodels = list;
    }
    p.fill_models(pane);
    if let Some(n) = note {
        win::set_text(p.status, n);
    }
}

/// Copilot was turned on or off, or signed in: the pane says so.
pub fn copilot_changed(pane: HWND) {
    if let Some(p) = state(pane) {
        if !p.busy && p.session.is_none() {
            p.greet();
            if p.cmodels.is_empty() {
                p.fill_models(pane);
            }
        }
        if !crate::copilot_on_pub() {
            p.acp = None;
            p.acp_ready = false;
            p.session = None;
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
        acp::WM_ACP => {
            let v = unsafe { Box::from_raw(l.0 as *mut Value) };
            if let Some(p) = state(h) {
                p.acp_message(h, *v);
            }
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            let Some(p) = state(h) else { return LRESULT(0) };
            match win::loword(w.0) as i32 {
                ID_SEND => p.send(h),
                ID_NEW => p.new_chat(),
                ID_SIGN_IN => {
                    if let Some(s) = studio() {
                        s.copilot_sign_in();
                    }
                }
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

/// A file's text as the person sees it: the open editor's, unsaved changes and all, else the disk's.
fn current_text(path: &Path) -> Option<String> {
    studio().and_then(|s| s.open_text_of(path)).or_else(|| std::fs::read_to_string(path).ok()).map(|t| t.replace("\r\n", "\n"))
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
        let title = tf("Copilot wants to change {0}", &[&rel]);
        match review(pane, &title, &rel, old.as_deref().unwrap_or(""), &new) {
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
fn review(pane: HWND, title: &str, rel: &str, old: &str, new: &str) -> Review {
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
    let title = win::wide(title);
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
    fn the_agent_is_told_where_it_is_and_its_errors_said_plainly() {
        let c = copilot_context(Path::new(r"C:\x\shop"));
        assert!(c.contains("\"shop\"") && c.contains(".phf") && c.contains("_ui.rs"));
        assert_eq!(plain_agent_error("Not authorized: token expired"), t("Sign in to GitHub Copilot first: Sign in... below, or Copilot on the status bar."));
        assert_eq!(plain_agent_error("You have exceeded your monthly quota"), t("This month's Copilot use is spent - it comes back next month, or with a paid plan."));
        assert_eq!(plain_agent_error("something else"), "something else");
        assert!(lsp_same_start(Path::new(r"c:\Work\Shop\src\a.rs"), Path::new(r"C:\work\shop")));
        assert!(!lsp_same_start(Path::new(r"C:\other\a.rs"), Path::new(r"C:\work\shop")));
    }
}
