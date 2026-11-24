//! Made with Phai Studio: a program whose documents open inside its main window (MDI).
//! `main_window.phf` is the main window's design and `main_window.rs` what its menus do;
//! `document_window.phf` and `document_window.rs` are one document. The `_ui.rs` files are what
//! Phai Studio writes from the designs.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod document_window;
mod document_window_ui;
mod main_window;
mod main_window_ui;
mod text;

fn main() {
    text::load();
    main_window_ui::MainWindowUi::run(main_window::MainWindow::default());
}
