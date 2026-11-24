//! DashboardPage: the shop at a glance. Four cards with one big number each, the latest orders,
//! and this month's goals as progress bars. The numbers are made up here (or worked out from the
//! sample orders); a real shop would ask its database for them.

#![allow(unused_variables, unused_imports)] // an event's method need not use all it is given (ui, e), nor the file all the prelude brings

use crate::dashboard_page_ui::{DashboardPageEvents, DashboardPageUi};
use crate::orders_page::{colour_statuses, ORDERS};
use phai_ui::prelude::*;

/// A whole number with commas between thousands: 48250 -> "48,250".
fn with_commas(n: u32) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// Sets a goal's bar and its "72%" label.
fn show_goal(bar: phai_ui::ctl::ProgressBar, label: phai_ui::ctl::Label, percent: i32) {
    bar.set_value(percent);
    label.set_text(&format!("{percent}%"));
}

#[derive(Default)]
pub struct DashboardPage {}

impl DashboardPageEvents for DashboardPage {
    /// The page is made: fill in the cards, the table and the goals.
    fn loaded(&mut self, ui: &DashboardPageUi) {
        // The cards. Today's sales and orders are worked out from the sample orders.
        let today: Vec<_> = ORDERS.iter().filter(|o| o.date == "2026-11-25" && o.status != "Cancelled").collect();
        let sales: u32 = today.iter().map(|o| o.total).sum();
        ui.sales_value.set_text(&format!("฿{}", with_commas(sales)));
        ui.sales_note.set_text(&tf("▲ {0}% on yesterday", &[&12]));

        ui.orders_value.set_text(&today.len().to_string());
        let waiting = today.iter().filter(|o| o.status == "Pending").count();
        ui.orders_note.set_text(&tf("{0} waiting for payment", &[&waiting]));

        ui.customers_value.set_text(&with_commas(1284));
        ui.customers_note.set_text(&tf("▲ {0} new this week", &[&37]));

        ui.stock_value.set_text("6");
        ui.stock_note.set_text(t("Order more soon"));

        // The latest orders: the first ten of the list, newest first.
        for o in ORDERS.iter().take(10) {
            ui.recent.rows_add(&[o.number, o.customer, &o.total.to_string(), t(o.status), o.time]);
        }
        colour_statuses(ui.recent, 3);

        // This month's goals, in percent (a progress bar runs from 0 to 100).
        show_goal(ui.goal1, ui.goal1_value, 72);
        show_goal(ui.goal2, ui.goal2_value, 58);
        show_goal(ui.goal3, ui.goal3_value, 91);

        ui.attention1.set_text(&tf("• {0} products low in stock", &[&6]));
        ui.attention2.set_text(&tf("• {0} orders waiting over 2 days", &[&3]));
        ui.attention3.set_text(&tf("• {0} refund to approve", &[&1]));
    }
}
