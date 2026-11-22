//! RadioButton: one choice of three - one method for all of them.

use crate::radio_demo_ui::{RadioDemoEvents, RadioDemoUi};

#[derive(Default)]
pub struct RadioDemo;

impl RadioDemoEvents for RadioDemo {
    fn loaded(&mut self, ui: &RadioDemoUi) {
        self.size_clicked(ui);
    }

    // help: which
    fn size_clicked(&mut self, ui: &RadioDemoUi) {
        // The three share this method; ask which is chosen now.
        let price = if ui.large.checked() {
            45
        } else if ui.medium.checked() {
            35
        } else {
            25
        };
        ui.price.set_text(&format!("{price} baht"));
    }
    // help: end
}
