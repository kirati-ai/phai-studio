//! A picture on a window, as WinForms' PictureBox: a PNG, JPEG, BMP or GIF read with Windows'
//! own imaging (WIC), drawn whole inside the control - its shape kept - and drawn again at every
//! size. A path that is not absolute is looked for beside the program, then where it runs.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::{GENERIC_READ, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::Graphics::Imaging::*;
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::win;

/// Where pictures with a relative path are found first - the designer sets it to the project.
static BASE: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Relative picture paths are found under `dir` first (Phai Studio: the project's folder).
pub fn set_base(dir: Option<&Path>) {
    *BASE.lock().unwrap_or_else(|e| e.into_inner()) = dir.map(Path::to_path_buf);
}

/// Where a picture's file is: as written when absolute; else under the base, beside the
/// program, or where it runs - the first that is there.
pub fn resolve(path: &str) -> Option<PathBuf> {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        return p.is_file().then_some(p);
    }
    let base = BASE.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let exe_dir = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf));
    [base, exe_dir, std::env::current_dir().ok()].into_iter().flatten().map(|d| d.join(&p)).find(|c| c.is_file())
}

/// A picture read into a 32-bit bitmap with its alpha premultiplied, and its size.
pub(crate) struct Image {
    pub(crate) bmp: Gdi::HBITMAP,
    pub(crate) w: i32,
    pub(crate) h: i32,
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            let _ = Gdi::DeleteObject(self.bmp.into());
        }
    }
}

pub(crate) fn load(path: &Path) -> Option<Image> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let factory: IWICImagingFactory = CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER).ok()?;
        let name = HSTRING::from(path.as_os_str());
        let decoder = factory.CreateDecoderFromFilename(&name, None, GENERIC_READ, WICDecodeMetadataCacheOnDemand).ok()?;
        let frame = decoder.GetFrame(0).ok()?;
        let conv = factory.CreateFormatConverter().ok()?;
        conv.Initialize(&frame, &GUID_WICPixelFormat32bppPBGRA, WICBitmapDitherTypeNone, None, 0.0, WICBitmapPaletteTypeCustom).ok()?;
        let (mut w, mut h) = (0u32, 0u32);
        conv.GetSize(&mut w, &mut h).ok()?;
        if w == 0 || h == 0 || w > 16384 || h > 16384 {
            return None;
        }
        let info = Gdi::BITMAPINFO {
            bmiHeader: Gdi::BITMAPINFOHEADER { biSize: std::mem::size_of::<Gdi::BITMAPINFOHEADER>() as u32, biWidth: w as i32, biHeight: -(h as i32), biPlanes: 1, biBitCount: 32, biCompression: Gdi::BI_RGB.0, ..Default::default() },
            ..Default::default()
        };
        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        let bmp = Gdi::CreateDIBSection(None, &info, Gdi::DIB_RGB_COLORS, &mut bits, None, 0).ok()?;
        let stride = w * 4;
        let buf = std::slice::from_raw_parts_mut(bits as *mut u8, (stride * h) as usize);
        if conv.CopyPixels(std::ptr::null(), stride, buf).is_err() {
            let _ = Gdi::DeleteObject(bmp.into());
            return None;
        }
        Some(Image { bmp, w: w as i32, h: h as i32 })
    }
}

struct Picture {
    image: Option<Image>,
    /// SizeMode: 0 zoom, 1 normal, 2 stretch, 3 center.
    mode: u8,
}

/// How the picture fills its box: `"zoom"`, `"normal"`, `"stretch"` or `"center"`.
pub fn set_size_mode(h: HWND, mode: &str) {
    if let Some(p) = unsafe { win::state::<Picture>(h) } {
        p.mode = match mode { "normal" => 1, "stretch" => 2, "center" => 3, _ => 0 };
        unsafe {
            let _ = Gdi::InvalidateRect(Some(h), None, false);
        }
    }
}

/// A picture control showing the file at `path` (nothing shown when it cannot be read).
pub fn picture(parent: HWND, id: i32, path: &str) -> HWND {
    let class = win::register("PhaiPicture", Some(proc_), None);
    let h = unsafe {
        wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPSIBLINGS, 0, 0, 10, 10, Some(parent), Some(wm::HMENU(id as isize as *mut _)), Some(win::instance()), None).unwrap_or_default()
    };
    win::set_state(h, Box::new(Picture { image: None, mode: 0 }));
    set_picture(h, path);
    h
}

