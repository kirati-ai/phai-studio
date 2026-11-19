//! Controls as objects, as WinForms has them: a `TextBox` with `text()`, `clear()`, `append_text()`,
//! a `ComboBox` with `items_add()` and `selected_index()`, a `ListView`, a `Grid`, a `Timer` with
//! `start()` and `stop()`. Each is its window's handle and nothing more - `Copy`, as cheap as the
//! handle - so `ui.name.text()` reads the box, and `ui.name.hwnd()` (or `*ui.name`) is the handle
//! for whatever Win32 or phai-ui call wants one, as WinForms' `Handle`.
//!
//! Every control has the methods all of WinForms' controls have - `text`, `set_text`, `enabled`,
//! `set_enabled`, `visible`, `show`, `hide`, `focus`, `bring_to_front`, `refresh`, `bounds`... -
//! and those of its kind.

use std::collections::HashMap;
use std::sync::Mutex;

use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::win;

fn send(h: HWND, msg: u32, w: usize, l: isize) -> isize {
    unsafe { wm::SendMessageW(h, msg, Some(WPARAM(w)), Some(LPARAM(l))).0 }
}

/// What every control has: the handle, its words, whether it can be used and seen, the focus,
/// its place. Each control type has these as its own methods too, so no `use` is needed.
pub trait Control: Copy {
    fn hwnd(self) -> HWND;

    fn text(self) -> String {
        win::text_of(self.hwnd())
    }
    fn set_text(self, s: &str) {
        win::set_text(self.hwnd(), s)
    }
    fn enabled(self) -> bool {
        unsafe { km::IsWindowEnabled(self.hwnd()) }.as_bool()
    }
    fn set_enabled(self, on: bool) {
        win::enable(self.hwnd(), on)
    }
    fn visible(self) -> bool {
        unsafe { wm::IsWindowVisible(self.hwnd()) }.as_bool()
    }
    fn set_visible(self, on: bool) {
        win::show(self.hwnd(), on)
    }
    fn show(self) {
        win::show(self.hwnd(), true)
    }
    fn hide(self) {
        win::show(self.hwnd(), false)
    }
    /// The keys go to it now.
    fn focus(self) {
        unsafe {
            let _ = km::SetFocus(Some(self.hwnd()));
        }
    }
    fn focused(self) -> bool {
        unsafe { km::GetFocus() == self.hwnd() }
    }
    /// Over the controls it overlaps.
    fn bring_to_front(self) {
        unsafe {
            let _ = wm::SetWindowPos(self.hwnd(), Some(wm::HWND_TOP), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOACTIVATE);
        }
    }
    fn send_to_back(self) {
        unsafe {
            let _ = wm::SetWindowPos(self.hwnd(), Some(wm::HWND_BOTTOM), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOACTIVATE);
        }
    }
    /// Drawn again now, with what is in it.
    fn refresh(self) {
        unsafe {
            let _ = Gdi::RedrawWindow(Some(self.hwnd()), None, None, Gdi::RDW_INVALIDATE | Gdi::RDW_ERASE | Gdi::RDW_ALLCHILDREN | Gdi::RDW_UPDATENOW | Gdi::RDW_FRAME);
        }
    }
    /// Drawn again when Windows next paints.
    fn invalidate(self) {
        unsafe {
            let _ = Gdi::InvalidateRect(Some(self.hwnd()), None, true);
        }
    }
    /// Where it is in its window, in pixels: `left, top, right, bottom`.
    fn bounds(self) -> RECT {
        win::rect_in_parent(self.hwnd())
    }
    /// Moved and sized, pixels. The window's layout puts it back where the form says when the
    /// window is sized; change the form for good.
    fn set_bounds(self, x: i32, y: i32, w: i32, h: i32) {
        win::move_to(self.hwnd(), x, y, w, h)
    }
    /// A point of it on the screen.
    fn point_to_screen(self, x: i32, y: i32) -> (i32, i32) {
        let mut p = POINT { x, y };
        unsafe {
            let _ = Gdi::ClientToScreen(self.hwnd(), &mut p);
        }
        (p.x, p.y)
    }
    /// A point of the screen in it.
    fn point_to_client(self, x: i32, y: i32) -> (i32, i32) {
        let mut p = POINT { x, y };
        unsafe {
            let _ = Gdi::ScreenToClient(self.hwnd(), &mut p);
        }
        (p.x, p.y)
    }
    /// The window it is in.
    fn find_form(self) -> HWND {
        unsafe { wm::GetAncestor(self.hwnd(), wm::GA_ROOT) }
    }
    /// Stops drawing while much changes - then [`Control::resume_layout`].
    fn suspend_layout(self) {
        send(self.hwnd(), wm::WM_SETREDRAW, 0, 0);
    }
    fn resume_layout(self) {
        send(self.hwnd(), wm::WM_SETREDRAW, 1, 0);
        self.refresh();
    }
    fn set_tooltip(self, text: &str) {
        crate::props::tooltip(self.hwnd(), text)
    }
    fn set_cursor(self, name: &str) {
        crate::props::cursor(self.hwnd(), name)
    }
}

