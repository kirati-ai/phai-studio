//! The start window, as Visual Studio's: shown when Phai Studio opens with nothing - the forms and
//! files opened lately on the left, searched, the pinned ones first; on the right the ways to
//! begin: clone a repository, open a file, make a new project, or go on without either. Along
//! the bottom, a tip.

use std::path::PathBuf;

use phai_ui::text::t;
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::WindowsAndMessaging as wm;

const ID_RECENT: i32 = 11;
const ID_SHOW: i32 = 12;
const ID_FIND: i32 = 13;
const ID_NEXT_TIP: i32 = 14;
const ID_CLONE: i32 = 19;
const ID_OPEN_FILE: i32 = 20;
const ID_NEW_PROJECT: i32 = 21;
const ID_CONTINUE: i32 = 2;
const ID_OPEN_RECENT: i32 = 1;

/// What the person chose.
#[derive(Clone, Debug, PartialEq)]
pub enum Choice {
    Recent(PathBuf),
    Clone,
    OpenFile,
    NewProject,
    Continue,
}

/// Tips, as Visual Studio's and Android Studio's: one along the bottom, the next on a click.
pub const TIPS: [&str; 12] = [
    "Ctrl+Q searches every command, Options page and file by name.",
    "Ctrl+T goes to any file or function of the solution.",
    "Shift+F4 shows the form on a phone and a tablet - turned, or side by side.",
    "Build > Run on Phone shows a QR code: the phone's camera opens the program, and it reloads after each build.",
    "A right-drag from a control in the designer offers its events - the chosen one written in the code.",
    "Format > Infer Anchors guesses how each control should follow the window as it is resized.",
    "One click on a file in Solution Explorer opens it in a preview tab; a double-click keeps it.",
    "Ctrl+Alt+click puts one more caret in the editor; Alt+drag makes a box of them.",
    "The green ▶ beside a #[test] runs that one test.",
    "View > Window Layouts keeps the panes as they are - Ctrl+Alt+1 brings them back.",
    "Tools > Keyboard Shortcuts gives any command a key of your own.",
    "A form's web.css goes into its page on the web after the theme - CSS for phones alone.",
];

struct Start {
    title: HWND,
    recent_label: HWND,
    find: HWND,
    recent: HWND,
    begin_label: HWND,
    buttons: Vec<(i32, HWND)>,
    show: HWND,
    tip: HWND,
    next_tip: HWND,
    tip_at: usize,
    /// Every recent file; `shown` those the search keeps, the pinned first.
    all: Vec<PathBuf>,
    shown: Vec<PathBuf>,
    pinned: Vec<PathBuf>,
    choice: Option<Choice>,
}

fn state<'a>(h: HWND) -> Option<&'a mut Start> {
    unsafe { win::state::<Start>(h) }
}

/// The recent files pinned, as the settings keep them.
pub fn pinned() -> Vec<PathBuf> {
    crate::setting_pub("pinned_recent").unwrap_or_default().split('|').filter(|s| !s.is_empty()).map(PathBuf::from).collect()
}

fn save_pinned(list: &[PathBuf]) {
    crate::remember_setting_pub("pinned_recent", &list.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("|"));
}

/// `all` as the list shows it for `query`: the pinned ones first (in the order pinned), then the
/// rest as they came; each kept when its name or folder has every word of `query`.
pub fn arrange(all: &[PathBuf], pinned: &[PathBuf], query: &str) -> Vec<PathBuf> {
    let q = query.trim().to_lowercase();
    let keep = |p: &PathBuf| q.is_empty() || q.split_whitespace().all(|w| p.display().to_string().to_lowercase().contains(w));
    let mut out: Vec<PathBuf> = pinned.iter().filter(|p| all.contains(p)).filter(|p| keep(p)).cloned().collect();
    out.extend(all.iter().filter(|p| !pinned.contains(p) && keep(p)).cloned());
    out
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
        let find = win::edit(hwnd, "", ID_FIND, font, false);
        let cue = win::wide(t("Search recent"));
        wm::SendMessageW(find, 0x1501 /* EM_SETCUEBANNER */, Some(WPARAM(1)), Some(LPARAM(cue.as_ptr() as isize)));
        let recent = win::list(hwnd, ID_RECENT, font, &[(t("Name"), 190), (t("Folder"), 330)]);
        let begin_label = win::label(hwnd, t("Get started"), 0, bold);
        let mut buttons = Vec::new();
        for (id, label) in [(ID_CLONE, t("Clone a repository...")), (ID_OPEN_FILE, t("Open a form or code file...")), (ID_NEW_PROJECT, t("Create a new project...")), (ID_CONTINUE, t("Continue without code →"))] {
            buttons.push((id, win::button(hwnd, label, id, font, false)));
        }
        // Enter on the list opens the file chosen in it.
        let open = win::button(hwnd, "", ID_OPEN_RECENT, font, true);
        win::show(open, false);
        let show = win::checkbox(hwnd, t("Show this window when Phai Studio starts"), ID_SHOW, font, show_again);
        let tip_at = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) / 86_400) as usize % TIPS.len();
        let tip = win::label(hwnd, &format!("{}  {}", t("Tip:"), t(TIPS[tip_at])), 0, font);
        win::set_css(tip, "muted", "");
        let next_tip = win::button(hwnd, t("Next tip"), ID_NEXT_TIP, font, false);
        win::set_state(hwnd, Box::new(Start { title, recent_label, find, recent, begin_label, buttons, show, tip, next_tip, tip_at, all: paths, shown: vec![], pinned: pinned(), choice: None }));
        refill(hwnd);
        let mut r = RECT { left: 0, top: 0, right: win::px(hwnd, 860), bottom: win::px(hwnd, 520) };
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

