//! NumericUpDown: a number chosen with its arrows, kept within its range.

use crate::number_demo_ui::{NumberDemoEvents, NumberDemoUi};

#[derive(Default)]
pub struct NumberDemo;

const PRICE: i32 = 120;

impl NumberDemoEvents for NumberDemo {
    fn loaded(&mut self, ui: &NumberDemoUi) {
        self.tickets_changed(ui);
    }

    // help: changed
    fn tickets_changed(&mut self, ui: &NumberDemoUi) {
        let n = ui.tickets.value();
        ui.price.set_text(&format!("{n} tickets: {} baht", n * PRICE));
    }
    // help: end
}
