// help: main
#![windows_subsystem = "windows"]

mod main_window;
mod main_window_ui;
mod name_form;
mod name_form_ui;
mod note_form;
mod note_form_ui;

fn main() {
    // Open the main window; the program ends when it closes.
    main_window_ui::MainWindowUi::run(main_window::MainWindow::default());
}
// help: end
