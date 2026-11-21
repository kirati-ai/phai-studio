//! A grid of cells to read and change, as WinForms' DataGridView, and a rich text box, as its
//! RichTextBox. The grid is Windows' list view with a box that opens over a cell on a
//! double-click (or F2, or typing); Enter keeps what is typed, Escape leaves the cell as it was.
//! A double-click under the last row adds a row. Its parent hears [`crate::win::VALUE_CHANGED`].
//! Each column has its own properties, as DevExpress's GridColumn ([`Column`]: shown or hidden,
//! aligned, read only, edited as text, a number, a date, a check box or a drop-down, sorted by a
//! click on its heading, sized or not); the grid its own ([`set_allow_add_rows`] and the rest).
//! The rich text box is Windows' own RichEdit (Msftedit.dll).

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::System::LibraryLoader::LoadLibraryW;
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::win;

const EDITOR_ID: i32 = 0x7E01;

/// How a column's cells are edited - DevExpress's ColumnEdit.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Editor {
    #[default]
    Text,
    /// Digits, a sign and a point; anything else is not kept.
    Number,
    /// A calendar that drops down; kept as `2026-11-21`.
    Date,
    /// A tick, drawn in the cell; a click turns it on and off (`1`/`0`, `true`/`false`...).
    Check,
    /// A drop-down of these, typing allowed.
    Combo(Vec<String>),
}

/// Where a column's words sit.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

/// A column's own properties, as DevExpress's GridColumn has them.
#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    /// Shown; a hidden column keeps its cells (a key the program needs, say).
    pub visible: bool,
    pub align: Align,
    /// Its cells read, not changed.
    pub read_only: bool,
    pub editor: Editor,
    /// A click on its heading sorts the rows by it.
    pub allow_sort: bool,
    /// Its edge can be dragged to size it.
    pub allow_size: bool,
    /// Sorted by it when the grid is filled: up (`Some(true)`) or down.
    pub sort: Option<bool>,
}

impl Default for Column {
    fn default() -> Column {
        Column { visible: true, align: Align::Left, read_only: false, editor: Editor::Text, allow_sort: true, allow_size: true, sort: None }
    }
}

impl Column {
    pub fn new() -> Column {
        Column::default()
    }
}

/// When a cell's box opens - DevExpress's EditorShowMode.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum EditOn {
    #[default]
    DoubleClick,
    Click,
    /// F2 (or Enter on a row) only.
    F2,
}

/// A grid's own properties, and what it remembers.
struct Opts {
    cols: Vec<Column>,
    /// Hidden columns' widths, to come back to.
    widths: Vec<i32>,
    add: bool,
    delete: bool,
    edit_on: EditOn,
    enter_next: bool,
    auto_width: bool,
    /// The column the rows are sorted by, and whether up.
    sorted: Option<(usize, bool)>,
    /// The column last clicked: F2 opens it.
    focus_col: usize,
}

impl Default for Opts {
    fn default() -> Opts {
        Opts { cols: vec![], widths: vec![], add: true, delete: true, edit_on: EditOn::DoubleClick, enter_next: false, auto_width: false, sorted: None, focus_col: 0 }
    }
}

static OPTS: std::sync::Mutex<Option<std::collections::HashMap<isize, Opts>>> = std::sync::Mutex::new(None);

fn with_opts<R>(h: HWND, f: impl FnOnce(&mut Opts) -> R) -> R {
    let mut m = OPTS.lock().unwrap_or_else(|e| e.into_inner());
    f(m.get_or_insert_with(Default::default).entry(h.0 as isize).or_default())
}

/// Column `i`'s properties (the defaults when none were given).
pub fn column(h: HWND, i: usize) -> Column {
    with_opts(h, |o| o.cols.get(i).cloned().unwrap_or_default())
}

