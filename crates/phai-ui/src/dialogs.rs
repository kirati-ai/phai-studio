//! Small modal windows: a line of text, several lines of text, and "waiting" with a Cancel button. From PhaiDesk's
//! console (`console/dialogs.rs`). Modal the way a dialog is: the owner is switched off and a loop
//! of their own runs until they close.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::WindowsAndMessaging::{self as wm, WINDOW_EX_STYLE, WINDOW_STYLE};

use crate::text::t;
use crate::win::{self, wide};

const ID_EDIT: i32 = 100;
const ID_OK: i32 = 1; // IDOK: Enter
const ID_CANCEL: i32 = 2; // IDCANCEL: Escape
const ID_LABEL: i32 = 101;
const WM_DONE: u32 = wm::WM_APP + 30;
const EM_LIMITTEXT: u32 = 0x00C5;

struct Ask {
    edit: HWND,
    label: HWND,
    ok: HWND,
    cancel: HWND,
    answer: Option<String>,
    done: bool,
    /// For a wait: set by Cancel, read by the work.
    cancelled: Option<Arc<AtomicBool>>,
    font: Gdi::HFONT,
    small: Gdi::HFONT,
    /// The box holds several lines and takes the window's height.
    lines: bool,
}

extern "system" fn proc_(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(s) = (unsafe { win::state::<Ask>(hwnd) }) else { return win::def(hwnd, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            let (cw, ch) = (win::loword(l.0 as usize) as i32, win::hiword(l.0 as usize) as i32);
            let m = win::px(hwnd, 11);
            // Windows' own sizes: buttons 75 x 23, boxes 23 high.
            let bw = win::px(hwnd, 75);
            let bh = win::px(hwnd, 23);
            let has_edit = !s.edit.is_invalid();
            let (label_h, eh) = if s.lines {
                let lh = win::px(hwnd, 20);
                (lh, ch - m * 4 - bh - lh)
            } else {
                let eh = win::px(hwnd, 23);
                (ch - m * 3 - bh - if has_edit { eh + m } else { 0 }, eh)
            };
            win::move_to(s.label, m, m, cw - 2 * m, label_h);
            if has_edit {
                win::move_to(s.edit, m, m + label_h + m, cw - 2 * m, eh);
            }
            win::move_to(s.cancel, cw - m - bw, ch - m - bh, bw, bh);
            if !s.ok.is_invalid() {
                win::move_to(s.ok, cw - m - 2 * bw - win::px(hwnd, 7), ch - m - bh, bw, bh);
            }
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            match win::loword(w.0) as i32 {
                ID_OK if !s.edit.is_invalid() => {
                    s.answer = Some(win::text_of(s.edit));
                    s.done = true;
                }
                ID_CANCEL => {
                    if let Some(c) = &s.cancelled {
                        c.store(true, Ordering::SeqCst);
                    }
                    s.done = true;
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_DONE => {
            s.done = true;
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            if let Some(c) = &s.cancelled {
                c.store(true, Ordering::SeqCst);
            }
            s.done = true;
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            if let Some(s) = unsafe { win::take_state::<Ask>(hwnd) } {
                unsafe {
                    let _ = Gdi::DeleteObject(s.font.into());
                    let _ = Gdi::DeleteObject(s.small.into());
                }
            }
            LRESULT(0)
        }
        _ => win::def(hwnd, msg, w, l),
    }
}

struct Spec<'a> {
    title: &'a str,
    prompt: &'a str,
    initial: Option<&'a str>,
    password: bool,
    ok_label: &'a str,
    lines: bool,
    cancel_label: &'a str,
    size: (i32, i32),
}

fn make(owner: Option<HWND>, spec: &Spec) -> HWND {
    win::init();
    let class = win::register("PhaiAsk", Some(proc_), None);
    let title = wide(spec.title);
    unsafe {
        let hwnd = wm::CreateWindowExW(
            wm::WS_EX_DLGMODALFRAME,
            windows::core::PCWSTR(class.as_ptr()),
            windows::core::PCWSTR(title.as_ptr()),
            wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU,
            wm::CW_USEDEFAULT,
            wm::CW_USEDEFAULT,
            100,
            100,
            owner,
            None,
            Some(win::instance()),
            None,
        )
        .unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let small = win::font(hwnd, 0, false);
        let label = win::child(hwnd, "STATIC", spec.prompt, WINDOW_STYLE(0x80), WINDOW_EX_STYLE(0), ID_LABEL, font);
        win::register_kind(label, win::Kind::Label);
        let edit = match spec.initial {
            Some(initial) => {
                let e = if spec.lines { win::edit_lines(hwnd, &initial.replace('\n', "\r\n"), ID_EDIT, font) } else { win::edit(hwnd, initial, ID_EDIT, font, spec.password) };
                wm::SendMessageW(e, EM_LIMITTEXT, Some(WPARAM(if spec.lines { 64000 } else { 500 })), None);
                e
            }
            None => HWND::default(),
        };
        let ok = if spec.initial.is_some() {
            win::button(hwnd, spec.ok_label, ID_OK, small, true)
        } else {
            HWND::default()
        };
        let cancel = win::button(hwnd, spec.cancel_label, ID_CANCEL, small, false);
        win::set_state(hwnd, Box::new(Ask { edit, label, ok, cancel, answer: None, done: false, cancelled: None, font, small, lines: spec.lines }));
        // Size in design pixels at this monitor's scale, centred on the owner.
        let (w, h) = (win::px(hwnd, spec.size.0), win::px(hwnd, spec.size.1));
        let mut r = Default::default();
        match owner {
            Some(o) => {
                let _ = wm::GetWindowRect(o, &mut r);
            }
            None => {
                let _ = wm::SystemParametersInfoW(wm::SPI_GETWORKAREA, 0, Some(&mut r as *mut _ as *mut _), Default::default());
            }
        }
        let x = r.left + (r.right - r.left - w) / 2;
        let y = r.top + (r.bottom - r.top - h) / 2;
        let _ = wm::SetWindowPos(hwnd, None, x, y, w, h, wm::SWP_NOZORDER);
        win::theme_frame(hwnd);
        hwnd
    }
}

/// Runs `hwnd` modally over `owner` until it says it is done; returns its answer.
fn modal(owner: Option<HWND>, hwnd: HWND, focus: HWND) -> Option<String> {
    unsafe {
        if let Some(o) = owner {
            win::enable(o, false);
        }
        win::show_top(hwnd);
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(if focus.is_invalid() { hwnd } else { focus }));
        let mut msg = wm::MSG::default();
        loop {
            if win::state::<Ask>(hwnd).is_none_or(|s| s.done) {
                break;
            }
            if !wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
                // WM_QUIT belongs to the outer loop; put it back.
                wm::PostQuitMessage(msg.wParam.0 as i32);
                break;
            }
            if !wm::IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
        }
        let answer = win::state::<Ask>(hwnd).and_then(|s| s.answer.take());
        if let Some(o) = owner {
            win::enable(o, true);
            let _ = wm::SetForegroundWindow(o);
        }
        let _ = wm::DestroyWindow(hwnd);
        answer
    }
}

