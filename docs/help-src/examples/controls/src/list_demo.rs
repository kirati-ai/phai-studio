//! ListView: a table of rows with columns - filled, chosen and read.

use crate::list_demo_ui::{ListDemoEvents, ListDemoUi};

#[derive(Default)]
pub struct ListDemo;

impl ListDemoEvents for ListDemo {
    // help: fill
    fn loaded(&mut self, ui: &ListDemoUi) {
        // A row: its cells, one for each column - and picture 0 of the list's Icons.
        ui.pupils.items_add_with_icon(&["Somchai", "P.5/1", "87"], 0);
        ui.pupils.items_add_with_icon(&["Malee", "P.5/2", "92"], 0);
        ui.pupils.items_add_with_icon(&["Narin", "P.5/1", "78"], 0);
    }
    // help: end

    // help: chosen
    fn pupils_chosen(&mut self, ui: &ListDemoUi) {
        if let Some(row) = ui.pupils.selected_index() {
            // cell(row, column) - both counted from 0.
            let name = ui.pupils.cell(row, 0);
            let score = ui.pupils.cell(row, 2);
            ui.shown.set_text(&format!("{name} scored {score}"));
        }
    }
    // help: end
}
