//! The Data grid's column editor, as DevExpress's Grid Designer has its Columns page: the columns
//! down the left - add, remove, move, or retrieve them from the bound table - and the chosen
//! one's properties beside them: caption, field, width, shown, alignment, read only, how a cell
//! is edited (text, number, date, check box, drop-down and its items), sorting and sizing, how
//! its values are shown, its footer total and its format rules.

use phai_form::ColumnProps;
use phai_ui::text::{t, tf};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::WindowsAndMessaging as wm;

/// A grid column as the editor holds it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Col {
    pub caption: String,
    pub width: i32,
    /// Its field in the bound table (empty when the grid is not bound).
    pub field: String,
    pub props: ColumnProps,
}

/// The bound table's fields, read when asked for (Retrieve Fields).
pub type Fields = Box<dyn Fn() -> Result<Vec<String>, String>>;

const ID_LIST: i32 = 10;
const ID_ADD: i32 = 11;
const ID_REMOVE: i32 = 12;
const ID_UP: i32 = 13;
const ID_DOWN: i32 = 14;
const ID_RETRIEVE: i32 = 15;
const ID_CAPTION: i32 = 20;
const ID_FIELD: i32 = 21;
const ID_WIDTH: i32 = 22;
const ID_VISIBLE: i32 = 23;
const ID_ALIGN: i32 = 24;
const ID_READ_ONLY: i32 = 25;
const ID_EDITOR: i32 = 26;
const ID_ITEMS: i32 = 27;
const ID_SORT: i32 = 28;
const ID_ALLOW_SORT: i32 = 29;
const ID_ALLOW_SIZE: i32 = 30;
const ID_FORMAT: i32 = 31;
const ID_SUMMARY: i32 = 32;
const ID_RULES: i32 = 33;
const ID_OK: i32 = 1;
const ID_CANCEL: i32 = 2;

const ALIGNS: [&str; 3] = ["left", "center", "right"];
const EDITORS: [&str; 5] = ["text", "number", "date", "check", "combo"];
const SORTS: [&str; 3] = ["", "asc", "desc"];
const SUMMARIES: [&str; 6] = ["", "sum", "count", "avg", "min", "max"];
/// The formats offered (others may be typed): none, numbers, baht, percent, dates.
const FORMATS: [&str; 9] = ["", "n0", "n2", "c0", "c2", "p0", "p2", "d", "dt"];

struct Dialog {
    cols: Vec<Col>,
    bound: bool,
    fields: Option<Fields>,
    /// The fields the Field box offers.
    known: Vec<String>,
    list: HWND,
    /// Labels and their boxes, in rows.
    rows: Vec<(HWND, HWND)>,
    caption: HWND,
    field: HWND,
    width: HWND,
    visible: HWND,
    align: HWND,
    read_only: HWND,
    editor: HWND,
    items: HWND,
    sort: HWND,
    allow_sort: HWND,
    allow_size: HWND,
    format: HWND,
    summary: HWND,
    rules: HWND,
    buttons: Vec<(i32, HWND)>,
    /// The boxes being filled: their changes are not the person's.
    loading: bool,
    ok: bool,
    done: bool,
}

fn state<'a>(h: HWND) -> Option<&'a mut Dialog> {
    unsafe { win::state::<Dialog>(h) }
}

