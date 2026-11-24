//! Made with Phai Studio. Two forms: `sign_in.phf` asks who is using the program, then
//! `main_window.phf` is the program itself. Each form's `_ui.rs` is what Phai Studio writes from
//! it, and its `.rs` what its controls do.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod main_window;
mod main_window_ui;
mod sign_in;
mod sign_in_ui;
mod text;

fn main() {
    text::load();
    // The main window runs the program; it opens the sign-in form first (main_window.rs).
    main_window_ui::MainWindowUi::run(main_window::MainWindow::default());
}
