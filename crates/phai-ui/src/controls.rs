//! Three controls drawn whole by the library, because the Windows ones cannot take a theme: a
//! group frame, tabs in the Windows 11 manner (a line under the chosen one, no folder tabs), and a
//! status bar. Each is a small window class of its own; the parent hears from tabs as
//! `WM_COMMAND` with the tab index in the high word.

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HFONT};
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HMENU, WINDOW_EX_STYLE};

use crate::theme::{Color, State};
use crate::win::{self, px, Kind};

fn create(parent: HWND, class: &str, proc_: wm::WNDPROC, text: &str, id: i32, f: HFONT, ex: WINDOW_EX_STYLE) -> HWND {
    let class = win::register(class, proc_, None);
    let text = win::wide(text);
    unsafe {
        let h = wm::CreateWindowExW(
            ex,
            PCWSTR(class.as_ptr()),
            PCWSTR(text.as_ptr()),
            wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPSIBLINGS,
            0,
            0,
            10,
            10,
            Some(parent),
            Some(HMENU(id as isize as *mut _)),
            Some(win::instance()),
            None,
        )
        .unwrap_or_default();
        win::set_font(h, f);
        h
    }
}

/// Each class keeps its font, as the standard controls do.
struct Common {
    font: HFONT,
}

unsafe fn common_messages(h: HWND, msg: u32, w: WPARAM, l: LPARAM, font: &mut HFONT) -> Option<LRESULT> {
    match msg {
        wm::WM_SETFONT => {
            *font = HFONT(w.0 as *mut _);
            if l.0 != 0 {
                let _ = Gdi::InvalidateRect(Some(h), None, true);
            }
            Some(LRESULT(0))
        }
        wm::WM_GETFONT => Some(LRESULT(font.0 as isize)),
        wm::WM_SETTEXT => {
            let r = wm::DefWindowProcW(h, msg, w, l);
            let _ = Gdi::InvalidateRect(Some(h), None, true);
            Some(r)
        }
        _ => None,
    }
}

// --- group ------------------------------------------------------------------------------------

/// A frame with a title that gathers the controls inside it. The controls are the parent's, laid
/// over it: the frame sits at the bottom of the z-order, and every sibling clips it.
pub fn group(parent: HWND, text: &str, id: i32, f: HFONT) -> HWND {
    let h = create(parent, "PhaiGroup", Some(group_proc), text, id, f, WINDOW_EX_STYLE(0));
    win::set_state(h, Box::new(Common { font: f }));
    mark(h, Kind::Group);
    unsafe {
        // Below its siblings, so it never paints over what it gathers.
        let _ = wm::SetWindowPos(h, Some(wm::HWND_BOTTOM), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOACTIVATE);
    }
    h
}

/// Joins the library's registry of kinds, so `set_css` and `style_of` work on these too.
fn mark(h: HWND, kind: Kind) {
    win::register_kind(h, kind);
}

extern "system" fn group_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        let Some(s) = win::state::<Common>(h) else { return wm::DefWindowProcW(h, msg, w, l) };
        if let Some(r) = common_messages(h, msg, w, l, &mut s.font) {
            return r;
        }
        match msg {
            wm::WM_ERASEBKGND => LRESULT(1),
            wm::WM_NCHITTEST => LRESULT(-1), // HTTRANSPARENT: clicks reach what is inside
            wm::WM_PAINT => {
                let mut ps = Gdi::PAINTSTRUCT::default();
                let dc = Gdi::BeginPaint(h, &mut ps);
                let st = win::style_of(h, Kind::Group, State::default());
                let mut r = RECT::default();
                let _ = wm::GetClientRect(h, &mut r);
                win::paint_behind(h, dc, &r);
                let text = win::text_of(h);
                let (tw, th) = win::measure(h, win::css_font(h, &st).unwrap_or(s.font), &text);
                let top = th / 2;
                let frame = RECT { top: r.top + top, ..r };
                win::round_rect(dc, frame, Color::TRANSPARENT, st.border_or(Color::rgb(229, 229, 229)), px(h, st.border_radius.unwrap_or(6) * 2));
                if !text.is_empty() {
                    let x = px(h, 10);
                    let gap = px(h, 4);
                    let back = RECT { left: x - gap, top: r.top, right: x + tw + gap, bottom: r.top + th };
                    win::paint_behind(h, dc, &back);
                    let mut tr = RECT { left: x, top: r.top, right: x + tw, bottom: r.top + th };
                    win::draw_text_styled(dc, h, &text, &mut tr, st.color_or(win::window_text_for(h)), Gdi::DT_LEFT | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX, &st);
                }
                let _ = Gdi::EndPaint(h, &ps);
                LRESULT(0)
            }
            wm::WM_NCDESTROY => {
                win::forget_kind(h);
                drop(win::take_state::<Common>(h));
                LRESULT(0)
            }
            _ => wm::DefWindowProcW(h, msg, w, l),
        }
    }
}

