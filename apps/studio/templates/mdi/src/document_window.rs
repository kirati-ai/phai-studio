//! DocumentWindow: one document, a page to type on, open inside the main window.
//! The window itself is in document_window_ui.rs, which Phai Studio writes from document_window.phf.

use crate::document_window_ui::{DocumentWindowEvents, DocumentWindowUi};
use crate::main_window::MainWindow;
use crate::main_window_ui::MainWindowUi;
use phai_ui::invoke::Invoker;
use phai_ui::prelude::*;

pub struct DocumentWindow {
    /// Its number, for its title: Document 1, Document 2...
    number: u32,
    /// The main window, to tell it when this document closes.
    main: Invoker,
}

impl DocumentWindow {
    pub fn new(number: u32, main: Invoker) -> Self {
        DocumentWindow { number, main }
    }
}

impl DocumentWindowEvents for DocumentWindow {
    fn loaded(&mut self, ui: &DocumentWindowUi) {
        ui.set_title(&tf("Document {0}", &[&self.number]));
    }

    /// Closed: the main window counts its documents again, once this one is gone.
    fn document_closed(&mut self, _ui: &DocumentWindowUi) {
        MainWindowUi::begin_invoke::<MainWindow>(self.main, |main, ui| main.documents_changed(ui));
    }
}
