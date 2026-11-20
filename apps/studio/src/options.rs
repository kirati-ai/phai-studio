//! Tools > Options, as Visual Studio has it: Phai Studio's own settings in one window, by kind in
//! a tree on the left - its look and language, what it opens with, the tabs of the panes, the
//! code editor's text, where projects take the Phai libraries from.

use phai_ui::text::t;
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::WindowsAndMessaging as wm;

const ID_TREE: i32 = 10;
const ID_OK: i32 = 1;
const ID_CANCEL: i32 = 2;
const ID_THEME: i32 = 20;
const ID_LANGUAGE: i32 = 21;
const ID_RECENT: i32 = 22;
const ID_STARTUP: i32 = 23;
const ID_TABBED: i32 = 24;
const ID_FONT_PT: i32 = 25;
const ID_GIT: i32 = 26;

/// Phai Studio's settings as Options shows them.
#[derive(Clone, Debug, PartialEq)]
pub struct Values {
    /// 0 as Windows is set, 1 light, 2 dark.
    pub theme: usize,
    pub thai: bool,
    /// How many files and projects File > Recent lists, 1 to 9.
    pub recent: usize,
    /// On startup: 0 the start window, 1 nothing open, 2 the project opened last.
    pub startup: usize,
    /// Project and Properties as tabs along the bottom.
    pub tabbed: bool,
    /// The code editor's text, points.
    pub font_pt: i32,
    /// Projects take the Phai libraries from GitHub, at this Phai Studio's release.
    pub git: bool,
}

pub const THEMES: [&str; 3] = ["windows", "light", "dark"];
pub const STARTUPS: [&str; 3] = ["start", "empty", "last"];

impl Values {
    /// The settings as saved; `get` reads one by name. What was never set is Phai Studio's default.
    pub fn load(get: impl Fn(&str) -> Option<String>) -> Values {
        let startup = match get("startup").as_deref() {
            Some(v) => STARTUPS.iter().position(|s| *s == v).unwrap_or(0),
            // Before Options: the start window's own "show at startup" tick.
            None if get("start_window").as_deref() == Some("0") => 1,
            None => 0,
        };
        Values {
            theme: get("theme").and_then(|v| THEMES.iter().position(|s| *s == v)).unwrap_or(1),
            thai: get("lang").as_deref() == Some("th"),
            recent: get("recent_max").and_then(|v| v.parse().ok()).unwrap_or(9).clamp(1, 9),
            startup,
            tabbed: get("side_tabbed").as_deref() != Some("0"),
            font_pt: get("editor_font_pt").and_then(|v| v.parse().ok()).unwrap_or(10).clamp(6, 40),
            git: get("library_from_git").as_deref() == Some("1"),
        }
    }

    /// Each setting by name, as it is saved.
    pub fn pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("theme", THEMES[self.theme].to_string()),
            ("lang", if self.thai { "th" } else { "en" }.to_string()),
            ("recent_max", self.recent.to_string()),
            ("startup", STARTUPS[self.startup].to_string()),
            // The start window's own tick says the same.
            ("start_window", if self.startup == 0 { "1" } else { "0" }.to_string()),
            ("side_tabbed", if self.tabbed { "1" } else { "0" }.to_string()),
            ("editor_font_pt", self.font_pt.to_string()),
            ("library_from_git", if self.git { "1" } else { "0" }.to_string()),
        ]
    }
}

/// Whether Windows is set to dark for apps (Settings > Personalisation > Colours).
pub fn windows_dark() -> bool {
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
    let mut v: u32 = 1;
    let mut size = 4u32;
    let key = win::wide(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize");
    let name = win::wide("AppsUseLightTheme");
    let ok = unsafe { RegGetValueW(HKEY_CURRENT_USER, PCWSTR(key.as_ptr()), PCWSTR(name.as_ptr()), RRF_RT_REG_DWORD, None, Some(&mut v as *mut u32 as *mut _), Some(&mut size)) };
    ok.is_ok() && v == 0
}

/// The pages, by the number each tree item keeps: its words and its controls.
const PAGES: usize = 5;

struct Dialog {
    tree: HWND,
    heading: HWND,
    /// Each page's controls: (label or 0, control) rows, top to bottom.
    pages: Vec<Vec<(HWND, HWND)>>,
    page: usize,
    theme: HWND,
    language: HWND,
    recent: HWND,
    startup: HWND,
    tabbed: HWND,
    font_pt: HWND,
    git: HWND,
    ok: HWND,
    cancel: HWND,
    chosen: Option<Values>,
    done: bool,
}

fn state<'a>(h: HWND) -> Option<&'a mut Dialog> {
    unsafe { win::state::<Dialog>(h) }
}

fn page_name(k: usize) -> &'static str {
    [t("General"), t("Startup"), t("Tabs and Windows"), t("Font"), t("Phai libraries")][k]
}

