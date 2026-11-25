//! The controls as a program's code uses them - `ui.name.text()`, `ui.ok.set_enabled(false)` -
//! the same types and methods as `phai_ui::ctl` on Windows, working on the page's elements.

use std::cell::RefCell;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};

use crate::form::input_of;
use crate::win::{el, make, HWND};

fn value_of(e: &HtmlElement) -> Option<String> {
    if let Some(i) = e.dyn_ref::<HtmlInputElement>() {
        return Some(i.value());
    }
    if let Some(t) = e.dyn_ref::<HtmlTextAreaElement>() {
        return Some(t.value());
    }
    if let Some(s) = e.dyn_ref::<HtmlSelectElement>() {
        let i = s.selected_index();
        return Some(if i < 0 { String::new() } else { s.options().item(i as u32).and_then(|o| o.text_content()).unwrap_or_default() });
    }
    None
}

/// The words a control shows: a box's text, a button's, a check box's, a group's title.
fn text_of(h: HWND) -> String {
    let Some(e) = el(h) else { return String::new() };
    if let Some(v) = value_of(&e) {
        return v;
    }
    for inner in ["legend", "label > span", ":scope > span"] {
        if let Ok(Some(x)) = e.query_selector(inner) {
            return x.text_content().unwrap_or_default();
        }
    }
    e.text_content().unwrap_or_default()
}

fn set_text_of(h: HWND, s: &str) {
    let Some(e) = el(h) else { return };
    if let Some(i) = e.dyn_ref::<HtmlInputElement>() {
        i.set_value(s);
        return;
    }
    if let Some(t) = e.dyn_ref::<HtmlTextAreaElement>() {
        t.set_value(s);
        return;
    }
    if let Some(sel) = e.dyn_ref::<HtmlSelectElement>() {
        let o = sel.options();
        let at = (0..o.length()).find(|i| o.item(*i).and_then(|x| x.text_content()).as_deref() == Some(s));
        sel.set_selected_index(at.map(|i| i as i32).unwrap_or(-1));
        return;
    }
    for inner in ["legend", ":scope > span"] {
        if let Ok(Some(x)) = e.query_selector(inner) {
            x.set_text_content(Some(s));
            return;
        }
    }
    e.set_text_content(Some(s));
}

/// What every control does - WinForms' Control.
pub trait Control: Copy {
    fn hwnd(self) -> HWND;
    fn text(self) -> String {
        text_of(self.hwnd())
    }
    fn set_text(self, s: &str) {
        set_text_of(self.hwnd(), s)
    }
    fn enabled(self) -> bool {
        el(self.hwnd()).is_some_and(|e| !e.has_attribute("disabled") && input_of(self.hwnd()).is_none_or(|i| !i.disabled()))
    }
    fn set_enabled(self, on: bool) {
        let h = self.hwnd();
        if let Some(e) = el(h) {
            if on {
                let _ = e.remove_attribute("disabled");
            } else {
                let _ = e.set_attribute("disabled", "");
            }
            e.class_list().toggle_with_force("disabled", !on).ok();
        }
        if let Some(i) = input_of(h) {
            i.set_disabled(!on);
        }
    }
    fn visible(self) -> bool {
        el(self.hwnd()).is_some_and(|e| e.style().get_property_value("display").unwrap_or_default() != "none")
    }
    fn set_visible(self, on: bool) {
        if let Some(e) = el(self.hwnd()) {
            let _ = e.style().set_property("display", if on { "" } else { "none" });
        }
    }
    fn show(self) {
        self.set_visible(true)
    }
    fn hide(self) {
        self.set_visible(false)
    }
    fn focus(self) {
        let h = self.hwnd();
        let target: Option<HtmlElement> = input_of(h).map(|i| i.unchecked_into()).or_else(|| el(h));
        if let Some(e) = target {
            let _ = e.focus();
        }
    }
    fn focused(self) -> bool {
        let active = crate::win::document().active_element();
        el(self.hwnd()).is_some_and(|e| active.as_ref().is_some_and(|a| e.contains(Some(a))))
    }
    /// The words shown when the mouse rests on it.
    fn set_tooltip(self, text: &str) {
        if let Some(e) = el(self.hwnd()) {
            e.set_title(text);
        }
    }
    /// A colour of its own, any CSS colour.
    fn set_back_color(self, css: &str) {
        if let Some(e) = el(self.hwnd()) {
            let _ = e.style().set_property("background", css);
        }
    }
    fn set_fore_color(self, css: &str) {
        if let Some(e) = el(self.hwnd()) {
            let _ = e.style().set_property("color", css);
        }
    }
    fn bring_to_front(self) {
        if let Some(e) = el(self.hwnd()) {
            let _ = e.style().set_property("z-index", "10");
        }
    }
    fn send_to_back(self) {
        if let Some(e) = el(self.hwnd()) {
            let _ = e.style().set_property("z-index", "0");
        }
    }
    fn refresh(self) {}
    /// Drawn again, when its Paint is the program's (`crate::paint`).
    fn invalidate(self) {
        crate::paint::invalidate(self.hwnd());
    }
    /// The size inside it, pixels - ClientSize.
    fn client_size(self) -> (i32, i32) {
        el(self.hwnd()).map(|e| (e.client_width(), e.client_height())).unwrap_or((0, 0))
    }
    fn suspend_layout(self) {}
    fn resume_layout(self) {}
}

