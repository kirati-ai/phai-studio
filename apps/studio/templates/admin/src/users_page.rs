//! UsersPage: the people who can sign in, in a grid. Add user and Edit open UserForm as a
//! dialog; Delete asks first. The list lives in `self.users` and the grid is filled from it
//! again after every change - the grid only shows what the program keeps.

#![allow(unused_variables, unused_imports)] // an event's method need not use all it is given (ui, e), nor the file all the prelude brings

use crate::user_form::UserForm;
use crate::user_form_ui::UserFormUi;
use crate::users_page_ui::{UsersPageEvents, UsersPageUi};
use phai_ui::prelude::*;

/// The roles, in the order of UserForm's drop-down.
pub const ROLES: [&str; 3] = ["Administrator", "Manager", "Staff"];

/// One person who can sign in.
#[derive(Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
    /// A place in ROLES.
    pub role: usize,
    /// May they sign in now?
    pub active: bool,
    pub last_seen: String,
}

/// "8 users" under the grid - those shown, after the search box.
pub fn show_count(ui: &UsersPageUi) {
    ui.count.set_text(&tf("{0} users shown", &[&ui.people.rows_count()]));
}

#[derive(Default)]
pub struct UsersPage {
    users: Vec<User>,
    /// The number the next new user gets.
    next_id: u32,
}

impl UsersPage {
    /// Puts every user in the grid, one row each.
    fn fill(&self, ui: &UsersPageUi) {
        let rows: Vec<Vec<String>> = self
            .users
            .iter()
            .map(|u| {
                let status = if u.active { t("Active") } else { t("Suspended") };
                vec![u.id.to_string(), u.name.clone(), u.email.clone(), t(ROLES[u.role]).to_string(), status.to_string(), u.last_seen.clone()]
            })
            .collect();
        ui.people.fill(&rows);
        show_count(ui);
    }

    /// The user on the chosen row: column 0 holds the ID, which finds them in `self.users`.
    fn chosen(&self, ui: &UsersPageUi) -> Option<usize> {
        let row = ui.people.selected_row()?;
        let id: u32 = ui.people.cell(row, 0).parse().ok()?;
        self.users.iter().position(|u| u.id == id)
    }
}

impl UsersPageEvents for UsersPage {
    /// The page is made: some sample users.
    fn loaded(&mut self, ui: &UsersPageUi) {
        let sample = [
            ("Somchai Kittisak", "somchai@example.com", 0, true, "Today 08:12"),
            ("Malee Wongsa", "malee@example.com", 1, true, "Today 09:40"),
            ("Anan Srisuk", "anan@example.com", 2, true, "Yesterday 17:05"),
            ("Pim Chaiyo", "pim@example.com", 2, true, "Today 10:02"),
            ("Kittipong Rakdee", "kittipong@example.com", 1, true, "2026-11-22"),
            ("Napat Boonma", "napat@example.com", 2, false, "2026-10-30"),
            ("Suda Inthara", "suda@example.com", 2, true, "Yesterday 12:47"),
            ("Wichai Thongdee", "wichai@example.com", 2, true, "2026-11-20"),
        ];
        for (name, email, role, active, last_seen) in sample {
            self.next_id += 1;
            self.users.push(User { id: self.next_id, name: name.into(), email: email.into(), role, active, last_seen: last_seen.into() });
        }
        self.next_id = 100; // new users from 101
        // Suspended people in grey.
        let mut status = ui.people.column(4);
        status.rules = phai_ui::grid::rules(&[&format!("= \"{}\" fore=#94a3b8 row", t("Suspended"))]);
        ui.people.set_column(4, status);
        self.fill(ui);
    }

    fn add_clicked(&mut self, ui: &UsersPageUi) {
        // Open UserForm and wait; Some(form) comes back with what was typed, once it is closed.
        let (answer, form) = UserFormUi::show_dialog_for(ui.hwnd, UserForm::new(t("Add user"), None));
        if let (DialogResult::Ok, Some(form)) = (answer, form) {
            self.next_id += 1;
            let user = User { id: self.next_id, name: form.name, email: form.email, role: form.role, active: form.active, last_seen: t("Never").to_string() };
            self.users.push(user);
            self.fill(ui);
        }
    }

    /// Edit clicked, or a row double-clicked.
    fn edit_clicked(&mut self, ui: &UsersPageUi) {
        let Some(i) = self.chosen(ui) else {
            message_box(ui.hwnd, t("Choose a user in the list first."), t("Users"), MessageBoxButtons::Ok, MessageBoxIcon::Information);
            return;
        };
        let (answer, form) = UserFormUi::show_dialog_for(ui.hwnd, UserForm::new(t("Edit user"), Some(self.users[i].clone())));
        if let (DialogResult::Ok, Some(form)) = (answer, form) {
            let user = &mut self.users[i];
            user.name = form.name;
            user.email = form.email;
            user.role = form.role;
            user.active = form.active;
            self.fill(ui);
        }
    }

    fn delete_clicked(&mut self, ui: &UsersPageUi) {
        let Some(i) = self.chosen(ui) else {
            message_box(ui.hwnd, t("Choose a user in the list first."), t("Users"), MessageBoxButtons::Ok, MessageBoxIcon::Information);
            return;
        };
        let question = tf("Delete {0}? They will no longer be able to sign in.", &[&self.users[i].name]);
        if message_box(ui.hwnd, &question, t("Delete user"), MessageBoxButtons::YesNo, MessageBoxIcon::Warning) == DialogResult::Yes {
            self.users.remove(i);
            self.fill(ui);
        }
    }
}
