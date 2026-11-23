//! A program made from a Phai Studio form. `main_window.phf` is the design,
//! `main_window_ui.rs` what Phai Studio wrote from it, `main_window.rs` the code that is ours.
//! `--tray`: a program that lives in its icon in the corner instead (`tray_window.phf`).

#![windows_subsystem = "windows"]

mod main_window;
mod main_window_ui;
mod text;
mod tray_window;
mod tray_window_ui;

fn main() {
    text::load();
    if std::env::args().any(|a| a == "--tray") {
        tray_window_ui::TrayWindowUi::run(tray_window::TrayWindow::default());
    } else {
        main_window_ui::MainWindowUi::run(main_window::MainWindow::default());
    }
}
