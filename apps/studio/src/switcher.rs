//! Ctrl+Tab, as Visual Studio's IDE Navigator: the open documents, the one used last first; Tab
//! (or Ctrl+Tab again) moves down, Shift+Tab up, and letting go of Ctrl opens the one chosen.

use phai_ui::text::t;
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging as wm;

struct Switcher {
    names: Vec<String>,
    at: usize,
    done: bool,
    chosen: Option<usize>,
    font: Gdi::HFONT,
    bold: Gdi::HFONT,
}

fn state<'a>(h: HWND) -> Option<&'a mut Switcher> {
    unsafe { win::state::<Switcher>(h) }
}

/// Which of `names` (in the order shown) to open: the second to begin with, as Ctrl+Tab goes to
/// the one before; `None` on Escape.
pub fn run(owner: HWND, names: Vec<String>) -> Option<usize> {
    if names.len() < 2 {
        return None;
    }
    win::init();
    let class = win::register("PhaiSwitcher", Some(proc_), None);
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_TOOLWINDOW | wm::WS_EX_TOPMOST, PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_POPUP | wm::WS_BORDER, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let bold = win::font(hwnd, 0, true);
        let rows = names.len().min(16) as i32;
        let (w, h) = (win::px(hwnd, 420), win::px(hwnd, 44 + rows * 26));
        win::set_state(hwnd, Box::new(Switcher { names, at: 1, done: false, chosen: None, font, bold }));
        let mut r = RECT::default();
        let _ = wm::GetWindowRect(owner, &mut r);
        let _ = wm::SetWindowPos(hwnd, None, r.left + (r.right - r.left - w) / 2, r.top + (r.bottom - r.top - h) / 3, w, h, wm::SWP_NOZORDER);
        win::show_top(hwnd);
        let _ = km::SetFocus(Some(hwnd));
        let _ = km::SetCapture(hwnd);
        let mut msg = wm::MSG::default();
        while state(hwnd).is_some_and(|s| !s.done) {
            if !wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
                wm::PostQuitMessage(msg.wParam.0 as i32);
                break;
            }
            // Ctrl let go - even before the window saw a key - opens the one chosen.
            if km::GetKeyState(0x11) >= 0 {
                if let Some(s) = state(hwnd) {
                    s.chosen = Some(s.at);
                    s.done = true;
                }
                break;
            }
            let _ = wm::TranslateMessage(&msg);
            wm::DispatchMessageW(&msg);
        }
        let _ = km::ReleaseCapture();
        let chosen = state(hwnd).and_then(|s| s.chosen);
        let _ = wm::DestroyWindow(hwnd);
        chosen
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(s) = state(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_KEYDOWN => {
            let n = s.names.len();
            match w.0 {
                0x09 | 0x28 if unsafe { km::GetKeyState(0x10) } >= 0 || w.0 == 0x28 => s.at = (s.at + 1) % n,
                0x09 | 0x26 => s.at = (s.at + n - 1) % n,
                0x1B => s.done = true,
                0x0D => {
                    s.chosen = Some(s.at);
                    s.done = true;
                }
                _ => {}
            }
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        wm::WM_LBUTTONUP => {
            let row = (win::point_of(l).1 - win::px(h, 36)) / win::px(h, 26);
            if row >= 0 && (row as usize) < s.names.len() {
                s.chosen = Some(row as usize);
                s.done = true;
            }
            LRESULT(0)
        }
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let screen = Gdi::BeginPaint(h, &mut ps);
            let buf = phai_ui::win::off_screen(h, screen);
            let dc = buf.dc;
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            let back = win::window_background_for(h);
            let text = win::window_text_for(h);
            let accent = phai_ui::theme::current().var_color("--accent-tint").unwrap_or(phai_ui::theme::Color::rgb(0xe7, 0xf3, 0xef));
            win::fill(dc, &r, back);
            let old = Gdi::SelectObject(dc, s.bold.into());
            let mut hr = RECT { left: win::px(h, 12), top: win::px(h, 8), right: r.right, bottom: win::px(h, 32) };
            win::draw_text(dc, h, t("Active Files"), &mut hr, text, Gdi::DT_SINGLELINE | Gdi::DT_VCENTER);
            Gdi::SelectObject(dc, s.font.into());
            for (i, name) in s.names.iter().enumerate().take(16) {
                let top = win::px(h, 36) + i as i32 * win::px(h, 26);
                let row = RECT { left: win::px(h, 6), top, right: r.right - win::px(h, 6), bottom: top + win::px(h, 26) };
                if i == s.at {
                    win::fill(dc, &row, accent);
                }
                let mut tr = RECT { left: row.left + win::px(h, 8), ..row };
                win::draw_text(dc, h, name, &mut tr, text, Gdi::DT_SINGLELINE | Gdi::DT_VCENTER | Gdi::DT_NOPREFIX | Gdi::DT_END_ELLIPSIS);
            }
            Gdi::SelectObject(dc, old);
            drop(buf);
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_NCDESTROY => {
            drop(unsafe { win::take_state::<Switcher>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

/// The text in `text` with every `what` made `with`; how many. `case`: as typed only.
pub fn replace_all(text: &str, what: &str, with: &str, case: bool) -> (String, usize) {
    let hits = phai_editor::model::find_all(text, what, case);
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for (a, b) in &hits {
        out.push_str(&text[last..*a]);
        out.push_str(with);
        last = *b;
    }
    out.push_str(&text[last..]);
    (out, hits.len())
}

#[cfg(test)]
mod tests {
    #[test]
    fn replaced_everywhere_case_or_not() {
        assert_eq!(super::replace_all("Foo foo FOO", "foo", "bar", false), ("bar bar bar".to_string(), 3));
        assert_eq!(super::replace_all("Foo foo FOO", "foo", "bar", true), ("Foo bar FOO".to_string(), 1));
        assert_eq!(super::replace_all("ไทย ไทย", "ไทย", "Thai", true).1, 2);
    }
}
