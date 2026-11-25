//! New Project, as Visual Studio's: the templates in a list down the left, by group, with a box to
//! search them; the chosen one's picture - the program it makes, running - its name and what it
//! makes on the right. Next goes on with it; a double-click or Enter in the list too.

use phai_ui::text::t;
use phai_ui::theme::{self, Color};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::templates::{self, Template, GROUPS};

const ID_SEARCH: i32 = 9;
const ID_LIST: i32 = 10;
const ID_PICTURE: i32 = 11;
const ID_OK: i32 = 1;
const ID_CANCEL: i32 = 2;

/// A line of the list: a group's heading, or a template.
#[derive(Clone, Copy)]
enum Row {
    Group(&'static str),
    Item(&'static Template),
}

struct Dialog {
    rows: Vec<Row>,
    chosen: Option<&'static Template>,
    answer: Option<&'static str>,
    done: bool,
    top: i32,
    list: HWND,
    picture: HWND,
    name: HWND,
    about: HWND,
    group: HWND,
    font: Gdi::HFONT,
    bold: Gdi::HFONT,
    big: Gdi::HFONT,
}

fn state<'a>(h: HWND) -> Option<&'a mut Dialog> {
    unsafe { win::state::<Dialog>(h) }
}

/// The template chosen (its key); `None` when cancelled. `PHAI_TEMPLATE`: that one at once (a key,
/// or its place in the list), for tests.
pub fn choose(owner: HWND) -> Option<&'static str> {
    if let Ok(v) = std::env::var("PHAI_TEMPLATE") {
        let all = templates::available();
        return v.parse::<usize>().ok().and_then(|i| all.get(i).map(|t| t.key)).or_else(|| all.iter().find(|t| t.key == v).map(|t| t.key));
    }
    let hwnd = make(owner);
    unsafe {
        win::enable(owner, false);
        win::show_top(hwnd);
        if let Some(d) = state(hwnd) {
            let _ = km::SetFocus(Some(d.list));
        }
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

/// The list's lines for `filter` (words in the name or what it makes, either language).
fn rows_for(filter: &str) -> Vec<Row> {
    let f = filter.trim().to_lowercase();
    let all = templates::available();
    let mut rows = Vec::new();
    for g in GROUPS {
        let items: Vec<&&Template> = all
            .iter()
            .filter(|tp| tp.group == *g)
            .filter(|tp| f.is_empty() || [tp.name, tp.about, t(tp.name), t(tp.about)].iter().any(|s| s.to_lowercase().contains(&f)))
            .collect();
        if items.is_empty() {
            continue;
        }
        rows.push(Row::Group(g));
        rows.extend(items.into_iter().map(|tp| Row::Item(tp)));
    }
    rows
}

/// The window, not yet shown - for [`choose`], and for a picture in a test.
pub fn make(owner: HWND) -> HWND {
    win::init();
    let class = win::register("PhaiNewProject", Some(proc_), None);
    let list_class = win::register("PhaiTemplateList", Some(list_proc), None);
    let title = win::wide(t("New Project"));
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_DLGMODALFRAME, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let bold = win::font(hwnd, 0, true);
        let big = win::font(hwnd, 14, true);
        let px = |v| win::px(hwnd, v);
        let (w, h) = (880, 540);
        let head = win::label(hwnd, t("What kind of project?"), 0, bold);
        win::move_to(head, px(16), px(12), px(320), px(24));
        let search = win::edit(hwnd, "", ID_SEARCH, font, false);
        phai_ui::props::placeholder(search, t("Search templates"));
        win::move_to(search, px(16), px(42), px(320), px(28));
        let list = wm::CreateWindowExW(Default::default(), PCWSTR(list_class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_TABSTOP | wm::WS_VSCROLL, 0, 0, 10, 10, Some(hwnd), Some(wm::HMENU(ID_LIST as isize as *mut _)), Some(win::instance()), None).unwrap_or_default();
        win::move_to(list, px(16), px(78), px(320), px(h - 78 - 60));
        let x = 356;
        let picture = phai_ui::picture::picture(hwnd, ID_PICTURE, "");
        win::move_to(picture, px(x), px(42), px(w - x - 16), px(300));
        let group = win::label(hwnd, "", 0, font);
        win::move_to(group, px(x), px(352), px(w - x - 16), px(22));
        let name = win::label(hwnd, "", 0, big);
        win::move_to(name, px(x), px(374), px(w - x - 16), px(32));
        // A STATIC without SS_CENTERIMAGE wraps its words.
        let about = win::child(hwnd, "STATIC", "", wm::WINDOW_STYLE(0x80 /* SS_NOPREFIX */), Default::default(), 0, font);
        win::register_kind(about, win::Kind::Label);
        win::move_to(about, px(x), px(410), px(w - x - 16), px(h - 410 - 60));
        let ok = win::button(hwnd, t("Next..."), ID_OK, font, true);
        let cancel = win::button(hwnd, t("Cancel"), ID_CANCEL, font, false);
        win::move_to(ok, px(w - 16 - 2 * 96 - 8), px(h - 44), px(96), px(28));
        win::move_to(cancel, px(w - 16 - 96), px(h - 44), px(96), px(28));
        let rows = rows_for("");
        win::set_state(hwnd, Box::new(Dialog { rows, chosen: None, answer: None, done: false, top: 0, list, picture, name, about, group, font, bold, big }));
        // `PHAI_SHOT_TEMPLATE=<key>`: that one chosen, for a picture of it (`--templates`).
        let wanted = std::env::var("PHAI_SHOT_TEMPLATE").ok().and_then(|k| state(hwnd)?.rows.iter().position(|r| matches!(r, Row::Item(tp) if tp.key == k)));
        choose_row(hwnd, wanted.or_else(|| first_item(hwnd)));
        let mut r = RECT { left: 0, top: 0, right: px(w), bottom: px(h) };
        let _ = wm::AdjustWindowRectEx(&mut r, wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU, false, wm::WS_EX_DLGMODALFRAME);
        let _ = wm::SetWindowPos(hwnd, None, 0, 0, r.right - r.left, r.bottom - r.top, wm::SWP_NOZORDER | wm::SWP_NOMOVE);
        win::center_on(hwnd, owner);
        win::theme_frame(hwnd);
        set_scroll(hwnd);
        hwnd
    }
}

fn first_item(dlg: HWND) -> Option<usize> {
    state(dlg)?.rows.iter().position(|r| matches!(r, Row::Item(_)))
}

/// Row `k` chosen: the picture, the name and what it makes shown, the row brought into view.
fn choose_row(dlg: HWND, k: Option<usize>) {
    let Some(d) = state(dlg) else { return };
    let tp = k.and_then(|k| d.rows.get(k)).and_then(|r| if let Row::Item(tp) = r { Some(*tp) } else { None });
    d.chosen = tp;
    win::set_text(d.name, tp.map(|tp| t(tp.name)).unwrap_or(""));
    win::set_text(d.about, tp.map(|tp| t(tp.about)).unwrap_or(""));
    win::set_text(d.group, tp.map(|tp| t(tp.group)).unwrap_or(""));
    // The picture, written beside the program's other temporary files for the Picture to load.
    let path = tp.and_then(|tp| {
        let png = templates::preview(tp.key)?;
        let p = std::env::temp_dir().join(format!("phai-template-{}.png", tp.key));
        std::fs::write(&p, png).ok()?;
        Some(p.to_string_lossy().to_string())
    });
    phai_ui::picture::set_picture(d.picture, path.as_deref().unwrap_or(""));
    if let Some(k) = k {
        let (row_top, row_h) = row_span(d, k);
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(d.list, &mut r);
        }
        if row_top < d.top {
            d.top = row_top;
        } else if row_top + row_h > d.top + r.bottom {
            d.top = row_top + row_h - r.bottom;
        }
    }
    unsafe {
        let _ = Gdi::InvalidateRect(Some(d.list), None, false);
    }
    set_scroll(dlg);
}

fn heights(d: &Dialog) -> (i32, i32) {
    (win::px(d.list, 30), win::px(d.list, 52))
}

/// Where row `k` starts in the list (from its top, unscrolled) and how tall it is.
fn row_span(d: &Dialog, k: usize) -> (i32, i32) {
    let (gh, ih) = heights(d);
    let h = |r: &Row| if matches!(r, Row::Group(_)) { gh } else { ih };
    (d.rows.iter().take(k).map(h).sum(), d.rows.get(k).map(h).unwrap_or(ih))
}

fn total(d: &Dialog) -> i32 {
    row_span(d, d.rows.len()).0
}

fn set_scroll(dlg: HWND) {
    let Some(d) = state(dlg) else { return };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(d.list, &mut r);
    }
    let all = total(d);
    d.top = d.top.clamp(0, (all - r.bottom).max(0));
    let si = wm::SCROLLINFO { cbSize: std::mem::size_of::<wm::SCROLLINFO>() as u32, fMask: wm::SIF_ALL, nMin: 0, nMax: all.max(0), nPage: r.bottom.max(0) as u32, nPos: d.top, nTrackPos: 0 };
    unsafe {
        windows::Win32::UI::Controls::SetScrollInfo(d.list, wm::SB_VERT, &si, true);
    }
}

fn chosen_index(d: &Dialog) -> Option<usize> {
    let tp = d.chosen?;
    d.rows.iter().position(|r| matches!(r, Row::Item(x) if x.key == tp.key))
}

/// The next template row from row `from`, `step` rows on (headings passed over).
fn step_item(d: &Dialog, from: Option<usize>, step: i32) -> Option<usize> {
    let items: Vec<usize> = d.rows.iter().enumerate().filter(|(_, r)| matches!(r, Row::Item(_))).map(|(k, _)| k).collect();
    if items.is_empty() {
        return None;
    }
    let at = from.and_then(|f| items.iter().position(|k| *k == f)).unwrap_or(0) as i32;
    Some(items[(at + step).clamp(0, items.len() as i32 - 1) as usize])
}

extern "system" fn list_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let dlg = unsafe { wm::GetParent(h) }.unwrap_or_default();
    let Some(d) = state(dlg) else { return win::def(h, msg, w, l) };
    let row_at = |y: i32| -> Option<usize> {
        let y = y + d.top;
        (0..d.rows.len()).find(|k| {
            let (top, ht) = row_span(d, *k);
            y >= top && y < top + ht
        })
    };
    match msg {
        wm::WM_GETDLGCODE => LRESULT(1 | 4 /* DLGC_WANTARROWS | DLGC_WANTALLKEYS (Enter) */),
        wm::WM_LBUTTONDOWN | wm::WM_LBUTTONDBLCLK => {
            unsafe {
                let _ = km::SetFocus(Some(h));
            }
            if let Some(k) = row_at(win::point_of(l).1).filter(|k| matches!(d.rows[*k], Row::Item(_))) {
                choose_row(dlg, Some(k));
                if msg == wm::WM_LBUTTONDBLCLK {
                    if let Some(d) = state(dlg) {
                        d.answer = d.chosen.map(|t| t.key);
                        d.done = d.answer.is_some();
                    }
                }
            }
            LRESULT(0)
        }
        wm::WM_KEYDOWN => {
            let now = chosen_index(d);
            let next = match w.0 {
                0x26 => step_item(d, now, -1),
                0x28 => step_item(d, now, 1),
                0x21 => step_item(d, now, -5),
                0x22 => step_item(d, now, 5),
                0x24 => step_item(d, None, 0),
                0x23 => step_item(d, None, 999),
                0x0D => {
                    d.answer = d.chosen.map(|t| t.key);
                    d.done = d.answer.is_some();
                    return LRESULT(0);
                }
                _ => return win::def(h, msg, w, l),
            };
            choose_row(dlg, next);
            LRESULT(0)
        }
        wm::WM_MOUSEWHEEL => {
            let delta = (w.0 >> 16) as u16 as i16 as i32;
            d.top -= delta * win::px(h, 52) / 120;
            set_scroll(dlg);
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        wm::WM_VSCROLL => {
            let (gh, _) = heights(d);
            let mut r = RECT::default();
            unsafe {
                let _ = wm::GetClientRect(h, &mut r);
            }
            match (w.0 & 0xFFFF) as i32 {
                0 => d.top -= gh,
                1 => d.top += gh,
                2 => d.top -= r.bottom,
                3 => d.top += r.bottom,
                4 | 5 => d.top = (w.0 >> 16) as i32,
                _ => {}
            }
            set_scroll(dlg);
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        wm::WM_SIZE => {
            set_scroll(dlg);
            LRESULT(0)
        }
        wm::WM_SETFOCUS | wm::WM_KILLFOCUS => {
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
            let muted = text.mix(back, 0.35);
            let accent = cur.var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f));
            let tint = cur.var_color("--accent-tint").unwrap_or(Color::rgb(0xe7, 0xf3, 0xef));
            win::fill(dc, &r, back);
            let pad = win::px(h, 10);
            let focused = km::GetFocus() == h;
            for (k, row) in d.rows.iter().enumerate() {
                let (top, ht) = row_span(d, k);
                let y = top - d.top;
                if y + ht < 0 || y > r.bottom {
                    continue;
                }
                let line = RECT { left: 0, top: y, right: r.right, bottom: y + ht };
                match row {
                    Row::Group(g) => {
                        let old = Gdi::SelectObject(dc, d.bold.into());
                        let mut tr = RECT { left: pad, top: y + win::px(h, 8), ..line };
                        win::draw_text(dc, h, t(g), &mut tr, accent, Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX);
                        Gdi::SelectObject(dc, old);
                    }
                    Row::Item(tp) => {
                        let on = d.chosen.is_some_and(|c| c.key == tp.key);
                        if on {
                            let card = RECT { left: win::px(h, 2), top: y + 2, right: r.right - win::px(h, 2), bottom: y + ht - 2 };
                            win::round_rect(dc, card, tint, if focused { accent } else { tint }, win::px(h, 6));
                        }
                        let old = Gdi::SelectObject(dc, d.bold.into());
                        let mut nr = RECT { left: pad + win::px(h, 8), top: y + win::px(h, 6), right: r.right - pad, bottom: y + win::px(h, 26) };
                        win::draw_text(dc, h, t(tp.name), &mut nr, text, Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX | Gdi::DT_END_ELLIPSIS);
                        Gdi::SelectObject(dc, d.font.into());
                        let mut ar = RECT { left: nr.left, top: y + win::px(h, 26), right: r.right - pad, bottom: y + ht - win::px(h, 4) };
                        win::draw_text(dc, h, t(tp.about), &mut ar, muted, Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX | Gdi::DT_END_ELLIPSIS);
                        Gdi::SelectObject(dc, old);
                    }
                }
            }
            if d.rows.is_empty() {
                let old = Gdi::SelectObject(dc, d.font.into());
                let mut tr = RECT { left: pad, top: pad, right: r.right - pad, bottom: r.bottom };
                win::draw_text(dc, h, t("No template has these words."), &mut tr, muted, Gdi::DT_WORDBREAK | Gdi::DT_NOPREFIX);
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
            match (win::loword(w.0) as i32, win::hiword(w.0)) {
                (ID_SEARCH, 0x0300 /* EN_CHANGE */) => {
                    let words = win::text_of(HWND(l.0 as *mut _));
                    let keep = d.chosen.map(|t| t.key);
                    d.rows = rows_for(&words);
                    d.top = 0;
                    let again = keep.and_then(|k| d.rows.iter().position(|r| matches!(r, Row::Item(x) if x.key == k)));
                    let k = again.or_else(|| first_item(h));
                    choose_row(h, k);
                }
                (ID_OK, _) => {
                    d.answer = d.chosen.map(|t| t.key);
                    d.done = d.answer.is_some();
                }
                (ID_CANCEL, _) => d.done = true,
                _ => return win::def(h, msg, w, l),
            }
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            d.done = true;
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            if let Some(d) = unsafe { win::take_state::<Dialog>(h) } {
                let _ = unsafe { Gdi::DeleteObject(d.big.into()) };
            }
            win::def(h, msg, w, l)
        }
        _ => win::ctl_color(msg, w, l).unwrap_or_else(|| win::def(h, msg, w, l)),
    }
}

#[cfg(test)]
mod tests {
    use crate::templates::{GROUPS, TEMPLATES};

    #[test]
    fn templates_have_thai() {
        let thai = |s: &str| crate::text::THAI.iter().any(|(en, _)| *en == s);
        for tp in TEMPLATES {
            for s in [tp.name, tp.about] {
                assert!(thai(s), "no Thai for {s:?}");
            }
        }
        for g in GROUPS {
            assert!(thai(g), "no Thai for the group {g:?}");
        }
    }

    #[test]
    fn search_keeps_groups_with_a_match() {
        let rows = super::rows_for("");
        assert!(rows.len() >= 2, "the two made in code at least");
        let none = super::rows_for("no such template here");
        assert!(none.is_empty());
    }
}
