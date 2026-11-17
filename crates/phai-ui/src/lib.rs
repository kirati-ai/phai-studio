//! **phai-ui** - Win32 windows the Windows 11 way, for Phai Studio and the programs it makes.
//!
//! Real Windows controls (small, no runtime, Thai shaped by Windows itself), themed with CSS,
//! placed by dock and anchor, worded in Thai and English. Copied - not moved - from PhaiDesk's
//! console, which keeps its own.
//!
//! | module | what it is |
//! | --- | --- |
//! | [`win`] | windows, fonts, DPI, buttons, checkboxes, radios, edits, lists, toolbar, the frame |
//! | [`controls`] | group frame, tabs, status bar - drawn whole by the library |
//! | [`theme`] | themes as CSS; the built-in light and dark ones |
//! | [`layout`] | dock and anchor |
//! | [`text`] | `t()` / `tf()`: Thai and English, English as the key |
//! | [`icon`] | icons from pixels and from Windows' icon font |
//! | [`dialogs`] | ask for a line of text; "waiting" with Cancel |
//! | [`shot`] | photograph a window to PNG |
//!
//! A program needs Common Controls 6 in its manifest, or Windows hands it the old controls:
//! see `apps/gallery/build.rs`.

pub mod controls;
pub mod dialogs;
pub mod grid;
pub mod icon;
pub mod layout;
pub mod picture;
#[cfg(feature = "shot")]
pub mod shot;
pub mod text;
pub mod theme;
pub mod win;

pub use text::{t, tf};
