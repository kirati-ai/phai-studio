//! Made with Phai Studio. `main_window.phf` is the window's design, `main_window_ui.rs`
//! what Phai Studio writes from it, and `main_window.rs` what its controls do.
//! `data_sources.rs` says where the customers are kept: a SQLite file beside the program.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod data_sources;
mod main_window;
mod main_window_ui;
mod text;

fn main() {
    text::load();
    // The file and its table made on the first run, before the window reads the rows.
    if let Err(e) = data_sources::prepare() {
        let nobody = windows::Win32::Foundation::HWND::default();
        phai_ui::form::message_box(nobody, &e, phai_ui::text::t("Customers"), phai_ui::form::MessageBoxButtons::Ok, phai_ui::form::MessageBoxIcon::Error);
    }
    main_window_ui::MainWindowUi::run(main_window::MainWindow::default());
}