macro_rules! control {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
        pub struct $name(pub HWND);

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
            pub fn set_tooltip(self, text: &str) {
                Control::set_tooltip(self, text)
            }
            pub fn set_back_color(self, css: &str) {
                Control::set_back_color(self, css)
            }
            pub fn set_fore_color(self, css: &str) {
                Control::set_fore_color(self, css)
            }
            pub fn bring_to_front(self) {
                Control::bring_to_front(self)
            }
            pub fn send_to_back(self) {
                Control::send_to_back(self)
            }
            pub fn refresh(self) {}
            pub fn invalidate(self) {
                Control::invalidate(self)
            }
            pub fn client_size(self) -> (i32, i32) {
                Control::client_size(self)
            }
            pub fn suspend_layout(self) {}
            pub fn resume_layout(self) {}
        }
    };
}

control!(Label);
control!(Button);
control!(CheckBox);
control!(RadioButton);
control!(TextBox);
control!(RichTextBox);
control!(MaskedTextBox);
control!(ComboBox);
control!(ListBox);
control!(CheckedListBox);
control!(ListView);
control!(Grid);
control!(DateTimePicker);
control!(MonthCalendar);
control!(NumericUpDown);
control!(TrackBar);
control!(ProgressBar);
control!(GroupBox);
control!(Panel);
control!(FlowPanel);
control!(TablePanel);
control!(TabControl);
control!(StatusBar);
control!(LinkLabel);
control!(PictureBox);
control!(WebBrowser);
control!(TreeView);
control!(SplitContainer);
control!(UserControl);
control!(ContextMenu);

pub use crate::tree::Node;

impl Button {
    /// A click, as a finger or the mouse would make it.
    pub fn perform_click(self) {
        if let Some(e) = el(self.0) {
            e.click();
        }
    }
}

impl CheckBox {
    pub fn checked(self) -> bool {
        input_of(self.0).is_some_and(|i| i.checked())
    }
    pub fn set_checked(self, on: bool) {
        crate::form::set_checked(self.0, on)
    }
}

impl RadioButton {
    pub fn checked(self) -> bool {
        input_of(self.0).is_some_and(|i| i.checked())
    }
    pub fn set_checked(self, on: bool) {
        crate::form::set_checked(self.0, on)
    }
}

