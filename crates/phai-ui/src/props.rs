//! The WinForms properties a control is given after it is made - ReadOnly, MaxLength,
//! PlaceholderText, GridLines and the rest - one function each, named as the `.phaiform` names
//! them. The generated code calls them by name; the designer calls [`apply`] with the same words,
//! so the canvas shows what the program will.

use std::collections::HashMap;
use std::sync::Mutex;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::win::{self, Kind};

fn send(h: HWND, msg: u32, w: usize, l: isize) -> isize {
    unsafe { wm::SendMessageW(h, msg, Some(WPARAM(w)), Some(LPARAM(l))).0 }
}

/// Turns style bits on or off, and has Windows draw the frame again.
fn style_bits(h: HWND, bits: u32, on: bool) {
    unsafe {
        let s = wm::GetWindowLongW(h, wm::GWL_STYLE) as u32;
        let n = if on { s | bits } else { s & !bits };
        if n != s {
            wm::SetWindowLongW(h, wm::GWL_STYLE, n as i32);
            let _ = wm::SetWindowPos(h, None, 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOZORDER | wm::SWP_NOACTIVATE | wm::SWP_FRAMECHANGED);
            let _ = windows::Win32::Graphics::Gdi::InvalidateRect(Some(h), None, true);
        }
    }
}

fn list_ex(h: HWND, bit: isize, on: bool) {
    send(h, cc::LVM_SETEXTENDEDLISTVIEWSTYLE, bit as usize, if on { bit } else { 0 });
}

fn is(h: HWND, k: Kind) -> bool {
    win::kind_of(h) == Some(k)
}

/// A rich text box: Windows' RichEdit class.
fn is_rich(h: HWND) -> bool {
    let mut buf = [0u16; 32];
    let n = unsafe { wm::GetClassNameW(h, &mut buf) } as usize;
    String::from_utf16_lossy(&buf[..n]).starts_with("RICHEDIT")
}

fn is_tree(h: HWND) -> bool {
    is(h, Kind::Tree)
}

// --- every control --------------------------------------------------------------------------

/// TabStop: whether Tab stops on it.
pub fn tab_stop(h: HWND, on: bool) {
    style_bits(h, wm::WS_TABSTOP.0, on);
}

static TIPS: Mutex<Option<HashMap<isize, isize>>> = Mutex::new(None);

/// ToolTip: words shown when the mouse rests on the control. Empty takes them away.
pub fn tooltip(h: HWND, text: &str) {
    unsafe {
        let root = wm::GetAncestor(h, wm::GA_ROOT);
        let tip = {
            let mut tips = TIPS.lock().unwrap_or_else(|e| e.into_inner());
            let map = tips.get_or_insert_with(HashMap::new);
            match map.get(&(root.0 as isize)).map(|t| HWND(*t as *mut _)).filter(|t| wm::IsWindow(Some(*t)).as_bool()) {
                Some(t) => t,
                None => {
                    let class = win::wide("tooltips_class32");
                    let t = wm::CreateWindowExW(wm::WS_EX_TOPMOST, PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WINDOW_STYLE(0x01 /* TTS_ALWAYSTIP */ | 0x02 /* TTS_NOPREFIX */), 0, 0, 0, 0, Some(root), None, Some(win::instance()), None).unwrap_or_default();
                    send(t, 0x0418 /* TTM_SETMAXTIPWIDTH */, 0, win::px(root, 400) as isize);
                    map.insert(root.0 as isize, t.0 as isize);
                    t
                }
            }
        };
        let mut words = win::wide(text);
        let mut ti = cc::TTTOOLINFOW { cbSize: std::mem::size_of::<cc::TTTOOLINFOW>() as u32, uFlags: cc::TTF_IDISHWND | cc::TTF_SUBCLASS, hwnd: root, uId: h.0 as usize, lpszText: windows::core::PWSTR(words.as_mut_ptr()), ..Default::default() };
        send(tip, cc::TTM_DELTOOLW, 0, &ti as *const _ as isize);
        if !text.is_empty() {
            ti.lpszText = windows::core::PWSTR(words.as_mut_ptr());
            send(tip, cc::TTM_ADDTOOLW, 0, &ti as *const _ as isize);
        }
    }
}

