//! MainWindow: what its controls do. Phai Studio made this file once; it is ours.
//! The window itself is in main_window_ui.rs, which Phai Studio writes from main_window.phf.
//! Its controls are objects, as WinForms': `ui.name.text()`, `ui.customers.items_add(...)`.

use crate::main_window_ui::{MainWindowEvents, MainWindowUi};
use phai_ui::text::tf;

#[derive(Default)]
pub struct MainWindow {
    customers: Vec<(String, String)>,
}

impl MainWindowEvents for MainWindow {
    fn loaded(&mut self, ui: &MainWindowUi) {
        self.customers = vec![("สมชาย ใจดี".into(), "081-234-5678".into()), ("Monrak Net".into(), "02-111-2222".into())];
        for (name, phone) in &self.customers {
            ui.customers.items_add(&[name, phone]);
        }
        ui.status.set_part_text(1, &tf("{0} customers", &[&self.customers.len()]));
    }

    fn new_customer(&mut self, ui: &MainWindowUi) {
        ui.name.clear();
        ui.name.focus();
    }

    fn exit(&mut self, ui: &MainWindowUi) {
        ui.close();
    }

    fn save(&mut self, ui: &MainWindowUi) {
        let name = ui.name.text();
        if !name.trim().is_empty() {
            ui.customers.items_add(&[&name, ""]);
            self.customers.push((name.clone(), String::new()));
            ui.status.set_part_text(0, &tf("Saved {0}", &[&name]));
            ui.status.set_part_text(1, &tf("{0} customers", &[&self.customers.len()]));
        }
    }

    fn name_changed(&mut self, ui: &MainWindowUi) {
        ui.save_button.set_enabled(!ui.name.text().trim().is_empty());
    }

    fn customer_chosen(&mut self, _ui: &MainWindowUi) {}
}
