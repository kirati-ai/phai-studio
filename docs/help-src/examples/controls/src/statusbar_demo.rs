//! StatusBar: a strip at the bottom of the window, in parts, telling how things stand.

use crate::statusbar_demo_ui::{StatusbarDemoEvents, StatusbarDemoUi};

#[derive(Default)]
pub struct StatusbarDemo;

impl StatusbarDemoEvents for StatusbarDemo {
    fn loaded(&mut self, ui: &StatusbarDemoUi) {
        ui.text.set_text("Type here and watch the bottom of the window.");
        self.text_changed(ui);
    }

    // help: parts
    fn text_changed(&mut self, ui: &StatusbarDemoUi) {
        // Part 0 is the wide one on the left; part 1 the one on the right (parts = [140]).
        let n = ui.text.text().chars().count();
        ui.status.set_part_text(0, "Ready");
        ui.status.set_part_text(1, &format!("{n} letters"));
    }
    // help: end
}
