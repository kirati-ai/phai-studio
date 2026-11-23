//! The program's everyday calls, as short as C#'s: the clipboard (`Clipboard.SetText`), a web
//! page or file opened as Windows opens it (`Process.Start`), the program's folder
//! (`Application.StartupPath`), the end of the program (`Application.Exit`), the time now
//! (`DateTime.Now`). All in the prelude.

use std::path::PathBuf;

use windows::core::PCWSTR;
use windows::Win32::UI::WindowsAndMessaging as wm;

pub use crate::convert::DateTime;

/// The words on the clipboard (what Ctrl+C copied); empty when it holds none - as C#'s
/// `Clipboard.GetText()`.
///
/// Example: `ui.notes.set_text(&clipboard_text());`
pub fn clipboard_text() -> String {
    crate::grid::clipboard_words().unwrap_or_default()
}

/// Puts words on the clipboard, for Ctrl+V anywhere - as C#'s `Clipboard.SetText(text)`.
///
/// Example: `set_clipboard_text(&ui.result.text());`
pub fn set_clipboard_text(text: &str) {
    crate::grid::put_on_clipboard(text);
}

/// Opens a web address, a file or a folder as a double-click in Windows would - the browser,
/// the PDF reader, Explorer - as C#'s `Process.Start(target)`.
///
/// Example: `open_with_windows("https://phai-studio.monrak.net");`
pub fn open_with_windows(target: &str) {
    let w: Vec<u16> = target.encode_utf16().chain(Some(0)).collect();
    unsafe {
        windows::Win32::UI::Shell::ShellExecuteW(None, windows::core::w!("open"), PCWSTR(w.as_ptr()), PCWSTR::null(), PCWSTR::null(), wm::SW_SHOWNORMAL);
    }
}

/// The folder the program is in - for files that go with it; as C#'s `Application.StartupPath`.
///
/// Example: `let db = app_folder().join("data.db");`
pub fn app_folder() -> PathBuf {
    std::env::current_exe().ok().and_then(|p| p.parent().map(PathBuf::from)).unwrap_or_else(|| PathBuf::from("."))
}

/// Ends the program, once what it is doing now is done - as C#'s `Application.Exit()`.
///
/// Example: `if ui.confirm("Quit?") { exit_app(); }`
pub fn exit_app() {
    unsafe {
        wm::PostQuitMessage(0);
    }
}

/// The date and time now, on this computer - as C#'s `DateTime.Now`. `.format(..)` writes it
/// as you like; `bbbb` is the Buddhist year.
///
/// Example: `ui.clock.set_text(&now().format("dd/MM/bbbb HH:mm"));`
pub fn now() -> DateTime {
    let t = unsafe { windows::Win32::System::SystemInformation::GetLocalTime() };
    DateTime { year: t.wYear as i32, month: t.wMonth as u32, day: t.wDay as u32, hour: t.wHour as u32, minute: t.wMinute as u32, second: t.wSecond as u32, millisecond: t.wMilliseconds as u32 }
}
