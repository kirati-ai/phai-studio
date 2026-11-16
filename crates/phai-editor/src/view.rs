//! The editor as a Win32 control: a [`Model`] drawn line by line in a fixed-width font, with line
//! numbers, colours from the theme, a caret, selection by mouse and keyboard, scroll bars and the
//! clipboard. Windows draws the text, so Thai shapes as it does everywhere else.
//!
//! The parent hears `WM_COMMAND` with the control's id and [`CHANGED`] in the high word whenever
//! the text changes.

use phai_ui::theme::{self, Color, State};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HDC, HFONT};
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HMENU};

use crate::model::{next_stop, Model, Move};
use crate::syntax::{line_tokens, Language, Token};

/// `WM_COMMAND`'s high word when the text changed (as `EN_CHANGE`).
pub const CHANGED: u16 = 0x0300;
/// The pointer rested on the text: the parent may ask the language server about [`hover_position`].
pub const HOVER: u16 = 0x0301;
/// Completion was asked for (Ctrl+Space, or `.` / `::` typed): see [`caret_lsp`] and [`show_completions`].
pub const COMPLETE: u16 = 0x0302;

/// A diagnostic placed in the text: byte offsets, how bad, what it says.
#[derive(Clone, Debug)]
struct Mark {
    from: usize,
    to: usize,
    error: bool,
    message: String,
}

/// Points of every editor's text; Ctrl+wheel, Ctrl+= and Ctrl+- change it for all of them.
static FONT_PT: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(10);

/// The editors' text size, points.
pub fn font_size() -> i32 {
    FONT_PT.load(std::sync::atomic::Ordering::Relaxed)
}

/// Sets every editor's text size, 6 to 40 points; open editors follow when they next paint.
pub fn set_font_size(pt: i32) {
    FONT_PT.store(pt.clamp(6, 40), std::sync::atomic::Ordering::Relaxed);
}

struct View {
    /// The size `font` was made at - behind [`font_size`] after a zoom elsewhere.
    pt: i32,
    model: Model,
    lang: Language,
    font: HFONT,
    line_h: i32,
    char_w: i32,
    /// The first line shown.
    top: usize,
    /// How far the text is scrolled right, device pixels.
    left: i32,
    /// Block-comment state at the start of each line, worked out again after an edit.
    comments: Vec<bool>,
    comments_for: usize,
    edits: usize,
    dragging: bool,
    high_surrogate: Option<u16>,
    /// A key already acted on whose character is still to come.
    swallow_char: bool,
    marks: Vec<Mark>,
    /// Where the pointer rested, for a hover: byte offset and screen point.
    hover: Option<(usize, i32, i32)>,
    tracking: bool,
    tip: Option<Box<crate::popup::Popup>>,
    list: Option<Box<crate::popup::Popup>>,
    /// Every completion the server gave; the list shows those that start with what is typed.
    all_items: Vec<crate::lsp::Completion>,
    /// Edits since the file was opened, as the server counts versions.
    pub version: i64,
    find: Option<FindBar>,
    /// What is being found, and every place it is - worked out again after each edit.
    query: String,
    hits: Vec<(usize, usize)>,
    hits_for: usize,
}

/// The bar at the editor's top right: a box to find, and - with Ctrl+H - a box to replace.
struct FindBar {
    panel: HWND,
    query: HWND,
    with: HWND,
    replacing: bool,
    count: HWND,
}

const ID_FIND: i32 = 1;
const ID_WITH: i32 = 2;
const ID_PREV: i32 = 3;
const ID_NEXT: i32 = 4;
const ID_CLOSE: i32 = 5;
const ID_REPLACE: i32 = 6;
const ID_ALL: i32 = 7;

fn wide_of(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

impl View {
    fn width_of(&self, dc: HDC, s: &str) -> i32 {
        if s.is_empty() {
            return 0;
        }
        // Tabs as four spaces.
        let w = wide_of(&s.replace('\t', "    "));
        let mut size = SIZE::default();
        unsafe {
            let _ = Gdi::GetTextExtentPoint32W(dc, &w, &mut size);
        }
        size.cx
    }

    fn gutter(&self) -> i32 {
        let digits = self.model.buf.line_count().to_string().len().max(3) as i32;
        digits * self.char_w + self.char_w * 2
    }

    fn visible_lines(&self, h: HWND) -> usize {
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(h, &mut r);
        }
        ((r.bottom - r.top) / self.line_h.max(1)).max(1) as usize
    }

    fn refresh_comments(&mut self) {
        if self.comments_for == self.edits && self.comments.len() == self.model.buf.line_count() {
            return;
        }
        let n = self.model.buf.line_count();
        self.comments = Vec::with_capacity(n);
        let mut open = false;
        for i in 0..n {
            self.comments.push(open);
            open = line_tokens(self.lang, &self.model.buf.line(i), open).1;
        }
        self.comments_for = self.edits;
    }

    /// The caret's place on screen.
    fn caret_point(&self, dc: HDC) -> (i32, i32) {
        let (line, col) = self.model.buf.position(self.model.caret);
        let s = self.model.buf.line(line);
        let x = self.gutter() + self.width_of(dc, &s[..col]) - self.left;
        let y = (line as i32 - self.top as i32) * self.line_h;
        (x, y)
    }

    /// The byte offset under a point in the client area.
    fn offset_at(&self, dc: HDC, x: i32, y: i32) -> usize {
        let line = (self.top as i32 + (y / self.line_h.max(1)).max(0)) as usize;
        let line = line.min(self.model.buf.line_count() - 1);
        let s = self.model.buf.line(line);
        let target = x - self.gutter() + self.left;
        let mut at = 0;
        while at < s.len() {
            let next = next_stop(&s, at);
            let mid = (self.width_of(dc, &s[..at]) + self.width_of(dc, &s[..next])) / 2;
            if target < mid {
                break;
            }
            at = next;
        }
        self.model.buf.line_start(line) + at
    }
}

fn view<'a>(h: HWND) -> Option<&'a mut View> {
    unsafe { win::state::<View>(h) }
}

