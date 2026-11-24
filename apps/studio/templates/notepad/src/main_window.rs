//! MainWindow: a text editor like Notepad. One file at a time; the title bar shows its name, and
//! a dot when it has changes not yet saved, which are asked about before they could be lost.
//! The window itself is in main_window_ui.rs, which Phai Studio writes from main_window.phf.

use std::path::{Path, PathBuf};

use crate::main_window_ui::{MainWindowEvents, MainWindowUi};
use phai_ui::components::ChosenFont;
use phai_ui::prelude::*;

pub struct MainWindow {
    /// The file being edited; `None` until it is saved for the first time.
    path: Option<PathBuf>,
    /// The text as it was opened or last saved, to tell a real change from none.
    saved: String,
    /// Changed since it was opened or saved.
    changed: bool,
    /// The words Find looks for.
    find_what: String,
    /// The font of the text, as the Font dialog last chose it.
    font: ChosenFont,
}

impl Default for MainWindow {
    fn default() -> Self {
        MainWindow {
            path: None,
            saved: String::new(),
            changed: false,
            find_what: String::new(),
            // The same as the form gives the text box (font_family, font_size).
            font: ChosenFont { face: "Consolas".into(), points: 11, bold: false, italic: false },
        }
    }
}

impl MainWindowEvents for MainWindow {
    fn loaded(&mut self, ui: &MainWindowUi) {
        // A file named when the program is started ("editor.exe notes.txt") opens at once.
        match std::env::args().skip(1).find(|a| !a.starts_with("--")) {
            Some(file) => self.open_path(ui, Path::new(&file)),
            None => self.show_title(ui),
        }
    }

    // --- File ---------------------------------------------------------------------------------

    fn file_new(&mut self, ui: &MainWindowUi) {
        if !self.may_lose_changes(ui) {
            return;
        }
        ui.editor.set_text("");
        self.path = None;
        self.saved.clear();
        self.changed = false;
        self.show_title(ui);
    }

    fn file_open(&mut self, ui: &MainWindowUi) {
        if !self.may_lose_changes(ui) {
            return;
        }
        // None: Cancel was pressed.
        if let Some(path) = ui.open_dialog.show_dialog() {
            self.open_path(ui, &path);
        }
    }

    fn file_save(&mut self, ui: &MainWindowUi) {
        self.save(ui);
    }

    fn file_save_as(&mut self, ui: &MainWindowUi) {
        self.save_as(ui);
    }

    fn exit(&mut self, ui: &MainWindowUi) {
        // Closing asks about unsaved changes (closing, below).
        ui.close();
    }

    /// The window is about to close: unsaved changes are asked about first.
    fn closing(&mut self, ui: &MainWindowUi, e: &ClosingEventArgs) {
        if !self.may_lose_changes(ui) {
            e.cancel();
        }
    }

    // --- Edit ---------------------------------------------------------------------------------

    /// The Edit menu is opening: greyed what cannot be done now.
    fn edit_menu_opening(&mut self, ui: &MainWindowUi) {
        let chosen = ui.editor.selection_length() > 0;
        ui.set_item_enabled("undo", ui.editor.can_undo());
        ui.set_item_enabled("cut", chosen);
        ui.set_item_enabled("copy", chosen);
        ui.set_item_enabled("paste", !clipboard_text().is_empty());
        ui.set_item_enabled("find_next", !self.find_what.is_empty());
    }

    fn undo(&mut self, ui: &MainWindowUi) {
        ui.editor.undo();
    }

    fn cut(&mut self, ui: &MainWindowUi) {
        ui.editor.cut();
    }

    fn copy(&mut self, ui: &MainWindowUi) {
        ui.editor.copy();
    }

    fn paste(&mut self, ui: &MainWindowUi) {
        // As plain words: colours and fonts copied from elsewhere are left behind.
        ui.editor.paste_text(&clipboard_text());
    }

    fn select_all(&mut self, ui: &MainWindowUi) {
        ui.editor.select_all();
    }

    fn find(&mut self, ui: &MainWindowUi) {
        if let Some(what) = ui.prompt(t("Find what:"), &self.find_what) {
            self.find_what = what;
            self.find_next(ui);
        }
    }

