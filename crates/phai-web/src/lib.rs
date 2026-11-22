//! Phai forms in a web browser. The same `.phf` and the same program code as on Windows,
//! compiled to WebAssembly: Phai Studio writes `src/web/<form>_ui.rs` from the form, a project
//! names this crate `phai-ui` when it builds for `wasm32` (Cargo's `package = "phai-web"`), and
//! the program's `phai_ui::ctl::TextBox`, `phai_ui::text::tf`... are these, working on the page.
//! The page installs as an app (a PWA) on Android and iOS.
//!
//! What a browser cannot do - NotifyIcon, printing, files on the disk by
//! their paths - is not here; a program that uses it does not build for the web, and Phai Studio
//! says which control or call is the reason.

pub mod ctl;
pub mod data;
pub mod events;
pub mod form;
pub mod menu;
pub mod paint;
#[path = "../../phai-ui/src/text.rs"]
pub mod text;
pub mod tree;
pub mod win;