/// The colours of the code, from the theme's `--code-*` variables, else the defaults below.
fn colour(t: Token, dark: bool) -> Color {
    let th = theme::current();
    let (name, light, darkc) = match t {
        Token::Keyword => ("--code-keyword", Color::rgb(0x00, 0x4f, 0xa8), Color::rgb(0x56, 0x9c, 0xd6)),
        Token::Type => ("--code-type", Color::rgb(0x12, 0x85, 0x5f), Color::rgb(0x4e, 0xc9, 0xb0)),
        Token::String => ("--code-string", Color::rgb(0xa3, 0x15, 0x15), Color::rgb(0xce, 0x91, 0x78)),
        Token::Number => ("--code-number", Color::rgb(0x09, 0x86, 0x58), Color::rgb(0xb5, 0xce, 0xa8)),
        Token::Comment => ("--code-comment", Color::rgb(0x6a, 0x73, 0x7d), Color::rgb(0x6a, 0x99, 0x55)),
        Token::Macro => ("--code-macro", Color::rgb(0x79, 0x5e, 0x26), Color::rgb(0xdc, 0xdc, 0xaa)),
        Token::Attribute => ("--code-attribute", Color::rgb(0x80, 0x80, 0x80), Color::rgb(0x9c, 0x9c, 0x9c)),
        Token::Plain => return win::window_text(),
    };
    th.var_color(name).unwrap_or(if dark { darkc } else { light })
}

/// An editor showing `text`, coloured as `path`'s kind of file.
pub fn create(parent: HWND, id: i32, text: &str, path: &str) -> HWND {
    let class = win::register_with(win::ClassStyle::DoubleClicks, "PhaiEditor", Some(proc_), None);
    unsafe {
        let h = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_VSCROLL | wm::WS_HSCROLL | wm::WS_TABSTOP | wm::WS_CLIPSIBLINGS, 0, 0, 10, 10, Some(parent), Some(HMENU(id as isize as *mut _)), Some(win::instance()), None).unwrap_or_default();
        let font = code_font(h);
        let (line_h, char_w) = metrics(h, font);
        win::set_state(h, Box::new(View { pt: font_size(), model: Model::new(text), lang: Language::of(path), font, line_h, char_w, top: 0, left: 0, comments: vec![], comments_for: usize::MAX, edits: 0, dragging: false, high_surrogate: None, swallow_char: false, marks: vec![], hover: None, tracking: false, tip: None, list: None, all_items: vec![], version: 1, find: None, query: String::new(), hits: vec![], hits_for: usize::MAX }));
        // The studio's shortcuts leave typing keys to a box: this is one.
        win::register_kind(h, win::Kind::Edit);
        let _ = windows::Win32::UI::Controls::SetWindowTheme(h, if theme::current().is_dark() { windows::core::w!("DarkMode_Explorer") } else { windows::core::w!("Explorer") }, PCWSTR::null());
        update_scroll(h);
        h
    }
}

fn code_font(h: HWND) -> HFONT {
    let mut lf = Gdi::LOGFONTW { lfHeight: -(font_size() * win::dpi(h) / 72), lfCharSet: Gdi::DEFAULT_CHARSET, lfQuality: Gdi::CLEARTYPE_QUALITY, ..Default::default() };
    for (i, u) in "Cascadia Mono".encode_utf16().enumerate() {
        lf.lfFaceName[i] = u;
    }
    unsafe { Gdi::CreateFontIndirectW(&lf) }
}

fn metrics(h: HWND, font: HFONT) -> (i32, i32) {
    unsafe {
        let dc = Gdi::GetDC(Some(h));
        let old = Gdi::SelectObject(dc, font.into());
        let mut tm = Gdi::TEXTMETRICW::default();
        let _ = Gdi::GetTextMetricsW(dc, &mut tm);
        Gdi::SelectObject(dc, old);
        Gdi::ReleaseDC(Some(h), dc);
        // A little more than the font's height: Thai marks above and below need the room.
        (tm.tmHeight + tm.tmExternalLeading + win::px(h, 4), tm.tmAveCharWidth.max(1))
    }
}

