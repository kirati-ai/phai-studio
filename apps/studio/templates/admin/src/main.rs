//! Made with Phai Studio: an admin dashboard. `main_window.phf` is the window - the menu on the
//! left and the header - and each page is a user control of its own: `dashboard_page.phf`,
//! `users_page.phf`, `orders_page.phf` and `settings_page.phf`. `user_form.phf` is the dialog
//! that adds or edits a user. Each `.phf` has a `_ui.rs` Phai Studio writes from it and a `.rs`
//! that is yours: what its controls do. The colours are in theme.css.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod dashboard_page;
mod dashboard_page_ui;
mod main_window;
mod main_window_ui;
mod orders_page;
mod orders_page_ui;
mod settings_page;
mod settings_page_ui;
mod text;
mod user_form;
mod user_form_ui;
mod users_page;
mod users_page_ui;

fn main() {
    text::load();
    main_window_ui::MainWindowUi::run(main_window::MainWindow::default());
}