/// A control type: the handle, `Copy`, `*x` or `x.hwnd()` the handle, and every [`Control`]
/// method of its own.
macro_rules! control {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
        pub struct $name(pub HWND);

        // A window's handle is the same on every thread; Windows passes a message sent to it
        // from another thread to the window's own. For the program's own state, see `invoke`.
        unsafe impl Send for $name {}
        unsafe impl Sync for $name {}

        impl std::ops::Deref for $name {
            type Target = HWND;
            fn deref(&self) -> &HWND {
                &self.0
            }
        }
        impl From<$name> for HWND {
            fn from(c: $name) -> HWND {
                c.0
            }
        }
        impl Control for $name {
            fn hwnd(self) -> HWND {
                self.0
            }
        }
        #[allow(dead_code)]
        impl $name {
            /// The window's handle, as WinForms' `Handle`.
            pub fn hwnd(self) -> HWND {
                self.0
            }
            pub fn text(self) -> String {
                Control::text(self)
            }
            pub fn set_text(self, s: &str) {
                Control::set_text(self, s)
            }
            pub fn enabled(self) -> bool {
                Control::enabled(self)
            }
            pub fn set_enabled(self, on: bool) {
                Control::set_enabled(self, on)
            }
            pub fn visible(self) -> bool {
                Control::visible(self)
            }
            pub fn set_visible(self, on: bool) {
                Control::set_visible(self, on)
            }
            pub fn show(self) {
                Control::show(self)
            }
            pub fn hide(self) {
                Control::hide(self)
            }
            pub fn focus(self) {
                Control::focus(self)
            }
            pub fn focused(self) -> bool {
                Control::focused(self)
            }
            pub fn bring_to_front(self) {
                Control::bring_to_front(self)
            }
            pub fn send_to_back(self) {
                Control::send_to_back(self)
            }
            pub fn refresh(self) {
                Control::refresh(self)
            }
            pub fn invalidate(self) {
                Control::invalidate(self)
            }
            pub fn bounds(self) -> RECT {
                Control::bounds(self)
            }
            pub fn set_bounds(self, x: i32, y: i32, w: i32, h: i32) {
                Control::set_bounds(self, x, y, w, h)
            }
            pub fn point_to_screen(self, x: i32, y: i32) -> (i32, i32) {
                Control::point_to_screen(self, x, y)
            }
            pub fn point_to_client(self, x: i32, y: i32) -> (i32, i32) {
                Control::point_to_client(self, x, y)
            }
            pub fn find_form(self) -> HWND {
                Control::find_form(self)
            }
            pub fn suspend_layout(self) {
                Control::suspend_layout(self)
            }
            pub fn resume_layout(self) {
                Control::resume_layout(self)
            }
            pub fn set_tooltip(self, text: &str) {
                Control::set_tooltip(self, text)
            }
            pub fn set_cursor(self, name: &str) {
                Control::set_cursor(self, name)
            }
        }
    };
}

