//! New Project, as Visual Studio's: the templates down the left, by group, two to a line, each a card
//! with a picture of the program it makes and its name under it - the chosen one raised on a shadow -
//! and a box to search them;
//! on the right, on a card of its own, the chosen one's picture, its name and what it makes. All
//! on a light grey, so the white cards stand out. Next goes on with it; a double-click or Enter in
//! the list too.

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
const ID_OK: i32 = 1;
const ID_CANCEL: i32 = 2;

/// A line of the list: a group's heading, or a template.
#[derive(Clone, Copy)]
enum Row {
    Group(&'static str),
    Item(&'static Template),
}

/// The window's size, in design pixels.
const W: i32 = 1040;
const H: i32 = 640;
/// The list's left and width; the right card's left.
const LIST_X: i32 = 20;
const LIST_W: i32 = 480;
const CARD_X: i32 = 520;
/// Cards on a line of the list.
const COLS: i32 = 2;
/// A card's picture: as wide as the card, this much as tall as it is wide.
const PIC_SHAPE: f32 = 0.62;
/// The big picture on the right card.
const PIC_H: i32 = 320;

/// The colours: the grey behind, the cards, their edge, the words.
#[derive(Clone, Copy)]
struct Colours {
    back: Color,
    card: Color,
    edge: Color,
    text: Color,
    muted: Color,
    accent: Color,
}

fn colours() -> Colours {
    let cur = theme::current();
    let surface = cur.var_color("--surface").unwrap_or_else(win::window_background);
    let text = cur.var_color("--text").unwrap_or_else(win::window_text);
    let dark = cur.is_dark();
    // Light: a soft grey behind white cards. Dark: the cards a little lighter than what is behind.
    let (back, card) = if dark { (surface, surface.mix(Color::rgb(255, 255, 255), 0.06)) } else { (surface.mix(Color::rgb(128, 128, 128), 0.13), surface) };
    let edge = cur.var_color("--border").unwrap_or(Color::rgb(0xd4, 0xd4, 0xd4));
    let accent = cur.var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f));
    Colours { back, card, edge, text, muted: text.mix(card, 0.38), accent }
}

fn hex(c: Color) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
}

/// A thin frame just inside `r`.
fn frame(dc: Gdi::HDC, r: RECT, line: Color) {
    win::fill(dc, &RECT { left: r.left, top: r.top, right: r.right, bottom: r.top + 1 }, line);
    win::fill(dc, &RECT { left: r.left, top: r.bottom - 1, right: r.right, bottom: r.bottom }, line);
    win::fill(dc, &RECT { left: r.left, top: r.top, right: r.left + 1, bottom: r.bottom }, line);
    win::fill(dc, &RECT { left: r.right - 1, top: r.top, right: r.right, bottom: r.bottom }, line);
}

/// A soft shadow under `r`: rings of grey, darker near it, a little lower than it.
fn shadow(dc: Gdi::HDC, h: HWND, r: RECT, back: Color, radius: i32) {
    let drop = win::px(h, 4);
    let n = win::px(h, 8).max(4);
    for i in (1..=n).rev() {
        let c = back.mix(Color::rgb(0, 0, 0), 0.24 * ((n - i + 1) as f32 / n as f32).powf(1.6));
        let ring = RECT { left: r.left - i, top: r.top - i + drop, right: r.right + i, bottom: r.bottom + i + drop };
        win::round_rect(dc, ring, c, c, radius + i);
    }
}

