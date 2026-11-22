//! OpenFileDialog: Windows' own window for choosing a file to open.

use crate::openfiledialog_demo_ui::{OpenfiledialogDemoEvents, OpenfiledialogDemoUi};

#[derive(Default)]
pub struct OpenfiledialogDemo;

impl OpenfiledialogDemoEvents for OpenfiledialogDemo {
    // help: open
    fn open_clicked(&mut self, ui: &OpenfiledialogDemoUi) {
        // None: the person pressed Cancel.
        let Some(path) = ui.open_dialog.show_dialog() else { return };
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                ui.contents.set_text(&text.replace('\n', "\r\n"));
                ui.set_title(&path.display().to_string());
            }
            Err(e) => ui.contents.set_text(&format!("Could not read it: {e}")),
        }
    }
    // help: end
}