// --- tabs ---------------------------------------------------------------------------------------

struct Tabs {
    font: HFONT,
    labels: Vec<String>,
    selected: usize,
    hot: Option<usize>,
    /// Documents' tabs: each has a close button (on the chosen one and the one under the pointer),
    /// and a middle click closes it too.
    closable: bool,
    /// The close button under the pointer.
    hot_close: bool,
}

/// The high bit of the index a closable tab strip sends: that tab's close button was clicked.
pub const TAB_CLOSE: usize = 0x8000;

/// Tabs along the top, the chosen one underlined in the accent colour. The parent shows the
/// chosen page's controls; it hears `WM_COMMAND` with `id` and the index in the high word.
pub fn tabs(parent: HWND, id: i32, f: HFONT, labels: &[&str]) -> HWND {
    let h = create(parent, "PhaiTabs", Some(tabs_proc), "", id, f, WINDOW_EX_STYLE(0));
    win::set_state(h, Box::new(Tabs { font: f, labels: labels.iter().map(|s| s.to_string()).collect(), selected: 0, hot: None, closable: false, hot_close: false }));
    mark(h, Kind::Tabs);
    h
}

/// Gives each tab a close button, as a document's tab has: the parent hears the tab's index with
/// [`TAB_CLOSE`] added when it is clicked (or the tab is middle-clicked).
pub fn tabs_closable(h: HWND) {
    unsafe {
        if let Some(t) = win::state::<Tabs>(h) {
            t.closable = true;
            let _ = Gdi::InvalidateRect(Some(h), None, true);
        }
    }
}

pub fn tabs_selected(h: impl Into<HWND>) -> usize {
    let h: HWND = h.into();
    unsafe { win::state::<Tabs>(h).map(|t| t.selected).unwrap_or(0) }
}

pub fn tabs_select(h: impl Into<HWND>, i: usize) {
    let h: HWND = h.into();
    unsafe {
        if let Some(t) = win::state::<Tabs>(h) {
            t.selected = i.min(t.labels.len().saturating_sub(1));
            let _ = Gdi::InvalidateRect(Some(h), None, true);
        }
    }
}

/// New labels, for a language switch.
pub fn tabs_relabel(h: HWND, labels: &[&str]) {
    unsafe {
        if let Some(t) = win::state::<Tabs>(h) {
            t.labels = labels.iter().map(|s| s.to_string()).collect();
            let _ = Gdi::InvalidateRect(Some(h), None, true);
        }
    }
}

/// Each tab's left and right edge, measured from its text.
fn tab_edges(h: HWND, t: &Tabs) -> Vec<(i32, i32)> {
    let pad = px(h, 12);
    let mut x = 0;
    t.labels
        .iter()
        .map(|l| {
            let font = win::css_font(h, &win::style_of(h, Kind::Tabs, State::default())).unwrap_or(t.font);
            let (w, _) = win::measure(h, font, l);
            let close = if t.closable { px(h, 20) } else { 0 };
            let e = (x, x + w + 2 * pad + close);
            x = e.1;
            e
        })
        .collect()
}

