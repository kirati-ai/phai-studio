//! Two of the debugger's windows: Modules (Debug > Windows > Modules, Ctrl+Alt+U) - the program
//! and every DLL it has loaded - and the Exception Helper, Visual Studio's box under the line a
//! panic or an exception stopped at: what happened, what it usually means, and the panic's own
//! words.

use std::path::PathBuf;

use phai_ui::text::{t, tf};
use phai_ui::theme::Color;
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::WindowsAndMessaging as wm;

// --- Modules ------------------------------------------------------------------------------------

struct Modules {
    list: HWND,
}

/// The Modules window over `owner`: each module's name, where it is loaded, and its file.
pub fn modules(owner: HWND, list: &[(PathBuf, u64)]) -> HWND {
    win::init();
    let class = win::register("PhaiModules", Some(modules_proc), None);
    let title = win::wide(t("Modules"));
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_TOOLWINDOW, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_THICKFRAME | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let lv = win::list(hwnd, 10, font, &[(t("Name"), 200), (t("Address"), 150), (t("Path"), 420)]);
        for (path, base) in list {
            let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.display().to_string());
            win::list_add(lv, &[&name, &format!("0x{base:016X}"), &path.display().to_string()]);
        }
        win::set_state(hwnd, Box::new(Modules { list: lv }));
        let mut r = RECT::default();
        let _ = wm::GetWindowRect(owner, &mut r);
        let (w, h) = (win::px(hwnd, 800), win::px(hwnd, 420));
        let _ = wm::SetWindowPos(hwnd, None, r.left + (r.right - r.left - w) / 2, r.top + (r.bottom - r.top - h) / 2, w, h, wm::SWP_NOZORDER);
        win::theme_frame(hwnd);
        win::show_top(hwnd);
        hwnd
    }
}