/// The language server's diagnostics for this file, shown as wavy lines under their text.
pub fn set_diagnostics(h: HWND, diags: &[crate::lsp::Diagnostic]) {
    let Some(v) = view(h) else { return };
    let offset = |(line, ch): (u32, u32)| {
        let l = (line as usize).min(v.model.buf.line_count() - 1);
        let s = v.model.buf.line(l);
        v.model.buf.line_start(l) + crate::lsp::from_lsp(&s, ch)
    };
    v.marks = diags
        .iter()
        .map(|d| {
            let (from, mut to) = (offset(d.start), offset(d.end));
            if to <= from {
                to = (from + 1).min(v.model.buf.len());
            }
            Mark { from, to, error: d.error, message: d.message.clone() }
        })
        .collect();
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

/// How many errors and warnings are shown.
pub fn diagnostic_counts(h: HWND) -> (usize, usize) {
    view(h).map(|v| (v.marks.iter().filter(|m| m.error).count(), v.marks.iter().filter(|m| !m.error).count())).unwrap_or((0, 0))
}

fn lsp_position_of(v: &View, offset: usize) -> (u32, u32) {
    let (line, col) = v.model.buf.position(offset);
    (line as u32, crate::lsp::to_lsp(&v.model.buf.line(line), col))
}

/// Where the pointer rests, as the language server counts.
pub fn hover_position(h: HWND) -> Option<(u32, u32)> {
    let v = view(h)?;
    v.hover.map(|(at, _, _)| lsp_position_of(v, at))
}

/// The caret, as the language server counts.
pub fn caret_lsp(h: HWND) -> (u32, u32) {
    view(h).map(|v| lsp_position_of(v, v.model.caret)).unwrap_or((0, 0))
}

/// Shows a hover's words where the pointer rests.
pub fn show_tip(h: HWND, text: &str) {
    let Some(v) = view(h) else { return };
    let Some((_, x, y)) = v.hover else { return };
    v.tip = Some(crate::popup::Popup::tip(h, v.font, text, x, y + v.line_h));
}

fn hide_tip(v: &mut View) {
    v.tip = None;
}

/// The identifier being typed before the caret - what a completion replaces.
fn word_before(v: &View) -> (usize, String) {
    let (line, col) = v.model.buf.position(v.model.caret);
    let s = v.model.buf.line(line);
    let start = s[..col].char_indices().rev().take_while(|(_, c)| c.is_alphanumeric() || *c == '_').last().map(|(i, _)| i).unwrap_or(col);
    (v.model.buf.line_start(line) + start, s[start..col].to_string())
}

/// The language server's completions, as a list under the caret, filtered by what is typed.
pub fn show_completions(h: HWND, items: Vec<crate::lsp::Completion>) {
    let Some(v) = view(h) else { return };
    v.all_items = items;
    filter_list(h);
}

fn filter_list(h: HWND) {
    let Some(v) = view(h) else { return };
    let (_, typed) = word_before(v);
    let lower = typed.to_lowercase();
    let shown: Vec<crate::lsp::Completion> = v.all_items.iter().filter(|c| c.label.to_lowercase().starts_with(&lower)).take(50).cloned().collect();
    if shown.is_empty() {
        v.list = None;
        return;
    }
    let mut pt = windows::Win32::Foundation::POINT::default();
    unsafe {
        let dc = Gdi::GetDC(Some(h));
        let old = Gdi::SelectObject(dc, v.font.into());
        let (x, y) = v.caret_point(dc);
        Gdi::SelectObject(dc, old);
        Gdi::ReleaseDC(Some(h), dc);
        pt.x = x;
        pt.y = y + v.line_h;
        let _ = Gdi::ClientToScreen(h, &mut pt);
    }
    match v.list.as_mut() {
        Some(list) => {
            list.items = shown;
            list.chosen = 0;
            list.place(pt.x, pt.y);
        }
        None => v.list = Some(crate::popup::Popup::list(h, v.font, shown, pt.x, pt.y)),
    }
}

/// Puts the chosen completion in place of the word being typed.
fn accept(h: HWND, v: &mut View) {
    let Some(c) = v.list.as_ref().and_then(|l| l.choice()).cloned() else { return };
    let (start, _) = word_before(v);
    v.model.anchor = start;
    v.model.insert(&c.insert);
    v.list = None;
    v.all_items.clear();
    changed(h);
}

fn notify(h: HWND, code: u16) {
    unsafe {
        let id = wm::GetDlgCtrlID(h) as usize;
        if let Ok(p) = wm::GetParent(h) {
            wm::SendMessageW(p, wm::WM_COMMAND, Some(WPARAM(((code as usize) << 16) | (id & 0xFFFF))), Some(LPARAM(h.0 as isize)));
        }
    }
}

fn refresh_hits(v: &mut View) {
    if v.hits_for == v.edits {
        return;
    }
    v.hits = if v.find.is_some() { crate::model::find_all(&v.model.buf.text(), &v.query, false) } else { vec![] };
    v.hits_for = v.edits;
}

/// "3 of 12" beside the box.
fn say_count(v: &mut View) {
    refresh_hits(v);
    let Some(f) = v.find.as_ref() else { return };
    let (a, _) = v.model.selection();
    let at = v.hits.iter().position(|hh| hh.0 == a).map(|i| i + 1).unwrap_or(0);
    win::set_text(f.count, &if v.query.is_empty() { String::new() } else { format!("{at}/{}", v.hits.len()) });
}

fn find_step(h: HWND, v: &mut View, forward: bool) {
    if v.query.is_empty() {
        return;
    }
    let q = v.query.clone();
    if v.model.find(&q, false, forward) {
        moved(h);
    }
    say_count(v);
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

/// The box to find in keeps Enter, Escape and F3 for itself.
unsafe extern "system" fn find_box(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, _: usize) -> LRESULT {
    use windows::Win32::UI::Shell::DefSubclassProc;
    let editor = unsafe { wm::GetParent(wm::GetParent(h).unwrap_or_default()).unwrap_or_default() };
    match msg {
        wm::WM_GETDLGCODE => LRESULT(0x0004), // DLGC_WANTALLKEYS
        wm::WM_KEYDOWN if matches!(w.0, 0x0D | 0x1B | 0x72) => {
            if let Some(v) = view(editor) {
                let shift = unsafe { km::GetKeyState(0x10) } < 0;
                let is_with = unsafe { wm::GetDlgCtrlID(h) } == ID_WITH;
                match w.0 {
                    0x1B => {
                        close_find(editor, v);
                        unsafe {
                            let _ = km::SetFocus(Some(editor));
                        }
                    }
                    0x0D if is_with => {
                        let text = win::text_of(h);
                        let q = v.query.clone();
                        v.model.replace_one(&q, &text, false);
                        changed(editor);
                        say_count(v);
                    }
                    _ => find_step(editor, v, !shift),
                }
            }
            LRESULT(0)
        }
        wm::WM_CHAR if matches!(w.0, 0x0D | 0x1B) => LRESULT(0),
        _ => unsafe { DefSubclassProc(h, msg, w, l) },
    }
}

extern "system" fn find_panel_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        // Its boxes and buttons speak to the editor.
        wm::WM_COMMAND => unsafe {
            match wm::GetParent(h) {
                Ok(p) => wm::SendMessageW(p, msg, Some(w), Some(l)),
                Err(_) => LRESULT(0),
            }
        },
        wm::WM_ERASEBKGND => {
            let dc = Gdi::HDC(w.0 as *mut _);
            let mut r = RECT::default();
            unsafe {
                let _ = wm::GetClientRect(h, &mut r);
            }
            let bg = theme::style("tabs", State::default()).background_or(win::window_background());
            win::fill(dc, &r, bg);
            let border = theme::style("group", State::default()).border_or(Color::rgb(0xd4, 0xd4, 0xd4));
            win::fill(dc, &RECT { top: r.bottom - 1, ..r }, border);
            win::fill(dc, &RECT { right: 1, ..r }, border);
            LRESULT(1)
        }
        _ => win::def(h, msg, w, l),
    }
}

