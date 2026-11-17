//! Git history of a file, as Visual Studio's View History, and its blame - who last changed each
//! line, when, and why - read from git itself.

use std::path::Path;

use phai_ui::text::{t, tf};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::gitpane::git;

const ID_LIST: i32 = 12;
const ID_DIFF: i32 = 13;
const ID_CLOSE: i32 = 2;

/// A commit that changed the file.
#[derive(Clone, Debug, PartialEq)]
pub struct Commit {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

/// `git log` read: hash, author, date and message, a tab between each.
pub fn commits_of(log: &str) -> Vec<Commit> {
    log.lines()
        .filter_map(|l| {
            let mut p = l.splitn(4, '\t');
            Some(Commit { hash: p.next()?.to_string(), author: p.next()?.to_string(), date: p.next()?.to_string(), message: p.next().unwrap_or("").to_string() })
        })
        .collect()
}

/// The commits that changed `file`, newest first, through renames.
pub fn history(repo: &Path, file: &Path) -> Vec<Commit> {
    let name = file.to_string_lossy();
    let (_, out) = git(repo, &["log", "--follow", "--date=short", "--format=%h%x09%an%x09%ad%x09%s", "--", &name]);
    commits_of(&out)
}

/// A day from a Unix time, `2026-11-17`.
pub fn day_of(secs: i64) -> String {
    // Days to a civil date (Howard Hinnant's algorithm).
    let z = secs.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

/// `git blame --line-porcelain` read: for each line, from the first, who changed it last, when,
/// and the commit's message - or that it is not committed yet.
pub fn blame_of(porcelain: &str) -> Vec<String> {
    let mut out = Vec::new();
    let (mut hash, mut author, mut time, mut summary) = (String::new(), String::new(), 0i64, String::new());
    for l in porcelain.lines() {
        if l.starts_with('\t') {
            out.push(if hash.chars().all(|c| c == '0') { t("Not committed yet").to_string() } else { format!("{author}, {} · {summary}", day_of(time)) });
        } else if let Some(a) = l.strip_prefix("author ") {
            author = a.to_string();
        } else if let Some(a) = l.strip_prefix("author-time ") {
            time = a.parse().unwrap_or(0);
        } else if let Some(a) = l.strip_prefix("summary ") {
            summary = a.to_string();
        } else if l.len() >= 40 && l[..40].chars().all(|c| c.is_ascii_hexdigit()) {
            hash = l[..40].to_string();
        }
    }
    out
}

/// Each line of `file` blamed, from the first; empty when it is not in a repository.
pub fn blame(repo: &Path, file: &Path) -> Vec<String> {
    let (ok, out) = git(repo, &["blame", "--line-porcelain", "--", &file.to_string_lossy()]);
    if ok {
        blame_of(&out)
    } else {
        vec![]
    }
}

struct Dialog {
    repo: std::path::PathBuf,
    file: std::path::PathBuf,
    list: HWND,
    diff: HWND,
    close: HWND,
    commits: Vec<Commit>,
    done: bool,
}

fn state<'a>(h: HWND) -> Option<&'a mut Dialog> {
    unsafe { win::state::<Dialog>(h) }
}

/// Git > View History of `file`: its commits, and the change each made to it.
pub fn open(owner: HWND, repo: &Path, file: &Path) {
    win::init();
    let commits = history(repo, file);
    // `PHAI_HISTORY_TEST`: printed, not shown - for testing.
    if std::env::var_os("PHAI_HISTORY_TEST").is_some() {
        for c in commits.iter().take(3) {
            println!("commit {} {} {} {}", c.hash, c.author.len(), c.date, c.message);
        }
        if let Some(c) = commits.first() {
            println!("diff lines: {}", show_of(repo, file, &c.hash).lines().count());
        }
        return;
    }
    let class = win::register("PhaiGitHistory", Some(proc_), None);
    let title = win::wide(&tf("History - {0}", &[&file.file_name().unwrap_or_default().to_string_lossy()]));
    unsafe {
        let hwnd = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_THICKFRAME | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let list = win::list(hwnd, ID_LIST, font, &[(t("Commit"), 80), (t("Author"), 150), (t("Date"), 100), (t("Message"), 480)]);
        let diff = win::edit_lines(hwnd, "", ID_DIFF, font);
        wm::SendMessageW(diff, 0x00CF /* EM_SETREADONLY */, Some(WPARAM(1)), None);
        let close = win::button(hwnd, t("Close"), ID_CLOSE, font, true);
        for c in &commits {
            win::list_add(list, &[&c.hash, &c.author, &c.date, &c.message]);
        }
        win::set_state(hwnd, Box::new(Dialog { repo: repo.to_path_buf(), file: file.to_path_buf(), list, diff, close, commits, done: false }));
        if state(hwnd).is_some_and(|d| !d.commits.is_empty()) {
            win::list_select(list, 0);
            show(hwnd, 0);
        } else {
            win::set_text(diff, t("No commit has this file yet."));
        }
        let (w, h) = (win::px(hwnd, 900), win::px(hwnd, 640));
        let mut r = RECT::default();
        let _ = wm::GetWindowRect(owner, &mut r);
        let _ = wm::SetWindowPos(hwnd, None, r.left + (r.right - r.left - w) / 2, r.top + (r.bottom - r.top - h) / 2, w, h, wm::SWP_NOZORDER);
        layout(hwnd);
        win::theme_frame(hwnd);
        win::enable(owner, false);
        win::show_top(hwnd);
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(list));
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
        win::enable(owner, true);
        let _ = wm::SetForegroundWindow(owner);
        let _ = wm::DestroyWindow(hwnd);
    }
}

/// The change commit `hash` made to `file`, as `git show` writes it.
fn show_of(repo: &Path, file: &Path, hash: &str) -> String {
    git(repo, &["show", "--format=commit %H%nAuthor: %an <%ae>%nDate:   %ad%n%n    %B", "--follow", hash, "--", &file.to_string_lossy()]).1
}

fn show(h: HWND, i: usize) {
    let Some(d) = state(h) else { return };
    let Some(c) = d.commits.get(i) else { return };
    let text = show_of(&d.repo, &d.file, &c.hash);
    win::set_text(d.diff, &text.replace('\n', "\r\n"));
}

fn layout(h: HWND) {
    let Some(d) = state(h) else { return };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let px = |v| win::px(h, v);
    let (m, row, bw, gap) = (px(11), px(26), px(96), px(6));
    let list_h = (r.bottom - 2 * m - row - 2 * gap) * 2 / 5;
    win::move_to(d.list, m, m, r.right - 2 * m, list_h);
    let top = m + list_h + gap;
    win::move_to(d.diff, m, top, r.right - 2 * m, r.bottom - top - row - m - gap);
    win::move_to(d.close, r.right - m - bw, r.bottom - m - row, bw, row);
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
            mm.ptMinTrackSize = POINT { x: win::px(h, 520), y: win::px(h, 380) };
            LRESULT(0)
        }
        wm::WM_COMMAND if win::loword(w.0) as i32 == ID_CLOSE => {
            d.done = true;
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.hwndFrom == d.list && hdr.code == cc::LVN_ITEMCHANGED {
                let it = unsafe { &*(l.0 as *const cc::NMLISTVIEW) };
                if it.iItem >= 0 && it.uNewState & 0x2 != 0 && it.uOldState & 0x2 == 0 {
                    show(h, it.iItem as usize);
                }
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
    fn log_and_blame_are_read() {
        let c = commits_of("a1b2c3d\tKirati\t2026-11-17\tFix the grid\n");
        assert_eq!(c[0].message, "Fix the grid");
        assert_eq!(day_of(0), "1970-01-01");
        assert_eq!(day_of(1_763_337_600), "2025-11-17");
        let p = "0123456789abcdef0123456789abcdef01234567 1 1 1\nauthor Ann\nauthor-time 1763337600\nsummary First\nfilename a.rs\n\tfn main() {\n0000000000000000000000000000000000000000 2 2 1\nauthor Not Committed Yet\nauthor-time 1763337600\nsummary Version of a.rs\nfilename a.rs\n\t}\n";
        let b = blame_of(p);
        assert_eq!(b[0], "Ann, 2025-11-17 · First");
        assert_eq!(b[1], t("Not committed yet"));
    }
}
