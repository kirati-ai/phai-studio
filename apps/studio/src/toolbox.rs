//! The toolbox, as Visual Studio's: a caption with a close button, a search box, and the controls
//! by group - Common controls, Containers, Menus and toolbars - each an icon and its name, the
//! Pointer first in every group. A click chooses the tool, a double-click puts it on the form at
//! once; a group's heading folds it. What is typed in the search box keeps only the controls whose
//! names have it.

use std::collections::HashMap;

use phai_form::Kind;
use phai_ui::icon::glyph;
use phai_ui::text::t;
use phai_ui::theme::{self, Color, State};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HFONT};
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HICON, HMENU};

use crate::{studio, Pane};

/// The pane's width, design pixels.
pub const WIDTH: i32 = 200;
const CAPTION: i32 = 28;
const SEARCH: i32 = 24;
const ROW: i32 = 22;
const ID_SEARCH: i32 = 3400;
const ID_CAPTION: i32 = 3401;

/// A line of the list: a group's heading (its index), or a tool (its command and glyph).
#[derive(Clone, Copy, PartialEq)]
enum Line {
    Group(usize),
    Tool(i32, u16),
}

/// The groups, in order; a kind's group is where [`group_of`] puts it.
fn groups() -> [&'static str; 4] {
    [t("Common controls"), t("Containers"), t("Menus and toolbars"), t("Components")]
}

fn group_of(k: Kind) -> usize {
    match k {
        Kind::Group | Kind::Tabs | Kind::Flow | Kind::Table | Kind::Panel | Kind::Split => 1,
        Kind::Statusbar => 2,
        k if k.is_component() => 3,
        _ => 0,
    }
}

struct Toolbox {
    caption: HWND,
    search: HWND,
    lines: Vec<Line>,
    folded: Vec<usize>,
    chosen: i32,
    hover: Option<usize>,
    top: usize,
    font: HFONT,
    bold: HFONT,
    /// Icons drawn for the colour and size they were drawn at.
    icons: HashMap<(u16, u32, i32), HICON>,
}

fn state<'a>(h: HWND) -> Option<&'a mut Toolbox> {
    unsafe { win::state::<Toolbox>(h) }
}

/// Makes the pane; the tools are `(command, name, glyph)` as [`crate`] lists them, Pointer first.
pub fn new(parent: HWND, id: i32, font: HFONT) -> HWND {
    let class = win::register_with(win::ClassStyle::DoubleClicks, "PhaiToolbox", Some(proc_), None);
    unsafe {
        let h = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPCHILDREN | wm::WS_CLIPSIBLINGS | wm::WS_VSCROLL, 0, 0, 10, 10, Some(parent), Some(HMENU(id as isize as *mut _)), Some(win::instance()), None).unwrap_or_default();
        let caption = phai_ui::controls::caption(h, t("Toolbox"), ID_CAPTION, font);
        let search = win::edit(h, "", ID_SEARCH, font, false);
        let bold = win::font(h, 0, true);
        win::set_state(h, Box::new(Toolbox { caption, search, lines: vec![], folded: vec![], chosen: crate::ID_TOOL_PUB, hover: None, top: 0, font, bold, icons: HashMap::new() }));
        relabel(h);
        h
    }
}

/// Shows `command` as the chosen tool.
pub fn choose(h: HWND, command: i32) {
    if let Some(s) = state(h) {
        s.chosen = command;
        unsafe {
            let _ = Gdi::InvalidateRect(Some(h), None, false);
        }
    }
}

/// The names again, in the language now chosen; the list again, for what is searched.
pub fn relabel(h: HWND) {
    let Some(s) = state(h) else { return };
    win::set_text(s.caption, t("Toolbox"));
    let cue = win::wide(t("Search Toolbox"));
    unsafe {
        wm::SendMessageW(s.search, 0x1501 /* EM_SETCUEBANNER */, Some(WPARAM(1)), Some(LPARAM(cue.as_ptr() as isize)));
    }
    fill(h);
}

