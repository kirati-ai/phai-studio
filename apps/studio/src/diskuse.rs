//! Tools > Disk Space of Builds: how much room each project's builds (its `target` folder) take,
//! when it was last built, and those ticked emptied - they come back on the next build. Only a
//! folder cargo made (it leaves a CACHEDIR.TAG in it) is ever taken away.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// One project's builds.
#[derive(Debug, Clone, PartialEq)]
pub struct Builds {
    /// The project (or the workspace) whose builds they are.
    pub project: PathBuf,
    /// Its `target` folder.
    pub target: PathBuf,
    pub bytes: u64,
    /// Days since it was last built; `None` when that cannot be told.
    pub days: Option<u64>,
}

/// A folder cargo made for builds: it puts a CACHEDIR.TAG in each.
pub fn made_by_cargo(target: &Path) -> bool {
    target.join("CACHEDIR.TAG").is_file()
}

/// The builds folder of a project: its own `target`, or its workspace's a few folders up.
pub fn target_of(project: &Path) -> Option<(PathBuf, PathBuf)> {
    project.ancestors().take(4).find(|d| made_by_cargo(&d.join("target"))).map(|d| (d.to_path_buf(), d.join("target")))
}

/// The projects to look at: the recent ones and the other Rust projects beside them (in the same
/// folder, one level), then the shared builds folder `CARGO_TARGET_DIR` names.
pub fn projects_near(recent: &[PathBuf]) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let mut add = |p: PathBuf| {
        if !out.contains(&p) {
            out.push(p);
        }
    };
    for p in recent {
        add(p.clone());
    }
    let parents: Vec<PathBuf> = recent.iter().filter_map(|p| p.parent().map(Path::to_path_buf)).collect();
    for parent in parents {
        let Ok(rd) = std::fs::read_dir(&parent) else { continue };
        let mut found: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|d| d.join("Cargo.toml").is_file()).collect();
        found.sort();
        for p in found {
            add(p);
        }
    }
    out
}

/// Each builds folder of `projects` once, measured, the biggest first; with the shared one
/// `shared` names (CARGO_TARGET_DIR) when cargo made it.
pub fn measure(projects: &[PathBuf], shared: Option<PathBuf>) -> Vec<Builds> {
    let mut out: Vec<Builds> = Vec::new();
    let mut found: Vec<(PathBuf, PathBuf)> = projects.iter().filter_map(|p| target_of(p)).collect();
    if let Some(s) = shared.filter(|s| made_by_cargo(s)) {
        found.push((s.clone(), s));
    }
    for (project, target) in found {
        if out.iter().any(|b| b.target == target) {
            continue;
        }
        let bytes = size_of(&target);
        out.push(Builds { project, days: days_since_built(&target), target, bytes });
    }
    out.sort_by_key(|b| std::cmp::Reverse(b.bytes));
    out
}

