//! The two small windows an editor opens beside its text: a tip (a hover's words, a diagnostic's
//! message) and the completion list. Neither takes the focus - typing stays in the editor, which
//! moves the list's choice with the arrow keys.

use phai_ui::theme::{self, Color, State};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HFONT};
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HMENU};

use crate::lsp::Completion;

pub struct Popup {
    pub hwnd: HWND,
    font: HFONT,
    /// A tip's lines, or nothing for a list.
    lines: Vec<String>,
    pub items: Vec<Completion>,
    pub chosen: usize,
    /// The choice is only a suggestion - the list opened by itself at a call's argument, nothing
    /// typed or moved to yet: Enter is a new line, not this choice; it is framed, not filled.
    pub soft: bool,
    row_h: i32,
    /// Its font is its own, made for it, deleted with it.
    own_font: bool,
}

const MAX_ROWS: usize = 10;

fn make(owner: HWND) -> HWND {
    let class = win::register("PhaiEditorPopup", Some(proc_), None);
    unsafe {
        let top = wm::GetAncestor(owner, wm::GA_ROOT);
        wm::CreateWindowExW(wm::WS_EX_NOACTIVATE | wm::WS_EX_TOPMOST | wm::WS_EX_TOOLWINDOW, PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_POPUP | wm::WS_BORDER, 0, 0, 10, 10, Some(top), Some(HMENU::default()), Some(win::instance()), None).unwrap_or_default()
    }
}

impl Popup {
    /// A tip with `text` below screen point `(x, y)`.
    pub fn tip(owner: HWND, font: HFONT, text: &str, x: i32, y: i32) -> Box<Popup> {
        let lines: Vec<String> = text.lines().take(14).map(|l| l.chars().take(110).collect()).collect();
        let p = Popup::new(owner, font, lines, vec![]);
        p.place(x, y);
        p
    }

    /// A tip whose bottom is at screen point `(x, y)` - Parameter Info above the line typed on,
    /// out of the way of the list that opens below it.
    pub fn tip_above(owner: HWND, font: HFONT, text: &str, x: i32, y: i32) -> Box<Popup> {
        let p = Popup::tip(owner, font, text, x, y);
        unsafe {
            let mut r = windows::Win32::Foundation::RECT::default();
            let _ = wm::GetWindowRect(p.hwnd, &mut r);
            let top = (y - (r.bottom - r.top)).max(0);
            let _ = wm::SetWindowPos(p.hwnd, Some(wm::HWND_TOPMOST), r.left, top, 0, 0, wm::SWP_NOACTIVATE | wm::SWP_NOSIZE);
        }
        p
    }

    /// A completion list below screen point `(x, y)`.
    /// In Windows' own letters (Segoe UI 9 pt), smaller than the code's, as Visual Studio's
    /// IntelliSense list; `font` is the code's, for nothing but the call.
    pub fn list(owner: HWND, font: HFONT, items: Vec<Completion>, x: i32, y: i32) -> Box<Popup> {
        let _ = font;
        let mut p = Popup::new(owner, win::font(owner, 0, false), vec![], items);
        p.own_font = true;
        p.place(x, y);
        p
    }

    fn new(owner: HWND, font: HFONT, lines: Vec<String>, items: Vec<Completion>) -> Box<Popup> {
        let hwnd = make(owner);
        let row_h = win::measure(hwnd, font, "Ag").1 + win::px(hwnd, 6);
        let b = Box::new(Popup { hwnd, font, lines, items, chosen: 0, soft: false, row_h, own_font: false });
        unsafe {
            wm::SetWindowLongPtrW(hwnd, wm::GWLP_USERDATA, &*b as *const Popup as isize);
        }
        b
    }

    fn rows(&self) -> Vec<String> {
        if self.items.is_empty() {
            self.lines.clone()
        } else {
            self.items.iter().take(MAX_ROWS).map(|c| c.label.clone()).collect()
        }
    }

