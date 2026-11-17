//! Git Changes, as Visual Studio's: the branch, the files changed since the last commit, a box
//! for the commit's message, and Commit All, Pull and Push. A double-click on a file shows how it
//! changed. Everything is Git's own command line, run in the repository the open file is in.

use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use phai_ui::text::{t, tf};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::WindowsAndMessaging as wm;

const ID_FILES: i32 = 11;
const ID_MESSAGE: i32 = 12;
const ID_OUTPUT: i32 = 13;
const ID_COMMIT: i32 = 1;
const ID_CLOSE: i32 = 2;
const ID_PULL: i32 = 20;
const ID_PUSH: i32 = 21;
const ID_REFRESH: i32 = 22;
const ID_STAGE: i32 = 23;
const ID_UNSTAGE: i32 = 24;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// A changed file: Git's two-letter status and its path in the repository.
#[derive(Clone, Debug, PartialEq)]
pub struct Change {
    pub code: String,
    pub path: String,
}

/// What `git status --porcelain` says, read: each changed file.
pub fn changes_of(porcelain: &str) -> Vec<Change> {
    porcelain
        .lines()
        .filter(|l| l.len() > 3)
        .map(|l| {
            let path = l[3..].trim().trim_matches('"').to_string();
            // A rename: `old -> new`, the new name kept.
            let path = path.rsplit(" -> ").next().unwrap_or(&path).to_string();
            Change { code: l[..2].to_string(), path }
        })
        .collect()
}

impl Change {
    /// Staged: its change is in the index, to go in the next commit.
    pub fn staged(&self) -> bool {
        let x = self.code.chars().next().unwrap_or(' ');
        x != ' ' && x != '?'
    }
}

/// A status in words.
pub fn status_words(code: &str) -> &'static str {
    match code.trim() {
        "??" => t("New"),
        c if c.contains('D') => t("Deleted"),
        c if c.contains('A') => t("Added"),
        c if c.contains('R') => t("Renamed"),
        c if c.contains('U') => t("Conflict"),
        _ => t("Modified"),
    }
}

/// Git run in `dir`: whether it succeeded, and what it said (both streams).
pub fn git(dir: &Path, args: &[&str]) -> (bool, String) {
    match Command::new("git").args(args).current_dir(dir).creation_flags(CREATE_NO_WINDOW).output() {
        Ok(o) => {
            let mut s = String::from_utf8_lossy(&o.stdout).to_string();
            let e = String::from_utf8_lossy(&o.stderr);
            if !e.trim().is_empty() {
                if !s.is_empty() && !s.ends_with('\n') {
                    s.push('\n');
                }
                s.push_str(&e);
            }
            (o.status.success(), s)
        }
        Err(e) => (false, format!("git: {e}")),
    }
}

/// The repository a file or folder is in, if any.
pub fn repository_of(p: &Path) -> Option<PathBuf> {
    let dir = if p.is_dir() { p.to_path_buf() } else { p.parent()?.to_path_buf() };
    let dir = std::path::absolute(if dir.as_os_str().is_empty() { Path::new(".") } else { &dir }).ok()?;
    let (ok, out) = git(&dir, &["rev-parse", "--show-toplevel"]);
    ok.then(|| PathBuf::from(out.trim()))
}

struct Pane {
    repo: PathBuf,
    branch: HWND,
    files: HWND,
    message_label: HWND,
    message: HWND,
    output: HWND,
    buttons: Vec<(i32, HWND)>,
    changes: Vec<Change>,
    done: bool,
}

fn state<'a>(h: HWND) -> Option<&'a mut Pane> {
    unsafe { win::state::<Pane>(h) }
}

