//! CheckedListBox: a list to tick several things in, and reading which are ticked.

use crate::checkedlist_demo_ui::{CheckedlistDemoEvents, CheckedlistDemoUi};

#[derive(Default)]
pub struct CheckedlistDemo;

impl CheckedlistDemoEvents for CheckedlistDemo {
    // help: fill
    fn loaded(&mut self, ui: &CheckedlistDemoUi) {
        for t in ["Cheese", "Ham", "Mushroom", "Pineapple", "Olives"] {
            ui.toppings.items_add(t);
        }
        ui.toppings.set_checked(0, true); // cheese, ticked from the start
    }
    // help: end

    // help: read
    fn order_clicked(&mut self, ui: &CheckedlistDemoUi) {
        let chosen: Vec<String> = ui.toppings.checked_indices().into_iter().map(|i| ui.toppings.item(i)).collect();
        if chosen.is_empty() {
            ui.summary.set_text("Nothing ticked yet.");
        } else {
            ui.summary.set_text(&format!("A pizza with {}.", chosen.join(", ")));
        }
    }
    // help: end
}
