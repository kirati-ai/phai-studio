//! Icons drawn here rather than shipped: from straight-alpha pixels, and from one glyph of
//! Windows' own icon font. From PhaiDesk (`crates/ui/src/tray.rs`, `console/win.rs`).

use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HICON};

use crate::theme::Color;

/// Glyphs in Segoe Fluent Icons / Segoe MDL2 Assets - the icons the rest of Windows draws.
pub mod glyph {
    pub const NEW: u16 = 0xE710; // Add
    pub const OPEN: u16 = 0xE8E5; // OpenFile
    pub const SAVE: u16 = 0xE74E;
    pub const UNDO: u16 = 0xE7A7;
    pub const REDO: u16 = 0xE7A6;
    pub const CUT: u16 = 0xE8C6;
    pub const COPY: u16 = 0xE8C8;
    pub const PASTE: u16 = 0xE77F;
    pub const DELETE: u16 = 0xE74D;
    pub const PLAY: u16 = 0xE768;
    pub const BUILD: u16 = 0xE90F; // Repair
    pub const SETTINGS: u16 = 0xE713;
    pub const REFRESH: u16 = 0xE72C;
    pub const SEARCH: u16 = 0xE721;
    pub const FOLDER: u16 = 0xE8B7;
    pub const DOCUMENT: u16 = 0xE8A5;
    pub const MONITOR: u16 = 0xE7F4;
    pub const CHAT: u16 = 0xE8F2;
    pub const APPS: u16 = 0xE71D;
    pub const UP: u16 = 0xE74A;
    pub const DOWNLOAD: u16 = 0xE896;
    pub const UPLOAD: u16 = 0xE898;
    pub const RENAME: u16 = 0xE8AC;
    pub const BRIGHTNESS: u16 = 0xE706;
    pub const LANGUAGE: u16 = 0xF2B7; // LocaleLanguage
}