control!(
    /// WinForms' Label.
    Label
);
control!(
    /// WinForms' Button.
    Button
);
control!(
    /// WinForms' CheckBox.
    CheckBox
);
control!(
    /// WinForms' RadioButton.
    RadioButton
);
control!(
    /// WinForms' TextBox - one line (`edit`) or several (`textarea`).
    TextBox
);
control!(
    /// WinForms' RichTextBox.
    RichTextBox
);
control!(
    /// WinForms' ComboBox.
    ComboBox
);
control!(
    /// WinForms' ListView, in Details.
    ListView
);
control!(
    /// WinForms' DataGridView: cells changed in place, rows added at the bottom.
    Grid
);
control!(
    /// WinForms' TreeView.
    TreeView
);
control!(
    /// WinForms' DateTimePicker.
    DateTimePicker
);
control!(
    /// WinForms' NumericUpDown.
    NumericUpDown
);
control!(
    /// WinForms' TrackBar.
    TrackBar
);
control!(
    /// WinForms' ProgressBar.
    ProgressBar
);
control!(
    /// WinForms' PictureBox.
    PictureBox
);
control!(
    /// WinForms' TabControl.
    TabControl
);
control!(
    /// WinForms' GroupBox.
    GroupBox
);
control!(
    /// WinForms' FlowLayoutPanel.
    FlowPanel
);
control!(
    /// WinForms' TableLayoutPanel.
    TablePanel
);
control!(
    /// WinForms' StatusStrip.
    StatusBar
);
control!(
    /// WinForms' ContextMenuStrip: the window it belongs to; it opens on a right-click.
    ContextMenu
);

impl Button {
    /// Clicks it, as the mouse would - its Click method runs.
    pub fn perform_click(self) {
        crate::form::click(self.0)
    }
}

/// A check box's three states - WinForms' CheckState.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckState {
    Unchecked,
    Checked,
    Indeterminate,
}

impl CheckBox {
    pub fn checked(self) -> bool {
        win::is_checked(self.0)
    }
    pub fn set_checked(self, on: bool) {
        win::set_checked(self.0, on)
    }
    pub fn check_state(self) -> CheckState {
        match win::check_state(self.0) {
            0 => CheckState::Unchecked,
            1 => CheckState::Checked,
            _ => CheckState::Indeterminate,
        }
    }
    pub fn set_check_state(self, s: CheckState) {
        win::set_check_state(self.0, s as u8)
    }
}

impl RadioButton {
    pub fn checked(self) -> bool {
        win::is_checked(self.0)
    }
    /// Chosen - the others of its group no longer.
    pub fn set_checked(self, on: bool) {
        win::set_checked(self.0, on)
    }
}

