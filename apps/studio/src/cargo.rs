//! Building and running a project with Cargo, on a thread of its own, its findings posted back to
//! the window. Cargo is asked for JSON (`--message-format=json`): every diagnostic comes with its
//! file, line and column, and the built program's path comes with it - nothing is guessed from
//! text meant for people.

use std::io::{BufRead, BufReader};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use serde::Deserialize;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging as wm;

/// Posted to the window when a build ends; `LPARAM` is a `Box<Outcome>` to take back.
pub const WM_BUILD_DONE: u32 = wm::WM_APP + 0x0B01;
/// Posted for each line Cargo says while it works; `LPARAM` is a `Box<String>` to take back.
pub const WM_BUILD_LINE: u32 = wm::WM_APP + 0x0B02;

/// A line to the window, as it comes.
fn post_line(target: isize, line: String) {
    let boxed = Box::into_raw(Box::new(line));
    unsafe {
        if wm::PostMessageW(Some(HWND(target as *mut _)), WM_BUILD_LINE, WPARAM(0), LPARAM(boxed as isize)).is_err() {
            drop(Box::from_raw(boxed));
        }
    }
}

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Clone, Debug, PartialEq)]
pub struct Diagnostic {
    pub error: bool,
    pub message: String,
    pub file: Option<PathBuf>,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Default)]
pub struct Outcome {
    pub ok: bool,
    pub diagnostics: Vec<Diagnostic>,
    /// The program that was built, when one was.
    pub executable: Option<PathBuf>,
    pub seconds: f32,
    /// Cargo could not be started at all.
    pub no_cargo: bool,
    /// Cargo's own words when it failed before compiling anything (a broken Cargo.toml).
    pub cargo_error: Option<String>,
    /// Run the program when the build succeeds.
    pub then_run: bool,
    /// What was done.
    pub job: Job,
    /// Run All Tests: each test and how it went.
    pub tests: Vec<TestResult>,
}

/// What the Build menu asks of Cargo, as Visual Studio's Build menu asks of MSBuild.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Job {
    #[default]
    Build,
    /// Clean, then build.
    Rebuild,
    /// What was built goes.
    Clean,
    /// `cargo clippy`: Run Code Analysis.
    Analyse,
    /// `cargo test`: Run All Tests.
    Test,
}

/// One test as `cargo test` reported it.
#[derive(Clone, Debug, PartialEq)]
pub struct TestResult {
    /// Its path in the crate: `model::tests::undo_works`.
    pub name: String,
    /// `ok`, `FAILED` or `ignored`.
    pub outcome: String,
    /// Why it failed: the panic's first words.
    pub message: String,
}

/// The tests in what `cargo test` printed: each `test NAME ... RESULT` line, with a failure's
/// panic message from its `---- NAME stdout ----` section.
pub fn tests_of(lines: &[String]) -> Vec<TestResult> {
    let mut out: Vec<TestResult> = Vec::new();
    for l in lines {
        let Some(rest) = l.strip_prefix("test ") else { continue };
        let Some((name, outcome)) = rest.rsplit_once(" ... ") else { continue };
        let outcome = outcome.split([' ', ',']).next().unwrap_or_default();
        if matches!(outcome, "ok" | "FAILED" | "ignored") {
            out.push(TestResult { name: name.trim().to_string(), outcome: outcome.to_string(), message: String::new() });
        }
    }
    let mut current: Option<String> = None;
    let mut panicked = false;
    for l in lines {
        if let Some(n) = l.strip_prefix("---- ").and_then(|r| r.strip_suffix(" stdout ----")) {
            current = Some(n.trim().to_string());
            panicked = false;
            continue;
        }
        let Some(n) = &current else { continue };
        if l.contains("panicked at") {
            panicked = true;
            continue;
        }
        if panicked && !l.trim().is_empty() {
            if let Some(t) = out.iter_mut().find(|t| &t.name == n && t.message.is_empty()) {
                t.message = l.trim().to_string();
            }
            current = None;
        }
    }
    out
}

/// A project's package name, from its `Cargo.toml`.
pub fn package_name(project: &Path) -> Option<String> {
    let text = std::fs::read_to_string(project.join("Cargo.toml")).ok()?;
    let mut in_package = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            in_package = line == "[package]";
        } else if in_package {
            if let Some(v) = line.strip_prefix("name") {
                let v = v.trim_start().strip_prefix('=')?.trim().trim_matches('"');
                return Some(v.to_string());
            }
        }
    }
    None
}

