//! Printing - WinForms' PrintDocument, PrintDialog, PageSetupDialog and PrintPreviewDialog in
//! one component. The program draws each page in its PrintPage event: words, lines, boxes,
//! pictures, measured in hundredths of an inch from the paper's corner as WinForms measures them.
//! It says whether another page follows (`e.set_has_more_pages(true)`). The same drawing goes to a
//! printer ([`PrintDocument::print`], [`PrintDocument::print_dialog`]), to a file through a
//! printer that writes one ([`PrintDocument::print_to_file`], "Microsoft Print to PDF"), or into a
//! window that shows the pages before anything is printed ([`PrintDocument::print_preview`]).

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{COLORREF, HGLOBAL, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HDC, HENHMETAFILE, HFONT};
use windows::Win32::Graphics::Printing as pr;
use windows::Win32::Storage::Xps as xps;
use windows::Win32::System::Memory as mem;
use windows::Win32::UI::Controls::Dialogs as dlg;
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::events;
use crate::text::{t, tf};
use crate::theme::{self, Color};
use crate::win::{self, wide};

/// A rectangle on the page, in hundredths of an inch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Bounds {
    pub fn right(&self) -> i32 {
        self.x + self.width
    }
    pub fn bottom(&self) -> i32 {
        self.y + self.height
    }
}

/// Where a line of words stands in the width it is given.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

/// One page to draw - WinForms' PrintPageEventArgs and the Graphics it carries. Every length is
/// in hundredths of an inch, from the paper's top-left corner.
pub struct PrintPageEventArgs {
    dc: HDC,
    /// The page being drawn, from 1.
    pub page: usize,
    /// The whole sheet of paper.
    pub page_bounds: Bounds,
    /// The sheet inside its margins: where the words usually go.
    pub margin_bounds: Bounds,
    /// Drawn for the preview window rather than for paper.
    pub preview: bool,
    more: Cell<bool>,
    font: Cell<HFONT>,
    color: Cell<COLORREF>,
    pen: Cell<i32>,
}

impl PrintPageEventArgs {
    fn new(dc: HDC, page: usize, page_bounds: Bounds, margin_bounds: Bounds, preview: bool) -> PrintPageEventArgs {
        let e = PrintPageEventArgs { dc, page, page_bounds, margin_bounds, preview, more: Cell::new(false), font: Cell::new(HFONT::default()), color: Cell::new(COLORREF(0)), pen: Cell::new(1) };
        e.set_font("Segoe UI", 10, false, false);
        unsafe {
            Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        }
        e
    }

    /// Another page follows this one - WinForms' `e.HasMorePages = true`.
    pub fn set_has_more_pages(&self, more: bool) {
        self.more.set(more);
    }
    pub fn has_more_pages(&self) -> bool {
        self.more.get()
    }

    /// The words from now on in `face`, `points` big.
    pub fn set_font(&self, face: &str, points: i32, bold: bool, italic: bool) {
        let mut lf = Gdi::LOGFONTW { lfHeight: -(points.max(1) * 100 / 72), lfWeight: if bold { 700 } else { 400 }, lfItalic: italic as u8, lfQuality: Gdi::CLEARTYPE_QUALITY, ..Default::default() };
        for (i, c) in face.encode_utf16().take(31).enumerate() {
            lf.lfFaceName[i] = c;
        }
        unsafe {
            let f = Gdi::CreateFontIndirectW(&lf);
            Gdi::SelectObject(self.dc, f.into());
            let old = self.font.replace(f);
            if !old.is_invalid() {
                let _ = Gdi::DeleteObject(old.into());
            }
        }
    }

    /// Words, lines and boxes from now on in `css` - any CSS colour; black when it is not one.
    pub fn set_color(&self, css: &str) {
        let c = theme::parse_color(css).map(|c| c.colorref()).unwrap_or(0);
        self.color.set(COLORREF(c));
    }

    /// Lines and box edges from now on `width` hundredths of an inch thick.
    pub fn set_pen_width(&self, width: i32) {
        self.pen.set(width.max(1));
    }

    /// `text` with its top-left corner at (`x`, `y`), on one line.
    pub fn draw_text(&self, text: &str, x: i32, y: i32) {
        let w = wide(text);
        let n = w.len().saturating_sub(1);
        unsafe {
            Gdi::SetTextColor(self.dc, self.color.get());
            let _ = Gdi::TextOutW(self.dc, x, y, &w[..n]);
        }
    }

    /// `text` wrapped into `area`, each line placed as `align` says: the height it took.
    pub fn draw_text_in(&self, text: &str, area: Bounds, align: Align) -> i32 {
        let mut w = wide(text);
        let n = w.len().saturating_sub(1);
        let a = match align {
            Align::Left => Gdi::DT_LEFT,
            Align::Center => Gdi::DT_CENTER,
            Align::Right => Gdi::DT_RIGHT,
        };
        let flags = a | Gdi::DT_WORDBREAK | Gdi::DT_NOPREFIX | Gdi::DT_EXPANDTABS;
        // Measured first: what DrawText answers while a page is being recorded is not its height.
        let mut m = RECT { left: area.x, top: area.y, right: area.right(), bottom: area.y };
        let mut r = RECT { left: area.x, top: area.y, right: area.right(), bottom: area.bottom() };
        unsafe {
            crate::win::draw_text_w(self.dc, &mut w[..n], &mut m, flags | Gdi::DT_CALCRECT);
            Gdi::SetTextColor(self.dc, self.color.get());
            crate::win::draw_text_w(self.dc, &mut w[..n], &mut r, flags);
        }
        m.bottom - m.top
    }

