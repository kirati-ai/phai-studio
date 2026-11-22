//! What a form is beyond its controls, as WinForms has it: its frame (FormBorderStyle, the title
//! bar's buttons, TopMost, ShowInTaskbar, Opacity), its Icon, the buttons Enter and Escape click
//! (AcceptButton, CancelButton), the order Tab goes through, and the answer a dialog gives
//! (DialogResult).

use std::collections::HashMap;
use std::sync::Mutex;

use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
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

thread_local! {
    /// The windows whose program objects are wanted back when they close, and those given back.
    static WANTED: std::cell::RefCell<std::collections::HashSet<isize>> = Default::default();
    static KEPT: std::cell::RefCell<HashMap<isize, Box<dyn std::any::Any>>> = Default::default();
}

/// `hwnd`'s program object is to be given back when it closes ([`keep`], [`take_kept`]) - what
/// `show_dialog_for` asks, so the program reads what was typed, as a WinForms form's fields after
/// `ShowDialog()`.
pub fn want_back(hwnd: HWND) {
    WANTED.with(|w| w.borrow_mut().insert(hwnd.0 as isize));
}

/// A closing window's program object, kept when [`want_back`] asked for it (and dropped now
/// otherwise).
pub fn keep(hwnd: HWND, events: Box<dyn std::any::Any>) {
    if WANTED.with(|w| w.borrow_mut().remove(&(hwnd.0 as isize))) {
        KEPT.with(|k| k.borrow_mut().insert(hwnd.0 as isize, events));
    }
}

/// The program object [`keep`] kept for `hwnd`.
pub fn take_kept<E: 'static>(hwnd: HWND) -> Option<E> {
    KEPT.with(|k| k.borrow_mut().remove(&(hwnd.0 as isize))).and_then(|b| b.downcast::<E>().ok()).map(|b| *b)
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

/// Whether menu item `id` of window `h`'s menu bar is ticked.
pub fn menu_checked(h: HWND, id: i32) -> bool {
    unsafe { wm::GetMenuState(wm::GetMenu(h), id as u32, wm::MF_BYCOMMAND) & wm::MF_CHECKED.0 != 0 }
}

/// Item `id` ticked or not, in the menu bar and on the toolbar alike - Checked.
pub fn set_item_checked(h: HWND, toolbar: Option<HWND>, id: i32, on: bool) {
    unsafe {
        let _ = wm::CheckMenuItem(wm::GetMenu(h), id as u32, (wm::MF_BYCOMMAND | if on { wm::MF_CHECKED } else { wm::MF_UNCHECKED }).0);
        if let Some(t) = toolbar {
            wm::SendMessageW(t, 0x0402 /* TB_CHECKBUTTON */, Some(WPARAM(id as usize)), Some(LPARAM(on as isize)));
        }
    }
}

/// Whether item `id` is ticked: on the toolbar when it is there, in the menu bar otherwise.
pub fn item_checked(h: HWND, toolbar: Option<HWND>, id: i32) -> bool {
    if let Some(t) = toolbar {
        let has = unsafe { wm::SendMessageW(t, 0x0419 /* TB_COMMANDTOINDEX */, Some(WPARAM(id as usize)), None).0 } >= 0;
        if has {
            return unsafe { wm::SendMessageW(t, 0x040A /* TB_ISBUTTONCHECKED */, Some(WPARAM(id as usize)), None).0 } != 0;
        }
    }
    menu_checked(h, id)
}

/// Item `id` usable or greyed, in the menu bar and on the toolbar alike - Enabled.
pub fn set_item_enabled(h: HWND, toolbar: Option<HWND>, id: i32, on: bool) {
    unsafe {
        let _ = wm::EnableMenuItem(wm::GetMenu(h), id as u32, wm::MF_BYCOMMAND | if on { wm::MF_ENABLED } else { wm::MF_GRAYED });
        let _ = wm::DrawMenuBar(h);
        if let Some(t) = toolbar {
            wm::SendMessageW(t, 0x0401 /* TB_ENABLEBUTTON */, Some(WPARAM(id as usize)), Some(LPARAM(on as isize)));
        }
    }
}

