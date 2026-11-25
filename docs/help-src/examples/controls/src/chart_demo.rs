//! Chart: a table drawn as columns, lines or a pie; and points added as they come in.

use crate::chart_demo_ui::{ChartDemoEvents, ChartDemoUi};

pub struct ChartDemo {
    price: f64,
    ticks: u32,
    seed: u32,
}

impl Default for ChartDemo {
    fn default() -> Self {
        ChartDemo { price: 100.0, ticks: 0, seed: 7 }
    }
}

impl ChartDemo {
    /// A step up or down, -1.0 to 1.0 - made up, for the example.
    fn wobble(&mut self) -> f64 {
        self.seed = self.seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        ((self.seed >> 16) % 2001) as f64 / 1000.0 - 1.0
    }
}

impl ChartDemoEvents for ChartDemo {
    fn loaded(&mut self, ui: &ChartDemoUi) {
        // A few points to start with, so the line is there from the first moment.
        for _ in 0..12 {
            self.clock_ticked(ui);
        }
    }

    // help: kinds
    fn column_clicked(&mut self, ui: &ChartDemoUi) {
        ui.sales.set_type("column");
    }

    fn line_clicked(&mut self, ui: &ChartDemoUi) {
        ui.sales.set_type("line");
    }

    fn pie_clicked(&mut self, ui: &ChartDemoUi) {
        // A pie draws the first series only: one slice a month.
        ui.sales.set_type("pie");
    }
    // help: end

    // help: live
    fn clock_ticked(&mut self, ui: &ChartDemoUi) {
        // Every half second: one more point. MaxPoints (30) keeps the last 30.
        self.ticks += 1;
        self.price = (self.price + self.wobble()).max(1.0);
        ui.price.add_point(&self.ticks.to_string(), &[self.price]);
    }
    // help: end

    // help: startstop
    fn start_stop_clicked(&mut self, ui: &ChartDemoUi) {
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