fn fill(h: HWND) {
    let Some(s) = state(h) else { return };
    let query = win::text_of(s.search).trim().to_lowercase();
    let tools = crate::toolbox_items();
    s.lines.clear();
    for (g, _) in groups().iter().enumerate() {
        // The group's tools by name, as Visual Studio sorts them; the Pointer above them.
        // Menu bar and Toolbar are no kind of control: they belong with the menus.
        let group = |id: i32| match id {
            crate::ID_TOOL_MENUBAR | crate::ID_TOOL_TOOLBAR => Some(2),
            _ => crate::tool_kind(id).map(group_of),
        };
        let mut mine: Vec<(i32, String, u16)> = tools.iter().filter(|(id, _, _)| group(*id) == Some(g)).map(|(id, n, gl)| (*id, n.to_string(), *gl)).collect();
        if !query.is_empty() {
            mine.retain(|(id, n, _)| n.to_lowercase().contains(&query) || crate::tool_kind(*id).is_some_and(|k| k.name().contains(&query)));
            if mine.is_empty() {
                continue;
            }
        }
        mine.sort_by_key(|(_, n, _)| n.to_lowercase());
        s.lines.push(Line::Group(g));
        if s.folded.contains(&g) && query.is_empty() {
            continue;
        }
        if query.is_empty() {
            s.lines.push(Line::Tool(crate::ID_TOOL_PUB, glyph::POINTER));
        }
        s.lines.extend(mine.into_iter().map(|(id, _, gl)| Line::Tool(id, gl)));
    }
    s.top = s.top.min(s.lines.len().saturating_sub(1));
    update_scroll(h);
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

fn name_of(id: i32) -> String {
    crate::toolbox_items().into_iter().find(|(c, _, _)| *c == id).map(|(_, n, _)| n.to_string()).unwrap_or_default()
}

/// Where the list starts, device pixels.
fn list_top(h: HWND) -> i32 {
    win::px(h, CAPTION + SEARCH + 8)
}

fn visible_rows(h: HWND) -> usize {
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    ((r.bottom - list_top(h)) / win::px(h, ROW).max(1)).max(1) as usize
}

fn update_scroll(h: HWND) {
    let Some(s) = state(h) else { return };
    let page = visible_rows(h);
    s.top = s.top.min(s.lines.len().saturating_sub(page));
    unsafe {
        let si = wm::SCROLLINFO { cbSize: std::mem::size_of::<wm::SCROLLINFO>() as u32, fMask: wm::SIF_ALL, nMin: 0, nMax: s.lines.len().max(1) as i32 - 1, nPage: page as u32, nPos: s.top as i32, nTrackPos: 0 };
        windows::Win32::UI::Controls::SetScrollInfo(h, wm::SB_VERT, &si, true);
    }
}

fn line_at(h: HWND, y: i32) -> Option<usize> {
    let s = state(h)?;
    if y < list_top(h) {
        return None;
    }
    let k = s.top + ((y - list_top(h)) / win::px(h, ROW).max(1)) as usize;
    (k < s.lines.len()).then_some(k)
}

/// The pane's caption - its buttons are the studio's to answer.
pub fn caption_of(h: HWND) -> HWND {
    state(h).map(|s| s.caption).unwrap_or_default()
}

fn icon(s: &mut Toolbox, g: u16, colour: Color, size: i32) -> Option<HICON> {
    let key = (g, colour.colorref(), size);
    if let Some(i) = s.icons.get(&key) {
        return Some(*i);
    }
    let i = phai_ui::icon::glyph_icon(g, size, colour)?;
    s.icons.insert(key, i);
    Some(i)
}

fn paint(h: HWND) {
    let Some(s) = state(h) else { return };
    unsafe {
        let mut ps = Gdi::PAINTSTRUCT::default();
        let dc = Gdi::BeginPaint(h, &mut ps);
        let mut r = RECT::default();
        let _ = wm::GetClientRect(h, &mut r);
        let bg = win::window_background();
        let text = win::window_text();
        let muted = theme::current().var_color("--muted").unwrap_or(Color::rgb(0x5f, 0x5f, 0x5f));
        let hover = theme::style("button", State { hover: true, ..Default::default() }).background_or(Color::rgb(231, 243, 239));
        let pressed = theme::style("button", State { pressed: true, ..Default::default() });
        let icon_colour = theme::style("toolbar", State::default()).icon_color.unwrap_or(text);
        // Drawn off screen, then put up at once: no flicker while the pointer moves.
        let mem = Gdi::CreateCompatibleDC(Some(dc));
        let bmp = Gdi::CreateCompatibleBitmap(dc, r.right.max(1), r.bottom.max(1));
        let old_bmp = Gdi::SelectObject(mem, bmp.into());
        win::fill(mem, &r, bg);
        let px = |v| win::px(h, v);
        let _ = muted;
        let old_font = Gdi::SelectObject(mem, s.font.into());
        let rows = visible_rows(h) + 1;
        let row = px(ROW);
        let names = groups();
        for k in s.top..(s.top + rows).min(s.lines.len()) {
            let y = list_top(h) + (k - s.top) as i32 * row;
            let lr = RECT { left: 0, top: y, right: r.right, bottom: y + row };
            match s.lines[k] {
                Line::Group(g) => {
                    if s.hover == Some(k) {
                        win::fill(mem, &lr, hover);
                    }
                    let open = !s.folded.contains(&g) || !win::text_of(s.search).trim().is_empty();
                    if let Some(i) = icon(s, if open { 0xE70D } else { 0xE76C }, text, px(10)) {
                        let _ = wm::DrawIconEx(mem, px(8), y + (row - px(10)) / 2, i, px(10), px(10), 0, None, wm::DI_NORMAL);
                    }
                    Gdi::SelectObject(mem, s.bold.into());
                    let mut tr = RECT { left: px(24), right: r.right - px(4), ..lr };
                    draw(mem, names[g], &mut tr, text, Gdi::DT_LEFT);
                    Gdi::SelectObject(mem, s.font.into());
                }
                Line::Tool(id, g) => {
                    // The Pointer is in every group, as in Visual Studio; only the first shows chosen.
                    let first = id != crate::ID_TOOL_PUB || !s.lines[..k].contains(&s.lines[k]);
                    let box_r = RECT { left: px(4), top: y + 1, right: r.right - px(4), bottom: y + row - 1 };
                    if s.chosen == id && first {
                        win::round_rect(mem, box_r, pressed.background_or(Color::rgb(208, 231, 223)), pressed.border_or(Color::rgb(18, 133, 95)), px(6));
                    } else if s.hover == Some(k) {
                        win::round_rect(mem, box_r, hover, hover, px(6));
                    }
                    if let Some(i) = icon(s, g, icon_colour, px(16)) {
                        let _ = wm::DrawIconEx(mem, px(22), y + (row - px(16)) / 2, i, px(16), px(16), 0, None, wm::DI_NORMAL);
                    }
                    let mut tr = RECT { left: px(48), right: r.right - px(6), ..lr };
                    draw(mem, &name_of(id), &mut tr, text, Gdi::DT_LEFT);
                }
            }
        }
        Gdi::SelectObject(mem, old_font);
        let _ = Gdi::BitBlt(dc, 0, 0, r.right, r.bottom, Some(mem), 0, 0, Gdi::SRCCOPY);
        Gdi::SelectObject(mem, old_bmp);
        let _ = Gdi::DeleteObject(bmp.into());
        let _ = Gdi::DeleteDC(mem);
        let _ = Gdi::EndPaint(h, &ps);
    }
}

fn draw(dc: Gdi::HDC, text: &str, r: &mut RECT, colour: Color, align: Gdi::DRAW_TEXT_FORMAT) {
    if text.is_empty() {
        return;
    }
    let mut w: Vec<u16> = text.encode_utf16().collect();
    unsafe {
        Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(colour.colorref()));
        Gdi::DrawTextW(dc, &mut w, r, align | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX | Gdi::DT_END_ELLIPSIS);
    }
}