/// Asks for one line of text. `None` when cancelled.
pub fn ask(owner: Option<HWND>, title: &str, prompt: &str, initial: &str, password: bool) -> Option<String> {
    let spec = Spec { title, prompt, initial: Some(initial), password, ok_label: t("OK"), lines: false, cancel_label: t("Cancel"), size: (360, 140) };
    let hwnd = make(owner, &spec);
    let edit = unsafe { win::state::<Ask>(hwnd).map(|s| s.edit).unwrap_or_default() };
    modal(owner, hwnd, edit)
}

/// Asks for several lines of text, `initial` to start from. `None` when cancelled. Lines come
/// back joined by `\n`.
pub fn ask_lines(owner: Option<HWND>, title: &str, prompt: &str, initial: &str) -> Option<String> {
    let spec = Spec { title, prompt, initial: Some(initial), password: false, ok_label: t("OK"), lines: true, cancel_label: t("Cancel"), size: (460, 380) };
    let hwnd = make(owner, &spec);
    let edit = unsafe { win::state::<Ask>(hwnd).map(|s| s.edit).unwrap_or_default() };
    modal(owner, hwnd, edit).map(|s| s.replace("\r\n", "\n"))
}

/// Shows `text` with a Cancel button while `work` runs on another thread. `work` is handed a flag
/// that Cancel sets, and should give up soon after. `None` when cancelled.
pub fn wait_for<T: Send + 'static>(owner: Option<HWND>, title: &str, text: &str, work: impl FnOnce(Arc<AtomicBool>) -> T + Send + 'static) -> Option<T> {
    let spec = Spec { title, prompt: text, initial: None, password: false, ok_label: "", lines: false, cancel_label: t("Cancel"), size: (360, 130) };
    let hwnd = make(owner, &spec);
    let cancelled = Arc::new(AtomicBool::new(false));
    unsafe {
        if let Some(s) = win::state::<Ask>(hwnd) {
            s.cancelled = Some(cancelled.clone());
        }
    }
    let (tx, rx) = mpsc::channel();
    let target = hwnd.0 as isize;
    let c = cancelled.clone();
    std::thread::spawn(move || {
        let _ = tx.send(work(c));
        unsafe {
            let _ = wm::PostMessageW(Some(HWND(target as *mut _)), WM_DONE, WPARAM(0), LPARAM(0));
        }
    });
    modal(owner, hwnd, HWND::default());
    if cancelled.load(Ordering::SeqCst) {
        return None;
    }
    rx.recv().ok()
}