fn open_find(h: HWND, v: &mut View, replacing: bool) {
    // A word chosen on one line is what to find.
    let sel = v.model.selected_text();
    if !sel.is_empty() && !sel.contains('\n') {
        v.query = sel;
    }
    if v.find.as_ref().is_some_and(|f| f.replacing != replacing) {
        close_find(h, v);
    }
    if v.find.is_none() {
        let class = win::register("PhaiFindBar", Some(find_panel_proc), None);
        let font = unsafe { HFONT(wm::SendMessageW(wm::GetParent(h).unwrap_or_default(), wm::WM_GETFONT, None, None).0 as *mut _) };
        let font = if font.is_invalid() { win::font(h, 0, false) } else { font };
        unsafe {
            let panel = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPCHILDREN, 0, 0, 10, 10, Some(h), Some(HMENU::default()), Some(win::instance()), None).unwrap_or_default();
            let query = win::edit(panel, &v.query, ID_FIND, font, false);
            let with = if replacing { win::edit(panel, "", ID_WITH, font, false) } else { HWND::default() };
            for e in [query, with] {
                if !e.is_invalid() {
                    let _ = windows::Win32::UI::Shell::SetWindowSubclass(e, Some(find_box), 9, 0);
                }
            }
            win::button(panel, "↑", ID_PREV, font, false);
            win::button(panel, "↓", ID_NEXT, font, false);
            win::button(panel, "✕", ID_CLOSE, font, false);
            if replacing {
                win::button(panel, phai_ui::text::t("Replace"), ID_REPLACE, font, false);
                win::button(panel, phai_ui::text::t("All"), ID_ALL, font, false);
            }
            let count = win::label(panel, "", 0, font);
            win::set_css(count, "muted", "");
            v.find = Some(FindBar { panel, query, with, replacing, count });
        }
        place_find(h, v);
    }
    if let Some(f) = v.find.as_ref() {
        if win::text_of(f.query) != v.query {
            win::set_text(f.query, &v.query);
        }
        unsafe {
            let _ = km::SetFocus(Some(f.query));
            wm::SendMessageW(f.query, 0x00B1 /* EM_SETSEL */, Some(WPARAM(0)), Some(LPARAM(-1)));
        }
    }
    v.hits_for = usize::MAX;
    say_count(v);
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

fn close_find(h: HWND, v: &mut View) {
    if let Some(f) = v.find.take() {
        unsafe {
            let _ = wm::DestroyWindow(f.panel);
        }
    }
    v.hits.clear();
    v.hits_for = usize::MAX;
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

/// The bar at the top right: the find box with its count and buttons, the replace row below.
fn place_find(h: HWND, v: &View) {
    let Some(f) = v.find.as_ref() else { return };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let px = |n| win::px(h, n);
    let (m, row, b) = (px(6), px(23), px(28));
    let width = px(380).min(r.right - px(20));
    let rows = if f.replacing { 2 } else { 1 };
    let height = m + rows * (row + m);
    win::move_to(f.panel, r.right - width - px(16), 0, width, height);
    let box_w = width - 2 * m - 3 * (b + px(2)) - px(52);
    win::move_to(f.query, m, m, box_w, row);
    win::move_to(f.count, m + box_w + px(4), m, px(48), row);
    let mut x = m + box_w + px(52);
    for id in [ID_PREV, ID_NEXT, ID_CLOSE] {
        unsafe {
            if let Ok(c) = wm::GetDlgItem(Some(f.panel), id) {
                win::move_to(c, x, m, b, row);
            }
        }
        x += b + px(2);
    }
    if f.replacing {
        let y = m + row + m;
        win::move_to(f.with, m, y, box_w, row);
        let bw = (width - m - box_w - px(4) - m - px(4)) / 2;
        unsafe {
            if let Ok(c) = wm::GetDlgItem(Some(f.panel), ID_REPLACE) {
                win::move_to(c, m + box_w + px(4), y, bw, row);
            }
            if let Ok(c) = wm::GetDlgItem(Some(f.panel), ID_ALL) {
                win::move_to(c, m + box_w + px(8) + bw, y, bw, row);
            }
        }
    }
}

/// Opens the find bar with `query` in it and the first place chosen - for checking by picture.
pub fn find_text(h: HWND, query: &str, replacing: bool) {
    if let Some(v) = view(h) {
        v.query = query.to_string();
        open_find(h, v, replacing);
        find_step(h, v, true);
    }
}

/// Opens the find bar (`replacing` with the replace row), as Ctrl+F and Ctrl+H do.
pub fn find(h: HWND, replacing: bool) {
    if let Some(v) = view(h) {
        open_find(h, v, replacing);
    }
}

/// The text now, with `\n` line ends.
pub fn text(h: HWND) -> String {
    view(h).map(|v| v.model.buf.text()).unwrap_or_default()
}

/// The text's version, as the language server counts edits.
pub fn version(h: HWND) -> i64 {
    view(h).map(|v| v.version).unwrap_or(1)
}

pub fn is_dirty(h: HWND) -> bool {
    view(h).is_some_and(|v| v.model.dirty)
}

pub fn set_clean(h: HWND) {
    if let Some(v) = view(h) {
        v.model.dirty = false;
    }
}

/// Undo (`back`) or redo, as Ctrl+Z / Ctrl+Y do.
pub fn undo_redo(h: HWND, back: bool) {
    if let Some(v) = view(h) {
        if back {
            v.model.undo();
        } else {
            v.model.redo();
        }
        changed(h);
    }
}

/// The caret's line and column, from 1, for a status bar.
pub fn caret_position(h: HWND) -> (usize, usize) {
    view(h).map(|v| {
        let (l, c) = v.model.buf.position(v.model.caret);
        let chars = v.model.buf.line(l)[..c].chars().count();
        (l + 1, chars + 1)
    }).unwrap_or((1, 1))
}

/// Puts the caret at a line and column from 1 - a double-clicked error - and shows it.
pub fn go_to(h: HWND, line: usize, column: usize) {
    if let Some(v) = view(h) {
        let l = line.saturating_sub(1).min(v.model.buf.line_count() - 1);
        let s = v.model.buf.line(l);
        let col = s.char_indices().nth(column.saturating_sub(1)).map(|(i, _)| i).unwrap_or(s.len());
        v.model.caret = v.model.buf.line_start(l) + col;
        v.model.anchor = v.model.caret;
        v.top = l.saturating_sub(5);
        update_scroll(h);
        place_caret(h);
        unsafe {
            let _ = Gdi::InvalidateRect(Some(h), None, false);
            let _ = km::SetFocus(Some(h));
        }
    }
}

fn update_scroll(h: HWND) {
    let Some(v) = view(h) else { return };
    let page = v.visible_lines(h);
    unsafe {
        let si = wm::SCROLLINFO { cbSize: std::mem::size_of::<wm::SCROLLINFO>() as u32, fMask: wm::SIF_ALL, nMin: 0, nMax: (v.model.buf.line_count() + page.saturating_sub(1)) as i32, nPage: page as u32, nPos: v.top as i32, nTrackPos: 0 };
        windows::Win32::UI::Controls::SetScrollInfo(h, wm::SB_VERT, &si, true);
        let mut r = RECT::default();
        let _ = wm::GetClientRect(h, &mut r);
        let widest = (0..v.model.buf.line_count()).map(|i| v.model.buf.line(i).chars().count()).max().unwrap_or(0) as i32 * v.char_w + v.gutter() + v.char_w * 4;
        let hs = wm::SCROLLINFO { cbSize: std::mem::size_of::<wm::SCROLLINFO>() as u32, fMask: wm::SIF_ALL, nMin: 0, nMax: widest, nPage: (r.right - r.left).max(1) as u32, nPos: v.left, nTrackPos: 0 };
        windows::Win32::UI::Controls::SetScrollInfo(h, wm::SB_HORZ, &hs, true);
    }
}

/// Scrolls so the caret is in view, and moves the caret there.
fn place_caret(h: HWND) {
    let Some(v) = view(h) else { return };
    let (line, _) = v.model.buf.position(v.model.caret);
    let page = v.visible_lines(h);
    if line < v.top {
        v.top = line;
    } else if line >= v.top + page {
        v.top = line + 1 - page;
    }
    unsafe {
        let dc = Gdi::GetDC(Some(h));
        let old = Gdi::SelectObject(dc, v.font.into());
        let (mut x, _) = v.caret_point(dc);
        let mut r = RECT::default();
        let _ = wm::GetClientRect(h, &mut r);
        let gutter = v.gutter();
        if x < gutter {
            v.left = (v.left - (gutter - x) - v.char_w * 8).max(0);
        } else if x > r.right - v.char_w * 2 {
            v.left += x - (r.right - v.char_w * 2) + v.char_w * 8;
        }
        (x, _) = v.caret_point(dc);
        let y = (line as i32 - v.top as i32) * v.line_h;
        Gdi::SelectObject(dc, old);
        Gdi::ReleaseDC(Some(h), dc);
        let _ = wm::SetCaretPos(x, y);
    }
    update_scroll(h);
}

fn changed(h: HWND) {
    if let Some(v) = view(h) {
        v.edits += 1;
        v.version += 1;
        // Old marks point at text that moved; the server sends fresh ones.
        v.marks.clear();
        hide_tip(v);
    }
    place_caret(h);
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
        let id = wm::GetDlgCtrlID(h) as usize;
        if let Ok(p) = wm::GetParent(h) {
            wm::SendMessageW(p, wm::WM_COMMAND, Some(WPARAM(((CHANGED as usize) << 16) | (id & 0xFFFF))), Some(LPARAM(h.0 as isize)));
        }
    }
}