macro_rules! text_edit {
    ($name:ident) => {
        impl $name {
            pub fn clear(self) {
                self.set_text("");
            }
            pub fn append_text(self, s: &str) {
                let t = self.text();
                self.set_text(&format!("{t}{s}"));
            }
            pub fn select_all(self) {
                if let Some(e) = el(self.0) {
                    if let Some(i) = e.dyn_ref::<HtmlInputElement>() {
                        i.select();
                    } else if let Some(t) = e.dyn_ref::<HtmlTextAreaElement>() {
                        t.select();
                    }
                }
            }
            pub fn read_only(self) -> bool {
                el(self.0).is_some_and(|e| e.has_attribute("readonly"))
            }
            pub fn set_read_only(self, on: bool) {
                if let Some(e) = el(self.0) {
                    if on {
                        let _ = e.set_attribute("readonly", "");
                    } else {
                        let _ = e.remove_attribute("readonly");
                    }
                }
            }
            pub fn set_max_length(self, n: i32) {
                if let Some(e) = el(self.0) {
                    let _ = e.set_attribute("maxlength", &n.to_string());
                }
            }
            /// Its words line by line.
            pub fn lines(self) -> Vec<String> {
                self.text().lines().map(str::to_string).collect()
            }
        }
    };
}

text_edit!(TextBox);
text_edit!(RichTextBox);
text_edit!(MaskedTextBox);

impl MaskedTextBox {
    /// What was typed, without the mask's own characters.
    pub fn value(self) -> String {
        self.text()
    }
    pub fn mask_completed(self) -> bool {
        !self.text().is_empty()
    }
}

fn select_of(h: HWND) -> Option<HtmlSelectElement> {
    el(h).and_then(|e| e.dyn_into().ok())
}

macro_rules! items {
    ($name:ident) => {
        impl $name {
            /// A line at the end: its place.
            pub fn items_add(self, s: &str) -> usize {
                let Some(sel) = select_of(self.0) else { return 0 };
                let o = make("option");
                o.set_text_content(Some(s));
                let _ = sel.append_child(&o);
                sel.length() as usize - 1
            }
            pub fn items_insert(self, at: usize, s: &str) {
                let Some(sel) = select_of(self.0) else { return };
                let o = make("option");
                o.set_text_content(Some(s));
                let before = sel.options().item(at as u32);
                let _ = sel.insert_before(&o, before.as_ref().map(|b| b.as_ref()));
            }
            pub fn items_remove_at(self, at: usize) {
                if let Some(sel) = select_of(self.0) {
                    sel.remove_with_index(at as i32);
                }
            }
            pub fn items_clear(self) {
                if let Some(sel) = select_of(self.0) {
                    sel.set_length(0);
                }
            }
            pub fn items_count(self) -> usize {
                select_of(self.0).map(|s| s.length() as usize).unwrap_or(0)
            }
            pub fn item(self, at: usize) -> String {
                select_of(self.0).and_then(|s| s.options().item(at as u32)).and_then(|o| o.text_content()).unwrap_or_default()
            }
            pub fn items(self) -> Vec<String> {
                (0..self.items_count()).map(|i| self.item(i)).collect()
            }
            pub fn selected_index(self) -> Option<usize> {
                select_of(self.0).map(|s| s.selected_index()).filter(|i| *i >= 0).map(|i| i as usize)
            }
            pub fn set_selected_index(self, i: Option<usize>) {
                if let Some(s) = select_of(self.0) {
                    s.set_selected_index(i.map(|i| i as i32).unwrap_or(-1));
                }
            }
            pub fn selected_item(self) -> Option<String> {
                self.selected_index().map(|i| self.item(i))
            }
            pub fn find_string_exact(self, s: &str) -> Option<usize> {
                (0..self.items_count()).find(|i| self.item(*i) == s)
            }
            pub fn find_string(self, prefix: &str) -> Option<usize> {
                let p = prefix.to_lowercase();
                (0..self.items_count()).find(|i| self.item(*i).to_lowercase().starts_with(&p))
            }
            pub fn begin_update(self) {}
            pub fn end_update(self) {}
        }
    };
}

items!(ComboBox);
items!(ListBox);
items!(CheckedListBox);

impl ListBox {
    /// Every chosen line - SelectedIndices.
    pub fn selected_indices(self) -> Vec<usize> {
        let Some(s) = select_of(self.0) else { return vec![] };
        let o = s.options();
        (0..o.length()).filter(|i| o.item(*i).and_then(|x| x.dyn_into::<web_sys::HtmlOptionElement>().ok()).is_some_and(|x| x.selected())).map(|i| i as usize).collect()
    }
}