extern "system" fn modules_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(m) = (unsafe { win::state::<Modules>(h) }) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            win::move_to(m.list, 0, 0, win::loword(l.0 as usize) as i32, win::hiword(l.0 as usize) as i32);
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const windows::Win32::UI::Controls::NMHDR) };
            if hdr.code == windows::Win32::UI::Controls::NM_CUSTOMDRAW {
                return win::list_custom_draw(l);
            }
            win::def(h, msg, w, l)
        }
        wm::WM_NCDESTROY => {
            drop(unsafe { win::take_state::<Modules>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

// --- Exception Helper ---------------------------------------------------------------------------

/// What a stop's reason means, for the Exception Helper: its title and a plain explanation.
pub fn explain(reason: &str) -> (String, String) {
    let code = reason.rsplit("0x").next().and_then(|c| u32::from_str_radix(c.trim(), 16).ok());
    if reason.starts_with("panic") {
        return (t("The program panicked").into(), t("It stopped itself on purpose: an unwrap() or expect() of None or an Err, an index past the end, an overflow, or a panic!() in the code. The message below says which; the Call Stack shows how it got here.").into());
    }
    match code {
        Some(0xC000_0005) => (tf("Access violation (0x{0})", &[&"C0000005"]), t("The program read or wrote memory it may not - a null or freed pointer, often from unsafe code or a DLL.").into()),
        Some(0xC000_0094) => (tf("Integer divided by zero (0x{0})", &[&"C0000094"]), t("A whole number was divided by 0.").into()),
        Some(0xC000_00FD) => (tf("Stack overflow (0x{0})", &[&"C00000FD"]), t("A function called itself, or others in a ring, too deep - a recursion that never ends, or a very large local array.").into()),
        Some(0xC000_001D) => (tf("Illegal instruction (0x{0})", &[&"C000001D"]), t("The processor met an instruction it cannot run - often an abort: the program ended itself after a second panic.").into()),
        Some(0xC000_0409) => (tf("Stack buffer overrun (0x{0})", &[&"C0000409"]), t("The program ended itself at once (process::abort or a failed security check).").into()),
        Some(c) => (tf("Exception 0x{0}", &[&format!("{c:08X}")]), t("Windows stopped the program with this exception. Exception Settings (Ctrl+Alt+E) chooses which ones stop it.").into()),
        None => (reason.to_string(), String::new()),
    }
}

/// The panic's own words from the program's last lines: `thread 'main' panicked at src\\main.rs:5:9:`
/// and the message after it.
pub fn panic_words(lines: &[String]) -> Option<String> {
    let at = lines.iter().rposition(|l| l.contains("panicked at"))?;
    let mut said: Vec<&str> = lines[at..].iter().map(|l| l.trim_end()).take_while(|l| !l.starts_with("note:")).collect();
    said.retain(|l| !l.is_empty());
    Some(said.join("\n"))
}

struct Helper {
    title: String,
    body: String,
    words: String,
    bold: Gdi::HFONT,
    font: Gdi::HFONT,
    mono: Gdi::HFONT,
}

/// The Exception Helper under screen point `at`; Escape or Continue closes it.
pub fn helper(owner: HWND, at: (i32, i32), reason: &str, words: Option<String>) -> HWND {
    win::init();
    let class = win::register("PhaiExceptionHelper", Some(helper_proc), None);
    let (title, body) = explain(reason);
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_TOOLWINDOW, PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_POPUP | wm::WS_BORDER, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let bold = win::font(hwnd, 11, true);
        let font = win::font(hwnd, 0, false);
        let mono = crate::bottom::mono_font(hwnd, 9);
        let words = words.unwrap_or_default();
        let lines = words.lines().count() as i32;
        let (w, h) = (win::px(hwnd, 520), win::px(hwnd, 120 + lines * 18));
        win::set_state(hwnd, Box::new(Helper { title, body, words, bold, font, mono }));
        let _ = wm::SetWindowPos(hwnd, Some(wm::HWND_TOP), at.0, at.1 + win::px(hwnd, 4), w, h, wm::SWP_NOACTIVATE);
        let _ = wm::ShowWindow(hwnd, wm::SW_SHOWNOACTIVATE);
        hwnd
    }
}

/// The panic's words, come after the helper was shown: put in it, and it made tall enough.
pub fn set_words(h: HWND, words: String) {
    let Some(s) = (unsafe { win::state::<Helper>(h) }) else { return };
    if s.words == words {
        return;
    }
    let lines = words.lines().count() as i32;
    s.words = words;
    unsafe {
        let mut r = RECT::default();
        let _ = wm::GetWindowRect(h, &mut r);
        let _ = wm::SetWindowPos(h, None, 0, 0, r.right - r.left, win::px(h, 120 + lines * 18), wm::SWP_NOMOVE | wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
        let _ = Gdi::InvalidateRect(Some(h), None, true);
    }
}

extern "system" fn helper_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(s) = (unsafe { win::state::<Helper>(h) }) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let screen = Gdi::BeginPaint(h, &mut ps);
            let buf = phai_ui::win::off_screen(h, screen);
            let dc = buf.dc;
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            let back = win::window_background_for(h);
            let text = win::window_text_for(h);
            win::fill(dc, &r, back);
            // A red band down the left, as Visual Studio's helper has.
            win::fill(dc, &RECT { right: win::px(h, 5), ..r }, Color::rgb(0xd1, 0x34, 0x38));
            let m = win::px(h, 16);
            let old = Gdi::SelectObject(dc, s.bold.into());
            let mut tr = RECT { left: m, top: win::px(h, 10), right: r.right - m, bottom: win::px(h, 34) };
            win::draw_text(dc, h, &s.title, &mut tr, Color::rgb(0xd1, 0x34, 0x38), Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX);
            Gdi::SelectObject(dc, s.font.into());
            let mut br = RECT { left: m, top: win::px(h, 38), right: r.right - m, bottom: win::px(h, 92) };
            win::draw_text(dc, h, &s.body, &mut br, text, Gdi::DT_WORDBREAK | Gdi::DT_NOPREFIX);
            if !s.words.is_empty() {
                Gdi::SelectObject(dc, s.mono.into());
                let mut wr = RECT { left: m, top: win::px(h, 94), right: r.right - m, bottom: r.bottom - win::px(h, 22) };
                win::draw_text(dc, h, &s.words, &mut wr, text, Gdi::DT_NOPREFIX | Gdi::DT_END_ELLIPSIS);
            }
            Gdi::SelectObject(dc, s.font.into());
            let mut nr = RECT { left: m, top: r.bottom - win::px(h, 22), right: r.right - m, bottom: r.bottom - win::px(h, 4) };
            win::draw_text(dc, h, t("F5 goes on, Shift+F5 stops - Esc or a click closes this"), &mut nr, text.mix(back, 0.4), Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX);
            Gdi::SelectObject(dc, old);
            drop(buf);
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        wm::WM_LBUTTONDOWN => unsafe {
            let _ = wm::DestroyWindow(h);
            LRESULT(0)
        },
        wm::WM_NCDESTROY => {
            drop(unsafe { win::take_state::<Helper>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stop_explained() {
        assert!(explain("panic").0.contains("panicked"));
        assert!(explain("exception 0xC0000005").0.contains("Access violation"));
        assert!(explain("first-chance exception 0xC0000094").0.contains("zero"));
        let lines: Vec<String> = ["hello", "thread 'main' panicked at src\\main.rs:5:9:", "index out of bounds: the len is 3 but the index is 7", "note: run with `RUST_BACKTRACE=1`"].iter().map(|s| s.to_string()).collect();
        assert_eq!(panic_words(&lines).unwrap(), "thread 'main' panicked at src\\main.rs:5:9:\nindex out of bounds: the len is 3 but the index is 7");
    }
}