fn moved(h: HWND) {
    place_caret(h);
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

/// Makes the font again at the size now asked for.
fn refont(h: HWND, v: &mut View) {
    if v.pt == font_size() {
        return;
    }
    unsafe {
        let _ = Gdi::DeleteObject(v.font.into());
    }
    v.font = code_font(h);
    (v.line_h, v.char_w) = metrics(h, v.font);
    v.pt = font_size();
    unsafe {
        let _ = wm::DestroyCaret();
        if km::GetFocus() == h {
            let _ = wm::CreateCaret(h, None, win::px(h, 2), v.line_h);
            let _ = wm::ShowCaret(Some(h));
        }
    }
    update_scroll(h);
    place_caret(h);
}

/// Text larger (`by` > 0), smaller, or back to 10 points (`by` 0).
pub fn zoom(h: HWND, by: i32) {
    set_font_size(if by == 0 { 10 } else { font_size() + by });
    if let Some(v) = view(h) {
        refont(h, v);
    }
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

fn paint(h: HWND, v: &mut View) {
    refont(h, v);
    v.refresh_comments();
    refresh_hits(v);
    unsafe {
        let mut ps = Gdi::PAINTSTRUCT::default();
        let screen = Gdi::BeginPaint(h, &mut ps);
        let mut r = RECT::default();
        let _ = wm::GetClientRect(h, &mut r);
        let (w, hgt) = (r.right - r.left, r.bottom - r.top);
        // Drawn off screen, then copied: no flicker while typing.
        let dc = Gdi::CreateCompatibleDC(Some(screen));
        let bmp = Gdi::CreateCompatibleBitmap(screen, w.max(1), hgt.max(1));
        let old_bmp = Gdi::SelectObject(dc, bmp.into());
        let old_font = Gdi::SelectObject(dc, v.font.into());
        let dark = theme::current().is_dark();
        let bg = theme::style("edit", State::default()).background_or(win::window_background());
        win::fill(dc, &r, bg);
        let muted = theme::current().var_color("--muted").unwrap_or(Color::rgb(0x80, 0x80, 0x80));
        let select_bg = theme::style("list", State { selected: true, ..Default::default() }).background_or(Color::rgb(0xcc, 0xe8, 0xff));
        let gutter = v.gutter();
        let (sa, sb) = v.model.selection();
        let caret_line = v.model.buf.position(v.model.caret).0;
        Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        let page = v.visible_lines(h) + 1;
        for line in v.top..(v.top + page).min(v.model.buf.line_count()) {
            let y = (line - v.top) as i32 * v.line_h;
            let s = v.model.buf.line(line);
            let start = v.model.buf.line_start(line);
            // Every place the find bar's words are, softly.
            if !v.hits.is_empty() {
                let hit_bg = theme::current().var_color("--find").unwrap_or(if dark { Color::rgb(0x61, 0x51, 0x1e) } else { Color::rgb(0xff, 0xe9, 0x8f) });
                for &(a, b) in v.hits.iter().filter(|(a, b)| *b > start && *a <= start + s.len()) {
                    let (a, b) = (a.max(start) - start, b.min(start + s.len()) - start);
                    let x0 = gutter + v.width_of(dc, &s[..a]) - v.left;
                    let x1 = gutter + v.width_of(dc, &s[..b]) - v.left;
                    win::fill(dc, &RECT { left: x0.max(gutter), top: y, right: x1, bottom: y + v.line_h }, hit_bg);
                }
            }
            // The selection on this line, end of line included when it runs on.
            if sb > sa && sb >= start && sa <= start + s.len() {
                let a = sa.max(start) - start;
                let b = (sb.min(start + s.len())) - start;
                let x0 = gutter + v.width_of(dc, &s[..a]) - v.left;
                let mut x1 = gutter + v.width_of(dc, &s[..b]) - v.left;
                if sb > start + s.len() {
                    x1 += v.char_w / 2;
                }
                win::fill(dc, &RECT { left: x0.max(gutter), top: y, right: x1, bottom: y + v.line_h }, select_bg);
            }
            // The text, token by token.
            let (tokens, _) = line_tokens(v.lang, &s, v.comments.get(line).copied().unwrap_or(false));
            let mut x = gutter - v.left;
            for (a, b, t) in tokens {
                let piece = &s[a..b];
                let pw = v.width_of(dc, piece);
                if x + pw >= gutter && x < w {
                    Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(colour(t, dark).colorref()));
                    let text = wide_of(&piece.replace('\t', "    "));
                    let _ = Gdi::ExtTextOutW(dc, x, y + win::px(h, 2), Gdi::ETO_OPTIONS(0), None, PCWSTR(text.as_ptr()), text.len() as u32, None);
                }
                x += pw;
            }
            // Wavy lines under what the language server found.
            for m in v.marks.iter().filter(|m| m.to > start && m.from <= start + s.len()) {
                let a = m.from.max(start) - start;
                let b = (m.to.min(start + s.len())).max(m.from.max(start)) - start;
                let x0 = gutter + v.width_of(dc, &s[..a]) - v.left;
                let x1 = (gutter + v.width_of(dc, &s[..b]) - v.left).max(x0 + v.char_w);
                let colour = if m.error { Color::rgb(0xe5, 0x14, 0x00) } else { Color::rgb(0xbf, 0x88, 0x03) };
                let pen = Gdi::CreatePen(Gdi::PS_SOLID, win::px(h, 1).max(1), windows::Win32::Foundation::COLORREF(colour.colorref()));
                let old = Gdi::SelectObject(dc, pen.into());
                let step = win::px(h, 2).max(2);
                let base = y + v.line_h - win::px(h, 2);
                let mut x = x0.max(gutter);
                let _ = Gdi::MoveToEx(dc, x, base, None);
                let mut up = true;
                while x < x1 {
                    x += step;
                    let _ = Gdi::LineTo(dc, x.min(x1), if up { base - step } else { base });
                    up = !up;
                }
                Gdi::SelectObject(dc, old);
                let _ = Gdi::DeleteObject(pen.into());
            }
            // The gutter on top, so text scrolled left goes under it.
            win::fill(dc, &RECT { left: 0, top: y, right: gutter - v.char_w / 2, bottom: y + v.line_h }, bg);
            let n = wide_of(&(line + 1).to_string());
            let mut nr = RECT { left: 0, top: y + win::px(h, 2), right: gutter - v.char_w, bottom: y + v.line_h };
            Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(if line == caret_line { win::window_text() } else { muted }.colorref()));
            let mut nn = n.clone();
            Gdi::DrawTextW(dc, &mut nn, &mut nr, Gdi::DT_RIGHT | Gdi::DT_SINGLELINE);
        }
        let _ = Gdi::BitBlt(screen, 0, 0, w, hgt, Some(dc), 0, 0, Gdi::SRCCOPY);
        Gdi::SelectObject(dc, old_font);
        Gdi::SelectObject(dc, old_bmp);
        let _ = Gdi::DeleteObject(bmp.into());
        let _ = Gdi::DeleteDC(dc);
        let _ = Gdi::EndPaint(h, &ps);
    }
}