/// Gives column `i` its properties: shown or hidden, aligned, sorted by now if it says so.
pub fn set_column(h: HWND, i: usize, c: Column) {
    let was = column(h, i);
    with_opts(h, |o| {
        if o.cols.len() <= i {
            o.cols.resize(i + 1, Column::default());
        }
        o.cols[i] = c.clone();
        if o.widths.len() <= i {
            o.widths.resize(i + 1, 0);
        }
    });
    unsafe {
        // Alignment: the first column of a Windows list is always at the left.
        let fmt = match c.align {
            Align::Left => cc::LVCFMT_LEFT,
            Align::Right => cc::LVCFMT_RIGHT,
            Align::Center => cc::LVCFMT_CENTER,
        };
        let col = cc::LVCOLUMNW { mask: cc::LVCF_FMT, fmt, ..Default::default() };
        wm::SendMessageW(h, cc::LVM_SETCOLUMNW, Some(WPARAM(i)), Some(LPARAM(&col as *const _ as isize)));
        let width = wm::SendMessageW(h, cc::LVM_GETCOLUMNWIDTH, Some(WPARAM(i)), None).0 as i32;
        if !c.visible && was.visible {
            with_opts(h, |o| o.widths[i] = width);
            wm::SendMessageW(h, cc::LVM_SETCOLUMNWIDTH, Some(WPARAM(i)), Some(LPARAM(0)));
        } else if c.visible && !was.visible {
            let back = with_opts(h, |o| o.widths[i]);
            wm::SendMessageW(h, cc::LVM_SETCOLUMNWIDTH, Some(WPARAM(i)), Some(LPARAM(if back > 0 { back } else { win::px(h, 100) } as isize)));
        }
    }
    if let Some(up) = c.sort {
        sort_by(h, i, up);
    }
    unsafe {
        let _ = windows::Win32::Graphics::Gdi::InvalidateRect(Some(h), None, true);
    }
}

/// AllowAddRows: a double-click under the last row adds one.
pub fn set_allow_add_rows(h: HWND, on: bool) {
    with_opts(h, |o| o.add = on);
}

/// AllowDeleteRows: Delete takes the chosen row out.
pub fn set_allow_delete_rows(h: HWND, on: bool) {
    with_opts(h, |o| o.delete = on);
}

/// EditorShowMode: what opens a cell's box.
pub fn set_edit_on(h: HWND, on: EditOn) {
    with_opts(h, |o| o.edit_on = on);
}

/// EnterMoveNextColumn: Enter keeps the cell and opens the next one.
pub fn set_enter_moves_next(h: HWND, on: bool) {
    with_opts(h, |o| o.enter_next = on);
}

/// ColumnAutoWidth: the shown columns share the grid's width - no scrolling sideways.
pub fn set_auto_width(h: HWND, on: bool) {
    with_opts(h, |o| o.auto_width = on);
    if on {
        fit_columns(h);
    }
}

/// Whether any column is drawn as check boxes - the list's custom draw asks.
pub fn has_check_columns(h: HWND) -> bool {
    let m = OPTS.lock().unwrap_or_else(|e| e.into_inner());
    m.as_ref().and_then(|m| m.get(&(h.0 as isize))).is_some_and(|o| o.cols.iter().any(|c| c.editor == Editor::Check))
}

/// Whether a cell's words say yes: `1`, `true`, `yes`, `y`, `on`, `✓`.
pub fn truthy(s: &str) -> bool {
    matches!(s.trim().to_lowercase().as_str(), "1" | "true" | "yes" | "y" | "t" | "on" | "✓")
}

/// The other answer, in the same way of writing it.
fn toggled(s: &str) -> String {
    let t = s.trim();
    let pairs = [("1", "0"), ("true", "false"), ("yes", "no"), ("y", "n"), ("t", "f"), ("on", "off"), ("✓", "")];
    for (yes, no) in pairs {
        let same_case = |w: &str| if t.chars().next().is_some_and(char::is_uppercase) { if t.len() > 1 && t.chars().nth(1).is_some_and(char::is_uppercase) { w.to_uppercase() } else { let mut c = w.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() } } else { w.to_string() };
        if t.eq_ignore_ascii_case(yes) {
            return same_case(no);
        }
        if !no.is_empty() && t.eq_ignore_ascii_case(no) {
            return same_case(yes);
        }
    }
    if t.is_empty() { "1".into() } else { "0".into() }
}

/// A check-box cell drawn by the list's custom draw: its background and a tick box in the middle.
/// False for any other cell.
pub fn draw_check_cell(h: HWND, dc: windows::Win32::Graphics::Gdi::HDC, row: usize, col: usize, bg: crate::theme::Color, fg: crate::theme::Color) -> bool {
    if column(h, col).editor != Editor::Check {
        return false;
    }
    let mut r = RECT { top: col as i32, left: 0 /* LVIR_BOUNDS */, ..Default::default() };
    unsafe {
        wm::SendMessageW(h, cc::LVM_GETSUBITEMRECT, Some(WPARAM(row)), Some(LPARAM(&mut r as *mut _ as isize)));
    }
    if col == 0 {
        let w0 = unsafe { wm::SendMessageW(h, cc::LVM_GETCOLUMNWIDTH, Some(WPARAM(0)), None).0 } as i32;
        r.right = r.left + w0;
    }
    // Inside the grid lines.
    let inner = RECT { right: r.right - 1, bottom: r.bottom - 1, ..r };
    win::fill(dc, &inner, bg);
    let size = win::px(h, 16).min(r.bottom - r.top);
    let glyph = if truthy(&cell(h, row, col)) { 0xE73A /* CheckboxComposite */ } else { 0xE739 /* Checkbox */ };
    if let Some(i) = crate::icon::glyph_icon(glyph, size, fg) {
        let x = r.left + ((r.right - r.left) - size) / 2;
        let y = r.top + ((r.bottom - r.top) - size) / 2;
        unsafe {
            let _ = wm::DrawIconEx(dc, x, y, i, size, size, 0, None, wm::DI_NORMAL);
            let _ = wm::DestroyIcon(i);
        }
    }
    true
}

