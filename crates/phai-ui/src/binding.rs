//! Data binding - WinForms' BindingSource and a control's DataBindings. A binding source holds
//! rows of named columns and which row is current; each bound control shows one column of the
//! current row - a box its words, a tick box whether it is ticked, a number box its number, a
//! date picker its date - and what a person changes there goes back into the row. Moving to
//! another row (`move_next`, `set_position`...) puts what the controls hold into the row they
//! showed, then shows the new one, and raises PositionChanged.
//!
//! A source bound to a database table (`bind = { table, columns, keys }` in the form) is filled
//! by the form's `reload_<id>()` and written back by `save_<id>()`; otherwise the program gives
//! it rows itself ([`BindingSource::set_rows`]).

use std::cell::RefCell;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::ctl;
use crate::events;
use crate::win;

/// Which of a control's values a column is bound to - the property name in WinForms'
/// `DataBindings.Add("Text", ...)`, chosen by the kind of control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
    /// Its words: a text box, a label, a button.
    Text,
    /// Ticked or not: a check box, a radio. `true` / `false` in the row.
    Checked,
    /// A NumericUpDown's number.
    Number,
    /// A TrackBar's or ProgressBar's value.
    Value,
    /// A DateTimePicker's date, `YYYY-MM-DD` in the row.
    Date,
    /// A MonthCalendar's day, `YYYY-MM-DD`.
    Calendar,
    /// A drop-down's item with these words chosen - typed in when it takes words.
    Combo,
    /// A ListBox's line with these words chosen.
    ListBox,
}

#[derive(Debug, Default)]
struct Inner {
    columns: Vec<String>,
    /// Each row with its place in what was loaded (1, 2, ...; 0 for a row added since).
    rows: Vec<(usize, Vec<String>)>,
    position: usize,
    bound: Vec<(HWND, Bound, usize)>,
}

/// WinForms' BindingSource: rows, the current one, and the controls that show it.
#[derive(Debug)]
pub struct BindingSource {
    pub owner: HWND,
    /// Its id on the form: PositionChanged comes to the form with it.
    pub id: i32,
    inner: RefCell<Inner>,
}

fn truthy(v: &str) -> bool {
    matches!(v.trim().to_ascii_lowercase().as_str(), "true" | "1" | "-1" | "yes" | "y" | "t" | "on")
}

/// `YYYY-MM-DD` (or with `/`, and anything after the day, as a database's time) as numbers.
fn parse_date(v: &str) -> Option<(i32, i32, i32)> {
    let v = v.trim();
    let mut it = v.get(..10.min(v.len()))?.split(['-', '/']).map(|p| p.parse::<i32>().ok());
    let (y, m, d) = (it.next()??, it.next()??, it.next()??);
    ((1601..=30827).contains(&y) && (1..=12).contains(&m) && (1..=31).contains(&d)).then_some((y, m, d))
}

fn date_text((y, m, d): (i32, i32, i32)) -> String {
    format!("{y:04}-{m:02}-{d:02}")
}

/// What control `h` holds, as the row keeps it.
pub fn read(h: HWND, how: Bound) -> String {
    match how {
        Bound::Text => win::text_of(h),
        Bound::Checked => win::is_checked(h).to_string(),
        Bound::Number | Bound::Value => win::value(h).to_string(),
        Bound::Date => date_text(win::date_value(h)),
        Bound::Calendar => date_text(crate::extra::calendar_date(h)),
        Bound::Combo => ctl::ComboBox(h).text(),
        Bound::ListBox => crate::extra::list_box_selected(h).map(|i| crate::extra::list_box_item(h, i)).unwrap_or_default(),
    }
}

/// Control `h` showing `v`.
pub fn write(h: HWND, how: Bound, v: &str) {
    match how {
        Bound::Text => win::set_text(h, v),
        Bound::Checked => win::set_checked(h, truthy(v)),
        Bound::Number => ctl::NumericUpDown(h).set_value(v.trim().parse::<f64>().map(|n| n.round() as i32).unwrap_or(0)),
        Bound::Value => win::set_value(h, v.trim().parse::<f64>().map(|n| n.round() as i32).unwrap_or(0)),
        Bound::Date => {
            if let Some((y, m, d)) = parse_date(v) {
                win::set_date(h, y, m, d);
            }
        }
        Bound::Calendar => {
            if let Some((y, m, d)) = parse_date(v) {
                crate::extra::calendar_set_date(h, y, m, d);
            }
        }
        Bound::Combo => {
            let w = win::wide(v);
            // CB_FINDSTRINGEXACT, then CB_SETCURSEL: the item with these words, or none.
            let i = unsafe { wm::SendMessageW(h, 0x0158, Some(WPARAM(usize::MAX)), Some(LPARAM(w.as_ptr() as isize))).0 };
            unsafe {
                wm::SendMessageW(h, 0x014E, Some(WPARAM(if i >= 0 { i as usize } else { usize::MAX })), None);
            }
            if i < 0 && !v.is_empty() {
                // A drop-down that takes words shows them as they are.
                win::set_text(h, v);
            }
        }
        Bound::ListBox => {
            let n = crate::extra::list_box_count(h);
            let at = (0..n).find(|i| crate::extra::list_box_item(h, *i) == v);
            crate::extra::list_box_select(h, at);
        }
    }
}

