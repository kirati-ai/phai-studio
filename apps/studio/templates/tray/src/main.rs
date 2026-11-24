//! Made with Phai Studio: a program that lives in its icon by the clock (a tray program).
//! `main_window.phf` is its window's design, with the icon and the icon's menu;
//! `main_window_ui.rs` what Phai Studio writes from it, and `main_window.rs` what they do.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod main_window;
mod main_window_ui;
mod text;

fn main() {
    text::load();
    main_window_ui::MainWindowUi::run(main_window::MainWindow::default());
}