static CURSORS: Mutex<Option<HashMap<isize, u16>>> = Mutex::new(None);

/// Cursor: the mouse pointer over the control - `default`, `hand`, `ibeam`, `wait`, `cross`,
/// `help`, `no`, `size_all`, `size_we`, `size_ns`.
pub fn cursor(h: HWND, name: &str) {
    let id: u16 = match name {
        "hand" => 32649,
        "ibeam" => 32513,
        "wait" => 32514,
        "cross" => 32515,
        "help" => 32651,
        "no" => 32648,
        "size_all" => 32646,
        "size_we" => 32644,
        "size_ns" => 32645,
        _ => 0,
    };
    let mut c = CURSORS.lock().unwrap_or_else(|e| e.into_inner());
    let map = c.get_or_insert_with(HashMap::new);
    if id == 0 {
        map.remove(&(h.0 as isize));
    } else {
        map.insert(h.0 as isize, id);
        unsafe {
            let _ = SetWindowSubclass(h, Some(cursor_proc), 0x5043, 0);
        }
    }
}

unsafe extern "system" fn cursor_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, _: usize) -> LRESULT {
    if msg == wm::WM_SETCURSOR && (l.0 & 0xFFFF) as u32 == wm::HTCLIENT {
        let id = CURSORS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|m| m.get(&(h.0 as isize)).copied());
        if let Some(id) = id {
            unsafe {
                if let Ok(c) = wm::LoadCursorW(None, PCWSTR(id as usize as *const u16)) {
                    wm::SetCursor(Some(c));
                }
            }
            return LRESULT(1);
        }
    }
    unsafe { DefSubclassProc(h, msg, w, l) }
}

// --- words ------------------------------------------------------------------------------------

/// TextAlign: `left`, `center` or `right`.
pub fn text_align(h: HWND, align: &str) {
    let bits = match align {
        "center" => 1,
        "right" => 2,
        _ => 0,
    };
    match win::kind_of(h) {
        Some(Kind::Button) => win::set_align(h, align),
        // STATIC's SS_LEFT/CENTER/RIGHT and EDIT's ES_LEFT/CENTER/RIGHT are the same two bits.
        _ => {
            style_bits(h, 0x3, false);
            style_bits(h, bits, true);
        }
    }
}

/// BorderStyle: `fixed` - a thin line round it - or `none`.
pub fn border(h: HWND, style: &str) {
    style_bits(h, wm::WS_BORDER.0, style == "fixed");
}

/// ReadOnly: its words can be read and copied, not changed.
pub fn read_only(h: HWND, on: bool) {
    if win::kind_of(h) == Some(Kind::List) {
        // A grid is a list that edits.
        crate::grid::set_read_only(h, on);
        return;
    }
    send(h, 0x00CF /* EM_SETREADONLY */, on as usize, 0);
}

/// MaxLength: the most letters it takes.
pub fn max_length(h: HWND, n: i32) {
    if is_rich(h) {
        send(h, 0x0435 /* EM_EXLIMITTEXT */, 0, n as isize);
    } else {
        send(h, 0x00C5 /* EM_LIMITTEXT */, n.max(0) as usize, 0);
    }
}

/// PlaceholderText: grey words while it is empty.
pub fn placeholder(h: HWND, text: &str) {
    let w = win::wide(text);
    send(h, 0x1501 /* EM_SETCUEBANNER */, 1, w.as_ptr() as isize);
}

/// CharacterCasing: `normal`, `upper` or `lower`.
pub fn char_casing(h: HWND, casing: &str) {
    style_bits(h, 0x0008 | 0x0010, false);
    match casing {
        "upper" => style_bits(h, 0x0008, true),
        "lower" => style_bits(h, 0x0010, true),
        _ => {}
    }
}

/// WordWrap: long lines wrap at the edge; off, the box scrolls sideways.
pub fn word_wrap(h: HWND, on: bool) {
    send(h, 0x0448 /* EM_SETTARGETDEVICE */, 0, (!on) as isize);
}

/// DetectUrls: web addresses show as links.
pub fn detect_urls(h: HWND, on: bool) {
    send(h, 0x045B /* EM_AUTOURLDETECT */, on as usize, 0);
}

