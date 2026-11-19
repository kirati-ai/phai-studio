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

/// The component tray under the form, as Visual Studio's: one chip per component (a timer) -
/// its index in the form and its rectangle, design pixels.
pub fn tray_chips(f: &phai_form::FormFile, size: [i32; 2]) -> Vec<(usize, [i32; 4])> {
    let (ox, top) = origin(f);
    let y = top + size[1] + 24;
    let mut x = ox;
    let mut out = Vec::new();
    for (i, c) in f.control.iter().enumerate().filter(|(_, c)| c.kind.is_component()) {
        let w = c.id.chars().count() as i32 * 7 + 44;
        out.push((i, [x, y, w, 28]));
        x += w + 8;
    }
    out
}

/// Where the form's client area starts on the canvas, design pixels.
pub fn origin(f: &phai_form::FormFile) -> (i32, i32) {
    (MARGIN, MARGIN + chrome(f))
}
/// Positions snap to this many design pixels.
pub const GRID: i32 = 4;
const HANDLE: i32 = 7;
/// The smart tag's size, design pixels.
const TAG: i32 = 15;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Drag {
    None,
    Create { kind: Kind, from: (i32, i32), to: (i32, i32) },
    /// Every chosen control moving together; where each started is in `Canvas::group`.
    Move { from: (i32, i32), d: (i32, i32) },
    /// A rectangle drawn on empty form: what it touches is chosen.
    Band { from: (i32, i32), to: (i32, i32) },
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
    /// The smart tag: the small ▸ by the chosen control that opens its common tasks.
    tag: HWND,
    /// Corner marks on the chosen controls other than the first.
    marks: Vec<HWND>,
    /// The chosen controls and where each was when a move began.
    group: Vec<(usize, [i32; 4])>,
    /// Snaplines on show while a move snaps.
    lines: Vec<HWND>,
    drag: Drag,
    cursor: PCWSTR,
    /// Format > Lock controls: they can be chosen, not moved or sized.
    pub locked: bool,
    /// Tab order mode: the number the next control clicked gets; `None` when off.
    tab_next: Option<usize>,
    /// Tab order mode's numbers on the controls.
    numbers: Vec<HWND>,
}

fn snap(v: i32) -> i32 {
    (v as f32 / GRID as f32).round() as i32 * GRID
}

