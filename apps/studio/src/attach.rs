//! Attach to Process, as Visual Studio's: the programs running on this computer - name, id, file -
//! filtered as typed, the project's own first; the one chosen is debugged where it is.

use std::path::{Path, PathBuf};

use phai_ui::text::{t, tf};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::System::ProcessStatus::EnumProcesses;
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::WindowsAndMessaging as wm;

const ID_FILTER: i32 = 11;
const ID_LIST: i32 = 12;
const ID_ATTACH: i32 = 1;
const ID_CANCEL: i32 = 2;
const ID_REFRESH: i32 = 20;

/// A running program: its id and its file.
#[derive(Clone, Debug, PartialEq)]
pub struct Process {
    pub pid: u32,
    pub exe: PathBuf,
}

/// Every program running that may be looked at (others' and Windows' own are left out), those
/// built in `project` first, then by name.
pub fn processes(project: Option<&Path>) -> Vec<Process> {
    let mut ids = vec![0u32; 4096];
    let mut got = 0u32;
    if unsafe { EnumProcesses(ids.as_mut_ptr(), (ids.len() * 4) as u32, &mut got) }.is_err() {
        return vec![];
    }
    ids.truncate(got as usize / 4);
    let me = std::process::id();
    let mut list: Vec<Process> = ids.into_iter().filter(|p| *p != 0 && *p != me).filter_map(|pid| phai_debug::process_image(pid).map(|exe| Process { pid, exe })).collect();
    let name = |p: &Process| p.exe.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
    list.sort_by_key(|p| (!project.is_some_and(|d| p.exe.starts_with(d)), name(p), p.pid));
    list
}

struct Dialog {
    project: Option<PathBuf>,
    filter: HWND,
    list: HWND,
    label: HWND,
    say: HWND,
    buttons: Vec<(i32, HWND)>,
    all: Vec<Process>,
    shown: Vec<Process>,
    chosen: Option<Process>,
    done: bool,
}

fn state<'a>(h: HWND) -> Option<&'a mut Dialog> {
    unsafe { win::state::<Dialog>(h) }
}

/// Asks which running program to debug; its id and file.
pub fn choose(owner: HWND, project: Option<&Path>) -> Option<(u32, PathBuf)> {
    win::init();
    let class = win::register("PhaiAttach", Some(proc_), None);
    let title = win::wide(t("Attach to Process"));
    unsafe {
        let hwnd = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_THICKFRAME | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let label = win::label(hwnd, t("Filter processes:"), 0, font);
        let filter = win::edit(hwnd, "", ID_FILTER, font, false);
        let list = win::list(hwnd, ID_LIST, font, &[(t("Process"), 200), (t("ID"), 70), (t("Path"), 460)]);
        let say = win::label(hwnd, "", 0, font);
        win::set_css(say, "muted", "");
        let mut buttons = Vec::new();
        for (id, text) in [(ID_ATTACH, t("Attach")), (ID_CANCEL, t("Cancel")), (ID_REFRESH, t("Refresh"))] {
            buttons.push((id, win::button(hwnd, text, id, font, id == ID_ATTACH)));
        }
        win::set_state(hwnd, Box::new(Dialog { project: project.map(Path::to_path_buf), filter, list, label, say, buttons, all: vec![], shown: vec![], chosen: None, done: false }));
        refresh(hwnd);
        let (w, h) = (win::px(hwnd, 760), win::px(hwnd, 520));
        let mut r = RECT::default();
        let _ = wm::GetWindowRect(owner, &mut r);
        let _ = wm::SetWindowPos(hwnd, None, r.left + (r.right - r.left - w) / 2, r.top + (r.bottom - r.top - h) / 2, w, h, wm::SWP_NOZORDER);
        layout(hwnd);
        win::theme_frame(hwnd);
        win::enable(owner, false);
        win::show_top(hwnd);
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(filter));
        let mut msg = wm::MSG::default();
        while state(hwnd).is_some_and(|d| !d.done) {
            if !wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
                wm::PostQuitMessage(msg.wParam.0 as i32);
                break;
            }
            if !wm::IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
        }
        let chosen = state(hwnd).and_then(|d| d.chosen.take());
        win::enable(owner, true);
        let _ = wm::SetForegroundWindow(owner);
        let _ = wm::DestroyWindow(hwnd);
        chosen.map(|p| (p.pid, p.exe))
    }
}

