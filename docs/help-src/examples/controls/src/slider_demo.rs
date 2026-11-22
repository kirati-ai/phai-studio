//! TrackBar: a volume chosen by dragging - read, and set from code.

use crate::slider_demo_ui::{SliderDemoEvents, SliderDemoUi};

#[derive(Default)]
pub struct SliderDemo;

impl SliderDemoEvents for SliderDemo {
    fn loaded(&mut self, ui: &SliderDemoUi) {
        self.volume_changed(ui);
    }

    // help: changed
    fn volume_changed(&mut self, ui: &SliderDemoUi) {
        ui.shown.set_text(&format!("Volume: {}", ui.volume.value()));
    }
    // help: end

    // help: set
    fn mute_clicked(&mut self, ui: &SliderDemoUi) {
        ui.volume.set_value(0);
        self.volume_changed(ui);
    }
    // help: end
}
