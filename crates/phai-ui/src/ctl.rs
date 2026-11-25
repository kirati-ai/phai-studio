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

pub use crate::binding::{BindingSource, Bound};
pub use crate::print::PrintDocument;
pub use crate::components::{ColorDialog, ErrorProvider, FolderBrowserDialog, FontDialog, NotifyIcon, OpenFileDialog, SaveFileDialog};

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
    /// Its inside's width and height in pixels - ClientSize; what a Paint method draws on.
    fn client_size(self) -> (i32, i32) {
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(self.hwnd(), &mut r);
        }
        (r.right - r.left, r.bottom - r.top)
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
    /// Drags `text` from it until the mouse is let go - from its MouseDown; true when a control
    /// with a DragDrop event took it (WinForms' DoDragDrop).
    fn do_drag_drop(self, text: &str) -> bool {
        crate::drag::do_drag_drop(self.hwnd(), text)
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
            /// The words it shows - a text box's typing, a button's or a label's caption.
            /// ไทย: ข้อความที่มันแสดง - สิ่งที่พิมพ์ในช่องพิมพ์ หรือข้อความบนปุ่ม/ป้าย
            /// Example: `let name = ui.name_box.text();`
            pub fn text(self) -> String {
                Control::text(self)
            }
            /// Changes the words it shows.
            /// ไทย: เปลี่ยนข้อความที่มันแสดง
            /// Example: `ui.ok.set_text("Save");`
            pub fn set_text(self, s: &str) {
                Control::set_text(self, s)
            }
            /// Whether it can be used now - `false` when it is greyed out.
            /// ไทย: ใช้งานได้อยู่ไหม - false เมื่อเป็นสีเทา
            /// Example: `if ui.ok.enabled() { ... }`
            pub fn enabled(self) -> bool {
                Control::enabled(self)
            }
            /// `true`: it can be used; `false`: greyed out, it cannot be clicked or typed in.
            /// ไทย: true ใช้งานได้ · false เป็นสีเทา กดหรือพิมพ์ไม่ได้
            /// Example: `ui.ok.set_enabled(false);`
            pub fn set_enabled(self, on: bool) {
                Control::set_enabled(self, on)
            }
            /// Whether it can be seen.
            /// ไทย: มองเห็นอยู่ไหม
            /// Example: `if ui.panel1.visible() { ... }`
            pub fn visible(self) -> bool {
                Control::visible(self)
            }
            /// `true` shows it, `false` hides it - the same as `show()` and `hide()`.
            /// ไทย: true แสดง · false ซ่อน - เหมือน show() กับ hide()
            /// Example: `ui.panel1.set_visible(false);`
            pub fn set_visible(self, on: bool) {
                Control::set_visible(self, on)
            }
            /// Shows it again after `hide()`.
            /// ไทย: แสดงอีกครั้งหลัง hide()
            /// Example: `ui.panel1.show();`
            pub fn show(self) {
                Control::show(self)
            }
            /// Hides it; it is still there, and `show()` brings it back.
            /// ไทย: ซ่อนไว้ ยังอยู่ที่เดิม show() เรียกกลับมาได้
            /// Example: `ui.panel1.hide();`
            pub fn hide(self) {
                Control::hide(self)
            }
            /// Puts the keyboard on it: what is typed next goes to it.
            /// ไทย: ให้แป้นพิมพ์มาที่มัน สิ่งที่พิมพ์ต่อไปจะเข้าที่มัน
            /// Example: `ui.name_box.focus();`
            pub fn focus(self) {
                Control::focus(self)
            }
            /// Whether the keyboard is on it now.
            /// ไทย: แป้นพิมพ์อยู่ที่มันไหม
            /// Example: `if ui.name_box.focused() { ... }`
            pub fn focused(self) -> bool {
                Control::focused(self)
            }
            /// Puts it on top of the controls it overlaps.
            /// ไทย: ยกขึ้นไปอยู่บนสุด เหนือ control ที่ซ้อนกัน
            /// Example: `ui.panel1.bring_to_front();`
            pub fn bring_to_front(self) {
                Control::bring_to_front(self)
            }
            /// Puts it behind the controls it overlaps.
            /// ไทย: ส่งไปอยู่ล่างสุด ใต้ control ที่ซ้อนกัน
            /// Example: `ui.panel1.send_to_back();`
            pub fn send_to_back(self) {
                Control::send_to_back(self)
            }
            /// Draws it again now - after changing what its Paint draws.
            /// ไทย: วาดใหม่ทันที - หลังเปลี่ยนสิ่งที่ Paint วาด
            /// Example: `ui.picture1.refresh();`
            pub fn refresh(self) {
                Control::refresh(self)
            }
            /// Asks for it to be drawn again when Windows is free - lighter than `refresh()`.
            /// ไทย: ขอให้วาดใหม่เมื่อ Windows ว่าง - เบากว่า refresh()
            /// Example: `ui.picture1.invalidate();`
            pub fn invalidate(self) {
                Control::invalidate(self)
            }
            /// Where it is on the window and how big: `left`, `top`, `right`, `bottom`.
            /// ไทย: ตำแหน่งและขนาดบนหน้าต่าง: left, top, right, bottom
            /// Example: `let r = ui.ok.bounds(); let width = r.right - r.left;`
            pub fn bounds(self) -> RECT {
                Control::bounds(self)
            }
            /// Its width and height inside its border.
            /// ไทย: ความกว้างและความสูงด้านใน (ไม่รวมขอบ)
            /// Example: `let (w, h) = ui.picture1.client_size();`
            pub fn client_size(self) -> (i32, i32) {
                Control::client_size(self)
            }
            /// Moves it and sets its size: left, top, width, height.
            /// ไทย: ย้ายและเปลี่ยนขนาด: ซ้าย, บน, กว้าง, สูง
            /// Example: `ui.ok.set_bounds(10, 10, 80, 28);`
            pub fn set_bounds(self, x: i32, y: i32, w: i32, h: i32) {
                Control::set_bounds(self, x, y, w, h)
            }
            /// A point on it, as a point on the screen.
            /// ไทย: แปลงจุดบน control เป็นจุดบนจอ
            /// Example: `let (sx, sy) = ui.ok.point_to_screen(0, 0);`
            pub fn point_to_screen(self, x: i32, y: i32) -> (i32, i32) {
                Control::point_to_screen(self, x, y)
            }
            /// A point on the screen, as a point on it.
            /// ไทย: แปลงจุดบนจอเป็นจุดบน control
            /// Example: `let (x, y) = ui.panel1.point_to_client(sx, sy);`
            pub fn point_to_client(self, x: i32, y: i32) -> (i32, i32) {
                Control::point_to_client(self, x, y)
            }
            /// The window (form) it is on.
            /// ไทย: หน้าต่าง (ฟอร์ม) ที่มันอยู่
            /// Example: `let form = ui.ok.find_form();`
            pub fn find_form(self) -> HWND {
                Control::find_form(self)
            }
            /// Stops arranging its controls while many are changed at once; `resume_layout()` after.
            /// ไทย: หยุดจัดตำแหน่งชั่วคราว ระหว่างเปลี่ยนหลายอย่างพร้อมกัน แล้วค่อย resume_layout()
            /// Example: `ui.panel1.suspend_layout();`
            pub fn suspend_layout(self) {
                Control::suspend_layout(self)
            }
            /// Arranges its controls again after `suspend_layout()`.
            /// ไทย: กลับมาจัดตำแหน่งต่อ หลัง suspend_layout()
            /// Example: `ui.panel1.resume_layout();`
            pub fn resume_layout(self) {
                Control::resume_layout(self)
            }
            /// The small words shown when the mouse rests on it.
            /// ไทย: ข้อความเล็ก ๆ ที่ขึ้นเมื่อชี้เมาส์ค้างไว้
            /// Example: `ui.ok.set_tooltip("Saves the score");`
            pub fn set_tooltip(self, text: &str) {
                Control::set_tooltip(self, text)
            }
            /// Starts dragging these words out of it, to drop somewhere else; `true` when they were dropped.
            /// ไทย: เริ่มลากข้อความนี้ออกไปวางที่อื่น ได้ true เมื่อวางสำเร็จ
            /// Example: `ui.list1.do_drag_drop(&ui.list1.text());`
            pub fn do_drag_drop(self, text: &str) -> bool {
                Control::do_drag_drop(self, text)
            }
            /// The mouse pointer's shape over it: `"hand"`, `"wait"`, `"ibeam"`, `"arrow"`, `"cross"`.
            /// ไทย: รูปเมาส์เมื่อชี้มัน: "hand" มือ, "wait" รอ, "ibeam" พิมพ์, "arrow" ลูกศร, "cross" กากบาท
            /// Example: `ui.link1.set_cursor("hand");`
            pub fn set_cursor(self, name: &str) {
                Control::set_cursor(self, name)
            }
        }
    };
}