/// The processes looked for again.
fn refresh(h: HWND) {
    let Some(d) = state(h) else { return };
    d.all = processes(d.project.as_deref());
    show(h);
}

/// The list as filtered: a process whose name, id or file has what is typed.
fn show(h: HWND) {
    let Some(d) = state(h) else { return };
    let f = win::text_of(d.filter).trim().to_lowercase();
    d.shown = d.all.iter().filter(|p| f.is_empty() || p.exe.to_string_lossy().to_lowercase().contains(&f) || p.pid.to_string() == f).cloned().collect();
    unsafe {
        wm::SendMessageW(d.list, cc::LVM_DELETEALLITEMS, None, None);
    }
    for p in &d.shown {
        win::list_add(d.list, &[&p.exe.file_name().unwrap_or_default().to_string_lossy(), &p.pid.to_string(), &p.exe.to_string_lossy()]);
    }
    if !d.shown.is_empty() {
        win::list_select(d.list, 0);
    }
    win::set_text(d.say, &tf("{0} processes", &[&d.shown.len()]));
}

fn attach(h: HWND) {
    let Some(d) = state(h) else { return };
    let i = unsafe { wm::SendMessageW(d.list, cc::LVM_GETNEXTITEM, Some(WPARAM(usize::MAX)), Some(LPARAM(2))).0 };
    if let Some(p) = usize::try_from(i).ok().and_then(|i| d.shown.get(i)) {
        d.chosen = Some(p.clone());
        d.done = true;
    }
}

fn layout(h: HWND) {
    let Some(d) = state(h) else { return };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let px = |v| win::px(h, v);
    let (m, row, bw, gap) = (px(11), px(26), px(96), px(6));
    let button = |id: i32| d.buttons.iter().find(|(i, _)| *i == id).map(|(_, b)| *b).unwrap_or_default();
    let lw = px(120);
    win::move_to(d.label, m, m, lw, row);
    win::move_to(d.filter, m + lw, m, r.right - 2 * m - lw - bw - gap, row);
    win::move_to(button(ID_REFRESH), r.right - m - bw, m, bw, row);
    let top = m + row + gap;
    win::move_to(d.list, m, top, r.right - 2 * m, r.bottom - top - row - m - gap);
    let by = r.bottom - m - row;
    win::move_to(d.say, m, by, r.right - 2 * bw - 3 * m, row);
    win::move_to(button(ID_ATTACH), r.right - 2 * bw - m - gap, by, bw, row);
    win::move_to(button(ID_CANCEL), r.right - bw - m, by, bw, row);
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(d) = state(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            layout(h);
            LRESULT(0)
        }
        wm::WM_GETMINMAXINFO => {
            let mm = unsafe { &mut *(l.0 as *mut wm::MINMAXINFO) };
            mm.ptMinTrackSize = POINT { x: win::px(h, 480), y: win::px(h, 320) };
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            let id = win::loword(w.0) as i32;
            match id {
                ID_ATTACH => attach(h),
                ID_CANCEL => d.done = true,
                ID_REFRESH => refresh(h),
                ID_FILTER if win::hiword(w.0) as u32 == wm::EN_CHANGE => show(h),
                _ => return win::def(h, msg, w, l),
            }
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.hwndFrom == d.list && hdr.code == cc::NM_DBLCLK {
                attach(h);
                return LRESULT(0);
            }
            win::def(h, msg, w, l)
        }
        wm::WM_CLOSE => {
            d.done = true;
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn running_programs_are_listed() {
        // A child of ours, from a folder taken as the project: it comes first.
        let exe = std::env::current_exe().unwrap();
        let mut child = std::process::Command::new("cmd.exe").args(["/c", "ping -n 5 127.0.0.1 >nul"]).spawn().unwrap();
        let list = processes(exe.parent());
        let found = list.iter().any(|p| p.pid == child.id());
        let _ = child.kill();
        let _ = child.wait();
        assert!(found, "a program just started is in the list");
        assert!(list.iter().all(|p| p.pid != std::process::id()), "Phai Studio leaves itself out");
    }
}