/// Edits `cols`; the columns as they are when OK is pressed. `fields` reads the bound table's
/// fields - `None` for a grid that is not bound.
pub fn edit(owner: HWND, cols: Vec<Col>, fields: Option<Fields>) -> Option<Vec<Col>> {
    win::init();
    let class = win::register("PhaiColumnEditor", Some(proc_), None);
    let title = win::wide(t("Grid Columns"));
    unsafe {
        let hwnd = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_THICKFRAME | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let list = win::list(hwnd, ID_LIST, font, &[(t("Column"), 150), (t("Field"), 130)]);
        win::grid_lines(list);
        let known: Vec<String> = cols.iter().map(|c| c.field.clone()).filter(|f| !f.is_empty()).collect();
        let bound = fields.is_some();
        let mut rows = Vec::new();
        let mut row = |label: &str, b: HWND| {
            rows.push((win::label(hwnd, label, 0, font), b));
            b
        };
        let caption = row(t("Caption"), win::edit(hwnd, "", ID_CAPTION, font, false));
        let known_refs: Vec<&str> = known.iter().map(String::as_str).collect();
        let field = row(t("Field name"), win::combo_with(hwnd, ID_FIELD, font, &known_refs, true));
        win::enable(field, bound);
        let width = row(t("Width"), win::edit(hwnd, "", ID_WIDTH, font, false));
        let visible = row("", win::checkbox(hwnd, t("Visible"), ID_VISIBLE, font, true));
        let align = row(t("Alignment"), win::combo(hwnd, ID_ALIGN, font, &[t("Left"), t("Center"), t("Right")]));
        let read_only = row("", win::checkbox(hwnd, t("Read only"), ID_READ_ONLY, font, false));
        let editor = row(t("Edited as"), win::combo(hwnd, ID_EDITOR, font, &[t("Text"), t("Number"), t("Date"), t("Check box"), t("Drop-down")]));
        let items = row(t("Drop-down items"), win::edit(hwnd, "", ID_ITEMS, font, false));
        let sort = row(t("Sorted"), win::combo(hwnd, ID_SORT, font, &[t("No"), t("Ascending"), t("Descending")]));
        let allow_sort = row("", win::checkbox(hwnd, t("A click on its heading sorts"), ID_ALLOW_SORT, font, true));
        let allow_size = row("", win::checkbox(hwnd, t("Its width can be dragged"), ID_ALLOW_SIZE, font, true));
        let format = row(t("Display format"), win::combo_with(hwnd, ID_FORMAT, font, &FORMATS, true));
        let summary = row(t("Footer total"), win::combo(hwnd, ID_SUMMARY, font, &[t("None"), t("Sum"), t("Count"), t("Average"), t("Min"), t("Max")]));
        let rules = row(t("Format rules"), win::button(hwnd, t("Format Rules..."), ID_RULES, font, false));
        let mut buttons = Vec::new();
        for (id, text, glyph) in [(ID_ADD, t("Add"), 0xE710u16), (ID_REMOVE, t("Remove"), 0xE74D), (ID_UP, t("Move Up"), 0xE74A), (ID_DOWN, t("Move Down"), 0xE74B), (ID_RETRIEVE, t("Retrieve Fields"), 0xE72C), (ID_OK, t("OK"), 0), (ID_CANCEL, t("Cancel"), 0)] {
            let b = win::button(hwnd, text, id, font, id == ID_OK);
            if glyph != 0 {
                win::set_button_glyph(b, glyph);
            }
            buttons.push((id, b));
        }
        if !bound {
            if let Some((_, b)) = buttons.iter().find(|(i, _)| *i == ID_RETRIEVE) {
                win::enable(*b, false);
            }
        }
        win::set_state(hwnd, Box::new(Dialog { cols, bound, fields, known, list, rows, caption, field, width, visible, align, read_only, editor, items, sort, allow_sort, allow_size, format, summary, rules, buttons, loading: false, ok: false, done: false }));
        fill_list(hwnd, 0);
        let (w, h) = (win::px(hwnd, 820), win::px(hwnd, 560));
        let _ = wm::SetWindowPos(hwnd, None, 0, 0, w, h, wm::SWP_NOZORDER | wm::SWP_NOMOVE);
        win::center_on(hwnd, owner);
        layout(hwnd);
        win::theme_frame(hwnd);
        win::enable(owner, false);
        win::show_top(hwnd);
        let mut msg = wm::MSG::default();
        while state(hwnd).is_some_and(|d| !d.done) {
            if !wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
                wm::PostQuitMessage(msg.wParam.0 as i32);
                break;
            }
            if !wm::IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
        }
        let out = state(hwnd).filter(|d| d.ok).map(|d| d.cols.clone());
        win::enable(owner, true);
        let _ = wm::SetForegroundWindow(owner);
        let _ = wm::DestroyWindow(hwnd);
        out
    }
}

