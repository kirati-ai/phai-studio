//! SignIn: who is using the program. The main window opens it first (main_window.rs) and reads
//! `user` from it when it has closed - set only when the password was right.

use crate::sign_in_ui::{SignInEvents, SignInUi};
use phai_ui::prelude::*;

/// Who may sign in, with their passwords - admin and 1234 to try it. A real program keeps its
/// users in its database, each password hashed (never as plain text), and checks them there.
const USERS: &[(&str, &str)] = &[("admin", "1234")];

#[derive(Default)]
pub struct SignIn {
    /// The user name the form starts with: the last one signed in, after Sign out.
    pub user_name: String,
    /// Who signed in; `None` until the password is right.
    pub user: Option<String>,
}

impl SignIn {
    /// The form with `user_name` typed in already.
    pub fn new(user_name: &str) -> SignIn {
        SignIn { user_name: user_name.to_string(), user: None }
    }
}

impl SignInEvents for SignIn {
    fn loaded(&mut self, ui: &SignInUi) {
        ui.user_name.set_text(&self.user_name);
    }

    fn shown(&mut self, ui: &SignInUi) {
        // The keyboard on the first box still to fill.
        if self.user_name.is_empty() {
            ui.user_name.focus();
        } else {
            ui.password.focus();
        }
    }

    fn sign_in_clicked(&mut self, ui: &SignInUi) {
        let name = ui.user_name.text().trim().to_string();
        let password = ui.password.text();
        if USERS.iter().any(|(u, p)| u.eq_ignore_ascii_case(&name) && *p == password) {
            // Right: the form closes, answering Ok; the main window reads `user`.
            self.user = Some(name);
            phai_ui::form::close_with(ui.hwnd, DialogResult::Ok);
        } else {
            // Wrong: say so, and let them type the password again.
            ui.message.set_text(t("The user name or password is wrong."));
            ui.password.set_text("");
            ui.password.focus();
        }
    }
}