fn body_of(h: HWND) -> Option<web_sys::Element> {
    el(h)?.query_selector("tbody").ok().flatten()
}

fn row_cells(row: &web_sys::Element) -> Vec<String> {
    let cells = row.children();
    (0..cells.length()).map(|i| cells.item(i).and_then(|c| c.text_content()).unwrap_or_default()).collect()
}

fn add_row(h: HWND, cells: &[&str], editable: bool) -> usize {
    let Some(body) = body_of(h) else { return 0 };
    let tr = make("tr");
    for c in cells {
        let td = make("td");
        td.set_text_content(Some(c));
        if editable {
            let _ = td.set_attribute("contenteditable", "true");
        }
        let _ = tr.append_child(&td);
    }
    let _ = body.append_child(&tr);
    body.children().length() as usize - 1
}

/// A bound grid filled from its table: each row tagged with its place (1, 2...), as Windows'
/// grid_fill does, so that saving knows which row each one was.
pub(crate) fn fill_tagged(h: HWND, rows: &[Vec<String>]) {
    if let Some(b) = body_of(h) {
        b.set_inner_html("");
    }
    for (i, r) in rows.iter().enumerate() {
        let cells: Vec<&str> = r.iter().map(String::as_str).collect();
        let at = add_row(h, &cells, true);
        if let Some(tr) = body_of(h).and_then(|b| b.children().item(at as u32)) {
            let _ = tr.set_attribute("data-tag", &(i + 1).to_string());
        }
    }
}

/// Every row with its tag: its place when the table was read, 0 for a row added since.
pub(crate) fn tagged_rows(h: HWND) -> Vec<(usize, Vec<String>)> {
    let Some(b) = body_of(h) else { return vec![] };
    let rows = b.children();
    (0..rows.length()).filter_map(|i| rows.item(i)).map(|r| (r.get_attribute("data-tag").and_then(|t| t.parse().ok()).unwrap_or(0), row_cells(&r))).collect()
}

/// A grid's own ways to add and take out rows, as on Windows: a double-click under the last row
/// adds one, Ctrl+Delete takes out the chosen one.
pub(crate) fn rows_editable(h: HWND) {
    let Some(e) = el(h) else { return };
    let add = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |ev: web_sys::MouseEvent| {
        let in_row = ev.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()).is_some_and(|t| t.closest("tr").ok().flatten().is_some());
        if !in_row {
            let n = Grid(h).rows().first().map(|r| r.len()).unwrap_or_else(|| el(h).and_then(|e| e.query_selector_all("th").ok()).map(|l| l.length() as usize).unwrap_or(1));
            let blank = vec![""; n.max(1)];
            let at = add_row(h, &blank, true);
            select_row(h, Some(at));
        }
    });
    let _ = e.add_event_listener_with_callback("dblclick", add.as_ref().unchecked_ref());
    add.forget();
    let del = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Delete" && ev.ctrl_key() {
            if let Some(r) = selected_row(h) {
                Grid(h).rows_remove_at(r);
                ev.prevent_default();
            }
        }
    });
    let _ = e.add_event_listener_with_callback("keydown", del.as_ref().unchecked_ref());
    del.forget();
}

fn cell_el(h: HWND, row: usize, col: usize) -> Option<web_sys::Element> {
    body_of(h)?.children().item(row as u32)?.children().item(col as u32)
}

fn selected_row(h: HWND) -> Option<usize> {
    let body = body_of(h)?;
    let rows = body.children();
    (0..rows.length()).find(|i| rows.item(*i).is_some_and(|r| r.class_list().contains("selected"))).map(|i| i as usize)
}

fn select_row(h: HWND, at: Option<usize>) {
    let Some(body) = body_of(h) else { return };
    let rows = body.children();
    for i in 0..rows.length() {
        if let Some(r) = rows.item(i) {
            r.class_list().toggle_with_force("selected", Some(i as usize) == at).ok();
        }
    }
}