/// Shows Options over `owner`, starting from `now`; the settings chosen, or `None` on Cancel.
/// `page` opens on that page (0 General ... 4 Phai libraries).
pub fn choose(owner: HWND, now: &Values, page: usize) -> Option<Values> {
    let hwnd = make(owner, now, page);
    unsafe {
        win::enable(owner, false);
        win::show_top(hwnd);
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
        chosen
    }
}

/// The window, filled from `now`, not yet shown - for [`choose`], and for a picture in a test.
pub fn make(owner: HWND, now: &Values, page: usize) -> HWND {
    win::init();
    let class = win::register("PhaiOptions", Some(proc_), None);
    let title = win::wide(t("Options"));
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_DLGMODALFRAME, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_THICKFRAME | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let bold = win::font(hwnd, 0, true);
        let tree = win::tree(hwnd, ID_TREE, font);
        let heading = win::label(hwnd, "", 0, bold);
        let label = |text: &str| win::label(hwnd, text, 0, font);
        let theme = win::combo(hwnd, ID_THEME, font, &[t("Use Windows setting"), t("Light"), t("Dark")]);
        let language = win::combo(hwnd, ID_LANGUAGE, font, &["English", "ไทย"]);
        let recent = win::number(hwnd, ID_RECENT, font, 1, 9, now.recent as i32);
        let startup = win::combo(hwnd, ID_STARTUP, font, &[t("Start Window"), t("Empty environment"), t("Most recent project")]);
        let tabbed = win::checkbox(hwnd, t("Project and Properties as tabs along the bottom"), ID_TABBED, font, now.tabbed);
        let font_pt = win::number(hwnd, ID_FONT_PT, font, 6, 40, now.font_pt);
        let git = win::checkbox(hwnd, t("Take the Phai libraries from GitHub, at this Phai Studio's release"), ID_GIT, font, now.git);
        // The note in two lines: a label does not wrap.
        let git_off = label(t("Off: from this machine, beside Phai Studio."));
        let git_on = label(t("On: a project builds on any machine, the same next year."));
        for n in [git_off, git_on] {
            win::set_css(n, "muted", "");
        }
        for (c, v) in [(theme, now.theme), (language, now.thai as usize), (startup, now.startup)] {
            wm::SendMessageW(c, 0x014E /* CB_SETCURSEL */, Some(WPARAM(v)), None);
        }
        let pages = vec![
            vec![(label(t("Color theme:")), theme), (label(t("Language:")), language), (label(t("Items in recently used lists:")), recent)],
            vec![(label(t("On startup, open:")), startup)],
            vec![(HWND::default(), tabbed)],
            vec![(label(t("Size (points):")), font_pt)],
            vec![(HWND::default(), git), (HWND::default(), git_off), (HWND::default(), git_on)],
        ];
        let ok = win::button(hwnd, t("OK"), ID_OK, font, true);
        let cancel = win::button(hwnd, t("Cancel"), ID_CANCEL, font, false);
        // The tree: each kind of setting, its pages under it.
        let mut items = [0isize; PAGES];
        for (group, kids) in [(t("Environment"), &[0usize, 1, 2][..]), (t("Text Editor"), &[3][..]), (t("Projects"), &[4][..])] {
            let top = win::tree_add(tree, 0, group, -1, kids[0] as isize);
            for k in kids {
                items[*k] = win::tree_add(tree, top, page_name(*k), -1, *k as isize);
            }
            win::tree_expand(tree, top);
        }
        win::set_state(hwnd, Box::new(Dialog { tree, heading, pages, page, theme, language, recent, startup, tabbed, font_pt, git, ok, cancel, chosen: None, done: false }));
        let (w, h) = (win::px(hwnd, 760), win::px(hwnd, 480));
        let mut r = RECT::default();
        let _ = wm::GetWindowRect(owner, &mut r);
        let _ = wm::SetWindowPos(hwnd, None, r.left + (r.right - r.left - w) / 2, r.top + (r.bottom - r.top - h) / 2, w, h, wm::SWP_NOZORDER);
        win::theme_frame(hwnd);
        wm::SendMessageW(tree, 0x110B /* TVM_SELECTITEM */, Some(WPARAM(9 /* TVGN_CARET */)), Some(LPARAM(items[page.min(PAGES - 1)])));
        show_page(hwnd, page.min(PAGES - 1));
        hwnd
    }
}

