//! BindingSource: rows of data, the current one shown in the text boxes bound to its columns.

use crate::bindingsource_demo_ui::{BindingsourceDemoEvents, BindingsourceDemoUi};

#[derive(Default)]
pub struct BindingsourceDemo;

impl BindingsourceDemoEvents for BindingsourceDemo {
    // help: rows
    fn loaded(&mut self, ui: &BindingsourceDemoUi) {
        // Each row in the columns' order: name, phone. The first one shows at once.
        ui.people.set_rows(vec![
            vec!["Somchai".into(), "081-111-1111".into()],
            vec!["Malee".into(), "082-222-2222".into()],
            vec!["Narin".into(), "083-333-3333".into()],
        ]);
        self.people_moved(ui);
    }
    // help: end

    // help: move
    fn back_clicked(&mut self, ui: &BindingsourceDemoUi) {
        ui.people.move_previous();
    }

    fn next_clicked(&mut self, ui: &BindingsourceDemoUi) {
        ui.people.move_next();
    }

    fn people_moved(&mut self, ui: &BindingsourceDemoUi) {
        // PositionChanged: another row is the current one.
        ui.place.set_text(&format!("{} of {}", ui.people.position() + 1, ui.people.count()));
    }
    // help: end
}
