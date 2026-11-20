//! Tools > Keyboard Shortcuts, as Visual Studio's Options > Environment > Keyboard: any menu
//! command given a key of one's own - pressed in a box, assigned, kept in the settings as
//! `keymap = id:Ctrl+Shift+K;id:F6`. A key given here comes before Phai Studio's own; the menus
//! show it.

use std::sync::Mutex;

use phai_ui::text::{t, tf};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::search::{Found, Target};

/// A key with its modifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Combo {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    /// The virtual-key code.
    pub vk: u16,
}

fn key_name(vk: u16) -> Option<String> {
    Some(match vk {
        0x30..=0x39 | 0x41..=0x5A => char::from(vk as u8).to_string(),
        0x70..=0x87 => format!("F{}", vk - 0x6F),
        0x08 => "Backspace".into(),
        0x09 => "Tab".into(),
        0x0D => "Enter".into(),
        0x20 => "Space".into(),
        0x21 => "PageUp".into(),
        0x22 => "PageDown".into(),
        0x23 => "End".into(),
        0x24 => "Home".into(),
        0x25 => "Left".into(),
        0x26 => "Up".into(),
        0x27 => "Right".into(),
        0x28 => "Down".into(),
        0x2D => "Insert".into(),
        0x2E => "Delete".into(),
        0xBA => ";".into(),
        0xBB => "=".into(),
        0xBC => ",".into(),
        0xBD => "-".into(),
        0xBE => ".".into(),
        0xBF => "/".into(),
        0xC0 => "`".into(),
        0xDB => "[".into(),
        0xDC => "\\".into(),
        0xDD => "]".into(),
        0xDE => "'".into(),
        _ => return None,
    })
}

impl Combo {
    /// As the menus write it: `Ctrl+Shift+K`.
    pub fn text(&self) -> String {
        let mut s = String::new();
        if self.ctrl {
            s.push_str("Ctrl+");
        }
        if self.shift {
            s.push_str("Shift+");
        }
        if self.alt {
            s.push_str("Alt+");
        }
        s.push_str(&key_name(self.vk).unwrap_or_else(|| format!("#{}", self.vk)));
        s
    }

    /// Read back from [`text`].
    pub fn parse(s: &str) -> Option<Combo> {
        let mut c = Combo { ctrl: false, shift: false, alt: false, vk: 0 };
        let parts: Vec<&str> = s.split('+').collect();
        // `Ctrl++` would split into an empty last part: none of the keys here is `+`.
        let (key, mods) = parts.split_last()?;
        for m in mods {
            match m.trim().to_ascii_lowercase().as_str() {
                "ctrl" => c.ctrl = true,
                "shift" => c.shift = true,
                "alt" => c.alt = true,
                _ => return None,
            }
        }
        let key = key.trim();
        c.vk = (1u16..=0xFE).find(|vk| key_name(*vk).is_some_and(|n| n.eq_ignore_ascii_case(key)))?;
        Some(c)
    }

    /// A key alone - a letter with no Ctrl or Alt - would type; it cannot be a shortcut.
    pub fn usable(&self) -> bool {
        self.ctrl || self.alt || (0x70..=0x87).contains(&self.vk)
    }
}

/// The keys given to commands: `id:Ctrl+Shift+K;id:F6` read.
pub fn read(text: &str) -> Vec<(i32, Combo)> {
    text.split(';')
        .filter_map(|pair| {
            let (id, keys) = pair.split_once(':')?;
            Some((id.trim().parse().ok()?, Combo::parse(keys)?))
        })
        .collect()
}

pub fn write(map: &[(i32, Combo)]) -> String {
    map.iter().map(|(id, c)| format!("{id}:{}", c.text())).collect::<Vec<_>>().join(";")
}

/// The keys as the settings have them, read once and again after a change.
static MAP: Mutex<Option<Vec<(i32, Combo)>>> = Mutex::new(None);

fn map() -> Vec<(i32, Combo)> {
    let mut m = MAP.lock().unwrap_or_else(|e| e.into_inner());
    m.get_or_insert_with(|| read(&crate::setting_pub("keymap").unwrap_or_default())).clone()
}

/// The command given the key down now, if one was.
pub fn command_for(vk: u16) -> Option<i32> {
    let down = |k: i32| unsafe { km::GetKeyState(k) } < 0;
    let now = Combo { ctrl: down(0x11), shift: down(0x10), alt: down(0x12), vk };
    map().into_iter().find(|(_, c)| *c == now).map(|(id, _)| id)
}