struct Dialog {
    rows: Vec<Row>,
    chosen: Option<&'static Template>,
    answer: Option<&'static str>,
    done: bool,
    top: i32,
    /// The row under the mouse.
    hot: Option<usize>,
    list: HWND,
    /// Each template's small picture, by its key; the chosen one's big picture.
    thumbs: Vec<(&'static str, phai_ui::picture::Bitmap)>,
    big_picture: Option<phai_ui::picture::Bitmap>,
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
    let list_class = win::register_with(win::ClassStyle::DoubleClicks, "PhaiTemplateList", Some(list_proc), None);
    let title = win::wide(t("New Project"));
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_DLGMODALFRAME, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let bold = win::font(hwnd, 0, true);
        let big = win::font(hwnd, 14, true);
        let px = |v| win::px(hwnd, v);
        let (w, h) = (W, H);
        let c = colours();
        win::set_window_colors(hwnd, Some(c.back), None);
        let head = win::label(hwnd, t("What kind of project?"), 0, bold);
        win::move_to(head, px(LIST_X), px(14), px(LIST_W), px(24));
        let search = win::edit(hwnd, "", ID_SEARCH, font, false);
        phai_ui::props::placeholder(search, t("Search templates"));
        win::move_to(search, px(LIST_X), px(44), px(LIST_W - 16), px(30));
        let list = wm::CreateWindowExW(Default::default(), PCWSTR(list_class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_TABSTOP | wm::WS_VSCROLL, 0, 0, 10, 10, Some(hwnd), Some(wm::HMENU(ID_LIST as isize as *mut _)), Some(win::instance()), None).unwrap_or_default();
        win::move_to(list, px(LIST_X - 8), px(82), px(LIST_W + 8), px(h - 82 - 68));
        // Its scroll bar dark in a dark theme, as the other lists'.
        let look = if theme::current().is_dark() { windows::core::w!("DarkMode_Explorer") } else { windows::core::w!("Explorer") };
        let _ = windows::Win32::UI::Controls::SetWindowTheme(list, look, PCWSTR::null());
        // On the right card: what it is, its name, what it makes - on the card's colour.
        let (x, right) = (CARD_X + 20, w - 40);
        let on_card = |l: HWND| {
            let _ = win::set_own_style(l, &format!("background: {}", hex(c.card)));
            l
        };
        let group = on_card(win::label(hwnd, "", 0, bold));
        win::set_css(group, "muted", "");
        win::move_to(group, px(x), px(36 + PIC_H + 16), px(right - x), px(22));
        let name = on_card(win::label(hwnd, "", 0, big));
        win::move_to(name, px(x), px(36 + PIC_H + 40), px(right - x), px(34));
        // A STATIC without SS_CENTERIMAGE wraps its words.
        let about = win::child(hwnd, "STATIC", "", wm::WINDOW_STYLE(0x80 /* SS_NOPREFIX */), Default::default(), 0, font);
        win::register_kind(about, win::Kind::Label);
        on_card(about);
        win::move_to(about, px(x), px(36 + PIC_H + 80), px(right - x), px(h - 88 - (36 + PIC_H + 80)));
        let ok = win::button(hwnd, t("Next..."), ID_OK, font, true);
        let cancel = win::button(hwnd, t("Cancel"), ID_CANCEL, font, false);
        win::move_to(ok, px(w - 20 - 2 * 104 - 8), px(h - 46), px(104), px(30));
        win::move_to(cancel, px(w - 20 - 104), px(h - 46), px(104), px(30));
        let rows = rows_for("");
        // The cards' pictures, made the size a card shows them.
        let (tw, th) = thumb_size(list);
        let thumbs = templates::available().iter().filter_map(|tp| Some((tp.key, phai_ui::picture::Bitmap::from_bytes(templates::preview(tp.key)?, tw, th)?))).collect();
        win::set_state(hwnd, Box::new(Dialog { rows, chosen: None, answer: None, done: false, top: 0, hot: None, list, thumbs, big_picture: None, name, about, group, font, bold, big }));
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
    // Its picture, made the size the right card shows it.
    let pic = big_rect(dlg);
    d.big_picture = tp.and_then(|tp| phai_ui::picture::Bitmap::from_bytes(templates::preview(tp.key)?, pic.right - pic.left, pic.bottom - pic.top));
    unsafe {
        let _ = Gdi::InvalidateRect(Some(dlg), Some(&pic), false);
    }
    if let Some(at) = k.and_then(|k| layout(d).get(k).copied()) {
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(d.list, &mut r);
        }
        if at.top < d.top {
            d.top = at.top;
        } else if at.bottom > d.top + r.bottom {
            d.top = at.bottom - r.bottom;
        }
    }
    unsafe {
        let _ = Gdi::InvalidateRect(Some(d.list), None, false);
    }
    set_scroll(dlg);
}

/// Where the big picture goes, in the window.
fn big_rect(dlg: HWND) -> RECT {
    let px = |v| win::px(dlg, v);
    RECT { left: px(CARD_X + 16), top: px(32), right: px(W - 36), bottom: px(32 + PIC_H) }
}

/// The right card, in the window.
fn card_rect(dlg: HWND) -> RECT {
    let px = |v| win::px(dlg, v);
    RECT { left: px(CARD_X), top: px(16), right: px(W - 20), bottom: px(H - 68) }
}

/// Round a card: the room between it and its place in the list, and inside it.
fn margin(list: HWND) -> i32 {
    win::px(list, 6)
}

fn inset(list: HWND) -> i32 {
    win::px(list, 8)
}

/// A card's place's width in the list `list`.
fn slot_width(list: HWND) -> i32 {
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(list, &mut r);
    }
    (r.right - win::px(list, 4)) / COLS
}

