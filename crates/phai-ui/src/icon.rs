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
    pub const CODE: u16 = 0xE943;
    pub const LIBRARY: u16 = 0xE8F1;
    pub const POINTER: u16 = 0xE7C9; // TouchPointer
    pub const FONT: u16 = 0xE8D2; // a label: words
    pub const CHECKBOX: u16 = 0xE73A; // CheckboxComposite
    pub const RADIO: u16 = 0xECCB; // RadioBtnOn
    pub const DROPDOWN: u16 = 0xE70D; // ChevronDown
    pub const LIST: u16 = 0xE8FD; // BulletedList
    pub const GROUP: u16 = 0xE8A9; // ViewAll
    pub const TABS: u16 = 0xE7C4; // TaskView
    pub const STATUS: u16 = 0xE8A1; // PreviewLink
    pub const BUTTON: u16 = 0xE815; // a button
    pub const FORM: u16 = 0xE737; // a window
    pub const ALIGN_LEFT: u16 = 0xE8E4;
    pub const ALIGN_CENTER: u16 = 0xE8E3;
    pub const ALIGN_RIGHT: u16 = 0xE8E2;
    pub const CALENDAR: u16 = 0xE787;
    pub const NUMBER: u16 = 0xE8EF; // Calculator
    pub const SLIDER: u16 = 0xE9E9; // Equalizer
    pub const PROGRESS: u16 = 0xE9D9; // Diagnostic
    pub const TEXTAREA: u16 = 0xE70B; // QuickNote
    pub const TREE: u16 = 0xE8B7; // Folder
    pub const PICTURE: u16 = 0xE8B9;
    pub const GLOBE: u16 = 0xE774; // Globe: the browser control
    pub const CHART: u16 = 0xE9D2; // AreaChart: the chart control
    pub const TIMER: u16 = 0xE916; // Stopwatch
    pub const MENU_BAR: u16 = 0xE700; // GlobalNavigationButton
    pub const TOOLBAR: u16 = 0xEC7A; // DeveloperTools
    pub const MENU_CONTEXT: u16 = 0xE712; // More
    pub const COLOR: u16 = 0xE790;
    pub const BACK: u16 = 0xE72B;
    pub const FORWARD: u16 = 0xE72A;
    pub const SAVE_ALL: u16 = 0xE78C; // SaveLocal
    pub const NEW_FOLDER: u16 = 0xE8F4;
    pub const STOP: u16 = 0xE71A;

    // The designer's Layout bar, as Visual Studio draws it: boxes and a line, drawn by
    // `super::shape_rects` rather than taken from the icon font, which has no such pictures.
    pub const LAY_GRID: u16 = 0x0180;
    pub const LAY_LEFTS: u16 = 0x0181;
    pub const LAY_CENTERS: u16 = 0x0182;
    pub const LAY_RIGHTS: u16 = 0x0183;
    pub const LAY_TOPS: u16 = 0x0184;
    pub const LAY_MIDDLES: u16 = 0x0185;
    pub const LAY_BOTTOMS: u16 = 0x0186;
    pub const LAY_SAME_WIDTH: u16 = 0x0187;
    pub const LAY_SIZE_GRID: u16 = 0x0188;
    pub const LAY_SAME_HEIGHT: u16 = 0x0189;
    pub const LAY_SAME_SIZE: u16 = 0x018A;
    pub const LAY_SPACE_ACROSS: u16 = 0x018B;
    pub const LAY_MORE_ACROSS: u16 = 0x018C;
    pub const LAY_LESS_ACROSS: u16 = 0x018D;
    pub const LAY_NONE_ACROSS: u16 = 0x018E;
    pub const LAY_SPACE_DOWN: u16 = 0x018F;
    pub const LAY_MORE_DOWN: u16 = 0x0190;
    pub const LAY_LESS_DOWN: u16 = 0x0191;
    pub const LAY_NONE_DOWN: u16 = 0x0192;
    pub const LAY_CENTER_ACROSS: u16 = 0x0193;
    pub const LAY_CENTER_DOWN: u16 = 0x0194;
    pub const LAY_FRONT: u16 = 0x0195;
    pub const LAY_BACK: u16 = 0x0196;
    pub const LAY_TAB_ORDER: u16 = 0x0197;
    // The debugger's steps, drawn as Visual Studio draws them: an arrow and the statement's dot.
    pub const STEP_INTO: u16 = 0x01A0;
    pub const STEP_OVER: u16 = 0x01A1;
    pub const STEP_OUT: u16 = 0x01A2;
    pub const LINK: u16 = 0xE71B; // Link
    pub const LISTBOX: u16 = 0xE8FD; // BulletedList
    pub const CHECKLIST: u16 = 0xE9D5; // CheckList
    pub const PANEL: u16 = 0xE7C3; // Page
    pub const MASKED: u16 = 0xE8C1; // Characters
    pub const MONTH: u16 = 0xE8BF; // CalendarDay
    pub const OPEN_FILE: u16 = 0xE8E5; // OpenFile
    pub const SAVE_FILE: u16 = 0xE792; // SaveAs
    pub const FOLDER_PICK: u16 = 0xE8B7; // Folder
    pub const PALETTE: u16 = 0xE790; // Color
    pub const FONT_PICK: u16 = 0xE8D2; // Font
    pub const ERROR_MARK: u16 = 0xE783; // Error
    pub const TRAY: u16 = 0xE7E7; // Taskbar
    pub const SPLIT: u16 = 0xE89F; // DockLeft
    pub const PRINT: u16 = 0xE749; // Print
    pub const BINDING: u16 = 0xE8F1; // Library
    pub const USER_CONTROL: u16 = 0xE7EF; // Contact
    pub const PAUSE: u16 = 0xE769;

    /// Every glyph by the name a form writes (`glyph = "SAVE"`).
    pub const NAMES: &[(&str, u16)] = &[
        ("NEW", NEW), ("OPEN", OPEN), ("SAVE", SAVE), ("UNDO", UNDO), ("REDO", REDO), ("CUT", CUT),
        ("COPY", COPY), ("PASTE", PASTE), ("DELETE", DELETE), ("PLAY", PLAY), ("BUILD", BUILD),
        ("SETTINGS", SETTINGS), ("REFRESH", REFRESH), ("SEARCH", SEARCH), ("FOLDER", FOLDER),
        ("DOCUMENT", DOCUMENT), ("MONITOR", MONITOR), ("CHAT", CHAT), ("APPS", APPS), ("UP", UP),
        ("DOWNLOAD", DOWNLOAD), ("UPLOAD", UPLOAD), ("RENAME", RENAME), ("BRIGHTNESS", BRIGHTNESS),
        ("LANGUAGE", LANGUAGE), ("CODE", CODE), ("LIBRARY", LIBRARY), ("POINTER", POINTER),
        ("FONT", FONT), ("CHECKBOX", CHECKBOX), ("RADIO", RADIO), ("DROPDOWN", DROPDOWN),
        ("LIST", LIST), ("GROUP", GROUP), ("TABS", TABS), ("STATUS", STATUS), ("BUTTON", BUTTON),
        ("FORM", FORM), ("ALIGN_LEFT", ALIGN_LEFT), ("ALIGN_CENTER", ALIGN_CENTER), ("ALIGN_RIGHT", ALIGN_RIGHT),
        ("CALENDAR", CALENDAR), ("NUMBER", NUMBER), ("SLIDER", SLIDER), ("PROGRESS", PROGRESS), ("TEXTAREA", TEXTAREA), ("COLOR", COLOR),
        ("BACK", BACK), ("FORWARD", FORWARD), ("SAVE_ALL", SAVE_ALL), ("NEW_FOLDER", NEW_FOLDER), ("STOP", STOP),
        ("LAY_GRID", LAY_GRID), ("LAY_LEFTS", LAY_LEFTS), ("LAY_CENTERS", LAY_CENTERS), ("LAY_RIGHTS", LAY_RIGHTS),
        ("LAY_TOPS", LAY_TOPS), ("LAY_MIDDLES", LAY_MIDDLES), ("LAY_BOTTOMS", LAY_BOTTOMS), ("LAY_SAME_WIDTH", LAY_SAME_WIDTH),
        ("LAY_SIZE_GRID", LAY_SIZE_GRID), ("LAY_SAME_HEIGHT", LAY_SAME_HEIGHT), ("LAY_SAME_SIZE", LAY_SAME_SIZE),
        ("LAY_SPACE_ACROSS", LAY_SPACE_ACROSS), ("LAY_MORE_ACROSS", LAY_MORE_ACROSS), ("LAY_LESS_ACROSS", LAY_LESS_ACROSS),
        ("LAY_NONE_ACROSS", LAY_NONE_ACROSS), ("LAY_SPACE_DOWN", LAY_SPACE_DOWN), ("LAY_MORE_DOWN", LAY_MORE_DOWN),
        ("LAY_LESS_DOWN", LAY_LESS_DOWN), ("LAY_NONE_DOWN", LAY_NONE_DOWN), ("LAY_CENTER_ACROSS", LAY_CENTER_ACROSS),
        ("LAY_CENTER_DOWN", LAY_CENTER_DOWN), ("LAY_FRONT", LAY_FRONT), ("LAY_BACK", LAY_BACK), ("LAY_TAB_ORDER", LAY_TAB_ORDER),
        ("STEP_INTO", STEP_INTO), ("STEP_OVER", STEP_OVER), ("STEP_OUT", STEP_OUT), ("PAUSE", PAUSE),
    ];

    pub fn by_name(name: &str) -> Option<u16> {
        NAMES.iter().find(|(n, _)| *n == name).map(|(_, g)| *g)
    }
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