    /// How wide and tall `text` is in the font now, wrapped at `width` (0: on one line).
    pub fn measure_text(&self, text: &str, width: i32) -> (i32, i32) {
        let mut w = wide(text);
        let n = w.len().saturating_sub(1);
        let wrap = if width > 0 { Gdi::DT_WORDBREAK } else { Gdi::DT_SINGLELINE };
        let mut r = RECT { left: 0, top: 0, right: width.max(1), bottom: 0 };
        unsafe {
            crate::win::draw_text_w(self.dc, &mut w[..n], &mut r, Gdi::DT_CALCRECT | Gdi::DT_NOPREFIX | Gdi::DT_EXPANDTABS | wrap);
        }
        (r.right - r.left, r.bottom - r.top)
    }

    /// The height of one line of words in the font now.
    pub fn line_height(&self) -> i32 {
        let mut tm = Gdi::TEXTMETRICW::default();
        unsafe {
            let _ = Gdi::GetTextMetricsW(self.dc, &mut tm);
        }
        tm.tmHeight + tm.tmExternalLeading
    }

    fn with_pen(&self, f: impl FnOnce()) {
        unsafe {
            let pen = Gdi::CreatePen(Gdi::PS_SOLID, self.pen.get(), self.color.get());
            let old = Gdi::SelectObject(self.dc, pen.into());
            let old_brush = Gdi::SelectObject(self.dc, Gdi::GetStockObject(Gdi::NULL_BRUSH));
            f();
            Gdi::SelectObject(self.dc, old_brush);
            Gdi::SelectObject(self.dc, old);
            let _ = Gdi::DeleteObject(pen.into());
        }
    }

    /// A straight line from (`x1`, `y1`) to (`x2`, `y2`).
    pub fn draw_line(&self, x1: i32, y1: i32, x2: i32, y2: i32) {
        self.with_pen(|| unsafe {
            let _ = Gdi::MoveToEx(self.dc, x1, y1, None);
            let _ = Gdi::LineTo(self.dc, x2, y2);
        });
    }

    /// The edge of a box.
    pub fn draw_rect(&self, b: Bounds) {
        self.with_pen(|| unsafe {
            let _ = Gdi::Rectangle(self.dc, b.x, b.y, b.right(), b.bottom());
        });
    }

    /// A box filled with `css`, any CSS colour.
    pub fn fill_rect(&self, b: Bounds, css: &str) {
        let c = theme::parse_color(css).unwrap_or(Color::rgb(0, 0, 0));
        unsafe {
            let brush = Gdi::CreateSolidBrush(COLORREF(c.colorref()));
            let _ = Gdi::FillRect(self.dc, &RECT { left: b.x, top: b.y, right: b.right(), bottom: b.bottom() }, brush);
            let _ = Gdi::DeleteObject(brush.into());
        }
    }

    /// The picture in file `path` (PNG, JPEG, BMP, GIF) stretched into `b`: false when it cannot
    /// be read.
    pub fn draw_image(&self, path: &Path, b: Bounds) -> bool {
        let found = crate::picture::resolve(&path.to_string_lossy());
        let Some(img) = found.as_deref().and_then(crate::picture::load) else { return false };
        let (w, h) = (img.w as usize, img.h as usize);
        // Its pixels laid on white paper: a printer cannot blend.
        let mut ds = Gdi::DIBSECTION::default();
        let got = unsafe { Gdi::GetObjectW(img.bmp.into(), std::mem::size_of::<Gdi::DIBSECTION>() as i32, Some(&mut ds as *mut _ as *mut _)) };
        if got == 0 || ds.dsBm.bmBits.is_null() {
            return false;
        }
        let mut bgra = unsafe { std::slice::from_raw_parts(ds.dsBm.bmBits as *const u8, w * h * 4) }.to_vec();
        for p in bgra.as_chunks_mut::<4>().0 {
            let clear = 255 - p[3];
            p[0] = p[0].saturating_add(clear);
            p[1] = p[1].saturating_add(clear);
            p[2] = p[2].saturating_add(clear);
            p[3] = 255;
        }
        let bmi = Gdi::BITMAPINFO { bmiHeader: Gdi::BITMAPINFOHEADER { biSize: std::mem::size_of::<Gdi::BITMAPINFOHEADER>() as u32, biWidth: w as i32, biHeight: -(h as i32), biPlanes: 1, biBitCount: 32, biCompression: Gdi::BI_RGB.0, ..Default::default() }, ..Default::default() };
        unsafe {
            Gdi::SetStretchBltMode(self.dc, Gdi::HALFTONE);
            Gdi::StretchDIBits(self.dc, b.x, b.y, b.width, b.height, 0, 0, w as i32, h as i32, Some(bgra.as_ptr() as *const _), &bmi, Gdi::DIB_RGB_COLORS, Gdi::SRCCOPY) != 0
        }
    }

