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
    /// How its values are shown (they are kept as they are): `n2` 1,234.50, `c2` ฿1,234.50,
    /// `p0` 15%, `d` 21/11/2026, `dt` with the time - DevExpress's DisplayFormat.
    pub format: String,
    /// What the footer under it says: a sum, a count...
    pub summary: Summary,
    /// Its cells (or rows) coloured by their values - DevExpress's format rules.
    pub rules: Vec<Rule>,
    /// The rows grouped by its values, a heading over each group (one column groups).
    pub group: bool,
}

/// A footer's total under a column.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Summary {
    #[default]
    None,
    Sum,
    Count,
    Average,
    Min,
    Max,
}

impl Summary {
    pub fn from_word(w: &str) -> Summary {
        match w {
            "sum" => Summary::Sum,
            "count" => Summary::Count,
            "avg" | "average" => Summary::Average,
            "min" => Summary::Min,
            "max" => Summary::Max,
            _ => Summary::None,
        }
    }
}

/// A format rule: cells whose value passes `op value` take these colours - the whole row when
/// `row`. Written as a line: `> 1000 back=#ffe4e4 fore=#b00020 row`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rule {
    /// `=`, `!=`, `>`, `>=`, `<`, `<=`, `contains`, `empty`, `notempty`.
    pub op: String,
    pub value: String,
    pub back: String,
    pub fore: String,
    pub row: bool,
}

