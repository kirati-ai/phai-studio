//! What an event's method is given - the same types as on Windows, so a program's methods have
//! one signature for both.

use std::cell::Cell;

use crate::win::HWND;

/// A key pressed or let go - WinForms' KeyEventArgs. `key` is Windows' virtual key (`0x0D` Enter).
#[derive(Debug, Default)]
pub struct KeyEventArgs {
    pub key: u16,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub(crate) handled: Cell<bool>,
}

impl KeyEventArgs {
    pub fn set_handled(&self) {
        self.handled.set(true);
    }
    pub fn handled(&self) -> bool {
        self.handled.get()
    }
}

/// A letter typed - WinForms' KeyPressEventArgs.
#[derive(Debug, Default)]
pub struct KeyPressEventArgs {
    pub ch: char,
    pub(crate) handled: Cell<bool>,
}

impl KeyPressEventArgs {
    pub fn set_handled(&self) {
        self.handled.set(true);
    }
    pub fn handled(&self) -> bool {
        self.handled.get()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MouseButton {
    #[default]
    None,
    Left,
    Right,
    Middle,
}

/// The mouse or a finger - WinForms' MouseEventArgs, from the control's top left.
#[derive(Debug, Default)]
pub struct MouseEventArgs {
    pub x: i32,
    pub y: i32,
    pub button: MouseButton,
    pub ctrl: bool,
    pub shift: bool,
}

/// The form about to close - WinForms' FormClosingEventArgs.
#[derive(Debug, Default)]
pub struct ClosingEventArgs {
    cancel: Cell<bool>,
}

impl ClosingEventArgs {
    pub fn cancel(&self) {
        self.cancel.set(true);
    }
    pub fn cancelled(&self) -> bool {
        self.cancel.get()
    }
}

/// A file dialog's Open or Save clicked - WinForms' FileOk. In a browser there is none to raise
/// it; the type is here so a program's code is the same for both.
#[derive(Debug, Default)]
pub struct FileOkEventArgs {
    pub files: Vec<std::path::PathBuf>,
    cancel: Cell<bool>,
}

impl FileOkEventArgs {
    pub fn file(&self) -> Option<&std::path::Path> {
        self.files.first().map(|p| p.as_path())
    }
    pub fn cancel(&self) {
        self.cancel.set(true);
    }
    pub fn cancelled(&self) -> bool {
        self.cancel.get()
    }
}

/// What was dropped. In a browser: words dragged from another control.
#[derive(Debug, Default)]
pub struct FileDropEventArgs {
    pub files: Vec<std::path::PathBuf>,
    pub text: Option<String>,
    pub source: Option<HWND>,
}

/// An item of a list - a column clicked, a row chosen.
#[derive(Debug, Default)]
pub struct ItemEventArgs {
    pub index: usize,
}

/// Windows' virtual key for a browser's `KeyboardEvent.key`.
pub fn virtual_key(key: &str) -> u16 {
    match key {
        "Enter" => 0x0D,
        "Escape" => 0x1B,
        "Tab" => 0x09,
        "Backspace" => 0x08,
        "Delete" => 0x2E,
        " " => 0x20,
        "ArrowLeft" => 0x25,
        "ArrowUp" => 0x26,
        "ArrowRight" => 0x27,
        "ArrowDown" => 0x28,
        "Home" => 0x24,
        "End" => 0x23,
        "PageUp" => 0x21,
        "PageDown" => 0x22,
        k if k.len() == 1 => {
            let c = k.chars().next().unwrap_or(' ').to_ascii_uppercase();
            if c.is_ascii_alphanumeric() {
                c as u16
            } else {
                0
            }
        }
        k if k.starts_with('F') => k[1..].parse::<u16>().map(|n| 0x6F + n).unwrap_or(0),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn keys_are_windows_keys() {
        assert_eq!(super::virtual_key("Enter"), 0x0D);
        assert_eq!(super::virtual_key("a"), 0x41);
        assert_eq!(super::virtual_key("7"), 0x37);
        assert_eq!(super::virtual_key("F1"), 0x70);
        assert_eq!(super::virtual_key("F12"), 0x7B);
        assert_eq!(super::virtual_key("Shift"), 0);
    }
}