extern "system" fn tabs_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    use windows::Win32::UI::Input::KeyboardAndMouse::{TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT};
    const WM_MOUSELEAVE: u32 = 0x02A3;
    unsafe {
        let Some(t) = win::state::<Tabs>(h) else { return wm::DefWindowProcW(h, msg, w, l) };
        if let Some(r) = common_messages(h, msg, w, l, &mut t.font) {
            return r;
        }
        let at = |x: i32, t: &Tabs| tab_edges(h, t).iter().position(|(a, b)| x >= *a && x < *b);
        // A closable tab's close button: the square at its right end.
        let close_box = |i: usize, t: &Tabs| -> Option<RECT> {
            let (_, b) = *tab_edges(h, t).get(i)?;
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            let size = px(h, 16);
            let top = (r.bottom - px(h, 3) - size) / 2;
            t.closable.then_some(RECT { left: b - px(h, 8) - size, right: b - px(h, 8), top, bottom: top + size })
        };
        let on_close = |x: i32, y: i32, t: &Tabs| at(x, t).and_then(|i| close_box(i, t).filter(|c| x >= c.left && x < c.right && y >= c.top && y < c.bottom).map(|_| i));
        match msg {
            wm::WM_ERASEBKGND => LRESULT(1),
            wm::WM_PAINT => {
                let mut ps = Gdi::PAINTSTRUCT::default();
                let dc = Gdi::BeginPaint(h, &mut ps);
                let mut r = RECT::default();
                let _ = wm::GetClientRect(h, &mut r);
                let base = win::style_of(h, Kind::Tabs, State::default());
                win::fill(dc, &r, base.background_or(win::window_background_for(h)));
                let rule = RECT { top: r.bottom - 1, ..r };
                win::fill(dc, &rule, base.border_or(Color::rgb(229, 229, 229)));
                for (i, (a, b)) in tab_edges(h, t).into_iter().enumerate() {
                    let st = win::style_of(h, Kind::Tabs, State { selected: i == t.selected, hover: t.hot == Some(i), ..Default::default() });
                    let close_w = if t.closable { px(h, 20) } else { 0 };
                    let mut tr = RECT { left: a, right: b - close_w, top: r.top, bottom: r.bottom - px(h, 3) };
                    win::draw_text_styled(dc, h, &t.labels[i], &mut tr, st.color_or(win::window_text_for(h)), Gdi::DT_CENTER | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX, &st);
                    // The close button, on the chosen tab and the one under the pointer.
                    if let Some(c) = close_box(i, t).filter(|_| i == t.selected || t.hot == Some(i)) {
                        if t.hot == Some(i) && t.hot_close {
                            win::round_rect(dc, c, base.border_or(Color::rgb(220, 220, 220)), base.border_or(Color::rgb(220, 220, 220)), px(h, 3));
                        }
                        let (cx, cy, k) = ((c.left + c.right) / 2, (c.top + c.bottom) / 2, px(h, 4));
                        let pen = Gdi::CreatePen(Gdi::PS_SOLID, px(h, 1).max(1), windows::Win32::Foundation::COLORREF(st.color_or(win::window_text_for(h)).colorref()));
                        let old = Gdi::SelectObject(dc, pen.into());
                        let _ = Gdi::MoveToEx(dc, cx - k, cy - k, None);
                        let _ = Gdi::LineTo(dc, cx + k + 1, cy + k + 1);
                        let _ = Gdi::MoveToEx(dc, cx + k, cy - k, None);
                        let _ = Gdi::LineTo(dc, cx - k - 1, cy + k + 1);
                        Gdi::SelectObject(dc, old);
                        let _ = Gdi::DeleteObject(pen.into());
                    }
                    if i == t.selected {
                        let inset = px(h, 12);
                        let line = RECT { left: a + inset, right: b - inset, top: r.bottom - px(h, 3), bottom: r.bottom };
                        win::round_rect(dc, line, st.accent_or(Color::rgb(0x12, 0x85, 0x5f)), st.accent_or(Color::rgb(0x12, 0x85, 0x5f)), px(h, 3));
                    } else if t.hot == Some(i) {
                        let inset = px(h, 16);
                        let line = RECT { left: a + inset, right: b - inset, top: r.bottom - px(h, 2), bottom: r.bottom };
                        win::fill(dc, &line, base.border_or(Color::rgb(200, 200, 200)));
                    }
                }
                let _ = Gdi::EndPaint(h, &ps);
                LRESULT(0)
            }
            wm::WM_LBUTTONDOWN | wm::WM_MBUTTONUP if t.closable && (msg == wm::WM_MBUTTONUP || on_close(win::point_of(l).0, win::point_of(l).1, t).is_some()) => {
                // Close: the parent decides (it may ask to save first).
                if let Some(i) = at(win::point_of(l).0, t) {
                    let id = wm::GetDlgCtrlID(h) as usize;
                    if let Ok(p) = wm::GetParent(h) {
                        wm::SendMessageW(p, wm::WM_COMMAND, Some(WPARAM(((i | TAB_CLOSE) << 16) | (id & 0xFFFF))), Some(LPARAM(h.0 as isize)));
                    }
                }
                LRESULT(0)
            }
            wm::WM_LBUTTONDOWN => {
                if let Some(i) = at(win::point_of(l).0, t) {
                    if i != t.selected {
                        t.selected = i;
                        let _ = Gdi::InvalidateRect(Some(h), None, true);
                        let id = wm::GetDlgCtrlID(h) as usize;
                        if let Ok(p) = wm::GetParent(h) {
                            wm::SendMessageW(p, wm::WM_COMMAND, Some(WPARAM((i << 16) | (id & 0xFFFF))), Some(LPARAM(h.0 as isize)));
                        }
                    }
                }
                LRESULT(0)
            }
            wm::WM_MOUSEMOVE => {
                let (x, y) = win::point_of(l);
                let hot = at(x, t);
                let hot_close = on_close(x, y, t).is_some();
                if hot_close != t.hot_close {
                    t.hot_close = hot_close;
                    let _ = Gdi::InvalidateRect(Some(h), None, true);
                }
                if hot != t.hot {
                    if t.hot.is_none() {
                        let mut tm = TRACKMOUSEEVENT { cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32, dwFlags: TME_LEAVE, hwndTrack: h, dwHoverTime: 0 };
                        let _ = TrackMouseEvent(&mut tm);
                    }
                    t.hot = hot;
                    let _ = Gdi::InvalidateRect(Some(h), None, true);
                }
                LRESULT(0)
            }
            WM_MOUSELEAVE => {
                t.hot = None;
                let _ = Gdi::InvalidateRect(Some(h), None, true);
                LRESULT(0)
            }
            wm::WM_NCDESTROY => {
                win::forget_kind(h);
                drop(win::take_state::<Tabs>(h));
                LRESULT(0)
            }
            _ => wm::DefWindowProcW(h, msg, w, l),
        }
    }
}

