//! New Project's templates, as Visual Studio's and Android Studio's New Project show them: a card
//! each, with a picture of what it makes - a Windows program, or an app for phones (a PWA).

use phai_ui::text::t;
use phai_ui::theme::{self, Color};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::WindowsAndMessaging as wm;

const ID_CARDS: i32 = 10;
const ID_OK: i32 = 1;
const ID_CANCEL: i32 = 2;

/// The templates, in the cards' order: its name and what it makes.
pub const TEMPLATES: [(&str, &str); 2] = [
    ("Windows program", "A window with a menu, a toolbar and a status bar - the form opens filling the screen."),
    ("Phone app (PWA)", "A phone-sized form, its controls one under another; Build > Run on Phone shows it on a phone at once."),
];

struct Dialog {
    chosen: usize,
    answer: Option<usize>,
    done: bool,
    font: Gdi::HFONT,
    bold: Gdi::HFONT,
}

fn state<'a>(h: HWND) -> Option<&'a mut Dialog> {
    unsafe { win::state::<Dialog>(h) }
}

/// The template chosen; `None` when cancelled.
pub fn choose(owner: HWND) -> Option<usize> {
    if let Ok(v) = std::env::var("PHAI_TEMPLATE") {
        return v.parse().ok();
    }
    let hwnd = make(owner);
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
        let answer = state(hwnd).and_then(|d| d.answer);
        win::enable(owner, true);
        let _ = wm::SetForegroundWindow(owner);
        let _ = wm::DestroyWindow(hwnd);
        answer
    }
}

/// The window, not yet shown - for [`choose`], and for a picture in a test.
pub fn make(owner: HWND) -> HWND {
    win::init();
    let class = win::register("PhaiNewProject", Some(proc_), None);
    let cards_class = win::register("PhaiTemplateCards", Some(cards_proc), None);
    let title = win::wide(t("New Project"));
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_DLGMODALFRAME, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let bold = win::font(hwnd, 0, true);
        let px = |v| win::px(hwnd, v);
        let head = win::label(hwnd, t("What kind of project?"), 0, bold);
        win::move_to(head, px(16), px(14), px(600), px(24));
        let cards = wm::CreateWindowExW(Default::default(), PCWSTR(cards_class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_TABSTOP, 0, 0, 10, 10, Some(hwnd), Some(wm::HMENU(ID_CARDS as isize as *mut _)), Some(win::instance()), None).unwrap_or_default();
        win::move_to(cards, px(16), px(46), px(608), px(300));
        let ok = win::button(hwnd, t("Next..."), ID_OK, font, true);
        let cancel = win::button(hwnd, t("Cancel"), ID_CANCEL, font, false);
        win::move_to(ok, px(640 - 16 - 2 * 96 - 8), px(362), px(96), px(28));
        win::move_to(cancel, px(640 - 16 - 96), px(362), px(96), px(28));
        let _ = cards;
        win::set_state(hwnd, Box::new(Dialog { chosen: 0, answer: None, done: false, font, bold }));
        let mut r = RECT { left: 0, top: 0, right: px(640), bottom: px(406) };
        let _ = wm::AdjustWindowRectEx(&mut r, wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU, false, wm::WS_EX_DLGMODALFRAME);
        let _ = wm::SetWindowPos(hwnd, None, 0, 0, r.right - r.left, r.bottom - r.top, wm::SWP_NOZORDER | wm::SWP_NOMOVE);
        win::center_on(hwnd, owner);
        win::theme_frame(hwnd);
        hwnd
    }
}

/// A picture of what template `k` makes, in `r`.
fn picture(dc: Gdi::HDC, r: RECT, k: usize, accent: Color, line: Color, back: Color) {
    let soft = line.mix(back, 0.55);
    let bar = |r: RECT, c: Color| win::fill(dc, &r, c);
    if k == 0 {
        // A window: its title bar, menu, toolbar, a few boxes, a status bar.
        let (w, h) = (r.right - r.left, r.bottom - r.top);
        let win_r = RECT { left: r.left + w / 10, top: r.top + h / 8, right: r.right - w / 10, bottom: r.bottom - h / 8 };
        win::round_rect(dc, win_r, back, line, 6);
        let ww = win_r.right - win_r.left;
        let x = win_r.left + 8;
        bar(RECT { left: win_r.left + 1, top: win_r.top + 1, right: win_r.right - 1, bottom: win_r.top + 16 }, soft.mix(back, 0.5));
        for (i, wdt) in [24, 28, 22].iter().enumerate() {
            bar(RECT { left: x + i as i32 * 34, top: win_r.top + 22, right: x + i as i32 * 34 + wdt, bottom: win_r.top + 26 }, soft);
        }
        for i in 0..5 {
            bar(RECT { left: x + i * 16, top: win_r.top + 34, right: x + i * 16 + 10, bottom: win_r.top + 44 }, soft);
        }
        bar(RECT { left: x, top: win_r.top + 56, right: x + ww / 4, bottom: win_r.top + 62 }, soft);
        bar(RECT { left: x + ww / 3, top: win_r.top + 52, right: win_r.right - 8, bottom: win_r.top + 66 }, soft.mix(back, 0.6));
        bar(RECT { left: x, top: win_r.top + 76, right: win_r.right - 8, bottom: win_r.bottom - 44 }, soft.mix(back, 0.75));
        bar(RECT { left: win_r.right - 8 - 44, top: win_r.bottom - 36, right: win_r.right - 8, bottom: win_r.bottom - 24 }, accent);
        bar(RECT { left: win_r.left + 1, top: win_r.bottom - 14, right: win_r.right - 1, bottom: win_r.bottom - 13 }, soft);
    } else {
        // A phone: a tall frame, its controls one under another, a wide button.
        let h = r.bottom - r.top - 16;
        let w = h * 9 / 19;
        let cx = (r.left + r.right) / 2;
        let ph = RECT { left: cx - w / 2, top: r.top + 8, right: cx + w / 2, bottom: r.top + 8 + h };
        win::round_rect(dc, ph, Color::rgb(0x22, 0x22, 0x26), Color::rgb(0x22, 0x22, 0x26), 16);
        let sc = RECT { left: ph.left + 5, top: ph.top + 5, right: ph.right - 5, bottom: ph.bottom - 5 };
        win::round_rect(dc, sc, back, back, 12);
        let (x0, x1) = (sc.left + 8, sc.right - 8);
        bar(RECT { left: cx - 14, top: sc.top + 4, right: cx + 14, bottom: sc.top + 8 }, Color::rgb(0x22, 0x22, 0x26));
        bar(RECT { left: x0, top: sc.top + 18, right: x0 + (x1 - x0) / 2, bottom: sc.top + 24 }, soft);
        let mut y = sc.top + 32;
        for k in 0..3 {
            bar(RECT { left: x0, top: y, right: x1 - k * 10, bottom: y + 4 }, soft);
            bar(RECT { left: x0, top: y + 8, right: x1, bottom: y + 22 }, soft.mix(back, 0.6));
            y += 30;
        }
        bar(RECT { left: x0, top: sc.bottom - 30, right: x1, bottom: sc.bottom - 14 }, accent);
    }
}

