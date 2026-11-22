//! Timer: something done again and again - a second counted, started and stopped.

use crate::timer_demo_ui::{TimerDemoEvents, TimerDemoUi};

#[derive(Default)]
pub struct TimerDemo {
    seconds: u32,
}

impl TimerDemoEvents for TimerDemo {
    // help: tick
    fn clock_ticked(&mut self, ui: &TimerDemoUi) {
        // Every Interval (1000 ms = one second) while it is enabled.
        self.seconds += 1;
        ui.time.set_text(&format!("{} s", self.seconds));
    }
    // help: end

    // help: startstop
    fn start_stop_clicked(&mut self, ui: &TimerDemoUi) {
        if ui.clock.enabled() {
            ui.clock.stop();
            ui.start_stop.set_text("Start");
        } else {
            ui.clock.start();
            ui.start_stop.set_text("Stop");
        }
    }
    // help: end
}