/// The rows sorted by column `col`, up or down - numbers as numbers, words without regard to
/// case - each row keeping its tag; the heading shows the arrow.
pub fn sort_by(h: HWND, col: usize, up: bool) {
    close_editor(h, true);
    let mut rows = grid_tagged_rows(h);
    let key = |r: &Vec<String>| r.get(col).cloned().unwrap_or_default();
    rows.sort_by(|(_, a), (_, b)| {
        let (a, b) = (key(a), key(b));
        let o = match (a.trim().replace(',', "").parse::<f64>(), b.trim().replace(',', "").parse::<f64>()) {
            (Ok(x), Ok(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
            _ => a.to_lowercase().cmp(&b.to_lowercase()),
        };
        if up { o } else { o.reverse() }
    });
    unsafe {
        wm::SendMessageW(h, wm::WM_SETREDRAW, Some(WPARAM(0)), None);
    }
    grid_clear(h);
    for (i, (t, r)) in rows.iter().enumerate() {
        let cells: Vec<&str> = r.iter().map(String::as_str).collect();
        grid_add(h, &cells);
        set_tag(h, i, *t);
    }
    unsafe {
        wm::SendMessageW(h, wm::WM_SETREDRAW, Some(WPARAM(1)), None);
        let _ = windows::Win32::Graphics::Gdi::InvalidateRect(Some(h), None, true);
    }
    with_opts(h, |o| o.sorted = Some((col, up)));
    // The arrow on the heading sorted by; none on the others.
    unsafe {
        let header = HWND(wm::SendMessageW(h, cc::LVM_GETHEADER, None, None).0 as *mut _);
        for i in 0..columns(h) {
            let mut it = cc::HDITEMW { mask: cc::HDI_FORMAT, ..Default::default() };
            wm::SendMessageW(header, cc::HDM_GETITEMW, Some(WPARAM(i)), Some(LPARAM(&mut it as *mut _ as isize)));
            let mut fmt = it.fmt.0 & !(cc::HDF_SORTUP.0 | cc::HDF_SORTDOWN.0);
            if i == col {
                fmt |= if up { cc::HDF_SORTUP.0 } else { cc::HDF_SORTDOWN.0 };
            }
            it.fmt = cc::HEADER_CONTROL_FORMAT_FLAGS(fmt);
            wm::SendMessageW(header, cc::HDM_SETITEMW, Some(WPARAM(i)), Some(LPARAM(&it as *const _ as isize)));
        }
    }
}

/// The shown columns made to share the grid's width, each keeping its share.
fn fit_columns(h: HWND) {
    let mut cr = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut cr);
    }
    let n = columns(h);
    let widths: Vec<i32> = (0..n).map(|i| unsafe { wm::SendMessageW(h, cc::LVM_GETCOLUMNWIDTH, Some(WPARAM(i)), None).0 as i32 }).collect();
    let total: i32 = widths.iter().sum();
    if total <= 0 || cr.right <= 0 || (total - cr.right).abs() <= 1 {
        return;
    }
    let mut used = 0;
    let last = (0..n).rev().find(|i| widths[*i] > 0);
    for (i, w) in widths.iter().enumerate() {
        if *w == 0 {
            continue;
        }
        let nw = if Some(i) == last { cr.right - used } else { (*w as i64 * cr.right as i64 / total as i64) as i32 };
        used += nw;
        unsafe {
            wm::SendMessageW(h, cc::LVM_SETCOLUMNWIDTH, Some(WPARAM(i)), Some(LPARAM(nw.max(1) as isize)));
        }
    }
}