/// A CheckOnClick item chosen: its tick turned over, menu and toolbar together; the tick now.
pub fn toggle_item(h: HWND, toolbar: Option<HWND>, id: i32, from_toolbar: bool) -> bool {
    // A toolbar button that stays down has turned itself over already.
    let on = if from_toolbar { item_checked(h, toolbar, id) } else { !menu_checked(h, id) };
    set_item_checked(h, toolbar, id, on);
    on
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

/// BackColor and ForeColor of this window alone: CSS colours, words that are not one left out.
pub fn set_colors(hwnd: HWND, background: Option<&str>, color: Option<&str>) {
    crate::win::set_window_colors(hwnd, background.and_then(crate::theme::parse_color), color.and_then(crate::theme::parse_color));
}

struct Background {
    image: crate::picture::Image,
    /// BackgroundImageLayout: `tile`, `none`, `center`, `stretch`, `zoom`.
    layout: String,
}

// A window's background picture lives as long as the window; one per window.
unsafe impl Send for Background {}

static BACKGROUNDS: Mutex<Option<HashMap<isize, Background>>> = Mutex::new(None);

/// BackgroundImage: a picture file under the window's controls - `tile`d (WinForms' own way),
/// at its size in the corner (`none`), in the `center`, `stretch`ed, or as large as fits (`zoom`).
/// A path relative to the program's folder, or where it runs, as a picture control's.
pub fn set_background_image(hwnd: HWND, path: &str, layout: &str) {
    let mut map = BACKGROUNDS.lock().unwrap_or_else(|e| e.into_inner());
    let m = map.get_or_insert_with(HashMap::new);
    match crate::picture::resolve(path).and_then(|p| crate::picture::load(&p)) {
        Some(image) => {
            m.insert(hwnd.0 as isize, Background { image, layout: layout.to_string() });
        }
        None => {
            m.remove(&(hwnd.0 as isize));
        }
    }
    unsafe {
        let _ = windows::Win32::Graphics::Gdi::InvalidateRect(Some(hwnd), None, true);
    }
}

/// Whether the window has a background picture.
pub fn has_background_image(hwnd: HWND) -> bool {
    BACKGROUNDS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().is_some_and(|m| m.contains_key(&(hwnd.0 as isize)))
}

/// Draws the window's background picture, if it has one, over its colour - from `WM_ERASEBKGND`.
pub fn paint_background(hwnd: HWND, dc: windows::Win32::Graphics::Gdi::HDC, r: &RECT) {
    use windows::Win32::Graphics::Gdi;
    let map = BACKGROUNDS.lock().unwrap_or_else(|e| e.into_inner());
    let Some(bg) = map.as_ref().and_then(|m| m.get(&(hwnd.0 as isize))) else { return };
    let (iw, ih) = (bg.image.w, bg.image.h);
    if iw <= 0 || ih <= 0 {
        return;
    }
    let mut places: Vec<RECT> = Vec::new();
    match bg.layout.as_str() {
        "none" => places.push(RECT { left: 0, top: 0, right: iw, bottom: ih }),
        "center" => {
            let (x, y) = ((r.right - iw) / 2, (r.bottom - ih) / 2);
            places.push(RECT { left: x, top: y, right: x + iw, bottom: y + ih });
        }
        "stretch" => places.push(*r),
        "zoom" => places.push(crate::picture::fit(*r, iw, ih)),
        _ => {
            let mut y = 0;
            while y < r.bottom {
                let mut x = 0;
                while x < r.right {
                    places.push(RECT { left: x, top: y, right: x + iw, bottom: y + ih });
                    x += iw;
                }
                y += ih;
            }
        }
    }
    unsafe {
        let src = Gdi::CreateCompatibleDC(Some(dc));
        let old = Gdi::SelectObject(src, bg.image.bmp.into());
        let blend = Gdi::BLENDFUNCTION { BlendOp: 0, BlendFlags: 0, SourceConstantAlpha: 255, AlphaFormat: 1 };
        let _ = Gdi::SetStretchBltMode(dc, Gdi::HALFTONE);
        for to in places {
            let _ = Gdi::AlphaBlend(dc, to.left, to.top, to.right - to.left, to.bottom - to.top, src, 0, 0, iw, ih, blend);
        }
        Gdi::SelectObject(src, old);
        let _ = Gdi::DeleteDC(src);
    }
}

static SCROLLS: Mutex<Option<HashMap<isize, (i32, i32)>>> = Mutex::new(None);

/// AutoScroll: smaller than designed, the window shows scroll bars and its controls move with
/// them. The window's procedure hands its scroll messages to [`scroll`].
pub fn set_auto_scroll(hwnd: HWND, on: bool) {
    let mut m = SCROLLS.lock().unwrap_or_else(|e| e.into_inner());
    let map = m.get_or_insert_with(HashMap::new);
    if on {
        map.entry(hwnd.0 as isize).or_insert((0, 0));
    } else {
        map.remove(&(hwnd.0 as isize));
    }
}

/// Whether the window scrolls.
pub fn scrolls(hwnd: HWND) -> bool {
    SCROLLS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().is_some_and(|m| m.contains_key(&(hwnd.0 as isize)))
}

fn offset(hwnd: HWND) -> (i32, i32) {
    SCROLLS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|m| m.get(&(hwnd.0 as isize)).copied()).unwrap_or((0, 0))
}

