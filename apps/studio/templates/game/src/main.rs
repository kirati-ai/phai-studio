//! Made with Phai Studio: Catch the Ball, a small game with a timer. `main_window.phf` is the
//! window's design, `main_window_ui.rs` what Phai Studio writes from it, and `main_window.rs`
//! the game itself: its state, its events and its two timers.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod main_window;
mod main_window_ui;
mod text;

fn main() {
    text::load();
    main_window_ui::MainWindowUi::run(main_window::MainWindow::default());
}
