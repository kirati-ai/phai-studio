//! UserControl: the project's own CounterBox, twice on one window.

use crate::counter_box_ui::CounterBoxUi;
use crate::usercontrol_demo_ui::{UsercontrolDemoEvents, UsercontrolDemoUi};

#[derive(Default)]
pub struct UsercontrolDemo;

impl UsercontrolDemoEvents for UsercontrolDemo {
    // help: read
    fn count_changed(&mut self, ui: &UsercontrolDemoUi) {
        // CounterBoxUi::with gives a CounterBox's own controls.
        let read = |h| CounterBoxUi::with(h, |c| c.value.text().parse::<i32>().unwrap_or(0)).unwrap_or(0);
        let people = read(ui.adults) + read(ui.children);
        ui.total.set_text(&format!("{people} people"));
    }
    // help: end
}
