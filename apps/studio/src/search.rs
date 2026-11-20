//! Search (Ctrl+Q), as Visual Studio's feature search: one box, and under it every menu command,
//! Options page and file of the project whose name matches - best first, with where it is and
//! its shortcut. Enter or a double-click does it: the command run, the page opened, the file
//! opened. A greyed command (not usable now) is not offered.

use std::path::PathBuf;

use phai_ui::text::t;
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::gotoall::score;

const ID_QUERY: i32 = 11;
const ID_LIST: i32 = 12;
const ID_GO: i32 = 1;
const ID_CANCEL: i32 = 2;

/// What a line of the list does.
#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    Command(i32),
    Options(usize),
    File(PathBuf),
}

/// A line: its name, where it is (Build > ..., Options > ..., the file's folder), its shortcut.
#[derive(Clone, Debug, PartialEq)]
pub struct Found {
    pub name: String,
    pub place: String,
    pub keys: String,
    pub target: Target,
}

/// Every command of `menu`, with the way to it: `&` taken out, the shortcut apart. Greyed ones
/// and separators left out; `usable` says which commands can be used now.
pub fn commands(menu: wm::HMENU, usable: &dyn Fn(i32) -> bool) -> Vec<Found> {
    let mut out = Vec::new();
    walk(menu, "", usable, &mut out);
    out
}

