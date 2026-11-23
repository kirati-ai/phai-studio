//! What a form's code uses most, in one line: `use phai_ui::prelude::*;` - then
//! `message_box(ui.hwnd, "Saved", "Shop", MessageBoxButtons::Ok, MessageBoxIcon::Information)`,
//! `t` and `tf` for the Thai table, `e: &KeyEventArgs`, `to_int(&text)`, `now()`,
//! `set_clipboard_text(..)`, `open_with_windows(url)`, with nothing more to type. Phai Studio puts
//! the line in every form's code file; phai-web has the same names, so the file builds for the
//! web too.

pub use crate::events::{ClosingEventArgs, FileDropEventArgs, FileOkEventArgs, ItemEventArgs, KeyEventArgs, KeyPressEventArgs, MouseButton, MouseEventArgs};
pub use crate::form::{message_box, DialogResult, MessageBoxButtons, MessageBoxIcon};
pub use crate::paint::PaintEventArgs;
pub use crate::print::PrintPageEventArgs;
pub use crate::app::{app_folder, clipboard_text, exit_app, now, open_with_windows, set_clipboard_text};
pub use crate::convert::{to_float, to_int, DateTime};
pub use crate::text::{t, tf};
