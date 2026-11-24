//! MainWindow: a program that lives in its icon in the corner of the taskbar (the notify icon).
//! Its window starts hidden (`start_hidden` in the form); the icon's right-click menu opens it,
//! turns "Start with Windows" on or off, and ends the program. Closing the window only hides it.
//! Phai Studio made this file once; it is yours.
//! The window itself is in main_window_ui.rs, which Phai Studio writes from main_window.phf.

use crate::main_window_ui::{MainWindowEvents, MainWindowUi};
use phai_ui::prelude::*;

#[derive(Default)]
pub struct MainWindow {
    /// Exit chosen: the window closes for good, not only out of sight.
    quitting: bool,
    /// The notice "still running" was shown once already.
    told: bool,
}

impl MainWindowEvents for MainWindow {
    fn loaded(&mut self, ui: &MainWindowUi) {
        self.show_startup_state(ui);
        // For the picture of the program Phai Studio shows: the window, not only the icon.
        if std::env::var_os("PHAI_SHOT").is_some() {
            ui.show_window();
        }
    }

    /// The menu is about to open: its tick says what Windows has now.
    fn tray_menu_opening(&mut self, ui: &MainWindowUi) {
        ui.set_menu_item_checked("start_with_windows", startup::is_on());
    }

    /// Open, or a double-click on the icon: the window comes to the front.
    fn open_window(&mut self, ui: &MainWindowUi) {
        ui.show_window();
        ui.activate();
    }

    /// Start with Windows: turned over, and the window says how it is now.
    fn start_with_windows(&mut self, ui: &MainWindowUi) {
        if !startup::set(!startup::is_on()) {
            ui.alert(t("Windows did not take the change."));
        }
        self.show_startup_state(ui);
    }

    fn hide_clicked(&mut self, ui: &MainWindowUi) {
        self.hide(ui);
    }

    /// The close box only hides the window; Exit on the icon's menu ends the program.
    fn window_closing(&mut self, ui: &MainWindowUi, e: &ClosingEventArgs) {
        if !self.quitting {
            e.cancel();
            self.hide(ui);
        }
    }

    fn exit(&mut self, ui: &MainWindowUi) {
        self.quitting = true;
        ui.close();
    }
}

impl MainWindow {
    /// Out of sight; the first time, a notice by the clock says where the program went.
    fn hide(&mut self, ui: &MainWindowUi) {
        ui.hide();
        if !self.told {
            self.told = true;
            ui.tray.show_balloon_tip(t("My tray program"), t("Still running here. Right-click the icon for its menu."));
        }
    }

    fn show_startup_state(&self, ui: &MainWindowUi) {
        let words = if startup::is_on() { t("Starts with Windows when you sign in.") } else { t("Does not start with Windows.") };
        ui.startup_state.set_text(words);
    }
}

/// "Start with Windows": Windows starts each program named under its Run key in the registry
/// when one signs in. reg.exe, which comes with Windows, reads and writes the key.
mod startup {
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};

    const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

    /// The program's name in the key (its file name, without .exe) and the full path to it.
    fn program() -> Option<(String, String)> {
        let path = std::env::current_exe().ok()?;
        let name = path.file_stem()?.to_string_lossy().into_owned();
        Some((name, path.to_string_lossy().into_owned()))
    }

    /// reg.exe with `args`, no black window; true when it did it.
    fn reg(args: &[&str]) -> bool {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let run = Command::new("reg").args(args).stdout(Stdio::null()).stderr(Stdio::null()).creation_flags(CREATE_NO_WINDOW).status();
        run.map(|s| s.success()).unwrap_or(false)
    }

    /// Whether Windows starts this program when one signs in.
    pub fn is_on() -> bool {
        program().is_some_and(|(name, _)| reg(&["query", RUN_KEY, "/v", &name]))
    }

    /// Starts with Windows (`on`) or no longer does; true when the registry took it.
    pub fn set(on: bool) -> bool {
        let Some((name, path)) = program() else { return false };
        if on {
            // In quotes, for a path with spaces in it.
            reg(&["add", RUN_KEY, "/v", &name, "/t", "REG_SZ", "/d", &format!("\"{path}\""), "/f"])
        } else {
            reg(&["delete", RUN_KEY, "/v", &name, "/f"])
        }
    }
}