/// The first column after `from` (or `from` itself when `this`) whose cells a box edits.
fn editable_from(h: HWND, from: usize, this: bool) -> Option<usize> {
    let n = columns(h);
    let start = if this { from } else { from + 1 };
    (start..n).find(|i| {
        let c = column(h, *i);
        c.visible && !c.read_only && c.editor != Editor::Check
    })
}

fn tell_changed(h: HWND) {
    unsafe {
        let id = wm::GetDlgCtrlID(h) as usize;
        if let Ok(p) = wm::GetParent(h) {
            wm::SendMessageW(p, wm::WM_COMMAND, Some(WPARAM(((win::VALUE_CHANGED as usize) << 16) | (id & 0xFFFF))), Some(LPARAM(h.0 as isize)));
        }
    }
}

/// A grid with these columns (heading, width in design pixels); no rows yet.
pub fn grid(parent: HWND, id: i32, f: windows::Win32::Graphics::Gdi::HFONT, columns: &[(&str, i32)]) -> HWND {
    let h = win::list(parent, id, f, columns);
    // A single border, as a DataGridView has.
    win::list_framed(h);
    const LVS_EX_GRIDLINES: isize = 0x1;
    const LVS_EX_FULLROWSELECT: isize = 0x20;
    const LVS_EX_DOUBLEBUFFER: isize = 0x10000;
    unsafe {
        wm::SendMessageW(h, cc::LVM_SETEXTENDEDLISTVIEWSTYLE, Some(WPARAM(0)), Some(LPARAM(LVS_EX_GRIDLINES | LVS_EX_FULLROWSELECT | LVS_EX_DOUBLEBUFFER)));
        let _ = SetWindowSubclass(h, Some(grid_proc), 7, 0);
    }
    h
}

/// The grid's cells, row by row.
pub fn grid_rows(h: impl Into<HWND>) -> Vec<Vec<String>> {
    let h: HWND = h.into();
    unsafe {
        let rows = wm::SendMessageW(h, cc::LVM_GETITEMCOUNT, None, None).0 as usize;
        let header = wm::SendMessageW(h, cc::LVM_GETHEADER, None, None).0;
        let cols = wm::SendMessageW(HWND(header as *mut _), cc::HDM_GETITEMCOUNT, None, None).0.max(0) as usize;
        (0..rows).map(|r| (0..cols).map(|c| cell(h, r, c)).collect()).collect()
    }
}

/// Adds a row at the end.
pub fn grid_add(h: impl Into<HWND>, cells: &[&str]) {
    let h: HWND = h.into();
    win::list_add(h, cells);
}

/// Empties the grid.
pub fn grid_clear(h: impl Into<HWND>) {
    let h: HWND = h.into();
    unsafe {
        wm::SendMessageW(h, cc::LVM_DELETEALLITEMS, None, None);
    }
}

/// A row's tag: a number the grid keeps with the row, whatever moves around it (rows taken out
/// above it, or added). A new row's is 0.
fn tag(h: HWND, row: usize) -> usize {
    let mut item = cc::LVITEMW { mask: cc::LVIF_PARAM, iItem: row as i32, ..Default::default() };
    unsafe {
        wm::SendMessageW(h, cc::LVM_GETITEMW, None, Some(LPARAM(&mut item as *mut _ as isize)));
    }
    item.lParam.0.max(0) as usize
}

fn set_tag(h: HWND, row: usize, value: usize) {
    let item = cc::LVITEMW { mask: cc::LVIF_PARAM, iItem: row as i32, lParam: LPARAM(value as isize), ..Default::default() };
    unsafe {
        wm::SendMessageW(h, cc::LVM_SETITEMW, None, Some(LPARAM(&item as *const _ as isize)));
    }
}

/// The grid emptied and filled with `rows` as a table gave them: each row tagged with its place
/// (1, 2, ...), so that after the person changes, adds and deletes rows, [`grid_tagged_rows`] still
/// says which row each one was - what a data binding needs to save.
pub fn grid_fill(h: impl Into<HWND>, rows: &[Vec<String>]) {
    let h: HWND = h.into();
    grid_clear(h);
    for (i, r) in rows.iter().enumerate() {
        let cells: Vec<&str> = r.iter().map(String::as_str).collect();
        grid_add(h, &cells);
        set_tag(h, i, i + 1);
    }
    // Sorted as it was.
    if let Some((col, up)) = with_opts(h, |o| o.sorted) {
        sort_by(h, col, up);
    }
}

/// The grid's rows with their tags: the place each had when [`grid_fill`] put it there, 0 for a row
/// added since.
pub fn grid_tagged_rows(h: impl Into<HWND>) -> Vec<(usize, Vec<String>)> {
    let h: HWND = h.into();
    grid_rows(h).into_iter().enumerate().map(|(i, r)| (tag(h, i), r)).collect()
}

