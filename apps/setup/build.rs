//! Common Controls 6, as every phai-ui program has - and `asInvoker`: Windows takes a program
//! named "setup" for an installer wanting an administrator and asks for one unless the manifest
//! says otherwise. Phai's setup installs for the one user, so it needs none.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTUAC:level='asInvoker' uiAccess='false'");
        println!(
            "cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
    }
    // Version information Windows shows in a file's Properties - and code signing checks: the
    // product's name and version (from Cargo.toml), who makes it.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        // Icon 1: what Explorer, the browser's downloads and the taskbar show.
        res.set_icon("../studio/phai-studio.ico");
        res.set("ProductName", "Phai Studio");
        res.set("FileDescription", "Phai Studio Setup");
        res.set("CompanyName", "Monrak Net");
        res.set("LegalCopyright", "Copyright (c) 2026 Monrak Net (Kirati Kittikhun) - MIT License");
        res.set("OriginalFilename", "phai-setup.exe");
        if let Err(e) = res.compile() {
            println!("cargo:warning=no version information in phai-setup.exe: {e}");
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../studio/phai-studio.ico");
}
