//! View > Device Preview: the form as a phone and a tablet would show it - their screens' sizes,
//! turned on their side, or several side by side, as Android Studio's previews and Xcode's
//! canvas do. The form is built alive on a window of each screen's size kept off the screen, laid
//! out by its anchors, docks and flows, and its picture drawn here in a device's frame - again
//! whenever the form changes, so the designer's work shows on every screen as it is done.

use phai_form::{live, FormFile};
use phai_ui::text::{t, tf};
use phai_ui::theme::{self, Color};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS};
use windows::Win32::UI::WindowsAndMessaging as wm;

const ID_DEVICE: i32 = 10;
const ID_ROTATE: i32 = 11;
const ID_SIDE: i32 = 12;
const TIMER: usize = 1;
/// Soon after the screens are built: their first pictures, once Windows has drawn them.
const TIMER_FIRST: usize = 2;

/// The screens, in design pixels (CSS pixels on the web): the size a page sees.
pub const DEVICES: [(&str, i32, i32); 4] = [("Phone", 390, 844), ("Small phone", 360, 640), ("Tablet", 820, 1180), ("Desktop", 1280, 800)];

/// Design pixels.
const BAR: i32 = 44;
const BEZEL: i32 = 14;
const LABEL: i32 = 26;
const GAP: i32 = 28;

/// One screen: the window the form is built on, and its last picture.
struct Screen {
    host: HWND,
    live: live::Live,
    /// Design pixels, turned already.
    size: (i32, i32),
    name: String,
    picture: Option<(Gdi::HBITMAP, i32, i32)>,
}

struct Devices {
    combo: HWND,
    rotate: HWND,
    side: HWND,
    font: Gdi::HFONT,
    /// The form as last built, to see when it changed.
    built: String,
    device: usize,
    turned: bool,
    side_by_side: bool,
    screens: Vec<Screen>,
}

/// The size of `device` turned or not.
pub fn screen_size(device: usize, turned: bool) -> (i32, i32) {
    let (_, w, h) = DEVICES[device.min(DEVICES.len() - 1)];
    if turned {
        (h, w)
    } else {
        (w, h)
    }
}

/// The screens to show: one device, or side by side the phone upright and on its side, and the tablet.
pub fn screens_of(device: usize, turned: bool, side_by_side: bool) -> Vec<(usize, bool)> {
    if side_by_side {
        vec![(0, false), (0, true), (2, false)]
    } else {
        vec![(device, turned)]
    }
}

/// Where the screens go in a window `area` wide and high (pixels), and how much they are made
/// smaller to fit: each frame's rectangle and the scale.
pub fn place(sizes: &[(i32, i32)], area: (i32, i32), bezel: i32, label: i32, gap: i32) -> (Vec<RECT>, f32) {
    let total_w: i32 = sizes.iter().map(|s| s.0 + 2 * bezel).sum::<i32>() + gap * (sizes.len() as i32 - 1).max(0);
    let max_h = sizes.iter().map(|s| s.1 + 2 * bezel).max().unwrap_or(1);
    let scale = ((area.0 - 2 * gap) as f32 / total_w as f32).min((area.1 - 2 * gap - label) as f32 / max_h as f32).clamp(0.1, 1.0);
    let used_w = (total_w as f32 * scale) as i32;
    let mut x = (area.0 - used_w) / 2;
    let mut out = Vec::new();
    for s in sizes {
        let w = ((s.0 + 2 * bezel) as f32 * scale) as i32;
        let h = ((s.1 + 2 * bezel) as f32 * scale) as i32;
        let y = gap + ((max_h as f32 * scale) as i32 - h) / 2;
        out.push(RECT { left: x, top: y, right: x + w, bottom: y + h });
        x += w + (gap as f32 * scale.max(0.5)) as i32;
    }
    (out, scale)
}