/// The rectangles, on a 16-unit square, of a picture drawn rather than taken from the font -
/// `(x, y, w, h)`. Lines are a unit thick.
pub fn shape_rects(g: u16) -> Option<Vec<(f32, f32, f32, f32)>> {
    use glyph::*;
    // A frame round the square, for the centring pictures; the grid's dots.
    let frame = vec![(1.0, 1.0, 14.0, 1.0), (1.0, 14.0, 14.0, 1.0), (1.0, 1.0, 1.0, 14.0), (14.0, 1.0, 1.0, 14.0)];
    let dots = |v: &mut Vec<(f32, f32, f32, f32)>| {
        for y in [1.0, 5.0, 9.0, 13.0] {
            for x in [1.0, 5.0, 9.0, 13.0] {
                v.push((x, y, 1.5, 1.5));
            }
        }
    };
    let outline = |x: f32, y: f32, w: f32, h: f32| vec![(x, y, w, 1.0), (x, y + h - 1.0, w, 1.0), (x, y, 1.0, h), (x + w - 1.0, y, 1.0, h)];
    let v = match g {
        LAY_GRID => {
            let mut v = vec![];
            dots(&mut v);
            v.push((5.0, 5.0, 6.0, 6.0));
            v
        }
        LAY_SIZE_GRID => {
            let mut v = vec![];
            dots(&mut v);
            v.extend(outline(5.0, 5.0, 9.0, 9.0));
            v.push((10.0, 10.0, 4.0, 4.0));
            v
        }
        LAY_LEFTS => vec![(1.0, 1.0, 1.0, 14.0), (3.0, 3.0, 11.0, 4.0), (3.0, 9.0, 7.0, 4.0)],
        LAY_CENTERS => vec![(7.5, 1.0, 1.0, 14.0), (2.0, 3.0, 12.0, 4.0), (4.0, 9.0, 8.0, 4.0)],
        LAY_RIGHTS => vec![(14.0, 1.0, 1.0, 14.0), (2.0, 3.0, 11.0, 4.0), (6.0, 9.0, 7.0, 4.0)],
        LAY_TOPS => vec![(1.0, 1.0, 14.0, 1.0), (3.0, 3.0, 4.0, 11.0), (9.0, 3.0, 4.0, 7.0)],
        LAY_MIDDLES => vec![(1.0, 7.5, 14.0, 1.0), (3.0, 2.0, 4.0, 12.0), (9.0, 4.0, 4.0, 8.0)],
        LAY_BOTTOMS => vec![(1.0, 14.0, 14.0, 1.0), (3.0, 2.0, 4.0, 11.0), (9.0, 6.0, 4.0, 7.0)],
        LAY_SAME_WIDTH => vec![(2.0, 2.0, 12.0, 4.0), (2.0, 10.0, 12.0, 4.0), (2.0, 7.5, 12.0, 1.0)],
        LAY_SAME_HEIGHT => vec![(2.0, 2.0, 4.0, 12.0), (10.0, 2.0, 4.0, 12.0), (7.5, 2.0, 1.0, 12.0)],
        LAY_SAME_SIZE => vec![(1.0, 1.0, 6.0, 6.0), (9.0, 1.0, 6.0, 6.0), (1.0, 9.0, 6.0, 6.0), (9.0, 9.0, 6.0, 6.0)],
        LAY_SPACE_ACROSS => vec![(1.0, 3.0, 3.0, 10.0), (6.5, 3.0, 3.0, 10.0), (12.0, 3.0, 3.0, 10.0)],
        LAY_MORE_ACROSS => vec![(0.0, 3.0, 3.0, 10.0), (13.0, 3.0, 3.0, 10.0), (4.0, 7.5, 8.0, 1.0), (4.0, 6.0, 1.0, 4.0), (11.0, 6.0, 1.0, 4.0)],
        LAY_LESS_ACROSS => vec![(1.0, 3.0, 3.0, 10.0), (12.0, 3.0, 3.0, 10.0), (5.0, 7.5, 6.0, 1.0), (7.5, 6.0, 1.0, 4.0)],
        LAY_NONE_ACROSS => vec![(3.0, 3.0, 5.0, 10.0), (8.0, 3.0, 5.0, 10.0)],
        LAY_SPACE_DOWN => vec![(3.0, 1.0, 10.0, 3.0), (3.0, 6.5, 10.0, 3.0), (3.0, 12.0, 10.0, 3.0)],
        LAY_MORE_DOWN => vec![(3.0, 0.0, 10.0, 3.0), (3.0, 13.0, 10.0, 3.0), (7.5, 4.0, 1.0, 8.0), (6.0, 4.0, 4.0, 1.0), (6.0, 11.0, 4.0, 1.0)],
        LAY_LESS_DOWN => vec![(3.0, 1.0, 10.0, 3.0), (3.0, 12.0, 10.0, 3.0), (7.5, 5.0, 1.0, 6.0), (6.0, 7.5, 4.0, 1.0)],
        LAY_NONE_DOWN => vec![(3.0, 3.0, 10.0, 5.0), (3.0, 8.0, 10.0, 5.0)],
        LAY_CENTER_ACROSS => {
            let mut v = frame;
            v.push((5.0, 6.0, 6.0, 4.0));
            v.push((7.5, 3.0, 1.0, 10.0));
            v
        }
        LAY_CENTER_DOWN => {
            let mut v = frame;
            v.push((6.0, 5.0, 4.0, 6.0));
            v.push((3.0, 7.5, 10.0, 1.0));
            v
        }
        LAY_FRONT => {
            let mut v = outline(1.0, 1.0, 9.0, 9.0);
            v.push((6.0, 6.0, 9.0, 9.0));
            v
        }
        LAY_BACK => {
            let mut v = vec![(1.0, 1.0, 9.0, 5.0), (1.0, 6.0, 5.0, 4.0)];
            v.extend(outline(6.0, 6.0, 9.0, 9.0));
            v
        }
        STEP_INTO => vec![(7.5, 1.0, 1.0, 8.0), (5.5, 6.0, 5.0, 1.0), (6.5, 7.0, 3.0, 1.0), (7.5, 8.0, 1.0, 2.0), (6.5, 12.0, 3.0, 3.0)],
        STEP_OVER => vec![(2.0, 4.0, 1.0, 4.0), (2.0, 3.0, 11.0, 1.0), (12.0, 3.0, 1.0, 5.0), (10.0, 6.0, 5.0, 1.0), (11.0, 7.0, 3.0, 1.0), (12.0, 8.0, 1.0, 1.0), (6.5, 12.0, 3.0, 3.0)],
        STEP_OUT => vec![(7.5, 3.0, 1.0, 7.0), (5.5, 4.0, 5.0, 1.0), (6.5, 3.0, 3.0, 1.0), (7.5, 1.0, 1.0, 2.0), (6.5, 12.0, 3.0, 3.0)],
        LAY_TAB_ORDER => {
            let mut v = outline(1.0, 2.0, 14.0, 12.0);
            v.extend([(4.0, 5.0, 2.0, 6.0), (8.0, 5.0, 4.0, 1.0), (11.0, 5.0, 1.0, 3.0), (8.0, 7.5, 4.0, 1.0), (8.0, 7.5, 1.0, 3.5), (8.0, 10.0, 4.0, 1.0)]);
            v
        }
        _ => return None,
    };
    Some(v)
}

