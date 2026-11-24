//! GitHub Copilot, through GitHub's own Copilot Language Server: grey suggestions as you type
//! (`textDocument/inlineCompletion`), the sign-in to GitHub, and the models the account offers -
//! the AI Assistant's Copilot side talks to the same server's agent (ACP, see `acp.rs`).
//!
//! The server is GitHub's (MIT, from github.com/github/copilot-language-server-release); its use
//! of Copilot is under GitHub's terms for the person's own account. Phai Studio does not ship it:
//! the first time Copilot is turned on, the Windows build of the newest release is downloaded
//! into `%LOCALAPPDATA%\Phai Studio\copilot`, checked against the SHA-256 GitHub gives for it.
//! Or, where Node.js has it installed (`npm install -g @github/copilot-language-server`), that.
//!
//! The sign-in is GitHub's device flow, kept by the server in its own place (as for every editor
//! that uses it) - never in Phai Studio's files.

use std::collections::HashMap;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use phai_editor::lsp::{self, Lsp};
use phai_ui::text::{t, tf};
use serde_json::{json, Value};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging as wm;

/// Posted for each message from the Copilot server (and the download's end); `LPARAM` is a
/// `Box<serde_json::Value>`.
pub const WM_COPILOT: u32 = wm::WM_APP + 0x0C02;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const RELEASES: &str = "https://api.github.com/repos/github/copilot-language-server-release/releases/latest";
const DOWNLOADS: &str = "https://github.com/github/copilot-language-server-release/releases/download/";
const EXE: &str = "copilot-language-server.exe";

/// How the server is run.
#[derive(Clone, Debug, PartialEq)]
pub enum Launch {
    /// GitHub's Windows build.
    Exe(PathBuf),
    /// Node.js and the npm package's `language-server.js`.
    Node(PathBuf, PathBuf),
    /// Phai Studio itself, standing in for the server (`PHAI_COPILOT_SERVER=fake`) - for tests.
    Fake(PathBuf),
}

/// Where the downloaded server is kept: `%LOCALAPPDATA%\Phai Studio\copilot` (`PHAI_COPILOT_DIR`
/// for a test's own).
pub fn server_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("PHAI_COPILOT_DIR") {
        return Some(PathBuf::from(d));
    }
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join("Phai Studio").join("copilot"))
}

/// The server to run: a test's, the one downloaded, or Node's - `None` when there is none yet.
pub fn find() -> Option<Launch> {
    if let Ok(s) = std::env::var("PHAI_COPILOT_SERVER") {
        if s == "fake" {
            return std::env::current_exe().ok().map(Launch::Fake);
        }
        let p = PathBuf::from(&s);
        if p.extension().is_some_and(|e| e == "js") {
            return Some(Launch::Node(node()?, p));
        }
        return p.is_file().then_some(Launch::Exe(p));
    }
    if let Some(exe) = server_dir().and_then(|d| downloaded(&d)) {
        return Some(Launch::Exe(exe));
    }
    let script = PathBuf::from(std::env::var_os("APPDATA")?).join("npm").join("node_modules").join("@github").join("copilot-language-server").join("dist").join("language-server.js");
    if script.is_file() {
        return Some(Launch::Node(node()?, script));
    }
    None
}

/// The newest server downloaded into `dir`: `dir\<version>\copilot-language-server.exe`.
pub fn downloaded(dir: &Path) -> Option<PathBuf> {
    let mut found: Vec<(String, PathBuf)> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.join(EXE).is_file() && !p.extension().is_some_and(|e| e == "part"))
        .filter_map(|p| Some((p.file_name()?.to_string_lossy().to_string(), p.join(EXE))))
        .collect();
    found.sort_by(|a, b| if crate::update::is_newer(&a.0, &b.0) { std::cmp::Ordering::Greater } else if crate::update::is_newer(&b.0, &a.0) { std::cmp::Ordering::Less } else { std::cmp::Ordering::Equal });
    found.pop().map(|(_, p)| p)
}

fn node() -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?).map(|d| d.join("node.exe")).find(|p| p.is_file())
}

/// The command that runs the server, its language server (`acp` false, `--stdio`) or its agent
/// (`--acp`), in `dir`.
pub fn command(l: &Launch, acp: bool, dir: &Path) -> Command {
    let mode = if acp { "--acp" } else { "--stdio" };
    let mut c = match l {
        Launch::Exe(p) => {
            let mut c = Command::new(p);
            c.arg(mode);
            c
        }
        Launch::Node(node, script) => {
            let mut c = Command::new(node);
            c.arg(script).arg(mode);
            c
        }
        Launch::Fake(me) => {
            let mut c = Command::new(me);
            c.arg("--fake-copilot").arg(mode);
            c
        }
    };
    c.current_dir(dir).creation_flags(CREATE_NO_WINDOW);
    // Node's "SQLite is an experimental feature" said on every start - nothing for a person.
    c.env("NODE_NO_WARNINGS", "1");
    c
}

// --- the download ------------------------------------------------------------------------------

/// GitHub's Windows build of a release of the server.
#[derive(Clone, Debug, PartialEq)]
pub struct Asset {
    pub version: String,
    pub name: String,
    pub url: String,
    /// Lower-case hex, never empty: a file GitHub gives no SHA-256 for is not taken.
    pub sha256: String,
    pub size: u64,
}

