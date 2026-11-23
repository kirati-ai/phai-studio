//! WinForms' SplitContainer: two panes side by side (or one over the other) with a bar between
//! them the mouse drags. The controls in each pane keep their anchors to that pane, so a list
//! anchored all round grows as its pane does. The window's [`crate::layout::Layout`] places them
//! ([`crate::layout::Layout::add_split`]); dragging the bar asks it to again.

use std::collections::HashMap;
use std::sync::Mutex;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::layout::{scale, unscale};
use crate::theme::Color;
use crate::win::{self, Kind};

/// The bar's thickness, design pixels - WinForms' SplitterWidth.
pub const BAR: i32 = 4;

#[derive(Clone, Copy, Debug)]
struct Split {
    vertical: bool,
    /// The first pane's width (or height), design pixels.
    distance: i32,
    /// Where a drag began: the mouse, the distance then.
    drag: Option<(i32, i32)>,
}

static SPLITS: Mutex<Option<HashMap<isize, Split>>> = Mutex::new(None);

fn with<R>(h: HWND, f: impl FnOnce(&mut Split) -> R) -> Option<R> {
    SPLITS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).get_mut(&(h.0 as isize)).map(f)
}

/// A split container: `vertical`, the panes are left and right (WinForms' Orientation.Vertical);
/// `distance`, the first pane's size in design pixels.
pub fn split(parent: HWND, id: i32, vertical: bool, distance: i32) -> HWND {
    let class = win::register("PhaiSplit", Some(proc_), None);
    let h = unsafe { wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPSIBLINGS, 0, 0, 10, 10, Some(parent), Some(wm::HMENU(id as isize as *mut _)), Some(win::instance()), None) }.unwrap_or_default();
    // Under the controls in its panes, as a group frame is.
    win::register_kind(h, Kind::Group);
    SPLITS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).insert(h.0 as isize, Split { vertical, distance, drag: None });
    h
}

/// Whether the panes are side by side.
pub fn is_vertical(h: HWND) -> bool {
    with(h, |s| s.vertical).unwrap_or(true)
}

/// The first pane's size now, design pixels - SplitterDistance.
pub fn distance(h: HWND) -> i32 {
    with(h, |s| s.distance).unwrap_or(0)
}

/// Moves the bar: the panes and what is in them are placed again.
pub fn set_distance(h: HWND, d: i32) {
    with(h, |s| s.distance = d.max(0));
    relayout(h);
}

/// A pane: x, y, width, height - design pixels.
pub type Pane = (i32, i32, i32, i32);

/// The two panes inside a split of `w` by `h` (design pixels) with the bar at `distance`.
pub fn panes(vertical: bool, distance: i32, w: i32, h: i32) -> (Pane, Pane) {
    let total = if vertical { w } else { h };
    let d = distance.clamp(0, (total - BAR).max(0));
    if vertical {
        ((0, 0, d, h), (d + BAR, 0, (w - d - BAR).max(0), h))
    } else {
        ((0, 0, w, d), (0, d + BAR, w, (h - d - BAR).max(0)))
    }
}

/// The window asked to place its controls again, as when it is sized.
fn relayout(h: HWND) {
    unsafe {
        if let Ok(parent) = wm::GetParent(h) {
            let mut c = RECT::default();
            let _ = wm::GetClientRect(parent, &mut c);
            wm::SendMessageW(parent, wm::WM_SIZE, Some(WPARAM(0)), Some(LPARAM(((c.bottom as isize) << 16) | (c.right as isize & 0xFFFF))));
            let _ = Gdi::InvalidateRect(Some(h), None, true);
        }
    }
}