fn register_mark() -> Vec<u16> {
    win::register("PhaiStudioMark", Some(mark_proc), None)
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
        Canvas { hwnd, surface, live: vec![], toolbar: None, handles: vec![], form_handles: vec![], tag: HWND::default(), marks: vec![], group: vec![], lines: vec![], drag: Drag::None, cursor: wm::IDC_ARROW, locked: false, tab_next: None, numbers: vec![] }
    }

    /// Builds the form again from the model: size, controls, handles.
    /// The window the canvas built for control `i` - to ask the theme what it gives it.
    pub fn control_window(&self, i: usize) -> Option<HWND> {
        self.live.get(i).copied()
    }

    pub fn rebuild(&mut self) {
        let Some(s) = studio() else { return };
        unsafe {
            wm::SendMessageW(self.surface, wm::WM_SETREDRAW, Some(WPARAM(0)), None);
            for h in self.live.drain(..).chain(self.toolbar.take()).chain(self.handles.drain(..)) {
                let _ = wm::DestroyWindow(h);
            }
            for h in self.marks.drain(..).chain(self.lines.drain(..)).chain(self.numbers.drain(..)) {
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
        live::arrange(f, &built.controls);
        for h in built.controls.iter().copied().chain(built.toolbar) {
            ghost(h);
        }
        self.live = built.controls;
        self.toolbar = built.toolbar;
        let (_, _, handle_class) = register_classes();
        self.handles = (0..8).map(|_| create_child(&handle_class, self.surface, WINDOW_EX_STYLE(0))).collect();
        self.form_handles = (0..3).map(|_| create_child(&handle_class, self.hwnd, WINDOW_EX_STYLE(0))).collect();
        if !self.tag.is_invalid() {
            unsafe {
                let _ = wm::DestroyWindow(self.tag);
            }
        }
        let tag_class = win::register("PhaiSmartTag", Some(tag_proc), None);
        self.tag = create_child(&tag_class, self.surface, WINDOW_EX_STYLE(0));
        win::show(self.tag, false);
        unsafe {
            wm::SendMessageW(self.surface, wm::WM_SETREDRAW, Some(WPARAM(1)), None);
            let _ = Gdi::RedrawWindow(Some(self.hwnd), None, None, Gdi::RDW_INVALIDATE | Gdi::RDW_ERASE | Gdi::RDW_ALLCHILDREN | Gdi::RDW_FRAME);
        }
        self.place_handles();
        self.show_numbers();
    }

    /// Tab order mode on or off: as Visual Studio's View > Tab Order, each control shows its place
    /// and a click gives it the next.
    pub fn set_tab_mode(&mut self, on: bool) {
        self.tab_next = on.then_some(0);
        self.show_numbers();
        if let Some(s) = studio() {
            s.select(None);
        }
    }

    pub fn tab_mode(&self) -> bool {
        self.tab_next.is_some()
    }

    fn show_numbers(&mut self) {
        for h in self.numbers.drain(..) {
            unsafe {
                let _ = wm::DestroyWindow(h);
            }
        }
        let Some(next) = self.tab_next else { return };
        let class = win::register("PhaiStudioTabNumber", Some(number_proc), None);
        let dpi = win::dpi(self.surface);
        for i in 0..self.live.len() {
            if !unsafe { wm::IsWindowVisible(self.live[i]) }.as_bool() {
                continue;
            }
            let [x, y, _, _] = self.rect_of(i);
            let h = create_child(&class, self.surface, WINDOW_EX_STYLE(0));
            win::set_text(h, &(i + 1).to_string());
            // Those already given their place in this round are in the accent; the rest, grey.
            unsafe {
                wm::SetWindowLongPtrW(h, wm::GWLP_USERDATA, (i < next) as isize);
            }
            let wide = if i + 1 >= 10 { 26 } else { 18 };
            win::move_to(h, scale(x, dpi), scale(y, dpi), scale(wide, dpi), scale(18, dpi));
            unsafe {
                let _ = wm::SetWindowPos(h, Some(wm::HWND_TOP), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOACTIVATE);
            }
            self.numbers.push(h);
        }
    }

    /// A click in tab order mode: the control takes the next place.
    fn tab_click(&mut self, i: usize) {
        let Some(s) = studio() else { return };
        let Some(next) = self.tab_next else { return };
        if i >= next {
            s.edit(|f| {
                let c = f.control.remove(i);
                f.control.insert(next, c);
            });
            self.tab_next = Some((next + 1).min(s.doc.form.control.len()));
        }
        self.rebuild();
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
        // While a group moves its windows are already where they are going: their rectangles are
        // read as they are, with nothing added.
        let moved = |r: [i32; 4]| r;
        let shown = match self.drag {
            Drag::Create { from, to, .. } | Drag::Band { from, to } => Some(norm(from, to)),
            Drag::Resize { now, .. } => Some(now),
            // A component has no place on the form: its chip in the tray shows it chosen.
            _ => s.selected.filter(|i| s.doc.form.control.get(*i).is_none_or(|c| !c.kind.is_component())).map(|i| moved(self.rect_of(i))),
        };
        // The tray drawn again, its chosen chip with it.
        let size = s.drag_size();
        let chips = tray_chips(&s.doc.form, size);
        if let (Some(first), Some(last)) = (chips.first(), chips.last()) {
            let r = RECT { left: scale(first.1[0], dpi) - 2, top: scale(first.1[1], dpi) - 2, right: scale(last.1[0] + last.1[2], dpi) + 2, bottom: scale(last.1[1] + last.1[3], dpi) + 2 };
            unsafe {
                let _ = Gdi::InvalidateRect(Some(self.hwnd), Some(&r), true);
            }
        }
        // The others chosen: a small mark at each corner, as Visual Studio shows them.
        for h in self.marks.iter() {
            unsafe {
                let _ = wm::DestroyWindow(*h);
            }
        }
        let class = register_mark();
        let mut marks = Vec::new();
        if !matches!(self.drag, Drag::Band { .. } | Drag::Create { .. }) {
            for i in s.also.iter().copied().filter(|i| *i < self.live.len()) {
                let [x, y, w, h] = moved(self.rect_of(i));
                for (cx, cy) in [(x, y), (x + w, y), (x, y + h), (x + w, y + h)] {
                    let m = create_child(&class, self.surface, WINDOW_EX_STYLE(0));
                    win::move_to(m, scale(cx, dpi) - hs / 2, scale(cy, dpi) - hs / 2, hs, hs);
                    unsafe {
                        let _ = wm::SetWindowPos(m, Some(wm::HWND_TOP), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOACTIVATE);
                    }
                    marks.push(m);
                }
            }
        }
        // Interior mutability through a raw pointer: the marks are this canvas's own windows.
        unsafe {
            let me = self as *const Canvas as *mut Canvas;
            (*me).marks = marks;
        }
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
        // The smart tag at the chosen control's top-right corner, when it has tasks.
        let tagged = matches!(self.drag, Drag::None) && s.also.is_empty() && s.selected.is_some_and(|i| s.doc.form.control.get(i).is_some_and(|c| !c.kind.is_component()));
        match (shown, tagged) {
            (Some(r), true) => {
                let (x, y) = self.tag_point(r);
                let ts = scale(TAG, dpi);
                win::move_to(self.tag, scale(x, dpi), scale(y, dpi), ts, ts);
                unsafe {
                    let _ = wm::SetWindowPos(self.tag, Some(wm::HWND_TOP), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOACTIVATE | wm::SWP_SHOWWINDOW);
                }
            }
            _ => win::show(self.tag, false),
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
            win::show(*h, s.selected.is_none() && s.doc.code.is_none());
        }
    }

    /// Where the smart tag goes for a control at `r`: just outside its top-right corner.
    fn tag_point(&self, r: [i32; 4]) -> (i32, i32) {
        (r[0] + r[2] + HANDLE / 2 + 2, r[1] - HANDLE / 2)
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
        // Windows puts the first child made on top, so the first in the form is found first.
        (0..self.live.len()).filter(|i| unsafe { wm::IsWindowVisible(self.live[*i]) }.as_bool()).find(|i| {
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
        if self.tab_next.is_some() {
            unsafe {
                let _ = km::ReleaseCapture();
            }
            if let Some(i) = self.hit(p) {
                self.tab_click(i);
            }
            return;
        }
        // The smart tag: the chosen control's tasks.
        if let Some(i) = s.selected.filter(|_| unsafe { wm::IsWindowVisible(self.tag) }.as_bool()) {
            let (tx, ty) = self.tag_point(self.rect_of(i));
            if p.0 >= tx && p.0 < tx + TAG && p.1 >= ty && p.1 < ty + TAG {
                unsafe {
                    let _ = km::ReleaseCapture();
                }
                s.smart_tag();
                return;
            }
        }
        if let Some(kind) = s.tool {
            let at = (snap(p.0), snap(p.1));
            self.drag = Drag::Create { kind, from: at, to: at };
        } else if let (Some(i), Some(handle), false) = (s.selected, self.hit_handle(p), self.locked || s.selected.is_some_and(|i| s.doc.form.control.get(i).is_some_and(|c| c.locked == Some(true)))) {
            let r = self.rect_of(i);
            self.drag = Drag::Resize { index: i, handle, from: p, rect: r, now: r };
        } else if let Some(i) = self.hit(p) {
            let ctrl = unsafe { km::GetKeyState(0x11) } < 0;
            if ctrl {
                // Ctrl+click adds a control to the choice, or takes it out.
                s.toggle(i);
                unsafe {
                    let _ = km::ReleaseCapture();
                }
                self.place_handles();
                return;
            }
            if !s.selection().contains(&i) {
                s.select(Some(i));
            } else if s.selected != Some(i) {
                // A click on one of several chosen makes it the one the others follow.
                s.make_primary(i);
            }
            self.group = s
                .selection()
                .into_iter()
                .filter(|j| s.doc.form.control[*j].dock.as_deref().is_none_or(|d| d == "none") && s.doc.form.control[*j].locked != Some(true))
                .map(|j| (j, self.rect_of(j)))
                .collect();
            if !self.group.is_empty() && !self.locked {
                self.drag = Drag::Move { from: p, d: (0, 0) };
            }
        } else {
            // Empty form: draw a band to choose what it touches.
            s.select(None);
            self.drag = Drag::Band { from: p, to: p };
        }
        self.place_handles();
    }

    fn moved(&mut self, l: LPARAM) {
        let Some(s) = studio() else { return };
        let p = self.design_point(self.surface, l);
        let dpi = win::dpi(self.surface);
        // What a moving group may snap to: every other control shown, worked out before the drag
        // is borrowed.
        let others: Vec<[i32; 4]> = if matches!(self.drag, Drag::Move { .. }) {
            (0..self.live.len()).filter(|i| !self.group.iter().any(|(g, _)| g == i) && unsafe { wm::IsWindowVisible(self.live[*i]) }.as_bool()).map(|i| self.rect_of(i)).collect()
        } else {
            vec![]
        };
        match &mut self.drag {
            Drag::Create { to, .. } => *to = (snap(p.0), snap(p.1)),
            Drag::Move { from, d } => {
                // The first one snaps to other controls' edges and the form's margin where it
                // comes near them, with a line to show it, else to the grid; the rest keep their
                // places beside it.
                let first = self.group[0].1;
                let raw = (p.0 - from.0, p.1 - from.1);
                let moved_rect = [first[0] + raw.0, first[1] + raw.1, first[2], first[3]];
                let ((sx, sy), lines) = phai_form::arrange::snaplines(moved_rect, &others, s.doc.form.form.size, phai_design::metrics::MARGIN);
                let snapped_x = lines.iter().any(|(v, _)| *v);
                let snapped_y = lines.iter().any(|(v, _)| !*v);
                *d = (
                    if snapped_x { raw.0 + sx } else { snap(first[0] + raw.0) - first[0] },
                    if snapped_y { raw.1 + sy } else { snap(first[1] + raw.1) - first[1] },
                );
                let d = *d;
                show_lines(&mut self.lines, self.surface, &lines, s.doc.form.form.size);
                for (i, r) in &self.group {
                    win::move_to(self.live[*i], scale(r[0] + d.0, dpi), scale(r[1] + d.1, dpi), scale(r[2], dpi), scale(r[3], dpi));
                }
            }
            Drag::Band { to, .. } => *to = p,
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
        show_lines(&mut self.lines, self.surface, &[], [0, 0]);
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
            Drag::Move { d, .. } => {
                if d != (0, 0) {
                    let group = std::mem::take(&mut self.group);
                    s.edit(|f| {
                        for (i, r) in &group {
                            f.control[*i].bounds = Some([r[0] + d.0, r[1] + d.1, r[2], r[3]]);
                        }
                    });
                    self.rebuild();
                    s.refresh_props();
                }
            }
            Drag::Band { from, to } => {
                let [bx, by, bw, bh] = norm(from, to);
                let touched: Vec<usize> = (0..self.live.len())
                    .filter(|i| unsafe { wm::IsWindowVisible(self.live[*i]) }.as_bool())
                    .filter(|i| {
                        let [x, y, w, h] = self.rect_of(*i);
                        x < bx + bw && bx < x + w && y < by + bh && by < y + h
                    })
                    .collect();
                if bw > 2 || bh > 2 {
                    s.select_many(&touched);
                }
            }
            Drag::Resize { index, rect, now, .. } => {
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
            Drag::Resize { now, .. } => format!("{}, {}  ·  {} × {}", now[0], now[1], now[2], now[3]),
            Drag::Move { d, .. } => {
                let r = self.group[0].1;
                format!("{}, {}", r[0] + d.0, r[1] + d.1)
            }
            Drag::Band { from, to } => {
                let r = norm(from, to);
                format!("{} × {}", r[2], r[3])
            }
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
        // A chip in the component tray: that component chosen.
        if let Some((i, _)) = tray_chips(&s.doc.form, s.drag_size()).into_iter().find(|(_, r)| p.0 >= r[0] && p.0 < r[0] + r[2] && p.1 >= r[1] && p.1 < r[1] + r[3]) {
            s.select(Some(i));
            self.place_handles();
            return;
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
        if vk == 0x1B && self.tab_next.is_some() {
            self.set_tab_mode(false);
            return true;
        }
        if s.selected.is_none() {
            return false;
        }
        let chosen = s.selection();
        let shift = unsafe { km::GetKeyState(km::VK_SHIFT.0 as i32) } < 0;
        let ctrl = unsafe { km::GetKeyState(km::VK_CONTROL.0 as i32) } < 0;
        // Locked - all, or one of those chosen: the arrows move nothing.
        if (self.locked || chosen.iter().any(|i| s.doc.form.control.get(*i).is_some_and(|c| c.locked == Some(true)))) && vk != 0x2E {
            return matches!(vk, 0x25..=0x28);
        }
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
        // Ctrl: on to the next line another control's edge or the margin makes, as snaplines do.
        let (dx, dy) = if ctrl {
            let first = s.selected.map(|i| self.rect_of(i)).unwrap_or_default();
            let others: Vec<[i32; 4]> = (0..self.live.len()).filter(|i| !chosen.contains(i) && unsafe { wm::IsWindowVisible(self.live[*i]) }.as_bool()).map(|i| self.rect_of(i)).collect();
            let form = s.doc.form.form.size;
            let m = phai_design::metrics::MARGIN;
            if dx != 0 {
                (phai_form::arrange::next_stop(first, &others, form, m, true, dx, shift), 0)
            } else {
                (0, phai_form::arrange::next_stop(first, &others, form, m, false, dy, shift))
            }
        } else {
            (dx, dy)
        };
        s.edit(|f| {
            for i in &chosen {
                let c = &mut f.control[*i];
                let [x, y, w, h] = c.rect();
                c.bounds = Some(if shift { [x, y, (w + dx).max(1), (h + dy).max(1)] } else { [x + dx, y + dy, w, h] });
            }
        });
        self.rebuild();
        s.refresh_props();
        s.say_selection();
        true
    }
}

/// The snaplines across the form, in the accent colour: shown, moved, or all hidden.
fn show_lines(windows: &mut Vec<HWND>, surface: HWND, lines: &[(bool, i32)], form: [i32; 2]) {
    let class = win::register("PhaiStudioSnap", Some(snap_proc), None);
    while windows.len() < lines.len() {
        windows.push(create_child(&class, surface, WINDOW_EX_STYLE(0)));
    }
    let dpi = win::dpi(surface);
    for (k, h) in windows.iter().enumerate() {
        match lines.get(k) {
            Some(&(vertical, at)) => {
                if vertical {
                    win::move_to(*h, scale(at, dpi), 0, 1.max(scale(1, dpi) / 2), scale(form[1], dpi));
                } else {
                    win::move_to(*h, 0, scale(at, dpi), scale(form[0], dpi), 1.max(scale(1, dpi) / 2));
                }
                unsafe {
                    let _ = wm::SetWindowPos(*h, Some(wm::HWND_TOP), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOACTIVATE | wm::SWP_SHOWWINDOW);
                }
            }
            None => win::show(*h, false),
        }
    }
}

extern "system" fn snap_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_NCHITTEST => LRESULT(-1),
        wm::WM_ERASEBKGND => {
            let dc = Gdi::HDC(w.0 as *mut _);
            let mut r = RECT::default();
            unsafe {
                let _ = wm::GetClientRect(h, &mut r);
            }
            win::fill(dc, &r, theme::current().var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f)));
            LRESULT(1)
        }
        _ => win::def(h, msg, w, l),
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
    // The component tray: a chip for each, its toolbox icon and name; the chosen one marked.
    let chips = tray_chips(f, size);
    if !chips.is_empty() {
        let chosen = studio().and_then(|s| s.selected);
        let accent = theme::current().var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f));
        for (i, r) in &chips {
            let c = &f.control[*i];
            let rr = RECT { left: p(r[0]), top: p(r[1]), right: p(r[0] + r[2]), bottom: p(r[1] + r[3]) };
            win::fill(dc, &rr, if chosen == Some(*i) { accent } else { border });
            win::fill(dc, &RECT { left: rr.left + 1, top: rr.top + 1, right: rr.right - 1, bottom: rr.bottom - 1 }, window);
            unsafe {
                Gdi::SelectObject(dc, icons.into());
                Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(accent.colorref()));
            }
            let g = crate::toolbox_items().iter().find(|(id, _, _)| crate::tool_kind(*id) == Some(c.kind)).map(|(_, _, g)| *g).unwrap_or(0);
            let mut gr = RECT { left: rr.left + p(6), top: rr.top, right: rr.left + p(26), bottom: rr.bottom };
            let mut one = [g];
            unsafe {
                Gdi::DrawTextW(dc, &mut one, &mut gr, Gdi::DT_CENTER | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE);
                Gdi::SelectObject(dc, font.into());
                Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(text.colorref()));
            }
            let mut tr = RECT { left: rr.left + p(30), top: rr.top, right: rr.right - p(4), bottom: rr.bottom };
            let mut words: Vec<u16> = c.id.encode_utf16().collect();
            unsafe {
                Gdi::DrawTextW(dc, &mut words, &mut tr, Gdi::DT_LEFT | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX);
            }
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
            if let Some(s) = studio().filter(|s| s.doc.code.is_none()) {
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
        // An editor on the canvas says its text changed; an editor takes the canvas's whole size.
        wm::WM_COMMAND if win::hiword(w.0) == phai_editor::view::CHANGED => {
            if let Some(s) = studio() {
                s.code_changed_in(HWND(l.0 as *mut _));
            }
            LRESULT(0)
        }
        wm::WM_COMMAND if win::hiword(w.0) == phai_editor::view::HOVER => {
            if let Some(s) = studio() {
                s.lsp_hover(HWND(l.0 as *mut _));
            }
            LRESULT(0)
        }
        wm::WM_COMMAND if win::hiword(w.0) == phai_editor::view::COMPLETE => {
            if let Some(s) = studio() {
                s.lsp_complete(HWND(l.0 as *mut _));
            }
            LRESULT(0)
        }
        wm::WM_COMMAND if win::hiword(w.0) == phai_editor::view::DEFINITION => {
            if let Some(s) = studio() {
                s.lsp_definition(HWND(l.0 as *mut _));
            }
            LRESULT(0)
        }
        wm::WM_COMMAND if win::hiword(w.0) == phai_editor::view::ACTIONS => {
            if let Some(s) = studio() {
                s.lsp_actions(HWND(l.0 as *mut _));
            }
            LRESULT(0)
        }
        wm::WM_COMMAND if win::hiword(w.0) == phai_editor::view::SIGNATURE => {
            if let Some(s) = studio() {
                s.lsp_signature(HWND(l.0 as *mut _));
            }
            LRESULT(0)
        }
        wm::WM_COMMAND if win::hiword(w.0) == phai_editor::view::PEEK => {
            if let Some(s) = studio() {
                s.lsp_peek(HWND(l.0 as *mut _));
            }
            LRESULT(0)
        }
        wm::WM_COMMAND if win::hiword(w.0) == phai_editor::view::BREAKPOINTS => {
            if let Some(s) = studio() {
                s.breakpoints_changed(HWND(l.0 as *mut _));
            }
            LRESULT(0)
        }
        wm::WM_COMMAND if win::hiword(w.0) == phai_editor::view::REFERENCES => {
            if let Some(s) = studio() {
                s.lsp_references(HWND(l.0 as *mut _));
            }
            LRESULT(0)
        }
        wm::WM_COMMAND if win::hiword(w.0) == phai_editor::view::RENAME => {
            if let Some(s) = studio() {
                s.lsp_rename(HWND(l.0 as *mut _));
            }
            LRESULT(0)
        }
        // The split's divider dragged: the upper pane takes or gives the room.
        phai_ui::controls::WM_SPLIT => {
            if let Some(s) = studio() {
                s.split_dragged(l.0 as i32);
            }
            LRESULT(0)
        }
        wm::WM_SIZE => {
            if let Some(s) = studio() {
                s.place_editors(win::loword(l.0 as usize) as i32, win::hiword(l.0 as usize) as i32);
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

/// A control's place in tab order mode: a small tag at its corner, white on the accent once given
/// its place in this round, on grey before.
extern "system" fn number_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_NCHITTEST => LRESULT(-1),
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            let done = wm::GetWindowLongPtrW(h, wm::GWLP_USERDATA) != 0;
            let bg = if done { theme::current().var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f)) } else { Color::rgb(0x5f, 0x5f, 0x5f) };
            win::fill(dc, &r, bg);
            let text = win::text_of(h);
            let font = win::font(h, 8, true);
            let old = Gdi::SelectObject(dc, font.into());
            let mut w16: Vec<u16> = text.encode_utf16().collect();
            Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
            Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(0x00FF_FFFF));
            Gdi::DrawTextW(dc, &mut w16, &mut r, Gdi::DT_CENTER | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE);
            Gdi::SelectObject(dc, old);
            let _ = Gdi::DeleteObject(font.into());
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        _ => win::def(h, msg, w, l),
    }
}

