//! MonthCalendar: a whole month on the window; the day clicked is read.

use crate::calendar_demo_ui::{CalendarDemoEvents, CalendarDemoUi};

#[derive(Default)]
pub struct CalendarDemo;

impl CalendarDemoEvents for CalendarDemo {
    // help: set
    fn loaded(&mut self, ui: &CalendarDemoUi) {
        ui.month.set_date(2026, 12, 25);
        self.month_changed(ui);
    }
    // help: end

    // help: changed
    fn month_changed(&mut self, ui: &CalendarDemoUi) {
        let (y, m, d) = ui.month.selection_start();
        ui.chosen.set_text(&format!("You chose {d}/{m}/{y}"));
    }
    // help: end
}
