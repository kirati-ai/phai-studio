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

/// The dock guides, as Visual Studio shows them while a pane is dragged: an arrow at the middle
/// of each edge of `area`, and a compass of four arrows round its centre - each `size` square.
/// The pointer over one puts the pane against that edge.
pub fn guides(area: RECT, size: i32) -> Vec<(Dock, RECT)> {
    let sq = |x: i32, y: i32| RECT { left: x, top: y, right: x + size, bottom: y + size };
    let (cx, cy) = ((area.left + area.right) / 2 - size / 2, (area.top + area.bottom) / 2 - size / 2);
    let m = size / 3;
    vec![
        (Dock::Left, sq(area.left + m, cy)),
        (Dock::Right, sq(area.right - m - size, cy)),
        (Dock::Top, sq(cx, area.top + m)),
        (Dock::Bottom, sq(cx, area.bottom - m - size)),
        (Dock::Left, sq(cx - size, cy)),
        (Dock::Right, sq(cx + size, cy)),
        (Dock::Top, sq(cx, cy - size)),
        (Dock::Bottom, sq(cx, cy + size)),
    ]
}

/// The guide under the pointer, if any.
pub fn guide_at(pt: POINT, guides: &[(Dock, RECT)]) -> Option<Dock> {
    guides.iter().find(|(_, r)| pt.x >= r.left && pt.x < r.right && pt.y >= r.top && pt.y < r.bottom).map(|(d, _)| *d)
}

/// One guide's window: `dock` says which way its arrow points. Shown without taking the focus.
pub fn guide_window(owner: HWND, dock: Dock, r: RECT) -> HWND {
    let class = win::register("PhaiDockGuide", Some(guide_proc), None);
    unsafe {
        let ex = wm::WS_EX_LAYERED | wm::WS_EX_TRANSPARENT | wm::WS_EX_TOOLWINDOW | wm::WS_EX_TOPMOST | wm::WS_EX_NOACTIVATE;
        let h = wm::CreateWindowExW(ex, PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_POPUP, r.left, r.top, r.right - r.left, r.bottom - r.top, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let _ = wm::SetLayeredWindowAttributes(h, COLORREF(0), 235, wm::LWA_ALPHA);
        let which = match dock {
            Dock::Left => 0,
            Dock::Right => 1,
            Dock::Top => 2,
            _ => 3,
        };
        wm::SetWindowLongPtrW(h, wm::GWLP_USERDATA, which);
        let _ = wm::ShowWindow(h, wm::SW_SHOWNOACTIVATE);
        h
    }
}

/// A guide lit (the pointer over it) or not.
pub fn light_guide(h: HWND, on: bool) {
    unsafe {
        let v = wm::GetWindowLongPtrW(h, wm::GWLP_USERDATA);
        let now = (v & 3) | if on { 4 } else { 0 };
        if now != v {
            wm::SetWindowLongPtrW(h, wm::GWLP_USERDATA, now);
            let _ = Gdi::InvalidateRect(Some(h), None, false);
        }
    }
}

extern "system" fn guide_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            let v = wm::GetWindowLongPtrW(h, wm::GWLP_USERDATA);
            let lit = v & 4 != 0;
            let accent = theme::current().var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f));
            let back = win::window_background();
            let line = theme::current().var_color("--border").unwrap_or(Color::rgb(0xa0, 0xa0, 0xa0));
            win::fill(dc, &r, line);
            let inner = RECT { left: r.left + 1, top: r.top + 1, right: r.right - 1, bottom: r.bottom - 1 };
            win::fill(dc, &inner, if lit { accent } else { back });
            // A small box with an arrow towards the edge, as Visual Studio draws it.
            let (w, hh) = (r.right, r.bottom);
            let u = w / 8;
            let ink = if lit { back } else { accent };
            let pts: [POINT; 3] = match v & 3 {
                0 => [POINT { x: 2 * u, y: hh / 2 }, POINT { x: 4 * u, y: hh / 2 - 2 * u }, POINT { x: 4 * u, y: hh / 2 + 2 * u }],
                1 => [POINT { x: w - 2 * u, y: hh / 2 }, POINT { x: w - 4 * u, y: hh / 2 - 2 * u }, POINT { x: w - 4 * u, y: hh / 2 + 2 * u }],
                2 => [POINT { x: w / 2, y: 2 * u }, POINT { x: w / 2 - 2 * u, y: 4 * u }, POINT { x: w / 2 + 2 * u, y: 4 * u }],
                _ => [POINT { x: w / 2, y: hh - 2 * u }, POINT { x: w / 2 - 2 * u, y: hh - 4 * u }, POINT { x: w / 2 + 2 * u, y: hh - 4 * u }],
            };
            let brush = Gdi::CreateSolidBrush(COLORREF(ink.colorref()));
            let pen = Gdi::CreatePen(Gdi::PS_SOLID, 1, COLORREF(ink.colorref()));
            let ob = Gdi::SelectObject(dc, brush.into());
            let op = Gdi::SelectObject(dc, pen.into());
            let _ = Gdi::Polygon(dc, &pts);
            // The strip the pane would take, beside the arrow.
            let strip = match v & 3 {
                0 => RECT { left: 4 * u + u, top: 2 * u, right: w - 2 * u, bottom: hh - 2 * u },
                1 => RECT { left: 2 * u, top: 2 * u, right: w - 5 * u, bottom: hh - 2 * u },
                2 => RECT { left: 2 * u, top: 5 * u, right: w - 2 * u, bottom: hh - 2 * u },
                _ => RECT { left: 2 * u, top: 2 * u, right: w - 2 * u, bottom: hh - 5 * u },
            };
            let _ = Gdi::FrameRect(dc, &strip, brush);
            Gdi::SelectObject(dc, ob);
            Gdi::SelectObject(dc, op);
            let _ = Gdi::DeleteObject(brush.into());
            let _ = Gdi::DeleteObject(pen.into());
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        wm::WM_ERASEBKGND => LRESULT(1),
        _ => win::def(h, msg, w, l),
    }
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
        // A guide under the pointer wins: the compass's right arrow, in the middle.
        let g = guides(area, 40);
        assert_eq!(guide_at(POINT { x: 500 + 40, y: 400 }, &g), Some(Dock::Right));
        assert_eq!(guide_at(POINT { x: 20, y: 400 }, &g), Some(Dock::Left));
        assert_eq!(guide_at(POINT { x: 500, y: 400 }, &g), None, "the compass's middle is no guide");
        let r = preview_rect(Target::Edge(Dock::Top), area, 200, 180);
        assert_eq!((r.top, r.bottom, r.left, r.right), (100, 280, 0, 1000));
    }
}