/// The smart tag: a small box with ▸, as Visual Studio's; a click on it goes to the surface below.
extern "system" fn tag_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
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
            // ▸, drawn: a small triangle pointing right.
            let (cx, cy, k) = (r.right / 2, r.bottom / 2, (r.bottom / 5).max(2));
            let pts = [windows::Win32::Foundation::POINT { x: cx - k / 2, y: cy - k }, windows::Win32::Foundation::POINT { x: cx - k / 2, y: cy + k }, windows::Win32::Foundation::POINT { x: cx + k, y: cy }];
            let brush = Gdi::CreateSolidBrush(windows::Win32::Foundation::COLORREF(accent.colorref()));
            let pen = Gdi::CreatePen(Gdi::PS_SOLID, 1, windows::Win32::Foundation::COLORREF(accent.colorref()));
            let (ob, op) = (Gdi::SelectObject(dc, brush.into()), Gdi::SelectObject(dc, pen.into()));
            let _ = Gdi::Polygon(dc, &pts);
            Gdi::SelectObject(dc, ob);
            Gdi::SelectObject(dc, op);
            let _ = Gdi::DeleteObject(brush.into());
            let _ = Gdi::DeleteObject(pen.into());
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        _ => win::def(h, msg, w, l),
    }
}

/// A corner mark of a control chosen with others: hollow, in the accent colour.
extern "system" fn mark_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
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
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
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
