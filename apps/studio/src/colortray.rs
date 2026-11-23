//! The colour tray: what a click on a colour opens, under it - in place of Windows' colour
//! dialog. The theme's own colours, shades of the colour now, standard colours, the ones used
//! lately; a box of every shade of one hue with the hue beside it, the colour as `#rrggbb`, and
//! an eyedropper that takes a colour from anywhere on the screen. A swatch clicked is chosen at
//! once; from the box, Enter or OK.

use phai_ui::text::t;
use phai_ui::theme::{self, Color};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging as wm;

const ID_HEX: i32 = 10;
const ID_OK: i32 = 11;
const ID_DROPPER: i32 = 12;
const ID_MORE: i32 = 13;

/// Design pixels.
const W: i32 = 316;
const M: i32 = 10;
const CELL: i32 = 20;
const GAP: i32 = 3;

/// Standard colours: a bright row, then greys - as Office's colour menu has them.
const STANDARD: [u32; 20] = [
    0xC00000, 0xFF0000, 0xFFC000, 0xFFFF00, 0x92D050, 0x00B050, 0x00B0F0, 0x0070C0, 0x002060, 0x7030A0, //
    0xFFFFFF, 0xF2F2F2, 0xD9D9D9, 0xBFBFBF, 0xA6A6A6, 0x808080, 0x595959, 0x404040, 0x262626, 0x000000,
];

fn rgb(v: u32) -> Color {
    Color::rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

pub fn hex(c: Color) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
}

/// Hue 0-360, saturation and value 0-1.
pub fn to_hsv(c: Color) -> (f32, f32, f32) {
    let (r, g, b) = (c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / d).rem_euclid(6.0))
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    (h, if max == 0.0 { 0.0 } else { d / max }, max)
}

