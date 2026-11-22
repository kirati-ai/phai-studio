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
    row_h: i32,
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

    /// A completion list below screen point `(x, y)`.
    pub fn list(owner: HWND, font: HFONT, items: Vec<Completion>, x: i32, y: i32) -> Box<Popup> {
        let p = Popup::new(owner, font, vec![], items);
        p.place(x, y);
        p
    }

    fn new(owner: HWND, font: HFONT, lines: Vec<String>, items: Vec<Completion>) -> Box<Popup> {
        let hwnd = make(owner);
        let row_h = win::measure(hwnd, font, "Ag").1 + win::px(hwnd, 6);
        let b = Box::new(Popup { hwnd, font, lines, items, chosen: 0, row_h });
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
        let w = rows.iter().map(|r| win::measure(self.hwnd, self.font, r).0).max().unwrap_or(0) + 2 * pad + if self.items.is_empty() { 0 } else { win::px(self.hwnd, 160) };
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
        }
    }
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
                    win::fill(dc, &line, chosen_bg);
                }
                let mut tr = RECT { left: pad, ..line };
                let mut words: Vec<u16> = row.encode_utf16().collect();
                Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(win::window_text().colorref()));
                if !words.is_empty() {
                    phai_ui::win::draw_text_w(dc, &mut words, &mut tr, Gdi::DT_LEFT | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX);
                }
                // A completion's detail, muted, at the right.
                if let Some(c) = p.items.get(i) {
                    if !c.detail.is_empty() {
                        let mut d: Vec<u16> = c.detail.chars().take(40).collect::<String>().encode_utf16().collect();
                        let mut dr = RECT { right: r.right - pad, ..line };
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
