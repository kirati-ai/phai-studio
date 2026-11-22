//! A small dialog that asks for a name.

use crate::name_form_ui::{NameFormEvents, NameFormUi};

// help: nameform
#[derive(Default)]
pub struct NameForm {
    /// What was typed, read when the window has closed.
    pub name: String,
}

impl NameFormEvents for NameForm {
    fn closed(&mut self, ui: &NameFormUi) {
        // Keep the text before the window is gone.
        self.name = ui.answer.text();
    }
}
// help: end
