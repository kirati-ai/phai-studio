//! The canvas: the space around the form, and the form itself (the surface) built from the model
//! with real controls - the same calls the generated program makes.
//!
//! The controls never see the mouse: each answers `WM_NCHITTEST` with "transparent", so every
//! click lands on the surface, which finds the control by its rectangle. Selection handles are
//! small windows on top, transparent too. Drags move the real window as the mouse goes and change
//! the model when the button comes up - one undo step per drag.

use phai_form::live;
use phai_form::{Control, Kind};
use phai_ui::layout::{scale, unscale};
use phai_ui::theme::{self, Color, State};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HMENU, WINDOW_EX_STYLE};

use crate::studio;

/// The form's top-left corner inside the canvas, design pixels.
pub const MARGIN: i32 = 16;
/// The form's title bar on the canvas, design pixels - Windows 11's.
pub const TITLE: i32 = 31;
/// The form's menu bar on the canvas, when it has menus.
pub const MENU: i32 = 22;

/// The height of the window frame drawn above the form: its title bar, and its menu bar.
pub fn chrome(f: &phai_form::FormFile) -> i32 {
    TITLE + if f.menu.is_empty() { 0 } else { MENU }
}

/// Where the form's client area starts on the canvas, design pixels.
pub fn origin(f: &phai_form::FormFile) -> (i32, i32) {
    (MARGIN, MARGIN + chrome(f))
}
/// Positions snap to this many design pixels.
pub const GRID: i32 = 4;
const HANDLE: i32 = 7;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Drag {
    None,
    Create { kind: Kind, from: (i32, i32), to: (i32, i32) },
    Move { index: usize, from: (i32, i32), rect: [i32; 4], now: [i32; 4] },
    Resize { index: usize, handle: usize, from: (i32, i32), rect: [i32; 4], now: [i32; 4] },
    Form { handle: usize, from: (i32, i32), size: [i32; 2], now: [i32; 2] },
}

pub struct Canvas {
    pub hwnd: HWND,
    pub surface: HWND,
    live: Vec<HWND>,
    toolbar: Option<HWND>,
    handles: Vec<HWND>,
    form_handles: Vec<HWND>,
    drag: Drag,
    cursor: PCWSTR,
}

fn snap(v: i32) -> i32 {
    (v as f32 / GRID as f32).round() as i32 * GRID
}

fn register_classes() -> (Vec<u16>, Vec<u16>, Vec<u16>) {
    (win::register("PhaiStudioCanvas", Some(canvas_proc), None), win::register_with(win::ClassStyle::DoubleClicks, "PhaiStudioSurface", Some(surface_proc), None), win::register("PhaiStudioHandle", Some(handle_proc), None))
}

fn create_child(class: &[u16], parent: HWND, ex: WINDOW_EX_STYLE) -> HWND {
    unsafe {
        wm::CreateWindowExW(ex, PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPSIBLINGS | wm::WS_CLIPCHILDREN, 0, 0, 10, 10, Some(parent), Some(HMENU::default()), Some(win::instance()), None).unwrap_or_default()
    }
}

