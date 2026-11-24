//! MainWindow: the admin window. The dark menu on the left picks a page; the header shows the
//! page's name, a search box and who is signed in. Each page is a user control of its own
//! (DashboardPage, UsersPage, OrdersPage, SettingsPage): this window only shows one and hides
//! the others. The look - the dark menu, the green "current" button - is in theme.css.

#![allow(unused_variables, unused_imports)] // an event's method need not use all it is given (ui, e), nor the file all the prelude brings

use crate::main_window_ui::{MainWindowEvents, MainWindowUi};
use crate::orders_page_ui::OrdersPageUi;
use crate::settings_page_ui::SettingsPageUi;
use crate::users_page_ui::UsersPageUi;
use phai_ui::prelude::*;

/// The pages, in the order of the menu.
const DASHBOARD: usize = 0;
const USERS: usize = 1;
const ORDERS: usize = 2;
const SETTINGS: usize = 3;

/// Each page's name, shown in the header.
const TITLES: [&str; 4] = ["Dashboard", "Users", "Orders", "Settings"];

#[derive(Default)]
pub struct MainWindow {
    /// The page showing now: DASHBOARD, USERS, ORDERS or SETTINGS.
    page: usize,
}

impl MainWindow {
    /// Shows page `page` and hides the others; its menu button is marked as the current one.
    fn show_page(&mut self, ui: &MainWindowUi, page: usize) {
        self.page = page;

        // One user control per page: only the chosen one is visible.
        let pages = [ui.dashboard, ui.users, ui.orders, ui.settings];
        for (i, p) in pages.iter().enumerate() {
            p.set_visible(i == page);
        }

        // The menu buttons: the chosen one gets the CSS class "current" as well as "nav", which
        // theme.css draws green (button.nav.current). The id is given again with the classes.
        let buttons = [
            (ui.nav_dashboard, "nav_dashboard"),
            (ui.nav_users, "nav_users"),
            (ui.nav_orders, "nav_orders"),
            (ui.nav_settings, "nav_settings"),
        ];
        for (i, (button, id)) in buttons.iter().enumerate() {
            let classes = if i == page { "nav current" } else { "nav" };
            phai_ui::win::set_css(button.hwnd(), classes, id);
        }

        ui.page_title.set_text(t(TITLES[page]));
    }
}

impl MainWindowEvents for MainWindow {
    /// The window is made and about to show: start on the Dashboard.
    fn loaded(&mut self, ui: &MainWindowUi) {
        self.show_page(ui, DASHBOARD);
    }

    fn nav_dashboard_clicked(&mut self, ui: &MainWindowUi) {
        self.show_page(ui, DASHBOARD);
    }

    fn nav_users_clicked(&mut self, ui: &MainWindowUi) {
        self.show_page(ui, USERS);
    }

    fn nav_orders_clicked(&mut self, ui: &MainWindowUi) {
        self.show_page(ui, ORDERS);
    }

    fn nav_settings_clicked(&mut self, ui: &MainWindowUi) {
        self.show_page(ui, SETTINGS);
    }

    /// A letter typed in the search box: both lists show only the rows with those words. Typed on
    /// a page without a list, it opens Orders to show what was found.
    fn search_changed(&mut self, ui: &MainWindowUi) {
        let words = ui.search.text();
        // UsersPageUi::with gives the controls of the Users page that is on this window.
        UsersPageUi::with(ui.users, |p| {
            p.people.find(&words);
            crate::users_page::show_count(p);
        });
        OrdersPageUi::with(ui.orders, |p| {
            p.list.find(&words);
            crate::orders_page::show_count(p);
        });
        if !words.is_empty() && (self.page == DASHBOARD || self.page == SETTINGS) {
            self.show_page(ui, ORDERS);
        }
    }

    /// Settings saved (the Settings page called notify_parent): the menu shows the new shop name.
    fn settings_saved(&mut self, ui: &MainWindowUi) {
        if let Some(name) = SettingsPageUi::with(ui.settings, |p| p.shop.text()) {
            if !name.trim().is_empty() {
                ui.brand.set_text(name.trim());
                ui.set_title(name.trim());
            }
        }
    }

    fn avatar_clicked(&mut self, ui: &MainWindowUi) {
        let who = tf("Signed in as {0} ({1}).", &[&ui.user_name.text(), &ui.user_role.text()]);
        message_box(ui.hwnd, &who, t("Account"), MessageBoxButtons::Ok, MessageBoxIcon::Information);
    }

    fn sign_out_clicked(&mut self, ui: &MainWindowUi) {
        let answer = message_box(ui.hwnd, t("Sign out and close the back office?"), t("Sign out"), MessageBoxButtons::YesNo, MessageBoxIcon::Question);
        if answer == DialogResult::Yes {
            ui.close();
        }
    }
}
