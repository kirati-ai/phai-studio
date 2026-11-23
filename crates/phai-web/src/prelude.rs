//! The same names as phai-ui's prelude, so a form's code that says `use phai_ui::prelude::*;`
//! builds for the web as it does for Windows. (Printing is not on the web: no PrintPageEventArgs.)

pub use crate::events::{ClosingEventArgs, FileDropEventArgs, FileOkEventArgs, ItemEventArgs, KeyEventArgs, KeyPressEventArgs, MouseButton, MouseEventArgs};
pub use crate::form::{message_box, DialogResult, MessageBoxButtons, MessageBoxIcon};
pub use crate::paint::PaintEventArgs;
pub use crate::app::{app_folder, clipboard_text, exit_app, now, open_with_windows, set_clipboard_text};
pub use crate::convert::{to_float, to_int, DateTime};
pub use crate::text::{t, tf};