impl Canvas {
    pub fn new(parent: HWND, id: i32) -> Canvas {
        let (canvas_class, surface_class, _) = register_classes();
        let hwnd = unsafe {
            wm::CreateWindowExW(Default::default(), PCWSTR(canvas_class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPCHILDREN | wm::WS_CLIPSIBLINGS, 0, 0, 10, 10, Some(parent), Some(HMENU(id as isize as *mut _)), Some(win::instance()), None).unwrap_or_default()
        };
        let surface = create_child(&surface_class, hwnd, WINDOW_EX_STYLE(0));
        Canvas { hwnd, surface, live: vec![], toolbar: None, handles: vec![], form_handles: vec![], drag: Drag::None, cursor: wm::IDC_ARROW }
    }

    /// Builds the form again from the model: size, controls, handles.
    pub fn rebuild(&mut self) {
        let Some(s) = studio() else { return };
        unsafe {
            wm::SendMessageW(self.surface, wm::WM_SETREDRAW, Some(WPARAM(0)), None);
            for h in self.live.drain(..).chain(self.toolbar.take()).chain(self.handles.drain(..)) {
                let _ = wm::DestroyWindow(h);
            }
            for h in self.form_handles.drain(..) {
                let _ = wm::DestroyWindow(h);
            }
        }
        let f = &s.doc.form;
        let (w, h) = (win::px(self.hwnd, f.form.size[0]), win::px(self.hwnd, f.form.size[1]));
        let (ox, oy) = origin(f);
        win::move_to(self.surface, win::px(self.hwnd, ox), win::px(self.hwnd, oy), w, h);
        let built = live::build(self.surface, f, s.font, 0);
        built.layout.apply(self.surface);
        for h in built.controls.iter().copied().chain(built.toolbar) {
            ghost(h);
        }
        self.live = built.controls;
        self.toolbar = built.toolbar;
        let (_, _, handle_class) = register_classes();
        self.handles = (0..8).map(|_| create_child(&handle_class, self.surface, WINDOW_EX_STYLE(0))).collect();
        self.form_handles = (0..3).map(|_| create_child(&handle_class, self.hwnd, WINDOW_EX_STYLE(0))).collect();
        unsafe {
            wm::SendMessageW(self.surface, wm::WM_SETREDRAW, Some(WPARAM(1)), None);
            let _ = Gdi::RedrawWindow(Some(self.hwnd), None, None, Gdi::RDW_INVALIDATE | Gdi::RDW_ERASE | Gdi::RDW_ALLCHILDREN | Gdi::RDW_FRAME);
        }
        self.place_handles();
    }

    /// The design rectangle control `i` really has now - docked ones included.
    fn rect_of(&self, i: usize) -> [i32; 4] {
        let Some(&h) = self.live.get(i) else { return [0; 4] };
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetWindowRect(h, &mut r);
            let mut pts = [POINT { x: r.left, y: r.top }, POINT { x: r.right, y: r.bottom }];
            Gdi::MapWindowPoints(None, Some(self.surface), &mut pts);
            let dpi = win::dpi(self.surface);
            [unscale(pts[0].x, dpi), unscale(pts[0].y, dpi), unscale(pts[1].x - pts[0].x, dpi), unscale(pts[1].y - pts[0].y, dpi)]
        }
    }

    /// The 8 handles around `r` (design px): corners and middles, clockwise from top-left.
    fn handle_points(r: [i32; 4]) -> [(i32, i32); 8] {
        let [x, y, w, h] = r;
        [(x, y), (x + w / 2, y), (x + w, y), (x + w, y + h / 2), (x + w, y + h), (x + w / 2, y + h), (x, y + h), (x, y + h / 2)]
    }

    pub fn place_handles(&self) {
        let Some(s) = studio() else { return };
        let dpi = win::dpi(self.surface);
        let hs = scale(HANDLE, dpi);
        let shown = match self.drag {
            Drag::Create { from, to, .. } => Some(norm(from, to)),
            Drag::Move { now, .. } | Drag::Resize { now, .. } => Some(now),
            _ => s.selected.map(|i| self.rect_of(i)),
        };
        for (i, h) in self.handles.iter().enumerate() {
            match shown {
                Some(r) => {
                    let (x, y) = Self::handle_points(r)[i];
                    win::move_to(*h, scale(x, dpi) - hs / 2, scale(y, dpi) - hs / 2, hs, hs);
                    unsafe {
                        let _ = wm::SetWindowPos(*h, Some(wm::HWND_TOP), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOACTIVATE | wm::SWP_SHOWWINDOW);
                    }
                }
                None => win::show(*h, false),
            }
        }
        // The form's own handles, outside it on the canvas: right, bottom-right, bottom.
        let size = match self.drag {
            Drag::Form { now, .. } => now,
            _ => s.doc.form.form.size,
        };
        let (m, top) = origin(&s.doc.form);
        let pts = [(m + size[0], top + size[1] / 2), (m + size[0], top + size[1]), (m + size[0] / 2, top + size[1])];
        for (h, (x, y)) in self.form_handles.iter().zip(pts) {
            win::move_to(*h, scale(x, dpi) - hs / 2 + hs / 2, scale(y, dpi) - hs / 2 + hs / 2, hs, hs);
            win::show(*h, s.selected.is_none());
        }
    }