/// One cell's text.
pub fn cell(h: impl Into<HWND>, row: usize, col: usize) -> String {
    let h: HWND = h.into();
    let mut buf = vec![0u16; 1024];
    let mut item = cc::LVITEMW { iSubItem: col as i32, pszText: windows::core::PWSTR(buf.as_mut_ptr()), cchTextMax: 1024, ..Default::default() };
    let n = unsafe { wm::SendMessageW(h, cc::LVM_GETITEMTEXTW, Some(WPARAM(row)), Some(LPARAM(&mut item as *mut _ as isize))).0 } as usize;
    String::from_utf16_lossy(&buf[..n.min(1024)])
}

/// Sets one cell's text.
pub fn set_cell(h: impl Into<HWND>, row: usize, col: usize, text: &str) {
    let h: HWND = h.into();
    let mut t = win::wide(text);
    let item = cc::LVITEMW { iSubItem: col as i32, pszText: windows::core::PWSTR(t.as_mut_ptr()), ..Default::default() };
    unsafe {
        wm::SendMessageW(h, cc::LVM_SETITEMTEXTW, Some(WPARAM(row)), Some(LPARAM(&item as *const _ as isize)));
    }
}

fn columns(h: HWND) -> usize {
    unsafe {
        let header = wm::SendMessageW(h, cc::LVM_GETHEADER, None, None).0;
        wm::SendMessageW(HWND(header as *mut _), cc::HDM_GETITEMCOUNT, None, None).0.max(0) as usize
    }
}

/// The cell under a point in the grid, if there is one.
fn hit(h: HWND, x: i32, y: i32) -> Option<(usize, usize)> {
    let mut info = cc::LVHITTESTINFO { pt: POINT { x, y }, ..Default::default() };
    let row = unsafe { wm::SendMessageW(h, cc::LVM_SUBITEMHITTEST, None, Some(LPARAM(&mut info as *mut _ as isize))).0 };
    (row >= 0 && info.iItem >= 0).then_some((info.iItem as usize, info.iSubItem.max(0) as usize))
}

/// The box over a cell - the column's kind of box - made, filled with the cell, all chosen. A
/// check box's cell is turned instead; a read-only column's opens nothing.
fn open_editor(h: HWND, row: usize, col: usize) {
    close_editor(h, true);
    let c = column(h, col);
    if !c.visible || c.read_only {
        return;
    }
    if c.editor == Editor::Check {
        let now = cell(h, row, col);
        set_cell(h, row, col, &toggled(&now));
        tell_changed(h);
        return;
    }
    let mut r = RECT { top: col as i32, left: 0 /* LVIR_BOUNDS */, ..Default::default() };
    unsafe {
        wm::SendMessageW(h, cc::LVM_GETSUBITEMRECT, Some(WPARAM(row)), Some(LPARAM(&mut r as *mut _ as isize)));
    }
    if col == 0 {
        // The first column's rectangle is the whole row: as wide as its column only.
        let w0 = unsafe { wm::SendMessageW(h, cc::LVM_GETCOLUMNWIDTH, Some(WPARAM(0)), None).0 } as i32;
        r.right = r.left + w0;
    }
    let font = windows::Win32::Graphics::Gdi::HFONT(unsafe { wm::SendMessageW(h, wm::WM_GETFONT, None, None).0 } as *mut _);
    let now = cell(h, row, col);
    let e = match &c.editor {
        Editor::Combo(items) => {
            let refs: Vec<&str> = items.iter().map(String::as_str).collect();
            let e = win::combo_with(h, EDITOR_ID, font, &refs, true);
            win::move_to(e, r.left, r.top, r.right - r.left, win::px(h, 240));
            win::set_text(e, &now);
            e
        }
        Editor::Date => {
            let e = win::date(h, EDITOR_ID, font);
            win::move_to(e, r.left, r.top, r.right - r.left, r.bottom - r.top);
            let mut p = now.trim().split(['-', '/']).map(|x| x.parse::<i32>().unwrap_or(0));
            if let (Some(y), Some(m), Some(d)) = (p.next(), p.next(), p.next()) {
                if y > 1600 && (1..=12).contains(&m) && (1..=31).contains(&d) {
                    win::set_date(e, y, m, d);
                }
            }
            e
        }
        _ => {
            let e = win::edit(h, &now, EDITOR_ID, font, false);
            win::move_to(e, r.left, r.top, r.right - r.left, r.bottom - r.top);
            e
        }
    };
    unsafe {
        wm::SetPropW(e, w!("PhaiGridCell"), Some(windows::Win32::Foundation::HANDLE(((row << 16) | col) as *mut _))).ok();
        wm::SendMessageW(e, 0x00B1 /* EM_SETSEL */, Some(WPARAM(0)), Some(LPARAM(-1)));
        let _ = SetWindowSubclass(e, Some(editor_proc), 8, h.0 as usize);
        // A drop-down's typing box is a child of its own: it hears the keys.
        if let Ok(inner) = wm::GetWindow(e, wm::GW_CHILD) {
            let _ = SetWindowSubclass(inner, Some(editor_proc), 8, h.0 as usize);
        }
        let _ = wm::SetWindowPos(e, Some(wm::HWND_TOP), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE);
        let _ = km::SetFocus(Some(e));
    }
}

