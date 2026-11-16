//! Moving a pane, as Visual Studio does: drag its caption and a shaded box shows where it will go -
//! against the window's left, right, top or bottom edge, or floating on its own where the pointer
//! lets go. A floating pane lives in a small window of its own; double-click that window's title
//! to dock it again.

use phai_ui::layout::Dock;
use phai_ui::text::t;
use phai_ui::theme::{self, Color};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::WindowsAndMessaging::{self as wm};

use crate::studio;

/// Where a dragged pane would go.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Target {
    Edge(Dock),
    Float(RECT),
}

/// The target for the pointer at `pt` (screen pixels) over the room `area` (screen pixels) the
/// panes share: near an edge, that edge; elsewhere, floating where the pointer is.
pub fn target_at(pt: POINT, area: RECT, zone: i32, float_size: (i32, i32)) -> Target {
    let inside = pt.x >= area.left && pt.x < area.right && pt.y >= area.top && pt.y < area.bottom;
    if inside {
        let d = [(pt.x - area.left, Dock::Left), (area.right - pt.x, Dock::Right), (pt.y - area.top, Dock::Top), (area.bottom - pt.y, Dock::Bottom)];
        if let Some((_, dock)) = d.iter().filter(|(dist, _)| *dist < zone).min_by_key(|(dist, _)| *dist) {
            return Target::Edge(*dock);
        }
    }
    Target::Float(RECT { left: pt.x - 40, top: pt.y - 12, right: pt.x - 40 + float_size.0, bottom: pt.y - 12 + float_size.1 })
}

/// The box shown for a target: the strip of `area` an edge would give (`across` wide, `down` high).
pub fn preview_rect(t: Target, area: RECT, across: i32, down: i32) -> RECT {
    match t {
        Target::Edge(Dock::Left) => RECT { right: area.left + across, ..area },
        Target::Edge(Dock::Right) => RECT { left: area.right - across, ..area },
        Target::Edge(Dock::Top) => RECT { bottom: area.top + down, ..area },
        Target::Edge(_) => RECT { top: area.bottom - down, ..area },
        Target::Float(r) => r,
    }
}

/// The shaded box: see-through, in the accent colour, taking no clicks and no focus.
pub fn preview_window(owner: HWND) -> HWND {
    let class = win::register("PhaiDockPreview", Some(preview_proc), None);
    unsafe {
        let ex = wm::WS_EX_LAYERED | wm::WS_EX_TRANSPARENT | wm::WS_EX_TOOLWINDOW | wm::WS_EX_TOPMOST | wm::WS_EX_NOACTIVATE;
        let h = wm::CreateWindowExW(ex, PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_POPUP, 0, 0, 10, 10, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let _ = wm::SetLayeredWindowAttributes(h, COLORREF(0), 96, wm::LWA_ALPHA);
        h
    }
}

extern "system" fn preview_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            let accent = theme::current().var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f));
            win::fill(dc, &r, accent);
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        wm::WM_ERASEBKGND => LRESULT(1),
        _ => win::def(h, msg, w, l),
    }
}

/// A small window of its own for a floating pane, named `title`, owned by the studio.
pub fn float_window(owner: HWND, title: &str, r: RECT) -> HWND {
    let class = win::register("PhaiFloatPane", Some(float_proc), None);
    let title = win::wide(title);
    unsafe {
        let style = wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_THICKFRAME | wm::WS_CLIPCHILDREN;
        let h = wm::CreateWindowExW(wm::WS_EX_TOOLWINDOW, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), style, r.left, r.top, r.right - r.left, r.bottom - r.top, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        win::theme_frame(h);
        h
    }
}

extern "system" fn float_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_SIZE => {
            // The pane fills its window.
            if let Ok(child) = unsafe { wm::GetWindow(h, wm::GW_CHILD) } {
                win::move_to(child, 0, 0, win::loword(l.0 as usize) as i32, win::hiword(l.0 as usize) as i32);
            }
            LRESULT(0)
        }
        // Closed: hidden, as Visual Studio hides a tool window; View shows it again.
        wm::WM_CLOSE => {
            if let Some(s) = studio() {
                s.show_pane(crate::Pane::Toolbox, false);
            }
            LRESULT(0)
        }
        // Its title double-clicked: back where it was docked.
        wm::WM_NCLBUTTONDBLCLK => {
            if let Some(s) = studio() {
                s.dock_toolbox(Target::Edge(s.toolbox_dock));
            }
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}

/// What the float window is called.
pub fn title() -> &'static str {
    t("Toolbox")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_edge_nearest_the_pointer_takes_the_pane_else_it_floats() {
        let area = RECT { left: 0, top: 100, right: 1000, bottom: 700 };
        let at = |x, y| target_at(POINT { x, y }, area, 60, (200, 400));
        assert_eq!(at(20, 400), Target::Edge(Dock::Left));
        assert_eq!(at(980, 400), Target::Edge(Dock::Right));
        assert_eq!(at(500, 130), Target::Edge(Dock::Top));
        assert_eq!(at(500, 690), Target::Edge(Dock::Bottom));
        assert!(matches!(at(500, 400), Target::Float(_)));
        assert!(matches!(at(500, 50), Target::Float(_)));
        let r = preview_rect(Target::Edge(Dock::Top), area, 200, 180);
        assert_eq!((r.top, r.bottom, r.left, r.right), (100, 280, 0, 1000));
    }
}
