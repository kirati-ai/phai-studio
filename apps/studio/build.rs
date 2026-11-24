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
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=phai-studio.ico");
}