/// A drawn picture's pixels: each pixel as much ink as the rectangles cover of it.
fn shape_pixels(rects: &[(f32, f32, f32, f32)], size: i32, colour: Color) -> Vec<u8> {
    let k = size as f32 / 16.0;
    let mut out = vec![0u8; (size * size * 4) as usize];
    for py in 0..size {
        for px in 0..size {
            let (x0, y0) = (px as f32, py as f32);
            let mut cover = 0.0f32;
            // A box is drawn as its outline, a line as itself: thin strokes, as the icon font's.
            let mut parts: Vec<(f32, f32, f32, f32)> = Vec::new();
            for &(x, y, w, h) in rects {
                if w >= 3.0 && h >= 3.0 {
                    parts.extend([(x, y, w, 1.0), (x, y + h - 1.0, w, 1.0), (x, y + 1.0, 1.0, h - 2.0), (x + w - 1.0, y + 1.0, 1.0, h - 2.0)]);
                } else {
                    parts.push((x, y, w, h));
                }
            }
            for (x, y, w, h) in parts {
                let (l, t, r, b) = (x * k, y * k, (x + w) * k, (y + h) * k);
                let ox = (r.min(x0 + 1.0) - l.max(x0)).max(0.0);
                let oy = (b.min(y0 + 1.0) - t.max(y0)).max(0.0);
                cover += ox * oy;
            }
            let a = (cover.min(1.0) * 255.0).round() as u8;
            let i = ((py * size + px) * 4) as usize;
            out[i..i + 4].copy_from_slice(&[colour.b, colour.g, colour.r, a]);
        }
    }
    out
}

