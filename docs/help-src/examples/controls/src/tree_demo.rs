//! TreeView: items with items under them - built in code, and the one chosen read.

use crate::tree_demo_ui::{TreeDemoEvents, TreeDemoUi};

#[derive(Default)]
pub struct TreeDemo;

impl TreeDemoEvents for TreeDemo {
    // help: build
    fn loaded(&mut self, ui: &TreeDemoUi) {
        // nodes_add gives back the item, to add others under it.
        let mammals = ui.zoo.nodes_add("Mammals");
        ui.zoo.nodes_add_under(mammals, "Elephant");
        ui.zoo.nodes_add_under(mammals, "Tiger");
        let birds = ui.zoo.nodes_add("Birds");
        ui.zoo.nodes_add_under(birds, "Hornbill");
        ui.zoo.nodes_add_under(birds, "Peacock");
        ui.zoo.expand_all();
    }
    // help: end

    // help: chosen
    fn zoo_chosen(&mut self, ui: &TreeDemoUi) {
        if let Some(name) = ui.zoo.selected_text() {
            ui.chosen.set_text(&name);
        }
    }
    // help: end
}