control!(
    /// A control the project made as a form of its own - WinForms' UserControl. The form's code
    /// reaches its controls with `TheirNameUi::with(ui.x, |c| ...)`.
    UserControl
);
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
    /// A web page inside the window - WinForms' WebView2 (phai-ui's `browser` feature).
    WebBrowser
);
control!(
    /// A chart of a table - WinForms' Chart: columns, bars, lines, areas, pies, candlesticks.
    Chart
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
control!(
    /// WinForms' Panel: holds controls, with a colour of its own or none.
    Panel
);
control!(
    /// WinForms' SplitContainer: two panes and the bar between them.
    SplitContainer
);

impl SplitContainer {
    /// The first pane's width (or height), design pixels.
    pub fn splitter_distance(self) -> i32 {
        crate::split::distance(self.0)
    }
    /// Moves the bar; the panes' controls follow.
    pub fn set_splitter_distance(self, d: i32) {
        crate::split::set_distance(self.0, d)
    }
}
control!(
    /// WinForms' ListBox.
    ListBox
);
control!(
    /// WinForms' CheckedListBox: lines each with a tick box.
    CheckedListBox
);
control!(
    /// WinForms' LinkLabel.
    LinkLabel
);
control!(
    /// WinForms' MaskedTextBox.
    MaskedTextBox
);
control!(
    /// WinForms' MonthCalendar.
    MonthCalendar
);

impl ListBox {
    /// Adds a line at the end (in its place, when sorted); where it went.
    pub fn items_add(self, s: &str) -> usize {
        crate::extra::list_box_add(self.0, s)
    }
    /// Puts a line in at place `at` (0 is the top); the lines below move down.
    /// ไทย: แทรกบรรทัดที่ตำแหน่ง at (บนสุดคือ 0) บรรทัดข้างล่างเลื่อนลง
    /// Example: `ui.list1.items_insert(0, "First");`
    pub fn items_insert(self, at: usize, s: &str) {
        crate::extra::list_box_insert(self.0, at, s)
    }
    /// Takes out the line at place `i` (0 is the top).
    /// ไทย: เอาบรรทัดที่ตำแหน่ง i ออก (บนสุดคือ 0)
    /// Example: `ui.list1.items_remove_at(0);`
    pub fn items_remove_at(self, i: usize) {
        crate::extra::list_box_remove_at(self.0, i)
    }
    /// Takes out every line.
    /// ไทย: ลบทุกบรรทัด
    /// Example: `ui.list1.items_clear();`
    pub fn items_clear(self) {
        crate::extra::list_box_clear(self.0)
    }
    /// How many lines it has.
    /// ไทย: มีกี่บรรทัด
    /// Example: `let n = ui.list1.items_count();`
    pub fn items_count(self) -> usize {
        crate::extra::list_box_count(self.0)
    }
    /// The words of the line at place `i` (0 is the top).
    /// ไทย: ข้อความของบรรทัดที่ตำแหน่ง i (บนสุดคือ 0)
    /// Example: `let first = ui.list1.item(0);`
    pub fn item(self, i: usize) -> String {
        crate::extra::list_box_item(self.0, i)
    }
    /// The place of the chosen line - `None` when none is chosen.
    /// ไทย: ตำแหน่งบรรทัดที่เลือกอยู่ - None เมื่อไม่ได้เลือก
    /// Example: `if let Some(i) = ui.list1.selected_index() { let s = ui.list1.item(i); }`
    pub fn selected_index(self) -> Option<usize> {
        crate::extra::list_box_selected(self.0)
    }
    /// Chooses the line at a place; `None` chooses none.
    /// ไทย: เลือกบรรทัดที่ตำแหน่งนี้ - None คือไม่เลือกเลย
    /// Example: `ui.list1.set_selected_index(Some(0));`
    pub fn set_selected_index(self, i: Option<usize>) {
        crate::extra::list_box_select(self.0, i)
    }
    /// Every chosen line - SelectedIndices.
    pub fn selected_indices(self) -> Vec<usize> {
        crate::extra::list_box_selected_all(self.0)
    }
    /// The chosen line's words - SelectedItem.
    pub fn selected_item(self) -> Option<String> {
        self.selected_index().map(|i| self.item(i))
    }
}

impl CheckedListBox {
    /// Adds a line with a tick box at the end.
    /// ไทย: เพิ่มบรรทัดที่มีช่องติ๊กต่อท้าย
    /// Example: `ui.checks1.items_add("Milk");`
    pub fn items_add(self, s: &str) {
        ListView(self.0).items_add(&[s]);
    }
    /// Takes out every line.
    /// ไทย: ลบทุกบรรทัด
    /// Example: `ui.checks1.items_clear();`
    pub fn items_clear(self) {
        ListView(self.0).items_clear()
    }
    /// How many lines it has.
    /// ไทย: มีกี่บรรทัด
    /// Example: `let n = ui.checks1.items_count();`
    pub fn items_count(self) -> usize {
        ListView(self.0).items_count()
    }
    /// The words of the line at place `i` (0 is the top).
    /// ไทย: ข้อความของบรรทัดที่ตำแหน่ง i (บนสุดคือ 0)
    /// Example: `let s = ui.checks1.item(0);`
    pub fn item(self, i: usize) -> String {
        ListView(self.0).cell(i, 0)
    }
    /// Whether the line at place `i` is ticked.
    /// ไทย: บรรทัดที่ตำแหน่ง i ติ๊กอยู่ไหม
    /// Example: `if ui.checks1.checked(0) { ... }`
    pub fn checked(self, i: usize) -> bool {
        crate::extra::checked_list_checked(self.0, i)
    }
    /// Ticks (`true`) or clears (`false`) the line at place `i`.
    /// ไทย: ติ๊ก (true) หรือเอาติ๊กออก (false) ที่บรรทัดตำแหน่ง i
    /// Example: `ui.checks1.set_checked(0, true);`
    pub fn set_checked(self, i: usize, on: bool) {
        crate::extra::checked_list_set_checked(self.0, i, on)
    }
    /// Every ticked line - CheckedIndices.
    pub fn checked_indices(self) -> Vec<usize> {
        (0..self.items_count()).filter(|i| self.checked(*i)).collect()
    }
    /// The place of the chosen line - `None` when none is chosen.
    /// ไทย: ตำแหน่งบรรทัดที่เลือกอยู่ - None เมื่อไม่ได้เลือก
    /// Example: `if let Some(i) = ui.checks1.selected_index() { ... }`
    pub fn selected_index(self) -> Option<usize> {
        ListView(self.0).selected_index()
    }
}

impl LinkLabel {
    /// Its words, without the link's marks.
    pub fn link_text(self) -> String {
        crate::extra::link_words(self.0)
    }
    /// New words, all a link - or the part between `<a>` and `</a>`.
    pub fn set_link_text(self, s: &str) {
        crate::win::set_text(self.0, &crate::extra::link_markup(s))
    }
}

impl MaskedTextBox {
    /// What was typed, without the mask's literals and blanks.
    pub fn value(self) -> String {
        crate::extra::masked_value(self.0)
    }
    /// Every place that must be filled is - MaskCompleted.
    pub fn mask_completed(self) -> bool {
        crate::extra::mask_completed(self.0)
    }
    /// Takes out what is typed in it.
    /// ไทย: ลบสิ่งที่พิมพ์ไว้
    /// Example: `ui.phone1.clear();`
    pub fn clear(self) {
        crate::win::set_text(self.0, "")
    }
}

impl MonthCalendar {
    /// The chosen day: (year, month, day).
    pub fn selection_start(self) -> (i32, i32, i32) {
        crate::extra::calendar_date(self.0)
    }
    /// Shows and chooses a day: year, month (1-12), day.
    /// ไทย: แสดงและเลือกวันนี้: ปี, เดือน (1-12), วัน
    /// Example: `ui.calendar1.set_date(2026, 11, 23);`
    pub fn set_date(self, y: i32, m: i32, d: i32) {
        crate::extra::calendar_set_date(self.0, y, m, d)
    }
}

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
    /// Whether it is ticked.
    /// ไทย: ติ๊กอยู่ไหม
    /// Example: `if ui.agree.checked() { ... }`
    pub fn checked(self) -> bool {
        win::is_checked(self.0)
    }
    /// Ticks it (`true`) or clears it (`false`).
    /// ไทย: ติ๊ก (true) หรือเอาติ๊กออก (false)
    /// Example: `ui.agree.set_checked(true);`
    pub fn set_checked(self, on: bool) {
        win::set_checked(self.0, on)
    }
    /// Ticked, not ticked, or half (a three-state box): `CheckState::Checked`, `Unchecked`, `Indeterminate`.
    /// ไทย: ติ๊ก, ไม่ติ๊ก หรือครึ่ง ๆ (กล่องสามสถานะ): CheckState::Checked, Unchecked, Indeterminate
    /// Example: `if ui.agree.check_state() == CheckState::Indeterminate { ... }`
    pub fn check_state(self) -> CheckState {
        match win::check_state(self.0) {
            0 => CheckState::Unchecked,
            1 => CheckState::Checked,
            _ => CheckState::Indeterminate,
        }
    }
    /// Sets it ticked, not ticked, or half.
    /// ไทย: ตั้งเป็นติ๊ก, ไม่ติ๊ก หรือครึ่ง ๆ
    /// Example: `ui.agree.set_check_state(CheckState::Checked);`
    pub fn set_check_state(self, s: CheckState) {
        win::set_check_state(self.0, s as u8)
    }
}