extern "system" fn cards_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let parent = unsafe { wm::GetParent(h) }.unwrap_or_default();
    let Some(d) = state(parent) else { return win::def(h, msg, w, l) };
    let card_at = |x: i32| -> usize {
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(h, &mut r);
        }
        usize::from(x >= r.right / 2)
    };
    match msg {
        wm::WM_GETDLGCODE => LRESULT(1 /* DLGC_WANTARROWS */),
        wm::WM_LBUTTONDOWN | wm::WM_LBUTTONDBLCLK => {
            d.chosen = card_at(win::point_of(l).0);
            unsafe {
                let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(h));
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            if msg == wm::WM_LBUTTONDBLCLK {
                d.answer = Some(d.chosen);
                d.done = true;
            }
            LRESULT(0)
        }
        wm::WM_KEYDOWN if matches!(w.0, 0x25 | 0x27) => {
            d.chosen = usize::from(w.0 == 0x27);
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
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
            let cur = theme::current();
            let back = win::window_background_for(h);
            let text = win::window_text_for(h);
            let accent = cur.var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f));
            let tint = cur.var_color("--accent-tint").unwrap_or(Color::rgb(0xe7, 0xf3, 0xef));
            let border = cur.var_color("--rule").unwrap_or(Color::rgb(0xdd, 0xdd, 0xdd));
            win::fill(dc, &r, back);
            let gap = win::px(h, 16);
            let cw = (r.right - gap) / 2;
            for (k, (name, about)) in TEMPLATES.iter().enumerate() {
                let on = k == d.chosen;
                let card = RECT { left: k as i32 * (cw + gap), top: 0, right: k as i32 * (cw + gap) + cw, bottom: r.bottom };
                win::round_rect(dc, card, if on { tint } else { back }, if on { accent } else { border }, win::px(h, 8));
                let pic = RECT { left: card.left + win::px(h, 12), top: card.top + win::px(h, 12), right: card.right - win::px(h, 12), bottom: card.top + win::px(h, 190) };
                picture(dc, pic, k, accent, text.mix(back, 0.3), back);
                let old = Gdi::SelectObject(dc, d.bold.into());
                let mut tr = RECT { left: card.left + win::px(h, 14), top: card.top + win::px(h, 204), right: card.right - win::px(h, 14), bottom: card.top + win::px(h, 228) };
                win::draw_text(dc, h, t(name), &mut tr, text, Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX);
                Gdi::SelectObject(dc, d.font.into());
                let mut ar = RECT { left: tr.left, top: tr.bottom + win::px(h, 4), right: tr.right, bottom: card.bottom - win::px(h, 8) };
                win::draw_text(dc, h, t(about), &mut ar, text.mix(back, 0.3), Gdi::DT_WORDBREAK | Gdi::DT_NOPREFIX);
                Gdi::SelectObject(dc, old);
            }
            drop(buf);
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        wm::WM_ERASEBKGND => LRESULT(1),
        _ => win::def(h, msg, w, l),
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(d) = state(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_COMMAND => {
            match win::loword(w.0) as i32 {
                ID_OK => {
                    d.answer = Some(d.chosen);
                    d.done = true;
                }
                ID_CANCEL => d.done = true,
                _ => return win::def(h, msg, w, l),
            }
            LRESULT(0)
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

#[cfg(test)]
mod tests {
    #[test]
    fn templates_have_thai() {
        for (name, about) in super::TEMPLATES {
            for s in [name, about] {
                assert!(crate::text::THAI.iter().any(|(en, _)| *en == s), "no Thai for {s:?}");
            }
        }
    }
}
