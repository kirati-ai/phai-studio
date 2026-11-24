//! OrdersPage: every order in a grid, a drop-down to show only one status, and the count and
//! total under the grid. The orders are sample data written below - a real shop would read them
//! from its database.

#![allow(unused_variables, unused_imports)] // an event's method need not use all it is given (ui, e), nor the file all the prelude brings

use crate::orders_page_ui::{OrdersPageEvents, OrdersPageUi};
use phai_ui::prelude::*;

/// One order: its number, date, customer, how many items, total in baht, and status.
pub struct Order {
    pub number: &'static str,
    pub date: &'static str,
    pub time: &'static str,
    pub customer: &'static str,
    pub items: u32,
    pub total: u32,
    pub status: &'static str,
}

/// The statuses an order can have, as the drop-down lists them after "All orders".
pub const STATUSES: [&str; 4] = ["Paid", "Pending", "Shipped", "Cancelled"];

/// Sample orders, the newest first. The Dashboard shows the first few too.
pub const ORDERS: &[Order] = &[
    Order { number: "#10248", date: "2026-11-25", time: "10:42", customer: "Malee Wongsa", items: 3, total: 2450, status: "Paid" },
    Order { number: "#10247", date: "2026-11-25", time: "10:15", customer: "Anan Srisuk", items: 1, total: 890, status: "Pending" },
    Order { number: "#10246", date: "2026-11-25", time: "09:58", customer: "Pim Chaiyo", items: 5, total: 6120, status: "Paid" },
    Order { number: "#10245", date: "2026-11-25", time: "09:31", customer: "Kittipong Rakdee", items: 2, total: 1340, status: "Shipped" },
    Order { number: "#10244", date: "2026-11-25", time: "09:07", customer: "Napat Boonma", items: 1, total: 450, status: "Cancelled" },
    Order { number: "#10243", date: "2026-11-25", time: "08:46", customer: "Suda Inthara", items: 4, total: 3980, status: "Paid" },
    Order { number: "#10242", date: "2026-11-25", time: "08:20", customer: "Wichai Thongdee", items: 2, total: 1760, status: "Shipped" },
    Order { number: "#10241", date: "2026-11-24", time: "19:12", customer: "Ploy Siriwan", items: 6, total: 8450, status: "Shipped" },
    Order { number: "#10240", date: "2026-11-24", time: "17:40", customer: "Somsak Pattana", items: 1, total: 990, status: "Paid" },
    Order { number: "#10239", date: "2026-11-24", time: "16:05", customer: "Jintana Kaewkla", items: 3, total: 2870, status: "Pending" },
    Order { number: "#10238", date: "2026-11-24", time: "14:33", customer: "Thanawat Meesuk", items: 2, total: 1520, status: "Shipped" },
    Order { number: "#10237", date: "2026-11-24", time: "11:18", customer: "Araya Chanthra", items: 7, total: 11250, status: "Shipped" },
    Order { number: "#10236", date: "2026-11-23", time: "20:47", customer: "Narin Phromma", items: 1, total: 650, status: "Cancelled" },
    Order { number: "#10235", date: "2026-11-23", time: "15:02", customer: "Kanya Rattana", items: 2, total: 2240, status: "Shipped" },
    Order { number: "#10234", date: "2026-11-23", time: "10:26", customer: "Prasert Ngamdee", items: 4, total: 4630, status: "Shipped" },
];

/// Colours a grid's Status column `col`: green Paid, amber Pending, blue Shipped, red Cancelled.
/// The rules compare the words shown, so they are made from the words in the language in use.
pub fn colour_statuses(grid: phai_ui::ctl::Grid, col: usize) {
    let colours = ["#15803d", "#b45309", "#1d4ed8", "#b91c1c"];
    let lines: Vec<String> = STATUSES.iter().zip(colours).map(|(s, c)| format!("= \"{}\" fore={c}", t(s))).collect();
    let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
    let mut column = grid.column(col);
    column.rules = phai_ui::grid::rules(&lines);
    grid.set_column(col, column);
}

/// "15 orders" under the grid - those shown, after the search box and the drop-down.
pub fn show_count(ui: &OrdersPageUi) {
    ui.count.set_text(&tf("{0} orders shown", &[&ui.list.rows_count()]));
}

#[derive(Default)]
pub struct OrdersPage {}

impl OrdersPageEvents for OrdersPage {
    /// The page is made: fill the grid and choose "All orders".
    fn loaded(&mut self, ui: &OrdersPageUi) {
        let rows: Vec<Vec<String>> = ORDERS
            .iter()
            .map(|o| vec![o.number.to_string(), format!("{} {}", o.date, o.time), o.customer.to_string(), o.items.to_string(), o.total.to_string(), t(o.status).to_string()])
            .collect();
        ui.list.fill(&rows);
        colour_statuses(ui.list, 5);
        ui.status_filter.set_selected_index(Some(0));
        show_count(ui);
    }

    /// Another status chosen: the grid keeps only the orders with it (0 is "All orders").
    fn status_filter_changed(&mut self, ui: &OrdersPageUi) {
        match ui.status_filter.selected_index() {
            Some(i) if i > 0 => ui.list.set_column_filter(5, Some(&[t(STATUSES[i - 1])])),
            _ => ui.list.set_column_filter(5, None),
        }
        show_count(ui);
    }
}