impl BindingSource {
    /// A source on `owner` with `id`, its rows having `columns`.
    pub fn new(owner: HWND, id: i32, columns: &[&str]) -> BindingSource {
        BindingSource { owner, id, inner: RefCell::new(Inner { columns: columns.iter().map(|c| c.trim().to_string()).filter(|c| !c.is_empty()).collect(), ..Default::default() }) }
    }

    /// Control `control` shows column `column` of the current row, its `how` - WinForms'
    /// `control.DataBindings.Add(...)`. A column it does not have yet is added.
    pub fn bind(&self, control: impl Into<HWND>, how: Bound, column: &str) {
        let h = control.into();
        {
            let mut s = self.inner.borrow_mut();
            let col = match s.columns.iter().position(|c| c == column) {
                Some(i) => i,
                None => {
                    s.columns.push(column.to_string());
                    for r in s.rows.iter_mut() {
                        r.1.push(String::new());
                    }
                    s.columns.len() - 1
                }
            };
            s.bound.retain(|b| b.0 != h);
            s.bound.push((h, how, col));
        }
        self.reset_bindings();
    }

    /// The bindings of control `control` taken away - `DataBindings.Clear()`.
    pub fn unbind(&self, control: impl Into<HWND>) {
        let h = control.into();
        self.inner.borrow_mut().bound.retain(|b| b.0 != h);
    }

    pub fn columns(&self) -> Vec<String> {
        self.inner.borrow().columns.clone()
    }

    fn column(&self, name: &str) -> Option<usize> {
        self.inner.borrow().columns.iter().position(|c| c == name)
    }

    /// What the bound controls hold put into the current row - `EndEdit()`. Moving, reading
    /// rows and saving do it first by themselves.
    pub fn end_edit(&self) {
        let mut s = self.inner.borrow_mut();
        let at = s.position;
        let n = s.columns.len();
        let values: Vec<(usize, String)> = s.bound.iter().filter(|b| unsafe { wm::IsWindow(Some(b.0)) }.as_bool()).map(|(h, how, col)| (*col, read(*h, *how))).collect();
        let checked: Vec<usize> = s.bound.iter().filter(|b| b.1 == Bound::Checked).map(|b| b.2).collect();
        if let Some(row) = s.rows.get_mut(at) {
            row.1.resize(n, String::new());
            for (col, v) in values {
                // A tick as the column keeps it: `true`/`false` where it held words, 1/0 as a
                // database's number otherwise.
                let v = if checked.contains(&col) && !matches!(row.1[col].trim().to_ascii_lowercase().as_str(), "true" | "false") { if v == "true" { "1".into() } else { "0".into() } } else { v };
                row.1[col] = v;
            }
        }
    }

    /// The current row shown in the bound controls again - `ResetBindings()`. With no rows
    /// they are emptied.
    pub fn reset_bindings(&self) {
        let (bound, row) = {
            let s = self.inner.borrow();
            (s.bound.clone(), s.rows.get(s.position).map(|r| r.1.clone()))
        };
        for (h, how, col) in bound {
            let v = row.as_ref().and_then(|r| r.get(col)).map(String::as_str).unwrap_or("");
            write(h, how, v);
        }
    }

    /// Every row as it is now, the controls' changes in it - the List.
    pub fn rows(&self) -> Vec<Vec<String>> {
        self.end_edit();
        self.inner.borrow().rows.iter().map(|r| r.1.clone()).collect()
    }

    /// Every row with its place in what was loaded (0 for one added since): what a table's
    /// binding saves.
    pub fn tagged_rows(&self) -> Vec<(usize, Vec<String>)> {
        self.end_edit();
        self.inner.borrow().rows.clone()
    }

    /// New rows, each in the columns' order: the first one shown - `DataSource = ...`.
    pub fn set_rows(&self, rows: Vec<Vec<String>>) {
        {
            let mut s = self.inner.borrow_mut();
            let n = s.columns.len();
            s.rows = rows.into_iter().enumerate().map(|(i, mut r)| {
                r.resize(n.max(r.len()), String::new());
                (i + 1, r)
            }).collect();
            s.position = 0;
        }
        self.reset_bindings();
        self.changed();
    }