/// The bytes of every file under `dir`, links not followed.
pub fn size_of(dir: &Path) -> u64 {
    let mut total = 0;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let Ok(kind) = e.file_type() else { continue };
            if kind.is_dir() {
                stack.push(e.path());
            } else if kind.is_file() {
                total += e.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    total
}

/// Days since the last build: cargo touches `.cargo-lock` in a profile's folder each time.
pub fn days_since_built(target: &Path) -> Option<u64> {
    let mut locks: Vec<PathBuf> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(target) {
        for e in rd.flatten() {
            let p = e.path();
            locks.push(p.join(".cargo-lock"));
            // A --target build: target\<triple>\<profile>\.cargo-lock.
            if let Ok(inner) = std::fs::read_dir(&p) {
                locks.extend(inner.flatten().map(|i| i.path().join(".cargo-lock")));
            }
        }
    }
    let when = |p: &Path| std::fs::metadata(p).and_then(|m| m.modified()).ok();
    let newest: Option<SystemTime> = locks.iter().filter_map(|p| when(p)).max().or_else(|| when(target));
    newest.and_then(|t| SystemTime::now().duration_since(t).ok()).map(|d| d.as_secs() / 86_400)
}

/// The builds folder taken away, the bytes it freed. Refused for a folder cargo did not make.
pub fn empty(target: &Path) -> Result<u64, String> {
    if !made_by_cargo(target) {
        return Err(format!("{} is not a builds folder cargo made", target.display()));
    }
    let bytes = size_of(target);
    std::fs::remove_dir_all(target).map_err(|e| format!("{}: {e}", target.display()))?;
    Ok(bytes)
}

/// A folder short enough for a line: the home folder as `~`, and a long one as its start and its
/// last two folders with `...` between.
pub fn short_path(p: &Path, home: Option<&Path>) -> String {
    let full = match home.and_then(|h| p.strip_prefix(h).ok()) {
        Some(rest) => Path::new("~").join(rest),
        None => p.to_path_buf(),
    };
    let text = full.display().to_string();
    let parts: Vec<String> = full.components().filter(|c| !matches!(c, std::path::Component::RootDir)).map(|c| c.as_os_str().to_string_lossy().to_string()).collect();
    if text.chars().count() <= 48 || parts.len() <= 4 {
        return text;
    }
    format!(r"{}\...\{}\{}", parts[0], parts[parts.len() - 2], parts[parts.len() - 1])
}

/// Bytes as people say them: 512 KB, 3.4 MB, 2.1 GB.
pub fn human(bytes: u64) -> String {
    const K: f64 = 1024.0;
    let b = bytes as f64;
    if b < K * K {
        format!("{:.0} KB", (b / K).ceil())
    } else if b < K * K * K {
        format!("{:.1} MB", b / (K * K))
    } else {
        format!("{:.1} GB", b / (K * K * K))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(root: &Path, name: &str, target_bytes: usize) -> PathBuf {
        let p = root.join(name);
        std::fs::create_dir_all(p.join("src")).unwrap();
        std::fs::write(p.join("Cargo.toml"), format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n")).unwrap();
        if target_bytes > 0 {
            let debug = p.join("target").join("debug");
            std::fs::create_dir_all(debug.join("deps")).unwrap();
            std::fs::write(p.join("target").join("CACHEDIR.TAG"), "Signature: 8a477f597d28d172789f06886806bc55\n").unwrap();
            std::fs::write(debug.join(".cargo-lock"), "").unwrap();
            std::fs::write(debug.join("deps").join("big.rlib"), vec![0u8; target_bytes]).unwrap();
        }
        p
    }

    #[test]
    fn the_projects_beside_a_recent_one_measured_biggest_first_and_emptied() {
        let root = std::env::temp_dir().join(format!("phai-disk-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let shop = project(&root, "shop", 3000);
        let _books = project(&root, "books", 9000);
        let _clean = project(&root, "clean", 0);
        // A workspace member: its builds are the workspace's.
        let ws = project(&root, "ws", 5000);
        let member = ws.join("crates").join("part");
        std::fs::create_dir_all(&member).unwrap();
        std::fs::write(member.join("Cargo.toml"), "[package]\nname = \"part\"\n").unwrap();
        // Not cargo's: no CACHEDIR.TAG, so never counted nor taken.
        let odd = root.join("odd");
        std::fs::create_dir_all(odd.join("target")).unwrap();
        std::fs::write(odd.join("Cargo.toml"), "").unwrap();
        std::fs::write(odd.join("target").join("keep.txt"), "mine").unwrap();

        let near = projects_near(std::slice::from_ref(&shop));
        assert_eq!(near[0], shop, "the recent one first");
        assert!(near.contains(&root.join("books")) && near.contains(&odd));
        let mut list = projects_near(&[shop.clone(), member.clone()]);
        list.dedup();
        let b = measure(&list, None);
        let names: Vec<String> = b.iter().map(|x| x.project.file_name().unwrap().to_string_lossy().to_string()).collect();
        assert_eq!(names, ["books", "ws", "shop"], "{b:?}");
        assert_eq!(b[0].bytes, 9000 + 44, "the files and the tag");
        assert_eq!(b[0].days, Some(0));
        assert_eq!(b.iter().filter(|x| x.project == ws).count(), 1, "the member's builds are the workspace's, once");

        assert_eq!(empty(&shop.join("target")), Ok(3000 + 44));
        assert!(!shop.join("target").exists() && shop.join("src").exists());
        assert!(empty(&odd.join("target")).is_err());
        assert!(odd.join("target").join("keep.txt").is_file(), "a folder cargo did not make stays");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_long_folder_shortened_for_its_line() {
        let home = Path::new(r"C:\Users\kim");
        assert_eq!(short_path(Path::new(r"D:\phai-tmp\shop"), Some(home)), r"D:\phai-tmp\shop");
        assert_eq!(short_path(Path::new(r"C:\Users\kim\work\shop"), Some(home)), r"~\work\shop");
        assert_eq!(short_path(Path::new(r"C:\Users\kim\AppData\Local\Temp\a1b2c3d4e5f6\scratchpad\tpl-business"), Some(home)), r"~\...\scratchpad\tpl-business");
        assert_eq!(short_path(Path::new(r"D:\Project.NET\phai-studio\docs\help-src\examples\form"), None), r"D:\...\examples\form");
    }

    #[test]
    fn sizes_as_people_say_them() {
        assert_eq!(human(0), "0 KB");
        assert_eq!(human(1), "1 KB");
        assert_eq!(human(512 * 1024), "512 KB");
        assert_eq!(human(3 * 1024 * 1024 + 400 * 1024), "3.4 MB");
        assert_eq!(human(2_254_857_830), "2.1 GB");
    }
}