// --- numbers and ranges ------------------------------------------------------------------------

fn arrows(h: HWND) -> HWND {
    unsafe { wm::GetWindow(h, wm::GW_CHILD).unwrap_or_default() }
}

/// Increment: what an arrow of a number box adds or takes away.
pub fn increment(h: HWND, n: i32) {
    #[repr(C)]
    struct Accel {
        sec: u32,
        inc: u32,
    }
    let a = Accel { sec: 0, inc: n.max(1) as u32 };
    send(arrows(h), 0x046B /* UDM_SETACCEL */, 1, &a as *const _ as isize);
}

/// Hexadecimal: the number in base 16.
pub fn hexadecimal(h: HWND, on: bool) {
    let ud = arrows(h);
    let pos = send(ud, 0x0472 /* UDM_GETPOS32 */, 0, 0);
    if on {
        style_bits(h, 0x2000 /* ES_NUMBER */, false);
    }
    send(ud, 0x046D /* UDM_SETBASE */, if on { 16 } else { 10 }, 0);
    send(ud, 0x0471 /* UDM_SETPOS32 */, 0, pos);
}

/// ThousandsSeparator: 1,000.
pub fn thousands(h: HWND, on: bool) {
    let ud = arrows(h);
    let pos = send(ud, 0x0472 /* UDM_GETPOS32 */, 0, 0);
    style_bits(ud, 0x0080 /* UDS_NOTHOUSANDS */, !on);
    send(ud, 0x0471 /* UDM_SETPOS32 */, 0, pos);
}

/// SmallChange: how far a slider's arrow keys move it.
pub fn small_change(h: HWND, n: i32) {
    win::with_range(h, |r| r.small = n.max(1));
}

/// LargeChange: how far Page Up and Page Down move a slider.
pub fn large_change(h: HWND, n: i32) {
    win::with_range(h, |r| r.large = n.max(1));
}

/// TickFrequency: a tick under the slider every so many; 0 none.
pub fn tick_frequency(h: HWND, n: i32) {
    win::with_range(h, |r| r.ticks = n.max(0));
}

/// Step: what [`perform_step`] adds to a progress bar.
pub fn step(h: HWND, n: i32) {
    win::with_range(h, |r| r.step = n);
}

/// Adds the progress bar's Step to its value - WinForms' PerformStep.
pub fn perform_step(h: HWND) {
    let step = unsafe { win::state::<win::Range>(h) }.map(|r| r.step).unwrap_or(10);
    win::set_value(h, win::value(h) + step);
}

/// Style = Marquee: a block that runs along, for work of unknown length.
pub fn marquee(h: HWND, on: bool) {
    win::with_range(h, |r| r.marquee = on);
    unsafe {
        if on {
            wm::SetTimer(Some(h), 1, 30, None);
        } else {
            let _ = wm::KillTimer(Some(h), 1);
        }
    }
}

// --- lists -------------------------------------------------------------------------------------

/// SelectedIndex: the item chosen, from 0; -1 none.
pub fn selected_index(h: HWND, i: i32) {
    send(h, 0x014E /* CB_SETCURSEL */, i as isize as usize, 0);
}

/// MaxDropDownItems: how many items the open list shows.
pub fn max_drop_down_items(h: HWND, n: i32) {
    send(h, 0x1701 /* CB_SETMINVISIBLE */, n.max(1) as usize, 0);
}

/// View: `details`, `list`, `small_icons` or `large_icons`.
pub fn view(h: HWND, v: &str) {
    let n = match v {
        "large_icons" => 0,
        "small_icons" => 2,
        "list" => 3,
        _ => 1,
    };
    send(h, 0x108E /* LVM_SETVIEW */, n, 0);
}

/// GridLines: lines between rows and columns.
pub fn grid_lines(h: HWND, on: bool) {
    list_ex(h, 0x1, on);
}

/// FullRowSelect: a click chooses the whole row.
pub fn full_row_select(h: HWND, on: bool) {
    if is_tree(h) {
        style_bits(h, 0x1000 /* TVS_FULLROWSELECT */, on);
    } else {
        list_ex(h, 0x20, on);
    }
}