/// An icon or cursor from straight-alpha BGRA pixels, top row first. The caller destroys it.
/// The mask is a real one: where a pixel is see-through it says so, so places that ignore alpha
/// show what is behind it rather than a black square.
pub fn from_bgra(pixels: &[u8], width: i32, height: i32, hotspot_x: i32, hotspot_y: i32, is_icon: bool) -> Option<HICON> {
    if width <= 0 || height <= 0 || pixels.len() < (width * height * 4) as usize {
        return None;
    }
    unsafe {
        let info = Gdi::BITMAPINFO {
            bmiHeader: Gdi::BITMAPINFOHEADER {
                biSize: std::mem::size_of::<Gdi::BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: Gdi::BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        let color = Gdi::CreateDIBSection(None, &info, Gdi::DIB_RGB_COLORS, &mut bits, None, 0).ok()?;
        if bits.is_null() {
            let _ = Gdi::DeleteObject(color.into());
            return None;
        }
        std::ptr::copy_nonoverlapping(pixels.as_ptr(), bits as *mut u8, (width * height * 4) as usize);
        // Rows of the mask are padded to 16 bits.
        let stride = (width as usize).div_ceil(16) * 2;
        let mut mask_bits = vec![0u8; stride * height as usize];
        for y in 0..height as usize {
            for x in 0..width as usize {
                if pixels[(y * width as usize + x) * 4 + 3] < 128 {
                    mask_bits[y * stride + x / 8] |= 0x80 >> (x % 8);
                }
            }
        }
        let mask = Gdi::CreateBitmap(width, height, 1, 1, Some(mask_bits.as_ptr() as *const _));
        let icon_info = wm::ICONINFO { fIcon: is_icon.into(), xHotspot: hotspot_x as u32, yHotspot: hotspot_y as u32, hbmMask: mask, hbmColor: color };
        let icon = wm::CreateIconIndirect(&icon_info).ok();
        let _ = Gdi::DeleteObject(color.into());
        let _ = Gdi::DeleteObject(mask.into());
        icon
    }
}

/// The pixels of one glyph of Windows' icon font, `size` square, in `colour`; `None` when neither
/// Segoe Fluent Icons nor Segoe MDL2 Assets is installed.
pub fn glyph_pixels(glyph: u16, size: i32, colour: Color) -> Option<Vec<u8>> {
    unsafe {
        let screen = Gdi::GetDC(None);
        let dc = Gdi::CreateCompatibleDC(Some(screen));
        Gdi::ReleaseDC(None, screen);
        let info = Gdi::BITMAPINFO {
            bmiHeader: Gdi::BITMAPINFOHEADER {
                biSize: std::mem::size_of::<Gdi::BITMAPINFOHEADER>() as u32,
                biWidth: size,
                biHeight: -size,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: Gdi::BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        let Ok(bmp) = Gdi::CreateDIBSection(Some(dc), &info, Gdi::DIB_RGB_COLORS, &mut bits, None, 0) else {
            let _ = Gdi::DeleteDC(dc);
            return None;
        };
        let old_bmp = Gdi::SelectObject(dc, bmp.into());
        let mut font = None;
        for face in ["Segoe Fluent Icons", "Segoe MDL2 Assets"] {
            let mut lf = Gdi::LOGFONTW { lfHeight: -(size * 8 / 10), lfQuality: Gdi::ANTIALIASED_QUALITY, lfCharSet: Gdi::DEFAULT_CHARSET, ..Default::default() };
            for (i, u) in face.encode_utf16().take(31).enumerate() {
                lf.lfFaceName[i] = u;
            }
            let f = Gdi::CreateFontIndirectW(&lf);
            let old = Gdi::SelectObject(dc, f.into());
            let mut got = [0u16; 32];
            let n = Gdi::GetTextFaceW(dc, Some(&mut got));
            Gdi::SelectObject(dc, old);
            if String::from_utf16_lossy(&got[..(n.max(1) as usize - 1)]) == face {
                font = Some(f);
                break;
            }
            let _ = Gdi::DeleteObject(f.into());
        }
        let Some(font) = font else {
            Gdi::SelectObject(dc, old_bmp);
            let _ = Gdi::DeleteObject(bmp.into());
            let _ = Gdi::DeleteDC(dc);
            return None;
        };
        let old_font = Gdi::SelectObject(dc, font.into());
        // White on black: the coverage of each pixel is then its brightness.
        let mut r = windows::Win32::Foundation::RECT { left: 0, top: 0, right: size, bottom: size };
        Gdi::FillRect(dc, &r, Gdi::HBRUSH(Gdi::GetStockObject(Gdi::BLACK_BRUSH).0));
        Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(0x00FF_FFFF));
        let mut text = [glyph];
        Gdi::DrawTextW(dc, &mut text, &mut r, Gdi::DT_CENTER | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX);
        let _ = Gdi::GdiFlush();
        let n = (size * size * 4) as usize;
        let src = std::slice::from_raw_parts(bits as *const u8, n);
        let mut pixels = vec![0u8; n];
        for (out, px) in pixels.as_chunks_mut::<4>().0.iter_mut().zip(src.as_chunks::<4>().0) {
            let a = px[0].max(px[1]).max(px[2]);
            out.copy_from_slice(&[colour.b, colour.g, colour.r, a]);
        }
        Gdi::SelectObject(dc, old_font);
        Gdi::SelectObject(dc, old_bmp);
        let _ = Gdi::DeleteObject(font.into());
        let _ = Gdi::DeleteObject(bmp.into());
        let _ = Gdi::DeleteDC(dc);
        Some(pixels)
    }
}

/// An icon from one glyph, in `colour`.
pub fn glyph_icon(glyph: u16, size: i32, colour: Color) -> Option<HICON> {
    from_bgra(&glyph_pixels(glyph, size, colour)?, size, size, 0, 0, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_glyph_is_drawn_in_the_colour_asked_with_clear_corners() {
        let Some(px) = glyph_pixels(glyph::SAVE, 32, Color::rgb(0x12, 0x85, 0x5f)) else {
            return; // a machine without the icon fonts: nothing to check
        };
        assert_eq!(px[3], 0, "the top-left corner is see-through");
        assert!(px.as_chunks::<4>().0.iter().any(|p| p[3] == 255), "some of the glyph is solid");
        assert!(px.as_chunks::<4>().0.iter().all(|p| p[..3] == [0x5f, 0x85, 0x12]), "BGRA, in the colour asked");
    }
}