fn item_text(menu: wm::HMENU, i: i32) -> String {
    let mut buf = [0u16; 256];
    let n = unsafe { wm::GetMenuStringW(menu, i as u32, Some(&mut buf), wm::MF_BYPOSITION) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

/// A menu's words as they read: `&Build` is Build, `&&` an ampersand.
pub fn plain(s: &str) -> String {
    s.replace("&&", "\u{1}").replace('&', "").replace('\u{1}', "&")
}

fn walk(menu: wm::HMENU, path: &str, usable: &dyn Fn(i32) -> bool, out: &mut Vec<Found>) {
    unsafe {
        for i in 0..wm::GetMenuItemCount(Some(menu)).max(0) {
            let text = item_text(menu, i);
            if text.is_empty() {
                continue;
            }
            let (words, keys) = text.split_once('\t').unwrap_or((&text, ""));
            let words = plain(words).trim_end_matches("...").trim().to_string();
            let sub = wm::GetSubMenu(menu, i);
            if !sub.is_invalid() {
                let deeper = if path.is_empty() { words } else { format!("{path} > {words}") };
                walk(sub, &deeper, usable, out);
                continue;
            }
            let id = wm::GetMenuItemID(menu, i);
            if id == u32::MAX || id == 0 || !usable(id as i32) {
                continue;
            }
            out.push(Found { name: words, place: path.to_string(), keys: keys.to_string(), target: Target::Command(id as i32) });
        }
    }
}

/// The best of `all` for `query`, best first; for nothing typed, the commands as the menus have them.
pub fn matches(all: &[Found], query: &str) -> Vec<Found> {
    let mut scored: Vec<(i32, usize)> = all
        .iter()
        .enumerate()
        .filter_map(|(i, f)| {
            // The name counts most; where it is counts too ("build web" finds Build > Build for Web).
            let by_name = score(&f.name, query);
            let by_all = score(&format!("{} {}", f.place, f.name), query).map(|s| s / 3);
            by_name.or(by_all).map(|s| (s + if matches!(f.target, Target::Command(_)) { 5 } else { 0 }, i))
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored.into_iter().take(200).map(|(_, i)| all[i].clone()).collect()
}

struct Dialog {
    query: HWND,
    list: HWND,
    all: Vec<Found>,
    shown: Vec<Found>,
    chosen: Option<Target>,
    done: bool,
}

fn state<'a>(h: HWND) -> Option<&'a mut Dialog> {
    unsafe { win::state::<Dialog>(h) }
}

/// Runs the search over `owner` until a choice or Escape; `query` typed in already.
pub fn run(owner: HWND, all: Vec<Found>, query: &str) -> Option<Target> {
    win::init();
    let class = win::register("PhaiSearch", Some(proc_), None);
    let title = win::wide(t("Search"));
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_TOOLWINDOW, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_THICKFRAME | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let q = win::edit(hwnd, query, ID_QUERY, font, false);
        let list = win::list(hwnd, ID_LIST, font, &[(t("Name"), 260), (t("Where"), 300), (t("Shortcut"), 110)]);
        let go = win::button(hwnd, "", ID_GO, font, true);
        win::show(go, false);
        win::set_state(hwnd, Box::new(Dialog { query: q, list, all, shown: vec![], chosen: None, done: false }));
        let mut r = RECT::default();
        let _ = wm::GetWindowRect(owner, &mut r);
        let (w, h) = (win::px(hwnd, 720), win::px(hwnd, 440));
        let _ = wm::SetWindowPos(hwnd, None, r.left + (r.right - r.left - w) / 2, r.top + win::px(hwnd, 90), w, h, wm::SWP_NOZORDER);
        layout(hwnd);
        refill(hwnd);
        win::theme_frame(hwnd);
        // `PHAI_SEARCH_TEST`: the best five printed, the best one taken, no window shown.
        if let Ok(typed) = std::env::var("PHAI_SEARCH_TEST") {
            // `PHAI_SEARCH_TEST=words`: those words typed.
            win::set_text(q, &typed);
            refill(hwnd);
            let shown = state(hwnd).map(|d| d.shown.clone()).unwrap_or_default();
            for f in shown.iter().take(5) {
                println!("search: {} | {} | {}", f.name, f.place, f.keys);
            }
            if std::env::var_os("PHAI_SHOT").is_none() {
                let _ = wm::DestroyWindow(hwnd);
                return shown.first().map(|f| f.target.clone());
            }
        }
        win::enable(owner, false);
        win::show_top(hwnd);
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(q));
        wm::SendMessageW(q, 0x00B1, Some(WPARAM(0)), Some(LPARAM(-1))); // EM_SETSEL: all, to type over
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
        let chosen = state(hwnd).and_then(|d| d.chosen.take());
        win::enable(owner, true);
        let _ = wm::SetForegroundWindow(owner);
        let _ = wm::DestroyWindow(hwnd);
        chosen
    }
}

fn layout(h: HWND) {
    let Some(d) = state(h) else { return };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let m = win::px(h, 8);
    let eh = win::px(h, 26);
    win::move_to(d.query, m, m, r.right - 2 * m, eh);
    win::move_to(d.list, m, m + eh + m, r.right - 2 * m, r.bottom - 3 * m - eh);
}

fn refill(h: HWND) {
    let Some(d) = state(h) else { return };
    let q = win::text_of(d.query);
    d.shown = matches(&d.all, &q);
    unsafe {
        wm::SendMessageW(d.list, cc::LVM_DELETEALLITEMS, None, None);
    }
    for f in &d.shown {
        let kind = match &f.target {
            Target::Command(_) => f.place.clone(),
            Target::Options(_) => format!("{} > {}", t("Options"), f.place),
            Target::File(_) => f.place.clone(),
        };
        win::list_add(d.list, &[&f.name, &kind, &f.keys]);
    }
    if !d.shown.is_empty() {
        win::list_select(d.list, 0);
    }
}

fn selected(d: &Dialog) -> Option<usize> {
    let i = unsafe { wm::SendMessageW(d.list, cc::LVM_GETNEXTITEM, Some(WPARAM(usize::MAX)), Some(LPARAM(cc::LVNI_SELECTED as isize))) }.0;
    (i >= 0).then_some(i as usize)
}

fn step(h: HWND, by: i32) {
    let Some(d) = state(h) else { return };
    if d.shown.is_empty() {
        return;
    }
    let now = selected(d).map(|i| i as i32).unwrap_or(-1);
    let next = (now + by).clamp(0, d.shown.len() as i32 - 1) as usize;
    win::list_select(d.list, next);
}

fn choose(h: HWND) {
    let Some(d) = state(h) else { return };
    if let Some(i) = selected(d).or((!d.shown.is_empty()).then_some(0)) {
        d.chosen = d.shown.get(i).map(|f| f.target.clone());
        d.done = true;
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_SIZE => {
            layout(h);
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            let id = win::loword(w.0) as i32;
            let code = win::hiword(w.0) as u32;
            match id {
                ID_QUERY if code == wm::EN_CHANGE => refill(h),
                ID_GO => choose(h),
                ID_CANCEL => {
                    if let Some(d) = state(h) {
                        d.done = true;
                    }
                }
                _ => return win::def(h, msg, w, l),
            }
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.idFrom == ID_LIST as usize {
                if hdr.code == cc::NM_DBLCLK {
                    choose(h);
                    return LRESULT(0);
                }
                if hdr.code == cc::NM_CUSTOMDRAW {
                    return win::list_custom_draw(l);
                }
            }
            win::def(h, msg, w, l)
        }
        wm::WM_CLOSE => {
            if let Some(d) = state(h) {
                d.done = true;
            }
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            drop(unsafe { win::take_state::<Dialog>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::ctl_color(msg, w, l).unwrap_or_else(|| win::def(h, msg, w, l)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(name: &str, place: &str, target: Target) -> Found {
        Found { name: name.into(), place: place.into(), keys: String::new(), target }
    }

    #[test]
    fn a_menu_s_words_as_they_read() {
        assert_eq!(plain("&Build"), "Build");
        assert_eq!(plain("Save && Close"), "Save & Close");
    }

    #[test]
    fn found_by_name_then_by_where_it_is() {
        let all = vec![
            f("Build for Web (PWA)", "Build", Target::Command(1)),
            f("Run on Phone", "Build", Target::Command(2)),
            f("Font", "Text Editor", Target::Options(8)),
            f("phone.rs", "apps/studio/src", Target::File("phone.rs".into())),
        ];
        let m = matches(&all, "phone");
        assert_eq!(m[0].target, Target::File("phone.rs".into()), "the name starting so first");
        assert!(m.iter().any(|x| x.target == Target::Command(2)));
        assert_eq!(matches(&all, "font")[0].target, Target::Options(8));
        assert_eq!(matches(&all, "build web")[0].target, Target::Command(1), "by where it is and its name");
        assert!(matches(&all, "zzz").is_empty());
    }
}