/// ShowColumnHeaders: the row of headings over a grid's columns.
pub fn show_column_headers(h: HWND, on: bool) {
    style_bits(h, 0x4000 /* LVS_NOCOLUMNHEADER */, !on);
}

/// ColumnAutoWidth: the columns share the grid's width.
pub fn column_auto_width(h: HWND, on: bool) {
    crate::grid::set_auto_width(h, on);
}

/// AllowAddRows: a double-click under the last row adds one.
pub fn allow_add_rows(h: HWND, on: bool) {
    crate::grid::set_allow_add_rows(h, on);
}

/// AllowDeleteRows: Delete takes the chosen row out.
pub fn allow_delete_rows(h: HWND, on: bool) {
    crate::grid::set_allow_delete_rows(h, on);
}

/// EditorShowMode: `double_click`, `click` or `f2` opens a cell for typing.
pub fn editor_show_mode(h: HWND, mode: &str) {
    crate::grid::set_edit_on(
        h,
        match mode {
            "click" => crate::grid::EditOn::Click,
            "f2" => crate::grid::EditOn::F2,
            _ => crate::grid::EditOn::DoubleClick,
        },
    );
}

/// EnterMoveNextColumn: Enter keeps the cell and opens the next.
pub fn enter_moves_next(h: HWND, on: bool) {
    crate::grid::set_enter_moves_next(h, on);
}

/// AllowColumnMoving: columns dragged by their headings into another order.
pub fn allow_column_moving(h: HWND, on: bool) {
    list_ex(h, 0x10 /* LVS_EX_HEADERDRAGDROP */, on);
}

/// MultiSelect: several rows chosen at once.
pub fn multi_select(h: HWND, on: bool) {
    style_bits(h, 0x0004 /* LVS_SINGLESEL */, !on);
}

/// CheckBoxes: a box to tick before each item.
pub fn check_boxes(h: HWND, on: bool) {
    if is_tree(h) {
        style_bits(h, 0x0100 /* TVS_CHECKBOXES */, on);
    } else {
        list_ex(h, 0x4, on);
    }
}

/// HeaderStyle: `clickable` headings, or `none`.
pub fn header(h: HWND, style: &str) {
    style_bits(h, 0x4000 /* LVS_NOCOLUMNHEADER */, style == "none");
}

/// HideSelection: the chosen row stops showing when the focus goes elsewhere.
pub fn hide_selection(h: HWND, on: bool) {
    style_bits(h, if is_tree(h) { 0x0020 } else { 0x0008 } /* TVS_ / LVS_SHOWSELALWAYS */, !on);
}

/// ShowLines: dotted lines join a tree's items to their parents.
pub fn show_lines(h: HWND, on: bool) {
    style_bits(h, 0x0002 /* TVS_HASLINES */, on);
}

/// ShowPlusMinus: the buttons that open and close a tree's items.
pub fn show_plus_minus(h: HWND, on: bool) {
    style_bits(h, 0x0001 /* TVS_HASBUTTONS */, on);
}

/// ShowRootLines: the top items have their buttons too.
pub fn show_root_lines(h: HWND, on: bool) {
    style_bits(h, 0x0004 /* TVS_LINESATROOT */, on);
}

/// Opens (or closes) every item of a tree - WinForms' ExpandAll and CollapseAll.
pub fn expand_all(h: HWND, on: bool) {
    const TVM_EXPAND: u32 = 0x1102;
    const TVM_GETNEXTITEM: u32 = 0x110A;
    fn walk(h: HWND, item: isize, on: bool) {
        let mut it = item;
        while it != 0 {
            send(h, TVM_EXPAND, if on { 2 } else { 1 }, it);
            walk(h, send(h, TVM_GETNEXTITEM, 4 /* TVGN_CHILD */, it), on);
            it = send(h, TVM_GETNEXTITEM, 1 /* TVGN_NEXT */, it);
        }
    }
    walk(h, send(h, TVM_GETNEXTITEM, 0 /* TVGN_ROOT */, 0), on);
}

// --- dates -------------------------------------------------------------------------------------

static CUSTOM: Mutex<Vec<isize>> = Mutex::new(Vec::new());

