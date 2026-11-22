//! Button: count the clicks, and a second button that is usable only when there is something to undo.

use crate::button_demo_ui::{ButtonDemoEvents, ButtonDemoUi};

#[derive(Default)]
pub struct ButtonDemo {
    clicks: u32,
}

impl ButtonDemoEvents for ButtonDemo {
    // help: loaded
    fn loaded(&mut self, ui: &ButtonDemoUi) {
        // Nothing to start again yet: greyed out.
        ui.reset.set_enabled(false);
    }
    // help: end

    // help: click
    fn click_me_clicked(&mut self, ui: &ButtonDemoUi) {
        self.clicks += 1;
        ui.count_label.set_text(&format!("Clicked {} times", self.clicks));
        ui.reset.set_enabled(true);
    }
    // help: end

    // help: reset
    fn reset_clicked(&mut self, ui: &ButtonDemoUi) {
        self.clicks = 0;
        ui.count_label.set_text("Not clicked yet");
        ui.reset.set_enabled(false);
        ui.click_me.focus(); // back to the first button, so Enter clicks it again
    }
    // help: end
}