    /// The device context, for drawing Windows' own way: its units are hundredths of an inch.
    pub fn hdc(&self) -> HDC {
        self.dc
    }
}

impl Drop for PrintPageEventArgs {
    fn drop(&mut self) {
        let f = self.font.get();
        if !f.is_invalid() {
            unsafe {
                Gdi::SelectObject(self.dc, Gdi::GetStockObject(Gdi::SYSTEM_FONT));
                let _ = Gdi::DeleteObject(f.into());
            }
        }
    }
}

/// What the page setup and the print dialog chose, kept for the next time.
#[derive(Clone, Debug)]
struct Settings {
    /// Empty: Windows' default printer.
    printer: String,
    landscape: bool,
    /// Left, top, right, bottom, in hundredths of an inch.
    margins: [i32; 4],
    /// The printer's own settings (DEVMODEW and what its driver adds) the dialogs answered.
    devmode: Vec<u8>,
}

/// WinForms' PrintDocument: the pages a program draws in its PrintPage event, printed, saved or
/// shown first. One on a form is a component, its field named as the form names it.
pub struct PrintDocument {
    pub owner: HWND,
    /// Its id on the form: the PrintPage event comes to the form with it.
    pub id: i32,
    /// The name in the printer's queue and the preview's title - DocumentName.
    pub document_name: String,
    settings: RefCell<Settings>,
    /// Draws each page in place of the PrintPage event - a report draws its own.
    painter: RefCell<Option<Painter>>,
}

/// What draws a page when no form's PrintPage event does.
pub type Painter = Box<dyn Fn(&PrintPageEventArgs)>;

impl std::fmt::Debug for PrintDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrintDocument").field("id", &self.id).field("document_name", &self.document_name).field("settings", &self.settings).finish_non_exhaustive()
    }
}

const MAX_PAGES: usize = 2000;

/// Windows' default printer, when there is one.
pub fn default_printer() -> Option<String> {
    unsafe {
        let mut n = 0u32;
        let _ = pr::GetDefaultPrinterW(None, &mut n);
        if n == 0 {
            return None;
        }
        let mut buf = vec![0u16; n as usize];
        if !pr::GetDefaultPrinterW(Some(PWSTR(buf.as_mut_ptr())), &mut n).as_bool() {
            return None;
        }
        let end = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
        Some(String::from_utf16_lossy(&buf[..end]))
    }
}

/// The printers this computer has - PrinterSettings.InstalledPrinters.
pub fn printers() -> Vec<String> {
    unsafe {
        let flags = pr::PRINTER_ENUM_LOCAL | pr::PRINTER_ENUM_CONNECTIONS;
        let (mut need, mut count) = (0u32, 0u32);
        let _ = pr::EnumPrintersW(flags, PCWSTR::null(), 4, None, &mut need, &mut count);
        if need == 0 {
            return Vec::new();
        }
        let mut buf = vec![0u8; need as usize];
        if pr::EnumPrintersW(flags, PCWSTR::null(), 4, Some(&mut buf), &mut need, &mut count).is_err() {
            return Vec::new();
        }
        let infos = std::slice::from_raw_parts(buf.as_ptr() as *const pr::PRINTER_INFO_4W, count as usize);
        infos.iter().filter_map(|i| i.pPrinterName.to_string().ok()).collect()
    }
}

/// The printer's own settings for `printer`, as its driver gives them.
fn devmode_of(printer: &str) -> Vec<u8> {
    let name = wide(printer);
    unsafe {
        let mut h = pr::PRINTER_HANDLE::default();
        if pr::OpenPrinterW(PCWSTR(name.as_ptr()), &mut h, None).is_err() {
            return Vec::new();
        }
        let size = pr::DocumentPropertiesW(None, h, PCWSTR(name.as_ptr()), None, None, 0);
        let mut buf = Vec::new();
        if size > 0 {
            buf = vec![0u8; size as usize];
            if pr::DocumentPropertiesW(None, h, PCWSTR(name.as_ptr()), Some(buf.as_mut_ptr() as *mut Gdi::DEVMODEW), None, 2 /* DM_OUT_BUFFER */) < 0 {
                buf.clear();
            }
        }
        let _ = pr::ClosePrinter(h);
        buf
    }
}

/// A copy of `bytes` Windows' dialogs can take, or none.
fn global_of(bytes: &[u8]) -> HGLOBAL {
    if bytes.is_empty() {
        return HGLOBAL::default();
    }
    unsafe {
        let Ok(g) = mem::GlobalAlloc(mem::GMEM_MOVEABLE, bytes.len()) else { return HGLOBAL::default() };
        let p = mem::GlobalLock(g) as *mut u8;
        if !p.is_null() {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
            let _ = mem::GlobalUnlock(g);
        }
        g
    }
}

/// What `g` holds, and it given back.
fn take_global(g: HGLOBAL) -> Vec<u8> {
    if g.is_invalid() {
        return Vec::new();
    }
    unsafe {
        let n = mem::GlobalSize(g);
        let p = mem::GlobalLock(g) as *const u8;
        let v = if p.is_null() { Vec::new() } else { std::slice::from_raw_parts(p, n).to_vec() };
        let _ = mem::GlobalUnlock(g);
        let _ = windows::Win32::Foundation::GlobalFree(Some(g));
        v
    }
}

