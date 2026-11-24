//! MainWindow: the customers, a box to search them, and the chosen one's details to change.
//!
//! The rows are held by the form's binding source `customers`, bound to the table of the same
//! name (`bind` in main_window.phf); the window reads them when it opens. The boxes on the right
//! are bound to its columns (`data_binding`), so they always show its current row, and what is
//! typed in them goes back into that row. The grid on the left lists the rows; choosing one there
//! makes it the current row. `ui.save_customers()` writes every change - rows changed, added and
//! deleted - to the file together, or none of them when one fails.

use crate::main_window_ui::{MainWindowEvents, MainWindowUi};
use phai_ui::prelude::*;

/// The binding source's columns by place, in the order `bind.columns` gives them.
const ID: usize = 0;
const NAME: usize = 1;
/// The grid shows the columns up to this one; the note only in its box.
const EMAIL: usize = 3;

#[derive(Default)]
pub struct MainWindow {
    /// The rows as the file holds them, last read - to know whether anything waits to be saved.
    saved: Vec<Vec<String>>,
}

impl MainWindowEvents for MainWindow {
    fn loaded(&mut self, ui: &MainWindowUi) {
        // The window has read the rows already (the binding source's `bind`).
        self.saved = ui.customers.rows();
        self.show_list(ui);
    }

    fn search_changed(&mut self, ui: &MainWindowUi) {
        self.show_list(ui);
    }

    fn row_chosen(&mut self, ui: &MainWindowUi) {
        // The row chosen in the grid becomes the binding source's current one: the boxes show it.
        // It is found by its id, as the grid may be searched or sorted.
        let Some(r) = ui.list.selected_row() else { return };
        let id = ui.list.cell(r, ID);
        if let Some(i) = ui.customers.rows().iter().position(|row| row[ID] == id) {
            ui.customers.set_position(i);
        }
    }

    fn customer_moved(&mut self, ui: &MainWindowUi) {
        self.show_status(ui, "");
    }

    fn new_clicked(&mut self, ui: &MainWindowUi) {
        // One new row at a time: its id is empty until it is saved, when SQLite numbers it.
        match ui.customers.rows().iter().position(|row| row[ID].is_empty()) {
            Some(i) => ui.customers.set_position(i),
            None => {
                ui.customers.add_new();
            }
        }
        self.show_list(ui);
        ui.name.focus();
    }

    fn save_clicked(&mut self, ui: &MainWindowUi) {
        self.save(ui);
    }

    fn delete_clicked(&mut self, ui: &MainWindowUi) {
        if ui.customers.count() == 0 {
            return;
        }
        let name = ui.customers.value("name");
        if message_box(ui.hwnd, &tf("Delete {0}?", &[&name]), t("Customers"), MessageBoxButtons::YesNo, MessageBoxIcon::Question) != DialogResult::Yes {
            return;
        }
        // Taken out of the binding source, then saved at once - with any other change waiting.
        ui.customers.remove_current();
        self.save(ui);
    }

    fn main_window_closing(&mut self, ui: &MainWindowUi, e: &ClosingEventArgs) {
        if ui.customers.rows() == self.saved {
            return;
        }
        match message_box(ui.hwnd, t("Save the changes before closing?"), t("Customers"), MessageBoxButtons::YesNoCancel, MessageBoxIcon::Question) {
            DialogResult::Yes if self.save(ui) => {}
            DialogResult::No => {}
            _ => e.cancel(), // the window stays open
        }
    }
}

impl MainWindow {
    /// The grid filled with the rows the search box finds (all of them when it is empty), the
    /// current row chosen in it.
    fn show_list(&self, ui: &MainWindowUi) {
        let words = ui.search.text().trim().to_lowercase();
        let found = |row: &Vec<String>| row[ID].is_empty() || row[NAME..=EMAIL].iter().any(|v| v.to_lowercase().contains(&words));
        let rows: Vec<Vec<String>> = ui.customers.rows().into_iter().filter(found).map(|row| row[..=EMAIL].to_vec()).collect();
        ui.list.fill(&rows);
        if ui.customers.count() > 0 {
            let id = ui.customers.value("id");
            if let Some(r) = (0..ui.list.rows_count()).find(|&r| ui.list.cell(r, ID) == id) {
                choose_row(ui.list, r);
            }
        }
        self.show_status(ui, "");
    }

    /// The status bar: `news` (or whether changes wait to be saved) and how many customers.
    fn show_status(&self, ui: &MainWindowUi, news: &str) {
        let left = if !news.is_empty() {
            news
        } else if ui.customers.rows() != self.saved {
            t("Not saved yet")
        } else {
            t("Ready")
        };
        ui.status.set_part_text(0, left);
        ui.status.set_part_text(1, &tf("{0} customers", &[&ui.customers.count()]));
    }

    /// Every change written to the file and the rows read again, the same customer still the
    /// current one. `false` when nothing could be saved - the changes are still there to fix.
    fn save(&mut self, ui: &MainWindowUi) -> bool {
        // The table needs a name for every customer.
        if let Some(i) = ui.customers.rows().iter().position(|row| row[NAME].trim().is_empty()) {
            ui.customers.set_position(i);
            self.show_list(ui);
            message_box(ui.hwnd, t("Type the customer's name."), t("Customers"), MessageBoxButtons::Ok, MessageBoxIcon::Warning);
            ui.name.focus();
            return false;
        }
        let id = ui.customers.value("id");
        if let Err(e) = ui.save_customers() {
            message_box(ui.hwnd, &e, t("Customers"), MessageBoxButtons::Ok, MessageBoxIcon::Error);
            return false;
        }
        self.saved = ui.customers.rows();
        // A new customer has its number now: the highest, the last row.
        match self.saved.iter().position(|row| row[ID] == id).filter(|_| !id.is_empty()) {
            Some(i) => ui.customers.set_position(i),
            None => ui.customers.move_last(),
        }
        self.show_list(ui);
        self.show_status(ui, t("Saved"));
        true
    }
}

/// Grid row `row` chosen and scrolled to, as a click on it would.
fn choose_row(grid: phai_ui::ctl::Grid, row: usize) {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::Controls::{LIST_VIEW_ITEM_STATE_FLAGS, LVIS_FOCUSED, LVIS_SELECTED, LVITEMW, LVM_ENSUREVISIBLE, LVM_SETITEMSTATE};
    use windows::Win32::UI::WindowsAndMessaging::SendMessageW;
    let on = LIST_VIEW_ITEM_STATE_FLAGS(LVIS_SELECTED.0 | LVIS_FOCUSED.0);
    let item = LVITEMW { stateMask: on, state: on, ..Default::default() };
    unsafe {
        SendMessageW(grid.hwnd(), LVM_SETITEMSTATE, Some(WPARAM(row)), Some(LPARAM(&item as *const _ as isize)));
        SendMessageW(grid.hwnd(), LVM_ENSUREVISIBLE, Some(WPARAM(row)), Some(LPARAM(0)));
    }
}
