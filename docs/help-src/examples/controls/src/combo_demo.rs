//! ComboBox: choosing from a list, and adding to it.

use crate::combo_demo_ui::{ComboDemoEvents, ComboDemoUi};

#[derive(Default)]
pub struct ComboDemo;

impl ComboDemoEvents for ComboDemo {
    // help: changed
    fn fruit_changed(&mut self, ui: &ComboDemoUi) {
        // The item chosen, if there is one.
        if let Some(f) = ui.fruit.selected_item() {
            ui.chosen.set_text(&format!("You chose {f}."));
        }
    }
    // help: end

    // help: add
    fn add_clicked(&mut self, ui: &ComboDemoUi) {
        let name = ui.new_fruit.text();
        let name = name.trim();
        if name.is_empty() || ui.fruit.find_string_exact(name).is_some() {
            return; // nothing typed, or already in the list
        }
        let at = ui.fruit.items_add(name);
        ui.fruit.set_selected_index(Some(at)); // and chosen
        self.fruit_changed(ui);
        ui.new_fruit.set_text("");
    }
    // help: end
}
