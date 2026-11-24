//! Embeds a manifest that names Common Controls 6 (PhaiDesk's `build.rs`). Without it Windows
//! hands the program the old controls: buttons as in Windows 2000, and image lists that ignore
//! alpha, so toolbar icons sit in black squares. Every program phai-ui makes needs these lines.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
    }
    // Version information Windows shows in a file's Properties - and code signing checks: the
    // product's name and version (from Cargo.toml), who makes it.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        // Icon 1: what Explorer, the taskbar and shortcuts show; the window loads it too.
        res.set_icon("phai-studio.ico");
        res.set("ProductName", "Phai Studio");
        res.set("FileDescription", "Phai Studio");
        res.set("CompanyName", "Monrak Net");
        res.set("LegalCopyright", "Copyright (c) 2026 Monrak Net (Kirati Kittikhun) - PolyForm Perimeter 1.0.1");
        res.set("OriginalFilename", "phai-studio.exe");
        if let Err(e) = res.compile() {
            println!("cargo:warning=no version information in phai-studio.exe: {e}");
        }
    }
    embed_templates();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=phai-studio.ico");
}

/// New Project's templates, each a folder of `templates/` - its forms, its code, its words in Thai
/// and a picture of it running - built into Phai Studio: `templates.rs` in OUT_DIR lists every
/// file as (template, path inside it, bytes). A form's `_ui.rs` is not taken: Phai Studio writes
/// it from the form when it makes the project, so the two never disagree.
fn embed_templates() {
    let root = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default()).join("templates");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut out = String::from("pub static TEMPLATE_FILES: &[(&str, &str, &[u8])] = &[\n");
    let mut keys: Vec<_> = std::fs::read_dir(&root).into_iter().flatten().flatten().filter(|e| e.path().is_dir()).collect();
    keys.sort_by_key(|e| e.file_name());
    for k in keys {
        let key = k.file_name().to_string_lossy().to_string();
        let mut files = Vec::new();
        walk(&k.path(), &mut files);
        files.sort();
        for f in files {
            let rel = f.strip_prefix(k.path()).unwrap_or(&f).to_string_lossy().replace('\\', "/");
            if rel.ends_with("_ui.rs") || rel.starts_with("target/") || rel == "Cargo.toml" || rel == "Cargo.lock" {
                continue;
            }
            out.push_str(&format!("    ({key:?}, {rel:?}, include_bytes!({:?})),\n", f.display().to_string()));
        }
    }
    out.push_str("];\n");
    let dest = std::path::Path::new(&std::env::var("OUT_DIR").unwrap_or_default()).join("templates.rs");
    let _ = std::fs::write(dest, out);
}

fn walk(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, files);
        } else {
            files.push(p);
        }
    }
}