impl Rule {
    /// A rule from its line; `None` when the line is not one.
    pub fn parse(line: &str) -> Option<Rule> {
        let line = line.trim();
        let (op, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
        let op = match op {
            "==" => "=",
            "<>" => "!=",
            o => o,
        };
        if !["=", "!=", ">", ">=", "<", "<=", "contains", "empty", "notempty"].contains(&op) {
            return None;
        }
        let mut r = Rule { op: op.to_string(), ..Default::default() };
        let mut rest = rest.trim();
        if !matches!(op, "empty" | "notempty") {
            if let Some(q) = rest.strip_prefix('"') {
                let end = q.find('"')?;
                r.value = q[..end].to_string();
                rest = q[end + 1..].trim();
            } else {
                let (v, more) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
                r.value = v.to_string();
                rest = more.trim();
            }
        }
        for word in rest.split_whitespace() {
            match word.split_once('=') {
                Some(("back", c)) => r.back = c.to_string(),
                Some(("fore", c)) => r.fore = c.to_string(),
                None if word == "row" => r.row = true,
                _ => return None,
            }
        }
        Some(r)
    }

    /// Its line again.
    pub fn text(&self) -> String {
        let mut s = self.op.clone();
        if !matches!(self.op.as_str(), "empty" | "notempty") {
            s += &if self.value.contains(char::is_whitespace) || self.value.is_empty() { format!(" \"{}\"", self.value) } else { format!(" {}", self.value) };
        }
        if !self.back.is_empty() {
            s += &format!(" back={}", self.back);
        }
        if !self.fore.is_empty() {
            s += &format!(" fore={}", self.fore);
        }
        if self.row {
            s += " row";
        }
        s
    }

    /// Whether a cell's words pass it: numbers compared as numbers, words without regard to case.
    pub fn matches(&self, cell: &str) -> bool {
        let (a, b) = (cell.trim(), self.value.trim());
        let num = |s: &str| s.replace(',', "").parse::<f64>().ok();
        let ord = match (num(a), num(b)) {
            (Some(x), Some(y)) => x.partial_cmp(&y),
            _ => Some(a.to_lowercase().cmp(&b.to_lowercase())),
        };
        use std::cmp::Ordering::*;
        match self.op.as_str() {
            "=" => ord == Some(Equal),
            "!=" => ord != Some(Equal),
            ">" => ord == Some(Greater),
            ">=" => matches!(ord, Some(Greater | Equal)),
            "<" => ord == Some(Less),
            "<=" => matches!(ord, Some(Less | Equal)),
            "contains" => a.to_lowercase().contains(&b.to_lowercase()),
            "empty" => a.is_empty(),
            "notempty" => !a.is_empty(),
            _ => false,
        }
    }
}

/// Rules from their lines, those that are rules.
pub fn rules(lines: &[&str]) -> Vec<Rule> {
    lines.iter().filter_map(|l| Rule::parse(l)).collect()
}

/// A number with commas between thousands and `decimals` places: 1,234.50.
fn grouped(n: f64, decimals: usize) -> String {
    let s = format!("{:.*}", decimals, n.abs());
    let (int, frac) = s.split_once('.').map(|(i, f)| (i.to_string(), Some(f.to_string()))).unwrap_or((s.clone(), None));
    let mut out = String::new();
    for (i, ch) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    if let Some(f) = frac {
        out = format!("{out}.{f}");
    }
    if n < 0.0 && out.chars().any(|c| c.is_ascii_digit() && c != '0') {
        out.insert(0, '-');
    }
    out
}

/// A value as `format` shows it (see [`Column::format`]); as it is when it does not fit.
pub fn format_value(format: &str, value: &str) -> String {
    let f = format.trim();
    if f.is_empty() || value.trim().is_empty() {
        return value.to_string();
    }
    let num = value.trim().replace(',', "").parse::<f64>();
    let places = |s: &str| s.parse::<usize>().unwrap_or(0).min(6);
    match (f.chars().next(), num) {
        (Some('n'), Ok(n)) => grouped(n, places(&f[1..])),
        (Some('c'), Ok(n)) => format!("฿{}", grouped(n, places(&f[1..]))),
        (Some('p'), Ok(n)) => format!("{}%", grouped(n * 100.0, places(&f[1..]))),
        _ if f == "d" || f == "dt" => {
            // 2026-11-21 (and a time after it): 21/11/2026.
            let v = value.trim();
            let (date, time) = v.split_once([' ', 'T']).unwrap_or((v, ""));
            let p: Vec<&str> = date.split('-').collect();
            if p.len() == 3 && p[0].len() == 4 {
                let t = if f == "dt" && time.len() >= 5 { format!(" {}", &time[..5]) } else { String::new() };
                format!("{}/{}/{}{t}", p[2], p[1], p[0])
            } else {
                value.to_string()
            }
        }
        _ => value.to_string(),
    }
}

impl Default for Column {
    fn default() -> Column {
        Column { visible: true, align: Align::Left, read_only: false, editor: Editor::Text, allow_sort: true, allow_size: true, sort: None, format: String::new(), summary: Summary::None, rules: vec![], group: false }
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
    /// Every other row's background.
    alt: Option<crate::theme::Color>,
    header_back: Option<crate::theme::Color>,
    header_fore: Option<crate::theme::Color>,
    /// The footer under the rows, its totals.
    footer: bool,
    /// What the find box looks for (lower case); empty: nothing.
    find: String,
    find_mode: FindMode,
    /// Ctrl+C copies the headings too.
    copy_headers: bool,
    /// The find box leaves only the rows it finds (else it marks them).
    find_filters: bool,
    /// Rows the find box left out, with their tags: kept, given back when it is emptied, saved
    /// with the rest.
    hidden: Vec<(usize, Vec<String>)>,
    /// The find box: a window beside the grid, over the strip kept above its headings.
    find_box: Option<isize>,
    /// Each filtered column's values that stay (as the cells hold them) - Excel's ticks.
    filters: std::collections::BTreeMap<usize, Vec<String>>,
    /// AllowFilter: a drop-down on each heading opens its filter.
    allow_filter: bool,
    /// HeaderMenu: a right-click on a heading offers sorting, filters and exporting.
    header_menu: bool,
    /// A detail grid's: the column that holds its master row's key, and the key now - only the
    /// rows with it shown.
    master_filter: Option<(usize, String)>,
}

/// When the find box shows - DevExpress's find panel.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum FindMode {
    /// Ctrl+F opens it.
    #[default]
    CtrlF,
    Always,
    Off,
}

impl Default for Opts {
    fn default() -> Opts {
        Opts { cols: vec![], widths: vec![], add: true, delete: true, edit_on: EditOn::DoubleClick, enter_next: false, auto_width: false, sorted: None, focus_col: 0, alt: None, header_back: None, header_fore: None, footer: false, find: String::new(), find_mode: FindMode::CtrlF, copy_headers: true, find_filters: true, hidden: vec![], find_box: None, filters: Default::default(), allow_filter: true, header_menu: true, master_filter: None }
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
    if c.group != was.group {
        regroup(h);
    }
    // A total now under it, or none: the footer's room again.
    if c.summary != was.summary {
        frame_changed(h);
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

fn has_opts(h: HWND) -> bool {
    OPTS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().is_some_and(|m| m.contains_key(&(h.0 as isize)))
}

fn color(s: &str) -> Option<crate::theme::Color> {
    if s.trim().is_empty() { None } else { crate::theme::parse_color(s) }
}

fn redraw(h: HWND) {
    unsafe {
        let _ = windows::Win32::Graphics::Gdi::RedrawWindow(Some(h), None, None, windows::Win32::Graphics::Gdi::RDW_INVALIDATE | windows::Win32::Graphics::Gdi::RDW_FRAME);
    }
}

/// AlternateRowColor: every other row's background (`""`: none).
pub fn set_alternate_row_color(h: HWND, c: &str) {
    with_opts(h, |o| o.alt = color(c));
    redraw(h);
}

/// HeaderBackColor and HeaderForeColor: the headings' colours (`""`: Windows' own).
pub fn set_header_back_color(h: HWND, c: &str) {
    with_opts(h, |o| o.header_back = color(c));
    redraw(header_of(h));
}

pub fn set_header_fore_color(h: HWND, c: &str) {
    with_opts(h, |o| o.header_fore = color(c));
    redraw(header_of(h));
}

/// ShowFooter: a row under the rows with each column's total ([`Column::summary`]).
pub fn set_show_footer(h: HWND, on: bool) {
    with_opts(h, |o| o.footer = on);
    frame_changed(h);
}

/// RowHeight: the rows this tall, in design pixels (0: as the font makes them).
pub fn set_row_height(h: HWND, px: i32) {
    use windows::Win32::UI::Controls as cc;
    unsafe {
        let list = if px > 0 { cc::ImageList_Create(1, win::px(h, px), cc::ILC_COLOR32, 1, 0) } else { cc::HIMAGELIST::default() };
        let old = wm::SendMessageW(h, 0x1003 /* LVM_SETIMAGELIST */, Some(WPARAM(1 /* LVSIL_SMALL */)), Some(LPARAM(list.0 as isize))).0;
        if old != 0 {
            let _ = cc::ImageList_Destroy(Some(cc::HIMAGELIST(old as _)));
        }
    }
}

/// FindPanel: `ctrl_f` (Ctrl+F opens the find box), `always` or `off`.
pub fn set_find_panel(h: HWND, mode: &str) {
    let m = match mode {
        "always" => FindMode::Always,
        "off" => FindMode::Off,
        _ => FindMode::CtrlF,
    };
    with_opts(h, |o| o.find_mode = m);
    if m == FindMode::Always {
        open_find(h, false);
    }
}

/// CopyColumnHeaders: Ctrl+C copies the headings with the rows.
pub fn set_copy_headers(h: HWND, on: bool) {
    with_opts(h, |o| o.copy_headers = on);
}

fn header_of(h: HWND) -> HWND {
    HWND(unsafe { wm::SendMessageW(h, cc::LVM_GETHEADER, None, None).0 } as *mut _)
}

fn frame_changed(h: HWND) {
    unsafe {
        let _ = wm::SetWindowPos(h, None, 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOZORDER | wm::SWP_NOACTIVATE | wm::SWP_FRAMECHANGED);
    }
}

/// A cell's colours: the grid's, every other row's, the rules' (the row's first, then the
/// cell's own), what the find box found, the chosen row's.
fn cell_colors(h: HWND, row: usize, col: usize, selected: bool) -> (crate::theme::Color, crate::theme::Color) {
    let (mut bg, mut fg) = win::list_colors(h);
    let (alt, cols, find) = with_opts(h, |o| (o.alt, o.cols.clone(), o.find.clone()));
    if row % 2 == 1 {
        if let Some(a) = alt {
            bg = a;
        }
    }
    let apply = |r: &Rule, bg: &mut crate::theme::Color, fg: &mut crate::theme::Color| {
        if let Some(c) = color(&r.back) {
            *bg = c;
        }
        if let Some(c) = color(&r.fore) {
            *fg = c;
        }
    };
    for (i, c) in cols.iter().enumerate() {
        let rules: Vec<&Rule> = c.rules.iter().filter(|r| r.row).collect();
        if !rules.is_empty() {
            let v = cell(h, row, i);
            for r in rules.into_iter().filter(|r| r.matches(&v)) {
                apply(r, &mut bg, &mut fg);
            }
        }
    }
    if let Some(c) = cols.get(col) {
        let v = cell(h, row, col);
        for r in c.rules.iter().filter(|r| !r.row && r.matches(&v)) {
            apply(r, &mut bg, &mut fg);
        }
    }
    if !find.is_empty() && cell(h, row, col).to_lowercase().contains(&find) {
        bg = crate::theme::Color::rgb(255, 236, 153);
        fg = crate::theme::Color::rgb(27, 27, 27);
    } else if selected {
        (bg, fg) = win::list_selection_colors(h);
    }
    (bg, fg)
}

/// `NM_CUSTOMDRAW` for a grid with properties of its own: each cell painted here - its colours,
/// its value as its format shows it, a check box. `None` for any other list.
pub fn custom_draw(l: LPARAM) -> Option<LRESULT> {
    use windows::Win32::Graphics::Gdi;
    let cd = unsafe { &mut *(l.0 as *mut cc::NMLVCUSTOMDRAW) };
    let h = cd.nmcd.hdr.hwndFrom;
    if !has_opts(h) {
        return None;
    }
    let stage = cd.nmcd.dwDrawStage.0;
    if stage == cc::CDDS_PREPAINT.0 {
        return Some(LRESULT(cc::CDRF_NOTIFYITEMDRAW as isize));
    }
    if stage == cc::CDDS_ITEMPREPAINT.0 {
        return Some(LRESULT(cc::CDRF_NOTIFYSUBITEMDRAW as isize));
    }
    if stage != cc::CDDS_ITEMPREPAINT.0 | cc::CDDS_SUBITEM.0 {
        return Some(LRESULT(cc::CDRF_DODEFAULT as isize));
    }
    let (row, col) = (cd.nmcd.dwItemSpec, cd.iSubItem.max(0) as usize);
    let selected = unsafe { wm::SendMessageW(h, cc::LVM_GETITEMSTATE, Some(WPARAM(row)), Some(LPARAM(cc::LVIS_SELECTED.0 as isize))).0 } as u32 & cc::LVIS_SELECTED.0 != 0;
    let (bg, fg) = cell_colors(h, row, col, selected);
    let dc = cd.nmcd.hdc;
    if draw_check_cell(h, dc, row, col, bg, fg) {
        return Some(LRESULT(cc::CDRF_SKIPDEFAULT as isize));
    }
    let mut r = RECT { top: col as i32, left: 0, ..Default::default() };
    unsafe {
        wm::SendMessageW(h, cc::LVM_GETSUBITEMRECT, Some(WPARAM(row)), Some(LPARAM(&mut r as *mut _ as isize)));
    }
    if col == 0 {
        let w0 = unsafe { wm::SendMessageW(h, cc::LVM_GETCOLUMNWIDTH, Some(WPARAM(0)), None).0 } as i32;
        r.right = r.left + w0;
    }
    if r.right <= r.left {
        return Some(LRESULT(cc::CDRF_SKIPDEFAULT as isize));
    }
    // Inside the grid lines.
    win::fill(dc, &RECT { right: r.right - 1, bottom: r.bottom - 1, ..r }, bg);
    let c = column(h, col);
    let text = format_value(&c.format, &cell(h, row, col));
    let pad = win::px(h, 6);
    let mut tr = RECT { left: r.left + pad, right: r.right - pad, ..r };
    let align = match c.align {
        Align::Left => Gdi::DT_LEFT,
        Align::Center => Gdi::DT_CENTER,
        Align::Right => Gdi::DT_RIGHT,
    };
    unsafe {
        let font = Gdi::HGDIOBJ(wm::SendMessageW(h, wm::WM_GETFONT, None, None).0 as *mut _);
        let old = Gdi::SelectObject(dc, font);
        Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(fg.colorref()));
        let mut w = win::wide(&text);
        w.pop();
        crate::win::draw_text_w(dc, &mut w, &mut tr, align | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_END_ELLIPSIS | Gdi::DT_NOPREFIX);
        Gdi::SelectObject(dc, old);
    }
    Some(LRESULT(cc::CDRF_SKIPDEFAULT as isize))
}

/// The footer's height: a row when it is shown, else none.
fn footer_height(h: HWND) -> i32 {
    if with_opts(h, |o| o.footer || o.cols.iter().any(|c| c.summary != Summary::None)) { win::px(h, 26) } else { 0 }
}

/// A column's total as its footer says it.
fn summary_text(h: HWND, col: usize, c: &Column) -> String {
    let rows = unsafe { wm::SendMessageW(h, cc::LVM_GETITEMCOUNT, None, None).0 } as usize;
    let values: Vec<String> = (0..rows).map(|r| cell(h, r, col)).filter(|v| !v.trim().is_empty()).collect();
    let nums: Vec<f64> = values.iter().filter_map(|v| v.trim().replace(',', "").parse::<f64>().ok()).collect();
    let show = |n: f64| {
        if !c.format.is_empty() {
            format_value(&c.format, &n.to_string())
        } else if nums.iter().all(|x| x.fract() == 0.0) && n.fract() == 0.0 {
            grouped(n, 0)
        } else {
            grouped(n, 2)
        }
    };
    let (label, value) = match c.summary {
        Summary::None => return String::new(),
        Summary::Count => (crate::text::t("Count"), grouped(values.len() as f64, 0)),
        Summary::Sum => (crate::text::t("Sum"), show(nums.iter().sum())),
        Summary::Average => (crate::text::t("Average"), if nums.is_empty() { String::new() } else { grouped(nums.iter().sum::<f64>() / nums.len() as f64, 2) }),
        Summary::Min => (crate::text::t("Min"), nums.iter().copied().fold(None, |m: Option<f64>, x| Some(m.map_or(x, |m| m.min(x)))).map(show).unwrap_or_default()),
        Summary::Max => (crate::text::t("Max"), nums.iter().copied().fold(None, |m: Option<f64>, x| Some(m.map_or(x, |m| m.max(x)))).map(show).unwrap_or_default()),
    };
    format!("{label} = {value}")
}

/// The footer painted in the edge below the rows: each column's total under it.
fn paint_footer(h: HWND) {
    use windows::Win32::Graphics::Gdi;
    let fh = footer_height(h);
    if fh == 0 {
        return;
    }
    unsafe {
        let mut wr = RECT::default();
        let _ = wm::GetWindowRect(h, &mut wr);
        let mut cr = RECT::default();
        let _ = wm::GetClientRect(h, &mut cr);
        let mut o = [windows::Win32::Foundation::POINT { x: 0, y: 0 }];
        Gdi::MapWindowPoints(Some(h), None, &mut o);
        let (ox, oy) = (o[0].x - wr.left, o[0].y - wr.top);
        let dc = Gdi::GetWindowDC(Some(h));
        let top = oy + cr.bottom;
        let back = with_opts(h, |o| o.header_back).unwrap_or_else(|| crate::theme::Color::rgb(243, 243, 243));
        let fore = with_opts(h, |o| o.header_fore).unwrap_or_else(|| win::list_colors(h).1);
        win::fill(dc, &RECT { left: ox, top, right: ox + cr.right, bottom: top + fh }, back);
        win::fill(dc, &RECT { left: ox, top, right: ox + cr.right, bottom: top + 1 }, win::list_border_color(h));
        let font = Gdi::HGDIOBJ(wm::SendMessageW(h, wm::WM_GETFONT, None, None).0 as *mut _);
        let old = Gdi::SelectObject(dc, font);
        Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(fore.colorref()));
        let header = header_of(h);
        let rgn = Gdi::CreateRectRgn(ox, top, ox + cr.right, top + fh);
        let _ = Gdi::SelectClipRgn(dc, Some(rgn));
        for i in 0..columns(h) {
            let c = column(h, i);
            if c.summary == Summary::None || !c.visible {
                continue;
            }
            let mut ir = RECT::default();
            wm::SendMessageW(header, cc::HDM_GETITEMRECT, Some(WPARAM(i)), Some(LPARAM(&mut ir as *mut _ as isize)));
            let mut p = [windows::Win32::Foundation::POINT { x: ir.left, y: 0 }, windows::Win32::Foundation::POINT { x: ir.right, y: 0 }];
            Gdi::MapWindowPoints(Some(header), Some(h), &mut p);
            let pad = win::px(h, 6);
            let mut tr = RECT { left: ox + p[0].x + pad, right: ox + p[1].x - pad, top: top + 1, bottom: top + fh };
            let mut w = win::wide(&summary_text(h, i, &c));
            w.pop();
            let align = match c.align {
                Align::Left => Gdi::DT_LEFT,
                Align::Center => Gdi::DT_CENTER,
                Align::Right => Gdi::DT_RIGHT,
            };
            crate::win::draw_text_w(dc, &mut w, &mut tr, align | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_END_ELLIPSIS | Gdi::DT_NOPREFIX);
        }
        let _ = Gdi::SelectClipRgn(dc, None);
        let _ = Gdi::DeleteObject(rgn.into());
        Gdi::SelectObject(dc, old);
        Gdi::ReleaseDC(Some(h), dc);
    }
}

/// The strip above the headings the find box sits on, in the headings' colour.
fn paint_find_strip(h: HWND) {
    let fs = find_strip(h);
    if fs == 0 {
        return;
    }
    unsafe {
        let mut wr = RECT::default();
        let _ = wm::GetWindowRect(h, &mut wr);
        let mut cr = RECT::default();
        let _ = wm::GetClientRect(h, &mut cr);
        let mut o = [windows::Win32::Foundation::POINT { x: 0, y: 0 }];
        windows::Win32::Graphics::Gdi::MapWindowPoints(Some(h), None, &mut o);
        let (ox, oy) = (o[0].x - wr.left, o[0].y - wr.top);
        let dc = windows::Win32::Graphics::Gdi::GetWindowDC(Some(h));
        let back = crate::theme::Color::rgb(243, 243, 243);
        win::fill(dc, &RECT { left: ox, top: oy - fs, right: ox + cr.right, bottom: oy }, back);
        win::fill(dc, &RECT { left: ox, top: oy - 1, right: ox + cr.right, bottom: oy }, win::list_border_color(h));
        windows::Win32::Graphics::Gdi::ReleaseDC(Some(h), dc);
    }
}

/// The headings in the grid's own colours, the sort arrow drawn too. `None`: Windows draws them.
fn header_custom_draw(h: HWND, l: LPARAM) -> Option<LRESULT> {
    use windows::Win32::Graphics::Gdi;
    let (back, fore, sorted, filtered) = with_opts(h, |o| (o.header_back, o.header_fore, o.sorted, o.filters.keys().copied().collect::<Vec<_>>()));
    let cd = unsafe { &mut *(l.0 as *mut cc::NMCUSTOMDRAW) };
    // A filtered column's heading shows the filter's mark.
    let mark = |dc: Gdi::HDC, r: RECT, colour: crate::theme::Color| {
        let size = win::px(h, 12);
        let drop = if filters_allowed(h) { unsafe { wm::GetSystemMetrics(wm::SM_CXVSCROLL) } } else { 0 };
        if let Some(icon) = crate::icon::glyph_icon(0xE71C, size, colour) {
            unsafe {
                let _ = wm::DrawIconEx(dc, r.right - drop - size - win::px(h, 4), r.top + (r.bottom - r.top - size) / 2, icon, size, size, 0, None, wm::DI_NORMAL);
                let _ = wm::DestroyIcon(icon);
            }
        }
    };
    if back.is_none() && fore.is_none() {
        if filtered.is_empty() {
            return None;
        }
        // Windows draws them; the mark after.
        if cd.dwDrawStage == cc::CDDS_PREPAINT {
            return Some(LRESULT(cc::CDRF_NOTIFYITEMDRAW as isize));
        }
        if cd.dwDrawStage == cc::CDDS_ITEMPREPAINT {
            return Some(LRESULT(if filtered.contains(&cd.dwItemSpec) { cc::CDRF_NOTIFYPOSTPAINT } else { cc::CDRF_DODEFAULT } as isize));
        }
        if cd.dwDrawStage == cc::CDDS_ITEMPOSTPAINT {
            mark(cd.hdc, cd.rc, crate::theme::Color::rgb(0, 95, 184));
        }
        return Some(LRESULT(cc::CDRF_DODEFAULT as isize));
    }
    let back = back.unwrap_or_else(|| win::list_colors(h).0);
    let fore = fore.unwrap_or_else(|| win::list_colors(h).1);
    if cd.dwDrawStage == cc::CDDS_PREPAINT {
        win::fill(cd.hdc, &cd.rc, back);
        return Some(LRESULT(cc::CDRF_NOTIFYITEMDRAW as isize));
    }
    if cd.dwDrawStage != cc::CDDS_ITEMPREPAINT {
        return Some(LRESULT(cc::CDRF_DODEFAULT as isize));
    }
    let i = cd.dwItemSpec;
    let r = cd.rc;
    win::fill(cd.hdc, &r, back);
    // The line between headings.
    win::fill(cd.hdc, &RECT { left: r.right - 1, ..r }, win::list_border_color(h));
    let mut buf = [0u16; 260];
    let mut it = cc::HDITEMW { mask: cc::HDI_TEXT, pszText: windows::core::PWSTR(buf.as_mut_ptr()), cchTextMax: 260, ..Default::default() };
    unsafe {
        wm::SendMessageW(cd.hdr.hwndFrom, cc::HDM_GETITEMW, Some(WPARAM(i)), Some(LPARAM(&mut it as *mut _ as isize)));
    }
    let n = buf.iter().position(|c| *c == 0).unwrap_or(0);
    let mut text: Vec<u16> = buf[..n].to_vec();
    let pad = win::px(h, 6);
    let arrow = win::px(h, 12);
    let drop = if filters_allowed(h) { unsafe { wm::GetSystemMetrics(wm::SM_CXVSCROLL) } } else { 0 };
    let marked = filtered.contains(&i);
    let mut tr = RECT { left: r.left + pad, right: r.right - pad - drop - if sorted.is_some_and(|(c, _)| c == i) { arrow } else { 0 } - if marked { arrow + pad } else { 0 }, ..r };
    let align = match column(h, i).align {
        Align::Left => Gdi::DT_LEFT,
        Align::Center => Gdi::DT_CENTER,
        Align::Right => Gdi::DT_RIGHT,
    };
    unsafe {
        Gdi::SetBkMode(cd.hdc, Gdi::TRANSPARENT);
        Gdi::SetTextColor(cd.hdc, windows::Win32::Foundation::COLORREF(fore.colorref()));
        crate::win::draw_text_w(cd.hdc, &mut text, &mut tr, align | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_END_ELLIPSIS | Gdi::DT_NOPREFIX);
    }
    if let Some((c, up)) = sorted.filter(|(c, _)| *c == i) {
        let _ = c;
        let size = win::px(h, 10);
        if let Some(icon) = crate::icon::glyph_icon(if up { 0xE70E } else { 0xE70D }, size, fore) {
            unsafe {
                let _ = wm::DrawIconEx(cd.hdc, r.right - pad - drop - size, r.top + (r.bottom - r.top - size) / 2, icon, size, size, 0, None, wm::DI_NORMAL);
                let _ = wm::DestroyIcon(icon);
            }
        }
    }
    if marked {
        let left = if sorted.is_some_and(|(c, _)| c == i) { arrow + pad } else { 0 };
        mark(cd.hdc, RECT { right: r.right - left, ..r }, fore);
    }
    // The filter's drop-down, Windows' own being skipped with the rest.
    if drop > 0 {
        let size = win::px(h, 8);
        if let Some(icon) = crate::icon::glyph_icon(0xE70D, size, fore) {
            unsafe {
                let _ = wm::DrawIconEx(cd.hdc, r.right - drop + (drop - size) / 2, r.top + (r.bottom - r.top - size) / 2, icon, size, size, 0, None, wm::DI_NORMAL);
                let _ = wm::DestroyIcon(icon);
            }
        }
    }
    Some(LRESULT(cc::CDRF_SKIPDEFAULT as isize))
}

const FIND_ID: i32 = 0x7E02;

fn find_box(h: HWND) -> Option<HWND> {
    with_opts(h, |o| o.find_box).map(|e| HWND(e as *mut _)).filter(|e| unsafe { wm::IsWindow(Some(*e)) }.as_bool())
}

/// The strip above the headings the find box sits on: its height, none when it is closed.
fn find_strip(h: HWND) -> i32 {
    if find_box(h).is_some() { win::px(h, 34) } else { 0 }
}

/// The find box, as DevExpress's find panel: in a strip above the headings, what is typed found
/// in the cells - the other rows set aside (or the cells marked) - Enter goes to the next row
/// that has it, Escape closes it.
fn open_find(h: HWND, focus: bool) {
    if with_opts(h, |o| o.find_mode) == FindMode::Off {
        return;
    }
    let e = match find_box(h) {
        Some(e) => e,
        None => {
            let Ok(parent) = (unsafe { wm::GetParent(h) }) else { return };
            let font = windows::Win32::Graphics::Gdi::HFONT(unsafe { wm::SendMessageW(h, wm::WM_GETFONT, None, None).0 } as *mut _);
            let e = win::edit(parent, "", FIND_ID, font, false);
            let cue = win::wide(crate::text::t("Find..."));
            unsafe {
                wm::SendMessageW(e, 0x1501 /* EM_SETCUEBANNER */, Some(WPARAM(1)), Some(LPARAM(cue.as_ptr() as isize)));
                let _ = SetWindowSubclass(e, Some(find_proc), 9, h.0 as usize);
            }
            with_opts(h, |o| o.find_box = Some(e.0 as isize));
            frame_changed(h);
            e
        }
    };
    place_find(h);
    if focus {
        unsafe {
            let _ = km::SetFocus(Some(e));
            wm::SendMessageW(e, 0x00B1, Some(WPARAM(0)), Some(LPARAM(-1)));
        }
    }
}

fn place_find(h: HWND) {
    let Some(e) = find_box(h) else { return };
    unsafe {
        let Ok(parent) = wm::GetParent(h) else { return };
        let mut wr = RECT::default();
        let _ = wm::GetWindowRect(h, &mut wr);
        let mut cr = RECT::default();
        let _ = wm::GetClientRect(h, &mut cr);
        let mut o = [windows::Win32::Foundation::POINT { x: 0, y: 0 }];
        windows::Win32::Graphics::Gdi::MapWindowPoints(Some(h), None, &mut o);
        // The strip's top: just above the client area (under the border).
        let mut at = [windows::Win32::Foundation::POINT { x: o[0].x, y: o[0].y - find_strip(h) }];
        windows::Win32::Graphics::Gdi::MapWindowPoints(None, Some(parent), &mut at);
        let pad = win::px(h, 5);
        let w = win::px(h, 320).min(cr.right - pad * 2).max(win::px(h, 60));
        win::move_to(e, at[0].x + pad, at[0].y + pad, w, find_strip(h) - pad * 2);
        // Its own style: the window it is in may not be shown yet.
        let visible = wm::GetWindowLongW(h, wm::GWL_STYLE) as u32 & wm::WS_VISIBLE.0 != 0;
        let _ = wm::SetWindowPos(e, Some(wm::HWND_TOP), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | if visible { wm::SWP_SHOWWINDOW } else { wm::SWP_HIDEWINDOW });
    }
}

fn close_find(h: HWND) {
    if with_opts(h, |o| o.find_mode) == FindMode::Always {
        if let Some(e) = find_box(h) {
            win::set_text(e, "");
        }
    } else if let Some(e) = find_box(h) {
        with_opts(h, |o| o.find_box = None);
        unsafe {
            let _ = wm::DestroyWindow(e);
        }
        frame_changed(h);
    }
    with_opts(h, |o| o.find.clear());
    apply_filter(h);
    unsafe {
        let _ = km::SetFocus(Some(h));
        let _ = windows::Win32::Graphics::Gdi::InvalidateRect(Some(h), None, true);
    }
}

/// The next row from `from` (round to the top) with a cell that has what is looked for: chosen.
fn find_next(h: HWND, from: usize) {
    let find = with_opts(h, |o| o.find.clone());
    if find.is_empty() {
        return;
    }
    let rows = unsafe { wm::SendMessageW(h, cc::LVM_GETITEMCOUNT, None, None).0 } as usize;
    let n = columns(h);
    for k in 0..rows {
        let r = (from + k) % rows.max(1);
        if (0..n).any(|c| column(h, c).visible && cell(h, r, c).to_lowercase().contains(&find)) {
            win::list_select(h, r);
            return;
        }
    }
}

unsafe extern "system" fn find_proc(e: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, grid: usize) -> LRESULT {
    let g = HWND(grid as *mut _);
    match msg {
        wm::WM_GETDLGCODE => LRESULT(4),
        wm::WM_KEYDOWN if w.0 == 0x0D => {
            find_next(g, chosen_row(g).map(|r| r + 1).unwrap_or(0));
            LRESULT(0)
        }
        wm::WM_KEYDOWN if w.0 == 0x1B => {
            close_find(g);
            LRESULT(0)
        }
        wm::WM_CHAR if w.0 == 0x0D || w.0 == 0x1B => LRESULT(0),
        // What is typed, cut, pasted or taken out: the grid looks again once the box has it.
        wm::WM_CHAR | wm::WM_PASTE | wm::WM_CUT | wm::WM_CLEAR | wm::WM_KEYDOWN | wm::WM_SETTEXT => {
            let r = unsafe { DefSubclassProc(e, msg, w, l) };
            unsafe {
                let _ = wm::PostMessageW(Some(g), wm::WM_APP + 0x7F, WPARAM(0), LPARAM(0));
            }
            r
        }
        _ => unsafe { DefSubclassProc(e, msg, w, l) },
    }
}

fn set_clipboard(h: HWND, text: &str) {
    use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData};
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    let w: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
    unsafe {
        if OpenClipboard(Some(h)).is_err() {
            return;
        }
        let _ = EmptyClipboard();
        if let Ok(mem) = GlobalAlloc(GMEM_MOVEABLE, w.len() * 2) {
            let p = GlobalLock(mem) as *mut u16;
            if !p.is_null() {
                std::ptr::copy_nonoverlapping(w.as_ptr(), p, w.len());
                let _ = GlobalUnlock(mem);
                let _ = SetClipboardData(13 /* CF_UNICODETEXT */, Some(windows::Win32::Foundation::HANDLE(mem.0)));
            }
        }
        let _ = CloseClipboard();
    }
}

fn get_clipboard(h: HWND) -> Option<String> {
    use windows::Win32::Foundation::HGLOBAL;
    use windows::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
    use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
    unsafe {
        OpenClipboard(Some(h)).ok()?;
        let out = (|| {
            let data = GetClipboardData(13).ok()?;
            let mem = HGLOBAL(data.0);
            let p = GlobalLock(mem) as *const u16;
            if p.is_null() {
                return None;
            }
            let mut n = 0;
            while *p.add(n) != 0 {
                n += 1;
            }
            let s = String::from_utf16_lossy(std::slice::from_raw_parts(p, n));
            let _ = GlobalUnlock(mem);
            Some(s)
        })();
        let _ = CloseClipboard();
        out
    }
}

/// The chosen rows as tab-separated lines (the shown columns; the headings first when it says),
/// as a spreadsheet takes them.
pub fn copy_rows(h: HWND) -> String {
    let n = columns(h);
    let shown: Vec<usize> = (0..n).filter(|c| column(h, *c).visible).collect();
    let mut lines = Vec::new();
    if with_opts(h, |o| o.copy_headers) {
        let header = header_of(h);
        let heads: Vec<String> = shown
            .iter()
            .map(|i| {
                let mut buf = [0u16; 260];
                let mut it = cc::HDITEMW { mask: cc::HDI_TEXT, pszText: windows::core::PWSTR(buf.as_mut_ptr()), cchTextMax: 260, ..Default::default() };
                unsafe {
                    wm::SendMessageW(header, cc::HDM_GETITEMW, Some(WPARAM(*i)), Some(LPARAM(&mut it as *mut _ as isize)));
                }
                String::from_utf16_lossy(&buf[..buf.iter().position(|c| *c == 0).unwrap_or(0)])
            })
            .collect();
        lines.push(heads.join("\t"));
    }
    let mut r = -1isize;
    loop {
        r = unsafe { wm::SendMessageW(h, cc::LVM_GETNEXTITEM, Some(WPARAM(r as usize)), Some(LPARAM(2))).0 };
        if r < 0 {
            break;
        }
        lines.push(shown.iter().map(|c| cell(h, r as usize, *c).replace(['\t', '\n', '\r'], " ")).collect::<Vec<_>>().join("\t"));
    }
    lines.join("\r\n")
}

/// Tab-separated lines pasted from the chosen row down: into the shown columns that may be
/// changed, rows added past the end when rows may be added. A line of the headings is passed by.
pub fn paste_rows(h: HWND, text: &str) {
    let n = columns(h);
    let shown: Vec<usize> = (0..n).filter(|c| column(h, *c).visible).collect();
    let start = chosen_row(h).unwrap_or(0);
    let mut row = start;
    let mut changed = false;
    let heads = copy_headers_line(h, &shown);
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        if line.trim_end() == heads {
            continue;
        }
        let rows = unsafe { wm::SendMessageW(h, cc::LVM_GETITEMCOUNT, None, None).0 } as usize;
        if row >= rows {
            if !with_opts(h, |o| o.add) {
                break;
            }
            let blank: Vec<&str> = vec![""; n.max(1)];
            grid_add(h, &blank);
        }
        for (k, v) in line.split('\t').enumerate() {
            let Some(&c) = shown.get(k) else { break };
            let col = column(h, c);
            if !col.read_only {
                set_cell(h, row, c, v.trim_end_matches('\r'));
                changed = true;
            }
        }
        row += 1;
    }
    if changed {
        tell_changed(h);
    }
}

