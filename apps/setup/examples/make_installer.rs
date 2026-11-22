//! Makes Phai Studio's own setup program: `cargo run -p phai-setup --example make_installer
//! --release` after `cargo build --release -p phai-studio -p phai-setup`. It packs, onto the
//! setup program, Phai Studio, the setup program again (Publish needs it beside Phai Studio), and
//! phai-ui's source - the library every new project builds with - its Cargo.toml made to stand
//! alone. Out comes `dist/PhaiStudio-<version>-setup.exe`.

use std::path::{Path, PathBuf};

/// phai-ui's Cargo.toml without the workspace: each `workspace = true` given the workspace's value.
fn standalone(manifest: &str, version: &str) -> String {
    manifest
        .lines()
        .map(|l| match l.trim() {
            "version.workspace = true" => format!("version = \"{version}\""),
            "edition.workspace = true" => "edition = \"2021\"".to_string(),
            "publish.workspace = true" => "publish = false".to_string(),
            "license.workspace = true" => "license = \"MIT\"".to_string(),
            "authors.workspace = true" => "authors = [\"Monrak Net\"]".to_string(),
            _ => l.replace("png = { workspace = true", "png = { version = \"0.17\"").replace("windows = { workspace = true", "windows = { version = \"0.62\""),
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

/// Every file under `dir` but Cargo's own output, by its path under `base` with `/`.
fn files_under(dir: &Path, base: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for p in rd.flatten().map(|e| e.path()) {
        let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
        if p.is_dir() {
            if name != "target" && !name.starts_with('.') {
                files_under(&p, base, out);
            }
        } else if let (Ok(data), Ok(rel)) = (std::fs::read(&p), p.strip_prefix(base)) {
            out.push((rel.to_string_lossy().replace('\\', "/"), data));
        }
    }
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let release = root.join("target").join("release");
    let read = |p: PathBuf| std::fs::read(&p).unwrap_or_else(|e| panic!("{}: {e} - build it first with --release", p.display()));
    let ws = std::fs::read_to_string(root.join("Cargo.toml")).unwrap_or_default();
    let version = ws
        .lines()
        .skip_while(|l| l.trim() != "[workspace.package]")
        .find_map(|l| l.trim().strip_prefix("version").and_then(|v| v.trim_start().strip_prefix('=')).map(|v| v.trim().trim_matches('"').to_string()))
        .unwrap_or_else(|| "0.1.0".into());
    let stub = read(release.join("phai-setup.exe"));
    let mut files = vec![("phai-studio.exe".to_string(), read(release.join("phai-studio.exe"))), ("phai-setup.exe".to_string(), stub.clone())];
    // phai-ui's source under phai-ui/, phai-web's (the same program in a browser) under phai-web/,
    // and phai-data's (a form bound to a database table) under
    // phai-data/, each Cargo.toml standing alone.
    for lib in ["phai-ui", "phai-data", "phai-web"] {
        let dir = root.join("crates").join(lib);
        let mut lib_files = Vec::new();
        files_under(&dir, &dir, &mut lib_files);
        for (rel, data) in lib_files {
            let data = if rel == "Cargo.toml" { standalone(&String::from_utf8_lossy(&data), &version).into_bytes() } else { data };
            files.push((format!("{lib}/{rel}"), data));
        }
    }
    let _ = std::fs::read(root.join("LICENSE")).map(|l| files.push(("LICENSE.txt".into(), l)));
    for text in ["MIT.txt", "PolyForm-Perimeter-1.0.1.md"] {
        let _ = std::fs::read(root.join("LICENSES").join(text)).map(|l| files.push((format!("LICENSES/{text}"), l)));
    }
    // The Thai guide, opened by Help > Phai Studio Guide (F1) from help\ beside the program - its
    // folders too (the Toolbox pages' icons/ and img/).
    fn add_dir(dir: &std::path::Path, under: &str, files: &mut Vec<(String, Vec<u8>)>) {
        let Ok(read) = std::fs::read_dir(dir) else { return };
        for e in read.flatten() {
            let Some(name) = e.file_name().to_str().map(str::to_string) else { continue };
            let path = e.path();
            if path.is_dir() {
                add_dir(&path, &format!("{under}/{name}"), files);
            } else if let Ok(data) = std::fs::read(&path) {
                files.push((format!("{under}/{name}"), data));
            }
        }
    }
    add_dir(&root.join("docs").join("help"), "help", &mut files);
    // Phai forms open in Phai Studio with their own icon - .phf, and .phaiform as they were called
    // before 0.2.6.
    files.push(("phf.ico".into(), read(root.join("apps").join("studio").join("phf.ico"))));
    let form = |ext: &str| phai_pack::FileType { ext: ext.into(), prog_id: "PhaiStudio.Form".into(), description: "Phai form".into(), icon: "phf.ico".into() };
    let m = phai_pack::Manifest { name: "Phai Studio".into(), version: version.clone(), publisher: "Monrak Net".into(), exe: "phai-studio.exe".into(), desktop: true, all_users: false, files: vec![], file_types: vec![form("phf"), form("phaiform")] };
    let out_dir = root.join("dist");
    std::fs::create_dir_all(&out_dir).expect("dist folder");
    let out = out_dir.join(format!("PhaiStudio-{version}-setup.exe"));
    let packed = phai_pack::pack(&stub, &m, &files);
    std::fs::write(&out, &packed).expect("the setup written");
    println!("{} files, {} KB -> {}", files.len(), packed.len() / 1024, out.display());
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_library_stands_alone() {
        let m = "version.workspace = true\nedition.workspace = true\npng = { workspace = true, optional = true }\nwindows = { workspace = true, features = [\n";
        let s = super::standalone(m, "1.2.3");
        assert!(s.contains("version = \"1.2.3\"") && s.contains("png = { version = \"0.17\", optional = true }") && s.contains("windows = { version = \"0.62\", features"));
        assert!(!s.contains("workspace"));
    }
}