// --- a form of several fields ------------------------------------------------------------------

/// A field of [`ask_fields`]: its label, what it starts with, and what kind of box it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Field<'a> {
    Text(&'a str, &'a str),
    Password(&'a str, &'a str),
    Check(&'a str, bool),
}

struct Fields {
    boxes: Vec<(HWND, HWND, bool)>,
    ok: HWND,
    cancel: HWND,
    answer: Option<Vec<String>>,
    done: bool,
    font: Gdi::HFONT,
    /// The labels' column: as wide as the widest, in pixels.
    label_w: i32,
}

extern "system" fn fields_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(s) = (unsafe { win::state::<Fields>(hwnd) }) else { return win::def(hwnd, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            let cw = win::loword(l.0 as usize) as i32;
            let ch = win::hiword(l.0 as usize) as i32;
            let (m, row, gap) = (win::px(hwnd, 11), win::px(hwnd, 23), win::px(hwnd, 8));
            let label_w = s.label_w;
            let mut y = m;
            for (label, b, check) in &s.boxes {
                if *check {
                    win::move_to(*b, m + label_w, y, cw - 2 * m - label_w, row);
                } else {
                    win::move_to(*label, m, y, label_w - gap, row);
                    win::move_to(*b, m + label_w, y, cw - 2 * m - label_w, row);
                }
                y += row + gap;
            }
            let (bw, bh) = (win::px(hwnd, 75), win::px(hwnd, 23));
            win::move_to(s.cancel, cw - m - bw, ch - m - bh, bw, bh);
            win::move_to(s.ok, cw - m - 2 * bw - win::px(hwnd, 7), ch - m - bh, bw, bh);
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            match win::loword(w.0) as i32 {
                ID_OK => {
                    s.answer = Some(
                        s.boxes
                            .iter()
                            .map(|(_, b, check)| if *check { if win::is_checked(*b) { "1".into() } else { "0".into() } } else { win::text_of(*b) })
                            .collect(),
                    );
                    s.done = true;
                }
                ID_CANCEL => s.done = true,
                _ => {}
            }
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            s.done = true;
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            if let Some(s) = unsafe { win::take_state::<Fields>(hwnd) } {
                unsafe {
                    let _ = Gdi::DeleteObject(s.font.into());
                }
            }
            LRESULT(0)
        }
        _ => win::def(hwnd, msg, w, l),
    }
}

