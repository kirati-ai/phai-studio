//! The rest of WinForms' everyday controls: ListBox, CheckedListBox, LinkLabel, MaskedTextBox and
//! MonthCalendar - Windows' own, themed as the others are. (Panel is [`crate::win::panel`].)

use std::collections::HashMap;
use std::sync::Mutex;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, SYSTEMTIME, WPARAM};
use windows::Win32::Graphics::Gdi::HFONT;
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{self as wm, WINDOW_EX_STYLE, WINDOW_STYLE};

use crate::theme::Color;
use crate::win::{self, wide, Kind};

fn send(h: HWND, msg: u32, w: usize, l: isize) -> isize {
    unsafe { wm::SendMessageW(h, msg, Some(WPARAM(w)), Some(LPARAM(l))).0 }
}

// --- ListBox -------------------------------------------------------------------------------------

const LB_ADDSTRING: u32 = 0x0180;
const LB_DELETESTRING: u32 = 0x0182;
const LB_RESETCONTENT: u32 = 0x0184;
const LB_SETSEL: u32 = 0x0185;
const LB_SETCURSEL: u32 = 0x0186;
const LB_GETSEL: u32 = 0x0187;
const LB_GETCURSEL: u32 = 0x0188;
const LB_GETTEXT: u32 = 0x0189;
const LB_GETTEXTLEN: u32 = 0x018A;
const LB_GETCOUNT: u32 = 0x018B;
const LB_INSERTSTRING: u32 = 0x0181;
/// Sent to the window as `WM_COMMAND` when the chosen item changes.
pub const LBN_SELCHANGE: u16 = 1;

/// A list of lines to choose from - WinForms' ListBox. `multi`: several, with Ctrl and Shift;
/// `sorted`: in alphabetical order.
pub fn list_box(parent: HWND, id: i32, f: HFONT, items: &[&str], multi: bool, sorted: bool) -> HWND {
    const LBS_NOTIFY: u32 = 0x0001;
    const LBS_SORT: u32 = 0x0002;
    const LBS_NOINTEGRALHEIGHT: u32 = 0x0100;
    const LBS_EXTENDEDSEL: u32 = 0x0800;
    let style = LBS_NOTIFY | LBS_NOINTEGRALHEIGHT | if multi { LBS_EXTENDEDSEL } else { 0 } | if sorted { LBS_SORT } else { 0 };
    let h = win::child(parent, "LISTBOX", "", WINDOW_STYLE(style) | wm::WS_TABSTOP | wm::WS_VSCROLL | wm::WS_BORDER, WINDOW_EX_STYLE(0), id, f);
    win::register_kind(h, Kind::List);
    for it in items {
        list_box_add(h, it);
    }
    win::theme_native(h);
    h
}

pub fn list_box_add(h: HWND, s: &str) -> usize {
    let w = wide(s);
    send(h, LB_ADDSTRING, 0, w.as_ptr() as isize).max(0) as usize
}

pub fn list_box_insert(h: HWND, at: usize, s: &str) {
    let w = wide(s);
    send(h, LB_INSERTSTRING, at, w.as_ptr() as isize);
}

pub fn list_box_remove_at(h: HWND, i: usize) {
    send(h, LB_DELETESTRING, i, 0);
}

pub fn list_box_clear(h: HWND) {
    send(h, LB_RESETCONTENT, 0, 0);
}

pub fn list_box_count(h: HWND) -> usize {
    send(h, LB_GETCOUNT, 0, 0).max(0) as usize
}

pub fn list_box_item(h: HWND, i: usize) -> String {
    let n = send(h, LB_GETTEXTLEN, i, 0);
    if n < 0 {
        return String::new();
    }
    let mut buf = vec![0u16; n as usize + 1];
    send(h, LB_GETTEXT, i, buf.as_mut_ptr() as isize);
    String::from_utf16_lossy(&buf[..n as usize])
}

fn is_multi(h: HWND) -> bool {
    unsafe { wm::GetWindowLongW(h, wm::GWL_STYLE) as u32 & 0x0800 != 0 }
}

/// The chosen line (the one with the focus, for several), if any.
pub fn list_box_selected(h: HWND) -> Option<usize> {
    if is_multi(h) {
        return list_box_selected_all(h).first().copied();
    }
    let i = send(h, LB_GETCURSEL, 0, 0);
    (i >= 0).then_some(i as usize)
}

