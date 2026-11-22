//! Menu bar: menus at the top of the window - each item runs the method named by its id.

use crate::menubar_demo_ui::{MenubarDemoEvents, MenubarDemoUi};
use phai_ui::form::{message_box, MessageBoxButtons, MessageBoxIcon};

#[derive(Default)]
pub struct MenubarDemo;

impl MenubarDemoEvents for MenubarDemo {
    // help: items
    fn new_note(&mut self, ui: &MenubarDemoUi) {
        ui.note.set_text("");
    }

    fn exit(&mut self, ui: &MenubarDemoUi) {
        ui.close();
    }

    fn about(&mut self, ui: &MenubarDemoUi) {
        message_box(ui.hwnd, "A menu bar made with Phai Studio.", "About", MessageBoxButtons::Ok, MessageBoxIcon::Information);
    }
    // help: end

    // help: checked
    fn big_text(&mut self, ui: &MenubarDemoUi) {
        // CheckOnClick: the tick turns over by itself; ask it which way it is now.
        let big = ui.item_checked("big_text");
        ui.note.set_text(if big { "Big text is on" } else { "Big text is off" });
    }
    // help: end
}