/// A project's version, from its `Cargo.toml` - `version.workspace = true` taken from the
/// workspace's own.
pub fn package_version(project: &Path) -> Option<String> {
    let read = |p: &Path, section: &str| -> Option<String> {
        let text = std::fs::read_to_string(p.join("Cargo.toml")).ok()?;
        let mut inside = false;
        for line in text.lines().map(str::trim) {
            if line.starts_with('[') {
                inside = line == section;
            } else if inside {
                if let Some(v) = line.strip_prefix("version") {
                    let v = v.trim_start().strip_prefix('=')?.trim();
                    return Some(v.trim_matches('"').to_string());
                }
            }
        }
        None
    };
    match read(project, "[package]") {
        Some(v) if !v.contains("workspace") && !v.starts_with('{') => Some(v),
        _ => {
            let ws = workspace_of(&project.join("Cargo.toml"))?;
            read(&ws, "[workspace.package]")
        }
    }
}

/// A `[package]` field of a project's Cargo.toml as written: `name`, `edition`, `description`...
/// - its value without quotes; `authors` as the list's items joined by commas.
pub fn package_field(project: &Path, key: &str) -> Option<String> {
    let text = std::fs::read_to_string(project.join("Cargo.toml")).ok()?;
    let mut inside = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line == "[package]";
        } else if inside {
            let Some(rest) = line.strip_prefix(key) else { continue };
            let Some(v) = rest.trim_start().strip_prefix('=') else { continue };
            let v = v.trim();
            if v.starts_with('[') {
                return Some(v.trim_matches(['[', ']']).split(',').map(|s| s.trim().trim_matches('"')).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(", "));
            }
            return Some(v.trim_matches('"').to_string());
        }
    }
    None
}

/// The text of a Cargo.toml with a `[package]` field set (`value` empty: taken out). Its other
/// lines, comments and order kept; a new field goes after the last one of `[package]`.
pub fn with_package_field(text: &str, key: &str, value: &str, list: bool) -> String {
    let quoted = |v: &str| format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\""));
    let line = if list {
        format!("{key} = [{}]", value.split(',').map(str::trim).filter(|s| !s.is_empty()).map(quoted).collect::<Vec<_>>().join(", "))
    } else {
        format!("{key} = {}", quoted(value))
    };
    let mut out: Vec<String> = Vec::new();
    let (mut inside, mut done, mut last_field) = (false, false, None);
    for l in text.lines() {
        let t = l.trim();
        if t.starts_with('[') {
            if inside && !done && !value.trim().is_empty() {
                let at = last_field.map(|i: usize| i + 1).unwrap_or(out.len());
                out.insert(at, line.clone());
                done = true;
            }
            inside = t == "[package]";
        } else if inside {
            let is_key = t.strip_prefix(key).is_some_and(|r| r.trim_start().starts_with('=') || r.starts_with('.'));
            if is_key {
                if !value.trim().is_empty() && !done {
                    out.push(line.clone());
                }
                done = true;
                continue;
            }
            if !t.is_empty() && !t.starts_with('#') {
                last_field = Some(out.len());
            }
        }
        out.push(l.to_string());
    }
    if inside && !done && !value.trim().is_empty() {
        let at = last_field.map(|i| i + 1).unwrap_or(out.len());
        out.insert(at, line);
    }
    out.join("\n") + "\n"
}

/// The table of Cargo.toml the web app's settings are kept in - cargo itself leaves it alone.
pub const WEB_APP_TABLE: &str = "[package.metadata.phai-web]";
/// The project's own settings Phai Studio keeps: the program file's icon.
pub const PHAI_TABLE: &str = "[package.metadata.phai]";

/// A field of `table` (`[package.metadata.phai-web]`) in a Cargo.toml's text, without quotes.
pub fn table_field(text: &str, table: &str, key: &str) -> Option<String> {
    let mut inside = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line == table;
        } else if inside {
            let Some(rest) = line.strip_prefix(key) else { continue };
            let Some(v) = rest.trim_start().strip_prefix('=') else { continue };
            let v = v.trim();
            let v = v.strip_prefix('"').and_then(|v| v.strip_suffix('"')).unwrap_or(v);
            return Some(v.replace("\\\"", "\"").replace("\\\\", "\\"));
        }
    }
    None
}