    /// The form's size as shown now - while its edge is dragged, the dragged size.
    pub fn shown_size(&self) -> Option<[i32; 2]> {
        match self.drag {
            Drag::Form { now, .. } => Some(now),
            _ => None,
        }
    }

    fn design_point(&self, h: HWND, l: LPARAM) -> (i32, i32) {
        let (x, y) = win::point_of(l);
        let dpi = win::dpi(h);
        (unscale(x, dpi), unscale(y, dpi))
    }

    /// The topmost control under design point `p`, by its real rectangle.
    fn hit(&self, p: (i32, i32)) -> Option<usize> {
        let s = studio()?;
        (0..self.live.len()).rev().filter(|i| unsafe { wm::IsWindowVisible(self.live[*i]) }.as_bool()).find(|i| {
            let [x, y, w, h] = self.rect_of(*i);
            // A group is chosen by its frame and title only, so what is inside it can be.
            if s.doc.form.control[*i].kind == Kind::Group {
                let inside = p.0 > x + 6 && p.0 < x + w - 6 && p.1 > y + 16 && p.1 < y + h - 6;
                return p.0 >= x && p.0 < x + w && p.1 >= y && p.1 < y + h && !inside;
            }
            p.0 >= x && p.0 < x + w && p.1 >= y && p.1 < y + h
        })
    }

    fn hit_handle(&self, p: (i32, i32)) -> Option<usize> {
        let s = studio()?;
        let i = s.selected?;
        if s.doc.form.control[i].dock.as_deref().is_some_and(|d| d != "none") {
            return None;
        }
        let r = self.rect_of(i);
        Self::handle_points(r).iter().position(|(x, y)| (p.0 - x).abs() <= HANDLE / 2 + 1 && (p.1 - y).abs() <= HANDLE / 2 + 1)
    }

    fn cursor_for_handle(handle: usize) -> PCWSTR {
        match handle {
            0 | 4 => wm::IDC_SIZENWSE,
            2 | 6 => wm::IDC_SIZENESW,
            1 | 5 => wm::IDC_SIZENS,
            _ => wm::IDC_SIZEWE,
        }
    }

    fn set_cursor(&mut self, c: PCWSTR) {
        self.cursor = c;
        unsafe {
            wm::SetCursor(wm::LoadCursorW(None, c).ok());
        }
    }

    // --- the surface's mouse ---

    fn down(&mut self, l: LPARAM) {
        let Some(s) = studio() else { return };
        let p = self.design_point(self.surface, l);
        unsafe {
            let _ = km::SetFocus(Some(self.surface));
            km::SetCapture(self.surface);
        }
        if let Some(kind) = s.tool {
            let at = (snap(p.0), snap(p.1));
            self.drag = Drag::Create { kind, from: at, to: at };
        } else if let (Some(i), Some(handle)) = (s.selected, self.hit_handle(p)) {
            let r = self.rect_of(i);
            self.drag = Drag::Resize { index: i, handle, from: p, rect: r, now: r };
        } else if let Some(i) = self.hit(p) {
            s.select(Some(i));
            let docked = s.doc.form.control[i].dock.as_deref().is_some_and(|d| d != "none");
            if !docked {
                let r = self.rect_of(i);
                self.drag = Drag::Move { index: i, from: p, rect: r, now: r };
            }
        } else {
            s.select(None);
        }
        self.place_handles();
    }

