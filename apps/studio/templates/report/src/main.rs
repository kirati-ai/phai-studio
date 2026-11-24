//! Made with Phai Studio. `main_window.phf` is the window's design, `main_window_ui.rs`
//! what Phai Studio writes from it, and `main_window.rs` what its controls do.
//! `sales_report.rs` is the report the window prints.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod main_window;
mod main_window_ui;
mod sales_report;
mod text;

fn main() {
    text::load();
    main_window_ui::MainWindowUi::run(main_window::MainWindow);
}