fn editor_of(h: HWND) -> Option<HWND> {
    unsafe { wm::GetDlgItem(Some(h), EDITOR_ID) }.ok().filter(|e| !e.is_invalid())
}

/// The box closed - its words kept in the cell when `keep` - and the parent told of a change.
fn close_editor(h: HWND, keep: bool) {
    let Some(e) = editor_of(h) else { return };
    let at = unsafe { wm::GetPropW(e, w!("PhaiGridCell")) }.0 as usize;
    let (row, col) = (at >> 16, at & 0xFFFF);
    let c = column(h, col);
    let text = match c.editor {
        Editor::Date => {
            let (y, m, d) = win::date_value(e);
            format!("{y:04}-{m:02}-{d:02}")
        }
        _ => win::text_of(e),
    };
    // A number column keeps numbers only (or nothing).
    let keep = keep && (c.editor != Editor::Number || text.trim().is_empty() || text.trim().replace(',', "").parse::<f64>().is_ok());
    unsafe {
        let _ = wm::RemovePropW(e, w!("PhaiGridCell"));
        let _ = wm::DestroyWindow(e);
    }
    if keep && text != cell(h, row, col) {
        set_cell(h, row, col, &text);
        tell_changed(h);
    }
    unsafe {
        let _ = km::SetFocus(Some(h));
    }
}

fn chosen_row(h: HWND) -> Option<usize> {
    let i = unsafe { wm::SendMessageW(h, cc::LVM_GETNEXTITEM, Some(WPARAM(usize::MAX)), Some(LPARAM(2))).0 };
    (i >= 0).then_some(i as usize)
}

static READ_ONLY: std::sync::Mutex<Vec<isize>> = std::sync::Mutex::new(Vec::new());

/// A grid whose cells are read, not changed: no editor opens, no row is added or taken out.
pub fn set_read_only(h: HWND, on: bool) {
    let mut v = READ_ONLY.lock().unwrap_or_else(|e| e.into_inner());
    v.retain(|x| *x != h.0 as isize);
    if on {
        v.push(h.0 as isize);
    }
}

fn is_read_only(h: HWND) -> bool {
    READ_ONLY.lock().unwrap_or_else(|e| e.into_inner()).contains(&(h.0 as isize))
}