fn refill(h: HWND) {
    let Some(s) = state(h) else { return };
    s.shown = arrange(&s.all, &s.pinned, &win::text_of(s.find));
    unsafe {
        wm::SendMessageW(s.recent, cc::LVM_DELETEALLITEMS, None, None);
    }
    for p in &s.shown {
        let pin = if s.pinned.contains(p) { "\u{2605} " } else { "" };
        let name = format!("{pin}{}", p.file_name().unwrap_or_default().to_string_lossy());
        let folder = p.parent().map(|d| d.display().to_string()).unwrap_or_default();
        win::list_add(s.recent, &[&name, &folder]);
    }
    if s.shown.is_empty() {
        win::list_add(s.recent, &[if s.all.is_empty() { t("Nothing opened yet") } else { t("Nothing matches") }, ""]);
    } else {
        win::list_select(s.recent, 0);
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
    let bottom = r.bottom - m - 2 * row - px(14);
    win::move_to(s.recent_label, m, top, left_w, row);
    win::move_to(s.find, m, top + row + px(6), left_w, px(26));
    let list_top = top + row + px(6) + px(32);
    win::move_to(s.recent, m, list_top, left_w, bottom - list_top);
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
        win::move_to(*b, x, y, bw, px(48));
        y += px(48) + px(10);
    }
    let tip_y = r.bottom - m - 2 * row - px(6);
    win::move_to(s.tip, m, tip_y, r.right - 2 * m - px(110), row);
    win::move_to(s.next_tip, r.right - m - px(100), tip_y, px(100), row);
    win::move_to(s.show, m, r.bottom - m - row + px(8), r.right - 2 * m, row);
}

fn chosen_recent(s: &Start) -> Option<PathBuf> {
    let i = unsafe { wm::SendMessageW(s.recent, cc::LVM_GETNEXTITEM, Some(WPARAM(usize::MAX)), Some(LPARAM(2))).0 };
    (i >= 0).then(|| s.shown.get(i as usize).cloned()).flatten()
}

/// The menu of a recent file right-clicked: pinned to the top or not, or taken off the list.
fn recent_menu(h: HWND) {
    let Some(s) = state(h) else { return };
    let Some(p) = chosen_recent(s) else { return };
    const PIN: u32 = 1;
    const FORGET: u32 = 2;
    let on = s.pinned.contains(&p);
    match win::popup(h, &[(PIN, if on { t("Unpin") } else { t("Pin to the top") }), (FORGET, t("Remove from the list"))]) {
        PIN => {
            if on {
                s.pinned.retain(|x| x != &p);
            } else {
                s.pinned.push(p);
            }
            save_pinned(&s.pinned);
        }
        FORGET => {
            s.all.retain(|x| x != &p);
            s.pinned.retain(|x| x != &p);
            save_pinned(&s.pinned);
            crate::forget_recent_pub(&p);
        }
        _ => return,
    }
    refill(h);
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
            if hdr.hwndFrom == s.recent && hdr.code == cc::NM_RCLICK {
                recent_menu(h);
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
                ID_FIND if win::hiword(w.0) as u32 == wm::EN_CHANGE => refill(h),
                ID_NEXT_TIP => {
                    s.tip_at = (s.tip_at + 1) % TIPS.len();
                    win::set_text(s.tip, &format!("{}  {}", t("Tip:"), t(TIPS[s.tip_at])));
                }
                ID_CLONE => s.choice = Some(Choice::Clone),
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
        _ => win::ctl_color(msg, w, l).unwrap_or_else(|| win::def(h, msg, w, l)),
    }
}

/// The folder a clone of `url` goes in under `parent`: the repository's name, `.git` taken off.
pub fn clone_folder(parent: &std::path::Path, url: &str) -> PathBuf {
    let name = url.trim().trim_end_matches('/').rsplit(['/', '\\', ':']).next().unwrap_or("repo").trim_end_matches(".git").to_string();
    parent.join(if name.is_empty() { "repo".to_string() } else { name })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_searched_with_the_pinned_first() {
        let all: Vec<PathBuf> = ["C:/a/shop/main.phaiform", "C:/b/bank/main.rs", "C:/c/shop2/x.rs"].iter().map(PathBuf::from).collect();
        let pinned = vec![PathBuf::from("C:/c/shop2/x.rs"), PathBuf::from("C:/gone.rs")];
        assert_eq!(arrange(&all, &pinned, ""), vec![all[2].clone(), all[0].clone(), all[1].clone()]);
        assert_eq!(arrange(&all, &pinned, "shop main"), vec![all[0].clone()]);
        assert!(arrange(&all, &pinned, "zzz").is_empty());
    }

    #[test]
    fn a_clone_goes_in_the_repository_s_name() {
        let p = std::path::Path::new("D:/src");
        assert_eq!(clone_folder(p, "https://github.com/me/shop.git"), p.join("shop"));
        assert_eq!(clone_folder(p, "git@github.com:me/bank/"), p.join("bank"));
    }

    #[test]
    fn every_tip_has_thai() {
        for tip in TIPS {
            assert!(crate::text::THAI.iter().any(|(en, _)| *en == tip), "no Thai for {tip:?}");
        }
    }
}