/// The window, filled, not yet shown - for [`open`], and for a picture in a test.
pub fn make(owner: HWND, form: &FormFile, device: usize, turned: bool, side_by_side: bool) -> HWND {
    win::init();
    let class = win::register("PhaiDevicePreview", Some(proc_), None);
    let title = win::wide(t("Device Preview"));
    unsafe {
        let hwnd = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_OVERLAPPEDWINDOW | wm::WS_CLIPCHILDREN, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let names: Vec<String> = DEVICES.iter().map(|(n, w, h)| format!("{} ({w} × {h})", t(n))).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        let combo = win::combo(hwnd, ID_DEVICE, font, &refs);
        wm::SendMessageW(combo, wm::CB_SETCURSEL, Some(WPARAM(device)), None);
        let rotate = win::button(hwnd, t("Rotate"), ID_ROTATE, font, false);
        let side = win::checkbox(hwnd, t("Phone, turned and tablet side by side"), ID_SIDE, font, side_by_side);
        win::set_state(hwnd, Box::new(Devices { combo, rotate, side, font, built: String::new(), device, turned, side_by_side, screens: Vec::new() }));
        rebuild(hwnd, form);
        // Most of the screen: devices are tall.
        let mut work = RECT::default();
        let _ = wm::SystemParametersInfoW(wm::SPI_GETWORKAREA, 0, Some(&mut work as *mut _ as *mut _), Default::default());
        let (w, h) = (((work.right - work.left) * 3 / 4).max(win::px(hwnd, 700)), (work.bottom - work.top) * 9 / 10);
        let _ = wm::SetWindowPos(hwnd, None, work.left + (work.right - work.left - w) / 2, work.top + (work.bottom - work.top - h) / 2, w, h, wm::SWP_NOZORDER);
        win::theme_frame(hwnd);
        wm::SetTimer(Some(hwnd), TIMER, 700, None);
        hwnd
    }
}

/// Opens the window for the form being designed.
pub fn open(owner: HWND, form: &FormFile) {
    let hwnd = make(owner, form, 0, false, false);
    win::show_top(hwnd);
}

/// The screens built again for `form`, as the device and its turn now say.
fn rebuild(hwnd: HWND, form: &FormFile) {
    let Some(d) = (unsafe { win::state::<Devices>(hwnd) }) else { return };
    for s in d.screens.drain(..) {
        drop_screen(s);
    }
    let class = win::register("PhaiDeviceScreen", Some(screen_proc), None);
    for (device, turned) in screens_of(d.device, d.turned, d.side_by_side) {
        let size = screen_size(device, turned);
        unsafe {
            // Off the screen, shown without taking the focus: drawn, never seen there.
            let host = wm::CreateWindowExW(wm::WS_EX_TOOLWINDOW | wm::WS_EX_NOACTIVATE, PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_POPUP | wm::WS_CLIPCHILDREN, -32000, -32000, win::px(hwnd, size.0), win::px(hwnd, size.1), Some(hwnd), None, Some(win::instance()), None).unwrap_or_default();
            let live = live::build(host, form, d.font, 0);
            live.layout.apply(host);
            let _ = wm::ShowWindow(host, wm::SW_SHOWNOACTIVATE);
            let (n, _, _) = DEVICES[device];
            let name = if turned { tf("{0}, turned", &[&t(n)]) } else { t(n).to_string() };
            d.screens.push(Screen { host, live, size, name, picture: None });
        }
    }
    d.built = form.to_toml();
    photograph(hwnd);
    unsafe {
        wm::SetTimer(Some(hwnd), TIMER_FIRST, 150, None);
    }
}

fn drop_screen(s: Screen) {
    unsafe {
        if let Some((b, _, _)) = s.picture {
            let _ = Gdi::DeleteObject(b.into());
        }
        let _ = wm::DestroyWindow(s.host);
    }
    drop(s.live);
}

/// Each screen's picture taken again.
fn photograph(hwnd: HWND) {
    let Some(d) = (unsafe { win::state::<Devices>(hwnd) }) else { return };
    const PW_RENDERFULLCONTENT: u32 = 2;
    unsafe {
        let screen_dc = Gdi::GetDC(None);
        for s in &mut d.screens {
            let mut r = RECT::default();
            let _ = wm::GetClientRect(s.host, &mut r);
            let (w, h) = (r.right, r.bottom);
            let dc = Gdi::CreateCompatibleDC(Some(screen_dc));
            let bmp = Gdi::CreateCompatibleBitmap(screen_dc, w, h);
            let old = Gdi::SelectObject(dc, bmp.into());
            let _ = Gdi::RedrawWindow(Some(s.host), None, None, Gdi::RDW_INVALIDATE | Gdi::RDW_UPDATENOW | Gdi::RDW_ALLCHILDREN);
            let _ = PrintWindow(s.host, dc, PRINT_WINDOW_FLAGS(PW_RENDERFULLCONTENT));
            Gdi::SelectObject(dc, old);
            let _ = Gdi::DeleteDC(dc);
            if let Some((b, _, _)) = s.picture.replace((bmp, w, h)) {
                let _ = Gdi::DeleteObject(b.into());
            }
        }
        Gdi::ReleaseDC(None, screen_dc);
        let _ = Gdi::InvalidateRect(Some(hwnd), None, false);
    }
}