/// The printer named in a DEVNAMES block.
fn printer_of_devnames(bytes: &[u8]) -> Option<String> {
    if bytes.len() < 8 {
        return None;
    }
    let off = u16::from_le_bytes([bytes[2], bytes[3]]) as usize * 2;
    let chars: Vec<u16> = bytes.get(off..)?.as_chunks::<2>().0.iter().map(|c| u16::from_le_bytes(*c)).take_while(|c| *c != 0).collect();
    Some(String::from_utf16_lossy(&chars))
}

fn devmode_ptr(bytes: &[u8]) -> Option<*const Gdi::DEVMODEW> {
    (bytes.len() >= std::mem::size_of::<Gdi::DEVMODEW>()).then_some(bytes.as_ptr() as *const Gdi::DEVMODEW)
}

/// Hundredths of an inch, as `dc`'s device units are measured.
fn units(dc: HDC, offset: bool) {
    unsafe {
        let (dx, dy) = (Gdi::GetDeviceCaps(Some(dc), Gdi::LOGPIXELSX), Gdi::GetDeviceCaps(Some(dc), Gdi::LOGPIXELSY));
        Gdi::SetMapMode(dc, Gdi::MM_ANISOTROPIC);
        let _ = Gdi::SetWindowExtEx(dc, 100, 100, None);
        let _ = Gdi::SetViewportExtEx(dc, dx.max(1), dy.max(1), None);
        // A printer cannot print to the very edge: its device starts where it can, so the
        // paper's corner is that far back.
        let (ox, oy) = if offset { (Gdi::GetDeviceCaps(Some(dc), Gdi::PHYSICALOFFSETX), Gdi::GetDeviceCaps(Some(dc), Gdi::PHYSICALOFFSETY)) } else { (0, 0) };
        let _ = Gdi::SetViewportOrgEx(dc, -ox, -oy, None);
    }
}

/// The sheet `dc` prints on, in hundredths of an inch; A4 for a screen.
fn paper_of(dc: HDC, landscape: bool) -> (i32, i32) {
    unsafe {
        // A screen answers its own width here: only a printer's sheet counts.
        let printer = Gdi::GetDeviceCaps(Some(dc), Gdi::TECHNOLOGY) == Gdi::DT_RASPRINTER as i32;
        let pw = if printer { Gdi::GetDeviceCaps(Some(dc), Gdi::PHYSICALWIDTH) } else { 0 };
        let ph = Gdi::GetDeviceCaps(Some(dc), Gdi::PHYSICALHEIGHT);
        let (dx, dy) = (Gdi::GetDeviceCaps(Some(dc), Gdi::LOGPIXELSX).max(1), Gdi::GetDeviceCaps(Some(dc), Gdi::LOGPIXELSY).max(1));
        if pw > 0 && ph > 0 {
            (pw * 100 / dx, ph * 100 / dy)
        } else if landscape {
            (1169, 827)
        } else {
            (827, 1169)
        }
    }
}

thread_local! {
    static PREVIEW: RefCell<Option<Preview>> = const { RefCell::new(None) };
}

struct Preview {
    pages: Vec<HENHMETAFILE>,
    at: usize,
    paper: (i32, i32),
    print: bool,
    font: HFONT,
    buttons: [HWND; 4],
}

const ID_PRINT: i32 = 1;
const ID_BACK: i32 = 2;
const ID_NEXT: i32 = 3;
const ID_CLOSE: i32 = 4;
const TIMER_SHOT: usize = 0x1F21;

impl PrintDocument {
    /// A document on `owner` with `id`; `landscape` the paper turned sideways.
    pub fn new(owner: HWND, id: i32, document_name: &str, landscape: bool) -> PrintDocument {
        PrintDocument { owner, id, document_name: if document_name.is_empty() { "document".into() } else { document_name.into() }, settings: RefCell::new(Settings { printer: String::new(), landscape, margins: [100; 4], devmode: Vec::new() }), painter: RefCell::new(None) }
    }

    /// Each page drawn by `paint` rather than by the PrintPage event: what a report does.
    pub fn set_painter(&self, paint: Painter) {
        *self.painter.borrow_mut() = Some(paint);
    }

    fn paint(&self, e: &PrintPageEventArgs) {
        match self.painter.borrow().as_ref() {
            Some(p) => p(e),
            None => events::raise(self.owner, self.id, events::PRINT_PAGE, e),
        }
    }

    /// The printer it prints on: empty for Windows' default - PrinterSettings.PrinterName.
    pub fn printer_name(&self) -> String {
        self.settings.borrow().printer.clone()
    }
    pub fn set_printer_name(&self, name: &str) {
        let mut s = self.settings.borrow_mut();
        if s.printer != name {
            s.printer = name.to_string();
            s.devmode.clear();
        }
    }
    /// The paper turned sideways - DefaultPageSettings.Landscape.
    pub fn landscape(&self) -> bool {
        self.settings.borrow().landscape
    }
    pub fn set_landscape(&self, on: bool) {
        self.settings.borrow_mut().landscape = on;
    }
    /// Left, top, right, bottom, in hundredths of an inch - DefaultPageSettings.Margins.
    pub fn margins(&self) -> [i32; 4] {
        self.settings.borrow().margins
    }
    pub fn set_margins(&self, left: i32, top: i32, right: i32, bottom: i32) {
        self.settings.borrow_mut().margins = [left, top, right, bottom];
    }