/// Shows another file in a picture control; an empty path shows none.
pub fn set_picture(h: impl Into<HWND>, path: &str) {
    let h: HWND = h.into();
    if let Some(p) = unsafe { win::state::<Picture>(h) } {
        p.image = (!path.trim().is_empty()).then(|| resolve(path.trim())).flatten().and_then(|f| load(&f));
        unsafe {
            let _ = Gdi::InvalidateRect(Some(h), None, true);
        }
    }
}

/// Where a `w` by `h` picture goes in `r`: as large as fits, its shape kept, in the middle.
pub fn fit(r: RECT, w: i32, h: i32) -> RECT {
    let (rw, rh) = (r.right - r.left, r.bottom - r.top);
    if w <= 0 || h <= 0 || rw <= 0 || rh <= 0 {
        return RECT { left: r.left, top: r.top, right: r.left, bottom: r.top };
    }
    let (sw, sh) = if rw as i64 * h as i64 <= rh as i64 * w as i64 { (rw, (rw as i64 * h as i64 / w as i64) as i32) } else { ((rh as i64 * w as i64 / h as i64) as i32, rh) };
    let x = r.left + (rw - sw) / 2;
    let y = r.top + (rh - sh) / 2;
    RECT { left: x, top: y, right: x + sw, bottom: y + sh }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_SIZE => {
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            let bg = win::window_background_for(h);
            win::fill(dc, &r, bg);
            let mode = win::state::<Picture>(h).map(|p| p.mode).unwrap_or(0);
            match win::state::<Picture>(h).and_then(|p| p.image.as_ref()) {
                Some(img) => {
                    let to = match mode {
                        1 => RECT { left: 0, top: 0, right: img.w, bottom: img.h },
                        2 => r,
                        3 => {
                            let (x, y) = ((r.right - img.w) / 2, (r.bottom - img.h) / 2);
                            RECT { left: x, top: y, right: x + img.w, bottom: y + img.h }
                        }
                        _ => fit(r, img.w, img.h),
                    };
                    let src = Gdi::CreateCompatibleDC(Some(dc));
                    let old = Gdi::SelectObject(src, img.bmp.into());
                    let blend = Gdi::BLENDFUNCTION { BlendOp: 0, BlendFlags: 0, SourceConstantAlpha: 255, AlphaFormat: 1 /* AC_SRC_ALPHA */ };
                    let _ = Gdi::SetStretchBltMode(dc, Gdi::HALFTONE);
                    let _ = Gdi::AlphaBlend(dc, to.left, to.top, to.right - to.left, to.bottom - to.top, src, 0, 0, img.w, img.h, blend);
                    Gdi::SelectObject(src, old);
                    let _ = Gdi::DeleteDC(src);
                }
                None => {
                    // No picture: a faint frame, so the control can be seen and placed.
                    let line = crate::theme::current().var_color("--border").unwrap_or(crate::theme::Color::rgb(0xc8, 0xc8, 0xc8));
                    win::fill(dc, &RECT { left: r.left, top: r.top, right: r.right, bottom: r.top + 1 }, line);
                    win::fill(dc, &RECT { left: r.left, top: r.bottom - 1, right: r.right, bottom: r.bottom }, line);
                    win::fill(dc, &RECT { left: r.left, top: r.top, right: r.left + 1, bottom: r.bottom }, line);
                    win::fill(dc, &RECT { left: r.right - 1, top: r.top, right: r.right, bottom: r.bottom }, line);
                }
            }
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        wm::WM_NCDESTROY => {
            let _ = unsafe { win::take_state::<Picture>(h) };
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picture_fits_whole_with_its_shape() {
        let r = RECT { left: 0, top: 0, right: 200, bottom: 100 };
        let f = fit(r, 400, 400);
        assert_eq!((f.left, f.top, f.right, f.bottom), (50, 0, 150, 100), "square in a wide box: as tall as the box, in the middle");
        let f = fit(r, 400, 100);
        assert_eq!((f.left, f.top, f.right, f.bottom), (0, 25, 200, 75));
    }
}
