//! **phai-ui** - Win32 windows the Windows 11 way, for Phai Studio and the programs it makes.
//!
//! Real Windows controls (small, no runtime, Thai shaped by Windows itself), themed with CSS,
//! placed by dock and anchor, worded in Thai and English. Copied - not moved - from PhaiDesk's
//! console, which keeps its own.
//!
//! | module | what it is |
//! | --- | --- |
//! | [`win`] | windows, fonts, DPI, buttons, checkboxes, radios, edits, lists, toolbar, the frame |
//! | [`components`] | file, folder, colour and font dialogs; ErrorProvider; NotifyIcon |
//! | [`split`] | SplitContainer: two panes and the bar between them |
//! | [`mdi`] | MDI: forms inside a form, cascaded or tiled |
//! | [`drag`] | words dragged from one control to another (DoDragDrop) |
//! | [`extra`] | ListBox, CheckedListBox, LinkLabel, MaskedTextBox, MonthCalendar |
//! | [`popup`] | context menus built as they open: items hidden, ticked, greyed by the program, menus within |
//! | [`controls`] | group frame, tabs, status bar - drawn whole by the library |
//! | [`theme`] | themes as CSS; the built-in light and dark ones |
//! | [`layout`] | dock and anchor |
//! | [`text`] | `t()` / `tf()`: Thai and English, English as the key |
//! | [`icon`] | icons from pixels and from Windows' icon font |
//! | [`print`] | PrintDocument, its dialogs and print preview |
//! | [`report`] | banded reports: headers, groups, totals - previewed, printed, saved as PDF |
//! | [`dialogs`] | ask for a line of text; "waiting" with Cancel |
//! | [`inspect`] | the Live Inspector: F12 in a debug build, a control's look and properties changed live |
//! | [`shot`] | photograph a window to PNG |
//!
//! A program needs Common Controls 6 in its manifest, or Windows hands it the old controls:
//! see `apps/gallery/build.rs`.

pub mod app;
pub mod binding;
pub mod browser;
pub mod components;
pub mod controls;
pub mod convert;
pub mod ctl;
pub mod dialogs;
pub mod drag;
pub mod events;
pub mod extra;
pub mod form;
pub mod grid;
pub mod gridfilter;
pub mod icon;
pub mod inspect;
pub mod invoke;
pub mod layout;
pub mod mdi;
pub mod picture;
pub mod paint;
pub mod popup;
pub mod prelude;
pub mod print;
pub mod props;
pub mod report;
#[cfg(feature = "shot")]
pub mod shot;
pub mod split;
pub mod text;
pub mod theme;
pub mod theme_edit;
pub mod win;
pub mod xlsx;

pub use text::{t, tf};