    /// The printer's name and its settings, the orientation set in them.
    fn device(&self) -> Option<(String, Vec<u8>)> {
        let s = self.settings.borrow().clone();
        let name = if s.printer.is_empty() { default_printer()? } else { s.printer };
        let mut dm = if s.devmode.is_empty() { devmode_of(&name) } else { s.devmode };
        if dm.len() >= std::mem::size_of::<Gdi::DEVMODEW>() {
            unsafe {
                let d = &mut *(dm.as_mut_ptr() as *mut Gdi::DEVMODEW);
                d.dmFields |= Gdi::DM_ORIENTATION;
                d.Anonymous1.Anonymous1.dmOrientation = if s.landscape { 2 } else { 1 };
            }
        }
        Some((name, dm))
    }

    fn printer_dc(&self) -> Option<HDC> {
        let (name, dm) = self.device()?;
        let n = wide(&name);
        let dc = unsafe { Gdi::CreateDCW(windows::core::w!("WINSPOOL"), PCWSTR(n.as_ptr()), PCWSTR::null(), devmode_ptr(&dm)) };
        (!dc.is_invalid()).then_some(dc)
    }

    fn bounds(&self, paper: (i32, i32)) -> (Bounds, Bounds) {
        let [l, t, r, b] = self.margins();
        let page = Bounds { x: 0, y: 0, width: paper.0, height: paper.1 };
        let inner = Bounds { x: l, y: t, width: (paper.0 - l - r).max(0), height: (paper.1 - t - b).max(0) };
        (page, inner)
    }

    /// Every page onto printer `dc`, written to `output` when it is a file: how many.
    fn print_on(&self, dc: HDC, output: Option<&Path>) -> usize {
        let name = wide(&self.document_name);
        let out = output.map(|p| wide(&p.to_string_lossy()));
        let doc = xps::DOCINFOW { cbSize: std::mem::size_of::<xps::DOCINFOW>() as i32, lpszDocName: PCWSTR(name.as_ptr()), lpszOutput: out.as_ref().map(|o| PCWSTR(o.as_ptr())).unwrap_or(PCWSTR::null()), ..Default::default() };
        let mut pages = 0;
        unsafe {
            if xps::StartDocW(dc, &doc) <= 0 {
                let _ = Gdi::DeleteDC(dc);
                return 0;
            }
            let paper = paper_of(dc, self.landscape());
            let (page, inner) = self.bounds(paper);
            loop {
                if xps::StartPage(dc) <= 0 {
                    break;
                }
                units(dc, true);
                pages += 1;
                let e = PrintPageEventArgs::new(dc, pages, page, inner, false);
                self.paint(&e);
                let more = e.has_more_pages();
                drop(e);
                xps::EndPage(dc);
                if !more || pages >= MAX_PAGES {
                    break;
                }
            }
            xps::EndDoc(dc);
            let _ = Gdi::DeleteDC(dc);
        }
        pages
    }

    /// Prints every page on the printer chosen last (Windows' default at first), asking nothing -
    /// WinForms' `Print()`: how many pages; 0 when there is no printer.
    pub fn print(&self) -> usize {
        match self.printer_dc() {
            Some(dc) => self.print_on(dc, None),
            None => 0,
        }
    }

    /// Prints through `printer` into file `path`, asking nothing - with "Microsoft Print to PDF" a
    /// PDF: how many pages.
    pub fn print_to_file(&self, printer: &str, path: &Path) -> usize {
        let keep = self.printer_name();
        self.set_printer_name(printer);
        let n = match self.printer_dc() {
            Some(dc) => self.print_on(dc, Some(path)),
            None => 0,
        };
        self.set_printer_name(&keep);
        n
    }

    /// Windows' Print dialog - which printer, how many copies - then the pages printed on it:
    /// how many; 0 when it was cancelled. WinForms' PrintDialog and `Print()` together.
    pub fn print_dialog(&self) -> usize {
        let s = self.settings.borrow().clone();
        let dm = self.device().map(|d| d.1).unwrap_or(s.devmode.clone());
        let mut pd = dlg::PRINTDLGW { lStructSize: std::mem::size_of::<dlg::PRINTDLGW>() as u32, hwndOwner: self.owner, hDevMode: global_of(&dm), Flags: dlg::PD_RETURNDC | dlg::PD_NOSELECTION | dlg::PD_NOPAGENUMS | dlg::PD_USEDEVMODECOPIESANDCOLLATE, nCopies: 1, ..Default::default() };
        let ok = unsafe { dlg::PrintDlgW(&mut pd) }.as_bool();
        let devmode = take_global(pd.hDevMode);
        let names = take_global(pd.hDevNames);
        if !ok || pd.hDC.is_invalid() {
            return 0;
        }
        {
            let mut st = self.settings.borrow_mut();
            if let Some(p) = printer_of_devnames(&names) {
                st.printer = p;
            }
            if let Some(d) = devmode_ptr(&devmode) {
                st.landscape = unsafe { (*d).Anonymous1.Anonymous1.dmOrientation } == 2;
            }
            st.devmode = devmode;
        }
        self.print_on(pd.hDC, None)
    }