fn layout_bar(hwnd: HWND, d: &Devices, cw: i32) {
    let px = |v| win::px(hwnd, v);
    win::move_to(d.combo, px(12), px(10), px(230), px(300));
    win::move_to(d.rotate, px(250), px(9), px(90), px(26));
    win::move_to(d.side, px(352), px(9), (cw - px(364)).max(px(100)), px(26));
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(d) = (unsafe { win::state::<Devices>(h) }) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            layout_bar(h, d, win::loword(l.0 as usize) as i32);
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        wm::WM_TIMER if w.0 == TIMER_FIRST => {
            unsafe {
                let _ = wm::KillTimer(Some(h), TIMER_FIRST);
            }
            photograph(h);
            LRESULT(0)
        }
        wm::WM_TIMER => {
            // The form changed in the designer: built again; else only photographed again,
            // for what changed on its own (a clock, a caret).
            if let Some(s) = crate::studio() {
                if s.doc.code.is_none() && s.doc.form.to_toml() != d.built {
                    let form = s.doc.form.clone();
                    rebuild(h, &form);
                    return LRESULT(0);
                }
            }
            photograph(h);
            LRESULT(0)
        }
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut rc = RECT::default();
            let _ = wm::GetClientRect(h, &mut rc);
            let cur = theme::current();
            let bg = cur.var_color("--canvas").or_else(|| cur.var_color("--surface-alt")).unwrap_or(win::window_background_for(h).mix(Color::rgb(128, 128, 128), 0.12));
            // Drawn on a bitmap first, so it does not flicker.
            let mem = Gdi::CreateCompatibleDC(Some(dc));
            let buf = Gdi::CreateCompatibleBitmap(dc, rc.right, rc.bottom);
            let old = Gdi::SelectObject(mem, buf.into());
            let bar = win::px(h, BAR);
            win::fill(mem, &RECT { bottom: bar, ..rc }, win::window_background_for(h));
            win::fill(mem, &RECT { top: bar, ..rc }, bg);
            let sizes: Vec<(i32, i32)> = d.screens.iter().map(|s| (win::px(h, s.size.0), win::px(h, s.size.1))).collect();
            let (frames, _) = place(&sizes, (rc.right, rc.bottom - bar), win::px(h, BEZEL), win::px(h, LABEL), win::px(h, GAP));
            let text = win::window_text_for(h);
            Gdi::SetStretchBltMode(mem, Gdi::HALFTONE);
            for (s, f) in d.screens.iter().zip(frames) {
                let f = RECT { top: f.top + bar, bottom: f.bottom + bar, ..f };
                let scale = (f.right - f.left) as f32 / (win::px(h, s.size.0 + 2 * BEZEL)) as f32;
                let bz = (win::px(h, BEZEL) as f32 * scale).max(4.0) as i32;
                win::round_rect(mem, f, Color::rgb(0x22, 0x22, 0x26), Color::rgb(0x55, 0x55, 0x5c), (win::px(h, 34) as f32 * scale).max(8.0) as i32);
                let inner = RECT { left: f.left + bz, top: f.top + bz, right: f.right - bz, bottom: f.bottom - bz };
                win::fill(mem, &inner, win::window_background_for(h));
                if let Some((b, bw, bh)) = s.picture {
                    let src = Gdi::CreateCompatibleDC(Some(mem));
                    let o = Gdi::SelectObject(src, b.into());
                    let _ = Gdi::StretchBlt(mem, inner.left, inner.top, inner.right - inner.left, inner.bottom - inner.top, Some(src), 0, 0, bw, bh, Gdi::SRCCOPY);
                    Gdi::SelectObject(src, o);
                    let _ = Gdi::DeleteDC(src);
                }
                let old_font = Gdi::SelectObject(mem, d.font.into());
                let mut lr = RECT { left: f.left - win::px(h, 40), top: f.bottom + win::px(h, 6), right: f.right + win::px(h, 40), bottom: f.bottom + win::px(h, 6 + LABEL) };
                win::draw_text(mem, h, &format!("{}  {} × {}", s.name, s.size.0, s.size.1), &mut lr, text, Gdi::DT_CENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX);
                Gdi::SelectObject(mem, old_font);
            }
            let _ = Gdi::BitBlt(dc, 0, 0, rc.right, rc.bottom, Some(mem), 0, 0, Gdi::SRCCOPY);
            Gdi::SelectObject(mem, old);
            let _ = Gdi::DeleteObject(buf.into());
            let _ = Gdi::DeleteDC(mem);
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_COMMAND => {
            let id = win::loword(w.0) as i32;
            let code = win::hiword(w.0) as u32;
            let changed = match id {
                ID_DEVICE if code == wm::CBN_SELCHANGE => {
                    d.device = win::combo_selected(d.combo).max(0) as usize;
                    true
                }
                ID_ROTATE => {
                    d.turned = !d.turned;
                    true
                }
                ID_SIDE => {
                    d.side_by_side = win::is_checked(d.side);
                    win::enable(d.combo, !d.side_by_side);
                    win::enable(d.rotate, !d.side_by_side);
                    true
                }
                _ => false,
            };
            if changed {
                let form = crate::studio().map(|s| s.doc.form.clone()).unwrap_or_else(|| phai_form::FormFile::new("MainWindow"));
                rebuild(h, &form);
                return LRESULT(0);
            }
            win::def(h, msg, w, l)
        }
        wm::WM_KEYDOWN if w.0 == 0x1B => unsafe {
            let _ = wm::DestroyWindow(h);
            LRESULT(0)
        },
        wm::WM_DESTROY => unsafe {
            let _ = wm::KillTimer(Some(h), TIMER);
            let _ = wm::KillTimer(Some(h), TIMER_FIRST);
            LRESULT(0)
        },
        wm::WM_NCDESTROY => {
            if let Some(mut d) = unsafe { win::take_state::<Devices>(h) } {
                for s in d.screens.drain(..) {
                    drop_screen(s);
                }
                unsafe {
                    let _ = Gdi::DeleteObject(d.font.into());
                }
            }
            win::def(h, msg, w, l)
        }
        _ => win::ctl_color(msg, w, l).unwrap_or_else(|| win::def(h, msg, w, l)),
    }
}

