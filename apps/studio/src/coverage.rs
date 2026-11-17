//! Code Coverage, as Visual Studio's Analyze Code Coverage: the project's tests run once with
//! Rust's own instrumentation (`-C instrument-coverage`), then LLVM's tools - Rust's free
//! `llvm-tools` component - say which lines ran and which did not.

use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use phai_ui::text::t;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// A source file's lines as the tests left them: each line with code, and how many times it ran.
#[derive(Clone, Debug, PartialEq)]
pub struct FileCoverage {
    pub path: PathBuf,
    pub lines: Vec<(u32, u64)>,
}

impl FileCoverage {
    pub fn covered(&self) -> usize {
        self.lines.iter().filter(|(_, n)| *n > 0).count()
    }
}

/// An lcov report read: each file, its lines and their counts.
pub fn lcov_of(text: &str) -> Vec<FileCoverage> {
    let mut out = Vec::new();
    let mut cur: Option<FileCoverage> = None;
    for l in text.lines() {
        if let Some(p) = l.strip_prefix("SF:") {
            cur = Some(FileCoverage { path: PathBuf::from(p), lines: vec![] });
        } else if let Some(d) = l.strip_prefix("DA:") {
            let mut it = d.split(',');
            if let (Some(c), Some(line), Some(n)) = (cur.as_mut(), it.next().and_then(|v| v.parse().ok()), it.next().and_then(|v| v.parse().ok())) {
                c.lines.push((line, n));
            }
        } else if l == "end_of_record" {
            if let Some(c) = cur.take() {
                out.push(c);
            }
        }
    }
    out
}

/// LLVM's tool `name` from Rust's `llvm-tools` component, if it is installed.
fn llvm_tool(cargo: &Path, name: &str) -> Option<PathBuf> {
    let rustc = cargo.with_file_name("rustc.exe");
    let out = Command::new(&rustc).args(["--print", "sysroot"]).creation_flags(CREATE_NO_WINDOW).output().ok()?;
    let sysroot = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    let host = Command::new(&rustc).arg("-vV").creation_flags(CREATE_NO_WINDOW).output().ok()?;
    let host = String::from_utf8_lossy(&host.stdout).lines().find_map(|l| l.strip_prefix("host: ").map(str::to_string))?;
    let p = sysroot.join("lib").join("rustlib").join(host).join("bin").join(format!("{name}.exe"));
    p.is_file().then_some(p)
}

/// Whether `llvm-tools` is there; when not, the words that say how to get it.
pub fn tools_ready(cargo: &Path) -> Result<(), String> {
    match (llvm_tool(cargo, "llvm-profdata"), llvm_tool(cargo, "llvm-cov")) {
        (Some(_), Some(_)) => Ok(()),
        _ => Err(t("Code coverage needs Rust's llvm-tools component (free). Install it with: rustup component add llvm-tools").to_string()),
    }
}

/// The project's tests run with coverage; each of its own source files' lines and counts.
pub fn run(project: &Path, cargo: &Path) -> Result<Vec<FileCoverage>, String> {
    tools_ready(cargo)?;
    let target = project.join("target").join("coverage");
    let raw = target.join("profraw");
    let _ = std::fs::remove_dir_all(&raw);
    std::fs::create_dir_all(&raw).map_err(|e| e.to_string())?;
    let cargo_with = |args: &[&str]| {
        let mut c = Command::new(cargo);
        c.args(args)
            .arg("--target-dir")
            .arg(&target)
            .current_dir(project)
            .env("RUSTFLAGS", "-C instrument-coverage")
            .env("CARGO_INCREMENTAL", "0")
            .env("LLVM_PROFILE_FILE", raw.join("%p-%m.profraw"))
            .creation_flags(CREATE_NO_WINDOW);
        c
    };
    // The tests run - failing ones too: what ran still counts.
    let ran = cargo_with(&["test", "--quiet"]).output().map_err(|e| e.to_string())?;
    let built = cargo_with(&["test", "--no-run", "--message-format=json"]).output().map_err(|e| e.to_string())?;
    let objects: Vec<String> = String::from_utf8_lossy(&built.stdout)
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .filter(|v| v["profile"]["test"].as_bool() == Some(true))
        .filter_map(|v| v["executable"].as_str().map(str::to_string))
        .collect();
    let profiles: Vec<PathBuf> = std::fs::read_dir(&raw).map(|d| d.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "profraw")).collect()).unwrap_or_default();
    if objects.is_empty() || profiles.is_empty() {
        let why = String::from_utf8_lossy(&ran.stderr).lines().filter(|l| l.contains("error")).take(3).collect::<Vec<_>>().join("\n");
        return Err(if why.is_empty() { t("The project has no tests to measure.").to_string() } else { why });
    }
    let data = target.join("coverage.profdata");
    let merged = Command::new(llvm_tool(cargo, "llvm-profdata").unwrap_or_default()).arg("merge").arg("-sparse").args(&profiles).arg("-o").arg(&data).creation_flags(CREATE_NO_WINDOW).output().map_err(|e| e.to_string())?;
    if !merged.status.success() {
        return Err(String::from_utf8_lossy(&merged.stderr).to_string());
    }
    let mut cov = Command::new(llvm_tool(cargo, "llvm-cov").unwrap_or_default());
    cov.args(["export", "-format=lcov"]).arg(format!("-instr-profile={}", data.display())).arg(&objects[0]);
    for o in &objects[1..] {
        cov.arg("-object").arg(o);
    }
    // Only the project's own code: not the standard library's, nor other crates'.
    cov.arg(r"-ignore-filename-regex=(\\rustc\\|/rustc/|\\.cargo\\|\\registry\\|\\target\\)");
    let out = cov.creation_flags(CREATE_NO_WINDOW).output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }
    let project = std::path::absolute(project).unwrap_or(project.to_path_buf());
    let mut files: Vec<FileCoverage> = lcov_of(&String::from_utf8_lossy(&out.stdout)).into_iter().filter(|f| f.path.starts_with(&project)).collect();
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lcov_is_read() {
        let f = lcov_of("SF:C:\\p\\src\\lib.rs\nFN:1,add\nDA:1,3\nDA:2,3\nDA:5,0\nLF:3\nLH:2\nend_of_record\n");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].lines, vec![(1, 3), (2, 3), (5, 0)]);
        assert_eq!(f[0].covered(), 2);
    }
}
