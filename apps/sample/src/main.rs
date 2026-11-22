//! A program made from a Phai Studio form. `main_window.phf` is the design,
//! `main_window_ui.rs` what Phai Studio wrote from it, `main_window.rs` the code that is ours.

#![windows_subsystem = "windows"]

mod main_window;
mod main_window_ui;
mod text;

fn main() {
    text::load();
    main_window_ui::MainWindowUi::run(main_window::MainWindow::default());
}