    fn moved(&mut self, l: LPARAM) {
        let Some(s) = studio() else { return };
        let p = self.design_point(self.surface, l);
        let dpi = win::dpi(self.surface);
        match &mut self.drag {
            Drag::Create { to, .. } => *to = (snap(p.0), snap(p.1)),
            Drag::Move { index, from, rect, now } => {
                *now = [snap(rect[0] + p.0 - from.0), snap(rect[1] + p.1 - from.1), rect[2], rect[3]];
                let n = *now;
                win::move_to(self.live[*index], scale(n[0], dpi), scale(n[1], dpi), scale(n[2], dpi), scale(n[3], dpi));
            }
            Drag::Resize { index, handle, from, rect, now } => {
                let (dx, dy) = (p.0 - from.0, p.1 - from.1);
                let [mut x, mut y, mut w, mut h] = *rect;
                if matches!(*handle, 0 | 6 | 7) {
                    let nx = snap(x + dx);
                    w += x - nx;
                    x = nx;
                }
                if matches!(*handle, 2..=4) {
                    w = snap(x + w + dx) - x;
                }
                if matches!(*handle, 0..=2) {
                    let ny = snap(y + dy);
                    h += y - ny;
                    y = ny;
                }
                if matches!(*handle, 4..=6) {
                    h = snap(y + h + dy) - y;
                }
                *now = [x, y, w.max(GRID), h.max(GRID)];
                let n = *now;
                win::move_to(self.live[*index], scale(n[0], dpi), scale(n[1], dpi), scale(n[2], dpi), scale(n[3], dpi));
            }
            Drag::Form { .. } => {}
            Drag::None => {
                let c = if s.tool.is_some() {
                    wm::IDC_CROSS
                } else if let Some(handle) = self.hit_handle(p) {
                    Self::cursor_for_handle(handle)
                } else if self.hit(p).is_some_and(|i| Some(i) == s.selected) {
                    wm::IDC_SIZEALL
                } else {
                    wm::IDC_ARROW
                };
                self.set_cursor(c);
                return;
            }
        }
        self.say_drag();
        self.place_handles();
    }

    fn up(&mut self) {
        let Some(s) = studio() else { return };
        unsafe {
            let _ = km::ReleaseCapture();
        }
        let drag = std::mem::replace(&mut self.drag, Drag::None);
        match drag {
            Drag::Create { kind, from, to } => {
                let mut r = norm(from, to);
                if r[2] < 8 && r[3] < 8 {
                    let (w, h) = kind.default_size();
                    r[2] = w;
                    r[3] = h;
                }
                s.edit(|f| {
                    let id = f.next_id(kind);
                    let mut c = Control::new(kind, &id, r);
                    if kind == Kind::Statusbar {
                        c.bounds = Some([0, 0, 0, 26]);
                    }
                    f.control.push(c);
                });
                s.tool = None;
                s.toolbox_reset();
                let last = s.doc.form.control.len() - 1;
                s.select(Some(last));
                self.rebuild();
            }
            Drag::Move { index, rect, now, .. } | Drag::Resize { index, rect, now, .. } => {
                if now != rect {
                    s.edit(|f| f.control[index].bounds = Some(now));
                    self.rebuild();
                    s.refresh_props();
                }
            }
            Drag::Form { size, now, .. } => {
                if now != size {
                    s.edit(|f| f.form.size = now);
                    self.rebuild();
                    s.refresh_props();
                }
            }
            Drag::None => {}
        }
        self.place_handles();
        s.say_selection();
    }

    fn say_drag(&self) {
        let Some(s) = studio() else { return };
        let text = match self.drag {
            Drag::Create { from, to, .. } => {
                let r = norm(from, to);
                format!("{}, {}  ·  {} × {}", r[0], r[1], r[2], r[3])
            }
            Drag::Move { now, .. } | Drag::Resize { now, .. } => format!("{}, {}  ·  {} × {}", now[0], now[1], now[2], now[3]),
            Drag::Form { now, .. } => format!("{} × {}", now[0], now[1]),
            Drag::None => return,
        };
        phai_ui::controls::status_text(s.status, 1, &text);
    }

    // --- the canvas's mouse: the form's own handles ---

    fn canvas_down(&mut self, l: LPARAM) {
        let Some(s) = studio() else { return };
        let p = self.design_point(self.hwnd, l);
        let size = s.doc.form.form.size;
        let (m, top) = origin(&s.doc.form);
        let pts = [(m + size[0], top + size[1] / 2), (m + size[0], top + size[1]), (m + size[0] / 2, top + size[1])];
        unsafe {
            let _ = km::SetFocus(Some(self.surface));
        }
        // The frame above the form: its menu bar opens the menus to edit, its title bar chooses the form.
        let inside_x = p.0 >= MARGIN && p.0 < MARGIN + size[0];
        if inside_x && p.1 >= MARGIN + TITLE && p.1 < top {
            s.select(None);
            self.place_handles();
            s.edit_menus();
            return;
        }
        if s.selected.is_none() {
            if let Some(handle) = pts.iter().position(|(x, y)| (p.0 - x).abs() <= HANDLE && (p.1 - y).abs() <= HANDLE) {
                unsafe {
                    km::SetCapture(self.hwnd);
                }
                self.drag = Drag::Form { handle, from: p, size, now: size };
                return;
            }
        }
        s.select(None);
        self.place_handles();
    }

