//! Toolbar: buttons with pictures under the menu - each runs the method named by its id.

use crate::toolbar_demo_ui::{ToolbarDemoEvents, ToolbarDemoUi};

#[derive(Default)]
pub struct ToolbarDemo;

impl ToolbarDemoEvents for ToolbarDemo {
    // help: items
    fn new_note(&mut self, ui: &ToolbarDemoUi) {
        ui.note.set_text("");
        ui.status.set_part_text(0, "A new note");
    }

    fn save_note(&mut self, ui: &ToolbarDemoUi) {
        let path = std::env::temp_dir().join("note.txt");
        let done = std::fs::write(&path, ui.note.text()).is_ok();
        ui.status.set_part_text(0, if done { "Saved" } else { "Could not save" });
    }
    // help: end

    // help: checked
    fn bold(&mut self, ui: &ToolbarDemoUi) {
        // A toolbar button that stays down: the program turns its tick over itself.
        let on = !ui.item_checked("bold");
        ui.set_item_checked("bold", on);
        ui.status.set_part_text(0, if on { "Bold on" } else { "Bold off" });
    }
    // help: end
}