// --- status bar -------------------------------------------------------------------------------

struct Status {
    font: HFONT,
    /// Widths in design pixels; the first part takes what the others leave.
    widths: Vec<i32>,
    texts: Vec<String>,
}

/// A status bar: one rule above, parts of text. Dock it to the bottom at [`status_height`].
pub fn status_bar(parent: HWND, id: i32, f: HFONT) -> HWND {
    let h = create(parent, "PhaiStatus", Some(status_proc), "", id, f, WINDOW_EX_STYLE(0));
    win::set_state(h, Box::new(Status { font: f, widths: vec![0], texts: vec![String::new()] }));
    mark(h, Kind::StatusBar);
    h
}

pub fn status_height(h: HWND) -> i32 {
    let font = unsafe { win::state::<Status>(h).map(|s| s.font).unwrap_or_default() };
    win::measure(h, font, "Ag").1 + px(h, 10)
}

/// The parts after the first, at the right, `widths` design pixels each.
pub fn status_parts(h: impl Into<HWND>, widths: &[i32]) {
    let h: HWND = h.into();
    unsafe {
        if let Some(s) = win::state::<Status>(h) {
            s.widths = std::iter::once(0).chain(widths.iter().copied()).collect();
            s.texts.resize(s.widths.len(), String::new());
            let _ = Gdi::InvalidateRect(Some(h), None, true);
        }
    }
}

pub fn status_text(h: impl Into<HWND>, part: usize, text: &str) {
    let h: HWND = h.into();
    unsafe {
        if let Some(s) = win::state::<Status>(h) {
            if part >= s.texts.len() {
                s.texts.resize(part + 1, String::new());
                s.widths.resize(part + 1, 120);
            }
            s.texts[part] = text.to_string();
            let _ = Gdi::InvalidateRect(Some(h), None, true);
        }
    }
}

