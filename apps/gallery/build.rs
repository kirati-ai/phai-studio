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
    println!("cargo:rerun-if-changed=build.rs");
}