fn chosen(d: &Dialog) -> Option<usize> {
    let i = unsafe { wm::SendMessageW(d.list, cc::LVM_GETNEXTITEM, Some(WPARAM(usize::MAX)), Some(LPARAM(2))).0 };
    usize::try_from(i).ok().filter(|i| *i < d.cols.len())
}

/// The list again, row `at` chosen and its properties shown.
fn fill_list(h: HWND, at: usize) {
    let Some(d) = state(h) else { return };
    unsafe {
        wm::SendMessageW(d.list, cc::LVM_DELETEALLITEMS, None, None);
    }
    for c in &d.cols {
        let caption = if c.props.visible == Some(false) { tf("{0} (hidden)", &[&c.caption]) } else { c.caption.clone() };
        win::list_add(d.list, &[&caption, &c.field]);
    }
    if !d.cols.is_empty() {
        win::list_select(d.list, at.min(d.cols.len() - 1));
    }
    load(h);
}

fn set_combo(h: HWND, i: usize) {
    unsafe {
        wm::SendMessageW(h, 0x014E /* CB_SETCURSEL */, Some(WPARAM(i)), None);
    }
}

/// The chosen column's properties into the boxes.
fn load(h: HWND) {
    let Some(d) = state(h) else { return };
    d.loading = true;
    let at = chosen(d);
    let c = at.and_then(|i| d.cols.get(i)).cloned().unwrap_or_default();
    for (_, b) in &d.rows {
        win::enable(*b, at.is_some());
    }
    win::enable(d.field, at.is_some() && d.bound);
    win::set_text(d.caption, &c.caption);
    win::set_text(d.field, &c.field);
    win::set_text(d.width, &c.width.to_string());
    win::set_checked(d.visible, c.props.visible != Some(false));
    set_combo(d.align, ALIGNS.iter().position(|a| Some(*a) == c.props.align.as_deref()).unwrap_or(0));
    win::set_checked(d.read_only, c.props.read_only == Some(true));
    let ed = EDITORS.iter().position(|a| Some(*a) == c.props.editor.as_deref()).unwrap_or(0);
    set_combo(d.editor, ed);
    win::set_text(d.items, &c.props.items.join(", "));
    win::enable(d.items, at.is_some() && ed == 4);
    set_combo(d.sort, SORTS.iter().position(|a| Some(*a) == c.props.sort.as_deref()).unwrap_or(0));
    win::set_checked(d.allow_sort, c.props.allow_sort != Some(false));
    win::set_checked(d.allow_size, c.props.allow_size != Some(false));
    win::set_text(d.format, c.props.format.as_deref().unwrap_or(""));
    set_combo(d.summary, SUMMARIES.iter().position(|a| Some(*a) == c.props.summary.as_deref()).unwrap_or(0));
    win::set_text(d.rules, &if c.props.rules.is_empty() { t("Format Rules...").to_string() } else { tf("Format Rules ({0})...", &[&c.props.rules.len()]) });
    d.loading = false;
}

/// The boxes into the chosen column.
fn store(h: HWND) {
    let Some(d) = state(h) else { return };
    if d.loading {
        return;
    }
    let Some(i) = chosen(d) else { return };
    let pick = |b: HWND, words: &[&str]| -> Option<String> {
        let k = win::combo_selected(b).max(0) as usize;
        words.get(k).filter(|w| !w.is_empty() && k > 0).map(|w| w.to_string())
    };
    let c = &mut d.cols[i];
    c.caption = win::text_of(d.caption).trim().to_string();
    c.field = win::text_of(d.field).trim().to_string();
    if let Ok(w) = win::text_of(d.width).trim().parse::<i32>() {
        c.width = w.clamp(10, 2000);
    }
    c.props.visible = (!win::is_checked(d.visible)).then_some(false);
    c.props.align = pick(d.align, &ALIGNS);
    c.props.read_only = win::is_checked(d.read_only).then_some(true);
    c.props.editor = pick(d.editor, &EDITORS);
    c.props.items = if c.props.editor.as_deref() == Some("combo") { win::text_of(d.items).split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect() } else { vec![] };
    c.props.sort = pick(d.sort, &SORTS);
    c.props.allow_sort = (!win::is_checked(d.allow_sort)).then_some(false);
    c.props.allow_size = (!win::is_checked(d.allow_size)).then_some(false);
    let f = win::text_of(d.format).trim().to_string();
    c.props.format = (!f.is_empty()).then_some(f);
    c.props.summary = pick(d.summary, &SUMMARIES);
    let caption = if c.props.visible == Some(false) { tf("{0} (hidden)", &[&c.caption]) } else { c.caption.clone() };
    let field = c.field.clone();
    let combo = c.props.editor.as_deref() == Some("combo");
    win::enable(d.items, combo);
    phai_ui::grid::set_cell(d.list, i, 0, &caption);
    phai_ui::grid::set_cell(d.list, i, 1, &field);
}