    /// The next place the Find words are, after the words chosen now - from the top again
    /// when there are no more below.
    fn find_next(&mut self, ui: &MainWindowUi) {
        if self.find_what.is_empty() {
            self.find(ui);
            return;
        }
        let after = ui.editor.selection_start() + ui.editor.selection_length();
        if ui.editor.find(&self.find_what, after).or_else(|| ui.editor.find(&self.find_what, 0)).is_some() {
            ui.editor.scroll_to_caret();
        } else {
            ui.alert(&tf("Cannot find \"{0}\"", &[&self.find_what]));
        }
        ui.editor.focus();
    }

    // --- Format -------------------------------------------------------------------------------

    /// Word Wrap ticked: long lines wrap at the window's edge; not ticked, they scroll sideways.
    fn word_wrap(&mut self, ui: &MainWindowUi) {
        // CheckOnClick: the tick has already turned over; ask which way it is now.
        phai_ui::props::word_wrap(ui.editor.hwnd(), ui.item_checked("word_wrap"));
    }

    fn choose_font(&mut self, ui: &MainWindowUi) {
        let Some(font) = ui.font_dialog.show_dialog_from(&self.font) else { return };
        let weight = if font.bold { "bold" } else { "normal" };
        let style = format!("font-family: {}; font-size: {}pt; font-weight: {weight}", font.face, font.points);
        let _ = phai_ui::win::set_own_style(ui.editor.hwnd(), &style);
        self.font = font;
    }

    // --- The text -----------------------------------------------------------------------------

    fn editor_changed(&mut self, ui: &MainWindowUi) {
        // The box also says "changed" when only its font changes: the words are compared.
        if !self.changed && ui.editor.text() != self.saved {
            self.changed = true;
            self.show_title(ui);
        }
    }
}

impl MainWindow {
    /// Reads a file into the editor.
    fn open_path(&mut self, ui: &MainWindowUi, path: &Path) {
        match std::fs::read(path) {
            Ok(bytes) => {
                // UTF-8, with or without the mark some editors put at the start.
                let text = String::from_utf8_lossy(bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes)).into_owned();
                ui.editor.set_text(&text);
                ui.editor.select(0, 0);
                // As the box holds it - its line ends are Windows' own.
                self.saved = ui.editor.text();
                self.path = Some(path.to_path_buf());
                self.changed = false;
                self.show_title(ui);
            }
            Err(e) => ui.alert(&tf("Cannot open {0}: {1}", &[&path.display(), &e])),
        }
    }

    /// Saves to the file it came from, or asks where for a new one. `false` when it was not saved.
    fn save(&mut self, ui: &MainWindowUi) -> bool {
        match self.path.clone() {
            Some(path) => self.write_to(ui, &path),
            None => self.save_as(ui),
        }
    }

    /// Asks where to save, then saves there. `false` when Cancel was pressed or it failed.
    fn save_as(&mut self, ui: &MainWindowUi) -> bool {
        let name = self.file_name();
        match ui.save_dialog.show_dialog_named(&name) {
            Some(path) => self.write_to(ui, &path),
            None => false,
        }
    }

    fn write_to(&mut self, ui: &MainWindowUi, path: &Path) -> bool {
        let text = ui.editor.text();
        match std::fs::write(path, &text) {
            Ok(()) => {
                self.saved = text;
                self.path = Some(path.to_path_buf());
                self.changed = false;
                self.show_title(ui);
                true
            }
            Err(e) => {
                ui.alert(&tf("Cannot save {0}: {1}", &[&path.display(), &e]));
                false
            }
        }
    }

    /// Before the text is replaced or the window closes: changes not saved are asked about.
    /// `true` when it is fine to go on - saved, or the person said not to save.
    fn may_lose_changes(&mut self, ui: &MainWindowUi) -> bool {
        if !self.changed {
            return true;
        }
        let question = tf("Save the changes to {0}?", &[&self.file_name()]);
        match ui.message_box(&question, t("Text editor"), MessageBoxButtons::YesNoCancel, MessageBoxIcon::Warning) {
            DialogResult::Yes => self.save(ui),
            DialogResult::No => true,
            _ => false,
        }
    }

    /// The file's name, or "Untitled" for a new one.
    fn file_name(&self) -> String {
        match self.path.as_ref().and_then(|p| p.file_name()) {
            Some(name) => name.to_string_lossy().into_owned(),
            None => t("Untitled").to_string(),
        }
    }

    /// "notes.txt - Text editor", with a dot before the name while there are unsaved changes.
    fn show_title(&self, ui: &MainWindowUi) {
        let name = if self.changed { format!("• {}", self.file_name()) } else { self.file_name() };
        ui.set_title(&tf("{0} - Text editor", &[&name]));
    }
}