/// The text with `key` of `table` set (`value` empty: taken out); the table added at the end
/// when there is none.
pub fn with_table_field(text: &str, table: &str, key: &str, value: &str) -> String {
    let line = format!("{key} = \"{}\"", value.trim().replace('\\', "\\\\").replace('"', "\\\""));
    let set = !value.trim().is_empty();
    let mut out: Vec<String> = Vec::new();
    let (mut inside, mut found_table, mut done, mut last_field) = (false, false, false, None);
    for l in text.lines() {
        let t = l.trim();
        if t.starts_with('[') {
            if inside && !done && set {
                let at = last_field.map(|i: usize| i + 1).unwrap_or(out.len());
                out.insert(at, line.clone());
                done = true;
            }
            inside = t == table;
            if inside {
                found_table = true;
                last_field = Some(out.len());
            }
        } else if inside {
            if t.strip_prefix(key).is_some_and(|r| r.trim_start().starts_with('=')) {
                if set && !done {
                    out.push(line.clone());
                }
                done = true;
                continue;
            }
            if !t.is_empty() && !t.starts_with('#') {
                last_field = Some(out.len());
            }
        }
        out.push(l.to_string());
    }
    if inside && !done && set {
        let at = last_field.map(|i| i + 1).unwrap_or(out.len());
        out.insert(at, line.clone());
        done = true;
    }
    if !found_table && set && !done {
        while out.last().is_some_and(|l| l.trim().is_empty()) {
            out.pop();
        }
        out.push(String::new());
        out.push(table.to_string());
        out.push(line);
    }
    out.join("\n") + "\n"
}

#[cfg(test)]
mod web_app_tests {
    use super::*;

    #[test]
    fn the_web_app_s_table_set_changed_and_taken_out() {
        let toml = "[package]\nname = \"shop\"\n\n[dependencies]\nserde = \"1\"\n";
        let t = with_table_field(toml, WEB_APP_TABLE, "name", "ร้าน \"ดี\"");
        assert!(t.ends_with("[package.metadata.phai-web]\nname = \"ร้าน \\\"ดี\\\"\"\n"), "{t}");
        assert_eq!(table_field(&t, WEB_APP_TABLE, "name").as_deref(), Some("ร้าน \"ดี\""));
        let t = with_table_field(&t, WEB_APP_TABLE, "orientation", "portrait");
        let t = with_table_field(&t, WEB_APP_TABLE, "name", "Shop");
        assert!(t.contains("[package.metadata.phai-web]\nname = \"Shop\"\norientation = \"portrait\"\n"), "{t}");
        assert!(t.contains("[dependencies]\nserde = \"1\"\n"), "the rest kept: {t}");
        let t = with_table_field(&t, WEB_APP_TABLE, "name", "");
        assert_eq!(table_field(&t, WEB_APP_TABLE, "name"), None);
        assert_eq!(table_field(&t, WEB_APP_TABLE, "orientation").as_deref(), Some("portrait"));
        assert_eq!(table_field(&t, "[package]", "name").as_deref(), Some("shop"));
    }
}

/// Cargo's arguments for one step of a job: the subcommand and its flags. `whole` is every
/// project of the workspace (Visual Studio's Solution), else the one project `package`.
pub fn job_args(job: Job, whole: bool, package: Option<&str>, release: bool) -> Vec<Vec<String>> {
    let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<String>>();
    let mut clean = s(&["clean"]);
    if !whole {
        if let Some(p) = package {
            clean.extend(s(&["-p", p]));
        }
    }
    #[allow(unused_mut)]
    let mut build = s(&[if job == Job::Analyse { "clippy" } else { "build" }, "--message-format=json"]);
    if whole {
        build.push("--workspace".into());
    }
    if release {
        build.push("--release".into());
        clean.push("--release".into());
    }
    match job {
        Job::Build | Job::Analyse => vec![build],
        Job::Test => {
            build[0] = "test".into();
            if !whole {
                if let Some(p) = package {
                    build.extend(s(&["-p", p]));
                }
            }
            vec![build]
        }
        Job::Rebuild => vec![clean, build],
        Job::Clean => vec![clean],
    }
}