/// The size of a card's picture.
fn thumb_size(list: HWND) -> (i32, i32) {
    let w = slot_width(list) - 2 * margin(list) - 2 * inset(list);
    (w, (w as f32 * PIC_SHAPE) as i32)
}

/// Each row's place in the list, from its top (unscrolled): a heading across it, the cards
/// `COLS` to a line - each card its picture and, under it, a line for its name.
fn layout(d: &Dialog) -> Vec<RECT> {
    let px = |v| win::px(d.list, v);
    let cw = slot_width(d.list);
    let (_, th) = thumb_size(d.list);
    let ch = 2 * margin(d.list) + 2 * inset(d.list) + th + px(6) + px(26);
    let gh = px(36);
    let mut out = Vec::with_capacity(d.rows.len());
    let (mut y, mut col) = (0, 0);
    for row in &d.rows {
        match row {
            Row::Group(_) => {
                if col > 0 {
                    y += ch;
                    col = 0;
                }
                out.push(RECT { left: 0, top: y, right: cw * COLS, bottom: y + gh });
                y += gh;
            }
            Row::Item(_) => {
                out.push(RECT { left: col * cw, top: y, right: (col + 1) * cw, bottom: y + ch });
                col += 1;
                if col == COLS {
                    col = 0;
                    y += ch;
                }
            }
        }
    }
    out
}

