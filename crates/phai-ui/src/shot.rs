//! Photographs a window to PNG - how it really looks, rounded corners and all. Bugs in how a
//! window looks do not show in code (PhaiDesk's lesson): every window is checked by its picture.

use std::path::Path;

use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi;
use windows::Win32::Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS};
use windows::Win32::UI::WindowsAndMessaging as wm;

/// The window's pixels, RGBA top row first, with its size.
pub fn capture(hwnd: HWND) -> Option<(Vec<u8>, u32, u32)> {
    const PW_RENDERFULLCONTENT: u32 = 2;
    unsafe {
        let mut r = RECT::default();
        wm::GetWindowRect(hwnd, &mut r).ok()?;
        let (w, h) = (r.right - r.left, r.bottom - r.top);
        if w <= 0 || h <= 0 {
            return None;
        }
        let screen = Gdi::GetDC(None);
        let dc = Gdi::CreateCompatibleDC(Some(screen));
        let bmp = Gdi::CreateCompatibleBitmap(screen, w, h);
        let old = Gdi::SelectObject(dc, bmp.into());
        let ok = PrintWindow(hwnd, dc, PRINT_WINDOW_FLAGS(PW_RENDERFULLCONTENT)).as_bool();
        let mut info = Gdi::BITMAPINFO {
            bmiHeader: Gdi::BITMAPINFOHEADER {
                biSize: std::mem::size_of::<Gdi::BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: Gdi::BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut px = vec![0u8; (w * h * 4) as usize];
        Gdi::SelectObject(dc, old);
        let lines = Gdi::GetDIBits(dc, bmp, 0, h as u32, Some(px.as_mut_ptr() as *mut _), &mut info, Gdi::DIB_RGB_COLORS);
        let _ = Gdi::DeleteObject(bmp.into());
        let _ = Gdi::DeleteDC(dc);
        Gdi::ReleaseDC(None, screen);
        if !ok || lines == 0 {
            return None;
        }
        for p in px.as_chunks_mut::<4>().0 {
            p.swap(0, 2);
            p[3] = 255;
        }
        Some((px, w as u32, h as u32))
    }
}

/// Saves the window as a PNG at `path`.
pub fn save(hwnd: HWND, path: &Path) -> std::io::Result<()> {
    let (px, w, h) = capture(hwnd).ok_or_else(|| std::io::Error::other("PrintWindow could not photograph the window"))?;
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut enc = png::Encoder::new(file, w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().map_err(std::io::Error::other)?;
    writer.write_image_data(&px).map_err(std::io::Error::other)?;
    Ok(())
}