/// The Windows build for this machine (x64 or Arm64) in GitHub's description of a release.
pub fn asset_of(json: &str, arm: bool) -> Option<Asset> {
    let v: Value = serde_json::from_str(json).ok()?;
    let version = v["tag_name"].as_str()?.trim_start_matches('v').to_string();
    let want = format!("copilot-language-server-win32-{}-", if arm { "arm64" } else { "x64" });
    let a = v["assets"].as_array()?.iter().find(|a| a["name"].as_str().is_some_and(|n| n.starts_with(&want) && n.ends_with(".zip")))?;
    let url = a["browser_download_url"].as_str()?.to_string();
    // Only from GitHub's own releases of it.
    if !url.starts_with(DOWNLOADS) {
        return None;
    }
    let sha256 = a["digest"].as_str()?.strip_prefix("sha256:")?.to_ascii_lowercase();
    if sha256.len() != 64 {
        return None;
    }
    Some(Asset { version, name: a["name"].as_str()?.to_string(), url, sha256, size: a["size"].as_u64().unwrap_or(0) })
}

fn arm() -> bool {
    std::env::var("PROCESSOR_ARCHITECTURE").is_ok_and(|a| a.eq_ignore_ascii_case("ARM64")) || std::env::var("PROCESSOR_ARCHITEW6432").is_ok_and(|a| a.eq_ignore_ascii_case("ARM64"))
}

/// The newest release's Windows build, asked of GitHub.
pub fn newest() -> Result<Asset, String> {
    let out = crate::update::curl(&["-s", "-f", "--max-time", "20", "-H", "Accept: application/vnd.github+json", RELEASES])?;
    asset_of(&String::from_utf8_lossy(&out), arm()).ok_or_else(|| t("GitHub's answer has no Windows build of Copilot's server").to_string())
}

/// The release downloaded, checked and unpacked into `dir\<version>`: the server's path.
pub fn download(a: &Asset, dir: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let zip = dir.join(&a.name);
    let _ = std::fs::remove_file(&zip);
    crate::update::curl(&["-s", "-f", "-L", "--max-time", "1800", "-o", &zip.to_string_lossy(), &a.url])?;
    let data = std::fs::read(&zip).map_err(|e| e.to_string())?;
    if crate::update::sha256_hex(&data) != a.sha256 {
        let _ = std::fs::remove_file(&zip);
        return Err(t("The download is not the file GitHub describes - it was deleted. Try again later.").to_string());
    }
    drop(data);
    // Unpacked beside, then put in place whole: a half-unpacked folder is never taken for one.
    let part = dir.join(format!("{}.part", a.version));
    let _ = std::fs::remove_dir_all(&part);
    std::fs::create_dir_all(&part).map_err(|e| e.to_string())?;
    // Windows' own tar.exe unpacks a zip.
    let tar = PathBuf::from(std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into())).join("System32").join("tar.exe");
    let ok = Command::new(tar).arg("-xf").arg(&zip).arg("-C").arg(&part).creation_flags(CREATE_NO_WINDOW).status().map(|s| s.success()).unwrap_or(false);
    let _ = std::fs::remove_file(&zip);
    if !ok || !part.join(EXE).is_file() {
        let _ = std::fs::remove_dir_all(&part);
        return Err(t("Copilot's server could not be unpacked.").to_string());
    }
    let to = dir.join(&a.version);
    let _ = std::fs::remove_dir_all(&to);
    std::fs::rename(&part, &to).map_err(|e| e.to_string())?;
    // Older versions: room given back.
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        if e.path() != to && e.path().join(EXE).is_file() {
            let _ = std::fs::remove_dir_all(e.path());
        }
    }
    Ok(to.join(EXE))
}

/// Downloads the server on a thread of its own; its end is posted to `window` as [`WM_COPILOT`]
/// with `{"phaiDownload": true, "error": ...}`.
pub fn download_in_background(window: HWND) {
    let target = window.0 as isize;
    std::thread::spawn(move || {
        let result = server_dir().ok_or_else(|| "no LOCALAPPDATA".to_string()).and_then(|dir| newest().and_then(|a| download(&a, &dir)));
        let v = match result {
            Ok(p) => json!({"phaiDownload": true, "path": p.to_string_lossy()}),
            Err(e) => json!({"phaiDownload": true, "error": e}),
        };
        let b = Box::into_raw(Box::new(v));
        unsafe {
            if wm::PostMessageW(Some(HWND(target as *mut _)), WM_COPILOT, windows::Win32::Foundation::WPARAM(0), windows::Win32::Foundation::LPARAM(b as isize)).is_err() {
                drop(Box::from_raw(b));
            }
        }
    });
}

// --- the language server -----------------------------------------------------------------------

/// Where the server is in its life, as the status bar says it.
#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Starting,
    SignedOut,
    /// Waiting for the person to finish at github.com/login/device: the code.
    SigningIn(String),
    SignedIn(String),
    /// Signed in, but the account has no Copilot.
    NoCopilot(String),
    /// What went wrong, in words.
    Trouble(String),
}

/// What an answer of ours was to.
#[derive(Clone, Debug)]
enum Ask {
    Initialize,
    Status,
    SignIn,
    FinishSignIn,
    SignOut,
    Inline { editor: isize, version: i64, at: (u32, u32) },
    Models(isize),
    Other,
}

/// What a message from the server asks Phai Studio to do.
#[derive(Debug, PartialEq)]
pub enum Said {
    Nothing,
    /// Started and initialized: the open files may be told it.
    Ready,
    /// The status bar's words have changed.
    Status,
    /// A suggestion for an editor, when it is still where it was asked.
    Suggestion { editor: HWND, version: i64, at: (u32, u32), item: Value },
    /// The device code to show, with what finishes the sign-in once it is entered.
    SignInCode { code: String, uri: String },
    /// The models for a pane of the AI Assistant: (id, name).
    Models { pane: HWND, list: Vec<(String, String)> },
}

/// The Copilot server as Phai Studio runs it.
pub struct Copilot {
    lsp: Lsp,
    asked: HashMap<i64, Ask>,
    pub state: State,
    pub ready: bool,
    /// Copilot Free, Pro... as the server says the account's plan.
    pub plan: String,
    /// The files told it, by path: their version.
    opened: HashMap<PathBuf, i64>,
    focused: Option<PathBuf>,
    /// The suggestion asked last: an older answer is passed by.
    last_inline: i64,
    /// The finishing command of a device-flow sign-in.
    finish: Option<Value>,
}