/// A list's rows chosen with a click or a tap.
pub(crate) fn rows_choosable(h: HWND) {
    let Some(body) = body_of(h) else { return };
    let c = Closure::<dyn FnMut(web_sys::Event)>::new(move |ev: web_sys::Event| {
        let row = ev.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()).and_then(|t| t.closest("tr").ok().flatten());
        if let (Some(row), Some(body)) = (row, body_of(h)) {
            let rows = body.children();
            let at = (0..rows.length()).find(|i| rows.item(*i).as_ref() == Some(&row)).map(|i| i as usize);
            select_row(h, at);
        }
    });
    let _ = body.add_event_listener_with_callback("click", c.as_ref().unchecked_ref());
    c.forget();
}

impl ListView {
    pub fn items_add(self, cells: &[&str]) -> usize {
        add_row(self.0, cells, false)
    }
    pub fn items_remove_at(self, at: usize) {
        if let Some(r) = body_of(self.0).and_then(|b| b.children().item(at as u32)) {
            r.remove();
        }
    }
    pub fn items_clear(self) {
        if let Some(b) = body_of(self.0) {
            b.set_inner_html("");
        }
    }
    pub fn items_count(self) -> usize {
        body_of(self.0).map(|b| b.children().length() as usize).unwrap_or(0)
    }
    pub fn cell(self, row: usize, col: usize) -> String {
        cell_el(self.0, row, col).and_then(|c| c.text_content()).unwrap_or_default()
    }
    pub fn set_cell(self, row: usize, col: usize, text: &str) {
        if let Some(c) = cell_el(self.0, row, col) {
            c.set_text_content(Some(text));
        }
    }
    pub fn selected_index(self) -> Option<usize> {
        selected_row(self.0)
    }
    pub fn selected_indices(self) -> Vec<usize> {
        selected_row(self.0).into_iter().collect()
    }
    pub fn select(self, at: usize) {
        select_row(self.0, Some(at))
    }
    pub fn ensure_visible(self, at: usize) {
        if let Some(r) = body_of(self.0).and_then(|b| b.children().item(at as u32)) {
            r.scroll_into_view();
        }
    }
    pub fn begin_update(self) {}
    pub fn end_update(self) {}
}

impl Grid {
    /// Every row's cells.
    pub fn rows(self) -> Vec<Vec<String>> {
        let Some(b) = body_of(self.0) else { return vec![] };
        let rows = b.children();
        (0..rows.length()).filter_map(|i| rows.item(i)).map(|r| row_cells(&r)).collect()
    }
    pub fn rows_add(self, cells: &[&str]) {
        add_row(self.0, cells, true);
    }
    pub fn rows_clear(self) {
        if let Some(b) = body_of(self.0) {
            b.set_inner_html("");
        }
    }
    pub fn rows_count(self) -> usize {
        body_of(self.0).map(|b| b.children().length() as usize).unwrap_or(0)
    }
    /// Row `at` taken out.
    pub fn rows_remove_at(self, at: usize) {
        if let Some(r) = body_of(self.0).and_then(|b| b.children().item(at as u32)) {
            r.remove();
        }
    }
    pub fn fill(self, rows: &[Vec<String>]) {
        self.rows_clear();
        for r in rows {
            let cells: Vec<&str> = r.iter().map(String::as_str).collect();
            add_row(self.0, &cells, true);
        }
    }
    pub fn cell(self, row: usize, col: usize) -> String {
        cell_el(self.0, row, col).and_then(|c| c.text_content()).unwrap_or_default()
    }
    pub fn set_cell(self, row: usize, col: usize, text: &str) {
        if let Some(c) = cell_el(self.0, row, col) {
            c.set_text_content(Some(text));
        }
    }
    pub fn selected_row(self) -> Option<usize> {
        selected_row(self.0)
    }
    pub fn clear_selection(self) {
        select_row(self.0, None)
    }
    pub fn set_read_only(self, on: bool) {
        if let Some(b) = body_of(self.0) {
            if let Ok(cells) = b.query_selector_all("td") {
                for i in 0..cells.length() {
                    if let Some(c) = cells.item(i).and_then(|n| n.dyn_into::<web_sys::Element>().ok()) {
                        let _ = c.set_attribute("contenteditable", if on { "false" } else { "true" });
                    }
                }
            }
        }
    }
}

