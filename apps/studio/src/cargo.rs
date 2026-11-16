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

/// The folder with the `Cargo.toml` that holds `file`.
pub fn project_of(file: &Path) -> Option<PathBuf> {
    file.parent()?.ancestors().find(|d| d.join("Cargo.toml").is_file()).map(Path::to_path_buf)
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

/// Builds `project` on another thread and posts [`WM_BUILD_DONE`] to `window` when it ends.
pub fn build(window: HWND, project: PathBuf, then_run: bool) {
    let target = window.0 as isize;
    std::thread::spawn(move || {
        let started = std::time::Instant::now();
        let mut out = Outcome { then_run, ..Default::default() };
        match find_cargo() {
            None => out.no_cargo = true,
            Some(cargo) => {
                let manifest = project.join("Cargo.toml");
                let child = Command::new(cargo)
                    .args(["build", "--message-format=json", "--manifest-path"])
                    .arg(&manifest)
                    .current_dir(&project)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn();
                match child {
                    Err(_) => out.no_cargo = true,
                    Ok(mut child) => {
                        // Cargo's progress and its own errors come on stderr; read it alongside.
                        let stderr = child.stderr.take();
                        let errs = std::thread::spawn(move || stderr.map(|s| BufReader::new(s).lines().map_while(Result::ok).collect::<Vec<_>>()).unwrap_or_default());
                        let mut other = Vec::new();
                        if let Some(stdout) = child.stdout.take() {
                            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                                take_line(&line, &project, &manifest, &mut out, &mut other);
                            }
                        }
                        let status = child.wait().ok();
                        let err_lines = errs.join().unwrap_or_default();
                        if !status.is_some_and(|s| s.success()) {
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
        out.seconds = started.elapsed().as_secs_f32();
        let boxed = Box::into_raw(Box::new(out));
        unsafe {
            if wm::PostMessageW(Some(HWND(target as *mut _)), WM_BUILD_DONE, WPARAM(0), LPARAM(boxed as isize)).is_err() {
                drop(Box::from_raw(boxed));
            }
        }
    });
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
}