pub fn from_hsv(h: f32, s: f32, v: f32) -> Color {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h.rem_euclid(360.0) / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let to = |f: f32| ((f + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    Color::rgb(to(r), to(g), to(b))
}

/// Ten shades of `c`: lighter at the left, `c` in the middle, darker at the right.
pub fn shades(c: Color) -> Vec<Color> {
    let white = Color::rgb(255, 255, 255);
    let black = Color::rgb(0, 0, 0);
    [0.85, 0.7, 0.5, 0.3, 0.0, 0.15, 0.3, 0.45, 0.6, 0.75].iter().enumerate().map(|(i, t)| if i < 4 { c.mix(white, *t) } else { c.mix(black, *t) }).collect()
}

/// The places on the tray, design pixels.
struct Places {
    theme: (i32, i32),
    shades: (i32, i32),
    standard: (i32, i32),
    recent: (i32, i32),
    square: RECT,
    hue: RECT,
    bottom: i32,
}

fn places() -> Places {
    let mut y = M + 18;
    let theme = (M, y);
    y += CELL + GAP + 22;
    let shades = (M, y);
    y += CELL + 22;
    let standard = (M, y);
    y += 2 * (CELL + GAP) + 20;
    let recent = (M, y);
    y += CELL + 14;
    let square = RECT { left: M, top: y, right: W - M - 30, bottom: y + 120 };
    let hue = RECT { left: W - M - 22, top: y, right: W - M, bottom: y + 120 };
    Places { theme, shades, standard, recent, square, hue, bottom: y + 130 }
}

struct Tray {
    /// Theme colours: (name, colour).
    theme: Vec<(String, Color)>,
    recent: Vec<Color>,
    /// The colour as it is, and as it was when the tray opened.
    hsv: (f32, f32, f32),
    was: Option<Color>,
    hex: HWND,
    /// 1 the box, 2 the hue bar, while the button is down there.
    dragging: u8,
    dropper: bool,
    filling: bool,
    chosen: Option<String>,
    done: bool,
}

impl Tray {
    fn colour(&self) -> Color {
        from_hsv(self.hsv.0, self.hsv.1, self.hsv.2)
    }

    fn set(&mut self, h: HWND, c: Color) {
        let (hue, s, v) = to_hsv(c);
        // A grey has no hue of its own: the one shown stays.
        self.hsv = (if s == 0.0 { self.hsv.0 } else { hue }, s, v);
        self.filling = true;
        win::set_text(self.hex, &hex(c));
        self.filling = false;
        unsafe {
            let _ = Gdi::InvalidateRect(Some(h), None, false);
        }
    }

    fn choose(&mut self, c: Color) {
        self.chosen = Some(hex(c));
        self.done = true;
    }
}

fn state<'a>(h: HWND) -> Option<&'a mut Tray> {
    unsafe { win::state::<Tray>(h) }
}

/// The cell of a row of swatches under `(x, y)` (device pixels), if one is there.
fn cell_at(h: HWND, row: (i32, i32), count: usize, x: i32, y: i32) -> Option<usize> {
    let px = |v| win::px(h, v);
    let (rx, ry) = (px(row.0), px(row.1));
    if y < ry || y >= ry + px(CELL) || x < rx {
        return None;
    }
    let k = ((x - rx) / px(CELL + GAP)) as usize;
    (k < count && (x - rx) % px(CELL + GAP) < px(CELL)).then_some(k)
}

/// Opens the tray under `anchor` (screen rectangle of what was clicked), starting at `start`
/// (`#rrggbb`, or anything else for none); the colour chosen, or `None` when it was left.
/// `theme` names the theme's colours shown first.
pub fn choose(owner: HWND, anchor: RECT, start: &str, theme: &[(String, Color)]) -> Option<String> {
    let class = win::register("PhaiColourTray", Some(proc_), None);
    let was = theme::parse_color(start);
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_TOOLWINDOW | wm::WS_EX_TOPMOST, PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_POPUP | wm::WS_BORDER | wm::WS_CLIPCHILDREN, 0, 0, 10, 10, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let px = |v| win::px(hwnd, v);
        let p = places();
        let hex_box = win::edit(hwnd, &was.map(hex).unwrap_or_default(), ID_HEX, font, false);
        win::move_to(hex_box, px(M + 52), px(p.bottom), px(96), px(26));
        // The eyedropper: the icon font's dropper on its face.
        let dropper = win::button(hwnd, "\u{EF3C}", ID_DROPPER, icon_font(hwnd), false);
        win::move_to(dropper, px(M + 154), px(p.bottom), px(32), px(26));
        phai_ui::props::tooltip(dropper, t("Eyedropper: take a colour from anywhere on the screen"));
        let more = win::button(hwnd, "…", ID_MORE, font, false);
        win::move_to(more, px(M + 192), px(p.bottom), px(32), px(26));
        phai_ui::props::tooltip(more, t("More colours: Windows' colour dialog"));
        let ok = win::button(hwnd, t("OK"), ID_OK, font, true);
        win::move_to(ok, px(W - M - 72), px(p.bottom), px(72), px(26));
        let recent: Vec<Color> = crate::setting_pub("recent_colours").unwrap_or_default().split(',').filter_map(theme::parse_color).take(10).collect();
        let start_hsv = was.map(to_hsv).unwrap_or((150.0, 0.85, 0.52));
        win::set_state(hwnd, Box::new(Tray { theme: theme.to_vec(), recent, hsv: start_hsv, was, hex: hex_box, dragging: 0, dropper: false, filling: false, chosen: None, done: false }));
        // Under what was clicked; above it when the screen ends first.
        let (w, h) = (px(W), px(p.bottom + 26 + M));
        let mon = Gdi::MonitorFromPoint(POINT { x: anchor.left, y: anchor.bottom }, Gdi::MONITOR_DEFAULTTONEAREST);
        let mut mi = Gdi::MONITORINFO { cbSize: std::mem::size_of::<Gdi::MONITORINFO>() as u32, ..Default::default() };
        let _ = Gdi::GetMonitorInfoW(mon, &mut mi);
        let x = anchor.left.min(mi.rcWork.right - w).max(mi.rcWork.left);
        let y = if anchor.bottom + h > mi.rcWork.bottom { anchor.top - h } else { anchor.bottom };
        let _ = wm::SetWindowPos(hwnd, None, x, y, w, h, wm::SWP_NOZORDER);
        win::theme_frame(hwnd);
        // `PHAI_TRAY_SHOT`: a picture of the tray, for checking - shown without taking the
        // foreground, photographed, closed.
        if let Ok(out) = std::env::var("PHAI_TRAY_SHOT") {
            let _ = wm::ShowWindow(hwnd, wm::SW_SHOWNOACTIVATE);
            let mut msg = wm::MSG::default();
            while wm::PeekMessageW(&mut msg, None, 0, 0, wm::PM_REMOVE).as_bool() {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
            let _ = phai_ui::shot::save(hwnd, std::path::Path::new(&out));
            let _ = wm::DestroyWindow(hwnd);
            return None;
        }
        let _ = wm::ShowWindow(hwnd, wm::SW_SHOW);
        let _ = km::SetFocus(Some(hex_box));
        let mut msg = wm::MSG::default();
        while state(hwnd).is_some_and(|s| !s.done) {
            if !wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
                wm::PostQuitMessage(msg.wParam.0 as i32);
                break;
            }
            // Enter: the colour as it is; Esc: none.
            if msg.message == wm::WM_KEYDOWN && (msg.wParam.0 == 0x0D || msg.wParam.0 == 0x1B) {
                if let Some(s) = state(hwnd) {
                    if s.dropper {
                        s.dropper = false;
                        let _ = km::ReleaseCapture();
                    } else if msg.wParam.0 == 0x0D {
                        let c = s.colour();
                        s.choose(c);
                    } else {
                        s.done = true;
                    }
                }
                continue;
            }
            if !wm::IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
        }
        let chosen = state(hwnd).and_then(|s| s.chosen.take());
        let _ = wm::DestroyWindow(hwnd);
        // The colours used lately, newest first.
        if let Some(c) = chosen.as_deref() {
            let mut list: Vec<String> = crate::setting_pub("recent_colours").unwrap_or_default().split(',').filter(|v| !v.is_empty() && !v.eq_ignore_ascii_case(c)).map(str::to_string).collect();
            list.insert(0, c.to_string());
            list.truncate(10);
            crate::remember_setting_pub("recent_colours", &list.join(","));
        }
        chosen
    }
}

fn paint(h: HWND) {
    let Some(s) = state(h) else { return };
    let px = |v| win::px(h, v);
    let p = places();
    unsafe {
        let mut ps = Gdi::PAINTSTRUCT::default();
        let screen = Gdi::BeginPaint(h, &mut ps);
        let buf = phai_ui::win::off_screen(h, screen);
        let dc = buf.dc;
        let mut r = RECT::default();
        let _ = wm::GetClientRect(h, &mut r);
        let bg = theme::current().var_color("--surface").unwrap_or(Color::rgb(255, 255, 255));
        let fg = theme::current().var_color("--muted").unwrap_or(Color::rgb(0x5f, 0x5f, 0x5f));
        let line = theme::current().var_color("--border").unwrap_or(Color::rgb(0xd4, 0xd4, 0xd4));
        win::fill(dc, &r, bg);
        let font = win::font(h, 0, false);
        let old = Gdi::SelectObject(dc, font.into());
        Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        Gdi::SetTextColor(dc, COLORREF(fg.colorref()));
        let say = |text: &str, y: i32| {
            let w: Vec<u16> = text.encode_utf16().collect();
            let _ = Gdi::TextOutW(dc, px(M), px(y), &w);
        };
        let swatch = |x: i32, y: i32, c: Color| {
            let cell = RECT { left: px(x), top: px(y), right: px(x + CELL), bottom: px(y + CELL) };
            win::fill(dc, &cell, line);
            win::fill(dc, &RECT { left: cell.left + 1, top: cell.top + 1, right: cell.right - 1, bottom: cell.bottom - 1 }, c);
        };
        let row = |at: (i32, i32), colours: &[Color]| {
            for (k, c) in colours.iter().enumerate() {
                swatch(at.0 + k as i32 * (CELL + GAP), at.1, *c);
            }
        };
        say(t("Theme colours"), p.theme.1 - 18);
        row(p.theme, &s.theme.iter().map(|(_, c)| *c).take(13).collect::<Vec<_>>());
        say(t("Shades of this colour"), p.shades.1 - 18);
        row(p.shades, &shades(s.colour()));
        say(t("Standard colours"), p.standard.1 - 18);
        row(p.standard, &STANDARD[..10].iter().map(|v| rgb(*v)).collect::<Vec<_>>());
        row((p.standard.0, p.standard.1 + CELL + GAP), &STANDARD[10..].iter().map(|v| rgb(*v)).collect::<Vec<_>>());
        say(t("Recent colours"), p.recent.1 - 18);
        if s.recent.is_empty() {
            let w: Vec<u16> = t("none yet").encode_utf16().collect();
            let _ = Gdi::TextOutW(dc, px(M + 110), px(p.recent.1 - 18), &w);
        }
        row(p.recent, &s.recent);
        // The box: saturation across, brightness down, for the hue now; the hue bar beside it.
        let sq = RECT { left: px(p.square.left), top: px(p.square.top), right: px(p.square.right), bottom: px(p.square.bottom) };
        let step = px(4).max(2);
        let (sw, sh) = (sq.right - sq.left, sq.bottom - sq.top);
        let mut y = 0;
        while y < sh {
            let mut x = 0;
            while x < sw {
                let c = from_hsv(s.hsv.0, x as f32 / sw as f32, 1.0 - y as f32 / sh as f32);
                win::fill(dc, &RECT { left: sq.left + x, top: sq.top + y, right: (sq.left + x + step).min(sq.right), bottom: (sq.top + y + step).min(sq.bottom) }, c);
                x += step;
            }
            y += step;
        }
        let hb = RECT { left: px(p.hue.left), top: px(p.hue.top), right: px(p.hue.right), bottom: px(p.hue.bottom) };
        let hh = hb.bottom - hb.top;
        let mut y = 0;
        while y < hh {
            win::fill(dc, &RECT { left: hb.left, top: hb.top + y, right: hb.right, bottom: (hb.top + y + 2).min(hb.bottom) }, from_hsv(360.0 * y as f32 / hh as f32, 1.0, 1.0));
            y += 2;
        }
        // Where the colour is: a ring in the box, a bar across the hue.
        let (cx, cy) = (sq.left + (s.hsv.1 * sw as f32) as i32, sq.top + ((1.0 - s.hsv.2) * sh as f32) as i32);
        let ring = if s.hsv.2 > 0.6 && s.hsv.1 < 0.5 { Color::rgb(0, 0, 0) } else { Color::rgb(255, 255, 255) };
        let pen = Gdi::CreatePen(Gdi::PS_SOLID, px(2).max(1), COLORREF(ring.colorref()));
        let op = Gdi::SelectObject(dc, pen.into());
        let ob = Gdi::SelectObject(dc, Gdi::GetStockObject(Gdi::NULL_BRUSH));
        let rr = px(5);
        let _ = Gdi::Ellipse(dc, cx - rr, cy - rr, cx + rr, cy + rr);
        let hy = hb.top + (s.hsv.0 / 360.0 * hh as f32) as i32;
        let _ = Gdi::Rectangle(dc, hb.left - 1, hy - px(2), hb.right + 1, hy + px(2));
        Gdi::SelectObject(dc, ob);
        Gdi::SelectObject(dc, op);
        let _ = Gdi::DeleteObject(pen.into());
        // As it was (left half) and as it is (right half).
        let b = RECT { left: px(M), top: px(p.bottom), right: px(M + 44), bottom: px(p.bottom + 26) };
        win::fill(dc, &b, line);
        let mid = (b.left + b.right) / 2;
        win::fill(dc, &RECT { left: b.left + 1, top: b.top + 1, right: mid, bottom: b.bottom - 1 }, s.was.unwrap_or(s.colour()));
        win::fill(dc, &RECT { left: mid, top: b.top + 1, right: b.right - 1, bottom: b.bottom - 1 }, s.colour());
        Gdi::SelectObject(dc, old);
        drop(buf);
        let _ = Gdi::EndPaint(h, &ps);
    }
}

/// Windows' icon font at the window's text size.
fn icon_font(h: HWND) -> Gdi::HFONT {
    let mut lf = Gdi::LOGFONTW { lfHeight: -win::px(h, 14), lfCharSet: Gdi::DEFAULT_CHARSET, lfQuality: Gdi::CLEARTYPE_QUALITY, ..Default::default() };
    for (i, u) in "Segoe MDL2 Assets".encode_utf16().enumerate() {
        lf.lfFaceName[i] = u;
    }
    unsafe { Gdi::CreateFontIndirectW(&lf) }
}

/// The colour on the screen under the pointer.
fn screen_colour() -> Option<Color> {
    unsafe {
        let mut pt = POINT::default();
        let _ = wm::GetCursorPos(&mut pt);
        let dc = Gdi::GetDC(None);
        let v = Gdi::GetPixel(dc, pt.x, pt.y).0;
        Gdi::ReleaseDC(None, dc);
        (v != 0xFFFF_FFFF).then(|| Color::rgb((v & 0xFF) as u8, ((v >> 8) & 0xFF) as u8, ((v >> 16) & 0xFF) as u8))
    }
}

/// A press or a drag in the box or the hue bar.
fn drag_to(h: HWND, x: i32, y: i32) {
    let Some(s) = state(h) else { return };
    let px = |v| win::px(h, v);
    let p = places();
    match s.dragging {
        1 => {
            let (l, t_, r, b) = (px(p.square.left), px(p.square.top), px(p.square.right), px(p.square.bottom));
            s.hsv.1 = ((x - l) as f32 / (r - l) as f32).clamp(0.0, 1.0);
            s.hsv.2 = 1.0 - ((y - t_) as f32 / (b - t_) as f32).clamp(0.0, 1.0);
        }
        2 => {
            let (t_, b) = (px(p.hue.top), px(p.hue.bottom));
            s.hsv.0 = (360.0 * (y - t_) as f32 / (b - t_) as f32).clamp(0.0, 359.9);
        }
        _ => return,
    }
    s.filling = true;
    win::set_text(s.hex, &hex(s.colour()));
    s.filling = false;
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(s) = state(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_PAINT => {
            paint(h);
            LRESULT(0)
        }
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_LBUTTONDOWN => {
            let (x, y) = win::point_of(l);
            if s.dropper {
                s.dropper = false;
                unsafe {
                    let _ = km::ReleaseCapture();
                }
                if let Some(c) = screen_colour() {
                    s.set(h, c);
                }
                return LRESULT(0);
            }
            let p = places();
            let px = |v| win::px(h, v);
            let theme: Vec<Color> = s.theme.iter().map(|(_, c)| *c).take(13).collect();
            let shades = shades(s.colour());
            let rows: [((i32, i32), Vec<Color>); 4] = [
                (p.theme, theme),
                (p.shades, shades),
                (p.standard, STANDARD[..10].iter().map(|v| rgb(*v)).collect()),
                ((p.standard.0, p.standard.1 + CELL + GAP), STANDARD[10..].iter().map(|v| rgb(*v)).collect()),
            ];
            for (at, colours) in rows.iter().chain(std::iter::once(&(p.recent, s.recent.clone()))) {
                if let Some(k) = cell_at(h, *at, colours.len(), x, y) {
                    s.choose(colours[k]);
                    return LRESULT(0);
                }
            }
            let inside = |r: &RECT| x >= px(r.left) && x < px(r.right) && y >= px(r.top) && y < px(r.bottom);
            s.dragging = if inside(&p.square) { 1 } else if inside(&p.hue) { 2 } else { 0 };
            if s.dragging != 0 {
                unsafe {
                    let _ = km::SetCapture(h);
                }
                drag_to(h, x, y);
            }
            LRESULT(0)
        }
        wm::WM_MOUSEMOVE => {
            if s.dropper {
                if let Some(c) = screen_colour() {
                    s.set(h, c);
                }
            } else {
                let (x, y) = win::point_of(l);
                drag_to(h, x, y);
            }
            LRESULT(0)
        }
        wm::WM_LBUTTONUP => {
            if s.dragging != 0 {
                s.dragging = 0;
                unsafe {
                    let _ = km::ReleaseCapture();
                }
            }
            LRESULT(0)
        }
        wm::WM_SETCURSOR if s.dropper => {
            unsafe {
                let _ = wm::SetCursor(wm::LoadCursorW(None, wm::IDC_CROSS).ok());
            }
            LRESULT(1)
        }
        wm::WM_COMMAND => {
            match (win::loword(w.0) as i32, win::hiword(w.0)) {
                (ID_OK, _) => {
                    let c = s.colour();
                    s.choose(c);
                }
                (ID_DROPPER, _) => {
                    s.dropper = true;
                    unsafe {
                        let _ = km::SetCapture(h);
                        let _ = wm::SetCursor(wm::LoadCursorW(None, wm::IDC_CROSS).ok());
                    }
                }
                (ID_MORE, _) => {
                    let start = win::text_of(s.hex);
                    if let Some(c) = crate::themeedit::choose_colour(h, &start) {
                        s.chosen = Some(c);
                        s.done = true;
                    }
                }
                (ID_HEX, 0x0300 /* EN_CHANGE */) if !s.filling => {
                    if let Some(c) = theme::parse_color(win::text_of(s.hex).trim()) {
                        let (hue, sat, v) = to_hsv(c);
                        s.hsv = (if sat == 0.0 { s.hsv.0 } else { hue }, sat, v);
                        unsafe {
                            let _ = Gdi::InvalidateRect(Some(h), None, false);
                        }
                    }
                }
                _ => return win::def(h, msg, w, l),
            }
            LRESULT(0)
        }
        // Clicked away: left, as a menu is.
        wm::WM_ACTIVATE if win::loword(w.0) == 0 && !s.dropper => {
            s.done = true;
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            drop(unsafe { win::take_state::<Tray>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_go_to_hsv_and_back() {
        for c in [Color::rgb(0x12, 0x85, 0x5f), Color::rgb(255, 0, 0), Color::rgb(0, 0, 0), Color::rgb(200, 200, 200), Color::rgb(0x1c, 0x6f, 0xd1)] {
            let (h, s, v) = to_hsv(c);
            let back = from_hsv(h, s, v);
            assert!((back.r as i32 - c.r as i32).abs() <= 1 && (back.g as i32 - c.g as i32).abs() <= 1 && (back.b as i32 - c.b as i32).abs() <= 1, "{c:?} -> {back:?}");
        }
        let sh = shades(Color::rgb(0x12, 0x85, 0x5f));
        assert_eq!(sh.len(), 10);
        assert_eq!(sh[4], Color::rgb(0x12, 0x85, 0x5f), "the colour itself in the middle");
        assert!(sh[0].luma() > sh[4].luma() && sh[9].luma() < sh[4].luma());
    }
}
