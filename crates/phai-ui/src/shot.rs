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
    // PW_RENDERFULLCONTENT takes what the screen shows, rounded corners and all - but for some
    // windows (a ListView in Details, seen in the guide's demos) it hands back the frame with a
    // black inside. Then the window is asked to draw itself instead (flag 0).
    let full = capture_with(hwnd, PW_RENDERFULLCONTENT)?;
    if mostly_black(&full) {
        if let Some(own) = capture_with(hwnd, 0).filter(|c| !mostly_black(c)) {
            return Some(own);
        }
    }
    Some(full)
}

/// More than nine pixels in ten of its inside pure black (the title bar left out, which the
/// black ones keep): a picture PrintWindow could not take.
fn mostly_black((px, w, h): &(Vec<u8>, u32, u32)) -> bool {
    let (w, h) = (*w as usize, *h as usize);
    let rows = h * 3 / 10..h * 9 / 10;
    let (mut all, mut black) = (0usize, 0usize);
    for y in rows {
        for p in px[y * w * 4..(y + 1) * w * 4].as_chunks::<4>().0 {
            all += 1;
            black += usize::from(p[0] == 0 && p[1] == 0 && p[2] == 0);
        }
    }
    all > 0 && black * 10 > all * 9
}

fn capture_with(hwnd: HWND, flags: u32) -> Option<(Vec<u8>, u32, u32)> {
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
        let ok = PrintWindow(hwnd, dc, PRINT_WINDOW_FLAGS(flags)).as_bool();
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
