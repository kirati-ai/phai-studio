//! TextBox: a name and a password typed in, checked and read.

use crate::edit_demo_ui::{EditDemoEvents, EditDemoUi};

#[derive(Default)]
pub struct EditDemo;

impl EditDemoEvents for EditDemo {
    // help: read
    fn sign_in_clicked(&mut self, ui: &EditDemoUi) {
        let name = ui.name.text();
        let name = name.trim(); // spaces at either end left out
        if name.is_empty() {
            ui.message.set_text("Please type your name.");
            ui.name.focus();
            return;
        }
        if ui.password.text().len() < 4 {
            ui.message.set_text("The password is too short.");
            return;
        }
        ui.message.set_text(&format!("Welcome, {name}!"));
    }
    // help: end

    // help: changed
    fn name_changed(&mut self, ui: &EditDemoUi) {
        // Runs with every letter typed.
        let n = ui.name.text().chars().count();
        ui.message.set_text(&format!("{n} of 30 letters"));
    }
    // help: end
}