/// `YYYY-MM-DD` as numbers.
fn ymd(s: &str) -> (i32, i32, i32) {
    let mut p = s.split('-').map(|x| x.parse::<i32>().unwrap_or(0));
    (p.next().unwrap_or(0), p.next().unwrap_or(0), p.next().unwrap_or(0))
}

fn date_value(h: HWND) -> (i32, i32, i32) {
    let s = input_of(h).map(|i| i.value()).unwrap_or_default();
    if s.is_empty() {
        let d = js_sys::Date::new_0();
        return (d.get_full_year() as i32, d.get_month() as i32 + 1, d.get_date() as i32);
    }
    ymd(&s)
}

fn set_date_value(h: HWND, y: i32, m: i32, d: i32) {
    if let Some(i) = input_of(h) {
        i.set_value(&format!("{y:04}-{m:02}-{d:02}"));
    }
}

impl DateTimePicker {
    pub fn value(self) -> (i32, i32, i32) {
        date_value(self.0)
    }
    pub fn set_value(self, year: i32, month: i32, day: i32) {
        set_date_value(self.0, year, month, day)
    }
    pub fn checked(self) -> bool {
        input_of(self.0).is_some_and(|i| !i.value().is_empty())
    }
}

impl MonthCalendar {
    pub fn selection_start(self) -> (i32, i32, i32) {
        date_value(self.0)
    }
    pub fn set_date(self, y: i32, m: i32, d: i32) {
        set_date_value(self.0, y, m, d)
    }
}

fn number(h: HWND) -> i32 {
    input_of(h).and_then(|i| i.value().trim().parse::<f64>().ok()).map(|n| n.round() as i32).unwrap_or(0)
}

fn set_number(h: HWND, v: i32) {
    if let Some(i) = input_of(h) {
        i.set_value(&v.to_string());
    }
}

impl NumericUpDown {
    pub fn value(self) -> i32 {
        number(self.0)
    }
    pub fn set_value(self, v: i32) {
        set_number(self.0, v)
    }
    pub fn up_button(self) {
        self.set_value(self.value() + 1)
    }
    pub fn down_button(self) {
        self.set_value(self.value() - 1)
    }
}

impl TrackBar {
    pub fn value(self) -> i32 {
        number(self.0)
    }
    pub fn set_value(self, v: i32) {
        set_number(self.0, v)
    }
}

impl ProgressBar {
    pub fn value(self) -> i32 {
        el(self.0).and_then(|e| e.dyn_into::<web_sys::HtmlProgressElement>().ok()).map(|p| p.value() as i32).unwrap_or(0)
    }
    pub fn set_value(self, v: i32) {
        if let Some(p) = el(self.0).and_then(|e| e.dyn_into::<web_sys::HtmlProgressElement>().ok()) {
            p.set_value(v.clamp(0, p.max() as i32) as f64);
        }
    }
    pub fn increment(self, n: i32) {
        self.set_value(self.value() + n)
    }
    pub fn perform_step(self) {
        self.increment(10)
    }
}

impl WebBrowser {
    /// Opens a page in the frame.
    pub fn navigate(self, url: &str) {
        if let Some(e) = el(self.0) {
            let _ = e.set_attribute("src", url);
        }
    }
    /// The address the frame was given.
    pub fn url(self) -> String {
        el(self.0).and_then(|e| e.get_attribute("src")).unwrap_or_default()
    }
    /// Loads the page again.
    pub fn reload(self) {
        let at = self.url();
        self.navigate(&at);
    }
    /// A frame cannot go back on the program's behalf in a browser; it does nothing here.
    pub fn go_back(self) {}
    /// The browser the page runs in decides about certificates; nothing to do here.
    pub fn allow_certificate_errors_for(self, _host: &str) {}
    /// A frame is always there in a browser.
    pub fn ready(self) -> bool {
        true
    }
}