    /// Windows' Page Setup dialog: the paper, which way round, the margins - false when
    /// cancelled. WinForms' PageSetupDialog.
    pub fn page_setup(&self) -> bool {
        let dm = self.device().map(|d| d.1).unwrap_or_default();
        let [l, t, r, b] = self.margins();
        // Thousandths of an inch, as the dialog takes them.
        let mut ps = dlg::PAGESETUPDLGW { lStructSize: std::mem::size_of::<dlg::PAGESETUPDLGW>() as u32, hwndOwner: self.owner, hDevMode: global_of(&dm), Flags: dlg::PSD_MARGINS | dlg::PSD_INTHOUSANDTHSOFINCHES, rtMargin: RECT { left: l * 10, top: t * 10, right: r * 10, bottom: b * 10 }, ..Default::default() };
        let ok = unsafe { dlg::PageSetupDlgW(&mut ps) }.as_bool();
        let devmode = take_global(ps.hDevMode);
        let names = take_global(ps.hDevNames);
        if !ok {
            return false;
        }
        let mut st = self.settings.borrow_mut();
        st.margins = [ps.rtMargin.left / 10, ps.rtMargin.top / 10, ps.rtMargin.right / 10, ps.rtMargin.bottom / 10];
        if let Some(p) = printer_of_devnames(&names) {
            st.printer = p;
        }
        if let Some(d) = devmode_ptr(&devmode) {
            st.landscape = unsafe { (*d).Anonymous1.Anonymous1.dmOrientation } == 2;
        }
        st.devmode = devmode;
        true
    }

    /// Every page drawn into a picture of its own, measured as the printer measures it: what the
    /// preview shows, and the sheet's size.
    fn record(&self) -> (Vec<HENHMETAFILE>, (i32, i32)) {
        let printer = self.printer_dc();
        let screen = unsafe { Gdi::GetDC(None) };
        let reference = printer.unwrap_or(screen);
        let paper = paper_of(reference, self.landscape());
        let (page, inner) = self.bounds(paper);
        // The picture's frame in hundredths of a millimetre.
        let frame = RECT { left: 0, top: 0, right: paper.0 * 254 / 10, bottom: paper.1 * 254 / 10 };
        let mut pages = Vec::new();
        loop {
            let dc = unsafe { Gdi::CreateEnhMetaFileW(Some(reference), PCWSTR::null(), Some(&frame), PCWSTR::null()) };
            if dc.is_invalid() {
                break;
            }
            units(dc, false);
            let e = PrintPageEventArgs::new(dc, pages.len() + 1, page, inner, true);
            self.paint(&e);
            let more = e.has_more_pages();
            drop(e);
            pages.push(unsafe { Gdi::CloseEnhMetaFile(dc) });
            if !more || pages.len() >= MAX_PAGES {
                break;
            }
        }
        unsafe {
            Gdi::ReleaseDC(None, screen);
            if let Some(p) = printer {
                let _ = Gdi::DeleteDC(p);
            }
        }
        (pages, paper)
    }

    /// How many pages the document has now, drawn without printing them.
    pub fn page_count(&self) -> usize {
        let (pages, _) = self.record();
        let n = pages.len();
        for p in pages {
            unsafe {
                let _ = Gdi::DeleteEnhMetaFile(Some(p));
            }
        }
        n
    }