fn command(h: HWND, id: i32, code: u32) {
    let Some(d) = state(h) else { return };
    match id {
        ID_ADD => {
            let n = d.cols.len() + 1;
            d.cols.push(Col { caption: tf("Column{0}", &[&n]), width: 120, ..Default::default() });
            let last = d.cols.len() - 1;
            fill_list(h, last);
            let _ = unsafe { windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(d.caption)) };
        }
        ID_REMOVE => {
            if let Some(i) = chosen(d) {
                d.cols.remove(i);
                fill_list(h, i.saturating_sub(usize::from(i == d.cols.len())));
            }
        }
        ID_UP | ID_DOWN => {
            if let Some(i) = chosen(d) {
                let j = if id == ID_UP { i.checked_sub(1) } else { (i + 1 < d.cols.len()).then_some(i + 1) };
                if let Some(j) = j {
                    d.cols.swap(i, j);
                    fill_list(h, j);
                }
            }
        }
        ID_RETRIEVE => {
            let Some(read) = d.fields.as_ref() else { return };
            match read() {
                Ok(fields) => {
                    // Every field of the table: those already there kept as they are, the rest added.
                    let mut added = 0;
                    for f in &fields {
                        if !d.cols.iter().any(|c| c.field == *f) {
                            d.cols.push(Col { caption: f.clone(), width: 120, field: f.clone(), ..Default::default() });
                            added += 1;
                        }
                    }
                    d.known = fields.clone();
                    unsafe {
                        wm::SendMessageW(d.field, 0x014B /* CB_RESETCONTENT */, None, None);
                        for f in &fields {
                            let w = win::wide(f);
                            wm::SendMessageW(d.field, 0x0143 /* CB_ADDSTRING */, None, Some(LPARAM(w.as_ptr() as isize)));
                        }
                    }
                    fill_list(h, chosen(d).unwrap_or(0));
                    win::message(Some(h), &tf("{0} fields added", &[&added]), t("Grid Columns"), wm::MB_ICONINFORMATION);
                }
                Err(e) => {
                    win::message(Some(h), &e, t("Grid Columns"), wm::MB_ICONWARNING);
                }
            }
        }
        ID_OK => {
            store(h);
            d.ok = true;
            d.done = true;
        }
        ID_CANCEL => d.done = true,
        ID_CAPTION | ID_WIDTH | ID_ITEMS if code == 0x0300 /* EN_CHANGE */ => store(h),
        ID_FIELD if code == 5 /* CBN_EDITCHANGE */ => store(h),
        ID_FIELD if code == 1 /* CBN_SELCHANGE */ => {
            // The text follows the choice only after this message: the choice itself is taken.
            let k = win::combo_selected(d.field);
            if let Some(f) = usize::try_from(k).ok().and_then(|k| d.known.get(k)).cloned() {
                d.loading = true;
                win::set_text(d.field, &f);
                d.loading = false;
                // A caption still the old field's name follows the new one.
                if let Some(i) = chosen(d) {
                    if d.cols[i].caption.is_empty() || d.cols[i].caption == d.cols[i].field {
                        d.loading = true;
                        win::set_text(d.caption, &f);
                        d.loading = false;
                    }
                }
                store(h);
            }
        }
        ID_ALIGN | ID_EDITOR | ID_SORT | ID_SUMMARY if code == 1 => store(h),
        ID_FORMAT if code == 5 /* CBN_EDITCHANGE */ => store(h),
        ID_FORMAT if code == 1 => {
            let k = win::combo_selected(d.format);
            if let Some(f) = usize::try_from(k).ok().and_then(|k| FORMATS.get(k)) {
                d.loading = true;
                win::set_text(d.format, f);
                d.loading = false;
                store(h);
            }
        }
        ID_RULES => {
            let Some(i) = chosen(d) else { return };
            let now = d.cols[i].props.rules.join("\n");
            let help = t("One rule a line: a test, then the colours - the whole row with row. Tests: = != > >= < <= contains empty notempty. Like: > 1000 back=#ffe4e4 fore=#b00020 row");
            if let Some(text) = phai_ui::dialogs::ask_lines(Some(h), t("Format Rules"), help, &now) {
                let mut kept = Vec::new();
                let mut bad = Vec::new();
                for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
                    match phai_ui::grid::Rule::parse(line) {
                        Some(r) => kept.push(r.text()),
                        None => bad.push(line.to_string()),
                    }
                }
                if !bad.is_empty() {
                    win::message(Some(h), &tf("Not rules, left out:\n{0}", &[&bad.join("\n")]), t("Format Rules"), wm::MB_ICONWARNING);
                }
                d.cols[i].props.rules = kept;
                load(h);
            }
        }
        ID_VISIBLE | ID_READ_ONLY | ID_ALLOW_SORT | ID_ALLOW_SIZE => store(h),
        _ => {}
    }
}

