//! ErrorProvider: a red mark beside a box filled in wrongly, its reason under the mouse.

use crate::errorprovider_demo_ui::{ErrorproviderDemoEvents, ErrorproviderDemoUi};

#[derive(Default)]
pub struct ErrorproviderDemo;

impl ErrorproviderDemoEvents for ErrorproviderDemo {
    fn loaded(&mut self, ui: &ErrorproviderDemoUi) {
        ui.email.set_text("somchai.example.com");
        self.send_clicked(ui);
    }

    // help: check
    fn send_clicked(&mut self, ui: &ErrorproviderDemoUi) {
        let email = ui.email.text();
        if email.contains('@') {
            ui.errors.set_error(ui.email, ""); // empty: the mark goes
        } else {
            ui.errors.set_error(ui.email, "An e-mail address has an @ in it");
        }
    }
    // help: end
}