    fn canvas_moved(&mut self, l: LPARAM) {
        let p = self.design_point(self.hwnd, l);
        if let Drag::Form { handle, from, size, now } = &mut self.drag {
            let w = if *handle <= 1 { snap(size[0] + p.0 - from.0) } else { size[0] };
            let h = if *handle >= 1 { snap(size[1] + p.1 - from.1) } else { size[1] };
            *now = [w.max(120), h.max(80)];
            let dpi = win::dpi(self.hwnd);
            let n = *now;
            let (ox, oy) = studio().map(|s| origin(&s.doc.form)).unwrap_or((MARGIN, MARGIN));
            win::move_to(self.surface, scale(ox, dpi), scale(oy, dpi), scale(n[0], dpi), scale(n[1], dpi));
            let _ = unsafe { Gdi::InvalidateRect(Some(self.hwnd), None, true) };
            self.say_drag();
            self.place_handles();
        } else {
            let Some(s) = studio() else { return };
            let size = s.doc.form.form.size;
            let (m, top) = origin(&s.doc.form);
            let c = if s.selected.is_none() && (p.0 - (m + size[0])).abs() <= HANDLE && (p.1 - (top + size[1])).abs() <= HANDLE {
                wm::IDC_SIZENWSE
            } else if s.selected.is_none() && (p.0 - (m + size[0])).abs() <= HANDLE && (p.1 - (top + size[1] / 2)).abs() <= HANDLE {
                wm::IDC_SIZEWE
            } else if s.selected.is_none() && (p.1 - (top + size[1])).abs() <= HANDLE && (p.0 - (m + size[0] / 2)).abs() <= HANDLE {
                wm::IDC_SIZENS
            } else {
                wm::IDC_ARROW
            };
            self.set_cursor(c);
        }
    }

    /// Arrow keys move the chosen control a pixel, with Shift they size it; Delete removes it.
    fn key(&mut self, vk: u16) -> bool {
        let Some(s) = studio() else { return false };
        let Some(i) = s.selected else { return false };
        let shift = unsafe { km::GetKeyState(km::VK_SHIFT.0 as i32) } < 0;
        let (dx, dy) = match vk {
            0x25 => (-1, 0),
            0x27 => (1, 0),
            0x26 => (0, -1),
            0x28 => (0, 1),
            0x2E => {
                s.delete_selected();
                return true;
            }
            _ => return false,
        };
        s.edit(|f| {
            let c = &mut f.control[i];
            let [x, y, w, h] = c.rect();
            c.bounds = Some(if shift { [x, y, (w + dx).max(1), (h + dy).max(1)] } else { [x + dx, y + dy, w, h] });
        });
        self.rebuild();
        s.refresh_props();
        s.say_selection();
        true
    }
}

fn norm(a: (i32, i32), b: (i32, i32)) -> [i32; 4] {
    [a.0.min(b.0), a.1.min(b.1), (a.0 - b.0).abs(), (a.1 - b.1).abs()]
}

/// Makes a control and everything inside it deaf to the mouse: clicks reach the surface.
fn ghost(h: HWND) {
    unsafe extern "system" fn deaf(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, _: usize) -> LRESULT {
        match msg {
            wm::WM_NCHITTEST => LRESULT(-1), // HTTRANSPARENT
            wm::WM_SETFOCUS => {
                // Typing goes to the canvas, never into a control being designed.
                if let Some(c) = studio().and_then(|s| s.canvas.as_ref()) {
                    let _ = km::SetFocus(Some(c.surface));
                }
                LRESULT(0)
            }
            wm::WM_NCDESTROY => {
                let _ = windows::Win32::UI::Shell::RemoveWindowSubclass(h, Some(deaf), 77);
                windows::Win32::UI::Shell::DefSubclassProc(h, msg, w, l)
            }
            _ => windows::Win32::UI::Shell::DefSubclassProc(h, msg, w, l),
        }
    }
    unsafe extern "system" fn each(h: HWND, _: LPARAM) -> windows::core::BOOL {
        let _ = windows::Win32::UI::Shell::SetWindowSubclass(h, Some(deaf), 77, 0);
        true.into()
    }
    unsafe {
        let _ = windows::Win32::UI::Shell::SetWindowSubclass(h, Some(deaf), 77, 0);
        let _ = wm::EnumChildWindows(Some(h), Some(each), LPARAM(0));
    }
}

