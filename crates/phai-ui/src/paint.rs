//! A control the program draws itself - WinForms' Paint event and its PaintEventArgs. A panel or a
//! picture whose `paint` event is set is drawn by the program's method each time Windows asks:
//! into a picture in memory first, then onto the screen in one step, so nothing flickers.
//! `ctl::Panel::invalidate()` asks for it again, as `Invalidate()` does.
//!
//! Places and sizes are the control's own pixels (`e.width`, `e.height`), its top left `0, 0`.

use windows::Win32::Foundation::{COLORREF, HWND, RECT};
use windows::Win32::Graphics::Gdi::{self, HDC};

use crate::theme::{self, Color};

/// What a Paint method draws on - WinForms' PaintEventArgs with its Graphics.
pub struct PaintEventArgs {
    dc: HDC,
    /// The control's size in pixels.
    pub width: i32,
    pub height: i32,
}

fn colour(css: &str) -> Color {
    theme::parse_color(css).unwrap_or(Color::rgb(0, 0, 0))
}

fn cr(c: Color) -> COLORREF {
    COLORREF(c.colorref())
}

impl PaintEventArgs {
    pub(crate) fn new(dc: HDC, width: i32, height: i32) -> PaintEventArgs {
        PaintEventArgs { dc, width, height }
    }

    /// The whole control in one colour - `#2a2a2c`, `white`, `var` colours of CSS.
    pub fn clear(&self, css: &str) {
        self.fill_rect(0, 0, self.width, self.height, css);
    }

    pub fn fill_rect(&self, x: i32, y: i32, w: i32, h: i32, css: &str) {
        crate::win::fill(self.dc, &RECT { left: x, top: y, right: x + w, bottom: y + h }, colour(css));
    }

    /// A line one pixel wide.
    pub fn draw_line(&self, x1: i32, y1: i32, x2: i32, y2: i32, css: &str) {
        unsafe {
            let pen = Gdi::CreatePen(Gdi::PS_SOLID, 1, cr(colour(css)));
            let old = Gdi::SelectObject(self.dc, pen.into());
            let _ = Gdi::MoveToEx(self.dc, x1, y1, None);
            let _ = Gdi::LineTo(self.dc, x2, y2);
            Gdi::SelectObject(self.dc, old);
            let _ = Gdi::DeleteObject(pen.into());
        }
    }

    /// Words in the control's font, their top left at `x, y`.
    pub fn draw_text(&self, text: &str, x: i32, y: i32, css: &str) {
        let mut r = RECT { left: x, top: y, right: self.width.max(x + 1), bottom: self.height.max(y + 1) };
        self.text(text, &mut r, css, Gdi::DT_NOPREFIX);
    }

    /// Words centred in the control - "Connecting...".
    pub fn draw_text_centered(&self, text: &str, css: &str) {
        let mut r = RECT { left: 0, top: 0, right: self.width, bottom: self.height };
        self.text(text, &mut r, css, Gdi::DT_CENTER | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX);
    }

    fn text(&self, text: &str, r: &mut RECT, css: &str, flags: Gdi::DRAW_TEXT_FORMAT) {
        if text.is_empty() {
            return;
        }
        unsafe {
            Gdi::SetBkMode(self.dc, Gdi::TRANSPARENT);
            Gdi::SetTextColor(self.dc, cr(colour(css)));
            let mut w: Vec<u16> = text.encode_utf16().collect();
            Gdi::DrawTextW(self.dc, &mut w, r, flags);
        }
    }

