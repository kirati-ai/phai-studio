//! Every phai-ui control in one window: the place to see a theme, a language and a scale at once.
//!
//! `phai-gallery` opens it. `phai-gallery --shot out.png [--dark] [--thai] [--tab 1] [--css file]
//! [--delay ms]` photographs it and exits - how each change to the library is checked.

#![windows_subsystem = "windows"]

use phai_ui::controls;
use phai_ui::icon::glyph;
use phai_ui::layout::{unscale, Anchor, Dock, Item, Layout, Rect};
use phai_ui::text::{self, t};
use phai_ui::theme::{self, Theme};
use phai_ui::win::{self, wide};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HFONT};
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HMENU};

const ID_EXIT: i32 = 100;
const ID_LIGHT: i32 = 101;
const ID_DARK: i32 = 102;
const ID_THAI: i32 = 103;
const ID_ENGLISH: i32 = 104;
const ID_NEW: i32 = 110;
const ID_OPEN: i32 = 111;
const ID_SAVE: i32 = 112;
const ID_BUILD: i32 = 113;
const ID_RUN: i32 = 114;
const ID_THEME: i32 = 115;
const ID_LANG: i32 = 116;
const ID_ASK: i32 = 117;
const ID_LOAD_CSS: i32 = 118;

const ID_TOOLBAR: i32 = 200;
const ID_TABS: i32 = 201;
const ID_STATUS: i32 = 202;
const ID_GROUP: i32 = 203;
const ID_NAME_LABEL: i32 = 204;
const ID_NAME: i32 = 205;
const ID_PASS_LABEL: i32 = 206;
const ID_PASS: i32 = 207;
const ID_REMEMBER: i32 = 208;
const ID_OFFLINE: i32 = 209;
const ID_THEME_LABEL: i32 = 210;
const ID_RADIO_LIGHT: i32 = 211;
const ID_RADIO_DARK: i32 = 212;
const ID_LANG_LABEL: i32 = 213;
const ID_COMBO: i32 = 214;
const ID_OK: i32 = 215;
const ID_CANCEL: i32 = 216;
const ID_LIST: i32 = 217;

const TIMER_SHOT: usize = 1;

/// The gallery's words in Thai.
static THAI: &[(&str, &str)] = &[
    ("&File", "&ไฟล์"),
    ("&View", "&มุมมอง"),
    ("Exit", "ออก"),
    ("Light", "สว่าง"),
    ("Theme from a CSS file...", "ธีมจากไฟล์ CSS..."),
    ("Theme: {0}", "ธีม: {0}"),
    ("Theme file has a mistake", "ไฟล์ธีมมีจุดผิด"),
    ("Dark", "มืด"),
    ("Language", "ภาษา"),
    ("Ask for a name...", "ถามชื่อ..."),
    ("New", "ใหม่"),
    ("Open", "เปิด"),
    ("Save", "บันทึก"),
    ("Build", "สร้าง"),
    ("Run", "รัน"),
    ("Theme", "ธีม"),
    ("Controls", "ตัวควบคุม"),
    ("List", "รายการ"),
    ("Account", "บัญชี"),
    ("Name", "ชื่อ"),
    ("Password", "รหัสผ่าน"),
    ("Remember me", "จำฉันไว้"),
    ("Work offline", "ทำงานออฟไลน์"),
    ("Theme:", "ธีม:"),
    ("Language:", "ภาษา:"),
    ("Thai", "ไทย"),
    ("English", "อังกฤษ"),
    ("Ready", "พร้อม"),
    ("File", "ไฟล์"),
    ("Size", "ขนาด"),
    ("Changed", "แก้ไขล่าสุด"),
    ("Your name", "ชื่อของคุณ"),
    ("Type a name", "พิมพ์ชื่อ"),
    ("Hello, {0}", "สวัสดี {0}"),
    ("Scale {0}%", "สเกล {0}%"),
];

