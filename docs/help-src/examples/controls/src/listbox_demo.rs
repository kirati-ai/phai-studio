//! ListBox: a to-do list - lines added, chosen and taken away.

use crate::listbox_demo_ui::{ListboxDemoEvents, ListboxDemoUi};

#[derive(Default)]
pub struct ListboxDemo;

impl ListboxDemoEvents for ListboxDemo {
    // help: chosen
    fn tasks_chosen(&mut self, ui: &ListboxDemoUi) {
        if let Some(t) = ui.tasks.selected_item() {
            ui.status.set_text(&format!("Chosen: {t}"));
        }
    }
    // help: end

    // help: add
    fn add_clicked(&mut self, ui: &ListboxDemoUi) {
        let task = ui.new_task.text();
        if !task.trim().is_empty() {
            ui.tasks.items_add(task.trim());
            ui.new_task.set_text("");
        }
    }
    // help: end

    // help: remove
    fn done_clicked(&mut self, ui: &ListboxDemoUi) {
        // The line chosen goes; nothing chosen, nothing happens.
        if let Some(i) = ui.tasks.selected_index() {
            ui.tasks.items_remove_at(i);
            ui.status.set_text(&format!("{} left to do", ui.tasks.items_count()));
        }
    }
    // help: end
}
