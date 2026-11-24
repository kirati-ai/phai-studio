//! UserForm: the dialog Add user and Edit open. It starts with a user's details (or empty ones)
//! and, when Save closes it, keeps what was typed for the Users page to read.

#![allow(unused_variables, unused_imports)] // an event's method need not use all it is given (ui, e), nor the file all the prelude brings

use crate::user_form_ui::{UserFormEvents, UserFormUi};
use crate::users_page::User;
use phai_ui::prelude::*;

#[derive(Default)]
pub struct UserForm {
    /// The dialog's title: "Add user" or "Edit user".
    title: &'static str,
    /// The user being edited; None when adding one.
    start: Option<User>,
    // What was typed, read when the window has closed.
    pub name: String,
    pub email: String,
    pub role: usize,
    pub active: bool,
}

impl UserForm {
    pub fn new(title: &'static str, start: Option<User>) -> UserForm {
        UserForm { title, start, ..Default::default() }
    }
}

impl UserFormEvents for UserForm {
    /// The dialog is about to show: put the user's details in the boxes.
    fn loaded(&mut self, ui: &UserFormUi) {
        ui.set_title(self.title);
        match &self.start {
            Some(u) => {
                ui.name.set_text(&u.name);
                ui.email.set_text(&u.email);
                ui.role.set_selected_index(Some(u.role));
                ui.active.set_checked(u.active);
            }
            None => ui.role.set_selected_index(Some(2)), // a new user is Staff to begin with
        }
    }

    /// The dialog has closed (Save or Cancel): keep what the boxes say before they are gone.
    fn closed(&mut self, ui: &UserFormUi) {
        self.name = ui.name.text().trim().to_string();
        self.email = ui.email.text().trim().to_string();
        self.role = ui.role.selected_index().unwrap_or(2);
        self.active = ui.active.checked();
    }
}