/// A screen's window: laid out again when sized; its background the form's.
extern "system" fn screen_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_ERASEBKGND => unsafe {
            let mut rc = RECT::default();
            let _ = wm::GetClientRect(h, &mut rc);
            win::fill(Gdi::HDC(w.0 as *mut _), &rc, win::window_background_for(h));
            LRESULT(1)
        },
        wm::WM_MOUSEACTIVATE => LRESULT(wm::MA_NOACTIVATE as isize),
        _ => win::ctl_color(msg, w, l).unwrap_or_else(|| win::def(h, msg, w, l)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screens_turned_and_side_by_side() {
        assert_eq!(screen_size(0, false), (390, 844));
        assert_eq!(screen_size(0, true), (844, 390));
        assert_eq!(screens_of(2, true, false), vec![(2, true)]);
        assert_eq!(screens_of(2, true, true).len(), 3);
    }

    #[test]
    fn screens_made_smaller_to_fit_never_larger() {
        let (frames, scale) = place(&[(390, 844)], (2000, 2000), 14, 26, 28);
        assert_eq!(scale, 1.0);
        assert_eq!(frames[0].right - frames[0].left, 390 + 28);
        let (frames, scale) = place(&[(390, 844), (844, 390), (820, 1180)], (1200, 700), 14, 26, 28);
        assert!(scale < 1.0);
        assert!(frames.iter().all(|f| f.left >= 0 && f.right <= 1200 && f.bottom <= 700), "{frames:?}");
        assert!(frames.windows(2).all(|p| p[0].right < p[1].left), "side by side, apart");
    }
}