/// The key given to `id` here, for the menus to show in place of their own.
pub fn key_of(id: i32) -> Option<String> {
    map().into_iter().find(|(i, _)| *i == id).map(|(_, c)| c.text())
}

const ID_LIST: i32 = 10;
const ID_FIND: i32 = 11;
const ID_KEY: i32 = 12;
const ID_ASSIGN: i32 = 13;
const ID_REMOVE: i32 = 14;
const ID_RESET: i32 = 15;
const ID_CLOSE: i32 = 2;

struct Dialog {
    find: HWND,
    list: HWND,
    said: HWND,
    all: Vec<Found>,
    shown: Vec<Found>,
    map: Vec<(i32, Combo)>,
    pressed: Option<Combo>,
    done: bool,
}

fn state<'a>(h: HWND) -> Option<&'a mut Dialog> {
    unsafe { win::state::<Dialog>(h) }
}

/// Runs the window over `owner` with `commands` (every menu command) until closed.
pub fn run(owner: HWND, commands: Vec<Found>) {
    win::init();
    let class = win::register("PhaiKeymap", Some(proc_), None);
    let key_class = win::register("PhaiKeyBox", Some(key_proc), None);
    let title = win::wide(t("Keyboard Shortcuts"));
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_DLGMODALFRAME, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_THICKFRAME | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let find = win::edit(hwnd, "", ID_FIND, font, false);
        let cue = win::wide(t("Search commands"));
        wm::SendMessageW(find, 0x1501 /* EM_SETCUEBANNER */, Some(WPARAM(1)), Some(LPARAM(cue.as_ptr() as isize)));
        let list = win::list(hwnd, ID_LIST, font, &[(t("Command"), 260), (t("Where"), 220), (t("Shortcut"), 150)]);
        let label = win::label(hwnd, t("Press the keys:"), 0, font);
        let key = wm::CreateWindowExW(Default::default(), PCWSTR(key_class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_TABSTOP, 0, 0, 10, 10, Some(hwnd), Some(wm::HMENU(ID_KEY as isize as *mut _)), Some(win::instance()), None).unwrap_or_default();
        win::set_font(key, font);
        let assign = win::button(hwnd, t("Assign"), ID_ASSIGN, font, true);
        let remove = win::button(hwnd, t("Remove"), ID_REMOVE, font, false);
        let reset = win::button(hwnd, t("Reset All"), ID_RESET, font, false);
        let close = win::button(hwnd, t("Close"), ID_CLOSE, font, false);
        let said = win::label(hwnd, "", 0, font);
        win::set_css(said, "muted", "");
        win::set_state(hwnd, Box::new(Dialog { find, list, said, all: commands, shown: vec![], map: map(), pressed: None, done: false }));
        let mut r = RECT::default();
        let _ = wm::GetWindowRect(owner, &mut r);
        let (w, h) = (win::px(hwnd, 720), win::px(hwnd, 520));
        let _ = wm::SetWindowPos(hwnd, None, r.left + (r.right - r.left - w) / 2, r.top + (r.bottom - r.top - h) / 2, w, h, wm::SWP_NOZORDER);
        let px = |v| win::px(hwnd, v);
        let mut c = RECT::default();
        let _ = wm::GetClientRect(hwnd, &mut c);
        let m = px(12);
        win::move_to(find, m, m, c.right - 2 * m, px(26));
        win::move_to(list, m, m + px(34), c.right - 2 * m, c.bottom - m - px(34) - px(124));
        let y = c.bottom - px(112);
        win::move_to(label, m, y, px(120), px(26));
        win::move_to(key, m + px(124), y, px(220), px(26));
        win::move_to(assign, m + px(352), y, px(90), px(26));
        win::move_to(remove, m + px(448), y, px(90), px(26));
        win::move_to(said, m, y + px(34), c.right - 2 * m, px(26));
        win::move_to(reset, m, c.bottom - m - px(26), px(110), px(26));
        win::move_to(close, c.right - m - px(90), c.bottom - m - px(26), px(90), px(26));
        refill(hwnd);
        win::theme_frame(hwnd);
        win::enable(owner, false);
        win::show_top(hwnd);
        let _ = km::SetFocus(Some(find));
        let mut msg = wm::MSG::default();
        while state(hwnd).is_some_and(|d| !d.done) {
            if !wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
                wm::PostQuitMessage(msg.wParam.0 as i32);
                break;
            }
            // The key box takes every key; elsewhere Tab moves on.
            if msg.hwnd == key || !wm::IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
        }
        win::enable(owner, true);
        let _ = wm::SetForegroundWindow(owner);
        let _ = wm::DestroyWindow(hwnd);
    }
}

