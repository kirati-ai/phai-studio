//! SettingsPage: a few of the shop's settings. Save changes keeps them in `self` and tells the
//! main window (notify_parent), which shows the new shop name. A real program would write them
//! to a file or a database here.

#![allow(unused_variables, unused_imports)] // an event's method need not use all it is given (ui, e), nor the file all the prelude brings

use crate::settings_page_ui::{SettingsPageEvents, SettingsPageUi};
use phai_ui::prelude::*;

#[derive(Default)]
pub struct SettingsPage {
    shop: String,
    email: String,
    currency: usize,
    low_stock: i32,
    daily_email: bool,
    sound: bool,
}

impl SettingsPageEvents for SettingsPage {
    fn loaded(&mut self, ui: &SettingsPageUi) {
        ui.currency.set_selected_index(Some(0));
    }

    fn save_clicked(&mut self, ui: &SettingsPageUi) {
        let email = ui.email.text();
        if !email.contains('@') {
            message_box(ui.hwnd, t("The email address does not look right."), t("Settings"), MessageBoxButtons::Ok, MessageBoxIcon::Warning);
            ui.email.focus();
            return;
        }
        self.shop = ui.shop.text();
        self.email = email;
        self.currency = ui.currency.selected_index().unwrap_or(0);
        self.low_stock = ui.low_stock.value();
        self.daily_email = ui.daily_email.checked();
        self.sound = ui.new_order_sound.checked();
        ui.saved.set_text(&tf("Saved at {0}.", &[&now().format("HH:mm")]));
        ui.notify_parent(); // the main window's settings_saved runs
    }
}