struct Gallery {
    hwnd: HWND,
    font: HFONT,
    bold: HFONT,
    toolbar: HWND,
    tabs: HWND,
    status: HWND,
    page1: Vec<HWND>,
    list: HWND,
    layout: Layout,
    // Kept to relabel on a language switch.
    labels: Vec<(HWND, &'static str)>,
    radio_light: HWND,
    radio_dark: HWND,
    combo: HWND,
    shot: Option<String>,
}

fn toolbar_items() -> Vec<(i32, &'static str, u16)> {
    vec![
        (ID_NEW, t("New"), glyph::NEW),
        (ID_OPEN, t("Open"), glyph::OPEN),
        (ID_SAVE, t("Save"), glyph::SAVE),
        (0, "", 0),
        (ID_BUILD, t("Build"), glyph::BUILD),
        (ID_RUN, t("Run"), glyph::PLAY),
        (0, "", 0),
        (ID_THEME, t("Theme"), glyph::BRIGHTNESS),
        (ID_LANG, t("Language"), glyph::LANGUAGE),
    ]
}

fn build_menu() -> HMENU {
    unsafe {
        let bar = wm::CreateMenu().unwrap_or_default();
        let add = |m: HMENU, id: i32, label: &str| {
            let w = wide(label);
            let _ = wm::AppendMenuW(m, wm::MF_STRING, id as usize, PCWSTR(w.as_ptr()));
        };
        let sub = |parent: HMENU, label: &str, m: HMENU| {
            let w = wide(label);
            let _ = wm::AppendMenuW(parent, wm::MF_POPUP, m.0 as usize, PCWSTR(w.as_ptr()));
        };
        let file = wm::CreatePopupMenu().unwrap_or_default();
        add(file, ID_ASK, t("Ask for a name..."));
        let _ = wm::AppendMenuW(file, wm::MF_SEPARATOR, 0, None);
        add(file, ID_EXIT, t("Exit"));
        sub(bar, t("&File"), file);
        let view = wm::CreatePopupMenu().unwrap_or_default();
        add(view, ID_LIGHT, t("Light"));
        add(view, ID_DARK, t("Dark"));
        add(view, ID_LOAD_CSS, t("Theme from a CSS file..."));
        let _ = wm::AppendMenuW(view, wm::MF_SEPARATOR, 0, None);
        add(view, ID_THAI, "ไทย");
        add(view, ID_ENGLISH, "English");
        let dark = theme::current().is_dark();
        wm::CheckMenuItem(view, if dark { ID_DARK } else { ID_LIGHT } as u32, wm::MF_CHECKED.0);
        wm::CheckMenuItem(view, if text::is_thai() { ID_THAI } else { ID_ENGLISH } as u32, wm::MF_CHECKED.0);
        sub(bar, t("&View"), view);
        bar
    }
}

impl Gallery {
    fn make(hwnd: HWND, shot: Option<String>, tab: usize) -> Gallery {
        let font = win::font(hwnd, 0, false);
        let bold = win::font(hwnd, 0, true);
        let toolbar = win::toolbar(hwnd, ID_TOOLBAR, &toolbar_items());
        let tabs = controls::tabs(hwnd, ID_TABS, font, &[t("Controls"), t("List")]);
        let status = controls::status_bar(hwnd, ID_STATUS, font);
        controls::status_parts(status, &[90, 70]);

        let group = controls::group(hwnd, t("Account"), ID_GROUP, font);
        let name_label = win::label(hwnd, t("Name"), ID_NAME_LABEL, font);
        let name = win::edit(hwnd, "สมชาย ใจดี", ID_NAME, font, false);
        let pass_label = win::label(hwnd, t("Password"), ID_PASS_LABEL, font);
        let pass = win::edit(hwnd, "secret", ID_PASS, font, true);
        let remember = win::checkbox(hwnd, t("Remember me"), ID_REMEMBER, font, true);
        let offline = win::checkbox(hwnd, t("Work offline"), ID_OFFLINE, font, false);
        let theme_label = win::label(hwnd, t("Theme:"), ID_THEME_LABEL, font);
        let dark = theme::current().is_dark();
        let radio_light = win::radio(hwnd, t("Light"), ID_RADIO_LIGHT, font, 1, !dark);
        let radio_dark = win::radio(hwnd, t("Dark"), ID_RADIO_DARK, font, 1, dark);
        let lang_label = win::label(hwnd, t("Language:"), ID_LANG_LABEL, font);
        let combo = win::combo(hwnd, ID_COMBO, font, &[t("English"), t("Thai")]);
        unsafe {
            wm::SendMessageW(combo, 0x014E /* CB_SETCURSEL */, Some(WPARAM(text::is_thai() as usize)), None);
        }
        let ok = win::button(hwnd, t("OK"), ID_OK, font, true);
        let cancel = win::button(hwnd, t("Cancel"), ID_CANCEL, font, false);

        let list = win::list(hwnd, ID_LIST, font, &[(t("File"), 200), (t("Size"), 70), (t("Changed"), 130)]);
        for row in [
            ["main.rs", "4 KB", "2026-11-16 09:12"],
            ["หน้าต่างหลัก.phaiform", "2 KB", "2026-11-16 09:40"],
            ["form_ui.rs", "11 KB", "2026-11-16 09:40"],
            ["form.rs", "3 KB", "2026-11-16 10:05"],
            ["รายงานภาษีซื้อ.phaiform", "6 KB", "2026-11-15 17:22"],
            ["Cargo.toml", "1 KB", "2026-11-14 08:00"],
        ] {
            win::list_add(list, &row);
        }
        unsafe {
            use windows::Win32::UI::Controls as cc;
            let item = cc::LVITEMW { stateMask: cc::LVIS_SELECTED, state: cc::LVIS_SELECTED, ..Default::default() };
            wm::SendMessageW(list, cc::LVM_SETITEMSTATE, Some(WPARAM(1)), Some(LPARAM(&item as *const _ as isize)));
        }

        // Windows' own spacing, in design pixels at 96 DPI: 11 from the edges, 7 between rows,
        // controls 23 high, buttons at least 75 wide. Widths that hold words are measured, in
        // both languages, so a switch to Thai never clips a label.
        let dpi = win::dpi(hwnd);
        let top_h = unscale(win::toolbar_height(toolbar), dpi);
        let status_h = unscale(controls::status_height(status), dpi);
        let wide_of = |words: &[&'static str]| words.iter().flat_map(|en| [*en, text::lookup(en).unwrap_or(en)]).map(|w| unscale(win::measure(hwnd, font, w).0, dpi)).max().unwrap_or(0);
        const M: i32 = 11;
        const G: i32 = 7;
        const ROW: i32 = 23;
        const W: i32 = 480;
        let tabs_h = 30;
        let y0 = top_h + tabs_h;
        let lw = wide_of(&["Name", "Password", "Theme:", "Language:"]);
        let cx = M + 12 + lw + 12; // controls start after the widest label
        let box_w = |words: &[&'static str]| wide_of(words) + 18 + 8 + 4; // box, gap, room
        let bw = (wide_of(&["OK", "Cancel"]) + 24).max(75);
        let (r1, r2, r3) = (M + 22, M + 22 + ROW + G, M + 22 + 2 * (ROW + G));
        let group_h = r3 + 20 + 12 - M;
        let r4 = M + group_h + M;
        let r5 = r4 + ROW + G;
        let r6 = r5 + ROW + M;
        let page_h = r6 + ROW + M;
        let mut layout = Layout::new((W, y0 + page_h + status_h));
        let at = |x, y, w, h, anchor| Item { bounds: Rect::new(x, y + y0, w, h), dock: Dock::None, anchor };
        let fill_w = W - M - 12 - cx;
        layout
            .add(toolbar, Item { bounds: Rect::new(0, 0, 0, top_h), dock: Dock::Top, anchor: Anchor::default() })
            .add(status, Item { bounds: Rect::new(0, 0, 0, status_h), dock: Dock::Bottom, anchor: Anchor::default() })
            .add(tabs, Item { bounds: Rect::new(0, 0, 0, tabs_h), dock: Dock::Top, anchor: Anchor::default() })
            .add(group, at(M, M, W - 2 * M, group_h, Anchor::TOP_LEFT_RIGHT))
            .add(name_label, at(M + 12, r1, lw, ROW, Anchor::TOP_LEFT))
            .add(name, at(cx, r1, fill_w, ROW, Anchor::TOP_LEFT_RIGHT))
            .add(pass_label, at(M + 12, r2, lw, ROW, Anchor::TOP_LEFT))
            .add(pass, at(cx, r2, fill_w, ROW, Anchor::TOP_LEFT_RIGHT))
            .add(remember, at(cx, r3, box_w(&["Remember me"]), 20, Anchor::TOP_LEFT))
            .add(offline, at(cx + box_w(&["Remember me"]) + 16, r3, box_w(&["Work offline"]), 20, Anchor::TOP_LEFT))
            .add(theme_label, at(M, r4, lw + 12, ROW, Anchor::TOP_LEFT))
            .add(radio_light, at(cx, r4, box_w(&["Light"]), ROW, Anchor::TOP_LEFT))
            .add(radio_dark, at(cx + box_w(&["Light"]) + 16, r4, box_w(&["Dark"]), ROW, Anchor::TOP_LEFT))
            .add(lang_label, at(M, r5, lw + 12, ROW, Anchor::TOP_LEFT))
            .add(combo, at(cx, r5, 160, 200, Anchor::TOP_LEFT))
            .add(ok, at(W - M - 2 * bw - G, r6, bw, ROW, Anchor::BOTTOM_RIGHT))
            .add(cancel, at(W - M - bw, r6, bw, ROW, Anchor::BOTTOM_RIGHT))
            .add(list, Item { bounds: Rect::default(), dock: Dock::Fill, anchor: Anchor::default() });
        win::enable(offline, false);

        let labels = vec![
            (name_label, "Name"),
            (pass_label, "Password"),
            (remember, "Remember me"),
            (offline, "Work offline"),
            (theme_label, "Theme:"),
            (radio_light, "Light"),
            (radio_dark, "Dark"),
            (lang_label, "Language:"),
            (ok, "OK"),
            (cancel, "Cancel"),
            (group, "Account"),
        ];
        let page1 = vec![group, name_label, name, pass_label, pass, remember, offline, theme_label, radio_light, radio_dark, lang_label, combo, ok, cancel];
        let g = Gallery { hwnd, font, bold, toolbar, tabs, status, page1, list, layout, labels, radio_light, radio_dark, combo, shot };
        controls::tabs_select(tabs, tab);
        g.show_page(tab);
        g.say_status();
        g
    }

    fn show_page(&self, i: usize) {
        for h in &self.page1 {
            win::show(*h, i == 0);
        }
        win::show(self.list, i == 1);
    }

    fn say_status(&self) {
        controls::status_text(self.status, 0, t("Ready"));
        controls::status_text(self.status, 1, &text::tf("Scale {0}%", &[&(win::dpi(self.hwnd) * 100 / 96)]));
        controls::status_text(self.status, 2, if text::is_thai() { "ไทย" } else { "English" });
    }

    fn set_theme(&mut self, dark: bool) {
        win::unwatch_theme(self.hwnd);
        theme::set(if dark { Theme::dark() } else { Theme::light() });
        self.after_theme();
    }

    /// Whatever set the theme: the radios and the menu say which, and everything repaints.
    fn after_theme(&mut self) {
        let dark = theme::current().is_dark();
        win::set_checked(if dark { self.radio_dark } else { self.radio_light }, true);
        unsafe {
            let old = wm::GetMenu(self.hwnd);
            let _ = wm::SetMenu(self.hwnd, Some(build_menu()));
            let _ = wm::DestroyMenu(old);
        }
        win::retheme(self.hwnd);
    }

    fn set_thai(&mut self, thai: bool) {
        text::set_thai(thai);
        for (h, en) in &self.labels {
            win::set_text(*h, t(en));
        }
        controls::tabs_relabel(self.tabs, &[t("Controls"), t("List")]);
        win::set_toolbar(self.toolbar, &toolbar_items());
        unsafe {
            let old = wm::GetMenu(self.hwnd);
            let _ = wm::SetMenu(self.hwnd, Some(build_menu()));
            let _ = wm::DestroyMenu(old);
            wm::SendMessageW(self.combo, 0x014E, Some(WPARAM(thai as usize)), None);
        }
        self.say_status();
    }

    fn command(&mut self, id: i32, code: u16) {
        match id {
            ID_EXIT => unsafe {
                let _ = wm::DestroyWindow(self.hwnd);
            },
            ID_LIGHT => self.set_theme(false),
            ID_DARK => self.set_theme(true),
            ID_THEME => {
                let dark = !theme::current().is_dark();
                self.set_theme(dark);
            }
            ID_RADIO_LIGHT | ID_RADIO_DARK => self.set_theme(id == ID_RADIO_DARK),
            ID_THAI => self.set_thai(true),
            ID_ENGLISH => self.set_thai(false),
            ID_LANG => self.set_thai(!text::is_thai()),
            ID_COMBO if code == 1 /* CBN_SELCHANGE */ => self.set_thai(win::combo_selected(self.combo) == 1),
            ID_TABS => self.show_page(code as usize),
            ID_LOAD_CSS => {
                if let Some(path) = open_css(self.hwnd) {
                    win::watch_theme(self.hwnd, &path);
                    controls::status_text(self.status, 0, &text::tf("Theme: {0}", &[&path.file_name().unwrap_or_default().to_string_lossy()]));
                }
            }
            ID_ASK | ID_OK => {
                if let Some(name) = phai_ui::dialogs::ask(Some(self.hwnd), t("Your name"), t("Type a name"), "", false) {
                    controls::status_text(self.status, 0, &text::tf("Hello, {0}", &[&name]));
                }
            }
            _ => {}
        }
    }

    /// A new monitor scale: fonts, icons and positions again.
    fn rescale(&mut self) {
        unsafe {
            let _ = Gdi::DeleteObject(self.font.into());
            let _ = Gdi::DeleteObject(self.bold.into());
        }
        self.font = win::font(self.hwnd, 0, false);
        self.bold = win::font(self.hwnd, 0, true);
        win::set_font_all(self.hwnd, self.font);
        win::set_toolbar(self.toolbar, &toolbar_items());
        let dpi = win::dpi(self.hwnd);
        let top_h = unscale(win::toolbar_height(self.toolbar), dpi);
        let status_h = unscale(controls::status_height(self.status), dpi);
        self.layout.items[0].1.bounds.h = top_h;
        // Positions below the bars were laid out for the old heights; at a new scale those differ
        // by a pixel at most, which the anchors absorb.
        self.layout.items[1].1.bounds.h = status_h;
        self.layout.apply(self.hwnd);
        self.say_status();
    }
}

extern "system" fn proc_(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(g) = (unsafe { win::state::<Gallery>(hwnd) }) else { return win::def(hwnd, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            g.layout.apply(hwnd);
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            g.command(win::loword(w.0) as i32, win::hiword(w.0));
            LRESULT(0)
        }
        0x02E0 /* WM_DPICHANGED */ => {
            let r = unsafe { &*(l.0 as *const RECT) };
            unsafe {
                let _ = wm::SetWindowPos(hwnd, None, r.left, r.top, r.right - r.left, r.bottom - r.top, wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
            }
            g.rescale();
            LRESULT(0)
        }
        win::WM_THEME_CHANGED => {
            if w.0 == 1 {
                g.after_theme();
            } else if let Some(e) = win::theme_error() {
                controls::status_text(g.status, 0, &e);
                win::message(Some(hwnd), &e, t("Theme file has a mistake"), wm::MB_ICONWARNING);
            }
            LRESULT(0)
        }
        wm::WM_TIMER if w.0 == TIMER_SHOT => {
            unsafe {
                let _ = wm::KillTimer(Some(hwnd), TIMER_SHOT);
            }
            if let Some(path) = g.shot.take() {
                if let Err(e) = phai_ui::shot::save(hwnd, std::path::Path::new(&path)) {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
            unsafe {
                let _ = wm::DestroyWindow(hwnd);
            }
            LRESULT(0)
        }
        wm::WM_DESTROY => {
            unsafe { wm::PostQuitMessage(0) };
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            if let Some(g) = unsafe { win::take_state::<Gallery>(hwnd) } {
                unsafe {
                    let _ = Gdi::DeleteObject(g.font.into());
                    let _ = Gdi::DeleteObject(g.bold.into());
                }
            }
            LRESULT(0)
        }
        _ => win::def(hwnd, msg, w, l),
    }
}

/// Asks for a `.css` file.
fn open_css(owner: HWND) -> Option<std::path::PathBuf> {
    use windows::Win32::UI::Controls::Dialogs::{GetOpenFileNameW, OFN_FILEMUSTEXIST, OFN_PATHMUSTEXIST, OPENFILENAMEW};
    // Pairs of description and pattern, each ending in a NUL, the list in two.
    let filter: Vec<u16> = ["CSS (*.css)", "*.css", ""].iter().flat_map(|s| s.encode_utf16().chain(Some(0))).collect();
    let mut file = [0u16; 1024];
    let mut ofn = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: owner,
        lpstrFilter: PCWSTR(filter.as_ptr()),
        lpstrFile: windows::core::PWSTR(file.as_mut_ptr()),
        nMaxFile: file.len() as u32,
        Flags: OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST,
        ..Default::default()
    };
    unsafe { GetOpenFileNameW(&mut ofn) }.as_bool().then(|| {
        let n = file.iter().position(|c| *c == 0).unwrap_or(0);
        std::path::PathBuf::from(String::from_utf16_lossy(&file[..n]))
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let flag = |f: &str| args.iter().any(|a| a == f);
    let value = |f: &str| args.iter().position(|a| a == f).and_then(|i| args.get(i + 1)).cloned();
    text::add(THAI);
    text::set_thai(flag("--thai"));
    let css = value("--css");
    if let Some(path) = &css {
        // Read once before any window, so the first paint is already in it; watched after.
        match theme::load(std::path::Path::new(path)) {
            Ok(t) => theme::set(t),
            Err(e) => {
                win::message(None, &e.to_string(), "Phai Gallery", wm::MB_ICONERROR);
                return;
            }
        }
    } else if flag("--dark") {
        theme::set(Theme::dark());
    }
    let shot = value("--shot");
    let tab = value("--tab").and_then(|s| s.parse().ok()).unwrap_or(0);

    win::init();
    let icon = phai_ui::icon::glyph_icon(glyph::APPS, 32, theme::Color::rgb(0x12, 0x85, 0x5f));
    let class = win::register("PhaiGallery", Some(proc_), icon);
    let title = wide("Phai Gallery");
    unsafe {
        let hwnd = wm::CreateWindowExW(
            Default::default(),
            PCWSTR(class.as_ptr()),
            PCWSTR(title.as_ptr()),
            wm::WS_OVERLAPPEDWINDOW | wm::WS_CLIPCHILDREN,
            wm::CW_USEDEFAULT,
            wm::CW_USEDEFAULT,
            100,
            100,
            None,
            Some(build_menu()),
            Some(win::instance()),
            None,
        )
        .unwrap_or_default();
        let g = Gallery::make(hwnd, shot.clone(), tab);
        win::set_state(hwnd, Box::new(g));
        win::theme_frame(hwnd);
        // The layout's design size, at this monitor's scale.
        let design = win::state::<Gallery>(hwnd).map(|g| g.layout.design).unwrap_or((480, 320));
        let style = wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32;
        let mut r = RECT { left: 0, top: 0, right: win::px(hwnd, design.0), bottom: win::px(hwnd, design.1) };
        let _ = windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(style), true, Default::default(), win::dpi(hwnd) as u32);
        let _ = wm::SetWindowPos(hwnd, None, 0, 0, r.right - r.left, r.bottom - r.top, wm::SWP_NOMOVE | wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
        win::show_top(hwnd);
        if let Some(path) = &css {
            win::watch_theme(hwnd, std::path::Path::new(path));
        }
        if shot.is_some() {
            wm::SetTimer(Some(hwnd), TIMER_SHOT, value("--delay").and_then(|d| d.parse().ok()).unwrap_or(600), None);
        }
        win::run_loop(Some(hwnd));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_sentence_in_the_gallery_has_thai() {
        phai_ui::text::add(super::THAI);
        let used = phai_ui::text::scan(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src")), &[]);
        assert!(used.len() > 20, "the scanner found only {}", used.len());
        assert_eq!(phai_ui::text::problems(&used), Vec::<String>::new());
    }
}