fn set_offset(hwnd: HWND, o: (i32, i32)) {
    if let Some(m) = SCROLLS.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
        m.insert(hwnd.0 as isize, o);
    }
}

/// The scroll bars for content `size` (pixels) in the window as it is now; where it is scrolled to.
pub(crate) fn update_scrollbars(hwnd: HWND, size: (i32, i32)) -> (i32, i32) {
    let mut c = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(hwnd, &mut c);
    }
    let (ox, oy) = offset(hwnd);
    let o = ((ox).clamp(0, (size.0 - c.right).max(0)), (oy).clamp(0, (size.1 - c.bottom).max(0)));
    set_offset(hwnd, o);
    for (bar, max, page, pos) in [(wm::SB_HORZ, size.0, c.right, o.0), (wm::SB_VERT, size.1, c.bottom, o.1)] {
        let si = wm::SCROLLINFO { cbSize: std::mem::size_of::<wm::SCROLLINFO>() as u32, fMask: wm::SIF_ALL, nMin: 0, nMax: (max - 1).max(0), nPage: page.max(0) as u32, nPos: pos, nTrackPos: 0 };
        unsafe {
            windows::Win32::UI::Controls::SetScrollInfo(hwnd, bar, &si, true);
        }
    }
    o
}

/// A scroll message to a window that scrolls: its new place kept; `true` when it was one, and
/// the window's layout should be applied again.
pub fn scroll(hwnd: HWND, msg: u32, w: WPARAM) -> bool {
    if !scrolls(hwnd) {
        return false;
    }
    let (bar, line) = match msg {
        wm::WM_VSCROLL | wm::WM_MOUSEWHEEL => (wm::SB_VERT, crate::win::px(hwnd, 20)),
        wm::WM_HSCROLL => (wm::SB_HORZ, crate::win::px(hwnd, 20)),
        _ => return false,
    };
    let mut si = wm::SCROLLINFO { cbSize: std::mem::size_of::<wm::SCROLLINFO>() as u32, fMask: wm::SIF_ALL, ..Default::default() };
    unsafe {
        let _ = wm::GetScrollInfo(hwnd, bar, &mut si);
    }
    let pos = if msg == wm::WM_MOUSEWHEEL {
        si.nPos - ((w.0 >> 16) as i16 as i32) * line * 3 / 120
    } else {
        match wm::SCROLLBAR_COMMAND((w.0 & 0xFFFF) as i32) {
            wm::SB_LINEUP => si.nPos - line,
            wm::SB_LINEDOWN => si.nPos + line,
            wm::SB_PAGEUP => si.nPos - si.nPage as i32,
            wm::SB_PAGEDOWN => si.nPos + si.nPage as i32,
            wm::SB_THUMBTRACK | wm::SB_THUMBPOSITION => si.nTrackPos,
            wm::SB_TOP => 0,
            wm::SB_BOTTOM => si.nMax,
            _ => si.nPos,
        }
    };
    let (ox, oy) = offset(hwnd);
    set_offset(hwnd, if bar == wm::SB_VERT { (ox, pos.max(0)) } else { (pos.max(0), oy) });
    true
}
