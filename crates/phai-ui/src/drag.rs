//! Dragging words from one control to another - WinForms' `DoDragDrop` and a DragDrop event on
//! the control it lands on. A program starts it in a control's MouseDown (or MouseMove); the
//! words follow the pointer in a small label; a control that hears its DragDrop event takes them
//! (`e.text`, `e.source`), anything else shows the "no" pointer and lets them fall. Files from
//! Explorer come to the same event as `e.files`.

use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::events::{self, FileDropEventArgs};
use crate::win::{self, wide};

thread_local! {
    static GHOST_TEXT: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

extern "system" fn ghost_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if msg == wm::WM_PAINT {
        unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            let th = crate::theme::current();
            let accent = th.var_color("--accent").unwrap_or(crate::theme::Color::rgb(0x12, 0x85, 0x5f));
            win::fill(dc, &r, accent);
            let f = win::font(h, 0, false);
            let old = Gdi::SelectObject(dc, f.into());
            Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
            Gdi::SetTextColor(dc, COLORREF(0x00FF_FFFF));
            let mut t = wide(&GHOST_TEXT.with(|g| g.borrow().clone()));
            let n = t.len().saturating_sub(1);
            let mut tr = RECT { left: r.left + win::px(h, 8), ..r };
            crate::win::draw_text_w(dc, &mut t[..n], &mut tr, Gdi::DT_SINGLELINE | Gdi::DT_VCENTER | Gdi::DT_NOPREFIX | Gdi::DT_END_ELLIPSIS);
            Gdi::SelectObject(dc, old);
            let _ = Gdi::DeleteObject(f.into());
            let _ = Gdi::EndPaint(h, &ps);
        }
        return LRESULT(0);
    }
    win::def(h, msg, w, l)
}

/// The control under screen point `p` that takes a drop: (its form, its id), and the control.
fn target_at(p: POINT, ghost: HWND) -> Option<(HWND, (HWND, i32))> {
    let mut h = unsafe { wm::WindowFromPoint(p) };
    if h == ghost {
        return None;
    }
    // A list's header or a combo's box: the control they belong to.
    while !h.is_invalid() {
        if let Some(t) = events::drop_target(h) {
            return Some((h, t));
        }
        h = unsafe { wm::GetParent(h) }.unwrap_or_default();
    }
    None
}

/// Drags `text` from `source` until the mouse button is let go: true when a control took it.
/// Esc gives it up. Call it from the source's MouseDown.
pub fn do_drag_drop(source: impl Into<HWND>, text: &str) -> bool {
    let source: HWND = source.into();
    GHOST_TEXT.with(|g| *g.borrow_mut() = text.to_string());
    let class = win::register("PhaiDragGhost", Some(ghost_proc), None);
    let ex = wm::WS_EX_LAYERED | wm::WS_EX_TRANSPARENT | wm::WS_EX_TOOLWINDOW | wm::WS_EX_NOACTIVATE | wm::WS_EX_TOPMOST;
    let ghost = unsafe { wm::CreateWindowExW(ex, PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_POPUP, 0, 0, 10, 10, None, None, Some(win::instance()), None) }.unwrap_or_default();
    // As wide as its words, up to a limit.
    let width = unsafe {
        let dc = Gdi::GetDC(Some(ghost));
        let f = win::font(ghost, 0, false);
        let old = Gdi::SelectObject(dc, f.into());
        let w = wide(text);
        let mut s = SIZE::default();
        let _ = Gdi::GetTextExtentPoint32W(dc, &w[..w.len().saturating_sub(1)], &mut s);
        Gdi::SelectObject(dc, old);
        let _ = Gdi::DeleteObject(f.into());
        Gdi::ReleaseDC(Some(ghost), dc);
        (s.cx + win::px(ghost, 16)).min(win::px(ghost, 320))
    };
    let height = win::px(ghost, 24);
    unsafe {
        let _ = wm::SetLayeredWindowAttributes(ghost, COLORREF(0), 210, wm::LWA_ALPHA);
        km::SetCapture(ghost);
    }
    let (no, yes) = unsafe { (wm::LoadCursorW(None, wm::IDC_NO).ok(), wm::LoadCursorW(None, wm::IDC_ARROW).ok()) };
    let mut over: Option<(HWND, (HWND, i32))> = None;
    let mut dropped = false;
    let mut msg = wm::MSG::default();
    loop {
        if !unsafe { wm::GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            unsafe { wm::PostQuitMessage(msg.wParam.0 as i32) };
            break;
        }
        match msg.message {
            wm::WM_MOUSEMOVE => {
                let mut p = POINT::default();
                unsafe {
                    let _ = wm::GetCursorPos(&mut p);
                    let _ = wm::SetWindowPos(ghost, Some(wm::HWND_TOPMOST), p.x + win::px(ghost, 14), p.y + win::px(ghost, 10), width, height, wm::SWP_NOACTIVATE | wm::SWP_SHOWWINDOW);
                }
                over = target_at(p, ghost).filter(|(h, _)| *h != source);
                unsafe {
                    wm::SetCursor(if over.is_some() { yes } else { no });
                }
            }
            wm::WM_LBUTTONUP => {
                if let Some((_, (form, id))) = over {
                    let a = FileDropEventArgs { files: Vec::new(), text: Some(text.to_string()), source: Some(source) };
                    events::raise(form, id, events::DRAG_DROP, &a);
                    dropped = true;
                }
                break;
            }
            wm::WM_KEYDOWN if msg.wParam.0 == 0x1B => break,
            wm::WM_CAPTURECHANGED => break,
            _ => unsafe {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            },
        }
        if unsafe { km::GetCapture() } != ghost {
            break;
        }
    }
    unsafe {
        let _ = km::ReleaseCapture();
        let _ = wm::DestroyWindow(ghost);
    }
    dropped
}

/// For checking: what dropping `text` from `source` on the control at screen point `p` does - the
/// drop as the mouse would make it, without the mouse.
pub fn drop_at(source: HWND, text: &str, p: POINT) -> bool {
    match target_at(p, HWND::default()).filter(|(h, _)| *h != source) {
        Some((_, (form, id))) => {
            let a = FileDropEventArgs { files: Vec::new(), text: Some(text.to_string()), source: Some(source) };
            events::raise(form, id, events::DRAG_DROP, &a);
            true
        }
        None => false,
    }
}