/// Chooses line `i`, or none.
pub fn list_box_select(h: HWND, i: Option<usize>) {
    if is_multi(h) {
        send(h, LB_SETSEL, 0, -1);
        if let Some(i) = i {
            send(h, LB_SETSEL, 1, i as isize);
        }
    } else {
        send(h, LB_SETCURSEL, i.map(|i| i as isize).unwrap_or(-1) as usize, 0);
    }
}

/// Every chosen line, in order.
pub fn list_box_selected_all(h: HWND) -> Vec<usize> {
    if !is_multi(h) {
        return list_box_selected(h).into_iter().collect();
    }
    (0..list_box_count(h)).filter(|i| send(h, LB_GETSEL, *i, 0) > 0).collect()
}

// --- CheckedListBox ------------------------------------------------------------------------------

/// A list of lines each with a tick box - WinForms' CheckedListBox. A tick changing comes as
/// `WM_NOTIFY` `LVN_ITEMCHANGED` with the state image changed ([`is_check_change`]).
pub fn checked_list(parent: HWND, id: i32, f: HFONT, items: &[&str]) -> HWND {
    const LVS_NOCOLUMNHEADER: u32 = 0x4000;
    const LVS_SINGLESEL: u32 = 0x0004;
    const LVS_EX_CHECKBOXES: isize = 0x4;
    const LVS_EX_FULLROWSELECT: isize = 0x20;
    const LVS_EX_DOUBLEBUFFER: isize = 0x10000;
    let h = win::list(parent, id, f, &[("", 200)]);
    unsafe {
        let s = wm::GetWindowLongW(h, wm::GWL_STYLE) as u32;
        wm::SetWindowLongW(h, wm::GWL_STYLE, (s | LVS_NOCOLUMNHEADER | LVS_SINGLESEL) as i32);
        wm::SendMessageW(h, cc::LVM_SETEXTENDEDLISTVIEWSTYLE, Some(WPARAM(0)), Some(LPARAM(LVS_EX_CHECKBOXES | LVS_EX_FULLROWSELECT | LVS_EX_DOUBLEBUFFER)));
        let _ = SetWindowSubclass(h, Some(checked_list_proc), 8, 0);
    }
    for it in items {
        win::list_add(h, &[it]);
    }
    h
}

unsafe extern "system" fn checked_list_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, _: usize) -> LRESULT {
    let r = unsafe { DefSubclassProc(h, msg, w, l) };
    if msg == wm::WM_SIZE {
        // Its one column as wide as it is.
        send(h, cc::LVM_SETCOLUMNWIDTH, 0, -2 /* LVSCW_AUTOSIZE_USEHEADER */);
    }
    r
}

/// Whether line `i` is ticked.
pub fn checked_list_checked(h: HWND, i: usize) -> bool {
    (send(h, cc::LVM_GETITEMSTATE, i, 0xF000) >> 12) == 2
}

pub fn checked_list_set_checked(h: HWND, i: usize, on: bool) {
    let item = cc::LVITEMW { stateMask: cc::LIST_VIEW_ITEM_STATE_FLAGS(0xF000), state: cc::LIST_VIEW_ITEM_STATE_FLAGS(if on { 2 } else { 1 } << 12), ..Default::default() };
    send(h, cc::LVM_SETITEMSTATE, i, &item as *const _ as isize);
}

/// Whether an `LVN_ITEMCHANGED` is a tick put or taken away (its row, then).
///
/// # Safety
/// `l` is the `LPARAM` of a `WM_NOTIFY` whose code is `LVN_ITEMCHANGED`.
pub unsafe fn is_check_change(l: LPARAM) -> Option<usize> {
    let it = unsafe { &*(l.0 as *const cc::NMLISTVIEW) };
    let (now, was) = ((it.uNewState & 0xF000) >> 12, (it.uOldState & 0xF000) >> 12);
    (it.uChanged.0 & cc::LVIF_STATE.0 != 0 && now != was && was != 0 && it.iItem >= 0).then_some(it.iItem as usize)
}

// --- LinkLabel -----------------------------------------------------------------------------------

