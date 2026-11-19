//! What a form is beyond its controls, as WinForms has it: its frame (FormBorderStyle, the title
//! bar's buttons, TopMost, ShowInTaskbar, Opacity), its Icon, the buttons Enter and Escape click
//! (AcceptButton, CancelButton), the order Tab goes through, and the answer a dialog gives
//! (DialogResult).

use std::collections::HashMap;
use std::sync::Mutex;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging as wm;

/// What a dialog answers - the button that closed it, or `Cancel` for its close box.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DialogResult {
    #[default]
    None,
    Ok,
    Cancel,
    Yes,
    No,
    Retry,
    Abort,
    Ignore,
}

impl DialogResult {
    /// The `.phaiform`'s word for it: `ok`, `cancel`...
    pub fn from_name(s: &str) -> DialogResult {
        match s {
            "ok" => DialogResult::Ok,
            "cancel" => DialogResult::Cancel,
            "yes" => DialogResult::Yes,
            "no" => DialogResult::No,
            "retry" => DialogResult::Retry,
            "abort" => DialogResult::Abort,
            "ignore" => DialogResult::Ignore,
            _ => DialogResult::None,
        }
    }
}

static RESULTS: Mutex<Option<HashMap<isize, DialogResult>>> = Mutex::new(None);

/// Sets the window's answer and closes it - a button with a DialogResult, clicked.
pub fn close_with(hwnd: HWND, r: DialogResult) {
    RESULTS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).insert(hwnd.0 as isize, r);
    unsafe {
        let _ = wm::DestroyWindow(hwnd);
    }
}

/// The answer a closed window gave: what [`close_with`] set, or `Cancel` when it was closed any
/// other way - as WinForms' ShowDialog.
pub fn take_result(hwnd: HWND) -> DialogResult {
    RESULTS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).remove(&(hwnd.0 as isize)).unwrap_or(DialogResult::Cancel)
}

/// The frame's style for FormBorderStyle (`sizable`, `fixed_single`, `fixed_dialog`, `fixed_tool`,
/// `sizable_tool`, `none`) and the title bar's buttons: (style, extended style).
pub fn frame(border: &str, maximize_box: bool, minimize_box: bool, control_box: bool, top_most: bool) -> (wm::WINDOW_STYLE, wm::WINDOW_EX_STYLE) {
    let mut s = match border {
        "none" => wm::WS_POPUP,
        "fixed_single" | "fixed_tool" | "fixed_dialog" => wm::WS_OVERLAPPED | wm::WS_CAPTION | wm::WS_BORDER,
        _ => wm::WS_OVERLAPPED | wm::WS_CAPTION | wm::WS_THICKFRAME,
    };
    let mut ex = wm::WINDOW_EX_STYLE(0);
    match border {
        "fixed_dialog" => ex |= wm::WS_EX_DLGMODALFRAME,
        "fixed_tool" | "sizable_tool" => ex |= wm::WS_EX_TOOLWINDOW,
        _ => {}
    }
    if border != "none" && control_box {
        s |= wm::WS_SYSMENU;
        if maximize_box && !border.ends_with("tool") {
            s |= wm::WS_MAXIMIZEBOX;
        }
        if minimize_box && !border.ends_with("tool") {
            s |= wm::WS_MINIMIZEBOX;
        }
    }
    if top_most {
        ex |= wm::WS_EX_TOPMOST;
    }
    (s | wm::WS_CLIPCHILDREN, ex)
}

/// A hidden window to own one that should have no taskbar button - ShowInTaskbar = false.
pub fn hidden_owner() -> HWND {
    let class = crate::win::register("PhaiHiddenOwner", Some(owner_proc), None);
    unsafe { wm::CreateWindowExW(wm::WS_EX_TOOLWINDOW, windows::core::PCWSTR(class.as_ptr()), windows::core::PCWSTR::null(), wm::WS_POPUP, 0, 0, 0, 0, None, None, Some(crate::win::instance()), None).unwrap_or_default() }
}

