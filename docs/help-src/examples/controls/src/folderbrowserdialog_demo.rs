//! FolderBrowserDialog: Windows' own window for choosing a folder.

use crate::folderbrowserdialog_demo_ui::{FolderbrowserdialogDemoEvents, FolderbrowserdialogDemoUi};

#[derive(Default)]
pub struct FolderbrowserdialogDemo;

impl FolderbrowserdialogDemoEvents for FolderbrowserdialogDemo {
    // help: choose
    fn choose_clicked(&mut self, ui: &FolderbrowserdialogDemoUi) {
        let Some(folder) = ui.folder_dialog.show_dialog() else { return };
        // How many things are in it.
        let n = std::fs::read_dir(&folder).map(|d| d.count()).unwrap_or(0);
        ui.chosen.set_text(&format!("{} - {n} things in it", folder.display()));
    }
    // help: end
}