/// Words that are a link - WinForms' LinkLabel. A click (or Enter on it) comes as `WM_NOTIFY`
/// `NM_CLICK` / `NM_RETURN`. `<a>` marks the link in the words; without one, all of it is.
pub fn link_label(parent: HWND, text: &str, id: i32, f: HFONT) -> HWND {
    let h = win::child(parent, "SysLink", &link_markup(text), wm::WS_TABSTOP, WINDOW_EX_STYLE(0), id, f);
    win::register_kind(h, Kind::Label);
    h
}

/// `text` as SysLink wants it: all a link unless it says which part is.
pub fn link_markup(text: &str) -> String {
    if text.contains("<a") {
        text.to_string()
    } else {
        format!("<a>{}</a>", text.replace('&', "&amp;").replace('<', "&lt;"))
    }
}

/// The words, without the link's marks.
pub fn link_words(h: HWND) -> String {
    let s = win::text_of(h);
    let mut out = String::new();
    let mut tag = false;
    for ch in s.chars() {
        match ch {
            '<' => tag = true,
            '>' if tag => tag = false,
            c if !tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&lt;", "<").replace("&amp;", "&")
}

// --- MaskedTextBox -------------------------------------------------------------------------------

/// One place of a mask: words that are always there, or a letter typed in.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Slot {
    Literal(char),
    /// The mask's letter for what may go here: 0 9 # L ? A a & C.
    Input(char),
}

fn slots(mask: &str) -> Vec<Slot> {
    let mut out = Vec::new();
    let mut it = mask.chars();
    while let Some(c) = it.next() {
        match c {
            '\\' => {
                if let Some(n) = it.next() {
                    out.push(Slot::Literal(n));
                }
            }
            '0' | '9' | '#' | 'L' | '?' | 'A' | 'a' | '&' | 'C' => out.push(Slot::Input(c)),
            c => out.push(Slot::Literal(c)),
        }
    }
    out
}

fn fits(m: char, c: char) -> bool {
    match m {
        '0' | '9' => c.is_ascii_digit(),
        '#' => c.is_ascii_digit() || c == '+' || c == '-',
        'L' | '?' => c.is_alphabetic(),
        'A' | 'a' => c.is_alphanumeric(),
        _ => !c.is_control(),
    }
}

fn required(m: char) -> bool {
    matches!(m, '0' | 'L' | 'A' | '&')
}

/// What the person sees as a mask fills: a typed letter, the literal, or `_` for a place to fill.
#[derive(Clone, Debug)]
pub struct Mask {
    slots: Vec<Slot>,
    typed: Vec<Option<char>>,
}

impl Mask {
    pub fn new(mask: &str) -> Mask {
        let slots = slots(mask);
        let typed = vec![None; slots.len()];
        Mask { slots, typed }
    }

    pub fn display(&self) -> String {
        self.slots.iter().zip(&self.typed).map(|(s, t)| match s {
            Slot::Literal(c) => *c,
            Slot::Input(_) => t.unwrap_or('_'),
        }).collect()
    }

    /// The letters typed, without the literals - WinForms' `TextMaskFormat.ExcludePromptAndLiterals`.
    pub fn value(&self) -> String {
        self.typed.iter().flatten().collect()
    }

    /// Every place that must be filled is.
    pub fn completed(&self) -> bool {
        self.slots.iter().zip(&self.typed).all(|(s, t)| !matches!(s, Slot::Input(m) if required(*m)) || t.is_some())
    }

    /// The first place to type at or after `pos`.
    fn next_input(&self, pos: usize) -> Option<usize> {
        (pos..self.slots.len()).find(|i| matches!(self.slots[*i], Slot::Input(_)))
    }

    /// `c` typed at `pos`: where the caret goes after it, or `None` when it does not fit.
    pub fn type_at(&mut self, pos: usize, c: char) -> Option<usize> {
        // A literal typed where it stands steps over it.
        if let Some(Slot::Literal(l)) = self.slots.get(pos) {
            if *l == c {
                return Some(pos + 1);
            }
        }
        let i = self.next_input(pos)?;
        let Slot::Input(m) = self.slots[i] else { return None };
        if !fits(m, c) {
            return None;
        }
        self.typed[i] = Some(match m {
            'L' | '?' => c,
            _ => c,
        });
        Some(self.next_input(i + 1).unwrap_or(self.slots.len()))
    }

    /// The place before `pos` emptied (Backspace); where the caret goes.
    pub fn back(&mut self, pos: usize) -> usize {
        let Some(i) = (0..pos.min(self.slots.len())).rev().find(|i| matches!(self.slots[*i], Slot::Input(_))) else { return 0 };
        self.typed[i] = None;
        i
    }

    /// The place at `pos` emptied (Delete).
    pub fn delete(&mut self, pos: usize) {
        if let Some(i) = self.next_input(pos) {
            self.typed[i] = None;
        }
    }

    /// Filled from `s`: its letters in turn into the places they fit, literals stepped over.
    pub fn fill(&mut self, s: &str) {
        self.typed = vec![None; self.slots.len()];
        let mut pos = 0;
        for c in s.chars() {
            if c == '_' {
                if let Some(i) = self.next_input(pos) {
                    pos = i + 1;
                }
                continue;
            }
            if let Some(p) = self.type_at(pos, c) {
                pos = p;
            }
        }
    }
}

static MASKS: Mutex<Option<HashMap<isize, Mask>>> = Mutex::new(None);

fn with_mask<R>(h: HWND, f: impl FnOnce(&mut Mask) -> R) -> Option<R> {
    MASKS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).get_mut(&(h.0 as isize)).map(f)
}