/// LSP's language for a file.
pub fn language_of(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref() {
        Some("rs") => "rust",
        Some("toml") | Some("phf") => "toml",
        Some("css") => "css",
        Some("md") => "markdown",
        Some("json") => "json",
        Some("html") | Some("htm") => "html",
        Some("js") => "javascript",
        Some("ts") => "typescript",
        Some("py") => "python",
        Some("sql") => "sql",
        Some("xml") => "xml",
        _ => "plaintext",
    }
}

impl Copilot {
    /// Starts the server for the folder `root`; what it says goes to `window`.
    pub fn start(launch: &Launch, root: &Path, window: HWND) -> Option<Copilot> {
        let lsp = Lsp::spawn(command(launch, false, root), root, window, WM_COPILOT)?;
        let mut c = Copilot { lsp, asked: HashMap::new(), state: State::Starting, ready: false, plan: String::new(), opened: HashMap::new(), focused: None, last_inline: 0, finish: None };
        let version = env!("CARGO_PKG_VERSION");
        let id = c.lsp.request(
            "initialize",
            json!({
                "processId": std::process::id(),
                "rootUri": lsp::uri_of(root),
                "workspaceFolders": [{ "uri": lsp::uri_of(root), "name": root.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default() }],
                "capabilities": { "workspace": { "workspaceFolders": true, "configuration": true }, "window": { "showDocument": { "support": true } } },
                "initializationOptions": {
                    "editorInfo": { "name": "Phai Studio", "version": version },
                    "editorPluginInfo": { "name": "Phai Studio Copilot", "version": version }
                }
            }),
        );
        c.asked.insert(id, Ask::Initialize);
        Some(c)
    }

    pub fn alive(&mut self) -> bool {
        self.lsp.alive()
    }

    /// The status bar's words for it.
    pub fn status_words(&self) -> String {
        match &self.state {
            State::Starting => t("Copilot: starting...").to_string(),
            State::SignedOut => t("Copilot: sign in").to_string(),
            State::SigningIn(code) => tf("Copilot: enter {0} at github.com/login/device", &[code]),
            State::SignedIn(user) => format!("Copilot: {user}"),
            State::NoCopilot(user) => tf("Copilot: {0} has no Copilot", &[user]),
            State::Trouble(e) => format!("Copilot: {e}"),
        }
    }

    pub fn signed_in(&self) -> bool {
        matches!(self.state, State::SignedIn(_))
    }

    fn ask(&mut self, method: &str, params: Value, what: Ask) -> i64 {
        let id = self.lsp.request(method, params);
        self.asked.insert(id, what);
        id
    }

    pub fn sign_in(&mut self) {
        self.ask("signIn", json!({}), Ask::SignIn);
    }

    /// The device code was shown: GitHub opened, and the server waits for the person there.
    pub fn finish_sign_in(&mut self) {
        let Some(cmd) = self.finish.take() else { return };
        self.ask("workspace/executeCommand", json!({ "command": cmd["command"], "arguments": cmd.get("arguments").cloned().unwrap_or(json!([])) }), Ask::FinishSignIn);
    }

    pub fn sign_out(&mut self) {
        self.ask("signOut", json!({}), Ask::SignOut);
    }

    /// The models the account offers for chat and the agent.
    pub fn models(&mut self, pane: HWND) {
        self.ask("copilot/models", json!({}), Ask::Models(pane.0 as isize));
    }

    pub fn is_open(&self, path: &Path) -> bool {
        self.opened.contains_key(path)
    }

    /// A file shown in an editor, told the server.
    pub fn open(&mut self, path: &Path, version: i64, text: &str) {
        if !self.ready || self.opened.contains_key(path) {
            return;
        }
        self.lsp.open_as(path, language_of(path), version, text);
        self.opened.insert(path.to_path_buf(), version);
    }

    pub fn change(&mut self, path: &Path, version: i64, text: &str) {
        if !self.ready {
            return;
        }
        if !self.opened.contains_key(path) {
            return self.open(path, version, text);
        }
        // The server's versions only go up.
        let v = self.opened.get(path).copied().unwrap_or(0).max(version - 1) + 1;
        self.lsp.did_change(path, v, text);
        self.opened.insert(path.to_path_buf(), v);
    }

    pub fn close(&mut self, path: &Path) {
        if self.opened.remove(path).is_some() {
            self.lsp.did_close(path);
        }
    }

    /// Asks for a suggestion at `at` (LSP line and character) in the editor `editor` showing `path`.
    pub fn suggest(&mut self, editor: HWND, path: &Path, text: &str, version: i64, at: (u32, u32), (tab_size, spaces): (usize, bool)) {
        if !self.ready || !self.signed_in() {
            return;
        }
        self.change_if_needed(path, version, text);
        if self.focused.as_deref() != Some(path) {
            self.lsp.notify("textDocument/didFocus", json!({ "textDocument": { "uri": lsp::uri_of(path) } }));
            self.focused = Some(path.to_path_buf());
        }
        // The last ask still out: no longer wanted.
        if self.asked.get(&self.last_inline).is_some_and(|a| matches!(a, Ask::Inline { .. })) {
            self.asked.remove(&self.last_inline);
            self.lsp.notify("$/cancelRequest", json!({ "id": self.last_inline }));
        }
        let v = self.opened.get(path).copied().unwrap_or(version);
        let id = self.ask(
            "textDocument/inlineCompletion",
            json!({
                "textDocument": { "uri": lsp::uri_of(path), "version": v },
                "position": { "line": at.0, "character": at.1 },
                "context": { "triggerKind": 2 },
                "formattingOptions": { "tabSize": tab_size, "insertSpaces": spaces }
            }),
            Ask::Inline { editor: editor.0 as isize, version, at },
        );
        self.last_inline = id;
    }