    /// A picture of `size` (width, height) pixels, 4 bytes each (blue, green, red, unused - what a
    /// screen capture gives), drawn into `place` (x, y, w, h): shrunk smoothly, grown as it is -
    /// `e.draw_pixels(&bgra, (pw, ph), e.fit(pw, ph))`.
    pub fn draw_pixels(&self, bgra: &[u8], size: (i32, i32), place: (i32, i32, i32, i32)) {
        let ((pw, ph), (x, y, w, h)) = (size, place);
        if pw <= 0 || ph <= 0 || bgra.len() < (pw * ph * 4) as usize {
            return;
        }
        let info = Gdi::BITMAPINFO {
            bmiHeader: Gdi::BITMAPINFOHEADER { biSize: std::mem::size_of::<Gdi::BITMAPINFOHEADER>() as u32, biWidth: pw, biHeight: -ph, biPlanes: 1, biBitCount: 32, biCompression: Gdi::BI_RGB.0, ..Default::default() },
            ..Default::default()
        };
        unsafe {
            Gdi::SetStretchBltMode(self.dc, if w < pw { Gdi::HALFTONE } else { Gdi::COLORONCOLOR });
            let _ = Gdi::SetBrushOrgEx(self.dc, 0, 0, None);
            Gdi::StretchDIBits(self.dc, x, y, w, h, 0, 0, pw, ph, Some(bgra.as_ptr() as *const _), &info, Gdi::DIB_RGB_COLORS, Gdi::SRCCOPY);
        }
    }

    /// Where a picture of `pw` x `ph` fits the control whole, its shape kept and centred:
    /// `(x, y, w, h)` - never larger than it is.
    pub fn fit(&self, pw: i32, ph: i32) -> (i32, i32, i32, i32) {
        fit(self.width, self.height, pw, ph)
    }

    /// The device context, for drawing with Windows' own calls.
    pub fn hdc(&self) -> HDC {
        self.dc
    }
}

/// Where `pw` x `ph` fits `cw` x `ch` whole, its shape kept, centred, never grown.
pub fn fit(cw: i32, ch: i32, pw: i32, ph: i32) -> (i32, i32, i32, i32) {
    if pw <= 0 || ph <= 0 || cw <= 0 || ch <= 0 {
        return (0, 0, 0, 0);
    }
    let s = (cw as f64 / pw as f64).min(ch as f64 / ph as f64).min(1.0);
    let (w, h) = ((pw as f64 * s).round() as i32, (ph as f64 * s).round() as i32);
    ((cw - w) / 2, (ch - h) / 2, w, h)
}

/// `WM_PAINT` of a control whose Paint the program draws: `draw` given the args on a picture in
/// memory, the picture then copied to the screen.
pub(crate) fn paint(h: HWND, draw: impl FnOnce(&PaintEventArgs)) {
    unsafe {
        let mut ps = Gdi::PAINTSTRUCT::default();
        let hdc = Gdi::BeginPaint(h, &mut ps);
        let mut r = RECT::default();
        let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(h, &mut r);
        let (w, hh) = (r.right.max(1), r.bottom.max(1));
        let dc = Gdi::CreateCompatibleDC(Some(hdc));
        let bitmap = Gdi::CreateCompatibleBitmap(hdc, w, hh);
        let old = Gdi::SelectObject(dc, bitmap.into());
        // The window's own colour under whatever is drawn, and its font for words.
        crate::win::fill(dc, &r, crate::win::window_background_for(h));
        let font = windows::Win32::UI::WindowsAndMessaging::SendMessageW(h, windows::Win32::UI::WindowsAndMessaging::WM_GETFONT, None, None).0;
        let old_font = (font != 0).then(|| Gdi::SelectObject(dc, Gdi::HGDIOBJ(font as *mut _)));
        draw(&PaintEventArgs::new(dc, w, hh));
        if let Some(f) = old_font {
            Gdi::SelectObject(dc, f);
        }
        let _ = Gdi::BitBlt(hdc, 0, 0, w, hh, Some(dc), 0, 0, Gdi::SRCCOPY);
        Gdi::SelectObject(dc, old);
        let _ = Gdi::DeleteObject(bitmap.into());
        let _ = Gdi::DeleteDC(dc);
        let _ = Gdi::EndPaint(h, &ps);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_picture_fits_whole_and_centred_never_grown() {
        assert_eq!(super::fit(1000, 600, 1920, 1080), (0, 18, 1000, 563));
        assert_eq!(super::fit(1000, 600, 800, 600), (100, 0, 800, 600), "the height fits as it is");
        assert_eq!(super::fit(2000, 2000, 800, 600), (600, 700, 800, 600), "never grown");
        assert_eq!(super::fit(0, 600, 800, 600), (0, 0, 0, 0));
    }
}
