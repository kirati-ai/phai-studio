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
                win::fill(dc, &r, win::window_background());
                let text = win::text_of(h);
                let (tw, th) = win::measure(h, s.font, &text);
                let top = th / 2;
                let frame = RECT { top: r.top + top, ..r };
                win::round_rect(dc, frame, Color::TRANSPARENT, st.border_or(Color::rgb(229, 229, 229)), px(h, st.border_radius.unwrap_or(6) * 2));
                if !text.is_empty() {
                    let x = px(h, 10);
                    let gap = px(h, 4);
                    let back = RECT { left: x - gap, top: r.top, right: x + tw + gap, bottom: r.top + th };
                    win::fill(dc, &back, win::window_background());
                    let mut tr = RECT { left: x, top: r.top, right: x + tw, bottom: r.top + th };
                    win::draw_text(dc, h, &text, &mut tr, st.color_or(win::window_text()), Gdi::DT_LEFT | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX);
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
}

/// Tabs along the top, the chosen one underlined in the accent colour. The parent shows the
/// chosen page's controls; it hears `WM_COMMAND` with `id` and the index in the high word.
pub fn tabs(parent: HWND, id: i32, f: HFONT, labels: &[&str]) -> HWND {
    let h = create(parent, "PhaiTabs", Some(tabs_proc), "", id, f, WINDOW_EX_STYLE(0));
    win::set_state(h, Box::new(Tabs { font: f, labels: labels.iter().map(|s| s.to_string()).collect(), selected: 0, hot: None }));
    mark(h, Kind::Tabs);
    h
}

pub fn tabs_selected(h: HWND) -> usize {
    unsafe { win::state::<Tabs>(h).map(|t| t.selected).unwrap_or(0) }
}

pub fn tabs_select(h: HWND, i: usize) {
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
            let (w, _) = win::measure(h, t.font, l);
            let e = (x, x + w + 2 * pad);
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
        match msg {
            wm::WM_ERASEBKGND => LRESULT(1),
            wm::WM_PAINT => {
                let mut ps = Gdi::PAINTSTRUCT::default();
                let dc = Gdi::BeginPaint(h, &mut ps);
                let mut r = RECT::default();
                let _ = wm::GetClientRect(h, &mut r);
                let base = win::style_of(h, Kind::Tabs, State::default());
                win::fill(dc, &r, base.background_or(win::window_background()));
                let rule = RECT { top: r.bottom - 1, ..r };
                win::fill(dc, &rule, base.border_or(Color::rgb(229, 229, 229)));
                for (i, (a, b)) in tab_edges(h, t).into_iter().enumerate() {
                    let st = win::style_of(h, Kind::Tabs, State { selected: i == t.selected, hover: t.hot == Some(i), ..Default::default() });
                    let mut tr = RECT { left: a, right: b, top: r.top, bottom: r.bottom - px(h, 3) };
                    win::draw_text(dc, h, &t.labels[i], &mut tr, st.color_or(win::window_text()), Gdi::DT_CENTER | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX);
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
                let hot = at(win::point_of(l).0, t);
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
pub fn status_parts(h: HWND, widths: &[i32]) {
    unsafe {
        if let Some(s) = win::state::<Status>(h) {
            s.widths = std::iter::once(0).chain(widths.iter().copied()).collect();
            s.texts.resize(s.widths.len(), String::new());
            let _ = Gdi::InvalidateRect(Some(h), None, true);
        }
    }
}

pub fn status_text(h: HWND, part: usize, text: &str) {
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
                win::fill(dc, &r, st.background_or(win::window_background()));
                let rule = RECT { bottom: r.top + 1, ..r };
                win::fill(dc, &rule, st.border_or(Color::rgb(229, 229, 229)));
                let fg = st.color_or(win::window_text());
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
                    win::draw_text(dc, h, text, &mut tr, fg, Gdi::DT_LEFT | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX | Gdi::DT_END_ELLIPSIS);
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