    fn change_if_needed(&mut self, path: &Path, version: i64, text: &str) {
        if !self.opened.contains_key(path) {
            self.open(path, version, text);
        }
    }

    /// A suggestion was shown.
    pub fn shown(&self, item: &Value) {
        self.lsp.notify("textDocument/didShowCompletion", json!({ "item": item }));
    }

    /// A suggestion was taken whole: its command says so to the server.
    pub fn accepted(&mut self, item: &Value) {
        if let Some(cmd) = item.get("command").filter(|c| c["command"].is_string()) {
            self.ask("workspace/executeCommand", json!({ "command": cmd["command"], "arguments": cmd.get("arguments").cloned().unwrap_or(json!([])) }), Ask::Other);
        }
    }

    /// Part of a suggestion was taken: `length` UTF-16 units of it.
    pub fn partly_accepted(&self, item: &Value, length: usize) {
        self.lsp.notify("textDocument/didPartiallyAcceptCompletion", json!({ "item": item, "acceptedLength": length }));
    }

    /// A message from the server: answered where it asks, kept where it tells.
    pub fn message(&mut self, msg: Value) -> Said {
        if std::env::var_os("PHAI_COPILOT_LOG").is_some() {
            let s = msg.to_string();
            eprintln!("copilot: {}", s.chars().take(400).collect::<String>());
        }
        // An answer of ours.
        if let (Some(id), None) = (msg.get("id").and_then(Value::as_i64), msg.get("method")) {
            let Some(ask) = self.asked.remove(&id) else { return Said::Nothing };
            let result = msg.get("result").cloned().unwrap_or(Value::Null);
            let error = msg["error"]["message"].as_str().map(str::to_string);
            return self.answered(ask, result, error);
        }
        let method = msg["method"].as_str().unwrap_or("").to_string();
        // The server asking something of us.
        if let Some(id) = msg.get("id") {
            let result = match method.as_str() {
                // Settings, by section: none of ours - the server's own defaults.
                "workspace/configuration" => Value::Array(msg["params"]["items"].as_array().map(|a| a.iter().map(|_| Value::Null).collect()).unwrap_or_default()),
                "window/showDocument" => {
                    let uri = msg["params"]["uri"].as_str().unwrap_or("");
                    json!({ "success": open_page(uri) })
                }
                // A notice about the account, with buttons: said on the status bar.
                "window/showMessageRequest" => {
                    if let Some(m) = msg["params"]["message"].as_str() {
                        self.state = match &self.state {
                            State::SignedIn(_) => State::Trouble(m.to_string()),
                            other => other.clone(),
                        };
                    }
                    Value::Null
                }
                _ => Value::Null,
            };
            self.lsp.answer(id, result);
            return Said::Nothing;
        }
        match method.as_str() {
            "didChangeStatus" => {
                let p = &msg["params"];
                let kind = p["kind"].as_str().unwrap_or("Normal");
                let words = p["message"].as_str().unwrap_or("").trim().to_string();
                match kind {
                    "Error" | "Warning" if !words.is_empty() => {
                        self.state = State::Trouble(plain_trouble(&words));
                        return Said::Status;
                    }
                    "Normal" if matches!(self.state, State::Trouble(_)) => {
                        // Mended: asked again who is signed in.
                        self.ask("checkStatus", json!({}), Ask::Status);
                    }
                    _ => {}
                }
                Said::Nothing
            }
            // Newer servers say who is signed in this way too.
            "didChangeStatus/v2" => {
                for s in msg["params"]["statuses"].as_array().into_iter().flatten() {
                    if s["category"] == "auth" {
                        let changed = self.auth(&s["result"]);
                        if changed {
                            return Said::Status;
                        }
                    }
                }
                Said::Nothing
            }
            "copilot/quotaChange" => {
                self.plan = msg["params"]["copilotPlan"].as_str().unwrap_or("").to_string();
                Said::Nothing
            }
            _ => Said::Nothing,
        }
    }

    /// A `checkStatus` (or sign-in) answer: who is signed in. Whether the state changed.
    fn auth(&mut self, r: &Value) -> bool {
        let user = r["user"].as_str().unwrap_or("").to_string();
        let now = match r["status"].as_str().unwrap_or("") {
            "OK" | "AlreadySignedIn" | "MaybeOk" if !user.is_empty() => State::SignedIn(user),
            "NotAuthorized" => State::NoCopilot(user),
            "NotSignedIn" => State::SignedOut,
            _ => return false,
        };
        let changed = now != self.state;
        self.state = now;
        changed
    }