/// What a text box and a rich text box both do.
pub trait TextEdit: Control {
    /// Empties it.
    fn clear(self) {
        win::set_text(self.hwnd(), "")
    }
    /// Words added at the end, the caret after them, scrolled to.
    fn append_text(self, s: &str) {
        let h = self.hwnd();
        let end = win::text_of(h).encode_utf16().count();
        send(h, 0x00B1 /* EM_SETSEL */, end, end as isize);
        let w = win::wide(s);
        send(h, 0x00C2 /* EM_REPLACESEL */, 0, w.as_ptr() as isize);
        send(h, 0x00B7 /* EM_SCROLLCARET */, 0, 0);
    }
    fn select_all(self) {
        send(self.hwnd(), 0x00B1, 0, -1);
    }
    /// Chooses `length` letters from `start` (UTF-16, as Windows counts).
    fn select(self, start: usize, length: usize) {
        send(self.hwnd(), 0x00B1, start, (start + length) as isize);
    }
    fn selection_start(self) -> usize {
        let mut a = 0u32;
        let mut b = 0u32;
        send(self.hwnd(), 0x00B0 /* EM_GETSEL */, &mut a as *mut _ as usize, &mut b as *mut _ as isize);
        a as usize
    }
    fn selection_length(self) -> usize {
        let mut a = 0u32;
        let mut b = 0u32;
        send(self.hwnd(), 0x00B0, &mut a as *mut _ as usize, &mut b as *mut _ as isize);
        (b - a.min(b)) as usize
    }
    /// The words chosen.
    fn selected_text(self) -> String {
        let all: Vec<u16> = win::text_of(self.hwnd()).encode_utf16().collect();
        let (a, n) = (self.selection_start().min(all.len()), self.selection_length());
        String::from_utf16_lossy(&all[a..(a + n).min(all.len())])
    }
    /// The chosen words replaced by `s` - or `s` put at the caret.
    fn paste_text(self, s: &str) {
        let w = win::wide(s);
        send(self.hwnd(), 0x00C2 /* EM_REPLACESEL */, 1, w.as_ptr() as isize);
    }
    fn copy(self) {
        send(self.hwnd(), wm::WM_COPY, 0, 0);
    }
    fn cut(self) {
        send(self.hwnd(), wm::WM_CUT, 0, 0);
    }
    fn paste(self) {
        send(self.hwnd(), wm::WM_PASTE, 0, 0);
    }
    fn undo(self) {
        send(self.hwnd(), 0x00C7 /* EM_UNDO */, 0, 0);
    }
    fn can_undo(self) -> bool {
        send(self.hwnd(), 0x00C6 /* EM_CANUNDO */, 0, 0) != 0
    }
    /// Scrolled so the caret shows.
    fn scroll_to_caret(self) {
        send(self.hwnd(), 0x00B7 /* EM_SCROLLCARET */, 0, 0);
    }
    fn read_only(self) -> bool {
        unsafe { wm::GetWindowLongW(self.hwnd(), wm::GWL_STYLE) as u32 & 0x0800 != 0 }
    }
    fn set_read_only(self, on: bool) {
        crate::props::read_only(self.hwnd(), on)
    }
    fn set_max_length(self, n: i32) {
        crate::props::max_length(self.hwnd(), n)
    }
    /// Its lines.
    fn lines(self) -> Vec<String> {
        win::text_of(self.hwnd()).lines().map(str::to_string).collect()
    }
}

impl TextEdit for TextBox {}
impl TextEdit for RichTextBox {}

macro_rules! text_edit_methods {
    ($name:ident) => {
        #[allow(dead_code)]
        impl $name {
            pub fn clear(self) {
                TextEdit::clear(self)
            }
            pub fn append_text(self, s: &str) {
                TextEdit::append_text(self, s)
            }
            pub fn select_all(self) {
                TextEdit::select_all(self)
            }
            pub fn select(self, start: usize, length: usize) {
                TextEdit::select(self, start, length)
            }
            pub fn selection_start(self) -> usize {
                TextEdit::selection_start(self)
            }
            pub fn selection_length(self) -> usize {
                TextEdit::selection_length(self)
            }
            pub fn selected_text(self) -> String {
                TextEdit::selected_text(self)
            }
            pub fn paste_text(self, s: &str) {
                TextEdit::paste_text(self, s)
            }
            pub fn copy(self) {
                TextEdit::copy(self)
            }
            pub fn cut(self) {
                TextEdit::cut(self)
            }
            pub fn paste(self) {
                TextEdit::paste(self)
            }
            pub fn undo(self) {
                TextEdit::undo(self)
            }
            pub fn can_undo(self) -> bool {
                TextEdit::can_undo(self)
            }
            pub fn scroll_to_caret(self) {
                TextEdit::scroll_to_caret(self)
            }
            pub fn read_only(self) -> bool {
                TextEdit::read_only(self)
            }
            pub fn set_read_only(self, on: bool) {
                TextEdit::set_read_only(self, on)
            }
            pub fn set_max_length(self, n: i32) {
                TextEdit::set_max_length(self, n)
            }
            pub fn lines(self) -> Vec<String> {
                TextEdit::lines(self)
            }
        }
    };
}
text_edit_methods!(TextBox);
text_edit_methods!(RichTextBox);