extern "system" fn status_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        let Some(s) = win::state::<Status>(h) else { return wm::DefWindowProcW(h, msg, w, l) };
        if let Some(r) = common_messages(h, msg, w, l, &mut s.font) {
            return r;
        }
        match msg {
            wm::WM_ERASEBKGND => LRESULT(1),
            wm::WM_PAINT => {
                let mut ps = Gdi::PAINTSTRUCT::default();
                let dc = Gdi::BeginPaint(h, &mut ps);
                let mut r = RECT::default();
                let _ = wm::GetClientRect(h, &mut r);
                let st = win::style_of(h, Kind::StatusBar, State::default());
                win::fill_style(dc, &r, &st, win::window_background_for(h));
                let rule = RECT { bottom: r.top + 1, ..r };
                win::fill(dc, &rule, st.border_or(Color::rgb(229, 229, 229)));
                let fg = st.color_or(win::window_text_for(h));
                let pad = px(h, 10);
                let rest: i32 = s.widths.iter().skip(1).map(|w| px(h, *w)).sum();
                let mut x = r.left;
                for (i, text) in s.texts.iter().enumerate() {
                    let width = if i == 0 { r.right - rest } else { px(h, s.widths[i]) };
                    if i > 0 {
                        let sep = RECT { left: x, right: x + 1, top: r.top + px(h, 5), bottom: r.bottom - px(h, 5) };
                        win::fill(dc, &sep, st.border_or(Color::rgb(229, 229, 229)));
                    }
                    let mut tr = RECT { left: x + pad, right: x + width - pad, top: r.top + 1, bottom: r.bottom };
                    win::draw_text_styled(dc, h, text, &mut tr, fg, Gdi::DT_LEFT | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX | Gdi::DT_END_ELLIPSIS, &st);
                    x += width;
                }
                let _ = Gdi::EndPaint(h, &ps);
                LRESULT(0)
            }
            wm::WM_NCDESTROY => {
                win::forget_kind(h);
                drop(win::take_state::<Status>(h));
                LRESULT(0)
            }
            _ => wm::DefWindowProcW(h, msg, w, l),
        }
    }
}

// --- panes: a caption with its buttons, and a splitter ------------------------------------------
//
// What makes a panel a docked pane, as Visual Studio's tool windows are: a caption naming it with
// three buttons - Window Position (▾), Auto Hide (the pin) and Close - and a splitter beside it to
// drag its size.

/// A caption's close button was pressed: `WM_COMMAND` with the caption's id and this high word.
pub const CAPTION_CLOSE: u16 = 1;
/// Its ▾ was pressed: the owner shows the Window Position menu under [`caption_menu_point`].
pub const CAPTION_MENU: u16 = 2;
/// Its pin was pressed: Auto Hide, on or off.
pub const CAPTION_PIN: u16 = 3;
/// The left button went down on its name: the owner may drag the pane somewhere else.
pub const CAPTION_DRAG: u16 = 4;

/// Sent while a splitter is dragged: `WPARAM` its id, `LPARAM` how many device pixels it moved
/// since the last one - right or down positive.
pub const WM_SPLIT: u32 = wm::WM_APP + 0x3A20;

struct Caption {
    font: HFONT,
    /// The button under the pointer: 0 menu, 1 pin, 2 close.
    hot: Option<usize>,
    /// Hidden on its own when not in use: the pin lies on its side.
    auto_hidden: bool,
}

/// A pane's caption: its name on the left in the muted colour, ▾, the pin and × on the right.
pub fn caption(parent: HWND, title: &str, id: i32, f: HFONT) -> HWND {
    let h = create(parent, "PhaiCaption", Some(caption_proc), title, id, f, WINDOW_EX_STYLE(0));
    win::set_state(h, Box::new(Caption { font: f, hot: None, auto_hidden: false }));
    mark(h, Kind::Label);
    h
}

/// Shows the pin as Visual Studio does: upright while the pane stays, on its side while it hides
/// itself.
pub fn caption_set_auto_hidden(h: HWND, on: bool) {
    if let Some(s) = unsafe { win::state::<Caption>(h) } {
        s.auto_hidden = on;
        unsafe {
            let _ = Gdi::InvalidateRect(Some(h), None, false);
        }
    }
}