    fn answered(&mut self, ask: Ask, result: Value, error: Option<String>) -> Said {
        match ask {
            Ask::Initialize => {
                if let Some(e) = error {
                    self.state = State::Trouble(e);
                    return Said::Status;
                }
                self.lsp.notify("initialized", json!({}));
                self.lsp.notify("workspace/didChangeConfiguration", json!({ "settings": {} }));
                self.ready = true;
                self.ask("checkStatus", json!({}), Ask::Status);
                Said::Ready
            }
            Ask::Status | Ask::SignOut => {
                if let Some(e) = error {
                    self.state = State::Trouble(e);
                } else if !self.auth(&result) && matches!(ask, Ask::SignOut) {
                    self.state = State::SignedOut;
                }
                Said::Status
            }
            Ask::SignIn => {
                if let Some(e) = error {
                    self.state = State::Trouble(e);
                    return Said::Status;
                }
                if let Some(code) = result["userCode"].as_str() {
                    self.finish = result.get("command").cloned().or(Some(json!({ "command": "github.copilot.finishDeviceFlow", "arguments": [] })));
                    self.state = State::SigningIn(code.to_string());
                    let uri = result["verificationUri"].as_str().unwrap_or("https://github.com/login/device").to_string();
                    return Said::SignInCode { code: code.to_string(), uri };
                }
                self.auth(&result);
                Said::Status
            }
            Ask::FinishSignIn => {
                match error {
                    Some(e) => self.state = State::Trouble(e),
                    None => {
                        if !self.auth(&result) && !self.signed_in() {
                            // Finished, but not said who: asked.
                            self.ask("checkStatus", json!({}), Ask::Status);
                        }
                    }
                }
                Said::Status
            }
            Ask::Inline { editor, version, at } => {
                if error.is_some() {
                    return Said::Nothing;
                }
                let items = match &result {
                    Value::Array(a) => a.clone(),
                    _ => result["items"].as_array().cloned().unwrap_or_default(),
                };
                match items.into_iter().find(|i| i["insertText"].is_string() || i["insertText"]["value"].is_string()) {
                    Some(item) => Said::Suggestion { editor: HWND(editor as *mut _), version, at, item },
                    None => Said::Nothing,
                }
            }
            Ask::Models(pane) => Said::Models { pane: HWND(pane as *mut _), list: if error.is_some() { vec![] } else { chat_models(&result) } },
            Ask::Other => Said::Nothing,
        }
    }
}

/// The text and range of an inline completion item: (start, end, text).
pub fn item_parts(item: &Value, at: (u32, u32)) -> ((u32, u32), (u32, u32), String) {
    let pos = |v: &Value| (v["line"].as_u64().unwrap_or(at.0 as u64) as u32, v["character"].as_u64().unwrap_or(at.1 as u64) as u32);
    let text = item["insertText"].as_str().or(item["insertText"]["value"].as_str()).unwrap_or("").to_string();
    match item.get("range") {
        Some(r) if r.is_object() => (pos(&r["start"]), pos(&r["end"]), text),
        _ => (at, at, text),
    }
}

/// The models of a `copilot/models` answer that chat and the agent may use: (id, name), the
/// account's default first.
pub fn chat_models(result: &Value) -> Vec<(String, String)> {
    let mut v: Vec<(bool, String, String)> = result
        .as_array()
        .into_iter()
        .flatten()
        .filter(|m| m["scopes"].as_array().is_some_and(|s| s.iter().any(|x| x == "agent-panel" || x == "chat-panel")))
        .filter_map(|m| {
            let id = m["id"].as_str()?.to_string();
            let name = m["modelName"].as_str().or(m["name"].as_str()).unwrap_or(&id).to_string();
            Some((!(m["isChatDefault"].as_bool().unwrap_or(false) || id == "auto"), id, name))
        })
        .collect();
    v.sort_by_key(|(later, _, _)| *later);
    v.dedup_by(|a, b| a.1 == b.1);
    v.into_iter().map(|(_, id, name)| (id, name)).collect()
}

/// What the server said went wrong, in a few words a person can act on.
pub fn plain_trouble(words: &str) -> String {
    let w = words.to_lowercase();
    if w.contains("not signed in") || w.contains("sign in") {
        t("sign in").to_string()
    } else if w.contains("quota") || w.contains("limit") {
        t("this month's free use is spent").to_string()
    } else if w.contains("network") || w.contains("fetch") || w.contains("econn") || w.contains("timed out") || w.contains("offline") {
        t("GitHub cannot be reached - is the computer online?").to_string()
    } else {
        words.chars().take(80).collect()
    }
}

/// Opens a web page the server names - only GitHub's.
pub fn open_page(uri: &str) -> bool {
    let ok = uri.starts_with("https://github.com/") || uri.starts_with("https://docs.github.com/") || uri.contains(".ghe.com/");
    if !ok || std::env::var_os("PHAI_SHOT").is_some() || std::env::var_os("PHAI_COPILOT_NO_BROWSER").is_some() {
        return ok;
    }
    use windows::core::PCWSTR;
    let u = phai_ui::win::wide(uri);
    let open = phai_ui::win::wide("open");
    unsafe {
        windows::Win32::UI::Shell::ShellExecuteW(None, PCWSTR(open.as_ptr()), PCWSTR(u.as_ptr()), PCWSTR::null(), PCWSTR::null(), wm::SW_SHOWNORMAL);
    }
    true
}

// --- a stand-in server, for tests --------------------------------------------------------------

/// `phai-studio --fake-copilot --stdio|--acp`: a server that speaks as Copilot's does, with no
/// GitHub behind it - signed in as `tester`, a suggestion for every ask, and an agent that answers
/// in pieces and changes the first `.rs` file it is asked about (after asking, as Copilot's does).
pub mod fake {
    use std::io::{BufRead, BufReader, Write};

    use serde_json::{json, Value};

    pub fn serve(acp: bool) -> ! {
        if acp {
            acp_serve()
        } else {
            lsp_serve()
        }
    }

    fn read_lsp(r: &mut impl BufRead) -> Option<Value> {
        let mut len = 0usize;
        loop {
            let mut h = String::new();
            if r.read_line(&mut h).ok()? == 0 {
                return None;
            }
            let h = h.trim();
            if h.is_empty() {
                break;
            }
            if let Some(v) = h.strip_prefix("Content-Length:") {
                len = v.trim().parse().ok()?;
            }
        }
        let mut body = vec![0u8; len];
        r.read_exact(&mut body).ok()?;
        serde_json::from_slice(&body).ok()
    }

    fn write_lsp(v: &Value) {
        let b = v.to_string();
        let mut o = std::io::stdout().lock();
        let _ = write!(o, "Content-Length: {}\r\n\r\n{}", b.len(), b);
        let _ = o.flush();
    }