fn total(d: &Dialog) -> i32 {
    layout(d).iter().map(|r| r.bottom).max().unwrap_or(0)
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

/// The card above (`down` false) or below row `from`: on the nearest line that way, the one most
/// over or under it.
fn line_item(d: &Dialog, from: Option<usize>, down: bool) -> Option<usize> {
    let all = layout(d);
    let Some(f) = from else { return step_item(d, None, 0) };
    let here = all[f];
    let items = d.rows.iter().enumerate().filter(|(_, r)| matches!(r, Row::Item(_))).map(|(k, _)| k);
    let beyond: Vec<usize> = items.filter(|k| if down { all[*k].top > here.top } else { all[*k].top < here.top }).collect();
    let line = if down { beyond.iter().map(|k| all[*k].top).min() } else { beyond.iter().map(|k| all[*k].top).max() };
    let Some(line) = line else { return Some(f) };
    beyond.into_iter().filter(|k| all[*k].top == line).min_by_key(|k| (all[*k].left - here.left).abs())
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
    let row_at = |(x, y): (i32, i32)| -> Option<usize> {
        let y = y + d.top;
        layout(d).iter().position(|r| x >= r.left && x < r.right && y >= r.top && y < r.bottom)
    };
    match msg {
        wm::WM_GETDLGCODE => LRESULT(1 | 4 /* DLGC_WANTARROWS | DLGC_WANTALLKEYS (Enter) */),
        wm::WM_LBUTTONDOWN | wm::WM_LBUTTONDBLCLK => {
            unsafe {
                let _ = km::SetFocus(Some(h));
            }
            if let Some(k) = row_at(win::point_of(l)).filter(|k| matches!(d.rows[*k], Row::Item(_))) {
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
                0x25 => step_item(d, now, -1),
                0x27 => step_item(d, now, 1),
                0x26 => line_item(d, now, false),
                0x28 => line_item(d, now, true),
                0x21 => step_item(d, now, -4),
                0x22 => step_item(d, now, 4),
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
        wm::WM_MOUSEMOVE => {
            let hot = row_at(win::point_of(l)).filter(|k| matches!(d.rows[*k], Row::Item(_)));
            if hot != d.hot {
                d.hot = hot;
                unsafe {
                    let mut tme = km::TRACKMOUSEEVENT { cbSize: std::mem::size_of::<km::TRACKMOUSEEVENT>() as u32, dwFlags: km::TME_LEAVE, hwndTrack: h, dwHoverTime: 0 };
                    let _ = km::TrackMouseEvent(&mut tme);
                    let _ = Gdi::InvalidateRect(Some(h), None, false);
                }
            }
            LRESULT(0)
        }
        0x02A3 /* WM_MOUSELEAVE */ => {
            d.hot = None;
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        wm::WM_MOUSEWHEEL => {
            let delta = (w.0 >> 16) as u16 as i16 as i32;
            d.top -= delta * win::px(h, 80) / 120;
            set_scroll(dlg);
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        wm::WM_VSCROLL => {
            let gh = win::px(h, 40);
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
            let c = colours();
            win::fill(dc, &r, c.back);
            let px = |v| win::px(h, v);
            let pad = px(8);
            let radius = px(8);
            let focused = km::GetFocus() == h;
            let (m, inner_pad) = (margin(h), inset(h));
            let (_, th) = thumb_size(h);
            for (k, (row, slot)) in d.rows.iter().zip(layout(d)).enumerate() {
                let y = slot.top - d.top;
                if slot.bottom - d.top < 0 || y > r.bottom {
                    continue;
                }
                let slot = RECT { top: y, bottom: slot.bottom - d.top, ..slot };
                match row {
                    Row::Group(g) => {
                        let old = Gdi::SelectObject(dc, d.bold.into());
                        let mut tr = RECT { left: slot.left + m + px(4), top: y + px(12), right: slot.right - pad, bottom: slot.bottom };
                        win::draw_text(dc, h, t(g), &mut tr, c.accent, Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX);
                        Gdi::SelectObject(dc, old);
                    }
                    Row::Item(tp) => {
                        let on = d.chosen.is_some_and(|x| x.key == tp.key);
                        let hot = d.hot == Some(k);
                        // The card - the chosen one raised on a shadow, framed in the accent.
                        let card = RECT { left: slot.left + m, top: slot.top + m, right: slot.right - m, bottom: slot.bottom - m };
                        if on {
                            shadow(dc, h, card, c.back, radius);
                            let edge = if focused { c.accent } else { c.accent.mix(c.card, 0.3) };
                            win::round_rect(dc, card, edge, edge, radius);
                            let inner = RECT { left: card.left + px(2), top: card.top + px(2), right: card.right - px(2), bottom: card.bottom - px(2) };
                            win::round_rect(dc, inner, c.card, c.card, radius - px(2));
                        } else {
                            win::round_rect(dc, card, c.card, if hot { c.edge.mix(c.text, 0.35) } else { c.edge }, radius);
                        }
                        // Its picture across the top, framed.
                        let tb = RECT { left: card.left + inner_pad, top: card.top + inner_pad, right: card.right - inner_pad, bottom: card.top + inner_pad + th };
                        if let Some((_, b)) = d.thumbs.iter().find(|(key, _)| *key == tp.key) {
                            let at = b.draw_in(dc, tb);
                            frame(dc, at, c.edge);
                        }
                        // Its name under it, in the middle - what it makes is on the right.
                        let old = Gdi::SelectObject(dc, d.bold.into());
                        let mut nr = RECT { left: tb.left + px(2), top: tb.bottom + px(6), right: tb.right - px(2), bottom: card.bottom - px(4) };
                        win::draw_text(dc, h, t(tp.name), &mut nr, c.text, Gdi::DT_SINGLELINE | Gdi::DT_CENTER | Gdi::DT_END_ELLIPSIS | Gdi::DT_NOPREFIX);
                        Gdi::SelectObject(dc, old);
                    }
                }
            }
            if d.rows.is_empty() {
                let old = Gdi::SelectObject(dc, d.font.into());
                let mut tr = RECT { left: pad, top: pad, right: r.right - pad, bottom: r.bottom };
                win::draw_text(dc, h, t("No template has these words."), &mut tr, c.muted, Gdi::DT_WORDBREAK | Gdi::DT_NOPREFIX);
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
        // The grey, the right card with its picture, and a line over the buttons.
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let screen = Gdi::BeginPaint(h, &mut ps);
            let buf = phai_ui::win::off_screen(h, screen);
            let dc = buf.dc;
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            let c = colours();
            win::fill(dc, &r, c.back);
            win::round_rect(dc, card_rect(h), c.card, c.edge, win::px(h, 10));
            if let Some(b) = &d.big_picture {
                let at = b.draw_in(dc, big_rect(h));
                frame(dc, at, c.edge);
            }
            let y = win::px(h, H - 58);
            win::fill(dc, &RECT { left: 0, top: y, right: r.right, bottom: y + 1 }, c.edge);
            drop(buf);
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        wm::WM_ERASEBKGND => LRESULT(1),
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
