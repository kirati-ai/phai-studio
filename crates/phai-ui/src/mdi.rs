//! WinForms' MDI: a window (IsMdiContainer) whose forms open inside it, each with its own title
//! bar, cascaded or tiled - Windows' own MDICLIENT. The generated code makes the client for a
//! form with `is_mdi_container = true` and lays it out as what docking leaves; a form's
//! `show_mdi_child(parent, events)` opens it inside. [`crate::win::def`] gives each the default
//! handling Windows asks for (DefFrameProc, DefMDIChildProc), and [`crate::win::run_loop`] the
//! MDI keys (Ctrl+F4, Ctrl+Tab).

use std::cell::Cell;
use std::collections::HashMap;
use std::sync::Mutex;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::theme::Color;
use crate::win;

/// Frame → its MDI client.
static CLIENTS: Mutex<Option<HashMap<isize, isize>>> = Mutex::new(None);

thread_local! {
    /// The frame the next form made opens inside, set by `show_mdi_child`.
    static NEXT: Cell<Option<isize>> = const { Cell::new(None) };
}

/// The id of the first child window: Windows numbers the children from it for the Window menu.
const FIRST_CHILD: u32 = 0xFF00;

/// The MDI client of `frame`: the area its child forms live in.
pub fn client(frame: HWND) -> HWND {
    let class = win::wide("MDICLIENT");
    let ccs = wm::CLIENTCREATESTRUCT { hWindowMenu: Default::default(), idFirstChild: FIRST_CHILD };
    let h = unsafe { wm::CreateWindowExW(wm::WINDOW_EX_STYLE(0), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_CLIPCHILDREN | wm::WS_VISIBLE | wm::WS_VSCROLL | wm::WS_HSCROLL, 0, 0, 10, 10, Some(frame), None, Some(win::instance()), Some(&ccs as *const _ as *const _)) }.unwrap_or_default();
    CLIENTS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).insert(frame.0 as isize, h.0 as isize);
    unsafe {
        let _ = SetWindowSubclass(h, Some(client_proc), 12, 0);
    }
    h
}

/// The theme's quiet colour behind the child forms, not Windows' grey.
unsafe extern "system" fn client_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, _: usize) -> LRESULT {
    if msg == wm::WM_ERASEBKGND {
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(h, &mut r);
        }
        let th = crate::theme::current();
        let c = th.var_color("--disabled-bg").or_else(|| th.var_color("--rule")).unwrap_or(Color::rgb(0xe8, 0xe8, 0xe8));
        win::fill(Gdi::HDC(w.0 as *mut _), &r, c);
        return LRESULT(1);
    }
    unsafe { DefSubclassProc(h, msg, w, l) }
}

/// `frame`'s MDI client, if it is an MDI container.
pub fn client_of(frame: HWND) -> Option<HWND> {
    CLIENTS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|m| m.get(&(frame.0 as isize)).map(|c| HWND(*c as *mut _)))
}

/// The next form made opens inside `frame` (the generated `show_mdi_child`).
pub fn set_next_parent(frame: HWND) {
    NEXT.with(|n| n.set(Some(frame.0 as isize)));
}

/// The frame the form being made opens inside, if it is to be a child form - asked once.
pub fn take_next() -> Option<HWND> {
    NEXT.with(|n| n.take()).map(|f| HWND(f as *mut _))
}

/// A child form of `frame` (which must be an MDI container) of `class`, `title`, `style`.
pub fn create_child(frame: HWND, class: &[u16], title: &[u16], style: wm::WINDOW_STYLE) -> HWND {
    let Some(client) = client_of(frame) else { return HWND::default() };
    let style = (style | wm::WS_CHILD | wm::WS_CLIPCHILDREN) & !wm::WS_POPUP;
    unsafe { wm::CreateMDIWindowW(PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), style, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, Some(client), Some(win::instance()), LPARAM(0)) }.unwrap_or_default()
}

/// Whether `h` is a child form inside an MDI container.
pub fn is_child(h: HWND) -> bool {
    unsafe { wm::GetWindowLongW(h, wm::GWL_EXSTYLE) as u32 & wm::WS_EX_MDICHILD.0 != 0 }
}

/// The default handling an MDI window needs instead of DefWindowProc, if `h` is one.
pub fn def(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> Option<LRESULT> {
    if is_child(h) {
        return Some(unsafe { wm::DefMDIChildProcW(h, msg, w, l) });
    }
    let client = client_of(h)?;
    if msg == wm::WM_NCDESTROY {
        if let Some(m) = CLIENTS.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
            m.remove(&(h.0 as isize));
        }
    }
    Some(unsafe { wm::DefFrameProcW(h, Some(client), msg, w, l) })
}

/// The MDI keys of a message, for the message loop: true when it was one.
pub fn translate(msg: &wm::MSG) -> bool {
    let clients: Vec<isize> = CLIENTS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().map(|m| m.values().copied().collect()).unwrap_or_default();
    clients.into_iter().any(|c| unsafe { wm::TranslateMDISysAccel(HWND(c as *mut _), msg) }.as_bool())
}

fn send(frame: HWND, msg: u32, w: usize) -> isize {
    match client_of(frame) {
        Some(c) => unsafe { wm::SendMessageW(c, msg, Some(WPARAM(w)), Some(LPARAM(0))).0 },
        None => 0,
    }
}

/// The child forms one over another, each a step down and right - LayoutMdi(Cascade).
pub fn cascade(frame: HWND) {
    send(frame, wm::WM_MDICASCADE, 0);
}

/// Side by side (`vertical`) or one over another - LayoutMdi(TileVertical / TileHorizontal).
pub fn tile(frame: HWND, vertical: bool) {
    send(frame, wm::WM_MDITILE, if vertical { 0 } else { 1 });
}

/// The minimized ones in a row - LayoutMdi(ArrangeIcons).
pub fn arrange_icons(frame: HWND) {
    send(frame, wm::WM_MDIICONARRANGE, 0);
}

/// The child form in front, if any - ActiveMdiChild.
pub fn active_child(frame: HWND) -> Option<HWND> {
    let h = HWND(send(frame, wm::WM_MDIGETACTIVE, 0) as *mut _);
    (!h.is_invalid()).then_some(h)
}

/// Every child form - MdiChildren.
pub fn children(frame: HWND) -> Vec<HWND> {
    let Some(c) = client_of(frame) else { return Vec::new() };
    let mut out = Vec::new();
    let mut h = unsafe { wm::GetWindow(c, wm::GW_CHILD) }.ok();
    while let Some(x) = h.filter(|x| !x.is_invalid()) {
        if is_child(x) {
            out.push(x);
        }
        h = unsafe { wm::GetWindow(x, wm::GW_HWNDNEXT) }.ok();
    }
    out
}

/// The MDI container a child form is in - MdiParent.
pub fn parent_of(child: HWND) -> Option<HWND> {
    let client = unsafe { wm::GetParent(child) }.ok()?;
    unsafe { wm::GetParent(client) }.ok()
}