/// The window around the form, as Windows 11 draws one: a border, the title with the minimise,
/// maximise and close glyphs, and the menu bar with the form's menus.
fn paint_frame(dc: Gdi::HDC, h: HWND, f: &phai_form::FormFile, size: [i32; 2]) {
    let dpi = win::dpi(h);
    let p = |v: i32| scale(v, dpi);
    let (ox, _) = origin(f);
    let top = MARGIN;
    let width = size[0];
    let bottom = origin(f).1 + size[1];
    let window = win::window_background();
    let text = win::window_text();
    let border = theme::style("group", State::default()).border_or(Color::rgb(229, 229, 229));
    // Border, one pixel outside.
    let outer = RECT { left: p(ox) - 1, top: p(top) - 1, right: p(ox + width) + 1, bottom: p(bottom) + 1 };
    win::fill(dc, &outer, border);
    // Title bar: a shade off the window, as Windows 11's is.
    let title_bg = if window.is_dark() { Color::rgb(32, 32, 32) } else { Color::rgb(238, 242, 246) };
    let title = RECT { left: p(ox), top: p(top), right: p(ox + width), bottom: p(top + TITLE) };
    win::fill(dc, &title, title_bg);
    let font = win::font(h, 0, false);
    let old = unsafe { Gdi::SelectObject(dc, font.into()) };
    unsafe {
        Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(text.colorref()));
    }
    let mut tr = RECT { left: p(ox + 12), ..title };
    let mut words: Vec<u16> = f.form.title.encode_utf16().collect();
    if !words.is_empty() {
        unsafe {
            Gdi::DrawTextW(dc, &mut words, &mut tr, Gdi::DT_LEFT | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX | Gdi::DT_END_ELLIPSIS);
        }
    }
    // Caption glyphs from the icon font: minimise, maximise, close.
    let mut lf = Gdi::LOGFONTW { lfHeight: -p(10), lfCharSet: Gdi::DEFAULT_CHARSET, lfQuality: Gdi::ANTIALIASED_QUALITY, ..Default::default() };
    for (i, u) in "Segoe Fluent Icons".encode_utf16().enumerate() {
        lf.lfFaceName[i] = u;
    }
    let icons = unsafe { Gdi::CreateFontIndirectW(&lf) };
    unsafe {
        Gdi::SelectObject(dc, icons.into());
    }
    for (k, g) in [0xE8BBu16, 0xE922, 0xE921].iter().enumerate() {
        let right = ox + width - 46 * k as i32;
        let mut gr = RECT { left: p(right - 46), top: p(top), right: p(right), bottom: p(top + TITLE) };
        let mut one = [*g];
        unsafe {
            Gdi::DrawTextW(dc, &mut one, &mut gr, Gdi::DT_CENTER | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE);
        }
    }
    unsafe {
        Gdi::SelectObject(dc, font.into());
    }
    // The menu bar, on the window's own colour.
    if !f.menu.is_empty() {
        let bar = RECT { left: p(ox), top: p(top + TITLE), right: p(ox + width), bottom: p(top + TITLE + MENU) };
        win::fill(dc, &bar, window);
        let mut x = ox + 8;
        for m in &f.menu {
            let label = phai_form::bars::shown(&m.label);
            let (w, _) = win::measure(h, font, &label);
            let lw = phai_ui::layout::unscale(w, dpi) + 16;
            let mut mr = RECT { left: p(x), top: bar.top, right: p(x + lw), bottom: bar.bottom };
            let mut words: Vec<u16> = m.label.encode_utf16().collect();
            unsafe {
                Gdi::DrawTextW(dc, &mut words, &mut mr, Gdi::DT_CENTER | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE);
            }
            x += lw;
        }
    }
    unsafe {
        Gdi::SelectObject(dc, old);
        let _ = Gdi::DeleteObject(icons.into());
        let _ = Gdi::DeleteObject(font.into());
    }
}

