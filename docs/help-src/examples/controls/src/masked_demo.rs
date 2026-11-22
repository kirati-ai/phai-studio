//! MaskedTextBox: a phone number typed into its shape, saved only when it is complete.

use crate::masked_demo_ui::{MaskedDemoEvents, MaskedDemoUi};

#[derive(Default)]
pub struct MaskedDemo;

impl MaskedDemoEvents for MaskedDemo {
    fn loaded(&mut self, ui: &MaskedDemoUi) {
        ui.save.set_enabled(false);
    }

    // help: changed
    fn phone_changed(&mut self, ui: &MaskedDemoUi) {
        // Every place of the mask filled?
        ui.save.set_enabled(ui.phone.mask_completed());
    }
    // help: end

    // help: value
    fn save_clicked(&mut self, ui: &MaskedDemoUi) {
        // text() has the dashes; value() only what was typed.
        ui.result.set_text(&format!("Saved {} (digits: {})", ui.phone.text(), ui.phone.value()));
    }
    // help: end
}
