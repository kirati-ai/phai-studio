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
        Job::Rebuild => vec![clean, build],
        Job::Clean => vec![clean],
    }
}

/// Where Cargo is: on the PATH, in `CARGO_HOME`, in the usual place, or in `D:\Tools\Rust`.
pub fn find_cargo() -> Option<PathBuf> {
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
        for args in job_args(job, whole, package.as_deref(), release) {
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
pub fn run(exe: &Path) -> std::io::Result<Child> {
    let mut c = Command::new(exe);
    if let Some(dir) = exe.parent() {
        c.current_dir(dir);
    }
    c.spawn()
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
        let form = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../sample/src/main_window.phaiform"));
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
}
