//! CounterBox: a user control - a number with - and + - that tells the form when it changes.

use crate::counter_box_ui::{CounterBoxEvents, CounterBoxUi};

// help: uc
#[derive(Default)]
pub struct CounterBox {
    n: i32,
}

impl CounterBoxEvents for CounterBox {
    fn minus_clicked(&mut self, ui: &CounterBoxUi) {
        self.n -= 1;
        ui.value.set_text(&self.n.to_string());
        ui.notify_parent(); // the form's on_change for it runs
    }

    fn plus_clicked(&mut self, ui: &CounterBoxUi) {
        self.n += 1;
        ui.value.set_text(&self.n.to_string());
        ui.notify_parent();
    }
}
// help: end
