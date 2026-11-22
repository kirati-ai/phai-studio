//! TablePanel: controls laid out in equal columns - labels on the left, boxes on the right.

use crate::table_demo_ui::{TableDemoEvents, TableDemoUi};

#[derive(Default)]
pub struct TableDemo;

impl TableDemoEvents for TableDemo {
    // help: join
    fn join_clicked(&mut self, ui: &TableDemoUi) {
        let full = format!("{} {}", ui.first.text().trim(), ui.last.text().trim());
        ui.set_title(full.trim());
    }
    // help: end
}