/// Format: `short`, `long`, `time`, or `custom` (then [`custom_format`]).
pub fn format(h: HWND, f: &str) {
    style_bits(h, 0x000C | 0x0004 | 0x0009, false);
    style_bits(h, match f {
        "long" => 0x0004,
        "time" => 0x0009,
        _ => 0x000C,
    }, true);
    send(h, 0x1032 /* DTM_SETFORMATW */, 0, 0);
    let mut c = CUSTOM.lock().unwrap_or_else(|e| e.into_inner());
    c.retain(|x| *x != h.0 as isize);
    if f == "custom" {
        c.push(h.0 as isize);
    }
}

/// CustomFormat: `dd MMM yyyy`, `HH:mm` - shown when the Format is `custom`.
pub fn custom_format(h: HWND, f: &str) {
    if CUSTOM.lock().unwrap_or_else(|e| e.into_inner()).contains(&(h.0 as isize)) {
        let w = win::wide(f);
        send(h, 0x1032 /* DTM_SETFORMATW */, 0, w.as_ptr() as isize);
    }
}

fn day(s: &str) -> Option<windows::Win32::Foundation::SYSTEMTIME> {
    let mut it = s.trim().split('-').map(|p| p.parse::<u16>().ok());
    let (y, m, d) = (it.next()??, it.next()??, it.next()??);
    Some(windows::Win32::Foundation::SYSTEMTIME { wYear: y, wMonth: m, wDay: d, ..Default::default() })
}

/// Value: the date shown, `2026-11-19`.
pub fn date(h: HWND, s: &str) {
    if let Some(st) = day(s) {
        win::set_date(h, st.wYear as i32, st.wMonth as i32, st.wDay as i32);
    }
}

fn date_range(h: HWND, s: &str, max: bool) {
    let Some(st) = day(s) else { return };
    let mut two = [windows::Win32::Foundation::SYSTEMTIME::default(); 2];
    let flag = send(h, 0x1003 /* DTM_GETRANGE */, 0, two.as_mut_ptr() as isize);
    two[max as usize] = st;
    let keep = flag & if max { 1 } else { 2 };
    send(h, 0x1004 /* DTM_SETRANGE */, (keep | if max { 2 } else { 1 }) as usize, two.as_ptr() as isize);
}

/// MinDate: the earliest date taken.
pub fn min_date(h: HWND, s: &str) {
    date_range(h, s, false);
}

/// MaxDate: the latest date taken.
pub fn max_date(h: HWND, s: &str) {
    date_range(h, s, true);
}

/// AutoSize: a label as wide as its words.
pub fn auto_size(h: HWND, on: bool) {
    win::set_auto_size(h, on)
}

/// ThreeState: a check box's click goes unticked, ticked, neither.
pub fn three_state(h: HWND, on: bool) {
    win::set_three_state(h, on)
}

/// CheckState = Indeterminate when the window opens.
pub fn indeterminate(h: HWND, on: bool) {
    if on {
        win::set_check_state(h, 2)
    }
}

/// Appearance: `normal`, or `button` - pressed while ticked.
pub fn appearance(h: HWND, a: &str) {
    win::set_push_like(h, a == "button")
}

/// Orientation of a slider: `horizontal` or `vertical`.
pub fn orientation(h: HWND, o: &str) {
    win::with_range(h, |r| r.vertical = o == "vertical");
}

/// LabelEdit: a tree's item renamed in place (a slow second click, or F2).
pub fn label_edit(h: HWND, on: bool) {
    style_bits(h, 0x0008 /* TVS_EDITLABELS */, on);
}

/// AllowDrop: files dragged from Explorer can be dropped on it - its DragDrop event.
pub fn allow_drop(h: HWND, on: bool) {
    unsafe { windows::Win32::UI::Shell::DragAcceptFiles(h, on) };
}

/// SizeMode of a picture: `zoom`, `normal`, `stretch` or `center`.
pub fn size_mode(h: HWND, mode: &str) {
    crate::picture::set_size_mode(h, mode);
}