/// One test to run, for the next test job: a ▶ clicked in the editor's margin.
pub static TEST_FILTER: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// Where Cargo is: on the PATH, in `CARGO_HOME`, in the usual place, or in `D:\Tools\Rust`.
pub fn find_cargo() -> Option<PathBuf> {
    // The one Tools > Options names, when it is there.
    if let Some(own) = crate::setting_pub("cargo_path").map(PathBuf::from).filter(|p| p.is_file()) {
        return Some(own);
    }
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
    places.into_iter().map(|d| d.join("cargo.exe")).find(|p| p.is_file())
}

/// Rust code as rustfmt writes it - the project's edition, its `rustfmt.toml` if it has one.
pub fn rustfmt(code: &str, file: &Path) -> Result<String, String> {
    use std::io::Write;
    let exe = find_cargo().map(|c| c.with_file_name("rustfmt.exe")).filter(|p| p.is_file()).ok_or("rustfmt is not installed: rustup component add rustfmt")?;
    let project = project_of(file);
    let edition = project
        .as_ref()
        .and_then(|d| std::fs::read_to_string(d.join("Cargo.toml")).ok())
        .and_then(|t| t.lines().find_map(|l| l.trim().strip_prefix("edition").and_then(|v| v.trim_start().strip_prefix('=')).map(|v| v.trim().trim_matches('"').to_string())))
        .filter(|e| e.chars().all(|c| c.is_ascii_digit()))
        .unwrap_or_else(|| "2021".into());
    let mut c = std::process::Command::new(exe);
    c.args(["--edition", &edition, "--emit", "stdout", "--quiet"]);
    if let Some(d) = &project {
        c.current_dir(d);
    }
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let mut child = c.stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn().map_err(|e| e.to_string())?;
    child.stdin.take().ok_or("no stdin")?.write_all(code.as_bytes()).map_err(|e| e.to_string())?;
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(err.lines().find(|l| l.starts_with("error")).unwrap_or("rustfmt could not read this code").to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n"))
}

/// The folder with the `Cargo.toml` that holds `file`.
pub fn project_of(file: &Path) -> Option<PathBuf> {
    file.parent()?.ancestors().find(|d| d.join("Cargo.toml").is_file()).map(Path::to_path_buf)
}

/// The Cargo workspace a file is in: the folder whose `Cargo.toml` has `[workspace]` above it, or
/// its own project when it is in none.
pub fn workspace_of(file: &Path) -> Option<PathBuf> {
    let own = project_of(file)?;
    let top = own.ancestors().filter(|d| std::fs::read_to_string(d.join("Cargo.toml")).is_ok_and(|t| t.lines().any(|l| l.trim() == "[workspace]"))).last();
    Some(top.map(Path::to_path_buf).unwrap_or(own))
}

/// Whether a file is someone else's code - the standard library, or a crate Cargo fetched - which
/// the language server of the project that uses it already knows.
pub fn is_library(file: &Path) -> bool {
    let p = file.to_string_lossy().replace('/', "\\").to_ascii_lowercase();
    ["\\toolchains\\", "\\registry\\src\\", "\\git\\checkouts\\"].iter().any(|m| p.contains(m))
}

#[derive(Deserialize)]
struct Line {
    reason: String,
    #[serde(default)]
    message: Option<Message>,
    #[serde(default)]
    executable: Option<PathBuf>,
    #[serde(default)]
    success: Option<bool>,
    #[serde(default)]
    manifest_path: Option<PathBuf>,
}

#[derive(Deserialize)]
struct Message {
    level: String,
    message: String,
    #[serde(default)]
    spans: Vec<Span>,
}

#[derive(Deserialize)]
struct Span {
    file_name: String,
    line_start: u32,
    column_start: u32,
    is_primary: bool,
}

/// What one line of Cargo's JSON adds to `out`. Lines that are not JSON are Cargo's own words.
pub fn take_line(line: &str, project: &Path, wanted_manifest: &Path, out: &mut Outcome, other: &mut Vec<String>) {
    let Ok(l) = serde_json::from_str::<Line>(line) else {
        if !line.trim().is_empty() {
            other.push(line.trim().to_string());
        }
        return;
    };
    match l.reason.as_str() {
        "compiler-message" => {
            let Some(m) = l.message else { return };
            let error = m.level == "error";
            if !error && m.level != "warning" {
                return;
            }
            // "aborting due to 2 previous errors" and "N warnings emitted" say nothing new.
            if m.spans.is_empty() && (m.message.starts_with("aborting due to") || m.message.contains("warning emitted") || m.message.contains("warnings emitted")) {
                return;
            }
            let span = m.spans.iter().find(|s| s.is_primary).or(m.spans.first());
            let d = Diagnostic {
                error,
                message: m.message,
                file: span.map(|s| {
                    let p = PathBuf::from(&s.file_name);
                    if p.is_absolute() {
                        p
                    } else {
                        project.join(p)
                    }
                }),
                line: span.map(|s| s.line_start).unwrap_or(0),
                column: span.map(|s| s.column_start).unwrap_or(0),
            };
            if !out.diagnostics.contains(&d) {
                out.diagnostics.push(d);
            }
        }
        "compiler-artifact" => {
            if l.executable.is_some() && l.manifest_path.as_deref().is_none_or(|m| same_file(m, wanted_manifest)) {
                out.executable = l.executable;
            }
        }
        "build-finished" => out.ok = l.success.unwrap_or(false),
        _ => {}
    }
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(x), Ok(y)) => x == y,
        _ => a == b,
    }
}