impl RichTextBox {
    /// Reads a text file into it (UTF-8).
    pub fn load_file(self, path: &std::path::Path) -> std::io::Result<()> {
        let s = std::fs::read_to_string(path)?;
        win::set_text(self.0, &s);
        Ok(())
    }
    /// Writes what it holds to a text file (UTF-8).
    pub fn save_file(self, path: &std::path::Path) -> std::io::Result<()> {
        std::fs::write(path, win::text_of(self.0))
    }
    /// Where `what` first is from `from` on, and chooses it - as WinForms' Find. UTF-16 places.
    pub fn find(self, what: &str, from: usize) -> Option<usize> {
        let all: Vec<u16> = win::text_of(self.0).replace("\r\n", "\r").encode_utf16().collect();
        let needle: Vec<u16> = what.encode_utf16().collect();
        if needle.is_empty() {
            return None;
        }
        let at = (from..all.len().saturating_sub(needle.len() - 1)).find(|i| all[*i..].starts_with(&needle))?;
        TextEdit::select(self, at, needle.len());
        Some(at)
    }
}

impl ComboBox {
    /// Adds an item at the end; its place.
    pub fn items_add(self, s: &str) -> usize {
        let w = win::wide(s);
        send(self.0, 0x0143 /* CB_ADDSTRING */, 0, w.as_ptr() as isize).max(0) as usize
    }
    pub fn items_insert(self, at: usize, s: &str) {
        let w = win::wide(s);
        send(self.0, 0x014A /* CB_INSERTSTRING */, at, w.as_ptr() as isize);
    }
    pub fn items_remove_at(self, at: usize) {
        send(self.0, 0x0144 /* CB_DELETESTRING */, at, 0);
    }
    pub fn items_clear(self) {
        send(self.0, 0x014B /* CB_RESETCONTENT */, 0, 0);
    }
    pub fn items_count(self) -> usize {
        send(self.0, 0x0146 /* CB_GETCOUNT */, 0, 0).max(0) as usize
    }
    pub fn item(self, at: usize) -> String {
        let n = send(self.0, 0x0149 /* CB_GETLBTEXTLEN */, at, 0);
        if n < 0 {
            return String::new();
        }
        let mut buf = vec![0u16; n as usize + 1];
        send(self.0, 0x0148 /* CB_GETLBTEXT */, at, buf.as_mut_ptr() as isize);
        String::from_utf16_lossy(&buf[..n as usize])
    }
    pub fn items(self) -> Vec<String> {
        (0..self.items_count()).map(|i| self.item(i)).collect()
    }
    /// The item chosen, from 0; `None` when none is.
    pub fn selected_index(self) -> Option<usize> {
        let i = win::combo_selected(self.0);
        (i >= 0).then_some(i as usize)
    }
    pub fn set_selected_index(self, i: Option<usize>) {
        send(self.0, 0x014E /* CB_SETCURSEL */, i.unwrap_or(usize::MAX), 0);
    }
    /// The chosen item's words.
    pub fn selected_item(self) -> Option<String> {
        self.selected_index().map(|i| self.item(i))
    }
    /// The first item starting with `prefix`, any case.
    pub fn find_string(self, prefix: &str) -> Option<usize> {
        let w = win::wide(prefix);
        let i = send(self.0, 0x014C /* CB_FINDSTRING */, usize::MAX, w.as_ptr() as isize);
        (i >= 0).then_some(i as usize)
    }
    /// The item that is exactly `s`, any case.
    pub fn find_string_exact(self, s: &str) -> Option<usize> {
        let w = win::wide(s);
        let i = send(self.0, 0x0158 /* CB_FINDSTRINGEXACT */, usize::MAX, w.as_ptr() as isize);
        (i >= 0).then_some(i as usize)
    }
    /// Opens or closes its list.
    pub fn set_dropped_down(self, open: bool) {
        send(self.0, 0x014F /* CB_SHOWDROPDOWN */, open as usize, 0);
    }
    /// Stops drawing while many items go in - then [`ComboBox::end_update`].
    pub fn begin_update(self) {
        Control::suspend_layout(self)
    }
    pub fn end_update(self) {
        Control::resume_layout(self)
    }
}