/// Asks for several things at once - a server, a port, a user, a password - each with its label.
/// The answers in order, a check box's as "1" or "0"; `None` when cancelled.
pub fn ask_fields(owner: Option<HWND>, title: &str, fields: &[Field]) -> Option<Vec<String>> {
    win::init();
    let class = win::register("PhaiAskFields", Some(fields_proc), None);
    let t_ = wide(title);
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_DLGMODALFRAME, windows::core::PCWSTR(class.as_ptr()), windows::core::PCWSTR(t_.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU, 0, 0, 100, 100, owner, None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let mut boxes = Vec::new();
        for (k, f) in fields.iter().enumerate() {
            let id = ID_EDIT + 10 + k as i32;
            match f {
                Field::Text(label, initial) | Field::Password(label, initial) => {
                    let l = win::label(hwnd, label, 0, font);
                    let e = win::edit(hwnd, initial, id, font, matches!(f, Field::Password(..)));
                    boxes.push((l, e, false));
                }
                Field::Check(label, on) => {
                    let c = win::checkbox(hwnd, label, id, font, *on);
                    boxes.push((HWND::default(), c, true));
                }
            }
        }
        let ok = win::button(hwnd, t("OK"), ID_OK, font, true);
        let cancel = win::button(hwnd, t("Cancel"), ID_CANCEL, font, false);
        let first = boxes.first().map(|b| b.1).unwrap_or_default();
        // The labels' column fits the longest label (110 to 300 design pixels); the window grows with it.
        let widest = fields.iter().filter_map(|f| match f {
            Field::Text(l, _) | Field::Password(l, _) => Some(win::measure(hwnd, font, l).0),
            Field::Check(..) => None,
        }).max().unwrap_or(0);
        let label_w = (widest + win::px(hwnd, 12)).clamp(win::px(hwnd, 110), win::px(hwnd, 300));
        win::set_state(hwnd, Box::new(Fields { boxes, ok, cancel, answer: None, done: false, font, label_w }));
        let rows = fields.len() as i32;
        let (w, h) = (win::px(hwnd, 290) + label_w, win::px(hwnd, 11 * 2 + rows * 31 + 23 + 50));
        let mut r = Default::default();
        match owner {
            Some(o) => {
                let _ = wm::GetWindowRect(o, &mut r);
            }
            None => {
                let _ = wm::SystemParametersInfoW(wm::SPI_GETWORKAREA, 0, Some(&mut r as *mut _ as *mut _), Default::default());
            }
        }
        let _ = wm::SetWindowPos(hwnd, None, r.left + (r.right - r.left - w) / 2, r.top + (r.bottom - r.top - h) / 2, w, h, wm::SWP_NOZORDER);
        win::theme_frame(hwnd);
        if let Some(o) = owner {
            win::enable(o, false);
        }
        win::show_top(hwnd);
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(first));
        let mut msg = wm::MSG::default();
        loop {
            if win::state::<Fields>(hwnd).is_none_or(|s| s.done) {
                break;
            }
            if !wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
                wm::PostQuitMessage(msg.wParam.0 as i32);
                break;
            }
            if !wm::IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
        }
        let answer = win::state::<Fields>(hwnd).and_then(|s| s.answer.take());
        if let Some(o) = owner {
            win::enable(o, true);
            let _ = wm::SetForegroundWindow(o);
        }
        let _ = wm::DestroyWindow(hwnd);
        answer
    }
}
