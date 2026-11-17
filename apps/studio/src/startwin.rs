//! The start window, as Visual Studio's: shown when Phai Studio opens with nothing - the forms and
//! files opened lately on the left, and on the right the ways to begin: open one, make a new
//! project, or go on without either.

use std::path::PathBuf;

use phai_ui::text::t;
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::WindowsAndMessaging as wm;

const ID_RECENT: i32 = 11;
const ID_SHOW: i32 = 12;
const ID_OPEN_FILE: i32 = 20;
const ID_NEW_PROJECT: i32 = 21;
const ID_CONTINUE: i32 = 2;
const ID_OPEN_RECENT: i32 = 1;

/// What the person chose.
#[derive(Clone, Debug, PartialEq)]
pub enum Choice {
    Recent(PathBuf),
    OpenFile,
    NewProject,
    Continue,
}

struct Start {
    title: HWND,
    recent_label: HWND,
    recent: HWND,
    begin_label: HWND,
    buttons: Vec<(i32, HWND)>,
    show: HWND,
    paths: Vec<PathBuf>,
    choice: Option<Choice>,
}

fn state<'a>(h: HWND) -> Option<&'a mut Start> {
    unsafe { win::state::<Start>(h) }
}

/// Shows the start window over `owner` until a choice is made; the choice, and whether it should
/// show again next time.
pub fn run(owner: HWND, paths: Vec<PathBuf>, show_again: bool) -> (Choice, bool) {
    win::init();
    let class = win::register("PhaiStartWindow", Some(proc_), None);
    let caption = win::wide("Phai Studio");
    unsafe {
        let hwnd = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR(caption.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let big = win::font(hwnd, 18, false);
        let bold = win::font(hwnd, 0, true);
        let title = win::label(hwnd, t("What would you like to do?"), 0, big);
        let recent_label = win::label(hwnd, t("Open recent"), 0, bold);
        let recent = win::list(hwnd, ID_RECENT, font, &[(t("Name"), 190), (t("Folder"), 330)]);
        for p in &paths {
            let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
            let folder = p.parent().map(|d| d.display().to_string()).unwrap_or_default();
            win::list_add(recent, &[&name, &folder]);
        }
        if paths.is_empty() {
            win::list_add(recent, &[t("Nothing opened yet"), ""]);
        } else {
            win::list_select(recent, 0);
        }
        let begin_label = win::label(hwnd, t("Get started"), 0, bold);
        let mut buttons = Vec::new();
        for (id, label) in [(ID_OPEN_FILE, t("Open a form or code file...")), (ID_NEW_PROJECT, t("Create a new project...")), (ID_CONTINUE, t("Continue without code →"))] {
            buttons.push((id, win::button(hwnd, label, id, font, false)));
        }
        // Enter on the list opens the file chosen in it.
        let open = win::button(hwnd, "", ID_OPEN_RECENT, font, true);
        win::show(open, false);
        let show = win::checkbox(hwnd, t("Show this window when Phai Studio starts"), ID_SHOW, font, show_again);
        win::set_state(hwnd, Box::new(Start { title, recent_label, recent, begin_label, buttons, show, paths, choice: None }));
        let mut r = RECT { left: 0, top: 0, right: win::px(hwnd, 860), bottom: win::px(hwnd, 460) };
        let _ = wm::AdjustWindowRect(&mut r, wm::WS_CAPTION | wm::WS_SYSMENU, false);
        let (w, h) = (r.right - r.left, r.bottom - r.top);
        let mut o = RECT::default();
        let _ = wm::GetWindowRect(owner, &mut o);
        let _ = wm::SetWindowPos(hwnd, None, o.left + (o.right - o.left - w) / 2, o.top + (o.bottom - o.top - h) / 2, w, h, wm::SWP_NOZORDER);
        layout(hwnd);
        win::theme_frame(hwnd);
        win::enable(owner, false);
        win::show_top(hwnd);
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(recent));
        let mut msg = wm::MSG::default();
        while state(hwnd).is_some_and(|s| s.choice.is_none()) {
            if !wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
                wm::PostQuitMessage(msg.wParam.0 as i32);
                break;
            }
            if !wm::IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
        }
        let again = state(hwnd).is_none_or(|s| win::is_checked(s.show));
        let choice = state(hwnd).and_then(|s| s.choice.clone()).unwrap_or(Choice::Continue);
        win::enable(owner, true);
        let _ = wm::SetForegroundWindow(owner);
        let _ = wm::DestroyWindow(hwnd);
        (choice, again)
    }
}

fn layout(h: HWND) {
    let Some(s) = state(h) else { return };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let px = |v| win::px(h, v);
    let (m, row) = (px(24), px(24));
    win::move_to(s.title, m, m, r.right - 2 * m, px(40));
    let top = m + px(56);
    let left_w = (r.right - 3 * m) * 62 / 100;
    win::move_to(s.recent_label, m, top, left_w, row);
    win::move_to(s.recent, m, top + row + px(6), left_w, r.bottom - top - row - px(6) - m - row - px(8));
    // Name and Folder share the list's width: no sideways scrolling.
    unsafe {
        let name_w = px(190);
        let rest = left_w - name_w - wm::GetSystemMetrics(wm::SM_CXVSCROLL) - px(4);
        wm::SendMessageW(s.recent, cc::LVM_SETCOLUMNWIDTH, Some(WPARAM(0)), Some(LPARAM(name_w as isize)));
        wm::SendMessageW(s.recent, cc::LVM_SETCOLUMNWIDTH, Some(WPARAM(1)), Some(LPARAM(rest.max(px(80)) as isize)));
    }
    let x = m * 2 + left_w;
    let bw = r.right - x - m;
    win::move_to(s.begin_label, x, top, bw, row);
    let mut y = top + row + px(6);
    for (_, b) in &s.buttons {
        win::move_to(*b, x, y, bw, px(52));
        y += px(52) + px(10);
    }
    win::move_to(s.show, m, r.bottom - m - row + px(8), r.right - 2 * m, row);
}

fn chosen_recent(s: &Start) -> Option<PathBuf> {
    let i = unsafe { wm::SendMessageW(s.recent, cc::LVM_GETNEXTITEM, Some(WPARAM(usize::MAX)), Some(LPARAM(2))).0 };
    (i >= 0).then(|| s.paths.get(i as usize).cloned()).flatten()
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(s) = state(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.hwndFrom == s.recent && hdr.code == cc::NM_DBLCLK {
                if let Some(p) = chosen_recent(s) {
                    s.choice = Some(Choice::Recent(p));
                }
                return LRESULT(0);
            }
            win::def(h, msg, w, l)
        }
        wm::WM_COMMAND => {
            match win::loword(w.0) as i32 {
                ID_OPEN_RECENT => {
                    if let Some(p) = chosen_recent(s) {
                        s.choice = Some(Choice::Recent(p));
                    }
                }
                ID_OPEN_FILE => s.choice = Some(Choice::OpenFile),
                ID_NEW_PROJECT => s.choice = Some(Choice::NewProject),
                ID_CONTINUE => s.choice = Some(Choice::Continue),
                _ => return win::def(h, msg, w, l),
            }
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            s.choice = Some(Choice::Continue);
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}