impl PictureBox {
    /// The picture at `path` - beside the page (`images/logo.png`) or a web address.
    pub fn load(self, path: &str) {
        if let Some(e) = el(self.0) {
            let _ = e.set_attribute("src", &path.replace('\\', "/"));
        }
    }
    pub fn set_size_mode(self, mode: &str) {
        if let Some(e) = el(self.0) {
            let fit = match mode {
                "stretch" => "fill",
                "normal" => "none",
                "center" => "none",
                _ => "contain",
            };
            let _ = e.style().set_property("object-fit", fit);
        }
    }
}

impl TabControl {
    pub fn selected_index(self) -> usize {
        el(self.0).and_then(|e| e.get_attribute("data-selected")).and_then(|s| s.parse().ok()).unwrap_or(0)
    }
    pub fn select_tab(self, i: usize) {
        crate::form::select_tab(self.0, HWND(0), i)
    }
}

impl StatusBar {
    /// The words in part `part` (from 0).
    pub fn set_part_text(self, part: usize, text: &str) {
        let Some(e) = el(self.0) else { return };
        while (e.children().length() as usize) <= part {
            let _ = e.append_child(&make("span"));
        }
        if let Some(p) = e.children().item(part as u32) {
            p.set_text_content(Some(text));
        }
    }
    pub fn set_parts(self, _widths: &[i32]) {}
}

impl LinkLabel {
    pub fn link_text(self) -> String {
        self.text()
    }
    pub fn set_link_text(self, s: &str) {
        self.set_text(s)
    }
}

type Tick = Box<dyn FnMut()>;

struct TimerState {
    ms: u32,
    running: Option<i32>,
    tick: std::rc::Rc<RefCell<Tick>>,
}

thread_local! {
    static TIMERS: RefCell<Vec<TimerState>> = const { RefCell::new(Vec::new()) };
}

/// A timer - WinForms' Timer: its tick every so many milliseconds while it runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Timer(pub usize);

impl Timer {
    pub(crate) fn make(ms: u32, on: bool, tick: Tick) -> Timer {
        let t = TIMERS.with(|l| {
            let mut l = l.borrow_mut();
            l.push(TimerState { ms: ms.max(1), running: None, tick: std::rc::Rc::new(RefCell::new(tick)) });
            Timer(l.len() - 1)
        });
        if on {
            t.start();
        }
        t
    }
    pub fn start(self) {
        if self.enabled() {
            return;
        }
        let Some((ms, tick)) = TIMERS.with(|l| l.borrow().get(self.0).map(|s| (s.ms, s.tick.clone()))) else { return };
        let c = Closure::<dyn FnMut()>::new(move || {
            if let Ok(mut f) = tick.try_borrow_mut() {
                f();
            }
        });
        let id = crate::win::window().set_interval_with_callback_and_timeout_and_arguments_0(c.as_ref().unchecked_ref(), ms as i32).unwrap_or(0);
        c.forget();
        TIMERS.with(|l| {
            if let Some(s) = l.borrow_mut().get_mut(self.0) {
                s.running = Some(id);
            }
        });
    }
    pub fn stop(self) {
        let id = TIMERS.with(|l| l.borrow_mut().get_mut(self.0).and_then(|s| s.running.take()));
        if let Some(id) = id {
            crate::win::window().clear_interval_with_handle(id);
        }
    }
    pub fn enabled(self) -> bool {
        TIMERS.with(|l| l.borrow().get(self.0).is_some_and(|s| s.running.is_some()))
    }
    pub fn set_enabled(self, on: bool) {
        if on {
            self.start()
        } else {
            self.stop()
        }
    }
    pub fn interval(self) -> u32 {
        TIMERS.with(|l| l.borrow().get(self.0).map(|s| s.ms).unwrap_or(0))
    }
    pub fn set_interval(self, ms: u32) {
        let was = self.enabled();
        self.stop();
        TIMERS.with(|l| {
            if let Some(s) = l.borrow_mut().get_mut(self.0) {
                s.ms = ms.max(1);
            }
        });
        if was {
            self.start();
        }
    }
}