/// A box that takes only what its mask allows, the mask's literals in place - WinForms'
/// MaskedTextBox. `0` a digit, `9` a digit or none, `#` a digit, + or -, `L` a letter, `?` a
/// letter or none, `A` a letter or digit, `a` one or none, `&` any letter, `C` any or none; `\`
/// makes the next one a literal; anything else is a literal: `000-000-0000`.
pub fn masked(parent: HWND, mask: &str, text: &str, id: i32, f: HFONT) -> HWND {
    let h = win::edit(parent, "", id, f, false);
    let mut m = Mask::new(mask);
    m.fill(text);
    MASKS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).insert(h.0 as isize, m);
    unsafe {
        let _ = SetWindowSubclass(h, Some(masked_proc), 9, 0);
    }
    show_mask(h, 0);
    h
}

/// The letters typed, without the literals and the blanks.
pub fn masked_value(h: HWND) -> String {
    with_mask(h, |m| m.value()).unwrap_or_default()
}

/// Every place that must be filled is.
pub fn mask_completed(h: HWND) -> bool {
    with_mask(h, |m| m.completed()).unwrap_or(true)
}

fn show_mask(h: HWND, caret: usize) {
    let Some(s) = with_mask(h, |m| m.display()) else { return };
    let w = wide(&s);
    unsafe {
        DefSubclassProc(h, wm::WM_SETTEXT, WPARAM(0), LPARAM(w.as_ptr() as isize));
    }
    send(h, 0x00B1 /* EM_SETSEL */, caret, caret as isize);
}

fn caret_of(h: HWND) -> usize {
    let (mut s, mut e) = (0u32, 0u32);
    send(h, 0x00B0 /* EM_GETSEL */, &mut s as *mut _ as usize, &mut e as *mut _ as isize);
    s as usize
}

unsafe extern "system" fn masked_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, _: usize) -> LRESULT {
    match msg {
        wm::WM_CHAR => {
            let c = char::from_u32(w.0 as u32).unwrap_or('\0');
            let pos = caret_of(h);
            if c == '\u{8}' {
                let to = with_mask(h, |m| m.back(pos)).unwrap_or(0);
                show_mask(h, to);
            } else if !c.is_control() {
                match with_mask(h, |m| m.type_at(pos, c)).flatten() {
                    Some(to) => show_mask(h, to),
                    // What does not fit is not taken.
                    None => return LRESULT(0),
                }
            } else {
                // Ctrl+C and the like: the box's own.
                return unsafe { DefSubclassProc(h, msg, w, l) };
            }
            // The program hears of it from the box itself: its words were set (EN_CHANGE).
            LRESULT(0)
        }
        wm::WM_KEYDOWN if w.0 == 0x2E /* VK_DELETE */ => {
            let pos = caret_of(h);
            with_mask(h, |m| m.delete(pos));
            show_mask(h, pos);
            LRESULT(0)
        }
        wm::WM_SETTEXT => {
            // Words set by the program fill the mask.
            let s = if l.0 == 0 { String::new() } else { unsafe { PCWSTR(l.0 as *const u16).to_string() }.unwrap_or_default() };
            with_mask(h, |m| m.fill(&s));
            show_mask(h, 0);
            LRESULT(1)
        }
        wm::WM_PASTE | wm::WM_CUT | wm::WM_CLEAR => LRESULT(0),
        wm::WM_NCDESTROY => {
            if let Some(m) = MASKS.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
                m.remove(&(h.0 as isize));
            }
            unsafe { DefSubclassProc(h, msg, w, l) }
        }
        _ => unsafe { DefSubclassProc(h, msg, w, l) },
    }
}

