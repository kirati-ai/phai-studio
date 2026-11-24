//! MainWindow: the program itself, for whoever signed in. It starts hidden (`start_hidden` in
//! main_window.phf) and opens the sign-in form first: the right password shows this window, and
//! File > Sign out hides it and asks again.
//!
//! The two forms pass data both ways: `SignIn::new(name)` gives the sign-in form the last user
//! name; `show_dialog_for` gives the form back when it closes, to read who signed in.

use crate::main_window_ui::{MainWindowEvents, MainWindowUi};
use crate::sign_in::SignIn;
use crate::sign_in_ui::SignInUi;
use phai_ui::prelude::*;

#[derive(Default)]
pub struct MainWindow {
    /// Who signed in; empty until someone has.
    user: String,
}

impl MainWindowEvents for MainWindow {
    fn loaded(&mut self, ui: &MainWindowUi) {
        // The window is made but not shown yet. begin_invoke runs the sign-in as soon as it is
        // ready, so the sign-in form is the first thing on the screen.
        MainWindowUi::begin_invoke(ui.invoker(), |me: &mut MainWindow, ui: &MainWindowUi| me.sign_in(ui));
    }

    fn sign_out(&mut self, ui: &MainWindowUi) {
        ui.hide();
        self.sign_in(ui);
    }

    fn exit(&mut self, ui: &MainWindowUi) {
        ui.close();
    }
}

impl MainWindow {
    /// The sign-in form, waited for: the right password shows this window for that user; Cancel
    /// (or its close box) ends the program.
    fn sign_in(&mut self, ui: &MainWindowUi) {
        let (answer, form) = SignInUi::show_dialog_for(ui.hwnd, SignIn::new(&self.user));
        match (answer, form.and_then(|f| f.user)) {
            (DialogResult::Ok, Some(user)) => {
                self.user = user;
                ui.welcome.set_text(&tf("Welcome, {0}", &[&self.user]));
                ui.signed_in_at.set_text(&tf("Signed in at {0}", &[&now().format("HH:mm, dd/MM/yyyy")]));
                ui.status.set_part_text(0, &tf("Signed in as {0}", &[&self.user]));
                ui.show_window();
            }
            _ => ui.close(),
        }
    }
}
