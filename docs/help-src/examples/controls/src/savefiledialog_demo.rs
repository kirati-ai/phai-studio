//! SaveFileDialog: Windows' own window for choosing where to save, and the name.

use crate::savefiledialog_demo_ui::{SavefiledialogDemoEvents, SavefiledialogDemoUi};

#[derive(Default)]
pub struct SavefiledialogDemo;

impl SavefiledialogDemoEvents for SavefiledialogDemo {
    fn loaded(&mut self, ui: &SavefiledialogDemoUi) {
        ui.contents.set_text("Shopping list:\r\n- rice\r\n- eggs");
    }

    // help: save
    fn save_clicked(&mut self, ui: &SavefiledialogDemoUi) {
        // A name in its box to begin with; None for Cancel.
        let Some(path) = ui.save_dialog.show_dialog_named("note.txt") else { return };
        match std::fs::write(&path, ui.contents.text()) {
            Ok(()) => ui.saved.set_text("Saved"),
            Err(e) => ui.saved.set_text(&format!("Not saved: {e}")),
        }
    }
    // help: end
}