fn clipboard_set(h: HWND, text: &str) {
    use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData};
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    let w: Vec<u16> = text.replace('\n', "\r\n").encode_utf16().chain(Some(0)).collect();
    unsafe {
        if OpenClipboard(Some(h)).is_err() {
            return;
        }
        let _ = EmptyClipboard();
        if let Ok(mem) = GlobalAlloc(GMEM_MOVEABLE, w.len() * 2) {
            let p = GlobalLock(mem) as *mut u16;
            if !p.is_null() {
                std::ptr::copy_nonoverlapping(w.as_ptr(), p, w.len());
                let _ = GlobalUnlock(mem);
                let _ = SetClipboardData(13, Some(windows::Win32::Foundation::HANDLE(mem.0)));
            }
        }
        let _ = CloseClipboard();
    }
}

fn clipboard_get(h: HWND) -> Option<String> {
    use windows::Win32::Foundation::HGLOBAL;
    use windows::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
    use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
    unsafe {
        OpenClipboard(Some(h)).ok()?;
        let out = (|| {
            let data = GetClipboardData(13).ok()?;
            let mem = HGLOBAL(data.0);
            let p = GlobalLock(mem) as *const u16;
            if p.is_null() {
                return None;
            }
            let mut n = 0;
            while *p.add(n) != 0 {
                n += 1;
            }
            let s = String::from_utf16_lossy(std::slice::from_raw_parts(p, n));
            let _ = GlobalUnlock(mem);
            Some(s.replace("\r\n", "\n"))
        })();
        let _ = CloseClipboard();
        out
    }
}