/// Opens Git Changes for the repository `repo` and runs it until closed.
pub fn open(owner: HWND, repo: PathBuf) {
    win::init();
    let class = win::register("PhaiGitChanges", Some(proc_), None);
    let title = win::wide(&tf("Git Changes - {0}", &[&repo.file_name().unwrap_or_default().to_string_lossy()]));
    unsafe {
        let hwnd = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_THICKFRAME | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let bold = win::font(hwnd, 0, true);
        let branch = win::label(hwnd, "", 0, bold);
        let files = win::list(hwnd, ID_FILES, font, &[(t("Status"), 90), (t("Staged"), 70), (t("File"), 380)]);
        let message_label = win::label(hwnd, t("Message:"), 0, font);
        let message = win::edit_lines(hwnd, "", ID_MESSAGE, font);
        let output = win::edit_lines(hwnd, "", ID_OUTPUT, font);
        wm::SendMessageW(output, 0x00CF /* EM_SETREADONLY */, Some(WPARAM(1)), None);
        let mut buttons = Vec::new();
        for (id, label) in [(ID_COMMIT, t("Commit All")), (ID_STAGE, t("Stage")), (ID_UNSTAGE, t("Unstage")), (ID_PULL, t("Pull")), (ID_PUSH, t("Push")), (ID_REFRESH, t("Refresh")), (ID_CLOSE, t("Close"))] {
            buttons.push((id, win::button(hwnd, label, id, font, id == ID_COMMIT)));
        }
        win::set_state(hwnd, Box::new(Pane { repo, branch, files, message_label, message, output, buttons, changes: vec![], done: false }));
        refresh(hwnd);
        let (w, h) = (win::px(hwnd, 640), win::px(hwnd, 620));
        let mut r = RECT::default();
        let _ = wm::GetWindowRect(owner, &mut r);
        let _ = wm::SetWindowPos(hwnd, None, r.left + (r.right - r.left - w) / 2, r.top + (r.bottom - r.top - h) / 2, w, h, wm::SWP_NOZORDER);
        win::theme_frame(hwnd);
        // `PHAI_GIT_STAGE=n`: the n-th file staged first - for testing.
        if let Some(n) = std::env::var("PHAI_GIT_STAGE").ok().and_then(|n| n.parse::<usize>().ok()) {
            win::list_select(files, n);
            stage(hwnd, true);
        }
        // `PHAI_GIT_TEST=message`: committed at once, what Git then says printed - for testing.
        if let Ok(m) = std::env::var("PHAI_GIT_TEST") {
            win::set_text(message, &m);
            commit(hwnd);
            let s = state(hwnd).map(|s| s.repo.clone()).unwrap_or_default();
            println!("{}", git(&s, &["log", "-1", "--format=%s"]).1.trim());
            println!("left: {}", state(hwnd).map(|s| s.changes.len()).unwrap_or(99));
            if std::env::var_os("PHAI_SHOT").is_none() {
                std::process::exit(0);
            }
        }
        win::enable(owner, false);
        win::show_top(hwnd);
        let mut msg = wm::MSG::default();
        while state(hwnd).is_some_and(|s| !s.done) {
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

/// The branch and the changed files again, from Git.
fn refresh(h: HWND) {
    let Some(s) = state(h) else { return };
    let (_, head) = git(&s.repo, &["status", "-sb", "--porcelain=v1"]);
    let first = head.lines().next().unwrap_or_default().trim_start_matches("## ").to_string();
    win::set_text(s.branch, &tf("Branch: {0}", &[&first]));
    let (ok, out) = git(&s.repo, &["status", "--porcelain=v1", "-uall"]);
    s.changes = if ok { changes_of(&out) } else { vec![] };
    unsafe {
        wm::SendMessageW(s.files, cc::LVM_DELETEALLITEMS, None, None);
    }
    for c in &s.changes {
        win::list_add(s.files, &[status_words(&c.code), if c.staged() { "✓" } else { "" }, &c.path]);
    }
    if !ok {
        say(h, &out);
    }
    let n = s.changes.len();
    let staged = s.changes.iter().filter(|c| c.staged()).count();
    // Some staged: only they go in the commit, as Visual Studio's Commit Staged.
    let label = if staged > 0 { tf("Commit Staged ({0})", &[&staged]) } else if n == 0 { t("Commit All").to_string() } else { tf("Commit All ({0})", &[&n]) };
    if let Some((_, b)) = s.buttons.iter().find(|(i, _)| *i == ID_COMMIT) {
        win::set_text(*b, &label);
    }
}

fn say(h: HWND, text: &str) {
    if let Some(s) = state(h) {
        win::set_text(s.output, &text.trim().replace("\r\n", "\n").replace('\n', "\r\n"));
    }
}

/// Commit All: every change staged and committed with the message typed.
fn commit(h: HWND) {
    let Some(s) = state(h) else { return };
    let message = win::text_of(s.message).replace("\r\n", "\n").trim().to_string();
    if message.is_empty() {
        say(h, t("Write a message first: what this commit does."));
        return;
    }
    if s.changes.is_empty() {
        say(h, t("Nothing to commit: no file has changed."));
        return;
    }
    let repo = s.repo.clone();
    // Nothing staged: everything goes in. Some staged: only those.
    if !s.changes.iter().any(|c| c.staged()) {
        let (ok, out) = git(&repo, &["add", "-A"]);
        if !ok {
            say(h, &out);
            return;
        }
    }
    let (ok, out) = git(&repo, &["commit", "-m", &message]);
    say(h, &out);
    if ok {
        if let Some(s) = state(h) {
            win::set_text(s.message, "");
        }
    }
    refresh(h);
}

/// The files chosen in the list.
fn chosen(h: HWND) -> Vec<Change> {
    let Some(s) = state(h) else { return vec![] };
    let mut out = Vec::new();
    let mut i: isize = -1;
    loop {
        i = unsafe { wm::SendMessageW(s.files, cc::LVM_GETNEXTITEM, Some(WPARAM(i as usize)), Some(LPARAM(2 /* LVNI_SELECTED */))).0 as isize };
        if i < 0 {
            break;
        }
        if let Some(c) = s.changes.get(i as usize) {
            out.push(c.clone());
        }
    }
    out
}

/// Stage or Unstage: the chosen files into the next commit, or out of it.
fn stage(h: HWND, on: bool) {
    let Some(s) = state(h) else { return };
    let repo = s.repo.clone();
    let files = chosen(h);
    if files.is_empty() {
        say(h, t("Choose the files first."));
        return;
    }
    let mut args: Vec<&str> = if on { vec!["add", "--"] } else { vec!["restore", "--staged", "--"] };
    args.extend(files.iter().map(|c| c.path.as_str()));
    let (_, out) = git(&repo, &args);
    say(h, &out);
    refresh(h);
}

/// Pull or Push: Git's words shown, the files again.
fn sync(h: HWND, args: &[&str]) {
    let Some(s) = state(h) else { return };
    let repo = s.repo.clone();
    say(h, &format!("> git {}", args.join(" ")));
    unsafe {
        let _ = Gdi::UpdateWindow(s.output);
        wm::SetCursor(wm::LoadCursorW(None, wm::IDC_WAIT).ok());
    }
    let (_, out) = git(&repo, args);
    say(h, &format!("> git {}\n{}", args.join(" "), out));
    refresh(h);
}

fn layout(h: HWND) {
    let Some(s) = state(h) else { return };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let px = |v| win::px(h, v);
    let (m, row, gap, bw) = (px(11), px(23), px(6), px(96));
    let w = r.right - 2 * m;
    let mut y = m;
    win::move_to(s.branch, m, y, w, row);
    y += row + gap;
    let rest = r.bottom - y - m - (row + gap) * 2 - px(64) - gap;
    let files_h = rest * 55 / 100;
    win::move_to(s.files, m, y, w, files_h);
    y += files_h + gap;
    win::move_to(s.message_label, m, y, w, row);
    y += row;
    win::move_to(s.message, m, y, w, px(64));
    y += px(64) + gap;
    let button = |id: i32| s.buttons.iter().find(|(i, _)| *i == id).map(|(_, b)| *b).unwrap_or_default();
    let mut x = m;
    for id in [ID_COMMIT, ID_STAGE, ID_UNSTAGE, ID_PULL, ID_PUSH, ID_REFRESH] {
        let bw = if id == ID_COMMIT { px(150) } else { px(78) };
        win::move_to(button(id), x, y, bw, row + px(4));
        x += bw + gap;
    }
    win::move_to(button(ID_CLOSE), r.right - m - bw, y, bw, row + px(4));
    y += row + px(4) + gap;
    win::move_to(s.output, m, y, w, r.bottom - y - m);
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(s) = state(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            layout(h);
            LRESULT(0)
        }
        wm::WM_GETMINMAXINFO => {
            let mm = unsafe { &mut *(l.0 as *mut wm::MINMAXINFO) };
            mm.ptMinTrackSize = POINT { x: win::px(h, 480), y: win::px(h, 440) };
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.hwndFrom == s.files && hdr.code == cc::NM_DBLCLK {
                let it = unsafe { &*(l.0 as *const cc::NMITEMACTIVATE) };
                if let Some(c) = (it.iItem >= 0).then(|| s.changes.get(it.iItem as usize).cloned()).flatten() {
                    // How it changed: the diff, or a new file's first lines.
                    let repo = s.repo.clone();
                    let text = if c.code.trim() == "??" {
                        std::fs::read_to_string(repo.join(&c.path)).map(|t| t.lines().take(200).map(|l| format!("+{l}")).collect::<Vec<_>>().join("\n")).unwrap_or_default()
                    } else {
                        git(&repo, &["diff", "HEAD", "--", &c.path]).1
                    };
                    say(h, &text);
                }
                return LRESULT(0);
            }
            win::def(h, msg, w, l)
        }
        wm::WM_COMMAND => {
            match win::loword(w.0) as i32 {
                ID_COMMIT => commit(h),
                ID_STAGE => stage(h, true),
                ID_UNSTAGE => stage(h, false),
                ID_PULL => sync(h, &["pull", "--ff-only"]),
                ID_PUSH => sync(h, &["push"]),
                ID_REFRESH => refresh(h),
                ID_CLOSE => s.done = true,
                _ => return win::def(h, msg, w, l),
            }
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            s.done = true;
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_status_is_read_into_changes() {
        let c = changes_of(" M src/main.rs\n?? notes.txt\nR  old.rs -> new.rs\n D gone.rs\n");
        assert_eq!(c.len(), 4);
        assert_eq!((c[0].code.as_str(), c[0].path.as_str()), (" M", "src/main.rs"));
        assert_eq!(c[2].path, "new.rs", "a rename keeps its new name");
        assert_eq!(status_words(&c[1].code), "New");
        assert_eq!(status_words(&c[3].code), "Deleted");
    }
}
