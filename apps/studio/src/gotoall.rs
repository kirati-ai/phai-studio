//! Go to All (Ctrl+T), as Visual Studio's: one box to type in, and under it the solution's files
//! and code symbols (functions, structs, ...) that match, best first; Enter or a double-click
//! goes there. Files are matched here; symbols come from the language server as they are typed.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use phai_ui::text::t;
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::studio;

const ID_QUERY: i32 = 11;
const ID_LIST: i32 = 12;
const ID_GO: i32 = 1;
const ID_CANCEL: i32 = 2;

/// One thing to go to: what it is called, what kind, where.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub name: String,
    pub kind: String,
    pub path: PathBuf,
    /// Line and UTF-16 column, from 0; none for a file (its top).
    pub at: Option<(u32, u32)>,
}

/// How well `name` matches `query`: none when it does not; higher is better - its start, a word's
/// start, then anywhere; every letter of the query in order counts as a weak match.
pub fn score(name: &str, query: &str) -> Option<i32> {
    let (n, q) = (name.to_lowercase(), query.trim().to_lowercase());
    if q.is_empty() {
        return Some(0);
    }
    if n.starts_with(&q) {
        return Some(300 - n.len() as i32);
    }
    if let Some(i) = n.find(&q) {
        let word_start = i == 0 || !n[..i].chars().last().is_some_and(char::is_alphanumeric);
        return Some(if word_start { 200 } else { 100 } - n.len() as i32);
    }
    let mut it = n.chars();
    q.chars().all(|c| it.any(|x| x == c)).then_some(10 - n.len() as i32)
}

struct Dialog {
    query: HWND,
    list: HWND,
    files: Vec<PathBuf>,
    root: PathBuf,
    symbols: Vec<Item>,
    shown: Vec<Item>,
    chosen: Option<Item>,
    done: bool,
}

static OPEN: Mutex<Option<isize>> = Mutex::new(None);

fn state<'a>(h: HWND) -> Option<&'a mut Dialog> {
    unsafe { win::state::<Dialog>(h) }
}

/// The language server's symbols for what is typed now - shown with the files.
pub fn set_symbols(items: Vec<Item>) {
    let Some(h) = *OPEN.lock().unwrap_or_else(|e| e.into_inner()) else { return };
    let h = HWND(h as *mut _);
    if let Some(d) = state(h) {
        d.symbols = items;
        refill(h);
    }
}

/// Runs Go to All over `owner` until a choice or Escape. `query` given: typed in already.
pub fn run(owner: HWND, files: Vec<PathBuf>, root: PathBuf, query: &str) -> Option<Item> {
    win::init();
    let class = win::register("PhaiGoToAll", Some(proc_), None);
    let title = win::wide(t("Go to All"));
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_TOOLWINDOW, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_THICKFRAME | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let q = win::edit(hwnd, query, ID_QUERY, font, false);
        let list = win::list(hwnd, ID_LIST, font, &[(t("Name"), 260), (t("Kind"), 90), (t("File"), 320)]);
        let go = win::button(hwnd, "", ID_GO, font, true);
        win::show(go, false);
        win::set_state(hwnd, Box::new(Dialog { query: q, list, files, root, symbols: vec![], shown: vec![], chosen: None, done: false }));
        *OPEN.lock().unwrap_or_else(|e| e.into_inner()) = Some(hwnd.0 as isize);
        let mut r = RECT::default();
        let _ = wm::GetWindowRect(owner, &mut r);
        let (w, h) = (win::px(hwnd, 720), win::px(hwnd, 420));
        let _ = wm::SetWindowPos(hwnd, None, r.left + (r.right - r.left - w) / 2, r.top + win::px(hwnd, 90), w, h, wm::SWP_NOZORDER);
        layout(hwnd);
        refill(hwnd);
        ask(hwnd);
        win::theme_frame(hwnd);
        // `PHAI_GOTO_TEST`: the best match taken after the symbols had a moment to come.
        if std::env::var_os("PHAI_GOTO_TEST").is_some() {
            let start = std::time::Instant::now();
            let mut msg = wm::MSG::default();
            let mut asked = std::time::Instant::now();
            while start.elapsed().as_secs() < 40 && state(hwnd).is_some_and(|d| d.symbols.is_empty()) {
                // The server still loading answers with nothing: asked again.
                if asked.elapsed().as_secs() >= 2 {
                    ask(hwnd);
                    asked = std::time::Instant::now();
                }
                while wm::PeekMessageW(&mut msg, None, 0, 0, wm::PM_REMOVE).as_bool() {
                    let _ = wm::TranslateMessage(&msg);
                    wm::DispatchMessageW(&msg);
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            let d = state(hwnd);
            for it in d.map(|d| d.shown.iter().take(5).cloned().collect::<Vec<_>>()).unwrap_or_default() {
                println!("goto: {} ({}) {}", it.name, it.kind, it.path.file_name().unwrap_or_default().to_string_lossy());
            }
            let pick = state(hwnd).and_then(|d| d.shown.first().cloned());
            *OPEN.lock().unwrap_or_else(|e| e.into_inner()) = None;
            let _ = wm::DestroyWindow(hwnd);
            return pick;
        }
        win::enable(owner, false);
        win::show_top(hwnd);
        // While the language server is still loading it answers with nothing: asked again.
        wm::SetTimer(Some(hwnd), 1, 1500, None);
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(q));
        let mut msg = wm::MSG::default();
        while state(hwnd).is_some_and(|d| !d.done) {
            if !wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
                wm::PostQuitMessage(msg.wParam.0 as i32);
                break;
            }
            // Up and Down in the box move the choice in the list.
            if msg.hwnd == q && msg.message == wm::WM_KEYDOWN && matches!(msg.wParam.0, 0x26 | 0x28) {
                step(hwnd, if msg.wParam.0 == 0x26 { -1 } else { 1 });
                continue;
            }
            if !wm::IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
        }
        *OPEN.lock().unwrap_or_else(|e| e.into_inner()) = None;
        let chosen = state(hwnd).and_then(|d| d.chosen.clone());
        win::enable(owner, true);
        let _ = wm::SetForegroundWindow(owner);
        let _ = wm::DestroyWindow(hwnd);
        chosen
    }
}

