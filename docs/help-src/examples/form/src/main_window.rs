//! The main window: what happens when it opens, its two buttons, closing and keys.

use crate::main_window_ui::{MainWindowEvents, MainWindowUi};
use crate::name_form::NameForm;
use crate::name_form_ui::NameFormUi;
use crate::note_form::NoteForm;
use crate::note_form_ui::NoteFormUi;
use phai_ui::events::{ClosingEventArgs, KeyEventArgs};
use phai_ui::form::{message_box, DialogResult, MessageBoxButtons, MessageBoxIcon};

// help: struct
#[derive(Default)]
pub struct MainWindow {
    // What the program remembers while the window is open.
    name: String,
}
// help: end

impl MainWindowEvents for MainWindow {
    // help: loaded
    fn loaded(&mut self, ui: &MainWindowUi) {
        // Runs once, just before the window shows.
        ui.set_title("Hello window");
        ui.greeting.set_text("Press a button below.");
    }
    // help: end

    // help: dialog
    fn ask_name_clicked(&mut self, ui: &MainWindowUi) {
        // Open NameForm over this window and wait until it closes.
        let (answer, form) = NameFormUi::show_dialog_for(ui.hwnd, NameForm::default());
        if let (DialogResult::Ok, Some(form)) = (answer, form) {
            self.name = form.name;
            ui.greeting.set_text(&format!("Hello, {}!", self.name));
        }
    }
    // help: end

    // help: show
    fn open_note_clicked(&mut self, ui: &MainWindowUi) {
        // Open NoteForm beside this window; this one carries on at once.
        NoteFormUi::show(ui.hwnd, NoteForm);
    }
    // help: end

    // help: closing
    fn main_window_closing(&mut self, ui: &MainWindowUi, e: &ClosingEventArgs) {
        let sure = message_box(ui.hwnd, "Close the program?", "Hello window", MessageBoxButtons::YesNo, MessageBoxIcon::Question);
        if sure != DialogResult::Yes {
            e.cancel(); // the window stays open
        }
    }
    // help: end

    // help: keys
    fn main_window_key_down(&mut self, ui: &MainWindowUi, e: &KeyEventArgs) {
        // F11 (key 0x7A): full screen and back. KeyPreview lets the window hear it first.
        if e.key == 0x7A {
            let on = !phai_ui::form::is_full_screen(ui.hwnd);
            phai_ui::form::set_full_screen(ui.hwnd, on);
        }
    }
    // help: end
}
