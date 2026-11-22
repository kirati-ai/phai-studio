//! SplitContainer: two panes with a bar between them the user drags.

use crate::split_demo_ui::{SplitDemoEvents, SplitDemoUi};

#[derive(Default)]
pub struct SplitDemo;

impl SplitDemoEvents for SplitDemo {
    fn loaded(&mut self, ui: &SplitDemoUi) {
        self.halves_changed(ui);
    }

    // help: moved
    fn halves_changed(&mut self, ui: &SplitDemoUi) {
        // The bar was dragged: how wide the left pane is now.
        ui.width.set_text(&format!("Left pane: {} px - drag the bar", ui.halves.splitter_distance()));
    }
    // help: end
}