/// The language server asked for symbols matching what is typed.
fn ask(h: HWND) {
    let Some(d) = state(h) else { return };
    let q = win::text_of(d.query);
    if q.trim().chars().count() >= 2 {
        if let Some(s) = studio() {
            s.lsp_symbols(q.trim());
        }
    }
}

/// The list again: files and symbols that match, best first, at most 200.
fn refill(h: HWND) {
    let Some(d) = state(h) else { return };
    let q = win::text_of(d.query);
    let mut all: Vec<(i32, Item)> = d
        .files
        .iter()
        .filter_map(|p| {
            let name = p.file_name()?.to_string_lossy().to_string();
            score(&name, &q).map(|s| (s, Item { name, kind: t("File").to_string(), path: p.clone(), at: None }))
        })
        .collect();
    // Symbols a little before files of the same score: they are what is usually looked for.
    all.extend(d.symbols.iter().filter_map(|it| score(&it.name, &q).map(|s| (s + 5, it.clone()))));
    all.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.name.cmp(&b.1.name)));
    all.truncate(200);
    d.shown = all.into_iter().map(|(_, it)| it).collect();
    unsafe {
        wm::SendMessageW(d.list, cc::LVM_DELETEALLITEMS, None, None);
    }
    for it in &d.shown {
        let rel = it.path.strip_prefix(&d.root).map(Path::to_path_buf).unwrap_or_else(|_| it.path.clone());
        let place = match it.at {
            Some((l, _)) => format!("{}:{}", rel.display(), l + 1),
            None => rel.display().to_string(),
        };
        win::list_add(d.list, &[&it.name, &it.kind, &place]);
    }
    if !d.shown.is_empty() {
        win::list_select(d.list, 0);
    }
}

fn step(h: HWND, by: i32) {
    let Some(d) = state(h) else { return };
    let n = d.shown.len() as i32;
    if n == 0 {
        return;
    }
    let at = unsafe { wm::SendMessageW(d.list, cc::LVM_GETNEXTITEM, Some(WPARAM(usize::MAX)), Some(LPARAM(2))).0 } as i32;
    win::list_select(d.list, (at + by).clamp(0, n - 1) as usize);
}

fn take(h: HWND) {
    let Some(d) = state(h) else { return };
    let at = unsafe { wm::SendMessageW(d.list, cc::LVM_GETNEXTITEM, Some(WPARAM(usize::MAX)), Some(LPARAM(2))).0 };
    if at >= 0 {
        d.chosen = d.shown.get(at as usize).cloned();
    }
    d.done = true;
}

fn layout(h: HWND) {
    let Some(d) = state(h) else { return };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let m = win::px(h, 8);
    let row = win::px(h, 26);
    win::move_to(d.query, m, m, r.right - 2 * m, row);
    win::move_to(d.list, m, m + row + m, r.right - 2 * m, r.bottom - row - 3 * m);
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(d) = state(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            layout(h);
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            match (win::loword(w.0) as i32, win::hiword(w.0)) {
                (ID_QUERY, 0x0300) => {
                    // Typed: the files at once, the symbols when the server answers.
                    d.symbols.clear();
                    refill(h);
                    ask(h);
                }
                (ID_GO, _) => take(h),
                (ID_CANCEL, _) => d.done = true,
                _ => return win::def(h, msg, w, l),
            }
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.hwndFrom == d.list && hdr.code == cc::NM_DBLCLK {
                take(h);
                return LRESULT(0);
            }
            win::def(h, msg, w, l)
        }
        wm::WM_TIMER => {
            if d.symbols.is_empty() {
                ask(h);
            }
            LRESULT(0)
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
    fn the_start_of_a_name_beats_a_word_inside_it_and_letters_in_order() {
        let s = |n| score(n, "main");
        assert!(s("main.rs") > s("main_window.rs"), "shorter, both at the start");
        assert!(s("main_window.rs") > s("domain_list.rs"));
        assert!(s("my_ui_main.rs") > s("domain_list.rs"), "a word's start beats the middle");
        assert!(score("MainWindow", "mwin").is_some(), "letters in order");
        assert_eq!(score("text.rs", "main"), None);
    }
}