/// The caption's buttons, left to right: ▾, the pin, ×.
fn caption_boxes(h: HWND) -> [RECT; 3] {
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let s = (r.bottom - r.top - 1).max(1);
    let b = |k: i32| RECT { left: r.right - s * (3 - k), right: r.right - s * (2 - k), top: r.top, bottom: r.top + s };
    [b(0), b(1), b(2)]
}

/// Where the Window Position menu opens: under the ▾, in screen pixels.
pub fn caption_menu_point(h: HWND) -> (i32, i32) {
    let m = caption_boxes(h)[0];
    let mut pt = windows::Win32::Foundation::POINT { x: m.left, y: m.bottom };
    unsafe {
        let _ = Gdi::ClientToScreen(h, &mut pt);
    }
    (pt.x, pt.y)
}

extern "system" fn caption_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    use windows::Win32::UI::Input::KeyboardAndMouse::{TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT};
    unsafe {
        let Some(s) = win::state::<Caption>(h) else { return wm::DefWindowProcW(h, msg, w, l) };
        if let Some(r) = common_messages(h, msg, w, l, &mut s.font) {
            return r;
        }
        match msg {
            wm::WM_ERASEBKGND => LRESULT(1),
            wm::WM_PAINT => {
                let mut ps = Gdi::PAINTSTRUCT::default();
                let dc = Gdi::BeginPaint(h, &mut ps);
                let mut r = RECT::default();
                let _ = wm::GetClientRect(h, &mut r);
                let base = win::style_of(h, Kind::Tabs, State::default());
                win::fill(dc, &r, base.background_or(win::window_background_for(h)));
                let rule = RECT { top: r.bottom - 1, ..r };
                win::fill(dc, &rule, base.border_or(Color::rgb(229, 229, 229)));
                let muted = crate::theme::current().var_color("--muted").unwrap_or(Color::rgb(0x5f, 0x5f, 0x5f));
                let boxes = caption_boxes(h);
                let mut tr = RECT { left: px(h, 10), right: boxes[0].left, ..r };
                win::draw_text(dc, h, &win::text_of(h), &mut tr, muted, Gdi::DT_LEFT | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX | Gdi::DT_END_ELLIPSIS);
                // The buttons' glyphs, the one under the pointer shaded.
                let mut lf = Gdi::LOGFONTW { lfHeight: -px(h, 9), lfCharSet: Gdi::DEFAULT_CHARSET, lfQuality: Gdi::ANTIALIASED_QUALITY, ..Default::default() };
                for (i, u) in "Segoe Fluent Icons".encode_utf16().enumerate() {
                    lf.lfFaceName[i] = u;
                }
                let icons = Gdi::CreateFontIndirectW(&lf);
                let old = Gdi::SelectObject(dc, icons.into());
                Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
                Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(muted.colorref()));
                let glyphs = [0xE70Du16, if s.auto_hidden { 0xE77A } else { 0xE718 }, 0xE8BB];
                for (k, b) in boxes.iter().enumerate() {
                    if s.hot == Some(k) {
                        let hover = crate::theme::style("button", State { hover: true, ..Default::default() });
                        win::fill(dc, b, hover.background_or(Color::rgb(0xe7, 0xf3, 0xef)));
                    }
                    let mut one = [glyphs[k]];
                    let mut gr = *b;
                    Gdi::DrawTextW(dc, &mut one, &mut gr, Gdi::DT_CENTER | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE);
                }
                Gdi::SelectObject(dc, old);
                let _ = Gdi::DeleteObject(icons.into());
                let _ = Gdi::EndPaint(h, &ps);
                LRESULT(0)
            }
            wm::WM_MOUSEMOVE => {
                let (x, y) = win::point_of(l);
                let hot = caption_boxes(h).iter().position(|b| x >= b.left && x < b.right && y >= b.top && y < b.bottom);
                if hot != s.hot {
                    s.hot = hot;
                    let mut t = TRACKMOUSEEVENT { cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32, dwFlags: TME_LEAVE, hwndTrack: h, dwHoverTime: 0 };
                    let _ = TrackMouseEvent(&mut t);
                    let _ = Gdi::InvalidateRect(Some(h), None, false);
                }
                LRESULT(0)
            }
            0x02A3 => {
                // WM_MOUSELEAVE
                s.hot = None;
                let _ = Gdi::InvalidateRect(Some(h), None, false);
                LRESULT(0)
            }
            wm::WM_LBUTTONDOWN if s.hot.is_none() => {
                let id = wm::GetDlgCtrlID(h) as usize;
                if let Ok(p) = wm::GetParent(h) {
                    wm::SendMessageW(p, wm::WM_COMMAND, Some(WPARAM(((CAPTION_DRAG as usize) << 16) | (id & 0xFFFF))), Some(LPARAM(h.0 as isize)));
                }
                LRESULT(0)
            }
            wm::WM_LBUTTONUP if s.hot.is_some() => {
                let code = [CAPTION_MENU, CAPTION_PIN, CAPTION_CLOSE][s.hot.unwrap_or(2)];
                let id = wm::GetDlgCtrlID(h) as usize;
                if let Ok(p) = wm::GetParent(h) {
                    wm::SendMessageW(p, wm::WM_COMMAND, Some(WPARAM(((code as usize) << 16) | (id & 0xFFFF))), Some(LPARAM(h.0 as isize)));
                }
                LRESULT(0)
            }
            wm::WM_NCDESTROY => {
                win::forget_kind(h);
                drop(win::take_state::<Caption>(h));
                LRESULT(0)
            }
            _ => wm::DefWindowProcW(h, msg, w, l),
        }
    }
}