extern "system" fn owner_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> windows::Win32::Foundation::LRESULT {
    crate::win::def(h, msg, w, l)
}

/// Opacity, percent: 100 solid, less shows what is behind.
pub fn set_opacity(hwnd: HWND, percent: i32) {
    unsafe {
        let ex = wm::GetWindowLongW(hwnd, wm::GWL_EXSTYLE) as u32;
        wm::SetWindowLongW(hwnd, wm::GWL_EXSTYLE, (ex | wm::WS_EX_LAYERED.0) as i32);
        let a = (percent.clamp(0, 100) * 255 / 100) as u8;
        let _ = wm::SetLayeredWindowAttributes(hwnd, windows::Win32::Foundation::COLORREF(0), a, wm::LWA_ALPHA);
    }
}

/// Icon: the window's from an `.ico` file's bytes, the large and the small at the screen's sizes.
pub fn set_icon(hwnd: HWND, ico: &[u8]) {
    unsafe {
        for (which, metric_x, metric_y) in [(wm::ICON_BIG, wm::SM_CXICON, wm::SM_CYICON), (wm::ICON_SMALL, wm::SM_CXSMICON, wm::SM_CYSMICON)] {
            let (cx, cy) = (wm::GetSystemMetrics(metric_x), wm::GetSystemMetrics(metric_y));
            let at = wm::LookupIconIdFromDirectoryEx(ico.as_ptr(), true, cx, cy, wm::LR_DEFAULTCOLOR);
            if at <= 0 || at as usize >= ico.len() {
                continue;
            }
            if let Ok(icon) = wm::CreateIconFromResourceEx(&ico[at as usize..], true, 0x0003_0000, cx, cy, wm::LR_DEFAULTCOLOR) {
                wm::SendMessageW(hwnd, wm::WM_SETICON, Some(WPARAM(which as usize)), Some(LPARAM(icon.0 as isize)));
            }
        }
    }
}

/// A click on a button, as the mouse would make it - AcceptButton's Enter, CancelButton's Escape.
pub fn click(button: HWND) {
    unsafe {
        wm::SendMessageW(button, 0x00F5 /* BM_CLICK */, None, None);
    }
}

/// TabIndex: Tab goes through the controls in this order.
pub fn tab_order(controls: &[HWND]) {
    let mut after = wm::HWND_TOP;
    for &h in controls {
        unsafe {
            let _ = wm::SetWindowPos(h, Some(after), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOACTIVATE);
        }
        after = h;
    }
}

/// A window's limits from MinimumSize and MaximumSize - the inside, at 100% scale - into the
/// `WM_GETMINMAXINFO` it is asked.
pub fn min_max(hwnd: HWND, l: LPARAM, min: Option<(i32, i32)>, max: Option<(i32, i32)>, menu: bool) {
    let info = unsafe { &mut *(l.0 as *mut wm::MINMAXINFO) };
    let outer = |(w, h): (i32, i32)| {
        let style = unsafe { wm::GetWindowLongW(hwnd, wm::GWL_STYLE) } as u32;
        let ex = unsafe { wm::GetWindowLongW(hwnd, wm::GWL_EXSTYLE) } as u32;
        let mut r = windows::Win32::Foundation::RECT { left: 0, top: 0, right: crate::win::px(hwnd, w), bottom: crate::win::px(hwnd, h) };
        let _ = unsafe { windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(style), menu, wm::WINDOW_EX_STYLE(ex), crate::win::dpi(hwnd) as u32) };
        (r.right - r.left, r.bottom - r.top)
    };
    if let Some(m) = min {
        let (w, h) = outer(m);
        info.ptMinTrackSize.x = w;
        info.ptMinTrackSize.y = h;
    }
    if let Some(m) = max {
        let (w, h) = outer(m);
        info.ptMaxTrackSize.x = w;
        info.ptMaxTrackSize.y = h;
        info.ptMaxSize.x = w;
        info.ptMaxSize.y = h;
    }
}