impl RadioButton {
    /// Whether it is the one chosen of its group.
    /// ไทย: ตัวนี้เป็นตัวที่เลือกในกลุ่มไหม
    /// Example: `if ui.option_cash.checked() { ... }`
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

impl TextBox {
    /// Its letters shown as dots, or as they are - UseSystemPasswordChar, set while it runs.
    pub fn set_password(self, on: bool) {
        send(self.0, 0x00CC /* EM_SETPASSWORDCHAR */, if on { 0x25CF } else { 0 }, 0);
        unsafe {
            let _ = windows::Win32::Graphics::Gdi::InvalidateRect(Some(self.0), None, true);
        }
    }
}
impl TextEdit for RichTextBox {}

macro_rules! text_edit_methods {
    ($name:ident) => {
        #[allow(dead_code)]
        impl $name {
            /// Takes out all its words.
            /// ไทย: ลบข้อความทั้งหมด
            /// Example: `ui.name_box.clear();`
            pub fn clear(self) {
                TextEdit::clear(self)
            }
            /// Adds words at the end and scrolls to show them.
            /// ไทย: ต่อข้อความท้ายสุด แล้วเลื่อนลงไปให้เห็น
            /// Example: `ui.log.append_text("Done\r\n");`
            pub fn append_text(self, s: &str) {
                TextEdit::append_text(self, s)
            }
            /// Chooses all its words - typing then replaces them.
            /// ไทย: เลือกข้อความทั้งหมด - พิมพ์ต่อจะแทนที่ทั้งหมด
            /// Example: `ui.name_box.select_all();`
            pub fn select_all(self) {
                TextEdit::select_all(self)
            }
            /// Chooses `length` letters starting at place `start` (0 is the first letter).
            /// ไทย: เลือกข้อความ length ตัว เริ่มที่ตำแหน่ง start (ตัวแรกคือ 0)
            /// Example: `ui.name_box.select(0, 3);`
            pub fn select(self, start: usize, length: usize) {
                TextEdit::select(self, start, length)
            }
            /// Where the chosen words begin (or where the caret is).
            /// ไทย: ตำแหน่งเริ่มของส่วนที่เลือก (หรือตำแหน่งเคอร์เซอร์)
            /// Example: `let at = ui.name_box.selection_start();`
            pub fn selection_start(self) -> usize {
                TextEdit::selection_start(self)
            }
            /// How many letters are chosen.
            /// ไทย: เลือกไว้กี่ตัว
            /// Example: `let n = ui.name_box.selection_length();`
            pub fn selection_length(self) -> usize {
                TextEdit::selection_length(self)
            }
            /// The chosen words.
            /// ไทย: ข้อความที่เลือกไว้
            /// Example: `let s = ui.name_box.selected_text();`
            pub fn selected_text(self) -> String {
                TextEdit::selected_text(self)
            }
            /// Puts words in at the caret, in place of the chosen ones.
            /// ไทย: ใส่ข้อความที่ตำแหน่งเคอร์เซอร์ แทนส่วนที่เลือก
            /// Example: `ui.notes.paste_text("Hello");`
            pub fn paste_text(self, s: &str) {
                TextEdit::paste_text(self, s)
            }
            /// Copies the chosen words to the clipboard - Ctrl+C.
            /// ไทย: คัดลอกส่วนที่เลือกไปคลิปบอร์ด - Ctrl+C
            /// Example: `ui.notes.copy();`
            pub fn copy(self) {
                TextEdit::copy(self)
            }
            /// Moves the chosen words to the clipboard - Ctrl+X.
            /// ไทย: ตัดส่วนที่เลือกไปคลิปบอร์ด - Ctrl+X
            /// Example: `ui.notes.cut();`
            pub fn cut(self) {
                TextEdit::cut(self)
            }
            /// Puts in the clipboard's words at the caret - Ctrl+V.
            /// ไทย: วางข้อความจากคลิปบอร์ดที่เคอร์เซอร์ - Ctrl+V
            /// Example: `ui.notes.paste();`
            pub fn paste(self) {
                TextEdit::paste(self)
            }
            /// Takes back the last change - Ctrl+Z.
            /// ไทย: ย้อนสิ่งที่แก้ล่าสุด - Ctrl+Z
            /// Example: `ui.notes.undo();`
            pub fn undo(self) {
                TextEdit::undo(self)
            }
            /// Whether there is a change to take back.
            /// ไทย: มีอะไรให้ย้อนไหม
            /// Example: `ui.undo_button.set_enabled(ui.notes.can_undo());`
            pub fn can_undo(self) -> bool {
                TextEdit::can_undo(self)
            }
            /// Scrolls so the caret can be seen.
            /// ไทย: เลื่อนให้เห็นเคอร์เซอร์
            /// Example: `ui.log.scroll_to_caret();`
            pub fn scroll_to_caret(self) {
                TextEdit::scroll_to_caret(self)
            }
            /// Whether it can only be read, not typed in.
            /// ไทย: อ่านอย่างเดียวอยู่ไหม (พิมพ์ไม่ได้)
            /// Example: `if ui.notes.read_only() { ... }`
            pub fn read_only(self) -> bool {
                TextEdit::read_only(self)
            }
            /// `true`: it can be read and copied, not typed in.
            /// ไทย: true อ่านและคัดลอกได้ แต่พิมพ์ไม่ได้
            /// Example: `ui.notes.set_read_only(true);`
            pub fn set_read_only(self, on: bool) {
                TextEdit::set_read_only(self, on)
            }
            /// The most letters that can be typed in it.
            /// ไทย: พิมพ์ได้มากสุดกี่ตัว
            /// Example: `ui.code_box.set_max_length(10);`
            pub fn set_max_length(self, n: i32) {
                TextEdit::set_max_length(self, n)
            }
            /// Its words, line by line.
            /// ไทย: ข้อความแยกเป็นบรรทัด
            /// Example: `for line in ui.notes.lines() { ... }`
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
    /// Puts a choice in at place `at` (0 is the top).
    /// ไทย: แทรกตัวเลือกที่ตำแหน่ง at (บนสุดคือ 0)
    /// Example: `ui.combo1.items_insert(0, "Any");`
    pub fn items_insert(self, at: usize, s: &str) {
        let w = win::wide(s);
        send(self.0, 0x014A /* CB_INSERTSTRING */, at, w.as_ptr() as isize);
    }
    /// Takes out the choice at place `at` (0 is the top).
    /// ไทย: เอาตัวเลือกที่ตำแหน่ง at ออก
    /// Example: `ui.combo1.items_remove_at(0);`
    pub fn items_remove_at(self, at: usize) {
        send(self.0, 0x0144 /* CB_DELETESTRING */, at, 0);
    }
    /// Takes out every choice.
    /// ไทย: ลบตัวเลือกทั้งหมด
    /// Example: `ui.combo1.items_clear();`
    pub fn items_clear(self) {
        send(self.0, 0x014B /* CB_RESETCONTENT */, 0, 0);
    }
    /// How many choices it has.
    /// ไทย: มีกี่ตัวเลือก
    /// Example: `let n = ui.combo1.items_count();`
    pub fn items_count(self) -> usize {
        send(self.0, 0x0146 /* CB_GETCOUNT */, 0, 0).max(0) as usize
    }
    /// The words of the choice at place `at` (0 is the top).
    /// ไทย: ข้อความของตัวเลือกที่ตำแหน่ง at (บนสุดคือ 0)
    /// Example: `let first = ui.combo1.item(0);`
    pub fn item(self, at: usize) -> String {
        let n = send(self.0, 0x0149 /* CB_GETLBTEXTLEN */, at, 0);
        if n < 0 {
            return String::new();
        }
        let mut buf = vec![0u16; n as usize + 1];
        send(self.0, 0x0148 /* CB_GETLBTEXT */, at, buf.as_mut_ptr() as isize);
        String::from_utf16_lossy(&buf[..n as usize])
    }
    /// All its choices.
    /// ไทย: ตัวเลือกทั้งหมด
    /// Example: `for s in ui.combo1.items() { ... }`
    pub fn items(self) -> Vec<String> {
        (0..self.items_count()).map(|i| self.item(i)).collect()
    }
    /// The item chosen, from 0; `None` when none is.
    pub fn selected_index(self) -> Option<usize> {
        let i = win::combo_selected(self.0);
        (i >= 0).then_some(i as usize)
    }
    /// Chooses the choice at a place; `None` chooses none.
    /// ไทย: เลือกตัวเลือกที่ตำแหน่งนี้ - None คือไม่เลือก
    /// Example: `ui.combo1.set_selected_index(Some(0));`
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
    /// Shows the changes made after `begin_update()`, all at once.
    /// ไทย: แสดงสิ่งที่เปลี่ยนหลัง begin_update() ทีเดียวทั้งหมด
    /// Example: `ui.combo1.end_update();`
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
    /// Adds a row showing picture `icon` of its `icons` (ImageIndex); its place.
    pub fn items_add_with_icon(self, cells: &[&str], icon: usize) -> usize {
        let at = self.items_add(cells);
        self.set_item_icon(at, icon);
        at
    }
    /// The picture row `at` shows: a place in the list's `icons`.
    pub fn set_item_icon(self, at: usize, icon: usize) {
        win::list_set_image(self.0, at, icon as i32);
    }
    /// The list's pictures, as the `icons` property writes them (`MONITOR #12855f, FOLDER`).
    pub fn set_icons(self, icons: &str) {
        crate::props::icons(self.0, icons);
    }
    /// Takes out the row at place `at` (0 is the top).
    /// ไทย: เอาแถวที่ตำแหน่ง at ออก (บนสุดคือ 0)
    /// Example: `ui.list1.items_remove_at(0);`
    pub fn items_remove_at(self, at: usize) {
        send(self.0, cc::LVM_DELETEITEM, at, 0);
    }
    /// Takes out every row.
    /// ไทย: ลบทุกแถว
    /// Example: `ui.list1.items_clear();`
    pub fn items_clear(self) {
        send(self.0, cc::LVM_DELETEALLITEMS, 0, 0);
    }
    /// How many rows it has.
    /// ไทย: มีกี่แถว
    /// Example: `let n = ui.list1.items_count();`
    pub fn items_count(self) -> usize {
        send(self.0, cc::LVM_GETITEMCOUNT, 0, 0).max(0) as usize
    }
    /// The words in row `row`, column `col` (both from 0).
    /// ไทย: ข้อความในแถว row คอลัมน์ col (นับจาก 0 ทั้งคู่)
    /// Example: `let name = ui.list1.cell(0, 1);`
    pub fn cell(self, row: usize, col: usize) -> String {
        win::list_text(self.0, row, col as i32)
    }
    /// Changes the words in row `row`, column `col` (both from 0).
    /// ไทย: เปลี่ยนข้อความในแถว row คอลัมน์ col (นับจาก 0 ทั้งคู่)
    /// Example: `ui.list1.set_cell(0, 1, "Somchai");`
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
    /// The places of every chosen row.
    /// ไทย: ตำแหน่งของทุกแถวที่เลือกไว้
    /// Example: `for i in ui.list1.selected_indices() { ... }`
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
    /// Scrolls so the row at place `at` can be seen.
    /// ไทย: เลื่อนให้เห็นแถวที่ตำแหน่ง at
    /// Example: `ui.list1.ensure_visible(ui.list1.items_count() - 1);`
    pub fn ensure_visible(self, at: usize) {
        send(self.0, cc::LVM_ENSUREVISIBLE, at, 0);
    }
    /// A row's tick, with CheckBoxes on.
    pub fn checked(self, at: usize) -> bool {
        (send(self.0, cc::LVM_GETITEMSTATE, at, 0xF000 /* LVIS_STATEIMAGEMASK */) >> 12) == 2
    }
    /// Ticks or clears the tick box of the row at place `at`.
    /// ไทย: ติ๊กหรือเอาติ๊กออกที่แถวตำแหน่ง at
    /// Example: `ui.list1.set_checked(0, true);`
    pub fn set_checked(self, at: usize, on: bool) {
        let it = cc::LVITEMW { stateMask: cc::LIST_VIEW_ITEM_STATE_FLAGS(0xF000), state: cc::LIST_VIEW_ITEM_STATE_FLAGS(if on { 2 } else { 1 } << 12), ..Default::default() };
        send(self.0, cc::LVM_SETITEMSTATE, at, &it as *const _ as isize);
    }
    /// Stops drawing while many rows are added; `end_update()` shows them all at once.
    /// ไทย: หยุดวาดระหว่างเพิ่มหลายแถว แล้ว end_update() แสดงทีเดียว
    /// Example: `ui.list1.begin_update();`
    pub fn begin_update(self) {
        Control::suspend_layout(self)
    }
    /// Shows the rows added after `begin_update()`, all at once.
    /// ไทย: แสดงแถวที่เพิ่มหลัง begin_update() ทีเดียวทั้งหมด
    /// Example: `ui.list1.end_update();`
    pub fn end_update(self) {
        Control::resume_layout(self)
    }
}

impl Grid {
    /// Every row's cells.
    pub fn rows(self) -> Vec<Vec<String>> {
        crate::grid::grid_rows(self.0)
    }
    /// Adds a row at the bottom, a cell for each column.
    /// ไทย: เพิ่มแถวท้ายตาราง ใส่ข้อความทีละคอลัมน์
    /// Example: `ui.grid1.rows_add(&["1", "Somchai", "90"]);`
    pub fn rows_add(self, cells: &[&str]) {
        crate::grid::grid_add(self.0, cells)
    }
    /// Takes out every row.
    /// ไทย: ลบทุกแถว
    /// Example: `ui.grid1.rows_clear();`
    pub fn rows_clear(self) {
        crate::grid::grid_clear(self.0)
    }
    /// How many rows it has.
    /// ไทย: มีกี่แถว
    /// Example: `let n = ui.grid1.rows_count();`
    pub fn rows_count(self) -> usize {
        send(self.0, cc::LVM_GETITEMCOUNT, 0, 0).max(0) as usize
    }
    /// Every row at once, in place of those it had.
    pub fn fill(self, rows: &[Vec<String>]) {
        crate::grid::grid_fill(self.0, rows)
    }
    /// The words in row `row`, column `col` (both from 0).
    /// ไทย: ข้อความในแถว row คอลัมน์ col (นับจาก 0 ทั้งคู่)
    /// Example: `let score = to_int(&ui.grid1.cell(0, 2));`
    pub fn cell(self, row: usize, col: usize) -> String {
        crate::grid::cell(self.0, row, col)
    }
    /// Changes the words in row `row`, column `col` (both from 0).
    /// ไทย: เปลี่ยนข้อความในแถว row คอลัมน์ col (นับจาก 0 ทั้งคู่)
    /// Example: `ui.grid1.set_cell(0, 2, "95");`
    pub fn set_cell(self, row: usize, col: usize, text: &str) {
        crate::grid::set_cell(self.0, row, col, text)
    }
    /// The place of the chosen row - `None` when none is chosen.
    /// ไทย: ตำแหน่งแถวที่เลือกอยู่ - None เมื่อไม่ได้เลือก
    /// Example: `if let Some(r) = ui.grid1.selected_row() { ... }`
    pub fn selected_row(self) -> Option<usize> {
        let i = send(self.0, cc::LVM_GETNEXTITEM, usize::MAX, 2);
        (i >= 0).then_some(i as usize)
    }
    /// No row chosen.
    pub fn clear_selection(self) {
        let it = cc::LVITEMW { stateMask: cc::LVIS_SELECTED, state: cc::LIST_VIEW_ITEM_STATE_FLAGS(0), ..Default::default() };
        send(self.0, cc::LVM_SETITEMSTATE, usize::MAX, &it as *const _ as isize);
    }
    /// `true`: its cells can be read, not changed.
    /// ไทย: true อ่านได้ แก้ช่องในตารางไม่ได้
    /// Example: `ui.grid1.set_read_only(true);`
    pub fn set_read_only(self, on: bool) {
        crate::grid::set_read_only(self.0, on)
    }
    /// The rows chosen, by place.
    pub fn selected_rows(self) -> Vec<usize> {
        crate::grid::selected_rows(self.0)
    }
    /// The rows sorted by a column, up or down - as a click on its heading.
    pub fn sort_by(self, col: usize, up: bool) {
        crate::grid::sort_by(self.0, col, up)
    }
    /// A column's properties.
    pub fn column(self, col: usize) -> crate::grid::Column {
        crate::grid::column(self.0, col)
    }
    /// How column `col` (from 0) shows and takes its cells.
    /// ไทย: ตั้งค่าคอลัมน์ col (นับจาก 0): รูปแบบตัวเลข, ชิดซ้ายขวา, ซ่อน...
    /// Example: `let mut c = ui.grid1.column(2); c.format = "n2".into(); ui.grid1.set_column(2, c);`
    pub fn set_column(self, col: usize, c: crate::grid::Column) {
        crate::grid::set_column(self.0, col, c)
    }
    /// Looks for words as the find box does; empty shows every row again.
    pub fn find(self, text: &str) {
        crate::grid::find(self.0, text)
    }
    /// Only the rows whose value in column `col` is one of `keep`, as a heading's filter leaves
    /// them; `None` shows them all again.
    pub fn set_column_filter(self, col: usize, keep: Option<&[&str]>) {
        crate::grid::set_column_filter(self.0, col, keep.map(|k| k.iter().map(|s| s.to_string()).collect()))
    }
    /// The values column `col`'s filter keeps; `None` when it has none.
    pub fn column_filter(self, col: usize) -> Option<Vec<String>> {
        crate::grid::column_filter(self.0, col)
    }
    /// Every column's filter taken off.
    pub fn clear_filters(self) {
        crate::grid::clear_filters(self.0)
    }
    /// Master-detail: this grid shows only the rows whose column `detail_col` holds `master`'s
    /// chosen row's value in `master_col`, following it.
    pub fn set_master(self, master: Grid, master_col: usize, detail_col: usize) {
        crate::grid::set_master(self.0, master.0, master_col, detail_col)
    }
    /// The rows shown written to an Excel workbook (.xlsx), numbers and dates as Excel's.
    pub fn export_xlsx(self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        crate::grid::export_xlsx(self.0, path.as_ref())
    }
    /// The rows shown printed to a PDF: how many pages, 0 when it could not.
    pub fn export_pdf(self, path: impl AsRef<std::path::Path>, title: &str) -> usize {
        crate::grid::export_pdf(self.0, path.as_ref(), title)
    }
    /// The rows shown as they would print, in a preview window.
    pub fn print_preview(self, title: &str) {
        crate::grid::print_preview(self.0, title)
    }
    /// The chosen rows as tab-separated lines, as Ctrl+C copies them.
    pub fn copy_rows(self) -> String {
        crate::grid::copy_rows(self.0)
    }
    /// The rows shown written to a CSV file Excel opens, Thai and all.
    pub fn export_csv(self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        crate::grid::export_csv(self.0, path.as_ref())
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
    /// Adds a node under `parent`; the new node, to add under in turn.
    /// ไทย: เพิ่มกิ่งใต้ parent ได้กิ่งใหม่ไว้เพิ่มใต้มันต่อได้
    /// Example: `let fruit = ui.tree1.nodes_add("Fruit"); ui.tree1.nodes_add_under(fruit, "Mango");`
    pub fn nodes_add_under(self, parent: Node, text: &str) -> Node {
        Node(win::tree_add(self.0, parent.0, text, -1, 0))
    }
    /// Takes out every node.
    /// ไทย: ลบทุกกิ่ง
    /// Example: `ui.tree1.nodes_clear();`
    pub fn nodes_clear(self) {
        win::tree_clear(self.0)
    }
    /// The lines as the form's Items write them: two spaces before an item per level down.
    pub fn set_nodes(self, lines: &[&str]) {
        win::tree_clear(self.0);
        win::tree_items(self.0, lines)
    }
    /// Opens every node, so all are seen.
    /// ไทย: กางทุกกิ่ง ให้เห็นทั้งหมด
    /// Example: `ui.tree1.expand_all();`
    pub fn expand_all(self) {
        crate::props::expand_all(self.0, true)
    }
    /// Closes every node.
    /// ไทย: หุบทุกกิ่ง
    /// Example: `ui.tree1.collapse_all();`
    pub fn collapse_all(self) {
        crate::props::expand_all(self.0, false)
    }
    /// The item chosen.
    pub fn selected_node(self) -> Option<Node> {
        let i = send(self.0, 0x110A /* TVM_GETNEXTITEM */, 9 /* TVGN_CARET */, 0);
        (i != 0).then_some(Node(i))
    }
    /// Chooses a node.
    /// ไทย: เลือกกิ่งนี้
    /// Example: `ui.tree1.select(fruit);`
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
    /// Opens a node, showing the nodes under it.
    /// ไทย: กางกิ่งนี้ ให้เห็นกิ่งข้างใต้
    /// Example: `ui.tree1.expand(fruit);`
    pub fn expand(self, n: Node) {
        send(self.0, 0x1102 /* TVM_EXPAND */, 2, n.0);
    }
    /// Closes a node, hiding the nodes under it.
    /// ไทย: หุบกิ่งนี้ ซ่อนกิ่งข้างใต้
    /// Example: `ui.tree1.collapse(fruit);`
    pub fn collapse(self, n: Node) {
        send(self.0, 0x1102, 1, n.0);
    }
}

impl DateTimePicker {
    /// The date shown, `(year, month, day)`.
    pub fn value(self) -> (i32, i32, i32) {
        win::date_value(self.0)
    }
    /// Sets the day it shows: year, month (1-12), day.
    /// ไทย: ตั้งวันที่ที่แสดง: ปี, เดือน (1-12), วัน
    /// Example: `ui.date1.set_value(2026, 11, 23);`
    pub fn set_value(self, year: i32, month: i32, day: i32) {
        win::set_date(self.0, year, month, day)
    }
    /// With ShowCheckBox: whether a date is given (its box ticked).
    pub fn checked(self) -> bool {
        win::date_checked(self.0)
    }
}

impl NumericUpDown {
    /// The number it shows.
    /// ไทย: ตัวเลขที่แสดงอยู่
    /// Example: `let qty = ui.qty.value();`
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
    /// Takes one step off the number, as its down arrow does.
    /// ไทย: ลดลงหนึ่งขั้น เหมือนกดลูกศรลง
    /// Example: `ui.qty.down_button();`
    pub fn down_button(self) {
        self.set_value(self.value() - self.step())
    }
}

impl TrackBar {
    /// Where its handle is, as a number.
    /// ไทย: ตำแหน่งของปุ่มเลื่อน เป็นตัวเลข
    /// Example: `let volume = ui.volume.value();`
    pub fn value(self) -> i32 {
        win::value(self.0)
    }
    /// Moves its handle to a number.
    /// ไทย: เลื่อนปุ่มไปที่ตัวเลขนี้
    /// Example: `ui.volume.set_value(50);`
    pub fn set_value(self, v: i32) {
        win::set_value(self.0, v)
    }
}

impl ProgressBar {
    /// How far it is filled, as a number.
    /// ไทย: เต็มไปแค่ไหน เป็นตัวเลข
    /// Example: `let done = ui.progress1.value();`
    pub fn value(self) -> i32 {
        win::value(self.0)
    }
    /// Fills it up to a number (0 to 100 unless set otherwise).
    /// ไทย: เติมถึงตัวเลขนี้ (0 ถึง 100 ถ้าไม่ได้ตั้งไว้อื่น)
    /// Example: `ui.progress1.set_value(40);`
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
    /// `true`: a moving bar, for work whose end is not known.
    /// ไทย: true แถบวิ่งไปมา สำหรับงานที่ไม่รู้ว่าจะเสร็จเมื่อไร
    /// Example: `ui.progress1.set_marquee(true);`
    pub fn set_marquee(self, on: bool) {
        crate::props::marquee(self.0, on)
    }
}

impl PictureBox {
    /// Shows another picture file.
    pub fn load(self, path: &str) {
        crate::picture::set_picture(self.0, path)
    }
    /// How its picture fits: `"normal"` as it is, `"stretch"` to fill, `"center"`, `"zoom"` to fit and keep its shape.
    /// ไทย: รูปพอดีกล่องแบบไหน: "normal" ขนาดจริง, "stretch" ยืดเต็ม, "center" กลาง, "zoom" ย่อขยายพอดีไม่เสียทรง
    /// Example: `ui.picture1.set_size_mode("zoom");`
    pub fn set_size_mode(self, mode: &str) {
        crate::picture::set_size_mode(self.0, mode)
    }
}

impl Chart {
    /// Its table: rows of cells, the first row the headings - what the categories are, then each series.
    /// ไทย: ข้อมูลของกราฟเป็นแถว แถวแรกคือหัวคอลัมน์ (ชื่อหมวด แล้วชื่อแต่ละชุดข้อมูล)
    /// Example: `ui.chart1.set_data(&[vec!["Month".into(), "Sales".into()], vec!["Jan".into(), "120".into()]]);`
    pub fn set_data(self, rows: &[Vec<String>]) {
        crate::chart::set_data(self.0, rows)
    }
    /// Its table as text: a line a row, commas between, the first line the headings.
    /// ไทย: ข้อมูลของกราฟเป็นข้อความ บรรทัดละแถว คั่นด้วยจุลภาค บรรทัดแรกคือหัวคอลัมน์
    /// Example: `ui.chart1.set_data_text("Month, Sales\nJan, 120\nFeb, 150");`
    pub fn set_data_text(self, text: &str) {
        crate::chart::set_data_text(self.0, text)
    }
    /// One more point at the end - a day, a second - with a value for each series.
    /// ไทย: เพิ่มจุดข้อมูลต่อท้าย พร้อมค่าของแต่ละชุด
    /// Example: `ui.price.add_point("10:31", &[101.25]);`
    pub fn add_point(self, label: &str, values: &[f64]) {
        crate::chart::add_point(self.0, label, values)
    }
    /// How many points it keeps as they are added, the oldest dropped; 0 keeps them all.
    /// ไทย: เก็บจุดล่าสุดไว้กี่จุด จุดเก่าสุดหายไปก่อน 0 คือเก็บทั้งหมด
    /// Example: `ui.price.set_max_points(120);`
    pub fn set_max_points(self, n: usize) {
        crate::chart::set_max_points(self.0, n)
    }
    /// What it draws: `"column"`, `"bar"`, `"line"`, `"area"`, `"pie"`, `"doughnut"` or `"candlestick"`.
    /// ไทย: ชนิดกราฟ: แท่งตั้ง แท่งนอน เส้น พื้นที่ วงกลม โดนัท หรือแท่งเทียน
    /// Example: `ui.chart1.set_type("line");`
    pub fn set_type(self, kind: &str) {
        crate::chart::set_type(self.0, kind)
    }
    /// The words over it.
    /// ไทย: ชื่อกราฟด้านบน
    /// Example: `ui.chart1.set_title("ยอดขายรายเดือน");`
    pub fn set_title(self, title: &str) {
        crate::chart::set_title(self.0, title)
    }
    /// Whether each series' colour and name shows.
    /// ไทย: แสดงคำอธิบายสีของแต่ละชุดหรือไม่
    /// Example: `ui.chart1.set_legend(false);`
    pub fn set_legend(self, on: bool) {
        crate::chart::set_legend(self.0, on)
    }
    /// Category `k` shown as the mouse over it would, its values in a box; `None` for none.
    /// ไทย: ชี้ที่หมวดที่ k เหมือนเอาเมาส์ไปวาง แสดงค่าในกล่อง
    /// Example: `ui.chart1.highlight(Some(2));`
    pub fn highlight(self, k: Option<usize>) {
        crate::chart::highlight(self.0, k)
    }
    /// Every point taken out; the headings stay.
    /// ไทย: ลบจุดข้อมูลทั้งหมด หัวคอลัมน์ยังอยู่
    /// Example: `ui.price.clear();`
    pub fn clear(self) {
        crate::chart::clear(self.0)
    }
}

impl WebBrowser {
    /// Opens a page.
    /// ไทย: เปิดหน้าเว็บ
    /// Example: `ui.page.navigate("https://example.com/");`
    pub fn navigate(self, url: &str) {
        crate::browser::navigate(self.0, url)
    }
    /// The address of the page shown now.
    /// ไทย: ที่อยู่ของหน้าที่แสดงอยู่
    /// Example: `let at = ui.page.url();`
    pub fn url(self) -> String {
        crate::browser::url(self.0)
    }
    /// Loads the page again.
    /// ไทย: โหลดหน้าใหม่อีกครั้ง
    /// Example: `ui.page.reload();`
    pub fn reload(self) {
        crate::browser::reload(self.0)
    }
    /// The page before this one, as a browser's Back button.
    /// ไทย: กลับไปหน้าก่อน
    /// Example: `ui.page.go_back();`
    pub fn go_back(self) {
        crate::browser::go_back(self.0)
    }
    /// Accepts a certificate the computer does not trust - one a server made itself - for this
    /// one host only, instead of WebView2's warning page.
    /// ไทย: ยอมรับใบรับรองที่เครื่องไม่รู้จัก (เช่นที่เซิร์ฟเวอร์ทำขึ้นเอง) เฉพาะ host นี้เท่านั้น
    /// Example: `ui.page.allow_certificate_errors_for("192.168.0.10");`
    pub fn allow_certificate_errors_for(self, host: &str) {
        crate::browser::allow_certificate_errors_for(self.0, host)
    }
    /// Whether a page is showing: WebView2 started on this computer.
    /// ไทย: แสดงหน้าเว็บได้แล้วหรือยัง
    /// Example: `if !ui.page.ready() { ... }`
    pub fn ready(self) -> bool {
        crate::browser::ready(self.0)
    }
}

impl TabControl {
    /// The place of the tab in front (0 is the first).
    /// ไทย: แท็บที่อยู่หน้าอยู่ลำดับที่เท่าไร (แท็บแรกคือ 0)
    /// Example: `if ui.tabs1.selected_index() == 0 { ... }`
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
    /// Starts it: its Tick comes every `interval` milliseconds.
    /// ไทย: เริ่มนับ: Tick จะมาทุก interval มิลลิวินาที
    /// Example: `ui.timer1.start();`
    pub fn start(self) {
        let (ms, _) = self.get();
        self.put((ms, true));
        win::timer_start(self.hwnd, self.id, ms);
    }
    /// Stops it: no more Tick.
    /// ไทย: หยุด ไม่มี Tick อีก
    /// Example: `ui.timer1.stop();`
    pub fn stop(self) {
        let (ms, _) = self.get();
        self.put((ms, false));
        win::timer_stop(self.hwnd, self.id);
    }
    /// Whether it is running.
    /// ไทย: กำลังทำงานอยู่ไหม
    /// Example: `if ui.timer1.enabled() { ... }`
    pub fn enabled(self) -> bool {
        self.get().1
    }
    /// `true` starts it, `false` stops it - as `start()` and `stop()`.
    /// ไทย: true เริ่ม · false หยุด - เหมือน start() กับ stop()
    /// Example: `ui.timer1.set_enabled(false);`
    pub fn set_enabled(self, on: bool) {
        if on {
            self.start()
        } else {
            self.stop()
        }
    }
    /// How many milliseconds between its Ticks (1000 is one second).
    /// ไทย: ห่างกันกี่มิลลิวินาทีต่อหนึ่ง Tick (1000 คือ 1 วินาที)
    /// Example: `let ms = ui.timer1.interval();`
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