fn key(h: HWND, v: &mut View, vk: u16) -> bool {
    let shift = unsafe { km::GetKeyState(0x10) } < 0;
    let ctrl = unsafe { km::GetKeyState(0x11) } < 0;
    // An open completion list takes the arrows, Enter, Tab and Escape.
    if v.list.is_some() {
        match vk {
            0x26 | 0x28 => {
                if let Some(l) = v.list.as_mut() {
                    l.move_choice(if vk == 0x26 { -1 } else { 1 });
                }
                return true;
            }
            0x0D | 0x09 => {
                accept(h, v);
                v.swallow_char = true;
                return true;
            }
            0x1B => {
                v.list = None;
                return true;
            }
            0x25 | 0x27 | 0x24 | 0x23 | 0x21 | 0x22 => v.list = None,
            _ => {}
        }
    }
    if vk == 0x20 && ctrl {
        notify(h, COMPLETE);
        return true;
    }
    match (vk, ctrl) {
        (0x46, true) => {
            open_find(h, v, false);
            return true;
        }
        (0x48, true) => {
            open_find(h, v, true);
            return true;
        }
        (0x72, _) => {
            find_step(h, v, !shift);
            return true;
        }
        (0x1B, _) if v.find.is_some() => {
            close_find(h, v);
            return true;
        }
        _ => {}
    }
    // Ctrl+= / Ctrl++ larger, Ctrl+- smaller, Ctrl+0 back to the usual size.
    if ctrl && matches!(vk, 0xBB | 0x6B | 0xBD | 0x6D | 0x30 | 0x60) {
        zoom(h, match vk {
            0xBB | 0x6B => 1,
            0xBD | 0x6D => -1,
            _ => 0,
        });
        return true;
    }
    let page = v.visible_lines(h).saturating_sub(1).max(1);
    let m = match vk {
        0x25 => Some(if ctrl { Move::WordLeft } else { Move::Left }),
        0x27 => Some(if ctrl { Move::WordRight } else { Move::Right }),
        0x26 => Some(Move::Up),
        0x28 => Some(Move::Down),
        0x24 => Some(if ctrl { Move::Top } else { Move::Home }),
        0x23 => Some(if ctrl { Move::Bottom } else { Move::End }),
        0x21 => Some(Move::PageUp(page)),
        0x22 => Some(Move::PageDown(page)),
        _ => None,
    };
    if let Some(m) = m {
        v.model.move_caret(m, shift);
        moved(h);
        return true;
    }
    match (vk, ctrl) {
        (0x2E, _) => {
            v.model.delete_forward();
            changed(h);
        }
        (0x41, true) => {
            v.model.select_all();
            moved(h);
        }
        (0x43, true) | (0x58, true) => {
            let s = v.model.selected_text();
            if !s.is_empty() {
                clipboard_set(h, &s);
                if vk == 0x58 {
                    v.model.insert("");
                    changed(h);
                }
            }
        }
        (0x56, true) => {
            if let Some(s) = clipboard_get(h) {
                v.model.insert(&s);
                changed(h);
            }
        }
        (0x5A, true) => {
            v.model.undo();
            changed(h);
        }
        (0x59, true) => {
            v.model.redo();
            changed(h);
        }
        _ => return false,
    }
    true
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(v) = view(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_PAINT => {
            paint(h, v);
            LRESULT(0)
        }
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_GETDLGCODE => LRESULT(0x0004 | 0x0002 | 0x0080 | 0x0001), // all keys, tab, chars, arrows
        wm::WM_SETFOCUS => {
            unsafe {
                let _ = wm::CreateCaret(h, None, win::px(h, 2), v.line_h);
                let _ = wm::ShowCaret(Some(h));
            }
            place_caret(h);
            LRESULT(0)
        }
        wm::WM_KILLFOCUS => {
            v.list = None;
            unsafe {
                let _ = wm::DestroyCaret();
            }
            LRESULT(0)
        }
        wm::WM_SIZE => {
            update_scroll(h);
            place_find(h, v);
            LRESULT(0)
        }
        // The find bar's buttons and boxes.
        wm::WM_COMMAND => {
            let (id, code) = (win::loword(w.0) as i32, win::hiword(w.0));
            match (id, code) {
                (ID_FIND, 0x0300) => {
                    // As the words are typed, the next place they are is chosen.
                    if let Some(f) = v.find.as_ref() {
                        v.query = win::text_of(f.query);
                    }
                    v.hits_for = usize::MAX;
                    v.model.caret = v.model.selection().0;
                    v.model.anchor = v.model.caret;
                    find_step(h, v, true);
                }
                (ID_NEXT, 0) => find_step(h, v, true),
                (ID_PREV, 0) => find_step(h, v, false),
                (ID_CLOSE, 0) => close_find(h, v),
                (ID_REPLACE, 0) => {
                    let with = v.find.as_ref().map(|f| win::text_of(f.with)).unwrap_or_default();
                    let q = v.query.clone();
                    v.model.replace_one(&q, &with, false);
                    changed(h);
                    say_count(v);
                }
                (ID_ALL, 0) => {
                    let with = v.find.as_ref().map(|f| win::text_of(f.with)).unwrap_or_default();
                    let q = v.query.clone();
                    let n = v.model.replace_all(&q, &with, false);
                    changed(h);
                    if let Some(f) = v.find.as_ref() {
                        win::set_text(f.count, &format!("{n} ✓"));
                    }
                }
                _ => {}
            }
            LRESULT(0)
        }
        wm::WM_KEYDOWN => {
            if key(h, v, w.0 as u16) {
                return LRESULT(0);
            }
            win::def(h, msg, w, l)
        }
        wm::WM_CHAR => {
            let unit = w.0 as u16;
            // The Enter or Tab that took a completion also comes as a character; it is spent.
            if std::mem::take(&mut v.swallow_char) && (unit == 0x0D || unit == 0x09) {
                return LRESULT(0);
            }
            let ctrl = unsafe { km::GetKeyState(0x11) } < 0;
            match unit {
                0x08 => {
                    v.model.backspace();
                    changed(h);
                }
                0x09 => {
                    let shift = unsafe { km::GetKeyState(0x10) } < 0;
                    if v.model.caret != v.model.anchor && v.model.selected_text().contains('\n') || shift {
                        v.model.indent_lines(shift);
                    } else {
                        let (line, col) = v.model.buf.position(v.model.caret);
                        let chars = v.model.buf.line(line)[..col].chars().count();
                        v.model.insert(&" ".repeat(4 - chars % 4));
                    }
                    changed(h);
                }
                0x0D => {
                    v.model.type_text("\n");
                    changed(h);
                }
                u if u < 0x20 || ctrl && u < 0x80 => {}
                0xD800..=0xDBFF => v.high_surrogate = Some(unit),
                0xDC00..=0xDFFF => {
                    if let Some(hi) = v.high_surrogate.take() {
                        let s = String::from_utf16_lossy(&[hi, unit]);
                        v.model.type_text(&s);
                        changed(h);
                    }
                }
                u => {
                    let s = String::from_utf16_lossy(&[u]);
                    let before_colon = {
                        let (line, col) = v.model.buf.position(v.model.caret);
                        v.model.buf.line(line)[..col].ends_with(':')
                    };
                    v.model.type_text(&s);
                    changed(h);
                    // A member or a path asks for completions; typing on narrows the open list.
                    if s == "." || (s == ":" && before_colon) {
                        notify(h, COMPLETE);
                    } else if v.list.is_some() {
                        if s.chars().all(|c| c.is_alphanumeric() || c == '_') {
                            filter_list(h);
                        } else {
                            v.list = None;
                        }
                    }
                }
            }
            LRESULT(0)
        }
        wm::WM_MOUSEMOVE if !v.dragging => {
            // Rest the pointer and a hover is asked; move it and any tip goes.
            if v.tip.is_some() {
                hide_tip(v);
            }
            if !v.tracking {
                let mut t = km::TRACKMOUSEEVENT { cbSize: std::mem::size_of::<km::TRACKMOUSEEVENT>() as u32, dwFlags: km::TME_HOVER | km::TME_LEAVE, hwndTrack: h, dwHoverTime: 500 };
                unsafe {
                    let _ = km::TrackMouseEvent(&mut t);
                }
                v.tracking = true;
            }
            win::def(h, msg, w, l)
        }
        0x02A1 /* WM_MOUSEHOVER */ => {
            v.tracking = false;
            let (x, y) = win::point_of(l);
            let at = unsafe {
                let dc = Gdi::GetDC(Some(h));
                let old = Gdi::SelectObject(dc, v.font.into());
                let at = v.offset_at(dc, x, y);
                Gdi::SelectObject(dc, old);
                Gdi::ReleaseDC(Some(h), dc);
                at
            };
            let mut pt = windows::Win32::Foundation::POINT { x, y };
            unsafe {
                let _ = Gdi::ClientToScreen(h, &mut pt);
            }
            v.hover = Some((at, pt.x, pt.y));
            // Over a wavy line, its message at once; then ask the server for more.
            if let Some(m) = v.marks.iter().find(|m| at >= m.from && at <= m.to) {
                let text = m.message.clone();
                v.tip = Some(crate::popup::Popup::tip(h, v.font, &text, pt.x, pt.y + v.line_h));
            } else {
                notify(h, HOVER);
            }
            LRESULT(0)
        }
        0x02A3 /* WM_MOUSELEAVE */ => {
            v.tracking = false;
            hide_tip(v);
            LRESULT(0)
        }
        wm::WM_LBUTTONDOWN | wm::WM_MOUSEMOVE if msg == wm::WM_LBUTTONDOWN || v.dragging => {
            v.list = None;
            let (x, y) = win::point_of(l);
            unsafe {
                let dc = Gdi::GetDC(Some(h));
                let old = Gdi::SelectObject(dc, v.font.into());
                let at = v.offset_at(dc, x, y);
                Gdi::SelectObject(dc, old);
                Gdi::ReleaseDC(Some(h), dc);
                if msg == wm::WM_LBUTTONDOWN {
                    let _ = km::SetFocus(Some(h));
                    km::SetCapture(h);
                    v.dragging = true;
                    let shift = km::GetKeyState(0x10) < 0;
                    v.model.caret = at;
                    if !shift {
                        v.model.anchor = at;
                    }
                } else {
                    v.model.caret = at;
                }
            }
            moved(h);
            LRESULT(0)
        }
        wm::WM_LBUTTONUP => {
            v.dragging = false;
            unsafe {
                let _ = km::ReleaseCapture();
            }
            LRESULT(0)
        }
        wm::WM_LBUTTONDBLCLK => {
            let (x, y) = win::point_of(l);
            unsafe {
                let dc = Gdi::GetDC(Some(h));
                let old = Gdi::SelectObject(dc, v.font.into());
                let at = v.offset_at(dc, x, y);
                Gdi::SelectObject(dc, old);
                Gdi::ReleaseDC(Some(h), dc);
                v.model.select_word(at);
            }
            moved(h);
            LRESULT(0)
        }
        wm::WM_MOUSEWHEEL if unsafe { km::GetKeyState(0x11) } < 0 => {
            let delta = (w.0 >> 16) as i16 as i32;
            zoom(h, delta.signum());
            LRESULT(0)
        }
        wm::WM_MOUSEWHEEL => {
            let delta = (w.0 >> 16) as i16 as i32;
            let lines = -(delta / 40);
            v.top = (v.top as i64 + lines as i64).clamp(0, v.model.buf.line_count().saturating_sub(1) as i64) as usize;
            update_scroll(h);
            place_caret(h);
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        wm::WM_VSCROLL | wm::WM_HSCROLL => {
            let bar = if msg == wm::WM_VSCROLL { wm::SB_VERT } else { wm::SB_HORZ };
            let mut si = wm::SCROLLINFO { cbSize: std::mem::size_of::<wm::SCROLLINFO>() as u32, fMask: wm::SIF_ALL, ..Default::default() };
            unsafe {
                let _ = wm::GetScrollInfo(h, bar, &mut si);
            }
            let step = if msg == wm::WM_VSCROLL { 1 } else { v.char_w * 4 };
            let pos = match wm::SCROLLBAR_COMMAND(win::loword(w.0) as i32) {
                wm::SB_LINEUP => si.nPos - step,
                wm::SB_LINEDOWN => si.nPos + step,
                wm::SB_PAGEUP => si.nPos - si.nPage as i32,
                wm::SB_PAGEDOWN => si.nPos + si.nPage as i32,
                wm::SB_THUMBTRACK | wm::SB_THUMBPOSITION => si.nTrackPos,
                _ => si.nPos,
            }
            .clamp(0, (si.nMax - si.nPage as i32 + 1).max(0));
            if msg == wm::WM_VSCROLL {
                v.top = pos as usize;
            } else {
                v.left = pos;
            }
            update_scroll(h);
            place_caret(h);
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            if let Some(v) = unsafe { win::take_state::<View>(h) } {
                unsafe {
                    let _ = Gdi::DeleteObject(v.font.into());
                }
            }
            win::forget_kind(h);
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}