fn layout(h: HWND) {
    let Some(d) = state(h) else { return };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let px = |v| win::px(h, v);
    let (m, row, gap) = (px(12), px(24), px(6));
    let button = |id: i32| d.buttons.iter().find(|(i, _)| *i == id).map(|(_, b)| *b).unwrap_or_default();
    let by = r.bottom - m - row;
    let list_w = (r.right - 2 * m) * 45 / 100;
    // The columns, and the buttons that change them under the list.
    let bh = row * 2 + gap;
    win::move_to(d.list, m, m, list_w, by - gap * 3 - bh - m);
    let ly = by - gap * 2 - bh;
    let bw = (list_w - gap * 2) / 3;
    for (k, id) in [ID_ADD, ID_REMOVE, ID_RETRIEVE].iter().enumerate() {
        win::move_to(button(*id), m + k as i32 * (bw + gap), ly, bw, row);
    }
    for (k, id) in [ID_UP, ID_DOWN].iter().enumerate() {
        win::move_to(button(*id), m + k as i32 * (bw + gap), ly + row + gap, bw, row);
    }
    // The chosen column's properties beside them.
    let x = m + list_w + px(16);
    let label_w = px(130);
    let box_w = r.right - m - x - label_w - gap;
    let mut y = m;
    for (label, b) in &d.rows {
        win::move_to(*label, x, y + px(3), label_w, row);
        let combo = win::kind_of(*b) == Some(win::Kind::Combo);
        win::move_to(*b, x + label_w + gap, y, box_w, if combo { px(200) } else { row });
        y += row + gap;
    }
    win::move_to(button(ID_CANCEL), r.right - m - px(90), by, px(90), row);
    win::move_to(button(ID_OK), r.right - m - px(90) * 2 - gap, by, px(90), row);
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(d) = state(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            layout(h);
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.hwndFrom == d.list && hdr.code == cc::LVN_ITEMCHANGED {
                let nm = unsafe { &*(l.0 as *const cc::NMLISTVIEW) };
                if nm.uNewState & cc::LVIS_SELECTED.0 != 0 {
                    load(h);
                }
                return LRESULT(0);
            }
            win::def(h, msg, w, l)
        }
        wm::WM_COMMAND => {
            command(h, win::loword(w.0) as i32, win::hiword(w.0) as u32);
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            d.done = true;
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}