unsafe extern "system" fn grid_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, _: usize) -> LRESULT {
    if is_read_only(h) && (msg == wm::WM_LBUTTONDBLCLK || (msg == wm::WM_KEYDOWN && (w.0 == 0x71 || w.0 == 0x2E))) {
        // Read only: a double-click, F2 or Delete do what a list's would.
        return unsafe { DefSubclassProc(h, msg, w, l) };
    }
    // The heading: a click sorts, a column that may not be sized keeps its width.
    if msg == wm::WM_NOTIFY {
        let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
        let header = unsafe { wm::SendMessageW(h, cc::LVM_GETHEADER, None, None).0 };
        if hdr.hwndFrom.0 as isize == header {
            let nm = unsafe { &*(l.0 as *const cc::NMHEADERW) };
            let i = nm.iItem.max(0) as usize;
            match hdr.code as i32 {
                -322 /* HDN_ITEMCLICKW */ => {
                    if column(h, i).allow_sort && column(h, i).visible {
                        let up = with_opts(h, |o| o.sorted).is_none_or(|(c, up)| c != i || !up);
                        sort_by(h, i, up);
                    }
                }
                -326 /* HDN_BEGINTRACKW */ | -306 /* HDN_BEGINTRACKA */ => {
                    let c = column(h, i);
                    if !c.allow_size || !c.visible || with_opts(h, |o| o.auto_width) {
                        return LRESULT(1);
                    }
                }
                -325 /* HDN_DIVIDERDBLCLICKW */ => {
                    let c = column(h, i);
                    if !c.allow_size || !c.visible {
                        return LRESULT(0);
                    }
                }
                _ => {}
            }
        }
        return unsafe { DefSubclassProc(h, msg, w, l) };
    }
    match msg {
        wm::WM_LBUTTONDOWN => {
            let r = unsafe { DefSubclassProc(h, msg, w, l) };
            let (x, y) = win::point_of(l);
            if let Some((row, col)) = hit(h, x, y) {
                with_opts(h, |o| o.focus_col = col);
                let c = column(h, col);
                // A check box turns at a click; Click opens any other cell's box.
                if c.editor == Editor::Check || with_opts(h, |o| o.edit_on) == EditOn::Click {
                    open_editor(h, row, col);
                }
            }
            r
        }
        wm::WM_LBUTTONDBLCLK => {
            let (x, y) = win::point_of(l);
            match hit(h, x, y) {
                Some((row, col)) => {
                    if column(h, col).editor != Editor::Check && with_opts(h, |o| o.edit_on) == EditOn::DoubleClick {
                        open_editor(h, row, col);
                    }
                }
                None if with_opts(h, |o| o.add) => {
                    // Under the last row: a new one, its first cell open.
                    let blank: Vec<&str> = vec![""; columns(h).max(1)];
                    grid_add(h, &blank);
                    let last = unsafe { wm::SendMessageW(h, cc::LVM_GETITEMCOUNT, None, None).0 } as usize - 1;
                    win::list_select(h, last);
                    if let Some(c) = editable_from(h, 0, true) {
                        open_editor(h, last, c);
                    }
                    tell_changed(h);
                }
                None => {}
            }
            LRESULT(0)
        }
        wm::WM_KEYDOWN if w.0 == 0x71 /* F2 */ => {
            if let Some(r) = chosen_row(h) {
                let at = with_opts(h, |o| o.focus_col);
                if let Some(c) = editable_from(h, at, true).or_else(|| editable_from(h, 0, true)) {
                    open_editor(h, r, c);
                }
            }
            LRESULT(0)
        }
        wm::WM_KEYDOWN if w.0 == 0x2E /* Delete: the chosen row goes */ => {
            if !with_opts(h, |o| o.delete) {
                return unsafe { DefSubclassProc(h, msg, w, l) };
            }
            if let Some(r) = chosen_row(h) {
                unsafe {
                    wm::SendMessageW(h, cc::LVM_DELETEITEM, Some(WPARAM(r)), None);
                }
                tell_changed(h);
            }
            LRESULT(0)
        }
        wm::WM_SIZE => {
            let r = unsafe { DefSubclassProc(h, msg, w, l) };
            if with_opts(h, |o| o.auto_width) {
                fit_columns(h);
            }
            r
        }
        wm::WM_NCDESTROY => {
            OPTS.lock().unwrap_or_else(|e| e.into_inner()).as_mut().map(|m| m.remove(&(h.0 as isize)));
            unsafe { DefSubclassProc(h, msg, w, l) }
        }
        m if m == wm::WM_APP + 0x7E => {
            grid_idle(h);
            LRESULT(0)
        }
        wm::WM_VSCROLL | wm::WM_HSCROLL | wm::WM_MOUSEWHEEL => {
            close_editor(h, true);
            unsafe { DefSubclassProc(h, msg, w, l) }
        }
        _ => unsafe { DefSubclassProc(h, msg, w, l) },
    }
}

unsafe extern "system" fn editor_proc(e: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, grid: usize) -> LRESULT {
    let g = HWND(grid as *mut _);
    match msg {
        wm::WM_GETDLGCODE => LRESULT(4 /* DLGC_WANTALLKEYS */),
        wm::WM_KEYDOWN if w.0 == 0x0D => {
            // EnterMoveNextColumn: the next cell's box, the next row's first at the end.
            let at = editor_of(g).map(|e| unsafe { wm::GetPropW(e, w!("PhaiGridCell")) }.0 as usize);
            close_editor(g, true);
            if let (Some(at), true) = (at, with_opts(g, |o| o.enter_next)) {
                let (row, col) = (at >> 16, at & 0xFFFF);
                let rows = unsafe { wm::SendMessageW(g, cc::LVM_GETITEMCOUNT, None, None).0 } as usize;
                match editable_from(g, col, false) {
                    Some(c) => open_editor(g, row, c),
                    None if row + 1 < rows => {
                        win::list_select(g, row + 1);
                        if let Some(c) = editable_from(g, 0, true) {
                            open_editor(g, row + 1, c);
                        }
                    }
                    None => {}
                }
            }
            LRESULT(0)
        }
        wm::WM_KEYDOWN if w.0 == 0x1B => {
            close_editor(g, false);
            LRESULT(0)
        }
        wm::WM_CHAR if w.0 == 0x0D || w.0 == 0x1B => LRESULT(0),
        wm::WM_KILLFOCUS => {
            let r = unsafe { DefSubclassProc(e, msg, w, l) };
            // Closed after this message is done with: never while Windows still uses the box.
            unsafe {
                let _ = wm::PostMessageW(Some(g), wm::WM_APP + 0x7E, WPARAM(0), LPARAM(0));
            }
            r
        }
        _ => unsafe { DefSubclassProc(e, msg, w, l) },
    }
}