fn show_page(h: HWND, page: usize) {
    let Some(d) = state(h) else { return };
    d.page = page;
    win::set_text(d.heading, page_name(page));
    for (k, rows) in d.pages.iter().enumerate() {
        for (a, b) in rows {
            if !a.is_invalid() {
                win::show(*a, k == page);
            }
            win::show(*b, k == page);
        }
    }
    layout(h);
}

fn layout(h: HWND) {
    let Some(d) = state(h) else { return };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let px = |v| win::px(h, v);
    let (m, row, gap, bw) = (px(12), px(26), px(10), px(96));
    let tree_w = px(220);
    let bottom = r.bottom - m - row - gap;
    win::move_to(d.tree, m, m, tree_w, bottom - m);
    let x = m + tree_w + px(16);
    let right = r.right - m;
    win::move_to(d.heading, x, m, right - x, row);
    let mut y = m + row + gap;
    let label_w = px(220);
    for (a, b) in &d.pages[d.page] {
        if a.is_invalid() {
            // A check box or a note: the whole width, a note as tall as its words need.
            let tall = row;
            win::move_to(*b, x, y, right - x, tall);
            y += tall + gap;
        } else {
            win::move_to(*a, x, y, label_w, row);
            let w = if win::kind_of(*b) == Some(win::Kind::Combo) { px(240) } else { px(90) };
            win::move_to(*b, x + label_w, y, w, if win::kind_of(*b) == Some(win::Kind::Combo) { px(200) } else { row });
            y += row + gap;
        }
    }
    let by = r.bottom - m - row;
    win::move_to(d.ok, r.right - 2 * bw - m - px(8), by, bw, row);
    win::move_to(d.cancel, r.right - bw - m, by, bw, row);
}

/// What the window says now.
fn read(d: &Dialog) -> Values {
    let number = |h: HWND, lo: i32, hi: i32, or: i32| win::text_of(h).trim().parse::<i32>().unwrap_or(or).clamp(lo, hi);
    Values {
        theme: win::combo_selected(d.theme).max(0) as usize,
        thai: win::combo_selected(d.language) == 1,
        recent: number(d.recent, 1, 9, 9) as usize,
        startup: win::combo_selected(d.startup).max(0) as usize,
        tabbed: win::is_checked(d.tabbed),
        font_pt: number(d.font_pt, 6, 40, 10),
        git: win::is_checked(d.git),
    }
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
            mm.ptMinTrackSize = POINT { x: win::px(h, 620), y: win::px(h, 380) };
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            match win::loword(w.0) as i32 {
                ID_OK => {
                    d.chosen = Some(read(d));
                    d.done = true;
                }
                ID_CANCEL => d.done = true,
                _ => return win::def(h, msg, w, l),
            }
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            const TVN_SELCHANGEDW: u32 = (-451i32) as u32;
            if hdr.hwndFrom == d.tree {
                if hdr.code == cc::NM_CUSTOMDRAW {
                    return win::tree_custom_draw(l);
                }
                if hdr.code == TVN_SELCHANGEDW {
                    if let Some(k) = win::tree_selected(d.tree) {
                        show_page(h, (k as usize).min(PAGES - 1));
                    }
                }
            }
            win::def(h, msg, w, l)
        }
        wm::WM_CLOSE => {
            d.done = true;
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            drop(unsafe { win::take_state::<Dialog>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_go_out_and_come_back_the_same() {
        let none = |_: &str| None;
        let d = Values::load(none);
        assert_eq!(d, Values { theme: 1, thai: false, recent: 9, startup: 0, tabbed: true, font_pt: 10, git: false }, "the defaults");
        let v = Values { theme: 2, thai: true, recent: 5, startup: 2, tabbed: false, font_pt: 13, git: true };
        let saved: std::collections::HashMap<&str, String> = v.pairs().into_iter().collect();
        assert_eq!(Values::load(|k| saved.get(k).cloned()), v);
        // Before Options, the start window's tick said whether it opens.
        assert_eq!(Values::load(|k| (k == "start_window").then(|| "0".to_string())).startup, 1);
    }
}
