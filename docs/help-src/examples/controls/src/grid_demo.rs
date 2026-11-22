//! DataGridView: a table people can type into - filled, read and sorted from code.

use crate::grid_demo_ui::{GridDemoEvents, GridDemoUi};

#[derive(Default)]
pub struct GridDemo;

impl GridDemoEvents for GridDemo {
    // help: fill
    fn loaded(&mut self, ui: &GridDemoUi) {
        ui.scores.rows_add(&["Somchai", "87"]);
        ui.scores.rows_add(&["Malee", "92"]);
        ui.scores.rows_add(&["Narin", "78"]);
    }
    // help: end

    // help: read
    fn total_clicked(&mut self, ui: &GridDemoUi) {
        // rows(): every row's cells, as typed - a score typed wrongly is left out.
        let scores: Vec<f64> = ui.scores.rows().iter().filter_map(|r| r.get(1)?.trim().parse().ok()).collect();
        if !scores.is_empty() {
            let average = scores.iter().sum::<f64>() / scores.len() as f64;
            ui.result.set_text(&format!("Average {average:.1}"));
        }
    }
    // help: end

    // help: sort
    fn best_clicked(&mut self, ui: &GridDemoUi) {
        ui.scores.sort_by(1, false); // column 1 (Score), largest first
    }
    // help: end
}
