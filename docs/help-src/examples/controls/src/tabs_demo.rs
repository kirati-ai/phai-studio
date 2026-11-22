//! TabControl: pages on one window - which is showing, and going to another.

use crate::tabs_demo_ui::{TabsDemoEvents, TabsDemoUi};

#[derive(Default)]
pub struct TabsDemo;

impl TabsDemoEvents for TabsDemo {
    fn loaded(&mut self, ui: &TabsDemoUi) {
        self.pages_changed(ui);
    }

    // help: changed
    fn pages_changed(&mut self, ui: &TabsDemoUi) {
        // Pages count from 0.
        ui.which.set_text(&format!("Page {} is showing", ui.pages.selected_index() + 1));
    }
    // help: end

    // help: select
    fn to_settings_clicked(&mut self, ui: &TabsDemoUi) {
        ui.pages.select_tab(1); // as a click on the Settings tab
        self.pages_changed(ui);
    }
    // help: end
}