// --- MonthCalendar -------------------------------------------------------------------------------

/// A month to choose a day in - WinForms' MonthCalendar. A day chosen comes as `WM_NOTIFY`
/// `MCN_SELCHANGE`.
pub fn calendar(parent: HWND, id: i32, f: HFONT) -> HWND {
    const MCS_NOTODAYCIRCLE: u32 = 0x0008;
    let h = win::child(parent, "SysMonthCal32", "", WINDOW_STYLE(MCS_NOTODAYCIRCLE) | wm::WS_TABSTOP, WINDOW_EX_STYLE(0), id, f);
    win::register_kind(h, Kind::Calendar);
    win::theme_native(h);
    h
}

/// Its colours from the theme's `calendar` rule: a dark theme draws it plain in them, as Windows'
/// own look of it is only light.
pub fn calendar_colors(h: HWND, back: Color, text: Color, accent: Color, dark: bool) {
    const MCM_SETCOLOR: u32 = 0x100A;
    for (part, c) in [(0usize, back), (4, back), (1, text), (2, accent), (3, Color::rgb(255, 255, 255)), (5, Color::rgb(0x8a, 0x8a, 0x8a))] {
        send(h, MCM_SETCOLOR, part, c.colorref() as isize);
    }
    unsafe {
        if dark {
            let _ = cc::SetWindowTheme(h, windows::core::w!(""), windows::core::w!(""));
        } else {
            let _ = cc::SetWindowTheme(h, PCWSTR::null(), PCWSTR::null());
        }
    }
}

/// The chosen day: (year, month, day).
pub fn calendar_date(h: HWND) -> (i32, i32, i32) {
    let mut st = SYSTEMTIME::default();
    send(h, 0x1001 /* MCM_GETCURSEL */, 0, &mut st as *mut _ as isize);
    (st.wYear as i32, st.wMonth as i32, st.wDay as i32)
}

pub fn calendar_set_date(h: HWND, y: i32, m: i32, d: i32) {
    let st = SYSTEMTIME { wYear: y as u16, wMonth: m as u16, wDay: d as u16, ..Default::default() };
    send(h, 0x1002 /* MCM_SETCURSEL */, 0, &st as *const _ as isize);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mask_takes_what_fits_and_steps_over_its_literals() {
        let mut m = Mask::new("000-000-0000");
        assert_eq!(m.display(), "___-___-____");
        let mut pos = 0;
        for c in "08x1234".chars() {
            if let Some(p) = m.type_at(pos, c) {
                pos = p;
            }
        }
        assert_eq!(m.display(), "081-234-____");
        assert_eq!(m.value(), "081234");
        assert!(!m.completed());
        m.fill("081-234-5678");
        assert_eq!(m.display(), "081-234-5678");
        assert!(m.completed());
        assert_eq!(m.back(8), 6, "Backspace after the dash empties the digit before it");
        assert_eq!(m.display(), "081-23_-5678");
        m.delete(3);
        assert_eq!(m.display(), "081-_3_-5678");
    }

    #[test]
    fn optional_places_and_escaped_literals() {
        let mut m = Mask::new("\\0LL-99");
        assert_eq!(m.display(), "0__-__");
        m.fill("0ab-1");
        assert_eq!(m.display(), "0ab-1_");
        assert!(m.completed(), "the 9s may stay empty");
    }

    #[test]
    fn link_words_lose_their_marks() {
        assert_eq!(link_markup("Monrak & co"), "<a>Monrak &amp; co</a>");
        assert_eq!(link_markup("see <a>here</a>"), "see <a>here</a>");
    }
}