fn bar_rect(h: HWND) -> RECT {
    let mut c = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut c);
    }
    let dpi = win::dpi(h);
    let (vertical, d) = with(h, |s| (s.vertical, s.distance)).unwrap_or((true, 0));
    let (w, ht) = (unscale(c.right, dpi), unscale(c.bottom, dpi));
    let (_, p2) = panes(vertical, d, w, ht);
    if vertical {
        RECT { left: scale(p2.0 - BAR, dpi), top: 0, right: scale(p2.0, dpi), bottom: c.bottom }
    } else {
        RECT { left: 0, top: scale(p2.1 - BAR, dpi), right: c.right, bottom: scale(p2.1, dpi) }
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let screen = Gdi::BeginPaint(h, &mut ps);
            let buf = crate::win::off_screen(h, screen);
            let dc = buf.dc;
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            match win::style_of(h, Kind::Group, Default::default()).background {
                Some(c) => win::fill(dc, &r, c),
                None => win::paint_behind(h, dc, &r),
            }
            // The bar: a thin line in its middle, the rule colour.
            let b = bar_rect(h);
            let line = crate::theme::current().var_color("--rule").unwrap_or(Color::rgb(0xe0, 0xe0, 0xe0));
            let t = win::px(h, 1).max(1);
            let mid = if is_vertical(h) { RECT { left: (b.left + b.right - t) / 2, right: (b.left + b.right + t) / 2, ..b } } else { RECT { top: (b.top + b.bottom - t) / 2, bottom: (b.top + b.bottom + t) / 2, ..b } };
            win::fill(dc, &mid, line);
            drop(buf);
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        wm::WM_SETCURSOR => {
            let mut p = windows::Win32::Foundation::POINT::default();
            unsafe {
                let _ = wm::GetCursorPos(&mut p);
                let _ = Gdi::ScreenToClient(h, &mut p);
            }
            let b = bar_rect(h);
            if p.x >= b.left && p.x < b.right && p.y >= b.top && p.y < b.bottom {
                unsafe {
                    let _ = wm::SetCursor(wm::LoadCursorW(None, if is_vertical(h) { wm::IDC_SIZEWE } else { wm::IDC_SIZENS }).ok());
                }
                return LRESULT(1);
            }
            win::def(h, msg, w, l)
        }
        wm::WM_LBUTTONDOWN => {
            let (x, y) = win::point_of(l);
            let b = bar_rect(h);
            if x >= b.left && x < b.right && y >= b.top && y < b.bottom {
                let at = if is_vertical(h) { x } else { y };
                with(h, |s| s.drag = Some((at, s.distance)));
                unsafe {
                    km::SetCapture(h);
                }
            }
            LRESULT(0)
        }
        wm::WM_MOUSEMOVE => {
            if let Some((from, d0)) = with(h, |s| s.drag).flatten() {
                let (x, y) = win::point_of(l);
                let at = if is_vertical(h) { x } else { y };
                let d = d0 + unscale(at - from, win::dpi(h));
                with(h, |s| s.distance = d.max(0));
                relayout(h);
            }
            LRESULT(0)
        }
        wm::WM_LBUTTONUP | wm::WM_CAPTURECHANGED => {
            if with(h, |s| s.drag.take()).flatten().is_some() {
                unsafe {
                    let _ = km::ReleaseCapture();
                    // SplitterMoved, as any change of value comes.
                    let id = wm::GetDlgCtrlID(h);
                    if let Ok(parent) = wm::GetParent(h) {
                        wm::SendMessageW(parent, wm::WM_COMMAND, Some(WPARAM((0x0300usize << 16) | (id as u16 as usize))), Some(LPARAM(h.0 as isize)));
                    }
                }
            }
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            if let Some(m) = SPLITS.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
                m.remove(&(h.0 as isize));
            }
            win::forget_kind(h);
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panes_share_what_the_bar_leaves() {
        assert_eq!(panes(true, 100, 300, 200), ((0, 0, 100, 200), (104, 0, 196, 200)));
        assert_eq!(panes(false, 50, 300, 200), ((0, 0, 300, 50), (0, 54, 300, 146)));
        assert_eq!(panes(true, 999, 300, 200), ((0, 0, 296, 200), (300, 0, 0, 200)), "the bar stays inside");
    }
}