    /// Sizes itself to its rows and shows below `(x, y)`, kept on the screen.
    pub fn place(&self, x: i32, y: i32) {
        let rows = self.rows();
        let pad = win::px(self.hwnd, 8);
        let label_w = rows.iter().map(|r| win::measure(self.hwnd, self.font, r).0).max().unwrap_or(0);
        // Room for the longest detail beside the longest name, up to a reasonable width; what is
        // longer still is cut with ... rather than drawn over the name.
        let detail_w = self.items.iter().take(MAX_ROWS).map(|c| win::measure(self.hwnd, self.font, &detail_of(c)).0).max().unwrap_or(0);
        let gap = if detail_w > 0 { win::px(self.hwnd, 24) } else { 0 };
        let w = (label_w + gap + detail_w + 2 * pad).min(win::px(self.hwnd, 760)).max(label_w + 2 * pad);
        let h = rows.len().max(1) as i32 * self.row_h + pad;
        unsafe {
            let mon = Gdi::MonitorFromPoint(windows::Win32::Foundation::POINT { x, y }, Gdi::MONITOR_DEFAULTTONEAREST);
            let mut mi = Gdi::MONITORINFO { cbSize: std::mem::size_of::<Gdi::MONITORINFO>() as u32, ..Default::default() };
            let _ = Gdi::GetMonitorInfoW(mon, &mut mi);
            let x = x.min(mi.rcWork.right - w).max(mi.rcWork.left);
            let y = if y + h > mi.rcWork.bottom { y - h - self.row_h } else { y };
            let _ = wm::SetWindowPos(self.hwnd, Some(wm::HWND_TOPMOST), x, y, w, h, wm::SWP_NOACTIVATE | wm::SWP_SHOWWINDOW);
            let _ = Gdi::InvalidateRect(Some(self.hwnd), None, true);
        }
    }

    pub fn move_choice(&mut self, by: i32) {
        if self.items.is_empty() {
            return;
        }
        let n = self.items.len().min(MAX_ROWS) as i32;
        self.chosen = ((self.chosen as i32 + by).rem_euclid(n)) as usize;
        self.soft = false;
        unsafe {
            let _ = Gdi::InvalidateRect(Some(self.hwnd), None, true);
        }
    }

    pub fn choice(&self) -> Option<&Completion> {
        self.items.get(self.chosen)
    }
}

impl Drop for Popup {
    fn drop(&mut self) {
        unsafe {
            wm::SetWindowLongPtrW(self.hwnd, wm::GWLP_USERDATA, 0);
            let _ = wm::DestroyWindow(self.hwnd);
            if self.own_font {
                let _ = Gdi::DeleteObject(self.font.into());
            }
        }
    }
}

/// A completion's detail as the list shows it: up to 60 letters.
fn detail_of(c: &Completion) -> String {
    let n = c.detail.chars().count();
    if n > 60 { format!("{}...", c.detail.chars().take(57).collect::<String>()) } else { c.detail.clone() }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let p = unsafe { (wm::GetWindowLongPtrW(h, wm::GWLP_USERDATA) as *const Popup).as_ref() };
    match (msg, p) {
        (wm::WM_MOUSEACTIVATE, _) => LRESULT(3), // MA_NOACTIVATE
        (wm::WM_ERASEBKGND, _) => LRESULT(1),
        (wm::WM_PAINT, Some(p)) => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            let bg = theme::style("edit", State::default()).background_or(win::window_background());
            win::fill(dc, &r, bg);
            let chosen_bg = theme::style("list", State { selected: true, ..Default::default() }).background_or(Color::rgb(0xe7, 0xf3, 0xef));
            let muted = theme::current().var_color("--muted").unwrap_or(Color::rgb(0x80, 0x80, 0x80));
            let pad = win::px(h, 8);
            let old = Gdi::SelectObject(dc, p.font.into());
            Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
            for (i, row) in p.rows().iter().enumerate() {
                let y = pad / 2 + i as i32 * p.row_h;
                let line = RECT { left: 0, top: y, right: r.right, bottom: y + p.row_h };
                if !p.items.is_empty() && i == p.chosen {
                    if p.soft {
                        let one = win::px(h, 1);
                        for edge in [RECT { bottom: line.top + one, ..line }, RECT { top: line.bottom - one, ..line }, RECT { right: line.left + one, ..line }, RECT { left: line.right - one, ..line }] {
                            win::fill(dc, &edge, muted);
                        }
                    } else {
                        win::fill(dc, &line, chosen_bg);
                    }
                }
                let mut tr = RECT { left: pad, ..line };
                let label_w = win::measure(h, p.font, row).0;
                let mut words: Vec<u16> = row.encode_utf16().collect();
                Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(win::window_text().colorref()));
                if !words.is_empty() {
                    phai_ui::win::draw_text_w(dc, &mut words, &mut tr, Gdi::DT_LEFT | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX);
                }
                // A completion's detail, muted, at the right.
                if let Some(c) = p.items.get(i) {
                    if !c.detail.is_empty() {
                        let mut d: Vec<u16> = detail_of(c).encode_utf16().collect();
                        // After the name, never over it.
                        let mut dr = RECT { left: pad + label_w + win::px(h, 24), right: r.right - pad, ..line };
                        if dr.right <= dr.left {
                            continue;
                        }
                        Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(muted.colorref()));
                        phai_ui::win::draw_text_w(dc, &mut d, &mut dr, Gdi::DT_RIGHT | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX | Gdi::DT_END_ELLIPSIS);
                    }
                }
            }
            Gdi::SelectObject(dc, old);
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        _ => win::def(h, msg, w, l),
    }
}
