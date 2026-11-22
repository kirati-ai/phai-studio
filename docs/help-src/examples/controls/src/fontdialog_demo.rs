//! FontDialog: Windows' own window for choosing a font.

use crate::fontdialog_demo_ui::{FontdialogDemoEvents, FontdialogDemoUi};

#[derive(Default)]
pub struct FontdialogDemo;

impl FontdialogDemoEvents for FontdialogDemo {
    // help: pick
    fn pick_clicked(&mut self, ui: &FontdialogDemoUi) {
        let Some(f) = ui.fonts.show_dialog() else { return };
        let style = match (f.bold, f.italic) {
            (true, true) => ", bold italic",
            (true, false) => ", bold",
            (false, true) => ", italic",
            _ => "",
        };
        ui.chosen.set_text(&format!("{}, {} pt{style}", f.face, f.points));
    }
    // help: end
}
