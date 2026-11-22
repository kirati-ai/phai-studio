//! DateTimePicker: a date chosen from a calendar, set and read in code.

use crate::date_demo_ui::{DateDemoEvents, DateDemoUi};

#[derive(Default)]
pub struct DateDemo;

impl DateDemoEvents for DateDemo {
    // help: set
    fn loaded(&mut self, ui: &DateDemoUi) {
        ui.birthday.set_value(2015, 6, 1); // year, month, day
        self.birthday_changed(ui);
    }
    // help: end

    // help: changed
    fn birthday_changed(&mut self, ui: &DateDemoUi) {
        let (year, month, day) = ui.birthday.value();
        ui.shown.set_text(&format!("Born on {day}/{month}/{year}"));
    }
    // help: end
}