impl ListView {
    /// Adds a row - its first cell and the rest; its place.
    pub fn items_add(self, cells: &[&str]) -> usize {
        win::list_add(self.0, cells);
        self.items_count().saturating_sub(1)
    }
    pub fn items_remove_at(self, at: usize) {
        send(self.0, cc::LVM_DELETEITEM, at, 0);
    }
    pub fn items_clear(self) {
        send(self.0, cc::LVM_DELETEALLITEMS, 0, 0);
    }
    pub fn items_count(self) -> usize {
        send(self.0, cc::LVM_GETITEMCOUNT, 0, 0).max(0) as usize
    }
    pub fn cell(self, row: usize, col: usize) -> String {
        win::list_text(self.0, row, col as i32)
    }
    pub fn set_cell(self, row: usize, col: usize, text: &str) {
        let mut w = win::wide(text);
        let it = cc::LVITEMW { iSubItem: col as i32, pszText: windows::core::PWSTR(w.as_mut_ptr()), ..Default::default() };
        send(self.0, cc::LVM_SETITEMTEXTW, row, &it as *const _ as isize);
    }
    /// The row chosen (the first, when several are).
    pub fn selected_index(self) -> Option<usize> {
        let i = send(self.0, cc::LVM_GETNEXTITEM, usize::MAX, 2 /* LVNI_SELECTED */);
        (i >= 0).then_some(i as usize)
    }
    pub fn selected_indices(self) -> Vec<usize> {
        let mut out = Vec::new();
        let mut i = -1isize;
        loop {
            i = send(self.0, cc::LVM_GETNEXTITEM, i as usize, 2);
            if i < 0 {
                return out;
            }
            out.push(i as usize);
        }
    }
    /// Chooses row `at` alone, and scrolls to it.
    pub fn select(self, at: usize) {
        win::list_select(self.0, at)
    }
    pub fn ensure_visible(self, at: usize) {
        send(self.0, cc::LVM_ENSUREVISIBLE, at, 0);
    }
    /// A row's tick, with CheckBoxes on.
    pub fn checked(self, at: usize) -> bool {
        (send(self.0, cc::LVM_GETITEMSTATE, at, 0xF000 /* LVIS_STATEIMAGEMASK */) >> 12) == 2
    }
    pub fn set_checked(self, at: usize, on: bool) {
        let it = cc::LVITEMW { stateMask: cc::LIST_VIEW_ITEM_STATE_FLAGS(0xF000), state: cc::LIST_VIEW_ITEM_STATE_FLAGS(if on { 2 } else { 1 } << 12), ..Default::default() };
        send(self.0, cc::LVM_SETITEMSTATE, at, &it as *const _ as isize);
    }
    pub fn begin_update(self) {
        Control::suspend_layout(self)
    }
    pub fn end_update(self) {
        Control::resume_layout(self)
    }
}

impl Grid {
    /// Every row's cells.
    pub fn rows(self) -> Vec<Vec<String>> {
        crate::grid::grid_rows(self.0)
    }
    pub fn rows_add(self, cells: &[&str]) {
        crate::grid::grid_add(self.0, cells)
    }
    pub fn rows_clear(self) {
        crate::grid::grid_clear(self.0)
    }
    pub fn rows_count(self) -> usize {
        send(self.0, cc::LVM_GETITEMCOUNT, 0, 0).max(0) as usize
    }
    /// Every row at once, in place of those it had.
    pub fn fill(self, rows: &[Vec<String>]) {
        crate::grid::grid_fill(self.0, rows)
    }
    pub fn cell(self, row: usize, col: usize) -> String {
        crate::grid::cell(self.0, row, col)
    }
    pub fn set_cell(self, row: usize, col: usize, text: &str) {
        crate::grid::set_cell(self.0, row, col, text)
    }
    pub fn selected_row(self) -> Option<usize> {
        let i = send(self.0, cc::LVM_GETNEXTITEM, usize::MAX, 2);
        (i >= 0).then_some(i as usize)
    }
    /// No row chosen.
    pub fn clear_selection(self) {
        let it = cc::LVITEMW { stateMask: cc::LVIS_SELECTED, state: cc::LIST_VIEW_ITEM_STATE_FLAGS(0), ..Default::default() };
        send(self.0, cc::LVM_SETITEMSTATE, usize::MAX, &it as *const _ as isize);
    }
    pub fn set_read_only(self, on: bool) {
        crate::grid::set_read_only(self.0, on)
    }
}