/// Does `job` to `project` (all of its workspace when `whole`) on another thread, each step in
/// turn, and posts [`WM_BUILD_DONE`] to `window` when it ends.
pub fn run_job(window: HWND, project: PathBuf, job: Job, whole: bool, then_run: bool, release: bool) {
    let target = window.0 as isize;
    std::thread::spawn(move || {
        let started = std::time::Instant::now();
        let mut out = Outcome { then_run, job, ok: true, ..Default::default() };
        let package = package_name(&project);
        let one = if job == Job::Test { TEST_FILTER.lock().unwrap_or_else(|e| e.into_inner()).take() } else { None };
        for mut args in job_args(job, whole, package.as_deref(), release) {
            // One test only: its path ends so (`web::tests::name` in `app::web::tests::name`).
            if let Some(name) = &one {
                args.push(name.clone());
            }
            step(target, &project, &args, &mut out);
            if !out.ok || out.no_cargo {
                break;
            }
        }
        out.seconds = started.elapsed().as_secs_f32();
        let boxed = Box::into_raw(Box::new(out));
        unsafe {
            if wm::PostMessageW(Some(HWND(target as *mut _)), WM_BUILD_DONE, WPARAM(0), LPARAM(boxed as isize)).is_err() {
                drop(Box::from_raw(boxed));
            }
        }
    });
}

/// The projects of a workspace: the folders its `[workspace] members` name (a `*` at the end of
/// one taken as every folder with a Cargo.toml there), or the workspace itself when it is a
/// package with none.
pub fn workspace_members(ws: &Path) -> Vec<PathBuf> {
    let text = std::fs::read_to_string(ws.join("Cargo.toml")).unwrap_or_default();
    let mut out = Vec::new();
    if let Some(i) = text.find("members") {
        let rest = &text[i..];
        if let (Some(a), Some(b)) = (rest.find('['), rest.find(']')) {
            for m in rest[a + 1..b].split(',').map(|s| s.trim().trim_matches('"')).filter(|s| !s.is_empty()) {
                if let Some(dir) = m.strip_suffix("/*") {
                    let mut found: Vec<PathBuf> = std::fs::read_dir(ws.join(dir)).map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.join("Cargo.toml").is_file()).collect()).unwrap_or_default();
                    found.sort();
                    out.extend(found);
                } else if ws.join(m).join("Cargo.toml").is_file() {
                    out.push(ws.join(m));
                }
            }
        }
    }
    if out.is_empty() && package_name(ws).is_some() {
        out.push(ws.to_path_buf());
    }
    out
}

/// Batch Build: each (project, Release?) built one after another with `job`, as one outcome -
/// every diagnostic, ok only when all were.
pub fn run_batch(window: HWND, list: Vec<(PathBuf, bool)>, job: Job) {
    let target = window.0 as isize;
    std::thread::spawn(move || {
        let started = std::time::Instant::now();
        let mut all = Outcome { job, ok: true, ..Default::default() };
        for (project, release) in list {
            let mut out = Outcome { job, ok: true, ..Default::default() };
            let package = package_name(&project);
            post_line(target, format!("------ {} {} ------", package.as_deref().unwrap_or("?"), if release { "Release" } else { "Debug" }));
            for args in job_args(job, false, package.as_deref(), release) {
                step(target, &project, &args, &mut out);
                if !out.ok || out.no_cargo {
                    break;
                }
            }
            all.ok &= out.ok;
            all.no_cargo |= out.no_cargo;
            all.diagnostics.extend(out.diagnostics);
            if out.cargo_error.is_some() {
                all.cargo_error = out.cargo_error;
            }
            if out.no_cargo {
                break;
            }
        }
        all.seconds = started.elapsed().as_secs_f32();
        let boxed = Box::into_raw(Box::new(all));
        unsafe {
            if wm::PostMessageW(Some(HWND(target as *mut _)), WM_BUILD_DONE, WPARAM(0), LPARAM(boxed as isize)).is_err() {
                drop(Box::from_raw(boxed));
            }
        }
    });
}