    /// How many rows - Count.
    pub fn count(&self) -> usize {
        self.inner.borrow().rows.len()
    }

    /// The current row's place, from 0 - Position.
    pub fn position(&self) -> usize {
        self.inner.borrow().position
    }

    /// Row `i` made current: the controls' changes kept in the row they showed, the new one shown,
    /// PositionChanged raised. Past the end: the last row.
    pub fn set_position(&self, i: usize) {
        let to = i.min(self.count().saturating_sub(1));
        if to == self.position() {
            return;
        }
        self.end_edit();
        self.inner.borrow_mut().position = to;
        self.reset_bindings();
        self.changed();
    }

    pub fn move_first(&self) {
        self.set_position(0);
    }
    pub fn move_previous(&self) {
        self.set_position(self.position().saturating_sub(1));
    }
    pub fn move_next(&self) {
        self.set_position(self.position() + 1);
    }
    pub fn move_last(&self) {
        self.set_position(self.count().saturating_sub(1));
    }

    /// The current row, the controls' changes in it - Current.
    pub fn current(&self) -> Option<Vec<String>> {
        self.end_edit();
        let s = self.inner.borrow();
        s.rows.get(s.position).map(|r| r.1.clone())
    }

    /// Column `column` of the current row; empty when there is none.
    pub fn value(&self, column: &str) -> String {
        let i = self.column(column);
        self.current().and_then(|r| i.and_then(|i| r.get(i).cloned())).unwrap_or_default()
    }

    /// Column `column` of the current row set, and shown in the controls bound to it.
    pub fn set_value(&self, column: &str, v: &str) {
        let Some(i) = self.column(column) else { return };
        self.end_edit();
        {
            let mut s = self.inner.borrow_mut();
            let at = s.position;
            if let Some(r) = s.rows.get_mut(at) {
                if r.1.len() <= i {
                    r.1.resize(i + 1, String::new());
                }
                r.1[i] = v.to_string();
            }
        }
        self.reset_bindings();
    }

    /// An empty row at the end, made current - `AddNew()`. Its place.
    pub fn add_new(&self) -> usize {
        self.end_edit();
        let at = {
            let mut s = self.inner.borrow_mut();
            let n = s.columns.len();
            s.rows.push((0, vec![String::new(); n]));
            s.position = s.rows.len() - 1;
            s.position
        };
        self.reset_bindings();
        self.changed();
        at
    }

    /// The current row taken out - `RemoveCurrent()`; the next one (or the one before) shown.
    pub fn remove_current(&self) {
        {
            let mut s = self.inner.borrow_mut();
            let at = s.position;
            if at < s.rows.len() {
                s.rows.remove(at);
            }
            s.position = at.min(s.rows.len().saturating_sub(1));
        }
        self.reset_bindings();
        self.changed();
    }

    fn changed(&self) {
        if !self.owner.is_invalid() {
            events::raise(self.owner, self.id, events::POSITION_CHANGED, &());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_and_ticks_read_as_a_database_writes_them() {
        assert_eq!(parse_date("2026-11-20"), Some((2026, 11, 20)));
        assert_eq!(parse_date("2026/1/5"), Some((2026, 1, 5)));
        assert_eq!(parse_date("2026-13-05"), None);
        assert_eq!(parse_date("2026-01-05 13:45:00"), Some((2026, 1, 5)));
        assert_eq!(parse_date(""), None);
        assert_eq!(date_text((2026, 1, 5)), "2026-01-05");
        assert!(truthy("1") && truthy("TRUE") && truthy("yes") && !truthy("0") && !truthy("false") && !truthy(""));
    }

    #[test]
    fn rows_move_and_change_without_controls() {
        let s = BindingSource::new(HWND::default(), 1, &["name", "phone", ""]);
        assert_eq!(s.columns(), vec!["name", "phone"]);
        s.set_rows(vec![vec!["Somchai".into(), "081".into()], vec!["Malee".into()]]);
        assert_eq!(s.count(), 2);
        assert_eq!(s.rows()[1], vec!["Malee", ""]);
        s.move_last();
        assert_eq!(s.value("name"), "Malee");
        s.set_value("phone", "02");
        s.move_next();
        assert_eq!(s.position(), 1);
        s.move_first();
        assert_eq!(s.value("name"), "Somchai");
        s.add_new();
        s.set_value("name", "New");
        assert_eq!(s.tagged_rows().iter().map(|r| r.0).collect::<Vec<_>>(), vec![1, 2, 0]);
        s.set_position(1);
        s.remove_current();
        assert_eq!(s.rows(), vec![vec!["Somchai".to_string(), "081".into()], vec!["New".into(), "".into()]]);
        assert_eq!(s.position(), 1);
        assert_eq!(s.value("nothing"), "");
    }
}
