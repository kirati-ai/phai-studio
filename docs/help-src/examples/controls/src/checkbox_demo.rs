//! CheckBox: a tick that lets the button be pressed, and reading the ticks when it is.

use crate::checkbox_demo_ui::{CheckboxDemoEvents, CheckboxDemoUi};

#[derive(Default)]
pub struct CheckboxDemo;

impl CheckboxDemoEvents for CheckboxDemo {
    fn loaded(&mut self, ui: &CheckboxDemoUi) {
        ui.go.set_enabled(false);
    }

    // help: click
    fn agree_clicked(&mut self, ui: &CheckboxDemoUi) {
        // Ticked or not, the button follows it.
        ui.go.set_enabled(ui.agree.checked());
    }
    // help: end

    // help: read
    fn go_clicked(&mut self, ui: &CheckboxDemoUi) {
        let news = if ui.news.checked() { "with news" } else { "without news" };
        ui.answer.set_text(&format!("Signed up, {news}."));
    }
    // help: end
}