/// One run of Cargo: its output to the Output pane, its JSON diagnostics into `out`.
fn step(target: isize, project: &Path, args: &[String], out: &mut Outcome) {
    {
        match find_cargo() {
            None => out.no_cargo = true,
            Some(cargo) => {
                let manifest = project.join("Cargo.toml");
                post_line(target, format!("> cargo {} --manifest-path {}", args.join(" "), manifest.display()));
                out.ok = false;
                let child = Command::new(cargo)
                    .args(args)
                    .arg("--manifest-path")
                    .arg(&manifest)
                    .current_dir(project)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn();
                match child {
                    Err(_) => out.no_cargo = true,
                    Ok(mut child) => {
                        // Cargo's progress and its own errors come on stderr; read it alongside.
                        let stderr = child.stderr.take();
                        let errs = std::thread::spawn(move || {
                            let mut kept = Vec::new();
                            if let Some(s) = stderr {
                                for line in BufReader::new(s).lines().map_while(Result::ok) {
                                    post_line(target, line.clone());
                                    kept.push(line);
                                }
                            }
                            kept
                        });
                        let mut other = Vec::new();
                        if let Some(stdout) = child.stdout.take() {
                            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                                take_line(&line, project, &manifest, out, &mut other);
                            }
                        }
                        // A test run's words: shown as they are, and read for each test.
                        for l in &other {
                            post_line(target, l.clone());
                        }
                        out.tests.extend(tests_of(&other));
                        let status = child.wait().ok();
                        let err_lines = errs.join().unwrap_or_default();
                        out.ok = status.is_some_and(|s| s.success());
                        if !out.ok {
                            out.ok = false;
                            if out.diagnostics.is_empty() {
                                let words: Vec<&String> = err_lines.iter().filter(|l| l.contains("error")).collect();
                                out.cargo_error = Some(if words.is_empty() { err_lines.join("\n") } else { words.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n") });
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Starts the built program, its folder as the working one.
pub fn run(exe: &Path, target: HWND) -> std::io::Result<Child> {
    let mut c = Command::new(exe);
    if let Some(dir) = exe.parent() {
        c.current_dir(dir);
    }
    // No console window: what the program prints goes to the Output pane, as in Visual Studio.
    let mut child = c.stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::null()).creation_flags(CREATE_NO_WINDOW).spawn()?;
    let target = target.0 as isize;
    if let Some(out) = child.stdout.take() {
        std::thread::spawn(move || BufReader::new(out).lines().map_while(Result::ok).for_each(|l| post_line(target, l)));
    }
    if let Some(err) = child.stderr.take() {
        std::thread::spawn(move || BufReader::new(err).lines().map_while(Result::ok).for_each(|l| post_line(target, l)));
    }
    Ok(child)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_json_becomes_diagnostics_and_the_program_path() {
        let project = Path::new(r"C:\p");
        let manifest = project.join("Cargo.toml");
        let mut out = Outcome::default();
        let mut other = Vec::new();
        let lines = [
            r#"{"reason":"compiler-message","message":{"level":"error","message":"cannot find value `x` in this scope","spans":[{"file_name":"src\\main.rs","line_start":7,"column_start":13,"is_primary":true}]}}"#,
            r#"{"reason":"compiler-message","message":{"level":"warning","message":"unused variable: `y`","spans":[{"file_name":"src\\main.rs","line_start":3,"column_start":9,"is_primary":true}]}}"#,
            r#"{"reason":"compiler-message","message":{"level":"error","message":"aborting due to 1 previous error","spans":[]}}"#,
            r#"{"reason":"compiler-message","message":{"level":"note","message":"a note","spans":[]}}"#,
            r#"{"reason":"compiler-artifact","executable":"C:\\p\\target\\debug\\app.exe"}"#,
            r#"{"reason":"build-finished","success":false}"#,
            "   Compiling app v0.1.0",
        ];
        for l in lines {
            take_line(l, project, &manifest, &mut out, &mut other);
        }
        assert_eq!(out.diagnostics.len(), 2, "the error and the warning; not the summary or the note");
        assert!(out.diagnostics[0].error);
        assert_eq!(out.diagnostics[0].file.as_deref(), Some(Path::new(r"C:\p\src\main.rs")));
        assert_eq!((out.diagnostics[0].line, out.diagnostics[0].column), (7, 13));
        assert!(!out.diagnostics[1].error);
        assert_eq!(out.executable.as_deref(), Some(Path::new(r"C:\p\target\debug\app.exe")));
        assert!(!out.ok);
        assert_eq!(other, vec!["Compiling app v0.1.0".to_string()]);
    }

    #[test]
    fn the_project_is_the_nearest_folder_with_a_cargo_toml() {
        let form = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../sample/src/main_window.phf"));
        let p = project_of(form).unwrap();
        assert!(p.join("Cargo.toml").is_file());
        assert!(p.ends_with("sample"), "{}", p.display());
    }

    #[test]
    fn build_menu_jobs_are_cargos_commands() {
        let a = job_args(Job::Rebuild, false, Some("sample"), false);
        assert_eq!(a[0], ["clean", "-p", "sample"]);
        assert_eq!(a[1], ["build", "--message-format=json"]);
        let a = job_args(Job::Analyse, true, Some("x"), true);
        assert_eq!(a, vec![vec!["clippy", "--message-format=json", "--workspace", "--release"]]);
        assert_eq!(job_args(Job::Clean, true, Some("x"), false), vec![vec!["clean"]]);
    }

    #[test]
    fn test_results_are_read_with_why_they_failed() {
        let lines: Vec<String> = [
            "running 3 tests",
            "test model::tests::adds ... ok",
            "test model::tests::slow ... ignored, needs a database",
            "test model::tests::breaks ... FAILED",
            "failures:",
            "---- model::tests::breaks stdout ----",
            "thread 'model::tests::breaks' panicked at src\\model.rs:9:5:",
            "assertion `left == right` failed",
            "test result: FAILED. 1 passed; 1 failed; 1 ignored",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let t = tests_of(&lines);
        assert_eq!(t.len(), 3);
        assert_eq!((t[0].name.as_str(), t[0].outcome.as_str()), ("model::tests::adds", "ok"));
        assert_eq!(t[1].outcome, "ignored");
        assert_eq!((t[2].outcome.as_str(), t[2].message.as_str()), ("FAILED", "assertion `left == right` failed"));
        assert_eq!(job_args(Job::Test, false, Some("app"), false), vec![vec!["test".to_string(), "--message-format=json".into(), "-p".into(), "app".into()]]);
    }

    #[test]
    fn a_workspace_lists_its_projects() {
        // This repository's own workspace, from the studio's folder.
        let ws = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
        let names: Vec<String> = workspace_members(&ws).iter().filter_map(|p| package_name(p)).collect();
        for want in ["phai-ui", "phai-studio", "phai-setup", "phai-pack"] {
            assert!(names.iter().any(|n| n == want), "{want} in {names:?}");
        }
    }

    #[test]
    fn package_fields_are_set_in_place() {
        let toml = "[package]\nname = \"shop\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nregex = \"1\"\n";
        let t = with_package_field(toml, "version", "1.2.0", false);
        assert!(t.contains("version = \"1.2.0\"") && !t.contains("0.1.0"));
        let t = with_package_field(&t, "description", "A shop", false);
        assert!(t.find("description").unwrap() < t.find("[dependencies]").unwrap(), "a new field stays in [package]");
        let t = with_package_field(&t, "authors", "Monrak Net, Phai", true);
        assert!(t.contains("authors = [\"Monrak Net\", \"Phai\"]"));
        let t = with_package_field(&t, "description", "", false);
        assert!(!t.contains("description"), "empty takes it out");
        assert!(t.contains("regex = \"1\""), "the rest kept");
        // version.workspace = true is replaced by a version of its own.
        let w = with_package_field("[package]\nname = \"a\"\nversion.workspace = true\n", "version", "2.0.0", false);
        assert!(w.contains("version = \"2.0.0\"") && !w.contains("workspace"));
    }
}