/// The pixels of one glyph of Windows' icon font, `size` square, in `colour`; `None` when neither
/// Segoe Fluent Icons nor Segoe MDL2 Assets is installed. A drawn picture's code gives its
/// rectangles instead.
pub fn glyph_pixels(glyph: u16, size: i32, colour: Color) -> Option<Vec<u8>> {
    if let Some(rects) = shape_rects(glyph) {
        return Some(shape_pixels(&rects, size, colour));
    }
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
        crate::win::draw_text_w(dc, &mut text, &mut r, Gdi::DT_CENTER | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX);
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

/// A glyph as a menu item's picture (`MENUITEMINFOW::hbmpItem`): the size of a menu's check mark,
/// in Windows' icon grey. Made once per glyph and kept - menus are built again often.
pub fn menu_bitmap(glyph: u16) -> Option<Gdi::HBITMAP> {
    menu_bitmap_in(glyph, Color::rgb(0x3c, 0x3c, 0x3c))
}

/// [`menu_bitmap`] in a colour of its own, as Visual Studio colours its menus' pictures.
pub fn menu_bitmap_in(glyph: u16, colour: Color) -> Option<Gdi::HBITMAP> {
    use std::collections::HashMap;
    use std::sync::Mutex;
    static MADE: Mutex<Option<HashMap<(u16, u32), isize>>> = Mutex::new(None);
    let mut made = MADE.lock().unwrap_or_else(|e| e.into_inner());
    let map = made.get_or_insert_with(HashMap::new);
    let key = (glyph, colour.colorref());
    if let Some(h) = map.get(&key) {
        return Some(Gdi::HBITMAP(*h as *mut _));
    }
    let size = unsafe { wm::GetSystemMetrics(wm::SM_CXMENUCHECK) }.max(16);
    let pixels = glyph_pixels(glyph, size, colour)?;
    unsafe {
        let info = Gdi::BITMAPINFO {
            bmiHeader: Gdi::BITMAPINFOHEADER { biSize: std::mem::size_of::<Gdi::BITMAPINFOHEADER>() as u32, biWidth: size, biHeight: -size, biPlanes: 1, biBitCount: 32, biCompression: 0, ..Default::default() },
            ..Default::default()
        };
        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let bmp = Gdi::CreateDIBSection(None, &info, Gdi::DIB_RGB_COLORS, &mut bits, None, 0).ok()?;
        if bits.is_null() {
            return None;
        }
        // Windows draws a menu's 32-bit picture with premultiplied alpha.
        let out = std::slice::from_raw_parts_mut(bits as *mut u8, pixels.len());
        for (o, p) in out.as_chunks_mut::<4>().0.iter_mut().zip(pixels.as_chunks::<4>().0) {
            let a = p[3] as u32;
            o[0] = (p[0] as u32 * a / 255) as u8;
            o[1] = (p[1] as u32 * a / 255) as u8;
            o[2] = (p[2] as u32 * a / 255) as u8;
            o[3] = p[3];
        }
        map.insert(key, bmp.0 as isize);
        Some(bmp)
    }
}

/// Puts a glyph beside a menu item, found by its command id.
pub fn set_menu_glyph(menu: wm::HMENU, id: u32, glyph: u16) {
    set_menu_glyph_in(menu, id, glyph, Color::rgb(0x3c, 0x3c, 0x3c));
}

/// [`set_menu_glyph`] in a colour of its own.
pub fn set_menu_glyph_in(menu: wm::HMENU, id: u32, glyph: u16, colour: Color) {
    if let Some(bmp) = menu_bitmap_in(glyph, colour) {
        let info = wm::MENUITEMINFOW { cbSize: std::mem::size_of::<wm::MENUITEMINFOW>() as u32, fMask: wm::MIIM_BITMAP, hbmpItem: bmp, ..Default::default() };
        unsafe {
            let _ = wm::SetMenuItemInfoW(menu, id, false, &info);
        }
    }
}

/// The program's own icon, from its resources (the `.ico` its `build.rs` embeds as icon 1), at
/// `size` pixels - Windows picks the best of the sizes inside. None when the program has none.
pub fn app_icon(size: i32) -> Option<HICON> {
    unsafe {
        let h = wm::LoadImageW(Some(crate::win::instance()), windows::core::PCWSTR(std::ptr::without_provenance(1)) /* MAKEINTRESOURCE(1) */, wm::IMAGE_ICON, size, size, wm::LR_DEFAULTCOLOR).ok()?;
        Some(HICON(h.0))
    }
}

/// Gives a window the program's icon at both sizes Windows shows: the title bar and Alt+Tab.
pub fn set_app_icon(hwnd: windows::Win32::Foundation::HWND) {
    unsafe {
        let small = wm::GetSystemMetrics(wm::SM_CXSMICON);
        let big = wm::GetSystemMetrics(wm::SM_CXICON);
        for (which, size) in [(wm::ICON_SMALL, small), (wm::ICON_BIG, big)] {
            if let Some(i) = app_icon(size) {
                wm::SendMessageW(hwnd, wm::WM_SETICON, Some(windows::Win32::Foundation::WPARAM(which as usize)), Some(windows::Win32::Foundation::LPARAM(i.0 as isize)));
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Every named glyph is one the icon font has: a missing one would draw an empty square.
    #[test]
    fn every_named_glyph_draws_something() {
        if glyph_pixels(glyph::SAVE, 32, Color::rgb(0, 0, 0)).is_none() {
            return; // no icon font here
        }
        for (name, g) in glyph::NAMES {
            let px = glyph_pixels(*g, 32, Color::rgb(0, 0, 0)).unwrap();
            let solid = px.as_chunks::<4>().0.iter().filter(|p| p[3] > 128).count();
            assert!(solid > 12, "{name} ({g:#X}) draws almost nothing - not in the font?");
        }
    }

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
