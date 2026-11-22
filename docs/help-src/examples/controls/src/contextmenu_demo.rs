//! ContextMenu: the menu a right-click on the box opens - each item a method.

use crate::contextmenu_demo_ui::{ContextmenuDemoEvents, ContextmenuDemoUi};

#[derive(Default)]
pub struct ContextmenuDemo;

impl ContextmenuDemoEvents for ContextmenuDemo {
    fn loaded(&mut self, ui: &ContextmenuDemoUi) {
        ui.note.set_text("Hello Bamboo");
    }

    // help: items
    fn upper_clicked(&mut self, ui: &ContextmenuDemoUi) {
        ui.note.set_text(&ui.note.text().to_uppercase());
    }

    fn lower_clicked(&mut self, ui: &ContextmenuDemoUi) {
        ui.note.set_text(&ui.note.text().to_lowercase());
    }

    fn clear_clicked(&mut self, ui: &ContextmenuDemoUi) {
        ui.note.clear();
    }
    // help: end
}
