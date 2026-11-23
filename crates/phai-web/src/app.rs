//! phai-ui's everyday calls on a web page, so a program that uses them builds for the web: the
//! clipboard (writing only - a page reads it only when the person allows, later), a link opened
//! in a new tab, the time now from the browser's clock.

use std::path::PathBuf;

pub use crate::convert::DateTime;

/// A page cannot read the clipboard at once: empty.
pub fn clipboard_text() -> String {
    String::new()
}

/// Words put on the clipboard - the browser asks the person the first time.
pub fn set_clipboard_text(text: &str) {
    if let Some(w) = web_sys::window() {
        let _ = w.navigator().clipboard().write_text(text);
    }
}

/// A web address opened in a new tab; a file has no meaning on a page.
pub fn open_with_windows(target: &str) {
    if let Some(w) = web_sys::window() {
        let _ = w.open_with_url_and_target(target, "_blank");
    }
}

/// A page has no folder of its own: here.
pub fn app_folder() -> PathBuf {
    PathBuf::from(".")
}

/// A page ends when its tab closes: nothing to do.
pub fn exit_app() {}

/// The local date and time now, by the browser's clock.
pub fn now() -> DateTime {
    let d = js_sys::Date::new_0();
    DateTime { year: d.get_full_year() as i32, month: d.get_month() + 1, day: d.get_date(), hour: d.get_hours(), minute: d.get_minutes(), second: d.get_seconds(), millisecond: d.get_milliseconds() }
}
