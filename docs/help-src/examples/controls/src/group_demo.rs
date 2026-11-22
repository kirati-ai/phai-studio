//! GroupBox: two questions on one window, each in its own frame.

use crate::group_demo_ui::{GroupDemoEvents, GroupDemoUi};

#[derive(Default)]
pub struct GroupDemo;

impl GroupDemoEvents for GroupDemo {
    // help: read
    fn order_clicked(&mut self, ui: &GroupDemoUi) {
        let how = if ui.send.checked() { "sent to you" } else { "picked up" };
        let pay = if ui.transfer.checked() { "a bank transfer" } else { "cash" };
        ui.summary.set_text(&format!("{how}, paid by {pay}"));
    }
    // help: end
}
