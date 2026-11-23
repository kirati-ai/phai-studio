//! TrayWindow: a program that lives in its icon in the corner of the taskbar. Its window starts
//! hidden (`start_hidden`); the icon's menu has a menu within it with a ticked item, and an item
//! shown only while it matters; a click on its notice opens the window.
//! The window itself is in tray_window_ui.rs, which Phai Studio writes from tray_window.phf.

use crate::tray_window_ui::{TrayWindowEvents, TrayWindowUi};
use phai_ui::text::t;

#[derive(Default)]
pub struct TrayWindow {
    /// Exit chosen: the window closes for good, not only out of sight.
    quitting: bool,
}

impl TrayWindowEvents for TrayWindow {
    /// The menu is about to open: End remote session only while there is one.
    fn tray_menu_opening(&mut self, ui: &TrayWindowUi) {
        ui.set_menu_item_visible("end_session", ui.menu_item_checked("pretend_session"));
    }

    fn open_window(&mut self, ui: &TrayWindowUi) {
        ui.show_window();
        ui.activate();
    }

    fn pretend_session(&mut self, _ui: &TrayWindowUi) {
        // CheckOnClick has turned its tick over already.
    }

    fn end_session(&mut self, ui: &TrayWindowUi) {
        ui.set_menu_item_checked("pretend_session", false);
    }

    fn english(&mut self, ui: &TrayWindowUi) {
        self.language(ui, false);
    }

    fn thai(&mut self, ui: &TrayWindowUi) {
        self.language(ui, true);
    }

    fn show_notice(&mut self, ui: &TrayWindowUi) {
        ui.tray.show_balloon_tip(t("Phai Tray"), t("Click this notice to open the window."));
    }

    fn notice_clicked(&mut self, ui: &TrayWindowUi) {
        self.open_window(ui);
    }

    fn hide_clicked(&mut self, ui: &TrayWindowUi) {
        ui.hide();
    }

    /// The close box only hides it; Exit ends the program.
    fn window_closing(&mut self, ui: &TrayWindowUi, e: &phai_ui::events::ClosingEventArgs) {
        if !self.quitting {
            e.cancel();
            ui.hide();
        }
    }

    fn exit(&mut self, ui: &TrayWindowUi) {
        self.quitting = true;
        ui.close();
    }
}

impl TrayWindow {
    /// Thai or English, the menu's tick on the one chosen.
    fn language(&mut self, ui: &TrayWindowUi, thai: bool) {
        phai_ui::text::set_thai(thai);
        ui.set_menu_item_checked("english", !thai);
        ui.set_menu_item_checked("thai", thai);
        ui.relabel();
        ui.tray.set_text(t("Phai Tray"));
    }
}