    /// The pages in a window before they are printed - WinForms' PrintPreviewDialog. Its Print
    /// button opens the Print dialog. It waits until the window closes, as `ShowDialog()` does.
    pub fn print_preview(&self) {
        let (pages, paper) = self.record();
        if pages.is_empty() {
            return;
        }
        let class = win::register("PhaiPrintPreview", Some(preview_proc), None);
        let title = wide(&tf("Print preview - {0}", &[&self.document_name]));
        let (w, h) = (win::px(self.owner, 760), win::px(self.owner, 820));
        let hwnd = unsafe { wm::CreateWindowExW(wm::WS_EX_DLGMODALFRAME, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_OVERLAPPEDWINDOW | wm::WS_CLIPCHILDREN, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, w, h, Some(self.owner), None, Some(win::instance()), None) }.unwrap_or_default();
        if hwnd.is_invalid() {
            return;
        }
        let font = win::font(hwnd, 0, false);
        let buttons = [
            win::button(hwnd, t("Print..."), ID_PRINT, font, true),
            win::button(hwnd, "\u{2039}", ID_BACK, font, false),
            win::button(hwnd, "\u{203A}", ID_NEXT, font, false),
            win::button(hwnd, t("Close"), ID_CLOSE, font, false),
        ];
        PREVIEW.with(|p| *p.borrow_mut() = Some(Preview { pages, at: 0, paper, print: false, font, buttons }));
        place_buttons(hwnd);
        win::center_on(hwnd, self.owner);
        let quiet = std::env::var_os("PHAI_SHOT").is_some() || std::env::var_os("PHAI_PREVIEW_SHOT").is_some();
        unsafe {
            let _ = wm::ShowWindow(hwnd, if quiet { wm::SW_SHOWNOACTIVATE } else { wm::SW_SHOW });
            if std::env::var_os("PHAI_PREVIEW_SHOT").is_some() {
                wm::SetTimer(Some(hwnd), TIMER_SHOT, 400, None);
            }
            let _ = km::EnableWindow(self.owner, false);
        }
        // Its own loop until it closes, the owner waiting behind it.
        let mut msg = wm::MSG::default();
        while unsafe { wm::IsWindow(Some(hwnd)) }.as_bool() {
            if !unsafe { wm::GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
                unsafe { wm::PostQuitMessage(msg.wParam.0 as i32) };
                break;
            }
            if msg.message == wm::WM_KEYDOWN && (msg.hwnd == hwnd || unsafe { wm::GetParent(msg.hwnd) }.ok() == Some(hwnd)) {
                match msg.wParam.0 as u16 {
                    0x1B => {
                        unsafe {
                            let _ = wm::DestroyWindow(hwnd);
                        }
                        continue;
                    }
                    0x25 | 0x21 => {
                        turn(hwnd, -1);
                        continue;
                    }
                    0x27 | 0x22 => {
                        turn(hwnd, 1);
                        continue;
                    }
                    _ => {}
                }
            }
            unsafe {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
        }
        unsafe {
            let _ = km::EnableWindow(self.owner, true);
            if !quiet {
                let _ = wm::SetForegroundWindow(self.owner);
            }
        }
        let print = PREVIEW.with(|p| p.borrow_mut().take()).map(|p| {
            for e in p.pages {
                unsafe {
                    let _ = Gdi::DeleteEnhMetaFile(Some(e));
                }
            }
            unsafe {
                let _ = Gdi::DeleteObject(p.font.into());
            }
            p.print
        });
        if print == Some(true) {
            self.print_dialog();
        }
    }
}

/// The strip of buttons along the preview's top.
fn place_buttons(hwnd: HWND) {
    let Some(b) = PREVIEW.with(|p| p.borrow().as_ref().map(|p| p.buttons)) else { return };
    let (pad, bh) = (win::px(hwnd, 10), win::px(hwnd, 30));
    let widths = [win::px(hwnd, 96), win::px(hwnd, 36), win::px(hwnd, 36), win::px(hwnd, 84)];
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(hwnd, &mut r);
    }
    let mut x = pad;
    for (i, h) in b.iter().enumerate().take(3) {
        unsafe {
            let _ = wm::SetWindowPos(*h, None, x, pad, widths[i], bh, wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
        }
        x += widths[i] + win::px(hwnd, 6);
    }
    unsafe {
        let _ = wm::SetWindowPos(b[3], None, r.right - pad - widths[3], pad, widths[3], bh, wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
    }
    enable_turns(hwnd);
}

fn enable_turns(_hwnd: HWND) {
    PREVIEW.with(|p| {
        if let Some(p) = p.borrow().as_ref() {
            win::enable(p.buttons[1], p.at > 0);
            win::enable(p.buttons[2], p.at + 1 < p.pages.len());
        }
    });
}

/// The page `by` before or after the one shown.
fn turn(hwnd: HWND, by: i32) {
    let changed = PREVIEW.with(|p| {
        let mut p = p.borrow_mut();
        let Some(p) = p.as_mut() else { return false };
        let to = (p.at as i32 + by).clamp(0, p.pages.len() as i32 - 1) as usize;
        let changed = to != p.at;
        p.at = to;
        changed
    });
    if changed {
        enable_turns(hwnd);
        unsafe {
            let _ = Gdi::InvalidateRect(Some(hwnd), None, false);
        }
    }
}

fn strip_height(hwnd: HWND) -> i32 {
    win::px(hwnd, 50)
}

fn paint_preview(hwnd: HWND, dc: HDC) {
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(hwnd, &mut r);
    }
    let th = theme::current();
    let surface = th.var_color("--surface").unwrap_or(Color::rgb(0xff, 0xff, 0xff));
    let desk = th.var_color("--rule").unwrap_or(Color::rgb(0xe5, 0xe5, 0xe5));
    let border = th.var_color("--border").unwrap_or(Color::rgb(0xd4, 0xd4, 0xd4));
    let text = th.var_color("--text").unwrap_or(Color::rgb(0x1b, 0x1b, 0x1b));
    let strip = strip_height(hwnd);
    win::fill(dc, &RECT { bottom: strip, ..r }, surface);
    win::fill(dc, &RECT { top: strip, ..r }, desk);
    PREVIEW.with(|p| {
        let p = p.borrow();
        let Some(p) = p.as_ref() else { return };
        // "Page 1 of 3", after the turning buttons.
        unsafe {
            let old = Gdi::SelectObject(dc, p.font.into());
            Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
            Gdi::SetTextColor(dc, COLORREF(text.colorref()));
            let mut words = wide(&tf("Page {0} of {1}", &[&(p.at + 1), &p.pages.len()]));
            let n = words.len().saturating_sub(1);
            let mut tr = RECT { left: win::px(hwnd, 10 + 96 + 6 + 36 + 6 + 36 + 14), top: 0, right: r.right - win::px(hwnd, 110), bottom: strip };
            crate::win::draw_text_w(dc, &mut words[..n], &mut tr, Gdi::DT_SINGLELINE | Gdi::DT_VCENTER | Gdi::DT_NOPREFIX);
            Gdi::SelectObject(dc, old);
        }
        // The sheet as big as the window lets it be, its shape kept.
        let pad = win::px(hwnd, 20);
        let (aw, ah) = ((r.right - 2 * pad).max(1), (r.bottom - strip - 2 * pad).max(1));
        let (pw, ph) = (p.paper.0.max(1), p.paper.1.max(1));
        let (w, h) = if aw * ph > ah * pw { (ah * pw / ph, ah) } else { (aw, aw * ph / pw) };
        let x = (r.right - w) / 2;
        let y = strip + pad + (ah - h) / 2;
        let sheet = RECT { left: x, top: y, right: x + w, bottom: y + h };
        let shadow = win::px(hwnd, 3);
        win::fill(dc, &RECT { left: x + shadow, top: y + shadow, right: x + w + shadow, bottom: y + h + shadow }, border);
        win::fill(dc, &sheet, Color::rgb(0xff, 0xff, 0xff));
        if let Some(e) = p.pages.get(p.at) {
            unsafe {
                let _ = Gdi::PlayEnhMetaFile(dc, *e, &sheet);
            }
        }
    });
}

extern "system" fn preview_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_ERASEBKGND => return LRESULT(1),
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            // Drawn aside first, so turning a page does not flicker.
            let mem = Gdi::CreateCompatibleDC(Some(dc));
            let bmp = Gdi::CreateCompatibleBitmap(dc, r.right.max(1), r.bottom.max(1));
            let old = Gdi::SelectObject(mem, bmp.into());
            paint_preview(h, mem);
            let _ = Gdi::BitBlt(dc, 0, 0, r.right, r.bottom, Some(mem), 0, 0, Gdi::SRCCOPY);
            Gdi::SelectObject(mem, old);
            let _ = Gdi::DeleteObject(bmp.into());
            let _ = Gdi::DeleteDC(mem);
            let _ = Gdi::EndPaint(h, &ps);
            return LRESULT(0);
        },
        wm::WM_SIZE => {
            place_buttons(h);
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
        }
        wm::WM_COMMAND => {
            match (w.0 & 0xFFFF) as i32 {
                ID_PRINT => {
                    PREVIEW.with(|p| {
                        if let Some(p) = p.borrow_mut().as_mut() {
                            p.print = true;
                        }
                    });
                    unsafe {
                        let _ = wm::DestroyWindow(h);
                    }
                }
                ID_BACK => turn(h, -1),
                ID_NEXT => turn(h, 1),
                ID_CLOSE => unsafe {
                    let _ = wm::DestroyWindow(h);
                },
                _ => {}
            }
            return LRESULT(0);
        }
        wm::WM_MOUSEWHEEL => {
            let delta = ((w.0 >> 16) & 0xFFFF) as u16 as i16;
            turn(h, if delta < 0 { 1 } else { -1 });
            return LRESULT(0);
        }
        wm::WM_TIMER if w.0 == TIMER_SHOT => unsafe {
            let _ = wm::KillTimer(Some(h), TIMER_SHOT);
            if let Some(path) = std::env::var_os("PHAI_PREVIEW_SHOT") {
                let _ = crate::shot::save(h, &PathBuf::from(path));
            }
            // A second page in the picture too, when there is one: `<name>-2.png`.
            if let Some(path) = std::env::var_os("PHAI_PREVIEW_SHOT") {
                let more = PREVIEW.with(|p| p.borrow().as_ref().is_some_and(|p| p.pages.len() > 1));
                if more {
                    turn(h, 1);
                    let _ = Gdi::UpdateWindow(h);
                    let p = PathBuf::from(path);
                    let second = p.with_file_name(format!("{}-2.png", p.file_stem().unwrap_or_default().to_string_lossy()));
                    let _ = crate::shot::save(h, &second);
                }
            }
            let _ = wm::DestroyWindow(h);
            return LRESULT(0);
        },
        _ => {}
    }
    win::def(h, msg, w, l)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_devnames_block_gives_its_printer() {
        // DEVNAMES: four offsets in characters, then the strings.
        let mut b: Vec<u8> = Vec::new();
        for v in [4u16, 9, 14, 0] {
            b.extend(v.to_le_bytes());
        }
        for s in ["drv\0\0", "Desk\0", "port\0"] {
            for c in s.encode_utf16() {
                b.extend(c.to_le_bytes());
            }
        }
        assert_eq!(printer_of_devnames(&b).as_deref(), Some("Desk"));
        assert_eq!(printer_of_devnames(&[1, 2]), None);
    }

    #[test]
    fn the_margins_make_the_inner_bounds() {
        let d = PrintDocument::new(HWND::default(), 5, "", false);
        assert_eq!(d.document_name, "document");
        d.set_margins(50, 100, 50, 100);
        let (page, inner) = d.bounds((850, 1100));
        assert_eq!(page, Bounds { x: 0, y: 0, width: 850, height: 1100 });
        assert_eq!(inner, Bounds { x: 50, y: 100, width: 750, height: 900 });
        assert_eq!(inner.right(), 800);
        assert_eq!(inner.bottom(), 1000);
    }
}