fn copy_headers_line(h: HWND, shown: &[usize]) -> String {
    let header = header_of(h);
    shown
        .iter()
        .map(|i| {
            let mut buf = [0u16; 260];
            let mut it = cc::HDITEMW { mask: cc::HDI_TEXT, pszText: windows::core::PWSTR(buf.as_mut_ptr()), cchTextMax: 260, ..Default::default() };
            unsafe {
                wm::SendMessageW(header, cc::HDM_GETITEMW, Some(WPARAM(*i)), Some(LPARAM(&mut it as *mut _ as isize)));
            }
            String::from_utf16_lossy(&buf[..buf.iter().position(|c| *c == 0).unwrap_or(0)])
        })
        .collect::<Vec<_>>()
        .join("\t")
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
    let mut rows = shown_tagged_rows(h);
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
        win::list_add(h, &cells);
        set_tag(h, i, *t);
    }
    if grouped_view(h) {
        regroup(h);
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
    filter_buttons(h);
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

/// Adds a row at the end (into its group, when the rows are grouped).
pub fn grid_add(h: impl Into<HWND>, cells: &[&str]) {
    let h: HWND = h.into();
    win::list_add(h, cells);
    if grouped_view(h) {
        regroup(h);
    }
}

/// Empties the grid.
pub fn grid_clear(h: impl Into<HWND>) {
    let h: HWND = h.into();
    unsafe {
        wm::SendMessageW(h, cc::LVM_DELETEALLITEMS, None, None);
    }
}

/// Every row gone - those the find box set aside too.
pub fn grid_clear_all(h: impl Into<HWND>) {
    let h: HWND = h.into();
    with_opts(h, |o| o.hidden.clear());
    grid_clear(h);
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
    with_opts(h, |o| o.hidden.clear());
    for (i, r) in rows.iter().enumerate() {
        let cells: Vec<&str> = r.iter().map(String::as_str).collect();
        win::list_add(h, &cells);
        set_tag(h, i, i + 1);
    }
    // Sorted as it was, found as it was, grouped.
    if let Some((col, up)) = with_opts(h, |o| o.sorted) {
        sort_by(h, col, up);
    }
    apply_filter(h);
    regroup(h);
    filter_buttons(h);
    follow_master(h);
}

/// The find box's filter again: rows without what it looks for set aside, those set aside that
/// have it (or all, when it is empty) back - in their first order, then sorted as the grid is.
fn apply_filter(h: HWND) {
    let (find, filters, mut columns_kept, master) = with_opts(h, |o| (o.find.clone(), o.find_filters, o.filters.clone(), o.master_filter.clone()));
    let find = if filters { find } else { String::new() };
    let hidden = with_opts(h, |o| std::mem::take(&mut o.hidden));
    if find.is_empty() && hidden.is_empty() && columns_kept.is_empty() && master.is_none() {
        return;
    }
    // A detail grid: its master row's rows, as a filter of their own over the person's.
    if let Some((c, key)) = master {
        let kept = columns_kept.entry(c).or_insert_with(|| vec![key.clone()]);
        kept.retain(|v| *v == key);
        if kept.is_empty() {
            kept.push("\u{0}none".into());
        }
    }
    close_editor(h, true);
    let mut all = shown_tagged_rows(h);
    all.extend(hidden);
    // Their first order: by tag, the new ones (tag 0) after, as they were.
    let mut keyed: Vec<(usize, usize, (usize, Vec<String>))> = all.into_iter().enumerate().map(|(i, r)| (if r.0 == 0 { usize::MAX } else { r.0 }, i, r)).collect();
    keyed.sort_by_key(|(t, i, _)| (*t, *i));
    let n = columns(h);
    let shown_cols: Vec<usize> = (0..n).filter(|c| column(h, *c).visible).collect();
    let found = |r: &Vec<String>| find.is_empty() || shown_cols.iter().any(|c| r.get(*c).is_some_and(|v| v.to_lowercase().contains(&find)));
    let has = |r: &Vec<String>| found(r) && passes(&columns_kept, r);
    let (keep, out): (Vec<_>, Vec<_>) = keyed.into_iter().map(|(_, _, r)| r).partition(|(_, r)| has(r));
    unsafe {
        wm::SendMessageW(h, wm::WM_SETREDRAW, Some(WPARAM(0)), None);
    }
    grid_clear(h);
    for (i, (t, r)) in keep.iter().enumerate() {
        let cells: Vec<&str> = r.iter().map(String::as_str).collect();
        win::list_add(h, &cells);
        set_tag(h, i, *t);
    }
    with_opts(h, |o| o.hidden = out);
    unsafe {
        wm::SendMessageW(h, wm::WM_SETREDRAW, Some(WPARAM(1)), None);
    }
    if let Some((col, up)) = with_opts(h, |o| o.sorted) {
        sort_by(h, col, up);
    }
    regroup(h);
    redraw(h);
}

/// Whether a row's values pass every column's filter.
pub fn passes(filters: &std::collections::BTreeMap<usize, Vec<String>>, row: &[String]) -> bool {
    filters.iter().all(|(c, keep)| keep.iter().any(|k| k == row.get(*c).map(String::as_str).unwrap_or("")))
}

/// A column's filter: only the rows whose value in column `col` is one of `keep` are shown, the
/// others set aside as the find box's are (saved with the rest); `None` shows them all again -
/// Excel's ticks on a heading, DevExpress's column filter.
pub fn set_column_filter(h: HWND, col: usize, keep: Option<Vec<String>>) {
    with_opts(h, |o| match keep {
        Some(k) => {
            o.filters.insert(col, k);
        }
        None => {
            o.filters.remove(&col);
        }
    });
    apply_filter(h);
    redraw(header_of(h));
    redraw(h);
}

/// Column `col`'s filter: the values it keeps, `None` when it keeps all.
pub fn column_filter(h: HWND, col: usize) -> Option<Vec<String>> {
    with_opts(h, |o| o.filters.get(&col).cloned())
}

/// Every column's filter taken off.
pub fn clear_filters(h: HWND) {
    with_opts(h, |o| o.filters.clear());
    apply_filter(h);
    redraw(header_of(h));
    redraw(h);
}

/// AllowFilter: a drop-down on each heading opens the column's filter (on at first).
pub fn set_allow_filter(h: HWND, on: bool) {
    with_opts(h, |o| o.allow_filter = on);
    if !on {
        clear_filters(h);
    }
    filter_buttons(h);
}

/// AllowFilter, without making the grid one with properties of its own.
fn filters_allowed(h: HWND) -> bool {
    let m = OPTS.lock().unwrap_or_else(|e| e.into_inner());
    m.as_ref().and_then(|m| m.get(&(h.0 as isize))).is_none_or(|o| o.allow_filter)
}

/// The headings' drop-downs: on each shown column's while filters are allowed.
fn filter_buttons(h: HWND) {
    const HDF_SPLITBUTTON: i32 = 0x0100_0000;
    let on = filters_allowed(h);
    let header = header_of(h);
    for i in 0..columns(h) {
        let want = on && column(h, i).visible;
        let mut it = cc::HDITEMW { mask: cc::HDI_FORMAT, ..Default::default() };
        unsafe {
            wm::SendMessageW(header, cc::HDM_GETITEMW, Some(WPARAM(i)), Some(LPARAM(&mut it as *mut _ as isize)));
        }
        let fmt = if want { it.fmt.0 | HDF_SPLITBUTTON } else { it.fmt.0 & !HDF_SPLITBUTTON };
        if fmt != it.fmt.0 {
            it.fmt = cc::HEADER_CONTROL_FORMAT_FLAGS(fmt);
            unsafe {
                wm::SendMessageW(header, cc::HDM_SETITEMW, Some(WPARAM(i)), Some(LPARAM(&it as *const _ as isize)));
            }
        }
    }
}

/// The filter box of column `col`, opened from its heading's drop-down.
fn open_filter(h: HWND, col: usize) {
    close_editor(h, true);
    let header = header_of(h);
    let mut r = RECT::default();
    unsafe {
        wm::SendMessageW(header, 0x1219 /* HDM_GETITEMDROPDOWNRECT */, Some(WPARAM(col)), Some(LPARAM(&mut r as *mut _ as isize)));
        let mut pts = [POINT { x: r.left, y: r.top }, POINT { x: r.right, y: r.bottom }];
        windows::Win32::Graphics::Gdi::MapWindowPoints(Some(header), None, &mut pts);
        r = RECT { left: pts[0].x, top: pts[0].y, right: pts[1].x, bottom: pts[1].y };
    }
    // The values of the rows the other columns' filters leave, as Excel offers them.
    let others: std::collections::BTreeMap<usize, Vec<String>> = with_opts(h, |o| o.filters.iter().filter(|(c, _)| **c != col).map(|(c, v)| (*c, v.clone())).collect());
    let rows = grid_tagged_rows(h);
    let values = crate::gridfilter::distinct(rows.iter().filter(|(_, r)| passes(&others, r)).map(|(_, r)| r.get(col).cloned().unwrap_or_default()));
    let format = column(h, col).format;
    let shown: Vec<(String, String)> = values.into_iter().map(|v| {
        let s = if v.trim().is_empty() { crate::text::t("(empty)").to_string() } else { format_value(&format, &v) };
        (v, s)
    }).collect();
    let chosen = column_filter(h, col);
    match crate::gridfilter::ask(h, r, shown, chosen.as_deref()) {
        Some(crate::gridfilter::Choice::Filter(keep)) => set_column_filter(h, col, keep),
        Some(crate::gridfilter::Choice::Sort(up)) => sort_by(h, col, up),
        None => {}
    }
}

/// A detail grid, its master, the master's key column, its own, and the key it shows now.
type Detail = (isize, isize, usize, usize, String);

/// Every detail grid and its master.
static DETAILS: std::sync::Mutex<Vec<Detail>> = std::sync::Mutex::new(Vec::new());

/// Master-detail, as DevExpress's and WinForms' BindingSource have it: `detail` shows only the
/// rows whose column `detail_col` holds the value of column `master_col` in `master`'s chosen row
/// (none when no row is chosen), following it as another row is chosen; a row added to `detail`
/// takes that value. The other rows are set aside, never lost - a bound detail's Save writes them
/// all.
pub fn set_master(detail: HWND, master: HWND, master_col: usize, detail_col: usize) {
    {
        let mut d = DETAILS.lock().unwrap_or_else(|e| e.into_inner());
        d.retain(|x| x.0 != detail.0 as isize);
        d.push((detail.0 as isize, master.0 as isize, master_col, detail_col, "\u{0}unset".into()));
    }
    follow_master(master);
}

/// The detail grids of `master` shown again for its chosen row, when it is another than before.
pub fn follow_master(master: HWND) {
    let mine: Vec<(isize, usize, usize, String)> = DETAILS.lock().unwrap_or_else(|e| e.into_inner()).iter().filter(|x| x.1 == master.0 as isize).map(|x| (x.0, x.2, x.3, x.4.clone())).collect();
    if mine.is_empty() {
        return;
    }
    let row = chosen_row(master);
    for (detail, mc, dc, was) in mine {
        let key = row.map(|r| cell(master, r, mc)).unwrap_or_else(|| "\u{0}none".into());
        if key == was {
            continue;
        }
        for x in DETAILS.lock().unwrap_or_else(|e| e.into_inner()).iter_mut().filter(|x| x.0 == detail && x.1 == master.0 as isize) {
            x.4 = key.clone();
        }
        let d = HWND(detail as *mut _);
        if !unsafe { wm::IsWindow(Some(d)) }.as_bool() {
            continue;
        }
        with_opts(d, |o| o.master_filter = Some((dc, key)));
        apply_filter(d);
        // A detail's own details follow it too.
        follow_master(d);
    }
}

/// A detail grid's master key, for a row added to it.
fn master_key(h: HWND) -> Option<(usize, String)> {
    with_opts(h, |o| o.master_filter.clone()).filter(|(_, k)| !k.starts_with('\u{0}'))
}

/// HeaderMenu: a right-click on a heading offers sorting, filters, grouping and exporting.
pub fn set_header_menu(h: HWND, on: bool) {
    with_opts(h, |o| o.header_menu = on);
}

/// The menu of a right-click on a heading - DevExpress's column header menu: sort by the column,
/// its filter, every filter off, group by it, the rows to Excel, to a PDF, or previewed.
fn heading_menu(h: HWND, header: HWND) {
    const SORT_UP: u32 = 1;
    const SORT_DOWN: u32 = 2;
    const FILTER: u32 = 3;
    const CLEAR: u32 = 4;
    const CLEAR_ALL: u32 = 5;
    const GROUP: u32 = 6;
    const XLSX: u32 = 7;
    const PDF: u32 = 8;
    const PREVIEW: u32 = 9;
    const CSV: u32 = 10;
    let mut pt = POINT::default();
    unsafe {
        let _ = wm::GetCursorPos(&mut pt);
        let _ = windows::Win32::Graphics::Gdi::ScreenToClient(header, &mut pt);
    }
    let mut hit = cc::HDHITTESTINFO { pt, ..Default::default() };
    let col = unsafe { wm::SendMessageW(header, cc::HDM_HITTEST, None, Some(LPARAM(&mut hit as *mut _ as isize))).0 };
    let col = (col >= 0).then_some(col as usize).filter(|c| column(h, *c).visible);
    let t = crate::text::t;
    let mut items: Vec<(u32, &str)> = Vec::new();
    if let Some(c) = col {
        let cl = column(h, c);
        if cl.allow_sort {
            items.extend([(SORT_UP, t("Sort A to Z")), (SORT_DOWN, t("Sort Z to A")), (0, "")]);
        }
        if filters_allowed(h) {
            items.push((FILTER, t("Filter...")));
            if column_filter(h, c).is_some() {
                items.push((CLEAR, t("Clear filter")));
            }
        }
        items.push((GROUP, if cl.group { t("Ungroup") } else { t("Group by this column") }));
    }
    if with_opts(h, |o| !o.filters.is_empty()) {
        items.push((CLEAR_ALL, t("Clear all filters")));
    }
    if !items.is_empty() {
        items.push((0, ""));
    }
    items.extend([(XLSX, t("Export to Excel...")), (PDF, t("Export to PDF...")), (CSV, t("Export to CSV...")), (PREVIEW, t("Print preview..."))]);
    let chosen = win::popup(h, &items);
    let title = {
        let root = unsafe { wm::GetAncestor(h, wm::GA_ROOT) };
        let t = win::text_of(root);
        if t.is_empty() { "Report".to_string() } else { t }
    };
    let save = |filter: &str, ext: &str| crate::components::file_dialog(h, true, "", filter, "", ext, &title, false).into_iter().next();
    match (chosen, col) {
        (SORT_UP, Some(c)) => sort_by(h, c, true),
        (SORT_DOWN, Some(c)) => sort_by(h, c, false),
        (FILTER, Some(c)) => open_filter(h, c),
        (CLEAR, Some(c)) => set_column_filter(h, c, None),
        (CLEAR_ALL, _) => clear_filters(h),
        (GROUP, Some(c)) => {
            let n = columns(h);
            let on = !column(h, c).group;
            for i in 0..n {
                let mut k = column(h, i);
                let want = on && i == c;
                if k.group != want {
                    k.group = want;
                    set_column(h, i, k);
                }
            }
        }
        (XLSX, _) => {
            if let Some(p) = save(t("Excel workbook|*.xlsx"), "xlsx") {
                if let Err(e) = export_xlsx(h, &p) {
                    win::message(Some(h), &e.to_string(), &title, wm::MB_ICONWARNING);
                }
            }
        }
        (CSV, _) => {
            if let Some(p) = save(t("CSV file|*.csv"), "csv") {
                if let Err(e) = export_csv(h, &p) {
                    win::message(Some(h), &e.to_string(), &title, wm::MB_ICONWARNING);
                }
            }
        }
        (PDF, _) => {
            if let Some(p) = save(t("PDF document|*.pdf"), "pdf") {
                export_pdf(h, &p, &title);
            }
        }
        (PREVIEW, _) => print_preview(h, &title),
        _ => {}
    }
}

/// The rows grouped by the grouping column's values - Windows' list groups, a heading
/// "Status: Paid (3)" over each, folded with a click - or not grouped when no column groups.
pub fn regroup(h: HWND) {
    const LVM_ENABLEGROUPVIEW: u32 = 0x109D;
    const LVM_INSERTGROUP: u32 = 0x1091;
    const LVM_REMOVEALLGROUPS: u32 = 0x10A0;
    let n = columns(h);
    let g = (0..n).find(|c| column(h, *c).group);
    unsafe {
        wm::SendMessageW(h, LVM_REMOVEALLGROUPS, None, None);
        wm::SendMessageW(h, LVM_ENABLEGROUPVIEW, Some(WPARAM(g.is_some() as usize)), None);
    }
    let Some(g) = g else { return };
    let rows = unsafe { wm::SendMessageW(h, cc::LVM_GETITEMCOUNT, None, None).0 } as usize;
    let values: Vec<String> = (0..rows).map(|r| cell(h, r, g)).collect();
    let mut groups: Vec<String> = values.clone();
    groups.sort_by_key(|v| v.to_lowercase());
    groups.dedup();
    // The column's heading, for the groups' headings.
    let caption = {
        let mut buf = [0u16; 260];
        let mut it = cc::HDITEMW { mask: cc::HDI_TEXT, pszText: windows::core::PWSTR(buf.as_mut_ptr()), cchTextMax: 260, ..Default::default() };
        unsafe {
            wm::SendMessageW(header_of(h), cc::HDM_GETITEMW, Some(WPARAM(g)), Some(LPARAM(&mut it as *mut _ as isize)));
        }
        String::from_utf16_lossy(&buf[..buf.iter().position(|c| *c == 0).unwrap_or(0)])
    };
    let format = column(h, g).format;
    for (k, v) in groups.iter().enumerate() {
        let count = values.iter().filter(|x| *x == v).count();
        let shown = if v.trim().is_empty() { crate::text::t("(empty)").to_string() } else { format_value(&format, v) };
        let mut text = win::wide(&format!("{caption}: {shown} ({count})"));
        let group = cc::LVGROUP {
            cbSize: std::mem::size_of::<cc::LVGROUP>() as u32,
            mask: cc::LVGF_HEADER | cc::LVGF_GROUPID | cc::LVGF_STATE,
            pszHeader: windows::core::PWSTR(text.as_mut_ptr()),
            iGroupId: k as i32 + 1,
            stateMask: cc::LVGS_COLLAPSIBLE,
            state: cc::LVGS_COLLAPSIBLE,
            ..Default::default()
        };
        unsafe {
            wm::SendMessageW(h, LVM_INSERTGROUP, Some(WPARAM(usize::MAX)), Some(LPARAM(&group as *const _ as isize)));
        }
    }
    for (r, v) in values.iter().enumerate() {
        let id = groups.iter().position(|x| x == v).map(|i| i as i32 + 1).unwrap_or(1);
        let item = cc::LVITEMW { mask: cc::LVIF_GROUPID, iItem: r as i32, iGroupId: id, ..Default::default() };
        unsafe {
            wm::SendMessageW(h, cc::LVM_SETITEMW, None, Some(LPARAM(&item as *const _ as isize)));
        }
    }
}

/// Whether a column groups the rows.
fn grouped_view(h: HWND) -> bool {
    with_opts(h, |o| o.cols.iter().any(|c| c.group))
}

/// FindFilter: the find box leaves only the rows it finds (else it only marks them).
pub fn set_find_filters(h: HWND, on: bool) {
    with_opts(h, |o| o.find_filters = on);
    apply_filter(h);
}

/// The shown rows and columns written to a CSV file (UTF-8, as Excel reads Thai), the headings
/// first, values as they are shown.
pub fn export_csv(h: HWND, path: &std::path::Path) -> std::io::Result<()> {
    let n = columns(h);
    let shown: Vec<usize> = (0..n).filter(|c| column(h, *c).visible).collect();
    let quote = |s: &str| if s.contains([',', '"', '\n', '\r']) { format!("\"{}\"", s.replace('"', "\"\"")) } else { s.to_string() };
    let mut out = String::from("\u{feff}");
    out += &copy_headers_line(h, &shown).split('\t').map(quote).collect::<Vec<_>>().join(",");
    out += "\r\n";
    let rows = unsafe { wm::SendMessageW(h, cc::LVM_GETITEMCOUNT, None, None).0 } as usize;
    for r in 0..rows {
        out += &shown.iter().map(|c| quote(&format_value(&column(h, *c).format, &cell(h, r, *c)))).collect::<Vec<_>>().join(",");
        out += "\r\n";
    }
    std::fs::write(path, out)
}

/// The shown columns, their headings and their widths in design pixels.
fn shown_columns(h: HWND) -> (Vec<usize>, Vec<String>, Vec<i32>) {
    let n = columns(h);
    let shown: Vec<usize> = (0..n).filter(|c| column(h, *c).visible).collect();
    let heads: Vec<String> = copy_headers_line(h, &shown).split('\t').map(String::from).collect();
    let scale = win::px(h, 100).max(1);
    let widths = shown.iter().map(|c| unsafe { wm::SendMessageW(h, cc::LVM_GETCOLUMNWIDTH, Some(WPARAM(*c)), None).0 as i32 } * 100 / scale).collect();
    (shown, heads, widths)
}

/// The rows shown and their totals written to an Excel workbook (.xlsx): numbers and dates kept
/// as Excel's, with the columns' formats, the headings bold and frozen with a filter on them -
/// DevExpress's ExportToXlsx.
pub fn export_xlsx(h: HWND, path: &std::path::Path) -> std::io::Result<()> {
    use crate::xlsx::{cell_of, Cell, Sheet};
    let (shown, headings, widths) = shown_columns(h);
    let cols: Vec<Column> = shown.iter().map(|c| column(h, *c)).collect();
    let rows = unsafe { wm::SendMessageW(h, cc::LVM_GETITEMCOUNT, None, None).0 } as usize;
    let data: Vec<Vec<Cell>> = (0..rows).map(|r| shown.iter().zip(&cols).map(|(c, col)| cell_of(&col.format, &cell(h, r, *c))).collect()).collect();
    let totals = cols.iter().any(|c| c.summary != Summary::None).then(|| {
        shown
            .iter()
            .zip(&cols)
            .map(|(c, col)| match summary_text(h, *c, col).split_once(" = ") {
                Some((_, v)) => match (col.summary, cell_of(&col.format, v)) {
                    (_, Cell::Text(_) | Cell::Empty) => Cell::Text(v.to_string()),
                    (Summary::Count, Cell::Number(x, _)) => Cell::Number(x, "#,##0".into()),
                    (_, n) => n,
                },
                None => Cell::Empty,
            })
            .collect()
    });
    let name = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let sheet = Sheet { name, headings, widths: widths.iter().map(|w| (*w as f64 / 7.0).clamp(4.0, 80.0)).collect(), rows: data, totals };
    crate::xlsx::write(path, &sheet)
}

/// The rows shown printed into a PDF at `path` - the title at the top, the headings on every
/// page, the totals at the end, the page's number at its foot - through Windows' own
/// "Microsoft Print to PDF": how many pages (0 when it could not).
pub fn export_pdf(h: HWND, path: &std::path::Path, title: &str) -> usize {
    let (report, data) = grid_report(h, title);
    let pdf = crate::print::printers().into_iter().find(|p| p.contains("PDF")).unwrap_or_else(|| "Microsoft Print to PDF".into());
    report.document(h, data).print_to_file(&pdf, path)
}

/// The grid's rows as they would print, in a preview window - DevExpress's ShowPrintPreview.
pub fn print_preview(h: HWND, title: &str) {
    let (report, data) = grid_report(h, title);
    report.preview(h, data);
}

/// The grid as a report: its shown columns across the page, as wide as the grid has them (the
/// page turned when they are wide), its rows' values as the grid shows them.
pub fn grid_report(h: HWND, title: &str) -> (crate::report::Report, crate::report::Data) {
    let (shown, headings, widths) = shown_columns(h);
    let cols: Vec<Column> = shown.iter().map(|c| column(h, *c)).collect();
    let totals: Vec<String> = shown.iter().zip(&cols).map(|(i, c)| summary_text(h, *i, c)).collect();
    let rows = unsafe { wm::SendMessageW(h, cc::LVM_GETITEMCOUNT, None, None).0 } as usize;
    let values: Vec<Vec<String>> = (0..rows).map(|r| shown.iter().zip(&cols).map(|(c, col)| format_value(&col.format, &cell(h, r, *c))).collect()).collect();
    report_of(title, &headings, &widths, &cols, &totals, values)
}

/// A report of a table: `headings` across the page as wide as `widths` share it, `values` row by
/// row, `totals` (a column's, or empty) at the end.
pub fn report_of(title: &str, headings: &[String], widths: &[i32], cols: &[Column], totals: &[String], values: Vec<Vec<String>>) -> (crate::report::Report, crate::report::Data) {
    use crate::print::Align as PAlign;
    use crate::report::{Band, BandKind, Data, Item, ItemKind, Report};
    let total_px: i32 = widths.iter().sum::<i32>().max(1);
    // A portrait page is 8.5 inches; wide tables turn it.
    let landscape = total_px > 760;
    let page = if landscape { 1100 } else { 850 } - 100;
    let mut x = 0;
    let at: Vec<(i32, i32)> = widths
        .iter()
        .map(|w| {
            let wd = w * page / total_px;
            let r = (x, wd);
            x += wd;
            r
        })
        .collect();
    let align = |i: usize| match cols.get(i).map(|c| c.align).unwrap_or_default() {
        Align::Left => PAlign::Left,
        Align::Center => PAlign::Center,
        Align::Right => PAlign::Right,
    };
    let names: Vec<String> = (0..headings.len()).map(|i| format!("c{i}")).collect();
    let mut head: Vec<Item> = at.iter().zip(headings).enumerate().map(|(i, (p, t))| Item { kind: ItemKind::Label, text: t.clone(), bounds: [p.0 + 3, 3, p.1 - 6, 20], align: align(i), ..Default::default() }).collect();
    head.push(Item { kind: ItemKind::Line, bounds: [0, 24, page, 0], ..Default::default() });
    let mut bands = vec![
        Band { kind: BandKind::ReportHeader, height: 45, items: vec![Item { kind: ItemKind::Label, text: title.to_string(), bounds: [0, 0, page, 35], ..Default::default() }], ..Default::default() },
        Band { kind: BandKind::PageHeader, height: 25, items: head, ..Default::default() },
        Band {
            kind: BandKind::Detail,
            height: 20,
            items: at.iter().zip(&names).enumerate().map(|(i, (p, n))| Item { kind: ItemKind::Field, field: n.clone(), bounds: [p.0 + 3, 2, p.1 - 6, 17], align: align(i), ..Default::default() }).collect(),
            ..Default::default()
        },
        Band {
            kind: BandKind::PageFooter,
            height: 25,
            items: vec![Item { kind: ItemKind::Label, text: crate::text::t("Page {page} of {pages}").to_string(), bounds: [0, 5, page, 18], align: PAlign::Right, ..Default::default() }],
            ..Default::default()
        },
    ];
    if totals.iter().any(|t| !t.is_empty()) {
        let mut items: Vec<Item> = at.iter().zip(totals).enumerate().filter(|(_, (_, t))| !t.is_empty()).map(|(i, (p, t))| Item { kind: ItemKind::Label, text: t.clone(), bounds: [p.0 + 3, 6, p.1 - 6, 18], align: align(i), bold: Some(true), ..Default::default() }).collect();
        items.push(Item { kind: ItemKind::Line, bounds: [0, 2, page, 0], ..Default::default() });
        bands.push(Band { kind: BandKind::ReportFooter, height: 28, items, ..Default::default() });
    }
    (Report { title: title.to_string(), landscape, bands, ..Default::default() }, Data::new(names, values))
}

/// The rows chosen, by place.
pub fn selected_rows(h: HWND) -> Vec<usize> {
    let mut out = Vec::new();
    let mut r = -1isize;
    loop {
        r = unsafe { wm::SendMessageW(h, cc::LVM_GETNEXTITEM, Some(WPARAM(r as usize)), Some(LPARAM(2))).0 };
        if r < 0 {
            break;
        }
        out.push(r as usize);
    }
    out
}

/// Looks for `text` as the find box does (an empty one shows every row again).
pub fn find(h: HWND, text: &str) {
    with_opts(h, |o| o.find = text.trim().to_lowercase());
    apply_filter(h);
    find_next(h, 0);
    redraw(h);
}

/// The grid's rows with their tags: the place each had when [`grid_fill`] put it there, 0 for a row
/// added since.
/// Rows the find box left out count too: they are the grid's, only not shown.
pub fn grid_tagged_rows(h: impl Into<HWND>) -> Vec<(usize, Vec<String>)> {
    let h: HWND = h.into();
    let mut rows = shown_tagged_rows(h);
    rows.extend(with_opts(h, |o| o.hidden.clone()));
    rows
}

/// The rows shown, with their tags.
fn shown_tagged_rows(h: HWND) -> Vec<(usize, Vec<String>)> {
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
        if c.group {
            regroup(h);
        }
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

unsafe extern "system" fn grid_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM, a: usize, b: usize) -> LRESULT {
    let r = unsafe { grid_proc_inner(h, msg, w, l, a, b) };
    // What may choose another row: a master's details follow.
    if matches!(msg, wm::WM_LBUTTONDOWN | wm::WM_LBUTTONUP | wm::WM_KEYDOWN | wm::WM_KEYUP | wm::WM_RBUTTONDOWN) || msg == cc::LVM_SETITEMSTATE || msg == cc::LVM_DELETEITEM || msg == cc::LVM_DELETEALLITEMS {
        follow_master(h);
    }
    if msg == wm::WM_NCDESTROY {
        DETAILS.lock().unwrap_or_else(|e| e.into_inner()).retain(|x| x.0 != h.0 as isize && x.1 != h.0 as isize);
    }
    r
}

unsafe fn grid_proc_inner(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, _: usize) -> LRESULT {
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
            if hdr.code == cc::NM_CUSTOMDRAW {
                if let Some(r) = header_custom_draw(h, l) {
                    return r;
                }
            }
            if hdr.code == cc::NM_RCLICK && with_opts(h, |o| o.header_menu) {
                heading_menu(h, hdr.hwndFrom);
                return LRESULT(1);
            }
            match hdr.code as i32 {
                -321 /* HDN_ITEMCHANGEDW */ | -301 /* HDN_ITEMCHANGEDA */ => {
                    let r = unsafe { DefSubclassProc(h, msg, w, l) };
                    redraw(h);
                    place_find(h);
                    return r;
                }
                -318 /* HDN_DROPDOWN */ => {
                    if filters_allowed(h) && column(h, i).visible {
                        open_filter(h, i);
                    }
                    return LRESULT(0);
                }
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
        m if m == wm::WM_APP + 0x7F => {
            let text = find_box(h).map(win::text_of).unwrap_or_default().trim().to_lowercase();
            if text != with_opts(h, |o| o.find.clone()) {
                with_opts(h, |o| o.find = text);
                apply_filter(h);
                find_next(h, 0);
                unsafe {
                    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(Some(h), None, true);
                }
            }
            LRESULT(0)
        }
        // The find box follows the grid: moved, sized, shown or hidden with it.
        wm::WM_WINDOWPOSCHANGED | wm::WM_SHOWWINDOW => {
            let r = unsafe { DefSubclassProc(h, msg, w, l) };
            place_find(h);
            r
        }
        wm::WM_KEYDOWN if unsafe { km::GetKeyState(0x11) } < 0 && matches!(w.0, 0x46 | 0x43 | 0x56 | 0x41) => {
            match w.0 {
                0x46 /* Ctrl+F */ => open_find(h, true),
                0x43 /* Ctrl+C */ => {
                    let text = copy_rows(h);
                    set_clipboard(h, &text);
                }
                0x56 /* Ctrl+V */ => {
                    if !is_read_only(h) {
                        if let Some(text) = get_clipboard(h) {
                            paste_rows(h, &text);
                        }
                    }
                }
                _ /* Ctrl+A */ => {
                    let it = cc::LVITEMW { stateMask: cc::LVIS_SELECTED, state: cc::LVIS_SELECTED, ..Default::default() };
                    unsafe {
                        wm::SendMessageW(h, cc::LVM_SETITEMSTATE, Some(WPARAM(usize::MAX)), Some(LPARAM(&it as *const _ as isize)));
                    }
                }
            }
            LRESULT(0)
        }
        wm::WM_NCCALCSIZE if w.0 != 0 => {
            let r = unsafe { DefSubclassProc(h, msg, w, l) };
            let fh = footer_height(h);
            let p = unsafe { &mut *(l.0 as *mut wm::NCCALCSIZE_PARAMS) };
            if fh > 0 {
                p.rgrc[0].bottom -= fh;
            }
            p.rgrc[0].top += find_strip(h);
            r
        }
        wm::WM_NCPAINT | wm::WM_PAINT => {
            let r = unsafe { DefSubclassProc(h, msg, w, l) };
            paint_footer(h);
            paint_find_strip(h);
            r
        }
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
                    if let Some((c, key)) = master_key(h) {
                        set_cell(h, last, c, &key);
                    }
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
            place_find(h);
            r
        }
        wm::WM_NCDESTROY => {
            if let Some(e) = find_box(h) {
                unsafe {
                    let _ = wm::DestroyWindow(e);
                }
            }
            OPTS.lock().unwrap_or_else(|e| e.into_inner()).as_mut().map(|m| m.remove(&(h.0 as isize)));
            unsafe { DefSubclassProc(h, msg, w, l) }
        }
        m if m == wm::WM_APP + 0x7E => {
            grid_idle(h);
            LRESULT(0)
        }
        wm::WM_VSCROLL | wm::WM_HSCROLL | wm::WM_MOUSEWHEEL | wm::WM_MOUSEHWHEEL => {
            close_editor(h, true);
            let r = unsafe { DefSubclassProc(h, msg, w, l) };
            // The footer's totals follow their columns sideways.
            if msg != wm::WM_VSCROLL {
                redraw(h);
            }
            r
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

    /// The find box sets rows aside, never loses them: a binding's save still sees every row
    /// with its tag, and an empty find gives them back in their first order. Grouping keeps
    /// every row's cells where they were.
    #[test]
    fn finding_sets_rows_aside_and_gives_them_back() {
        win::init();
        extern "system" fn host_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
            win::def(h, msg, w, l)
        }
        let class = win::register("PhaiGridFindHost", Some(host_proc), None);
        let host = unsafe { wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_OVERLAPPEDWINDOW, 0, 0, 400, 300, None, None, Some(win::instance()), None).unwrap_or_default() };
        let font = win::font(host, 0, false);
        let g = grid(host, 5, font, &[("Name", 120), ("Status", 80)]);
        let rows: Vec<Vec<String>> = [["Somchai", "Open"], ["Malee", "Paid"], ["Anan", "Paid"]].iter().map(|r| r.iter().map(|s| s.to_string()).collect()).collect();
        grid_fill(g, &rows);
        find(g, "PAID");
        assert_eq!(grid_rows(g).len(), 2, "only the rows found are shown");
        let mut all = grid_tagged_rows(g);
        all.sort();
        assert_eq!(all.iter().map(|(t, _)| *t).collect::<Vec<_>>(), vec![1, 2, 3], "every row is still the grid's");
        find(g, "");
        assert_eq!(grid_rows(g), rows, "all back, in their first order");
        set_column(g, 1, Column { group: true, ..Column::new() });
        assert_eq!(grid_rows(g), rows, "grouping moves no cell");
        unsafe {
            let _ = wm::DestroyWindow(host);
        }
    }

    /// Values shown as their column's format says, kept as they are underneath; what does not
    /// fit a format shown as it is.
    #[test]
    fn display_formats() {
        assert_eq!(format_value("n2", "1234.5"), "1,234.50");
        assert_eq!(format_value("n0", "-1234567"), "-1,234,567");
        assert_eq!(format_value("c2", "99"), "฿99.00");
        assert_eq!(format_value("p0", "0.15"), "15%");
        assert_eq!(format_value("d", "2026-11-21"), "21/11/2026");
        assert_eq!(format_value("dt", "2026-11-21 08:30:00"), "21/11/2026 08:30");
        assert_eq!(format_value("n2", "abc"), "abc");
        assert_eq!(format_value("", "x"), "x");
    }

    /// A rule's line read and written back; numbers compared as numbers.
    #[test]
    fn format_rules() {
        let r = Rule::parse("> 1000 back=#ffe4e4 fore=red row").unwrap();
        assert_eq!((r.op.as_str(), r.value.as_str(), r.back.as_str(), r.fore.as_str(), r.row), (">", "1000", "#ffe4e4", "red", true));
        assert_eq!(Rule::parse(&r.text()), Some(r.clone()));
        assert!(r.matches("1,500") && !r.matches("999") && !r.matches("200"));
        let c = Rule::parse("contains \"ส่งแล้ว\" fore=green").unwrap();
        assert!(c.matches("ส่งแล้ววันนี้") && !c.matches("รอส่ง"));
        assert!(Rule::parse("empty back=#eee").unwrap().matches("  "));
        assert!(Rule::parse("between 1 2").is_none() && Rule::parse("> 1 colour=red").is_none());
        assert_eq!(rules(&["= 1 back=#eee", "nonsense"]).len(), 1);
    }

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