fn canvas_of(h: HWND) -> Option<&'static mut Canvas> {
    let s = studio()?;
    let c = s.canvas.as_mut()?;
    (c.hwnd == h || c.surface == h).then_some(c)
}

extern "system" fn canvas_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_ERASEBKGND => {
            let dc = Gdi::HDC(w.0 as *mut _);
            let mut r = RECT::default();
            unsafe {
                let _ = wm::GetClientRect(h, &mut r);
            }
            let bg = theme::style("canvas", State::default()).background_or(Color::rgb(243, 243, 243));
            win::fill(dc, &r, bg);
            if let Some(s) = studio() {
                paint_frame(dc, h, &s.doc.form, s.drag_size());
            }
            LRESULT(1)
        }
        wm::WM_LBUTTONDOWN => {
            if let Some(c) = canvas_of(h) {
                c.canvas_down(l);
            }
            LRESULT(0)
        }
        wm::WM_MOUSEMOVE => {
            if let Some(c) = canvas_of(h) {
                c.canvas_moved(l);
            }
            LRESULT(0)
        }
        wm::WM_LBUTTONUP => {
            if let Some(c) = canvas_of(h) {
                if matches!(c.drag, Drag::Form { .. }) {
                    c.up();
                }
            }
            LRESULT(0)
        }
        wm::WM_SETCURSOR if win::loword(l.0 as usize) == 1 => {
            if let Some(c) = canvas_of(h) {
                unsafe {
                    wm::SetCursor(wm::LoadCursorW(None, c.cursor).ok());
                }
                return LRESULT(1);
            }
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

extern "system" fn surface_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_LBUTTONDOWN => {
            if let Some(c) = canvas_of(h) {
                c.down(l);
            }
            LRESULT(0)
        }
        wm::WM_LBUTTONDBLCLK => {
            if let Some(s) = studio() {
                let y = phai_ui::layout::unscale(win::point_of(l).1, win::dpi(h));
                if !s.doc.form.toolbar.is_empty() && y < win::TOOLBAR_HEIGHT && s.selected.is_none() {
                    s.edit_toolbar();
                } else {
                    s.add_handler_for_selected();
                }
            }
            LRESULT(0)
        }
        wm::WM_MOUSEMOVE => {
            if let Some(c) = canvas_of(h) {
                c.moved(l);
            }
            LRESULT(0)
        }
        wm::WM_LBUTTONUP => {
            if let Some(c) = canvas_of(h) {
                c.up();
            }
            LRESULT(0)
        }
        wm::WM_SETCURSOR if win::loword(l.0 as usize) == 1 => {
            if let Some(c) = canvas_of(h) {
                unsafe {
                    wm::SetCursor(wm::LoadCursorW(None, c.cursor).ok());
                }
                return LRESULT(1);
            }
            win::def(h, msg, w, l)
        }
        wm::WM_GETDLGCODE => LRESULT(0x0001 | 0x0004), // arrows and every key: the canvas's
        wm::WM_KEYDOWN => {
            if let Some(c) = canvas_of(h) {
                if c.key(w.0 as u16) {
                    return LRESULT(0);
                }
            }
            win::def(h, msg, w, l)
        }
        // The surface is the form: it draws the form's background, and nothing it holds may send
        // a command anywhere.
        wm::WM_COMMAND => LRESULT(0),
        _ => win::def(h, msg, w, l),
    }
}

extern "system" fn handle_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_NCHITTEST => LRESULT(-1),
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            let accent = theme::current().var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f));
            win::fill(dc, &r, accent);
            let inner = RECT { left: r.left + 1, top: r.top + 1, right: r.right - 1, bottom: r.bottom - 1 };
            win::fill(dc, &inner, Color::rgb(255, 255, 255));
            let core = RECT { left: r.left + 2, top: r.top + 2, right: r.right - 2, bottom: r.bottom - 2 };
            win::fill(dc, &core, accent);
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        _ => win::def(h, msg, w, l),
    }
}