fn refill(h: HWND) {
    let Some(d) = state(h) else { return };
    let q = win::text_of(d.find);
    d.shown = if q.trim().is_empty() { d.all.clone() } else { crate::search::matches(&d.all, &q) };
    unsafe {
        wm::SendMessageW(d.list, cc::LVM_DELETEALLITEMS, None, None);
    }
    for f in &d.shown {
        let id = match f.target {
            Target::Command(id) => id,
            _ => 0,
        };
        let own = d.map.iter().find(|(i, _)| *i == id).map(|(_, c)| format!("{} *", c.text()));
        win::list_add(d.list, &[&f.name, &f.place, own.as_deref().unwrap_or(&f.keys)]);
    }
}

fn chosen(d: &Dialog) -> Option<i32> {
    let i = unsafe { wm::SendMessageW(d.list, cc::LVM_GETNEXTITEM, Some(WPARAM(usize::MAX)), Some(LPARAM(cc::LVNI_SELECTED as isize))) }.0;
    match d.shown.get(usize::try_from(i).ok()?)?.target {
        Target::Command(id) => Some(id),
        _ => None,
    }
}

fn save(d: &mut Dialog) {
    crate::remember_setting_pub("keymap", &write(&d.map));
    *MAP.lock().unwrap_or_else(|e| e.into_inner()) = Some(d.map.clone());
    if let Some(s) = crate::studio() {
        s.rebuild_menu_pub();
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(d) = state(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_COMMAND => {
            let id = win::loword(w.0) as i32;
            match id {
                ID_FIND if win::hiword(w.0) as u32 == wm::EN_CHANGE => refill(h),
                ID_ASSIGN => match (chosen(d), d.pressed) {
                    (Some(cmd), Some(c)) if c.usable() => {
                        // The key taken from whatever had it here before.
                        let before = d.map.iter().find(|(i, k)| *k == c && *i != cmd).map(|(i, _)| *i);
                        d.map.retain(|(i, k)| *i != cmd && *k != c);
                        d.map.push((cmd, c));
                        save(d);
                        let name = d.all.iter().find(|f| f.target == Target::Command(cmd)).map(|f| f.name.clone()).unwrap_or_default();
                        let said = match before.and_then(|b| d.all.iter().find(|f| f.target == Target::Command(b))) {
                            Some(f) => tf("{0} is {1} now - {2} no longer has it", &[&c.text(), &name, &f.name]),
                            None => tf("{0} is {1} now", &[&c.text(), &name]),
                        };
                        win::set_text(d.said, &said);
                        refill(h);
                    }
                    (None, _) => win::set_text(d.said, t("Choose a command in the list first.")),
                    _ => win::set_text(d.said, t("Press keys with Ctrl or Alt, or a function key, in the box first.")),
                },
                ID_REMOVE => {
                    if let Some(cmd) = chosen(d) {
                        d.map.retain(|(i, _)| *i != cmd);
                        save(d);
                        win::set_text(d.said, t("Its own key back, as Phai Studio has it."));
                        refill(h);
                    }
                }
                ID_RESET => {
                    d.map.clear();
                    save(d);
                    win::set_text(d.said, t("Every command has Phai Studio's own keys again."));
                    refill(h);
                }
                ID_CLOSE => d.done = true,
                _ => return win::def(h, msg, w, l),
            }
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.idFrom == ID_LIST as usize && hdr.code == cc::NM_CUSTOMDRAW {
                return win::list_custom_draw(l);
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
        _ => win::ctl_color(msg, w, l).unwrap_or_else(|| win::def(h, msg, w, l)),
    }
}

/// The box keys are pressed in: it shows them as a shortcut and keeps them.
extern "system" fn key_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let parent = unsafe { wm::GetParent(h) }.unwrap_or_default();
    match msg {
        wm::WM_GETDLGCODE => LRESULT(4 /* DLGC_WANTALLKEYS */),
        wm::WM_LBUTTONDOWN => {
            unsafe {
                let _ = km::SetFocus(Some(h));
            }
            LRESULT(0)
        }
        wm::WM_SETFOCUS | wm::WM_KILLFOCUS => {
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, true);
            }
            win::def(h, msg, w, l)
        }
        wm::WM_KEYDOWN | wm::WM_SYSKEYDOWN => {
            let vk = w.0 as u16;
            // Modifiers alone wait for the key with them.
            if !matches!(vk, 0x10 | 0x11 | 0x12 | 0x5B | 0x5C) {
                let down = |k: i32| unsafe { km::GetKeyState(k) } < 0;
                if let Some(d) = state(parent) {
                    d.pressed = Some(Combo { ctrl: down(0x11), shift: down(0x10), alt: down(0x12), vk });
                    let now = d.pressed.map(|c| c.text()).unwrap_or_default();
                    // Who has it now, as Visual Studio says "Shortcut currently used by".
                    let used = d.map.iter().find(|(_, c)| Some(*c) == d.pressed).and_then(|(i, _)| d.all.iter().find(|f| f.target == Target::Command(*i)));
                    let used = used.or_else(|| d.all.iter().find(|f| f.keys == now));
                    win::set_text(d.said, &match used {
                        Some(f) => tf("{0} is used by {1} now", &[&now, &f.name]),
                        None => tf("{0} is free", &[&now]),
                    });
                }
                unsafe {
                    let _ = Gdi::InvalidateRect(Some(h), None, true);
                }
            }
            LRESULT(0)
        }
        wm::WM_CHAR | wm::WM_SYSCHAR => LRESULT(0),
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            let focused = km::GetFocus() == h;
            let accent = phai_ui::theme::current().var_color("--accent").unwrap_or(phai_ui::theme::Color::rgb(0x12, 0x85, 0x5f));
            let border = if focused { accent } else { phai_ui::theme::Color::rgb(0xbb, 0xbb, 0xbb) };
            win::fill(dc, &r, border);
            let inner = RECT { left: 1, top: 1, right: r.right - 1, bottom: r.bottom - 1 };
            win::fill(dc, &inner, win::window_background_for(h));
            let text = state(parent).and_then(|d| d.pressed).map(|c| c.text()).unwrap_or_else(|| if focused { t("(press keys)").to_string() } else { String::new() });
            let font = wm::SendMessageW(h, wm::WM_GETFONT, None, None);
            let old = Gdi::SelectObject(dc, Gdi::HGDIOBJ(font.0 as *mut _));
            let mut tr = RECT { left: win::px(h, 8), ..inner };
            win::draw_text(dc, h, &text, &mut tr, win::window_text_for(h), Gdi::DT_SINGLELINE | Gdi::DT_VCENTER | Gdi::DT_NOPREFIX);
            Gdi::SelectObject(dc, old);
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        wm::WM_SETFONT => {
            unsafe {
                wm::SetPropW(h, PCWSTR(win::wide("font").as_ptr()), Some(windows::Win32::Foundation::HANDLE(w.0 as *mut _))).ok();
            }
            LRESULT(0)
        }
        wm::WM_GETFONT => LRESULT(unsafe { wm::GetPropW(h, PCWSTR(win::wide("font").as_ptr())) }.0 as isize),
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_written_and_read_back() {
        let c = Combo { ctrl: true, shift: true, alt: false, vk: 0x4B };
        assert_eq!(c.text(), "Ctrl+Shift+K");
        assert_eq!(Combo::parse("Ctrl+Shift+K"), Some(c));
        assert_eq!(Combo::parse("ctrl+F5").map(|c| c.text()), Some("Ctrl+F5".to_string()));
        assert_eq!(Combo::parse("Alt+,").map(|c| c.vk), Some(0xBC));
        assert_eq!(Combo::parse("Hyper+K"), None);
        assert!(!Combo::parse("K").unwrap().usable() && Combo::parse("F6").unwrap().usable());
        let map = vec![(299, c), (102, Combo::parse("F6").unwrap())];
        assert_eq!(write(&map), "299:Ctrl+Shift+K;102:F6");
        assert_eq!(read(&write(&map)), map);
        assert_eq!(read("x:Ctrl+K;5:Nope;7:Ctrl+J"), vec![(7, Combo::parse("Ctrl+J").unwrap())]);
    }
}
