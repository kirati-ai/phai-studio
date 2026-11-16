//! MainWindow: what its controls do. Phai Studio made this file once; it is ours.
//! The window itself is in main_window_ui.rs, which Phai Studio writes from main_window.phaiform.

use crate::main_window_ui::{MainWindowEvents, MainWindowUi};
use phai_ui::text::tf;
use phai_ui::{controls, win};

#[derive(Default)]
pub struct MainWindow {
    customers: Vec<(String, String)>,
}

impl MainWindowEvents for MainWindow {
    fn loaded(&mut self, ui: &MainWindowUi) {
        self.customers = vec![("สมชาย ใจดี".into(), "081-234-5678".into()), ("Monrak Net".into(), "02-111-2222".into())];
        for (name, phone) in &self.customers {
            win::list_add(ui.customers, &[name, phone]);
        }
        controls::status_text(ui.status, 1, &tf("{0} customers", &[&self.customers.len()]));
    }

    fn new_customer(&mut self, ui: &MainWindowUi) {
        win::set_text(ui.name, "");
        unsafe {
            let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(ui.name));
        }
    }

    fn exit(&mut self, ui: &MainWindowUi) {
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(ui.hwnd);
        }
    }

    fn save(&mut self, ui: &MainWindowUi) {
        let name = win::text_of(ui.name);
        if !name.trim().is_empty() {
            win::list_add(ui.customers, &[&name, ""]);
            self.customers.push((name.clone(), String::new()));
            controls::status_text(ui.status, 0, &tf("Saved {0}", &[&name]));
            controls::status_text(ui.status, 1, &tf("{0} customers", &[&self.customers.len()]));
        }
    }

    fn name_changed(&mut self, ui: &MainWindowUi) {
        win::enable(ui.save_button, !win::text_of(ui.name).trim().is_empty());
    }

    fn customer_chosen(&mut self, _ui: &MainWindowUi) {}
}