    fn lsp_serve() -> ! {
        let mut r = BufReader::new(std::io::stdin().lock());
        // The open files' text, to make a suggestion that fits.
        let mut texts: std::collections::HashMap<String, String> = Default::default();
        let mut signed_in = std::env::var_os("PHAI_FAKE_SIGNED_OUT").is_none();
        while let Some(m) = read_lsp(&mut r) {
            let method = m["method"].as_str().unwrap_or("");
            let id = m.get("id").cloned();
            let answer = |result: Value| {
                if let Some(id) = &id {
                    write_lsp(&json!({"jsonrpc": "2.0", "id": id, "result": result}));
                }
            };
            match method {
                "initialize" => answer(json!({"capabilities": {"textDocumentSync": {"openClose": true, "change": 2}, "inlineCompletionProvider": {}}, "serverInfo": {"name": "fake copilot", "version": "0"}})),
                "checkStatus" => answer(if signed_in { json!({"status": "OK", "user": "tester"}) } else { json!({"status": "NotSignedIn"}) }),
                "signIn" => answer(json!({"status": "PromptUserDeviceFlow", "userCode": "ABCD-1234", "verificationUri": "https://github.com/login/device", "command": {"command": "github.copilot.finishDeviceFlow", "arguments": [], "title": "Sign in"}})),
                "signOut" => {
                    signed_in = false;
                    answer(json!({"status": "NotSignedIn"}))
                }
                "workspace/executeCommand" => {
                    if m["params"]["command"] == "github.copilot.finishDeviceFlow" {
                        signed_in = true;
                        answer(json!({"status": "OK", "user": "tester"}))
                    } else {
                        eprintln!("fake copilot: accepted {}", m["params"]["arguments"]);
                        answer(Value::Null)
                    }
                }
                "copilot/models" => answer(json!([
                    {"id": "gpt-41-copilot", "modelName": "GPT-4.1 Copilot", "scopes": ["completion"]},
                    {"id": "auto", "modelName": "Auto", "scopes": ["inline", "chat-panel", "agent-panel"]},
                    {"id": "gpt-5", "modelName": "GPT-5", "scopes": ["chat-panel", "agent-panel"]},
                    {"id": "claude-sonnet-4.5", "modelName": "Claude Sonnet 4.5", "scopes": ["chat-panel", "agent-panel"]},
                    {"id": "gemini-2.5-pro", "modelName": "Gemini 2.5 Pro", "scopes": ["chat-panel", "agent-panel"]}
                ])),
                "textDocument/didOpen" => {
                    texts.insert(m["params"]["textDocument"]["uri"].as_str().unwrap_or("").to_string(), m["params"]["textDocument"]["text"].as_str().unwrap_or("").to_string());
                }
                "textDocument/didChange" => {
                    if let Some(t) = m["params"]["contentChanges"].as_array().and_then(|a| a.last()).and_then(|c| c["text"].as_str()) {
                        texts.insert(m["params"]["textDocument"]["uri"].as_str().unwrap_or("").to_string(), t.to_string());
                    }
                }
                "textDocument/inlineCompletion" => {
                    // The line so far, and the rest of a function that adds - as Copilot would.
                    let uri = m["params"]["textDocument"]["uri"].as_str().unwrap_or("");
                    let (line, ch) = (m["params"]["position"]["line"].as_u64().unwrap_or(0) as usize, m["params"]["position"]["character"].as_u64().unwrap_or(0) as usize);
                    let text = texts.get(uri).cloned().unwrap_or_default();
                    let l = text.lines().nth(line).unwrap_or("").to_string();
                    let before: String = l.encode_utf16().take(ch).collect::<Vec<u16>>().iter().map(|u| char::from_u32(*u as u32).unwrap_or('?')).collect();
                    let suggestion = if before.trim_start().starts_with("fn") { format!("{before}(a: i32, b: i32) -> i32 {{\n    a + b\n}}") } else { format!("{before}// Copilot was here") };
                    answer(json!({"items": [{"insertText": suggestion, "range": {"start": {"line": line, "character": 0}, "end": {"line": line, "character": ch}}, "command": {"title": "Completion Accepted", "command": "github.copilot.didAcceptCompletionItem", "arguments": ["fake-1"]}}]}))
                }
                "textDocument/didShowCompletion" => eprintln!("fake copilot: shown"),
                "textDocument/didPartiallyAcceptCompletion" => eprintln!("fake copilot: partly accepted {}", m["params"]["acceptedLength"]),
                "shutdown" => answer(Value::Null),
                "exit" => std::process::exit(0),
                _ => {
                    if id.is_some() {
                        answer(Value::Null)
                    }
                }
            }
        }
        std::process::exit(0)
    }

    fn write_acp(v: &Value) {
        let mut o = std::io::stdout().lock();
        let _ = writeln!(o, "{v}");
        let _ = o.flush();
    }

