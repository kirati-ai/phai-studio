//! Auto Hide, as Visual Studio has it: a pane whose pin is pulled leaves its room to the form and
//! becomes a tab on a thin strip at its edge, its name written down the strip. Pointing at the tab
//! or clicking it slides the pane out over the form; it goes again once the pointer and the focus
//! have left it. This file is the strip; the studio moves the panes.

use phai_ui::text::t;
use phai_ui::theme::{self, Color, State};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HFONT};
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HMENU};

use crate::{studio, Pane};

/// The strip's width, design pixels.
pub const WIDTH: i32 = 24;

struct Strip {
    /// The left edge's strip (its tabs' bar on the right), or the right edge's.
    left: bool,
    tabs: Vec<Pane>,
    hover: Option<usize>,
    font: HFONT,
    /// The font turned to run down the strip.
    down: HFONT,
}

/// A pane's name on its tab.
pub fn name_of(p: Pane) -> &'static str {
    match p {
        Pane::Toolbox => t("Toolbox"),
        Pane::Explorer => t("Project"),
        Pane::Properties => t("Properties"),
        Pane::Output => t("Errors and output"),
    }
}

pub fn new(parent: HWND, left: bool, font: HFONT) -> HWND {
    let class = win::register("PhaiAutoHideStrip", Some(proc_), None);
    unsafe {
        let h = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_CLIPSIBLINGS, 0, 0, 10, 10, Some(parent), Some(HMENU::default()), Some(win::instance()), None).unwrap_or_default();
        win::set_state(h, Box::new(Strip { left, tabs: vec![], hover: None, font, down: down_font(h, font) }));
        h
    }
}

fn down_font(h: HWND, font: HFONT) -> HFONT {
    unsafe {
        let mut lf = Gdi::LOGFONTW::default();
        Gdi::GetObjectW(font.into(), std::mem::size_of::<Gdi::LOGFONTW>() as i32, Some(&mut lf as *mut _ as *mut _));
        if lf.lfHeight == 0 {
            lf.lfHeight = -win::px(h, 12);
        }
        lf.lfEscapement = 2700;
        lf.lfOrientation = 2700;
        Gdi::CreateFontIndirectW(&lf)
    }
}

/// The tabs it shows; none, and the studio gives it no room.
pub fn set_tabs(h: HWND, tabs: Vec<Pane>) {
    if let Some(s) = unsafe { win::state::<Strip>(h) } {
        s.tabs = tabs;
        s.hover = None;
        unsafe {
            let _ = Gdi::InvalidateRect(Some(h), None, false);
        }
    }
}

/// Each tab's span down the strip, device pixels.
fn spans(h: HWND, s: &Strip) -> Vec<(i32, i32)> {
    let gap = win::px(h, 6);
    let mut y = win::px(h, 4);
    s.tabs
        .iter()
        .map(|p| {
            let (w, _) = win::measure(h, s.font, name_of(*p));
            let span = (y, y + w + win::px(h, 16));
            y = span.1 + gap;
            span
        })
        .collect()
}

fn paint(h: HWND) {
    let Some(s) = (unsafe { win::state::<Strip>(h) }) else { return };
    unsafe {
        let mut ps = Gdi::PAINTSTRUCT::default();
        let screen = Gdi::BeginPaint(h, &mut ps);
        let buf = phai_ui::win::off_screen(h, screen);
        let dc = buf.dc;
        let mut r = RECT::default();
        let _ = wm::GetClientRect(h, &mut r);
        let base = theme::style("tabs", State::default());
        win::fill(dc, &r, base.background_or(win::window_background()));
        let rule = base.border_or(Color::rgb(229, 229, 229));
        let accent = theme::current().var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f));
        let muted = theme::current().var_color("--muted").unwrap_or(Color::rgb(0x5f, 0x5f, 0x5f));
        // The edge it stands against the form by.
        win::fill(dc, &if s.left { RECT { left: r.right - 1, ..r } } else { RECT { right: 1, ..r } }, rule);
        let bar = win::px(h, 3);
        let (_, text_h) = win::measure(h, s.font, "Ag");
        let old = Gdi::SelectObject(dc, s.down.into());
        Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        for (k, (a, b)) in spans(h, s).into_iter().enumerate() {
            let on = s.hover == Some(k);
            // A bar along the tab, as Visual Studio marks one: the accent under the pointer.
            let br = if s.left { RECT { left: r.right - bar - 1, right: r.right - 1, top: a, bottom: b } } else { RECT { left: 1, right: 1 + bar, top: a, bottom: b } };
            win::fill(dc, &br, if on { accent } else { rule });
            // A line across each end of the tab, as the other tabs have down their sides.
            let sep = win::window_background().mix(win::window_text(), 0.18);
            win::fill(dc, &RECT { top: a, bottom: a + 1, ..r }, sep);
            win::fill(dc, &RECT { top: b - 1, bottom: b, ..r }, sep);
            Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(if on { win::window_text() } else { muted }.colorref()));
            // Turned a quarter clockwise, the words hang left of where they start.
            let x = (r.right + text_h) / 2 - if s.left { bar / 2 } else { -bar / 2 };
            let w: Vec<u16> = name_of(s.tabs[k]).encode_utf16().collect();
            let _ = Gdi::TextOutW(dc, x, a + win::px(h, 8), &w);
        }
        Gdi::SelectObject(dc, old);
        drop(buf);
        let _ = Gdi::EndPaint(h, &ps);
    }
}

fn tab_at(h: HWND, y: i32) -> Option<usize> {
    let s = unsafe { win::state::<Strip>(h) }?;
    spans(h, s).iter().position(|(a, b)| y >= *a && y < *b)
}

fn open(h: HWND, k: usize) {
    let Some(p) = (unsafe { win::state::<Strip>(h) }).and_then(|s| s.tabs.get(k).copied()) else { return };
    if let Some(st) = studio() {
        st.slide_out(p);
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_PAINT => {
            paint(h);
            LRESULT(0)
        }
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_MOUSEMOVE => {
            let (_, y) = win::point_of(l);
            let k = tab_at(h, y);
            if let Some(s) = unsafe { win::state::<Strip>(h) } {
                if s.hover != k {
                    s.hover = k;
                    unsafe {
                        let _ = Gdi::InvalidateRect(Some(h), None, false);
                    }
                }
            }
            // Resting on a tab opens it, as in Visual Studio.
            let mut t = km::TRACKMOUSEEVENT { cbSize: std::mem::size_of::<km::TRACKMOUSEEVENT>() as u32, dwFlags: km::TME_HOVER | km::TME_LEAVE, hwndTrack: h, dwHoverTime: 400 };
            unsafe {
                let _ = km::TrackMouseEvent(&mut t);
            }
            LRESULT(0)
        }
        0x02A1 /* WM_MOUSEHOVER */ | wm::WM_LBUTTONDOWN => {
            let (_, y) = win::point_of(l);
            if let Some(k) = tab_at(h, y) {
                open(h, k);
            }
            LRESULT(0)
        }
        0x02A3 /* WM_MOUSELEAVE */ => {
            if let Some(s) = unsafe { win::state::<Strip>(h) } {
                s.hover = None;
                unsafe {
                    let _ = Gdi::InvalidateRect(Some(h), None, false);
                }
            }
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            if let Some(s) = unsafe { win::take_state::<Strip>(h) } {
                unsafe {
                    let _ = Gdi::DeleteObject(s.down.into());
                }
            }
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}
