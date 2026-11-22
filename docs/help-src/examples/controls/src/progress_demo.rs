//! ProgressBar: how much of a job is done, a step at a time.

use crate::progress_demo_ui::{ProgressDemoEvents, ProgressDemoUi};

#[derive(Default)]
pub struct ProgressDemo;

impl ProgressDemoEvents for ProgressDemo {
    // help: step
    fn next_clicked(&mut self, ui: &ProgressDemoUi) {
        ui.work.perform_step(); // adds Step (10)
        let v = ui.work.value();
        ui.done.set_text(&format!("{v}% done"));
        if v >= 100 {
            ui.next.set_enabled(false);
        }
    }
    // help: end

    // help: again
    fn again_clicked(&mut self, ui: &ProgressDemoUi) {
        ui.work.set_value(0);
        ui.done.set_text("");
        ui.next.set_enabled(true);
    }
    // help: end
}