/// The properties [`apply`] knows, by their `.phaiform` names, with what their values are: `b` true
/// or false, `n` a whole number, `t` words - how the Live Inspector writes them into a form.
pub const NAMES: &[(&str, char)] = &[
    ("tab_stop", 'b'), ("tooltip", 't'), ("cursor", 't'), ("text_align", 't'), ("border", 't'), ("read_only", 'b'),
    ("max_length", 'n'), ("placeholder", 't'), ("char_casing", 't'), ("word_wrap", 'b'), ("detect_urls", 'b'),
    ("increment", 'n'), ("hexadecimal", 'b'), ("thousands", 'b'), ("small_change", 'n'), ("large_change", 'n'),
    ("tick_frequency", 'n'), ("step", 'n'), ("marquee", 'b'), ("selected_index", 'n'), ("max_drop_down_items", 'n'),
    ("view", 't'), ("grid_lines", 'b'), ("full_row_select", 'b'), ("multi_select", 'b'), ("check_boxes", 'b'),
    ("header", 't'), ("hide_selection", 'b'), ("show_lines", 'b'), ("show_plus_minus", 'b'), ("show_root_lines", 'b'),
    ("expand_all", 'b'), ("format", 't'), ("custom_format", 't'), ("date", 't'), ("min_date", 't'), ("max_date", 't'),
    ("size_mode", 't'), ("auto_size", 'b'), ("three_state", 'b'), ("indeterminate", 'b'), ("appearance", 't'),
    ("orientation", 't'), ("label_edit", 'b'), ("allow_drop", 'b'),
];

/// One property by its `.phaiform` name and its value as words - `true`, `40`, `left` - as the
/// designer has them. A name this does not know does nothing.
pub fn apply(h: HWND, name: &str, value: &str) {
    let yes = value == "true";
    let n = value.trim().parse::<i32>().unwrap_or(0);
    match name {
        "tab_stop" => tab_stop(h, yes),
        "tooltip" => tooltip(h, value),
        "cursor" => cursor(h, value),
        "text_align" => text_align(h, value),
        "border" => border(h, value),
        "read_only" => read_only(h, yes),
        "max_length" => max_length(h, n),
        "placeholder" => placeholder(h, value),
        "char_casing" => char_casing(h, value),
        "word_wrap" => word_wrap(h, yes),
        "detect_urls" => detect_urls(h, yes),
        "increment" => increment(h, n),
        "hexadecimal" => hexadecimal(h, yes),
        "thousands" => thousands(h, yes),
        "small_change" => small_change(h, n),
        "large_change" => large_change(h, n),
        "tick_frequency" => tick_frequency(h, n),
        "step" => step(h, n),
        "marquee" => marquee(h, yes),
        "selected_index" => selected_index(h, n),
        "max_drop_down_items" => max_drop_down_items(h, n),
        "view" => view(h, value),
        "grid_lines" => grid_lines(h, yes),
        "full_row_select" => full_row_select(h, yes),
        "multi_select" => multi_select(h, yes),
        "show_column_headers" => show_column_headers(h, yes),
        "column_auto_width" => column_auto_width(h, yes),
        "allow_add_rows" => allow_add_rows(h, yes),
        "allow_delete_rows" => allow_delete_rows(h, yes),
        "editor_show_mode" => editor_show_mode(h, value),
        "enter_moves_next" => enter_moves_next(h, yes),
        "allow_column_moving" => allow_column_moving(h, yes),
        "check_boxes" => check_boxes(h, yes),
        "header" => header(h, value),
        "hide_selection" => hide_selection(h, yes),
        "show_lines" => show_lines(h, yes),
        "show_plus_minus" => show_plus_minus(h, yes),
        "show_root_lines" => show_root_lines(h, yes),
        "expand_all" => expand_all(h, yes),
        "format" => format(h, value),
        "custom_format" => custom_format(h, value),
        "date" => date(h, value),
        "min_date" => min_date(h, value),
        "max_date" => max_date(h, value),
        "size_mode" => size_mode(h, value),
        "auto_size" => auto_size(h, yes),
        "three_state" => three_state(h, yes),
        "indeterminate" => indeterminate(h, yes),
        "appearance" => appearance(h, value),
        "orientation" => orientation(h, value),
        "label_edit" => label_edit(h, yes),
        "allow_drop" => allow_drop(h, yes),
        _ => {}
    }
}