/// Called by the grid's subclass on WM_APP + 0x7E: a box that lost the focus closes, kept.
pub fn grid_idle(h: HWND) {
    if let Some(e) = editor_of(h) {
        let focus = unsafe { km::GetFocus() };
        // Still in the box - or in a drop-down's typing box, or its calendar.
        if focus != e && !unsafe { wm::IsChild(e, focus) }.as_bool() {
            close_editor(h, true);
        }
    }
}

/// A rich text box: Windows' RichEdit, its words `text`, many lines, scrolled as they grow.
pub fn richtext(parent: HWND, id: i32, f: windows::Win32::Graphics::Gdi::HFONT, text: &str) -> HWND {
    unsafe {
        let _ = LoadLibraryW(w!("Msftedit.dll"));
    }
    const ES_MULTILINE: u32 = 0x0004;
    const ES_AUTOVSCROLL: u32 = 0x0040;
    const ES_WANTRETURN: u32 = 0x1000;
    let style = wm::WINDOW_STYLE(ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN) | wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_TABSTOP | wm::WS_VSCROLL | wm::WS_BORDER;
    let t = win::wide(text);
    let h = unsafe { wm::CreateWindowExW(Default::default(), w!("RICHEDIT50W"), PCWSTR(t.as_ptr()), style, 0, 0, 10, 10, Some(parent), Some(wm::HMENU(id as isize as *mut _)), Some(win::instance()), None).unwrap_or_default() };
    unsafe {
        wm::SendMessageW(h, wm::WM_SETFONT, Some(WPARAM(f.0 as usize)), Some(LPARAM(1)));
        // EN_CHANGE to the parent, as an edit box sends it.
        const EM_SETEVENTMASK: u32 = 0x0445;
        const ENM_CHANGE: isize = 0x0001;
        wm::SendMessageW(h, EM_SETEVENTMASK, Some(WPARAM(0)), Some(LPARAM(ENM_CHANGE)));
        // The theme's window colour behind the words.
        const EM_SETBKGNDCOLOR: u32 = 0x0443;
        wm::SendMessageW(h, EM_SETBKGNDCOLOR, Some(WPARAM(0)), Some(LPARAM(win::window_background_for(h).colorref() as isize)));
    }
    win::register_kind(h, win::Kind::Edit);
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_double_click_under_the_rows_adds_one_and_enter_keeps_what_is_typed() {
        win::init();
        extern "system" fn host_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
            win::def(h, msg, w, l)
        }
        let class = win::register("PhaiGridTestHost", Some(host_proc), None);
        let host = unsafe { wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_OVERLAPPEDWINDOW, 0, 0, 400, 300, None, None, Some(win::instance()), None).unwrap_or_default() };
        let font = win::font(host, 0, false);
        let g = grid(host, 5, font, &[("Name", 120), ("Age", 60)]);
        win::move_to(g, 0, 0, 380, 260);
        grid_add(g, &["Somchai", "40"]);
        unsafe {
            // Far below the one row: a new row, its first cell open for typing.
            wm::SendMessageW(g, wm::WM_LBUTTONDBLCLK, Some(WPARAM(0)), Some(LPARAM((200 << 16) | 20)));
        }
        let e = editor_of(g).expect("a box over the new cell");
        win::set_text(e, "Malee");
        unsafe {
            wm::SendMessageW(e, wm::WM_KEYDOWN, Some(WPARAM(0x0D)), Some(LPARAM(0)));
        }
        assert!(editor_of(g).is_none(), "Enter closes the box");
        assert_eq!(grid_rows(g), vec![vec!["Somchai".to_string(), "40".into()], vec!["Malee".into(), String::new()]]);
        unsafe {
            let _ = wm::DestroyWindow(host);
        }
    }
}