/// An item of a tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Node(pub isize);

impl TreeView {
    /// A top item; to add under it, [`TreeView::nodes_add_under`].
    pub fn nodes_add(self, text: &str) -> Node {
        Node(win::tree_add(self.0, 0, text, -1, 0))
    }
    pub fn nodes_add_under(self, parent: Node, text: &str) -> Node {
        Node(win::tree_add(self.0, parent.0, text, -1, 0))
    }
    pub fn nodes_clear(self) {
        win::tree_clear(self.0)
    }
    /// The lines as the form's Items write them: two spaces before an item per level down.
    pub fn set_nodes(self, lines: &[&str]) {
        win::tree_clear(self.0);
        win::tree_items(self.0, lines)
    }
    pub fn expand_all(self) {
        crate::props::expand_all(self.0, true)
    }
    pub fn collapse_all(self) {
        crate::props::expand_all(self.0, false)
    }
    /// The item chosen.
    pub fn selected_node(self) -> Option<Node> {
        let i = send(self.0, 0x110A /* TVM_GETNEXTITEM */, 9 /* TVGN_CARET */, 0);
        (i != 0).then_some(Node(i))
    }
    pub fn select(self, n: Node) {
        send(self.0, 0x110B /* TVM_SELECTITEM */, 9, n.0);
    }
    /// An item's words.
    pub fn node_text(self, n: Node) -> String {
        let mut buf = [0u16; 512];
        let mut it = cc::TVITEMW { mask: cc::TVIF_TEXT, hItem: cc::HTREEITEM(n.0), pszText: windows::core::PWSTR(buf.as_mut_ptr()), cchTextMax: buf.len() as i32, ..Default::default() };
        send(self.0, 0x113E /* TVM_GETITEMW */, 0, &mut it as *mut _ as isize);
        let len = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
        String::from_utf16_lossy(&buf[..len])
    }
    /// The chosen item's words.
    pub fn selected_text(self) -> Option<String> {
        self.selected_node().map(|n| self.node_text(n))
    }
    pub fn expand(self, n: Node) {
        send(self.0, 0x1102 /* TVM_EXPAND */, 2, n.0);
    }
    pub fn collapse(self, n: Node) {
        send(self.0, 0x1102, 1, n.0);
    }
}

impl DateTimePicker {
    /// The date shown, `(year, month, day)`.
    pub fn value(self) -> (i32, i32, i32) {
        win::date_value(self.0)
    }
    pub fn set_value(self, year: i32, month: i32, day: i32) {
        win::set_date(self.0, year, month, day)
    }
    /// With ShowCheckBox: whether a date is given (its box ticked).
    pub fn checked(self) -> bool {
        win::date_checked(self.0)
    }
}

impl NumericUpDown {
    pub fn value(self) -> i32 {
        win::value(self.0)
    }
    /// Kept within its range.
    pub fn set_value(self, v: i32) {
        let ud = unsafe { wm::GetWindow(self.0, wm::GW_CHILD).unwrap_or_default() };
        send(ud, 0x0471 /* UDM_SETPOS32 */, 0, v as isize);
    }
    fn step(self) -> i32 {
        #[repr(C)]
        #[derive(Default)]
        struct Accel {
            sec: u32,
            inc: u32,
        }
        let ud = unsafe { wm::GetWindow(self.0, wm::GW_CHILD).unwrap_or_default() };
        let mut a = Accel::default();
        send(ud, 0x046C /* UDM_GETACCEL */, 1, &mut a as *mut _ as isize);
        (a.inc as i32).max(1)
    }
    /// Up by its Increment.
    pub fn up_button(self) {
        self.set_value(self.value() + self.step())
    }
    pub fn down_button(self) {
        self.set_value(self.value() - self.step())
    }
}

