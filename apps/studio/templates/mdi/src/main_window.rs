//! MainWindow: the window the documents open in (`is_mdi_container` in its form), and what its
//! menus do. Phai Studio made this file once; it is yours.
//! The window itself is in main_window_ui.rs, which Phai Studio writes from main_window.phf.

use crate::document_window::DocumentWindow;
use crate::document_window_ui::DocumentWindowUi;
use crate::main_window_ui::{MainWindowEvents, MainWindowUi};
use phai_ui::prelude::*;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};

#[derive(Default)]
pub struct MainWindow {
    /// How many documents were made so far: the next one's number in its title.
    made: u32,
}

impl MainWindowEvents for MainWindow {
    /// The window is on the screen: one empty document to start with.
    fn first_shown(&mut self, ui: &MainWindowUi) {
        self.new_document(ui);
        // For the picture of the program Phai Studio shows: two documents, one over the other.
        if std::env::var_os("PHAI_SHOT").is_some() {
            self.new_document(ui);
        }
    }

    /// File > New: a new document inside the window.
    fn new_document(&mut self, ui: &MainWindowUi) {
        self.made += 1;
        // The document tells this window when it closes, so the count below stays right.
        DocumentWindowUi::show_mdi_child(ui.hwnd, DocumentWindow::new(self.made, ui.invoker()));
        self.documents_changed(ui);
    }

    /// File > Close: closes the document in front, as its own close box would.
    fn close_document(&mut self, ui: &MainWindowUi) {
        if let Some(doc) = ui.active_mdi_child() {
            unsafe {
                let _ = PostMessageW(Some(doc), WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }
    }

    fn exit(&mut self, ui: &MainWindowUi) {
        ui.close();
    }

    // The Window menu: the documents laid out in the window.
    fn cascade(&mut self, ui: &MainWindowUi) {
        ui.layout_mdi("cascade");
    }

    fn tile_horizontally(&mut self, ui: &MainWindowUi) {
        ui.layout_mdi("tile_horizontal");
    }

    fn tile_vertically(&mut self, ui: &MainWindowUi) {
        ui.layout_mdi("tile_vertical");
    }

    /// The documents made small (minimized) in a row at the bottom.
    fn arrange_icons(&mut self, ui: &MainWindowUi) {
        ui.layout_mdi("arrange_icons");
    }

    fn about(&mut self, ui: &MainWindowUi) {
        ui.message_box(t("A program with documents in one window, made with Phai Studio."), t("About"), MessageBoxButtons::Ok, MessageBoxIcon::Information);
    }
}

impl MainWindow {
    /// A document opened or closed: the status bar says how many are open, and the menu items
    /// that need a document are grey without one.
    pub fn documents_changed(&mut self, ui: &MainWindowUi) {
        let open = ui.mdi_children().len();
        let words = match open {
            0 => t("No documents open").to_string(),
            1 => t("1 document open").to_string(),
            n => tf("{0} documents open", &[&n]),
        };
        ui.status.set_part_text(0, &words);
        for item in ["close_document", "cascade", "tile_horizontally", "tile_vertically", "arrange_icons"] {
            ui.set_item_enabled(item, open > 0);
        }
    }
}