struct Splitter {
    font: HFONT,
    across: bool,
    last: Option<i32>,
}

/// A bar to drag: `across` sizes things left and right (a vertical bar), else up and down. The
/// parent hears [`WM_SPLIT`] as it moves.
pub fn splitter(parent: HWND, id: i32, across: bool) -> HWND {
    let h = create(parent, "PhaiSplitter", Some(splitter_proc), "", id, HFONT::default(), WINDOW_EX_STYLE(0));
    win::set_state(h, Box::new(Splitter { font: HFONT::default(), across, last: None }));
    h
}

extern "system" fn splitter_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture};
    unsafe {
        let Some(s) = win::state::<Splitter>(h) else { return wm::DefWindowProcW(h, msg, w, l) };
        if let Some(r) = common_messages(h, msg, w, l, &mut s.font) {
            return r;
        }
        let across = s.across;
        let screen = |l: LPARAM| {
            let (x, y) = win::point_of(l);
            let mut pt = windows::Win32::Foundation::POINT { x, y };
            let _ = Gdi::ClientToScreen(h, &mut pt);
            if across {
                pt.x
            } else {
                pt.y
            }
        };
        match msg {
            wm::WM_ERASEBKGND => {
                let dc = Gdi::HDC(w.0 as *mut _);
                let mut r = RECT::default();
                let _ = wm::GetClientRect(h, &mut r);
                win::paint_behind(h, dc, &r);
                // A hairline in the middle, as a divider.
                let rule = if across { RECT { left: r.right / 2, right: r.right / 2 + 1, ..r } } else { RECT { top: r.bottom / 2, bottom: r.bottom / 2 + 1, ..r } };
                win::fill(dc, &rule, win::style_of(h, Kind::Tabs, State::default()).border_or(Color::rgb(229, 229, 229)));
                LRESULT(1)
            }
            wm::WM_SETCURSOR => {
                wm::SetCursor(wm::LoadCursorW(None, if across { wm::IDC_SIZEWE } else { wm::IDC_SIZENS }).ok());
                LRESULT(1)
            }
            wm::WM_LBUTTONDOWN => {
                SetCapture(h);
                s.last = Some(screen(l));
                LRESULT(0)
            }
            wm::WM_MOUSEMOVE => {
                if let Some(last) = s.last {
                    let now = screen(l);
                    if now != last {
                        s.last = Some(now);
                        let id = wm::GetDlgCtrlID(h) as usize;
                        if let Ok(p) = wm::GetParent(h) {
                            wm::SendMessageW(p, WM_SPLIT, Some(WPARAM(id)), Some(LPARAM((now - last) as isize)));
                        }
                    }
                }
                LRESULT(0)
            }
            wm::WM_LBUTTONUP => {
                s.last = None;
                let _ = ReleaseCapture();
                LRESULT(0)
            }
            wm::WM_NCDESTROY => {
                drop(win::take_state::<Splitter>(h));
                LRESULT(0)
            }
            _ => wm::DefWindowProcW(h, msg, w, l),
        }
    }
}
