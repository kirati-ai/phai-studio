//! FlowPanel: controls laid out one after another - hide one and the rest close up.

use crate::flow_demo_ui::{FlowDemoEvents, FlowDemoUi};

#[derive(Default)]
pub struct FlowDemo;

impl FlowDemoEvents for FlowDemo {
    // help: hide
    fn hide_green_clicked(&mut self, ui: &FlowDemoUi) {
        // The panel lays its controls out again: the others move up into the gap.
        let showing = ui.green.visible();
        ui.green.set_visible(!showing);
        ui.hide_green.set_text(if showing { "Show green" } else { "Hide green" });
    }
    // help: end
}
