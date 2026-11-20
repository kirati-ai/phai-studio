//! Build > Run on Phone: the program built for the web, served to this network, and a window with
//! its address as a QR code - a phone's camera opens it. The page reloads by itself after each
//! build, as Expo Go or Flutter's hot reload do on a phone.

use phai_ui::text::{t, tf};
use phai_ui::theme::Color;
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::qr;

const ID_OPEN: i32 = 10;
const ID_COPY: i32 = 11;
const ID_CLOSE: i32 = 2; // IDCANCEL: Escape

/// Design pixels.
const W: i32 = 400;
const M: i32 = 14;
const QR: i32 = 250;
const TEXT_H: i32 = 128;
const BUTTON_H: i32 = 26;

struct Phone {
    url: String,
    code: Option<qr::Code>,
    bold: Gdi::HFONT,
    font: Gdi::HFONT,
}

/// The window, filled, not yet shown - for [`show`], and for a picture in a test.
pub fn make(owner: HWND, url: &str) -> HWND {
    win::init();
    let class = win::register("PhaiRunOnPhone", Some(proc_), None);
    let title = win::wide(t("Run on Phone"));
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_DLGMODALFRAME, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let bold = win::font(hwnd, 0, true);
        let px = |v| win::px(hwnd, v);
        let y = M + QR + M + TEXT_H + M;
        let bw = 120;
        let open = win::button(hwnd, t("Open Here"), ID_OPEN, font, false);
        win::move_to(open, px(M), px(y), px(bw), px(BUTTON_H));
        let copy = win::button(hwnd, t("Copy Address"), ID_COPY, font, false);
        win::move_to(copy, px(M + bw + 8), px(y), px(bw), px(BUTTON_H));
        let close = win::button(hwnd, t("Close"), ID_CLOSE, font, true);
        win::move_to(close, px(W - M - 90), px(y), px(90), px(BUTTON_H));
        win::set_state(hwnd, Box::new(Phone { url: url.to_string(), code: qr::encode(url), bold, font }));
        // The client area's size, the frame around it.
        let mut r = RECT { left: 0, top: 0, right: px(W), bottom: px(y + BUTTON_H + M) };
        let _ = wm::AdjustWindowRectEx(&mut r, wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU, false, wm::WS_EX_DLGMODALFRAME);
        let _ = wm::SetWindowPos(hwnd, None, 0, 0, r.right - r.left, r.bottom - r.top, wm::SWP_NOZORDER | wm::SWP_NOMOVE);
        win::center_on(hwnd, owner);
        win::theme_frame(hwnd);
        hwnd
    }
}

/// Shows the address of the program on this network until closed.
pub fn show(owner: HWND, url: &str) {
    let hwnd = make(owner, url);
    win::show_top(hwnd);
    win::modal_loop(hwnd, owner);
}

/// What the window says under the code.
fn words(url: &str) -> [String; 3] {
    [
        t("Point the phone's camera at the code, or type the address:").to_string(),
        url.to_string(),
        tf("The phone must be on this computer's Wi-Fi; if Windows Firewall asks, allow Phai Studio on private networks. The page reloads by itself after each build. To install it on the phone as an app, the page must come from an https:// site - {0}.", &[&t("Build > Publish")]),
    ]
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(d) = (unsafe { win::state::<Phone>(h) }) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut rc = RECT::default();
            let _ = wm::GetClientRect(h, &mut rc);
            let bg = win::window_background_for(h);
            win::fill(dc, &rc, bg);
            let px = |v| win::px(h, v);
            // The code on white whatever the theme: a camera reads dark on light.
            let (qx, qy, side) = (px((W - QR) / 2), px(M), px(QR));
            win::fill(dc, &RECT { left: qx, top: qy, right: qx + side, bottom: qy + side }, Color::rgb(255, 255, 255));
            if let Some(code) = &d.code {
                // Four modules of light around, as the standard asks.
                let n = code.size as i32 + 8;
                let cell = side / n;
                let off_x = qx + (side - cell * n) / 2 + cell * 4;
                let off_y = qy + (side - cell * n) / 2 + cell * 4;
                let black = Color::rgb(0, 0, 0);
                for y in 0..code.size {
                    for x in 0..code.size {
                        if code.at(x, y) {
                            let l = off_x + x as i32 * cell;
                            let t = off_y + y as i32 * cell;
                            win::fill(dc, &RECT { left: l, top: t, right: l + cell, bottom: t + cell }, black);
                        }
                    }
                }
            }
            let text = win::window_text_for(h);
            let [ask, url, note] = words(&d.url);
            let mut y = px(M + QR + M);
            let mut line = |s: &str, f: Gdi::HFONT, colour: Color, height: i32, wrap: bool| {
                let old = Gdi::SelectObject(dc, f.into());
                let mut r = RECT { left: px(M), top: y, right: px(W - M), bottom: y + px(height) };
                let flags = if wrap { Gdi::DT_WORDBREAK } else { Gdi::DT_SINGLELINE | Gdi::DT_CENTER | Gdi::DT_END_ELLIPSIS };
                win::draw_text(dc, h, s, &mut r, colour, flags | Gdi::DT_NOPREFIX);
                Gdi::SelectObject(dc, old);
                y += px(height);
            };
            line(&ask, d.font, text, 20, false);
            line(&url, d.bold, theme_accent(), 24, false);
            line(&note, d.font, text.mix(bg, 0.3), TEXT_H - 44, true);
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_COMMAND => {
            match win::loword(w.0) as i32 {
                ID_OPEN => crate::open_with_windows(&d.url),
                ID_COPY => crate::copy_text(h, &d.url),
                ID_CLOSE => unsafe {
                    let _ = wm::DestroyWindow(h);
                },
                _ => return win::def(h, msg, w, l),
            }
            LRESULT(0)
        }
        wm::WM_CLOSE => unsafe {
            let _ = wm::DestroyWindow(h);
            LRESULT(0)
        },
        wm::WM_NCDESTROY => {
            drop(unsafe { win::take_state::<Phone>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::ctl_color(msg, w, l).unwrap_or_else(|| win::def(h, msg, w, l)),
    }
}

fn theme_accent() -> Color {
    phai_ui::theme::current().var_color("--accent").unwrap_or(Color::rgb(0, 120, 212))
}