fn click(h: HWND, _x: i32, y: i32, double: bool) {
    let Some(st) = state(h) else { return };
    let Some(k) = line_at(h, y) else { return };
    match st.lines[k] {
        Line::Group(g) => {
            if !win::text_of(st.search).trim().is_empty() {
                return;
            }
            if let Some(at) = st.folded.iter().position(|f| *f == g) {
                st.folded.remove(at);
            } else {
                st.folded.push(g);
            }
            fill(h);
        }
        Line::Tool(id, _) => {
            let Some(s) = studio() else { return };
            if double {
                if let Some(kind) = crate::tool_kind(id) {
                    if s.doc.code.is_none() {
                        s.add_at_margin(kind);
                    }
                }
                return;
            }
            st.chosen = id;
            s.command(id);
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
        }
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_PAINT => {
            paint(h);
            LRESULT(0)
        }
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_SIZE => {
            if let Some(s) = state(h) {
                let width = win::loword(l.0 as usize) as i32;
                let px = |v| win::px(h, v);
                win::move_to(s.caption, 0, 0, width, px(CAPTION));
                win::move_to(s.search, px(6), px(CAPTION) + px(2), (width - px(12)).max(0), px(SEARCH));
            }
            update_scroll(h);
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        wm::WM_COMMAND if win::loword(w.0) as i32 == ID_CAPTION => {
            if let Some(s) = studio() {
                s.pane_button(Pane::Toolbox, win::hiword(w.0), HWND(l.0 as *mut _));
            }
            LRESULT(0)
        }
        wm::WM_COMMAND if win::loword(w.0) as i32 == ID_SEARCH && win::hiword(w.0) == 0x0300 /* EN_CHANGE */ => {
            if let Some(s) = state(h) {
                s.top = 0;
            }
            fill(h);
            LRESULT(0)
        }
        wm::WM_LBUTTONDOWN | wm::WM_LBUTTONDBLCLK => {
            let (x, y) = win::point_of(l);
            click(h, x, y, msg == wm::WM_LBUTTONDBLCLK);
            LRESULT(0)
        }
        wm::WM_MOUSEMOVE => {
            let (_, y) = win::point_of(l);
            let k = line_at(h, y);
            if let Some(s) = state(h) {
                if s.hover != k {
                    s.hover = k;
                    let mut t = km::TRACKMOUSEEVENT { cbSize: std::mem::size_of::<km::TRACKMOUSEEVENT>() as u32, dwFlags: km::TME_LEAVE, hwndTrack: h, dwHoverTime: 0 };
                    unsafe {
                        let _ = km::TrackMouseEvent(&mut t);
                        let _ = Gdi::InvalidateRect(Some(h), None, false);
                    }
                }
            }
            LRESULT(0)
        }
        0x02A3 /* WM_MOUSELEAVE */ => {
            if let Some(s) = state(h) {
                s.hover = None;
                unsafe {
                    let _ = Gdi::InvalidateRect(Some(h), None, false);
                }
            }
            LRESULT(0)
        }
        wm::WM_MOUSEWHEEL => {
            let delta = (w.0 >> 16) as i16 as i32;
            if let Some(s) = state(h) {
                s.top = (s.top as i64 - (delta / 40) as i64).max(0) as usize;
            }
            update_scroll(h);
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        wm::WM_VSCROLL => {
            let mut si = wm::SCROLLINFO { cbSize: std::mem::size_of::<wm::SCROLLINFO>() as u32, fMask: wm::SIF_ALL, ..Default::default() };
            unsafe {
                let _ = wm::GetScrollInfo(h, wm::SB_VERT, &mut si);
            }
            let pos = match wm::SCROLLBAR_COMMAND(win::loword(w.0) as i32) {
                wm::SB_LINEUP => si.nPos - 1,
                wm::SB_LINEDOWN => si.nPos + 1,
                wm::SB_PAGEUP => si.nPos - si.nPage as i32,
                wm::SB_PAGEDOWN => si.nPos + si.nPage as i32,
                wm::SB_THUMBTRACK | wm::SB_THUMBPOSITION => si.nTrackPos,
                _ => si.nPos,
            };
            if let Some(s) = state(h) {
                s.top = pos.max(0) as usize;
            }
            update_scroll(h);
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        wm::WM_DESTROY => {
            if let Some(s) = unsafe { win::take_state::<Toolbox>(h) } {
                for i in s.icons.values() {
                    unsafe {
                        let _ = wm::DestroyIcon(*i);
                    }
                }
            }
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}
