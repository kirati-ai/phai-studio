//! What a form's code uses most, in one line: `use phai_ui::prelude::*;` - then
//! `message_box(ui.hwnd, "Saved", "Shop", MessageBoxButtons::Ok, MessageBoxIcon::Information)`,
//! `t` and `tf` for the Thai table, `e: &KeyEventArgs`, with nothing more to type. Phai Studio puts
//! the line in every form's code file; phai-web has the same names, so the file builds for the
//! web too.

pub use crate::events::{ClosingEventArgs, FileDropEventArgs, FileOkEventArgs, ItemEventArgs, KeyEventArgs, KeyPressEventArgs, MouseButton, MouseEventArgs};
pub use crate::form::{message_box, DialogResult, MessageBoxButtons, MessageBoxIcon};
pub use crate::paint::PaintEventArgs;
pub use crate::print::PrintPageEventArgs;
pub use crate::text::{t, tf};