    fn acp_serve() -> ! {
        let mut lines = BufReader::new(std::io::stdin().lock()).lines();
        let mut next = 1000;
        let mut cwd = String::new();
        let mut model = "auto".to_string();
        while let Some(Ok(line)) = lines.next() {
            let Ok(m) = serde_json::from_str::<Value>(&line) else { continue };
            let method = m["method"].as_str().unwrap_or("").to_string();
            let id = m.get("id").cloned().unwrap_or(Value::Null);
            let answer = |result: Value| write_acp(&json!({"jsonrpc": "2.0", "id": id, "result": result}));
            match method.as_str() {
                "initialize" => answer(json!({"protocolVersion": 1, "agentCapabilities": {"loadSession": true}, "agentInfo": {"name": "fake copilot", "version": "0"}, "authMethods": []})),
                "session/new" => {
                    cwd = m["params"]["cwd"].as_str().unwrap_or("").to_string();
                    answer(json!({"sessionId": "s1", "modes": {"availableModes": [], "currentModeId": "agent"}}))
                }
                "session/set_model" => {
                    model = m["params"]["modelId"].as_str().unwrap_or("auto").to_string();
                    answer(json!({}))
                }
                "session/prompt" => {
                    let sid = m["params"]["sessionId"].clone();
                    let words: String = m["params"]["prompt"].as_array().into_iter().flatten().filter_map(|b| b["text"].as_str()).collect::<Vec<_>>().join(" ");
                    let update = |u: Value| write_acp(&json!({"jsonrpc": "2.0", "method": "session/update", "params": {"sessionId": sid, "update": u}}));
                    for piece in [format!("({model}) "), "I will ".into(), "look at ".into(), "the code.".into()] {
                        update(json!({"sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": piece}}));
                    }
                    // A file to change: the main.rs under the folder it works in.
                    let file = std::path::Path::new(&cwd).join("src").join("main.rs");
                    let wants_edit = words.to_lowercase().contains("change") || words.to_lowercase().contains("แก้");
                    if wants_edit && file.is_file() {
                        let old = std::fs::read_to_string(&file).unwrap_or_default().replace("\r\n", "\n");
                        let first = old.lines().next().unwrap_or("").to_string();
                        let new_first = format!("// Changed by Copilot\n{first}");
                        let new = old.replacen(&first, &new_first, 1);
                        let diff = format!("\ndiff --git a/x b/x\n--- a/x\n+++ b/x\n@@ -1,1 +1,2 @@\n+// Changed by Copilot\n {first}\n");
                        let path = file.to_string_lossy().to_string();
                        update(json!({"sessionUpdate": "tool_call", "toolCallId": "c1", "title": "apply_patch", "kind": "edit", "status": "pending"}));
                        next += 1;
                        write_acp(&json!({"jsonrpc": "2.0", "id": next, "method": "session/request_permission", "params": {"sessionId": sid, "toolCall": {"toolCallId": "c1", "kind": "edit", "title": "Update file", "locations": [{"path": path}], "rawInput": {"fileName": path, "diff": diff}},
                            "options": [{"optionId": "allow_once", "name": "Allow once", "kind": "allow_once"}, {"optionId": "allow_always", "name": "Always allow", "kind": "allow_always"}, {"optionId": "reject_once", "name": "Deny", "kind": "reject_once"}]}}));
                        // Its answer.
                        let mut allowed = false;
                        for l in lines.by_ref() {
                            let Ok(l) = l else { break };
                            let Ok(a) = serde_json::from_str::<Value>(&l) else { continue };
                            if a["id"] == json!(next) && a.get("method").is_none() {
                                allowed = a["result"]["outcome"]["optionId"].as_str().is_some_and(|o| o.starts_with("allow"));
                                break;
                            }
                        }
                        if allowed {
                            let _ = std::fs::write(&file, &new);
                            update(json!({"sessionUpdate": "tool_call_update", "toolCallId": "c1", "status": "completed", "content": [{"type": "diff", "path": path, "oldText": old, "newText": new}]}));
                            update(json!({"sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": "\nI added a line at the top of src/main.rs."}}));
                        } else {
                            update(json!({"sessionUpdate": "tool_call_update", "toolCallId": "c1", "status": "failed"}));
                            update(json!({"sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": "\nThe change was declined."}}));
                        }
                    }
                    answer(json!({"stopReason": "end_turn"}))
                }
                "session/cancel" => {}
                _ => {
                    if !id.is_null() {
                        answer(Value::Null)
                    }
                }
            }
        }
        std::process::exit(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn github_s_release_gives_the_windows_build_checked() {
        let json = r#"{"tag_name":"1.574.1","assets":[
            {"name":"copilot-language-server-darwin-arm64-1.574.1.zip","digest":"sha256:11c5e96dd3225bf2724090949a2b4f54c1be33b3f36b82f1b32620f29b25fe16","browser_download_url":"https://github.com/github/copilot-language-server-release/releases/download/1.574.1/copilot-language-server-darwin-arm64-1.574.1.zip"},
            {"name":"copilot-language-server-win32-arm64-1.574.1.zip","size":103345101,"digest":"sha256:84c72a5beb301684f4ae7b5b8716bdd6fffce3c460d3fed84069bf4ceea0d832","browser_download_url":"https://github.com/github/copilot-language-server-release/releases/download/1.574.1/copilot-language-server-win32-arm64-1.574.1.zip"},
            {"name":"copilot-language-server-win32-x64-1.574.1.zip","size":106173254,"digest":"sha256:6FBA3EDB2A48B1975F113C0ED960A0D5D3A0205E0EDD511CBE402E80FC0E5B90","browser_download_url":"https://github.com/github/copilot-language-server-release/releases/download/1.574.1/copilot-language-server-win32-x64-1.574.1.zip"}]}"#;
        let a = asset_of(json, false).unwrap();
        assert_eq!(a.version, "1.574.1");
        assert_eq!(a.name, "copilot-language-server-win32-x64-1.574.1.zip");
        assert_eq!(a.sha256, "6fba3edb2a48b1975f113c0ed960a0d5d3a0205e0edd511cbe402e80fc0e5b90");
        assert_eq!(a.size, 106173254);
        assert!(asset_of(json, true).unwrap().name.contains("arm64"));
        assert!(asset_of(&json.replace("github.com/github/copilot-language-server-release", "example.com/x"), false).is_none(), "only GitHub's own releases");
        assert!(asset_of(&json.replace("sha256:6FBA", "md5:6FBA"), false).is_none(), "no SHA-256, not taken");
    }

    /// The real thing: GitHub asked, the newest Windows build downloaded, checked and unpacked into
    /// a folder of the test's own (`PHAI_COPILOT_DIR`, or the temporary folder), and run.
    #[test]
    #[ignore]
    fn the_newest_server_downloads_checks_and_runs() {
        let dir = std::env::var_os("PHAI_COPILOT_DIR").map(PathBuf::from).unwrap_or_else(|| std::env::temp_dir().join("phai-copilot-download"));
        let a = newest().unwrap();
        assert_eq!(a.sha256.len(), 64, "GitHub gives its SHA-256");
        let exe = download(&a, &dir).unwrap();
        assert_eq!(downloaded(&dir).unwrap(), exe);
        let out = Command::new(&exe).arg("--version").output().unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), a.version);
        let bad = Asset { sha256: "0".repeat(64), version: "0.0.1".into(), ..a };
        assert!(download(&bad, &dir).is_err(), "a file that is not what GitHub says is refused");
        assert!(!dir.join("0.0.1").exists());
    }

    /// A project of the test's own: Cargo.toml and src/main.rs as given; settings of its own too.
    fn project(name: &str, main: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("phai-copilot-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::create_dir_all(dir.join("settings")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"copilot_try\"\nversion = \"0.1.0\"\nedition = \"2021\"\n").unwrap();
        std::fs::write(dir.join("src").join("main.rs"), main).unwrap();
        std::fs::write(dir.join("settings").join("settings.txt"), "rust_analyzer=0\n").unwrap();
        dir
    }

    fn studio(dir: &Path) -> Command {
        let exe = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/phai-studio.exe");
        let mut c = Command::new(exe);
        c.arg(dir.join("src").join("main.rs")).env("PHAI_SETTINGS_DIR", dir.join("settings")).env("PHAI_COPILOT", "1").env("PHAI_COPILOT_SERVER", "fake");
        c
    }

    /// Phai Studio (built first) with the stand-in server: "fn add" typed, the suggestion grey after
    /// it, Tab takes it.
    #[test]
    #[ignore]
    fn a_suggestion_is_shown_and_tab_takes_it() {
        let dir = project("tab", "fn main() {\n}\n\n");
        let out = studio(&dir).env("PHAI_COPILOT_TYPE", "4:1:fn add").env("PHAI_COPILOT_TAB", "1").output().unwrap();
        let said = String::from_utf8_lossy(&out.stdout);
        assert!(said.contains(r#"suggestion "(a: i32, b: i32) -> i32 {\n    a + b\n}""#), "{said}");
        assert!(said.contains("after Tab, line 4: \"fn add(a: i32, b: i32) -> i32 {\""), "{said}");
        assert!(said.contains("shown false"), "{said}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The AI Assistant with Copilot chosen and the stand-in agent: in Act mode its change is made
    /// (accepted), in Ask mode not.
    #[test]
    #[ignore]
    fn copilot_s_agent_changes_a_file_only_in_act_mode() {
        for (act, changed) in [(true, true), (false, false)] {
            let dir = project("agent", "fn main() {\n}\n");
            let ask = if act { "act:change main.rs" } else { "change main.rs" };
            let shot = dir.join("pane.png");
            let _ = studio(&dir).env("PHAI_AI_PROVIDER", "copilot").env("PHAI_AI_ASK", ask).env("PHAI_AI_REVIEW", "accept").env("PHAI_SHOT", &shot).env("PHAI_SHOT_DELAY", "12000").status().unwrap();
            let text = std::fs::read_to_string(dir.join("src").join("main.rs")).unwrap();
            assert_eq!(text.starts_with("// Changed by Copilot\n"), changed, "act {act}: {text}");
            assert!(shot.is_file());
            let _ = std::fs::remove_dir_all(&dir);
        }
    }

    #[test]
    fn the_newest_download_is_run() {
        let dir = std::env::temp_dir().join(format!("phai-copilot-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for v in ["1.9.0", "1.10.2", "1.574.1.part"] {
            std::fs::create_dir_all(dir.join(v)).unwrap();
            std::fs::write(dir.join(v).join(EXE), b"").unwrap();
        }
        std::fs::create_dir_all(dir.join("2.0.0")).unwrap();
        assert_eq!(downloaded(&dir).unwrap(), dir.join("1.10.2").join(EXE), "the newest whole one - not one half unpacked, not an empty folder");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn models_and_suggestions_are_read() {
        let m = json!([
            {"id": "gpt-41-copilot", "modelName": "GPT-4.1 Copilot", "scopes": ["completion"]},
            {"id": "gpt-5", "modelName": "GPT-5", "scopes": ["chat-panel", "agent-panel"]},
            {"id": "auto", "modelName": "Auto", "scopes": ["inline", "chat-panel", "agent-panel"]}
        ]);
        assert_eq!(chat_models(&m), vec![("auto".to_string(), "Auto".to_string()), ("gpt-5".to_string(), "GPT-5".to_string())]);
        let item = json!({"insertText": "    b", "range": {"start": {"line": 2, "character": 0}, "end": {"line": 2, "character": 4}}});
        assert_eq!(item_parts(&item, (2, 4)), ((2, 0), (2, 4), "    b".to_string()));
        assert_eq!(item_parts(&json!({"insertText": "x"}), (1, 3)), ((1, 3), (1, 3), "x".to_string()));
        assert_eq!(language_of(Path::new("a/main.rs")), "rust");
        assert_eq!(language_of(Path::new("Form1.phf")), "toml");
        assert_eq!(plain_trouble("You have exceeded your monthly quota"), t("this month's free use is spent"));
    }

    #[test]
    fn only_github_s_pages_are_opened() {
        std::env::set_var("PHAI_COPILOT_NO_BROWSER", "1");
        assert!(open_page("https://github.com/login/device"));
        assert!(!open_page("https://example.com/login"));
        assert!(!open_page("file:///C:/Windows/notepad.exe"));
    }
}
