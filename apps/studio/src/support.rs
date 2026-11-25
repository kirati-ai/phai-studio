//! Help > Support Phai Studio: the ways to help it - a star, the Rust Lovers Community, a problem
//! reported - and a gift by PromptPay, whose QR code stays hidden until Show the QR code is
//! pressed, the window growing to show it.

use phai_ui::text::t;
use phai_ui::win;
use std::path::Path;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::WindowsAndMessaging as wm;

const ID_CLOSE: i32 = 1;
const ID_QR: i32 = 3;
const ID_PICTURE: i32 = 4;
/// The links, in their order down the window.
const ID_LINKS: i32 = 10;

/// The PromptPay QR code (Thai QR Payment), as the owner gave it.
const PROMPTPAY: &[u8] = include_bytes!("../art/promptpay.png");

const REPO: &str = env!("CARGO_PKG_REPOSITORY");
const W: i32 = 540;
const M: i32 = 20;
const ICON: i32 = 48;
/// Where the QR button's row is, and the QR code's size when shown (its picture is 885 by 1200).
const QR_ROW: i32 = 268;
const QR_W: i32 = 300;
const QR_H: i32 = 407;

/// The links: what they say, where they go.
fn links() -> [(&'static str, String); 3] {
    [
        (t("Give it a star on GitHub"), REPO.to_string()),
        (t("Ask and answer in the Rust Lovers Community"), format!("{}/board", crate::SITE)),
        (t("Report a problem or an idea"), format!("{REPO}/issues")),
    ]
}

struct Support {
    icon: Option<wm::HICON>,
    picture: HWND,
    qr: HWND,
    close: HWND,
    shown: bool,
    done: bool,
}

fn state<'a>(h: HWND) -> Option<&'a mut Support> {
    unsafe { win::state::<Support>(h) }
}

/// Shows the window over `owner` until it is closed.
pub fn show(owner: HWND) {
    let hwnd = make(owner);
    unsafe {
        win::enable(owner, false);
        win::show_top(hwnd);
        let mut msg = wm::MSG::default();
        // `PHAI_ASK_SHOT=<file.png>`: photographed - with the QR code shown when
        // `PHAI_SUPPORT_QR=1` - then closed.
        if let Some(to) = std::env::var_os("PHAI_ASK_SHOT") {
            if std::env::var_os("PHAI_SUPPORT_QR").is_some() {
                show_qr(hwnd, true);
            }
            for _ in 0..30 {
                while wm::PeekMessageW(&mut msg, None, 0, 0, wm::PM_REMOVE).as_bool() {
                    let _ = wm::TranslateMessage(&msg);
                    wm::DispatchMessageW(&msg);
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            let _ = phai_ui::shot::save(hwnd, Path::new(&to));
            if let Some(s) = state(hwnd) {
                s.done = true;
            }
        }
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

/// The window, filled, the QR code hidden; not yet shown.
fn make(owner: HWND) -> HWND {
    win::init();
    let class = win::register("PhaiSupport", Some(proc_), None);
    let title = win::wide(t("Support Phai Studio"));
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_DLGMODALFRAME, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let bold = win::font(hwnd, 0, true);
        let px = |v| win::px(hwnd, v);
        let at = |h: HWND, x: i32, y: i32, w: i32, ht: i32| win::move_to(h, px(x), px(y), px(w), px(ht));
        // Words that run over more than one line: a label that wraps, from the top.
        let wrapping = |h: HWND| {
            let style = wm::GetWindowLongW(h, wm::GWL_STYLE);
            wm::SetWindowLongW(h, wm::GWL_STYLE, style & !0x200 /* SS_CENTERIMAGE */);
            h
        };
        let x = M + ICON + 16;
        let name = win::label(hwnd, t("Support Phai Studio"), 0, win::font(hwnd, 14, false));
        at(name, x, M, W - x - M, 32);
        let why = wrapping(win::label(hwnd, t("Phai Studio is free for people, schools and small firms. If it helps you, these help it grow:"), 0, font));
        at(why, x, M + 34, W - x - M, 40);
        for (i, (words, url)) in links().iter().enumerate() {
            let l = phai_ui::extra::link_label(hwnd, &format!("• <a href=\"{url}\">{words}</a>"), ID_LINKS + i as i32, font);
            at(l, x, 104 + i as i32 * 24, W - x - M, 22);
        }
        let give = win::label(hwnd, t("A gift by PromptPay (any Thai bank)"), 0, bold);
        at(give, x, 190, W - x - M, 22);
        let pays = wrapping(win::label(hwnd, t("Any amount helps pay for the server, for signing the program and for the time to make it. Thank you."), 0, font));
        win::set_css(pays, "muted", "");
        at(pays, x, 214, W - x - M, 44);
        let qr = win::button(hwnd, t("Show the QR code"), ID_QR, font, false);
        at(qr, x, QR_ROW, 200, 30);
        let picture = phai_ui::picture::picture(hwnd, ID_PICTURE, "");
        at(picture, (W - QR_W) / 2, QR_ROW + 44, QR_W, QR_H);
        win::show(picture, false);
        let close = win::button(hwnd, t("Close"), ID_CLOSE, font, true);
        let icon = phai_ui::icon::app_icon(px(ICON));
        win::set_state(hwnd, Box::new(Support { icon, picture, qr, close, shown: false, done: false }));
        place(hwnd);
        win::center_on(hwnd, owner);
        win::theme_frame(hwnd);
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(close));
        hwnd
    }
}

/// The Close button under what is shown, and the window's height to fit.
fn place(hwnd: HWND) {
    let Some(s) = state(hwnd) else { return };
    let px = |v| win::px(hwnd, v);
    let foot = if s.shown { QR_ROW + 44 + QR_H + 16 } else { QR_ROW + 44 };
    win::move_to(s.close, px(W - M - 100), px(foot), px(100), px(30));
    let mut r = RECT { left: 0, top: 0, right: px(W), bottom: px(foot + 30 + M) };
    unsafe {
        let style = wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32;
        let exs = wm::GetWindowLongW(hwnd, wm::GWL_EXSTYLE) as u32;
        let _ = windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(style), false, wm::WINDOW_EX_STYLE(exs), win::dpi(hwnd) as u32);
        let _ = wm::SetWindowPos(hwnd, None, 0, 0, r.right - r.left, r.bottom - r.top, wm::SWP_NOMOVE | wm::SWP_NOZORDER);
        let _ = Gdi::InvalidateRect(Some(hwnd), None, true);
    }
}

/// Shows or hides the QR code, the window growing or shrinking with it - kept on the screen.
fn show_qr(hwnd: HWND, on: bool) {
    let Some(s) = state(hwnd) else { return };
    if on {
        // The picture, written beside the program's other temporary files for the Picture to load.
        let p = std::env::temp_dir().join("phai-studio-promptpay.png");
        if std::fs::write(&p, PROMPTPAY).is_ok() {
            phai_ui::picture::set_picture(s.picture, &p.to_string_lossy());
        }
    }
    s.shown = on;
    win::show(s.picture, on);
    win::set_text(s.qr, if on { t("Hide the QR code") } else { t("Show the QR code") });
    place(hwnd);
    // Grown past the bottom of the screen: moved up to fit.
    unsafe {
        let mut r = RECT::default();
        let _ = wm::GetWindowRect(hwnd, &mut r);
        let mon = Gdi::MonitorFromWindow(hwnd, Gdi::MONITOR_DEFAULTTONEAREST);
        let mut mi = Gdi::MONITORINFO { cbSize: std::mem::size_of::<Gdi::MONITORINFO>() as u32, ..Default::default() };
        if Gdi::GetMonitorInfoW(mon, &mut mi).as_bool() && r.bottom > mi.rcWork.bottom {
            let top = (mi.rcWork.bottom - (r.bottom - r.top)).max(mi.rcWork.top);
            let _ = wm::SetWindowPos(hwnd, None, r.left, top, 0, 0, wm::SWP_NOSIZE | wm::SWP_NOZORDER);
        }
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(s) = state(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_PAINT => {
            let mut ps = Gdi::PAINTSTRUCT::default();
            unsafe {
                let dc = Gdi::BeginPaint(h, &mut ps);
                if let Some(icon) = s.icon {
                    let (m, size) = (win::px(h, M), win::px(h, ICON));
                    let _ = wm::DrawIconEx(dc, m, m, icon, size, size, 0, None, wm::DI_NORMAL);
                }
                let _ = Gdi::EndPaint(h, &ps);
            }
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const windows::Win32::UI::Controls::NMHDR) };
            // NM_CLICK / NM_RETURN on a link: its address in the browser.
            let code = hdr.code as i32;
            let i = hdr.idFrom as i32 - ID_LINKS;
            if (code == -2 || code == -4) && (0..links().len() as i32).contains(&i) {
                crate::open_with_windows(&links()[i as usize].1);
                return LRESULT(0);
            }
            win::def(h, msg, w, l)
        }
        wm::WM_COMMAND => {
            match win::loword(w.0) as i32 {
                ID_CLOSE | 2 /* IDCANCEL: Esc */ => s.done = true,
                ID_QR => {
                    let on = !s.shown;
                    show_qr(h, on);
                }
                _ => return win::def(h, msg, w, l),
            }
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            s.done = true;
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            if let Some(icon) = s.icon {
                let _ = unsafe { wm::DestroyIcon(icon) };
            }
            drop(unsafe { win::take_state::<Support>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_qr_code_is_a_png() {
        assert_eq!(&PROMPTPAY[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn its_words_are_in_thai() {
        let thai = |s: &str| crate::text::THAI.iter().any(|(en, _)| *en == s);
        let words = ["Support Phai Studio", "Phai Studio is free for people, schools and small firms. If it helps you, these help it grow:", "A gift by PromptPay (any Thai bank)", "Any amount helps pay for the server, for signing the program and for the time to make it. Thank you.", "Show the QR code", "Hide the QR code", "Support Phai Studio...", "Give it a star on GitHub", "Ask and answer in the Rust Lovers Community", "Report a problem or an idea"];
        for s in words {
            assert!(thai(s), "no Thai for {s:?}");
        }
    }
}