impl TrackBar {
    pub fn value(self) -> i32 {
        win::value(self.0)
    }
    pub fn set_value(self, v: i32) {
        win::set_value(self.0, v)
    }
}

impl ProgressBar {
    pub fn value(self) -> i32 {
        win::value(self.0)
    }
    pub fn set_value(self, v: i32) {
        win::set_value(self.0, v)
    }
    /// Adds its Step.
    pub fn perform_step(self) {
        crate::props::perform_step(self.0)
    }
    /// Adds `n`.
    pub fn increment(self, n: i32) {
        win::set_value(self.0, win::value(self.0) + n)
    }
    pub fn set_marquee(self, on: bool) {
        crate::props::marquee(self.0, on)
    }
}

impl PictureBox {
    /// Shows another picture file.
    pub fn load(self, path: &str) {
        crate::picture::set_picture(self.0, path)
    }
    pub fn set_size_mode(self, mode: &str) {
        crate::picture::set_size_mode(self.0, mode)
    }
}

impl TabControl {
    pub fn selected_index(self) -> usize {
        crate::controls::tabs_selected(self.0)
    }
    /// Goes to page `i`, its controls shown, as a click on its tab would.
    pub fn select_tab(self, i: usize) {
        crate::controls::tabs_select(self.0, i);
        unsafe {
            if let Ok(parent) = wm::GetParent(self.0) {
                let id = wm::GetDlgCtrlID(self.0) as usize;
                wm::SendMessageW(parent, wm::WM_COMMAND, Some(WPARAM(id | (i << 16))), Some(LPARAM(self.0 .0 as isize)));
            }
        }
    }
}

impl StatusBar {
    /// The words of part `part`, from 0 at the left.
    pub fn set_part_text(self, part: usize, text: &str) {
        crate::controls::status_text(self.0, part, text)
    }
    /// The parts' widths, from the right.
    pub fn set_parts(self, widths: &[i32]) {
        crate::controls::status_parts(self.0, widths)
    }
}

/// Each timer's (window, id) to its interval and whether it runs.
type Intervals = HashMap<(isize, i32), (u32, bool)>;
static INTERVALS: Mutex<Option<Intervals>> = Mutex::new(None);

/// WinForms' Timer: every Interval milliseconds, its Tick method - while it runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timer {
    /// The window whose message loop it ticks in.
    pub hwnd: HWND,
    pub id: i32,
}

unsafe impl Send for Timer {}
unsafe impl Sync for Timer {}

impl Timer {
    /// A timer of window `hwnd`, every `ms`; started when `on`.
    pub fn new(hwnd: HWND, id: i32, ms: u32, on: bool) -> Timer {
        INTERVALS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).insert((hwnd.0 as isize, id), (ms.max(1), on));
        win::timer(hwnd, id, ms, on);
        Timer { hwnd, id }
    }
    fn get(self) -> (u32, bool) {
        INTERVALS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|m| m.get(&(self.hwnd.0 as isize, self.id)).copied()).unwrap_or((1000, false))
    }
    fn put(self, v: (u32, bool)) {
        INTERVALS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).insert((self.hwnd.0 as isize, self.id), v);
    }
    pub fn start(self) {
        let (ms, _) = self.get();
        self.put((ms, true));
        win::timer_start(self.hwnd, self.id, ms);
    }
    pub fn stop(self) {
        let (ms, _) = self.get();
        self.put((ms, false));
        win::timer_stop(self.hwnd, self.id);
    }
    pub fn enabled(self) -> bool {
        self.get().1
    }
    pub fn set_enabled(self, on: bool) {
        if on {
            self.start()
        } else {
            self.stop()
        }
    }
    pub fn interval(self) -> u32 {
        self.get().0
    }
    /// Milliseconds between ticks; a running timer goes on at the new pace.
    pub fn set_interval(self, ms: u32) {
        let (_, on) = self.get();
        self.put((ms.max(1), on));
        if on {
            win::timer_start(self.hwnd, self.id, ms);
        }
    }
}
