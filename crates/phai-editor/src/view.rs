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
/// Go To Definition was asked for (F12, or Ctrl+click) at [`caret_lsp`].
pub const DEFINITION: u16 = 0x0303;
/// Rename was asked for (F2) at [`caret_lsp`].
pub const RENAME: u16 = 0x0304;
/// Find All References was asked for (Shift+F12) at [`caret_lsp`].
pub const REFERENCES: u16 = 0x0305;
/// A breakpoint was set or taken away (F9, or a click in the margin): see [`breakpoints`].
pub const BREAKPOINTS: u16 = 0x0306;
/// Peek Definition was asked for (Alt+F12) at [`caret_lsp`]: see [`show_peek`].
pub const PEEK: u16 = 0x0307;
/// Parameter Info was asked for (Ctrl+Shift+Space, or `(` / `,` typed) at [`caret_lsp`].
pub const SIGNATURE: u16 = 0x0308;
/// Quick Actions were asked for (Ctrl+.) for [`selection_lsp`].
pub const ACTIONS: u16 = 0x0309;
/// Escape with nothing of the editor's own open - a peek window closes on it.
pub const ESCAPE: u16 = 0x030A;
/// A test's ▶ in the margin was clicked: [`test_clicked`] names it.
pub const RUN_TEST: u16 = 0x030B;

/// How a line differs from the file as git last had it: the bar in the margin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    Added,
    Modified,
    /// Lines taken out just below this one.
    Removed,
    /// A line that is not in the other file (Compare Files' left side).
    Deleted,
}

/// The tests in `text`: each `fn` with `#[test]` above it (attributes between allowed) - its
/// line from 0 and its name.
pub fn tests_in(text: &str) -> Vec<(usize, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        let t = l.trim_start();
        let rest = t.strip_prefix("pub ").unwrap_or(t);
        let rest = rest.strip_prefix("async ").unwrap_or(rest);
        let Some(after) = rest.strip_prefix("fn ") else { continue };
        let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        let mut j = i;
        let mut is_test = false;
        while j > 0 {
            j -= 1;
            let a = lines[j].trim();
            if a.starts_with("#[") {
                if a.starts_with("#[test]") || a.starts_with("#[tokio::test") {
                    is_test = true;
                }
            } else if !a.starts_with("//") {
                break;
            }
        }
        if is_test && !name.is_empty() {
            out.push((i, name));
        }
    }
    out
}

/// What View shows besides the text: spaces and tabs as dots and arrows, a line at each indent
/// level, a line at a column (0 none) - as Visual Studio's View White Space, indent guides and
/// column guide.
pub fn set_view_options(white_space: bool, indent_guides: bool, column: usize) {
    WHITE_SPACE.store(white_space, std::sync::atomic::Ordering::Relaxed);
    INDENT_GUIDES.store(indent_guides, std::sync::atomic::Ordering::Relaxed);
    COLUMN_GUIDE.store(column, std::sync::atomic::Ordering::Relaxed);
}

static WORD_WRAP: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Word Wrap: long lines go on onto the rows below rather than past the window's edge.
pub fn set_word_wrap(on: bool) {
    WORD_WRAP.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// Where each row begins, from each character's byte offset, the character, and where it ends
/// across the line (pixels or columns): no row wider than `max`, broken after the last space that
/// fits, else where it runs out.
pub fn wrap_by(ends: &[(usize, char, i32)], max: i32) -> Vec<usize> {
    let mut out = vec![0];
    let mut row_x = 0;
    let mut row_start = 0usize;
    let mut last_space: Option<(usize, i32)> = None;
    for (k, &(i, c, end)) in ends.iter().enumerate() {
        if end - row_x > max && i > row_start {
            let (at, x) = match last_space {
                Some((sp, x)) if sp > row_start => (sp, x),
                _ => (i, if k > 0 { ends[k - 1].2 } else { 0 }),
            };
            out.push(at);
            row_start = at;
            row_x = x;
            last_space = None;
        }
        if c == ' ' {
            if let Some(&(j, _, _)) = ends.get(k + 1) {
                last_space = Some((j, end));
            }
        }
    }
    out
}

/// Where each row of `s` begins when no row holds more than `cols` columns (a tab to its stop, a
/// Thai mark on the letter before it) - for a test; the editor measures with its font.
pub fn wrap_points(s: &str, cols: usize, tab: usize) -> Vec<usize> {
    let mut col = 0i32;
    let ends: Vec<(usize, char, i32)> = s
        .char_indices()
        .map(|(i, c)| {
            if !crate::model::joins_previous(c) {
                col += if c == '\t' { tab.max(1) as i32 - col % tab.max(1) as i32 } else { 1 };
            }
            (i, c, col)
        })
        .collect();
    wrap_by(&ends, cols as i32)
}

/// Each character of `s` with where it ends across the line, as the font draws it (a tab as the
/// spaces it stands for).
fn char_ends(dc: HDC, s: &str) -> Vec<(usize, char, i32)> {
    let tab = crate::prefs::tab_spaces();
    let mut wide: Vec<u16> = Vec::with_capacity(s.len());
    let mut marks: Vec<(usize, char, usize)> = Vec::with_capacity(s.len());
    for (i, c) in s.char_indices() {
        if c == '\t' {
            wide.extend(tab.encode_utf16());
        } else {
            let mut b = [0u16; 2];
            wide.extend_from_slice(c.encode_utf16(&mut b));
        }
        marks.push((i, c, wide.len()));
    }
    if wide.is_empty() {
        return Vec::new();
    }
    let mut dx = vec![0i32; wide.len()];
    let mut size = SIZE::default();
    unsafe {
        let _ = Gdi::GetTextExtentExPointW(dc, PCWSTR(wide.as_ptr()), wide.len() as i32, i32::MAX, None, Some(dx.as_mut_ptr()), &mut size);
    }
    marks.into_iter().map(|(i, c, end)| (i, c, dx[end - 1])).collect()
}

static BRACKET_COLOURS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
static STICKY_SCROLL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
static SCROLL_MAP: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Bracket pair colours, sticky scroll and the scroll bar map, as Visual Studio's options have them.
pub fn set_view_extras(bracket_colours: bool, sticky_scroll: bool, scroll_map: bool) {
    BRACKET_COLOURS.store(bracket_colours, std::sync::atomic::Ordering::Relaxed);
    STICKY_SCROLL.store(sticky_scroll, std::sync::atomic::Ordering::Relaxed);
    SCROLL_MAP.store(scroll_map, std::sync::atomic::Ordering::Relaxed);
}

/// The scroll bar map's width, design pixels.
const MAP_W: i32 = 10;

static WHITE_SPACE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static INDENT_GUIDES: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
static COLUMN_GUIDE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Expand Selection: the selection grown one step (the word, inside the brackets, the brackets...).
pub fn expand_selection(h: HWND) {
    let Some(v) = view(h) else { return };
    let (a, b) = v.model.selection();
    let text = v.model.buf.text();
    let (x, y) = crate::tools::expand(&text, a, b);
    if (x, y) == (a, b) {
        return;
    }
    v.grown.push((a, b));
    v.model.extra.clear();
    v.model.anchor = x;
    v.model.caret = y;
    moved(h);
}

/// Shrink Selection: back to what Expand Selection grew it from.
pub fn shrink_selection(h: HWND) {
    let Some(v) = view(h) else { return };
    let Some((a, b)) = v.grown.pop() else { return };
    v.model.anchor = a;
    v.model.caret = b;
    moved(h);
}

/// Surround With: the lines the selection touches put in `head`'s block, its placeholder chosen.
pub fn surround(h: HWND, head: &str) {
    let Some(v) = view(h) else { return };
    let (a, b) = v.model.selection();
    let (la, _) = v.model.buf.position(a);
    let (lb, cb) = v.model.buf.position(b);
    // A selection ending at a line's very start does not take that line.
    let lb = if cb == 0 && lb > la { lb - 1 } else { lb };
    let start = v.model.buf.line_start(la);
    let end = v.model.buf.line_start(lb) + v.model.buf.line(lb).len();
    let text = v.model.buf.text();
    let new = crate::tools::surround(&text[start..end], head, &crate::prefs::indent());
    v.model.anchor = start;
    v.model.caret = end;
    v.model.edit_many(&[(start, end, &new)]);
    // The placeholder - `condition`, `items`... - chosen to type over.
    let word = head.split_whitespace().find(|w| ["condition", "items", "option"].contains(&w.trim_end_matches('{')));
    if let Some(at) = word.and_then(|w| new.find(w).map(|i| (i, w.len()))) {
        v.model.anchor = start + at.0;
        v.model.caret = start + at.0 + at.1;
    }
    changed(h);
}

/// `text` typed in at the caret, in place of what is chosen.
pub fn insert_text(h: HWND, text: &str) {
    let Some(v) = view(h) else { return };
    v.model.insert(text);
    changed(h);
}

/// Inlay hints to show after their lines (lines from 0): `total: i32` for a type the code does
/// not write - quiet, as Visual Studio's inline hints.
pub fn set_inlay_hints(h: HWND, hints: Vec<(usize, String)>) {
    if let Some(v) = view(h) {
        v.hints = hints;
        unsafe {
            let _ = Gdi::InvalidateRect(Some(h), None, false);
        }
    }
}

/// Tracepoints to show, lines from 1: a red diamond in the margin, as Visual Studio draws them.
pub fn set_tracepoints(h: HWND, lines: &[u32]) {
    if let Some(v) = view(h) {
        v.tracepoints = lines.iter().filter(|l| **l > 0).map(|l| *l as usize - 1).collect();
        unsafe {
            let _ = Gdi::InvalidateRect(Some(h), None, false);
        }
    }
}

/// Where line `line` (from 1) begins on the screen, its bottom - for a window put under it; none
/// when it is not in view.
pub fn line_screen(h: HWND, line: u32) -> Option<(i32, i32)> {
    let v = view(h)?;
    let row = v.row_of((line as usize).checked_sub(1)?);
    if row < v.top {
        return None;
    }
    let mut pt = windows::Win32::Foundation::POINT { x: v.gutter(), y: ((row - v.top) as i32 + 1) * v.line_h };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
        if pt.y > r.bottom {
            return None;
        }
        let _ = Gdi::ClientToScreen(h, &mut pt);
    }
    Some((pt.x, pt.y))
}

/// The lines changed from the file as git last had it, lines from 0: the bar beside them.
pub fn set_changes(h: HWND, changes: &[(usize, Change)]) {
    if let Some(v) = view(h) {
        v.changes = changes.iter().copied().collect();
        unsafe {
            let _ = Gdi::InvalidateRect(Some(h), None, false);
        }
    }
}

/// The test whose ▶ was clicked last.
pub fn test_clicked(h: HWND) -> Option<String> {
    view(h).and_then(|v| v.clicked_test.clone())
}

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
    /// The face `font` was made in - behind Options after it names another.
    face: String,
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
    /// Places that can fold, (first line, last line), worked out for `regions_for` edits.
    regions: Vec<(usize, usize)>,
    regions_for: usize,
    /// The folded ones, by first line: their lines after the first are hidden.
    folded: Vec<(usize, usize)>,
    /// The lines shown, one per row on screen, in order - every line when nothing is folded.
    /// Each row on screen: its line, and where in the line it begins (past 0 when a long line
    /// is wrapped onto rows below).
    rows: Vec<(usize, usize)>,
    /// Word Wrap: columns a row holds (0 off), and what the rows were worked out for.
    wrap_cols: usize,
    rows_cols: usize,
    /// The find bar's choices: case, whole word, regular expression - kept while the file is open.
    opts: crate::model::FindOpts,
    /// Code lenses: a line, the place they speak of, their words ("3 references").
    lenses: Vec<(usize, (u32, u32), String)>,
    /// Bookmarked lines, from 0.
    bookmarks: Vec<usize>,
    /// Alt+drag: where the box began, (line, column in characters).
    box_from: Option<(usize, usize)>,
    /// The tip is a Peek Definition: it stays while the pointer moves, until Esc or a change.
    peeking: bool,
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
    /// Lines with a breakpoint, from 0.
    breakpoints: Vec<usize>,
    /// The line the debugger is stopped at, from 0.
    current: Option<usize>,
    /// Lines in the text at the last edit - for moving breakpoints with the lines they are on.
    lines_were: usize,
    /// Breakpoints with a condition, from 0.
    conditional: Vec<usize>,
    /// Breakpoints turned off in the Breakpoints window: drawn hollow, not stopped at.
    disabled: Vec<usize>,
    /// Code coverage: each measured line (from 0) and whether the tests ran it.
    coverage: std::collections::HashMap<usize, bool>,
    /// The lines changed since git's last commit of the file.
    changes: std::collections::HashMap<usize, Change>,
    /// The tests' lines and names, as of `tests_for` edits.
    tests: Vec<(usize, String)>,
    tests_for: usize,
    clicked_test: Option<String>,
    /// Tracepoints' lines (from 0).
    tracepoints: Vec<usize>,
    /// The selections Expand Selection grew from, for Shrink Selection.
    grown: Vec<(usize, usize)>,
    /// Inlay hints by line (from 0).
    hints: Vec<(usize, String)>,
    /// How deep in brackets each line begins, as of `depths_for` edits.
    depths: Vec<i32>,
    depths_for: usize,
    /// The lines sticky scroll holds at the top now.
    sticky: Vec<usize>,
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
const ID_CASE: i32 = 8;
const ID_WORD: i32 = 9;
const ID_REGEX: i32 = 10;

fn wide_of(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

impl View {
    fn width_of(&self, dc: HDC, s: &str) -> i32 {
        if s.is_empty() {
            return 0;
        }
        // A tab as the spaces it stands for (Options > Text Editor > Tabs).
        let w = wide_of(&s.replace('\t', &crate::prefs::tab_spaces()));
        let mut size = SIZE::default();
        unsafe {
            let _ = Gdi::GetTextExtentPoint32W(dc, &w, &mut size);
        }
        size.cx
    }

    fn gutter(&self) -> i32 {
        // Line numbers off (Options): no room for them, only for the marks and the folding boxes.
        let digits = if crate::prefs::get().line_numbers { self.model.buf.line_count().to_string().len().max(3) as i32 } else { 0 };
        // Room at the left for a breakpoint's dot and the debugger's arrow; at the right, the
        // folding boxes.
        self.line_h + digits * self.char_w + self.char_w * 4
    }

    /// The fold regions and the rows shown again, when the text changed.
    fn refresh_rows(&mut self) {
        let n = self.model.buf.line_count();
        if self.regions_for != self.edits || self.rows.is_empty() || self.rows.last().is_some_and(|l| l.0 >= n) || self.rows_cols != self.wrap_cols {
            if self.regions_for != self.edits {
                self.regions = crate::model::fold_regions(&self.model.buf.text());
                self.regions_for = self.edits;
                // A fold whose block is gone opens; one still there keeps to its block's end.
                let regions = &self.regions;
                self.folded = self.folded.iter().filter_map(|(a, _)| regions.iter().find(|(r, _)| r == a).copied()).collect();
            }
            let cols = self.wrap_cols;
            let mut rows = Vec::with_capacity(n);
            // Wrapped: each line measured with the editor's font, in pixels.
            let measure = (cols > 0).then(|| unsafe {
                let dc = Gdi::GetDC(None);
                let old = Gdi::SelectObject(dc, self.font.into());
                (dc, old)
            });
            let max = cols as i32 * self.char_w;
            for l in (0..n).filter(|l| !self.folded.iter().any(|(a, z)| l > a && l <= z)) {
                match measure {
                    None => rows.push((l, 0)),
                    Some((dc, _)) => {
                        let s = self.model.buf.line(l);
                        rows.extend(wrap_by(&char_ends(dc, &s), max).into_iter().map(|at| (l, at)));
                    }
                }
            }
            if let Some((dc, old)) = measure {
                unsafe {
                    Gdi::SelectObject(dc, old);
                    Gdi::ReleaseDC(None, dc);
                }
            }
            self.rows = rows;
            self.rows_cols = cols;
        }
    }

    /// The row a line is on - a hidden line's, the row of the fold that hides it.
    fn row_of(&self, line: usize) -> usize {
        // Not worked out yet (before the first paint): nothing is folded.
        if self.rows.is_empty() {
            return line;
        }
        let r = self.rows.partition_point(|x| x.0 < line);
        if self.rows.get(r).is_some_and(|x| x.0 == line) {
            r
        } else {
            r.saturating_sub(1)
        }
    }

    /// The row a place in a line is on - past the first when the line is wrapped.
    fn row_of_pos(&self, line: usize, col: usize) -> usize {
        let first = self.row_of(line);
        let mut r = first;
        while self.rows.get(r + 1).is_some_and(|x| x.0 == line && x.1 <= col) {
            r += 1;
        }
        r
    }

    /// Where a row begins in its line, and where it ends.
    fn seg_of(&self, row: usize) -> (usize, Option<usize>) {
        let Some(&(line, at)) = self.rows.get(row) else { return (0, None) };
        let end = self.rows.get(row + 1).filter(|x| x.0 == line).map(|x| x.1);
        (at, end)
    }

    /// Word Wrap's columns for the window's width now (0 when it is off).
    fn set_wrap_width(&mut self, h: HWND) {
        self.wrap_cols = if WORD_WRAP.load(std::sync::atomic::Ordering::Relaxed) {
            let mut r = RECT::default();
            unsafe {
                let _ = wm::GetClientRect(h, &mut r);
            }
            (((r.right - self.gutter()) / self.char_w.max(1)) - 2).max(20) as usize
        } else {
            0
        };
        if self.wrap_cols > 0 {
            self.left = 0;
        }
    }

    /// The line on a row.
    fn line_of(&self, row: usize) -> usize {
        if self.rows.is_empty() {
            return row.min(self.model.buf.line_count().saturating_sub(1));
        }
        self.rows.get(row).or(self.rows.last()).map(|x| x.0).unwrap_or(0)
    }

    fn row_count(&self) -> usize {
        if self.rows.is_empty() {
            return self.model.buf.line_count().max(1);
        }
        self.rows.len().max(1)
    }

    fn is_hidden(&self, line: usize) -> bool {
        self.folded.iter().any(|(a, z)| line > *a && line <= *z)
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
        let row = self.row_of_pos(line, col);
        let (seg, _) = self.seg_of(row);
        let shift = if seg > 0 { self.width_of(dc, &s[..seg.min(s.len())]) } else { 0 };
        let x = self.gutter() + self.width_of(dc, &s[..col]) - self.left - shift;
        let y = (row as i32 - self.top as i32) * self.line_h;
        (x, y)
    }

    /// The byte offset under a point in the client area.
    fn offset_at(&self, dc: HDC, x: i32, y: i32) -> usize {
        let row = self.top + (y / self.line_h.max(1)).max(0) as usize;
        let line = self.line_of(row);
        let line = line.min(self.model.buf.line_count() - 1);
        let s = self.model.buf.line(line);
        // A wrapped line's row: only its own piece.
        let (seg, end) = if self.rows.get(row).is_some_and(|x| x.0 == line) { self.seg_of(row) } else { (0, None) };
        let seg = seg.min(s.len());
        let end = end.unwrap_or(s.len()).min(s.len());
        let target = x - self.gutter() + self.left + if seg > 0 { self.width_of(dc, &s[..seg]) } else { 0 };
        let mut at = seg;
        while at < end {
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
        win::set_state(h, Box::new(View { pt: font_size(), face: crate::prefs::get().font, model: Model::new(text), lang: Language::of(path), font, line_h, char_w, top: 0, left: 0, comments: vec![], comments_for: usize::MAX, edits: 0, dragging: false, high_surrogate: None, swallow_char: false, marks: vec![], hover: None, tracking: false, tip: None, box_from: None, opts: Default::default(), lenses: vec![], bookmarks: vec![], regions: vec![], regions_for: usize::MAX, folded: vec![], rows: vec![], wrap_cols: 0, rows_cols: 0, peeking: false, list: None, all_items: vec![], version: 1, find: None, query: String::new(), hits: vec![], hits_for: usize::MAX, breakpoints: vec![], current: None, lines_were: 0, conditional: vec![], disabled: vec![], coverage: Default::default(), changes: Default::default(), tests: vec![], tests_for: usize::MAX, clicked_test: None, tracepoints: vec![], grown: vec![], hints: vec![], depths: vec![], depths_for: usize::MAX, sticky: vec![] }));
        // The studio's shortcuts leave typing keys to a box: this is one.
        win::register_kind(h, win::Kind::Edit);
        let _ = windows::Win32::UI::Controls::SetWindowTheme(h, if theme::current().is_dark() { windows::core::w!("DarkMode_Explorer") } else { windows::core::w!("Explorer") }, PCWSTR::null());
        update_scroll(h);
        h
    }
}

fn code_font(h: HWND) -> HFONT {
    let mut lf = Gdi::LOGFONTW { lfHeight: -(font_size() * win::dpi(h) / 72), lfCharSet: Gdi::DEFAULT_CHARSET, lfQuality: Gdi::CLEARTYPE_QUALITY, ..Default::default() };
    // The font Options > Text Editor > Font names.
    for (i, u) in crate::prefs::get().font.encode_utf16().take(31).enumerate() {
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

/// The path of names at a byte offset - `order.total` - as a data tip or Add Watch takes it.
fn path_at(v: &View, at: usize) -> Option<String> {
    let (line, col) = v.model.buf.position(at);
    let s = v.model.buf.line(line);
    let ok = |c: char| c.is_alphanumeric() || c == '_' || c == '.';
    let start = s[..col].char_indices().rev().take_while(|(_, c)| ok(*c)).last().map(|(i, _)| i).unwrap_or(col);
    let end = s[col..].char_indices().find(|(_, c)| !(c.is_alphanumeric() || *c == '_')).map(|(i, _)| col + i).unwrap_or(s.len());
    let p = s[start..end].trim_matches('.').to_string();
    (!p.is_empty() && !p.starts_with(|c: char| c.is_ascii_digit())).then_some(p)
}

/// The path of names under the resting pointer.
pub fn hover_path(h: HWND) -> Option<String> {
    let v = view(h)?;
    let (at, _, _) = v.hover?;
    path_at(v, at)
}

/// The path of names at the caret - or what is selected.
pub fn caret_path(h: HWND) -> Option<String> {
    let v = view(h)?;
    let sel = v.model.selected_text();
    if !sel.trim().is_empty() && !sel.contains('\n') {
        return Some(sel.trim().to_string());
    }
    path_at(v, v.model.caret)
}

/// The caret's line, from 1.
pub fn caret_line(h: HWND) -> u32 {
    view(h).map(|v| v.model.buf.position(v.model.caret).0 as u32 + 1).unwrap_or(1)
}

/// The breakpoints that have a condition, lines from 1: drawn with a white bar across.
pub fn set_conditional(h: HWND, lines: &[u32]) {
    if let Some(v) = view(h) {
        v.conditional = lines.iter().filter(|l| **l > 0).map(|l| *l as usize - 1).collect();
        unsafe {
            let _ = Gdi::InvalidateRect(Some(h), None, false);
        }
    }
}

/// Code coverage to show: lines from 1 and whether they ran - a bar at the margin's left, green
/// or red; none to take it away.
pub fn set_coverage(h: HWND, lines: &[(u32, bool)]) {
    if let Some(v) = view(h) {
        v.coverage = lines.iter().filter(|(l, _)| *l > 0).map(|(l, ran)| (*l as usize - 1, *ran)).collect();
        unsafe {
            let _ = Gdi::InvalidateRect(Some(h), None, false);
        }
    }
}

/// The breakpoints turned off, lines from 1: drawn as a hollow circle.
pub fn set_disabled(h: HWND, lines: &[u32]) {
    if let Some(v) = view(h) {
        v.disabled = lines.iter().filter(|l| **l > 0).map(|l| *l as usize - 1).collect();
        unsafe {
            let _ = Gdi::InvalidateRect(Some(h), None, false);
        }
    }
}

/// The caret, as the language server counts.
pub fn caret_lsp(h: HWND) -> (u32, u32) {
    view(h).map(|v| lsp_position_of(v, v.model.caret)).unwrap_or((0, 0))
}

/// The selection (or the caret) as the language server counts places: start and end.
pub fn selection_lsp(h: HWND) -> ((u32, u32), (u32, u32)) {
    view(h).map(|v| {
        let (a, b) = v.model.selection();
        (lsp_position_of(v, a), lsp_position_of(v, b))
    }).unwrap_or(((0, 0), (0, 0)))
}

/// Where the caret is on the screen: its left, and the bottom of its line - for a menu under it.
pub fn caret_screen(h: HWND) -> (i32, i32) {
    let Some(v) = view(h) else { return (0, 0) };
    let mut pt = windows::Win32::Foundation::POINT::default();
    unsafe {
        let _ = wm::GetCaretPos(&mut pt);
        let _ = Gdi::ClientToScreen(h, &mut pt);
    }
    (pt.x, pt.y + v.line_h)
}

/// Shows a hover's words where the pointer rests.
pub fn show_tip(h: HWND, text: &str) {
    let Some(v) = view(h) else { return };
    let Some((_, x, y)) = v.hover else { return };
    v.tip = Some(crate::popup::Popup::tip(h, v.font, text, x, y + v.line_h));
}

fn hide_tip(v: &mut View) {
    v.tip = None;
    v.peeking = false;
}

/// Peek Definition: `text` (a heading line, then the code) in a window under the caret's line,
/// kept until Esc, a click or a change - the file and the caret stay where they are.
pub fn show_peek(h: HWND, text: &str) {
    let Some(v) = view(h) else { return };
    let mut pt = windows::Win32::Foundation::POINT::default();
    unsafe {
        let _ = wm::GetCaretPos(&mut pt);
        let _ = Gdi::ClientToScreen(h, &mut pt);
    }
    v.tip = Some(crate::popup::Popup::tip(h, v.font, text, pt.x, pt.y + v.line_h));
    v.peeking = true;
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
    v.hits = if v.find.is_some() { crate::model::find_matches(&v.model.buf.text(), &v.query, v.opts) } else { vec![] };
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
    if v.model.find_with(&q, v.opts, forward) {
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
                        v.model.replace_one_with(&q, &text, v.opts);
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
            // Match case, Whole word, Regular expression - as Visual Studio's find box has them.
            for (id, label, on) in [(ID_CASE, "Aa", v.opts.case), (ID_WORD, "ab|", v.opts.word), (ID_REGEX, ".*", v.opts.regex)] {
                win::checkbox(panel, label, id, font, on);
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
    let width = px(500).min(r.right - px(20));
    let rows = if f.replacing { 2 } else { 1 };
    let height = m + rows * (row + m);
    win::move_to(f.panel, r.right - width - px(16), 0, width, height);
    let opts_w = px(46) + px(52) + px(44);
    let box_w = width - 2 * m - 3 * (b + px(2)) - px(52) - opts_w;
    win::move_to(f.query, m, m, box_w, row);
    let mut ox = m + box_w + px(4);
    for (id, w) in [(ID_CASE, px(46)), (ID_WORD, px(52)), (ID_REGEX, px(44))] {
        unsafe {
            if let Ok(c) = wm::GetDlgItem(Some(f.panel), id) {
                win::move_to(c, ox, m, w, row);
            }
        }
        ox += w;
    }
    win::move_to(f.count, ox, m, px(48), row);
    let mut x = ox + px(48);
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

/// Format Document: the text made `new` as one undo step; false when it was so already.
pub fn replace_whole(h: HWND, new: &str) -> bool {
    let Some(v) = view(h) else { return false };
    let done = v.model.replace_whole(new);
    if done {
        changed(h);
    }
    done
}

/// What is chosen in the editor, if anything.
pub fn selected_text(h: HWND) -> String {
    view(h).map(|v| v.model.selected_text()).unwrap_or_default()
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

/// Selects an LSP range - a definition's name - and shows it a few lines from the top.
pub fn select_lsp(h: HWND, start: (u32, u32), end: (u32, u32)) {
    if let Some(v) = view(h) {
        let at = |(line, ch): (u32, u32)| {
            let l = (line as usize).min(v.model.buf.line_count() - 1);
            v.model.buf.line_start(l) + crate::lsp::from_lsp(&v.model.buf.line(l), ch)
        };
        v.model.anchor = at(start);
        v.model.caret = at(end);
        unfold_at(v, start.0 as usize);
        let row = v.row_of(start.0 as usize);
        let page = v.visible_lines(h);
        if row < v.top || row >= v.top + page {
            v.top = row.saturating_sub(5);
        }
        update_scroll(h);
        place_caret(h);
        unsafe {
            let _ = Gdi::InvalidateRect(Some(h), None, false);
            let _ = km::SetFocus(Some(h));
        }
    }
}

fn toggle_line(h: HWND, v: &mut View, line: usize) {
    match v.breakpoints.iter().position(|l| *l == line) {
        Some(i) => {
            v.breakpoints.remove(i);
        }
        None => v.breakpoints.push(line),
    }
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
    notify(h, BREAKPOINTS);
}

/// F9: a breakpoint on the caret's line, or none.
pub fn toggle_breakpoint(h: HWND) {
    if let Some(v) = view(h) {
        let line = v.model.buf.position(v.model.caret).0;
        toggle_line(h, v, line);
    }
}

/// The lines with a breakpoint, from 1, in order.
pub fn breakpoints(h: HWND) -> Vec<u32> {
    let mut b: Vec<u32> = view(h).map(|v| v.breakpoints.iter().map(|l| *l as u32 + 1).collect()).unwrap_or_default();
    b.sort_unstable();
    b
}

/// Puts the breakpoints on these lines, from 1 - kept from before, or moved by the debugger.
pub fn set_breakpoints(h: HWND, lines: &[u32]) {
    if let Some(v) = view(h) {
        v.breakpoints = lines.iter().filter(|l| **l > 0).map(|l| *l as usize - 1).collect();
        v.breakpoints.dedup();
        unsafe {
            let _ = Gdi::InvalidateRect(Some(h), None, false);
        }
    }
}

/// Shows the debugger stopped at `line` (from 1) - a yellow line and arrow, scrolled into view -
/// or nowhere.
pub fn set_current(h: HWND, line: Option<u32>) {
    let Some(v) = view(h) else { return };
    v.current = line.filter(|l| *l > 0).map(|l| l as usize - 1);
    if let Some(l) = v.current {
        unfold_at(v, l);
        let page = v.visible_lines(h);
        let row = v.row_of(l);
        if row < v.top || row >= v.top + page {
            v.top = row.saturating_sub(page / 3);
            update_scroll(h);
        }
        v.model.caret = v.model.buf.line_start(l.min(v.model.buf.line_count() - 1));
        v.model.anchor = v.model.caret;
        place_caret(h);
    }
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

/// Alt+Up / Alt+Down from a menu: the chosen lines moved.
pub fn move_lines(h: HWND, up: bool) {
    if let Some(v) = view(h) {
        if v.model.move_lines(up) {
            changed(h);
        }
    }
}

/// Opens the fold that hides `line`, if one does.
fn unfold_at(v: &mut View, line: usize) {
    if v.is_hidden(line) {
        v.folded.retain(|(a, z)| !(line > *a && line <= *z));
        v.rows.clear();
        v.refresh_rows();
    }
}

/// Folds the block that starts at `line`, or opens it.
fn toggle_fold(h: HWND, line: usize) {
    let Some(v) = view(h) else { return };
    v.refresh_rows();
    if let Some(i) = v.folded.iter().position(|(a, _)| *a == line) {
        v.folded.remove(i);
    } else if let Some(&r) = v.regions.iter().find(|(a, _)| *a == line) {
        v.folded.push(r);
        v.folded.sort();
        // The caret inside goes to the block's first line.
        let (cl, _) = v.model.buf.position(v.model.caret);
        if cl > r.0 && cl <= r.1 {
            v.model.caret = v.model.buf.line_start(r.0) + v.model.buf.line(r.0).len();
            v.model.anchor = v.model.caret;
        }
    }
    v.rows.clear();
    v.refresh_rows();
    v.top = v.top.min(v.row_count().saturating_sub(1));
    update_scroll(h);
    place_caret(h);
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

/// Ctrl+M, Ctrl+M: the innermost block the caret is in (or starts) folds, or opens.
pub fn toggle_fold_at_caret(h: HWND) {
    let Some(v) = view(h) else { return };
    v.refresh_rows();
    let (line, _) = v.model.buf.position(v.model.caret);
    let r = v.folded.iter().find(|(a, _)| *a == line).copied().or_else(|| v.regions.iter().filter(|(a, z)| *a <= line && line <= *z).max_by_key(|(a, _)| *a).copied());
    if let Some((a, _)) = r {
        toggle_fold(h, a);
    }
}

/// Ctrl+M, Ctrl+L: everything folded opens; else every block folds.
pub fn toggle_all_folds(h: HWND) {
    let Some(v) = view(h) else { return };
    v.refresh_rows();
    if v.folded.is_empty() {
        v.folded = v.regions.clone();
        let (cl, _) = v.model.buf.position(v.model.caret);
        if v.is_hidden(cl) {
            v.model.caret = 0;
            v.model.anchor = 0;
        }
    } else {
        v.folded.clear();
    }
    v.rows.clear();
    v.refresh_rows();
    v.top = v.top.min(v.row_count().saturating_sub(1));
    update_scroll(h);
    place_caret(h);
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

/// Bookmarks: the caret's line marked or unmarked (Ctrl+K, Ctrl+K), the next or the one before
/// (Ctrl+K, Ctrl+N / Ctrl+P, round the end), or all cleared (Ctrl+K, Ctrl+L).
pub enum Bookmark {
    Toggle,
    Next,
    Previous,
    Clear,
}

pub fn bookmark(h: HWND, what: Bookmark) -> bool {
    let Some(v) = view(h) else { return false };
    let line = v.model.buf.position(v.model.caret).0;
    match what {
        Bookmark::Toggle => {
            if let Some(i) = v.bookmarks.iter().position(|b| *b == line) {
                v.bookmarks.remove(i);
            } else {
                v.bookmarks.push(line);
                v.bookmarks.sort();
            }
        }
        Bookmark::Clear => v.bookmarks.clear(),
        Bookmark::Next | Bookmark::Previous => {
            let next = matches!(what, Bookmark::Next);
            let to = if next { v.bookmarks.iter().find(|b| **b > line).or(v.bookmarks.first()) } else { v.bookmarks.iter().rev().find(|b| **b < line).or(v.bookmarks.last()) };
            let Some(&to) = to else { return false };
            go_to(h, to + 1, 1);
            return true;
        }
    }
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
    true
}

/// The bookmarked lines, from 1 - for checking.
pub fn bookmarks(h: HWND) -> Vec<usize> {
    view(h).map(|v| v.bookmarks.iter().map(|b| b + 1).collect()).unwrap_or_default()
}

/// Insert Snippet: the snippet's word typed at the caret and made its code.
pub fn insert_snippet(h: HWND, word: &str) {
    if let Some(v) = view(h) {
        v.model.insert(word);
        v.model.expand_snippet();
        changed(h);
    }
}

/// The code lenses to show: (line from 0, the place they speak of, their words).
pub fn set_lenses(h: HWND, lenses: Vec<(u32, (u32, u32), String)>) {
    if let Some(v) = view(h) {
        v.lenses = lenses.into_iter().map(|(l, at, w)| (l as usize, at, w)).collect();
        unsafe {
            let _ = Gdi::InvalidateRect(Some(h), None, false);
        }
    }
}

/// The code lens under a point, if one is there: the place it speaks of.
fn lens_at(h: HWND, v: &View, x: i32, y: i32) -> Option<(u32, u32)> {
    let line = v.line_of(v.top + (y / v.line_h.max(1)).max(0) as usize);
    let (_, at, words) = v.lenses.iter().find(|(l, at, _)| *l == line && (crate::prefs::get().lenses || at.1 == u32::MAX))?;
    // Words only (a line's blame): nothing to click.
    if at.1 == u32::MAX {
        return None;
    }
    unsafe {
        let dc = Gdi::GetDC(Some(h));
        let old = Gdi::SelectObject(dc, v.font.into());
        let s = v.model.buf.line(line);
        let x0 = v.gutter() + v.width_of(dc, &s) - v.left + v.char_w * 3;
        let x1 = x0 + v.width_of(dc, words);
        Gdi::SelectObject(dc, old);
        Gdi::ReleaseDC(Some(h), dc);
        (x >= x0 && x <= x1).then_some(*at)
    }
}

/// How many carets there are - for checking.
pub fn caret_count(h: HWND) -> usize {
    view(h).map(|v| 1 + v.model.extra.len()).unwrap_or(0)
}

/// Box selection from (line, column) to (line, column), lines and columns from 1 - for checking.
pub fn box_select(h: HWND, from: (usize, usize), to: (usize, usize)) {
    if let Some(v) = view(h) {
        v.model.box_select((from.0 - 1, from.1 - 1), (to.0 - 1, to.1 - 1));
        moved(h);
    }
}

/// How many lines are folded away now - for checking.
pub fn hidden_lines(h: HWND) -> usize {
    view(h).map(|v| v.model.buf.line_count() - v.rows.len().min(v.model.buf.line_count())).unwrap_or(0)
}

/// Ctrl+D: the line (or the chosen part) again.
pub fn duplicate(h: HWND) {
    if let Some(v) = view(h) {
        v.model.duplicate();
        changed(h);
    }
}

/// Comments the chosen lines or takes the comment off, as Ctrl+/ does.
pub fn toggle_comment(h: HWND) {
    if let Some(v) = view(h) {
        v.model.toggle_comment("//");
        changed(h);
    }
}

/// Makes a language server's changes to the text, as one undo step; how many were made.
pub fn apply_lsp_edits(h: HWND, edits: &[crate::lsp::TextEdit]) -> usize {
    let Some(v) = view(h) else { return 0 };
    let text = v.model.buf.text();
    let spans = crate::lsp::edit_spans(&text, edits);
    v.model.edit_many(&spans);
    changed(h);
    spans.len()
}

/// Puts the caret at a line and column from 1 - a double-clicked error - and shows it.
pub fn go_to(h: HWND, line: usize, column: usize) {
    if let Some(v) = view(h) {
        let l = line.saturating_sub(1).min(v.model.buf.line_count() - 1);
        let s = v.model.buf.line(l);
        let col = s.char_indices().nth(column.saturating_sub(1)).map(|(i, _)| i).unwrap_or(s.len());
        v.model.caret = v.model.buf.line_start(l) + col;
        v.model.anchor = v.model.caret;
        unfold_at(v, l);
        v.top = v.row_of(l).saturating_sub(5);
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
        let si = wm::SCROLLINFO { cbSize: std::mem::size_of::<wm::SCROLLINFO>() as u32, fMask: wm::SIF_ALL, nMin: 0, nMax: (v.row_count() + page.saturating_sub(1)) as i32, nPage: page as u32, nPos: v.top as i32, nTrackPos: 0 };
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
    caret_shown(h, true);
}

/// The caret put where it is on the screen after the view scrolled (the wheel, the scroll bar,
/// the scroll bar map): the view stays where it was scrolled to, the caret out of sight if it is.
fn caret_in_place(h: HWND) {
    caret_shown(h, false);
}

/// The caret put on the screen - and, when `follow`, the view scrolled to show it.
fn caret_shown(h: HWND, follow: bool) {
    let Some(v) = view(h) else { return };
    let (line, _) = v.model.buf.position(v.model.caret);
    // The caret in a folded block opens it, as Visual Studio does.
    if v.is_hidden(line) {
        unfold_at(v, line);
    }
    v.set_wrap_width(h);
    v.refresh_rows();
    let col = v.model.buf.position(v.model.caret).1;
    let row = v.row_of_pos(line, col);
    let page = v.visible_lines(h);
    if !follow {
    } else if row < v.top {
        v.top = row;
    } else if row >= v.top + page {
        v.top = row + 1 - page;
    }
    unsafe {
        let dc = Gdi::GetDC(Some(h));
        let old = Gdi::SelectObject(dc, v.font.into());
        let (mut x, _) = v.caret_point(dc);
        let mut r = RECT::default();
        let _ = wm::GetClientRect(h, &mut r);
        let gutter = v.gutter();
        if v.wrap_cols > 0 || !follow {
            // Wrapped: nothing to scroll sideways.
        } else if x < gutter {
            v.left = (v.left - (gutter - x) - v.char_w * 8).max(0);
        } else if x > r.right - v.char_w * 2 {
            v.left += x - (r.right - v.char_w * 2) + v.char_w * 8;
        }
        (x, _) = v.caret_point(dc);
        let y = (row as i32 - v.top as i32) * v.line_h;
        Gdi::SelectObject(dc, old);
        Gdi::ReleaseDC(Some(h), dc);
        let _ = wm::SetCaretPos(x, y);
    }
    update_scroll(h);
}

fn changed(h: HWND) {
    if let Some(v) = view(h) {
        // Lines added or taken away above a breakpoint move it with its line.
        let now = v.model.buf.line_count();
        if v.lines_were != 0 && now != v.lines_were {
            let at = v.model.buf.position(v.model.caret).0;
            let d = now as isize - v.lines_were as isize;
            let from = if d > 0 { at as isize - d } else { at as isize };
            for b in v.breakpoints.iter_mut().chain(v.bookmarks.iter_mut()) {
                if *b as isize > from {
                    *b = (*b as isize + d).max(0) as usize;
                }
            }
            v.breakpoints.dedup();
            v.bookmarks.dedup();
        }
        v.lines_were = now;
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
    let face = crate::prefs::get().font;
    if v.pt == font_size() && v.face == face {
        return;
    }
    v.face = face;
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
    // The tests' ▶: found again when the text changed.
    if v.tests_for != v.edits {
        v.tests = if v.lang == Language::Rust { tests_in(&v.model.buf.text()) } else { Vec::new() };
        v.tests_for = v.edits;
    }
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
        // The bracket by the caret and its match, boxed as Visual Studio shows them.
        let prefs = crate::prefs::get();
        let tab = crate::prefs::tab_spaces();
        let braces = if sa == sb && prefs.braces { crate::model::matching_brace(&v.model.buf.text(), v.model.caret) } else { None };
        let brace_bg = theme::current().var_color("--brace").unwrap_or(if dark { Color::rgb(0x3e, 0x52, 0x6b) } else { Color::rgb(0xdb, 0xe0, 0xe6) });
        Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        let page = v.visible_lines(h) + 1;
        let colours = BRACKET_COLOURS.load(std::sync::atomic::Ordering::Relaxed) && v.lang == Language::Rust;
        if colours && v.depths_for != v.edits {
            let mut d = 0;
            v.depths = (0..v.model.buf.line_count())
                .map(|l| {
                    let at = d;
                    d = crate::tools::brackets(&v.model.buf.line(l), d).1;
                    at
                })
                .collect();
            v.depths_for = v.edits;
        }
        v.set_wrap_width(h);
        v.refresh_rows();
        for row in v.top..(v.top + page).min(v.row_count()) {
            let line = v.line_of(row);
            let y = (row - v.top) as i32 * v.line_h;
            let s = v.model.buf.line(line);
            let start = v.model.buf.line_start(line);
            // A wrapped line's row: the line drawn shifted by where its piece begins - the gutter
            // covers what is before it, the window's edge what is after.
            let (seg, seg_end) = v.seg_of(row);
            let seg = seg.min(s.len());
            let left = v.left + if seg > 0 { v.width_of(dc, &s[..seg]) } else { 0 };
            // What is past the row's break belongs to the row below.
            Gdi::SelectClipRgn(dc, None);
            if let Some(e) = seg_end {
                let x_end = gutter + v.width_of(dc, &s[..e.min(s.len())]) - left;
                let _ = Gdi::IntersectClipRect(dc, 0, y, x_end.max(gutter), y + v.line_h);
            }
            // The line the debugger stopped at, in yellow, as Visual Studio shows it.
            if v.current == Some(line) {
                let at = theme::current().var_color("--debug-line").unwrap_or(if dark { Color::rgb(0x5c, 0x52, 0x14) } else { Color::rgb(0xff, 0xf1, 0x9c) });
                win::fill(dc, &RECT { left: gutter, top: y, right: w, bottom: y + v.line_h }, at);
            }
            // Every place the find bar's words are, softly.
            if !v.hits.is_empty() {
                let hit_bg = theme::current().var_color("--find").unwrap_or(if dark { Color::rgb(0x61, 0x51, 0x1e) } else { Color::rgb(0xff, 0xe9, 0x8f) });
                for &(a, b) in v.hits.iter().filter(|(a, b)| *b > start && *a <= start + s.len()) {
                    let (a, b) = (a.max(start) - start, b.min(start + s.len()) - start);
                    let x0 = gutter + v.width_of(dc, &s[..a]) - left;
                    let x1 = gutter + v.width_of(dc, &s[..b]) - left;
                    win::fill(dc, &RECT { left: x0.max(gutter), top: y, right: x1, bottom: y + v.line_h }, hit_bg);
                }
            }
            if let Some((p, q)) = braces {
                for at in [p, q].into_iter().filter(|a| *a >= start && *a < start + s.len()) {
                    let a = at - start;
                    let x0 = gutter + v.width_of(dc, &s[..a]) - left;
                    let x1 = gutter + v.width_of(dc, &s[..a + 1]) - left;
                    win::fill(dc, &RECT { left: x0.max(gutter), top: y, right: x1, bottom: y + v.line_h }, brace_bg);
                }
            }
            // The selection on this line, end of line included when it runs on.
            if sb > sa && sb >= start && sa <= start + s.len() {
                let a = sa.max(start) - start;
                let b = (sb.min(start + s.len())) - start;
                let x0 = gutter + v.width_of(dc, &s[..a]) - left;
                let mut x1 = gutter + v.width_of(dc, &s[..b]) - left;
                if sb > start + s.len() {
                    x1 += v.char_w / 2;
                }
                win::fill(dc, &RECT { left: x0.max(gutter), top: y, right: x1, bottom: y + v.line_h }, select_bg);
            }
            // The other carets' selections, and the carets themselves as thin bars.
            for &(c, a) in &v.model.extra {
                let (ea, eb) = (c.min(a), c.max(a));
                if eb > ea && eb >= start && ea <= start + s.len() {
                    let a0 = ea.max(start) - start;
                    let b0 = eb.min(start + s.len()) - start;
                    let x0 = gutter + v.width_of(dc, &s[..a0]) - left;
                    let x1 = gutter + v.width_of(dc, &s[..b0]) - left;
                    win::fill(dc, &RECT { left: x0.max(gutter), top: y, right: x1, bottom: y + v.line_h }, select_bg);
                }
                if c >= start && c <= start + s.len() {
                    let cx = gutter + v.width_of(dc, &s[..c - start]) - left;
                    if cx >= gutter {
                        win::fill(dc, &RECT { left: cx, top: y, right: cx + win::px(h, 2), bottom: y + v.line_h }, win::window_text());
                    }
                }
            }
            // The text, token by token.
            let (tokens, _) = line_tokens(v.lang, &s, v.comments.get(line).copied().unwrap_or(false));
            let mut x = gutter - left;
            for (a, b, t) in tokens {
                let piece = &s[a..b];
                let pw = v.width_of(dc, piece);
                if x + pw >= gutter && x < w {
                    Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(colour(t, dark).colorref()));
                    let text = wide_of(&piece.replace('\t', &tab));
                    let _ = Gdi::ExtTextOutW(dc, x, y + win::px(h, 2), Gdi::ETO_OPTIONS(0), None, PCWSTR(text.as_ptr()), text.len() as u32, None);
                }
                x += pw;
            }
            // Indent guides, white space, the column guide - drawn over the text, quietly.
            let guide = muted.mix(bg, 0.55);
            let tab_cols = crate::prefs::get().tab_size.max(1);
            if INDENT_GUIDES.load(std::sync::atomic::Ordering::Relaxed) {
                let lead: usize = s.chars().take_while(|c| *c == ' ' || *c == '\t').map(|c| if c == '\t' { tab_cols } else { 1 }).sum();
                let mut col = tab_cols;
                while col < lead {
                    let gx = gutter + col as i32 * v.char_w - left;
                    if gx >= gutter {
                        win::fill(dc, &RECT { left: gx, top: y, right: gx + 1, bottom: y + v.line_h }, guide);
                    }
                    col += tab_cols;
                }
            }
            if WHITE_SPACE.load(std::sync::atomic::Ordering::Relaxed) {
                for (i, c) in s.char_indices().filter(|(_, c)| *c == ' ' || *c == '\t') {
                    let x0 = gutter + v.width_of(dc, &s[..i]) - left;
                    if x0 < gutter {
                        continue;
                    }
                    if c == ' ' {
                        let d = win::px(h, 2).max(2);
                        let cx = x0 + v.char_w / 2 - d / 2;
                        let cy = y + v.line_h / 2 - d / 2;
                        win::fill(dc, &RECT { left: cx, top: cy, right: cx + d, bottom: cy + d }, guide);
                    } else {
                        Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(guide.colorref()));
                        let arrow = wide_of("\u{2192}");
                        let _ = Gdi::ExtTextOutW(dc, x0, y + win::px(h, 2), Gdi::ETO_OPTIONS(0), None, PCWSTR(arrow.as_ptr()), arrow.len() as u32, None);
                    }
                }
            }
            let column = COLUMN_GUIDE.load(std::sync::atomic::Ordering::Relaxed);
            if column > 0 {
                let gx = gutter + column as i32 * v.char_w - left;
                if gx >= gutter {
                    win::fill(dc, &RECT { left: gx, top: y, right: gx + 1, bottom: y + v.line_h }, guide);
                }
            }
            // Bracket pairs in colours by how deep they are, as Visual Studio's bracket pair colorization.
            if colours {
                const PAIRS: [(u8, u8, u8); 3] = [(0xc0, 0x8a, 0x00), (0xa4, 0x3d, 0xbf), (0x1f, 0x6f, 0xd1)];
                const PAIRS_DARK: [(u8, u8, u8); 3] = [(0xff, 0xd7, 0x00), (0xda, 0x70, 0xd6), (0x17, 0x9f, 0xff)];
                let depth = v.depths.get(line).copied().unwrap_or(0);
                for (i, d) in crate::tools::brackets(&s, depth).0 {
                    let (r, g, b) = if dark { PAIRS_DARK[d as usize % 3] } else { PAIRS[d as usize % 3] };
                    let bx = gutter + v.width_of(dc, &s[..i]) - left;
                    if bx < gutter {
                        continue;
                    }
                    Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(Color::rgb(r, g, b).colorref()));
                    let ch = wide_of(&s[i..i + 1]);
                    let _ = Gdi::ExtTextOutW(dc, bx, y + win::px(h, 2), Gdi::ETO_OPTIONS(0), None, PCWSTR(ch.as_ptr()), ch.len() as u32, None);
                }
            }
            // Inlay hints after the line: the types it does not write, quiet.
            if let Some((_, words)) = v.hints.iter().find(|(l, _)| *l == line) {
                let x0 = x + v.char_w * 2;
                Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(muted.mix(bg, 0.25).colorref()));
                let text = wide_of(words);
                let _ = Gdi::ExtTextOutW(dc, x0, y + win::px(h, 2), Gdi::ETO_OPTIONS(0), None, PCWSTR(text.as_ptr()), text.len() as u32, None);
                x = x0 + v.width_of(dc, words);
            }
            // Code lenses after the line, quiet, as Visual Studio shows them above it.
            // Reference counts only while Options shows them; a line's blame always.
            if let Some((_, _, words)) = v.lenses.iter().find(|(l, at, _)| *l == line && (prefs.lenses || at.1 == u32::MAX)) {
                let x0 = x + v.char_w * 3;
                Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(muted.colorref()));
                let text = wide_of(words);
                let _ = Gdi::ExtTextOutW(dc, x0, y + win::px(h, 2), Gdi::ETO_OPTIONS(0), None, PCWSTR(text.as_ptr()), text.len() as u32, None);
            }
            // Wavy lines under what the language server found.
            for m in v.marks.iter().filter(|m| m.to > start && m.from <= start + s.len()) {
                let a = m.from.max(start) - start;
                let b = (m.to.min(start + s.len())).max(m.from.max(start)) - start;
                let x0 = gutter + v.width_of(dc, &s[..a]) - left;
                let x1 = (gutter + v.width_of(dc, &s[..b]) - left).max(x0 + v.char_w);
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
            if seg > 0 {
                continue;
            }
            let n = wide_of(&if prefs.line_numbers { (line + 1).to_string() } else { String::new() });
            let mut nr = RECT { left: 0, top: y + win::px(h, 2), right: gutter - v.char_w * 3, bottom: y + v.line_h };
            // A block that can fold: a small box, − open, + folded; a folded one shows … after its line.
            if let Some(&(a, z)) = v.regions.iter().find(|(a, _)| *a == line) {
                let folded = v.folded.iter().any(|(f, _)| *f == a);
                let bs = (v.char_w * 3 / 2).max(win::px(h, 9));
                let bx = gutter - v.char_w * 5 / 2;
                let by = y + (v.line_h - bs) / 2;
                let frame = RECT { left: bx, top: by, right: bx + bs, bottom: by + bs };
                win::fill(dc, &frame, muted);
                win::fill(dc, &RECT { left: bx + 1, top: by + 1, right: bx + bs - 1, bottom: by + bs - 1 }, bg);
                let t = 1.max(win::px(h, 1));
                win::fill(dc, &RECT { left: bx + 2 * t + 1, top: by + bs / 2 - t / 2, right: bx + bs - 2 * t - 1, bottom: by + bs / 2 - t / 2 + t }, muted);
                if folded {
                    win::fill(dc, &RECT { left: bx + bs / 2 - t / 2, top: by + 2 * t + 1, right: bx + bs / 2 - t / 2 + t, bottom: by + bs - 2 * t - 1 }, muted);
                    let s_line = v.model.buf.line(line);
                    let x_end = gutter + v.width_of(dc, &s_line) - left + v.char_w;
                    let last = v.model.buf.line(z);
                    let label = format!("…{}", last.trim());
                    let lw = v.width_of(dc, &label) + v.char_w;
                    let tag = RECT { left: x_end, top: y + win::px(h, 2), right: x_end + lw, bottom: y + v.line_h - win::px(h, 1) };
                    win::fill(dc, &tag, muted);
                    win::fill(dc, &RECT { left: tag.left + 1, top: tag.top + 1, right: tag.right - 1, bottom: tag.bottom - 1 }, bg);
                    Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(muted.colorref()));
                    let lt = wide_of(&label);
                    let _ = Gdi::ExtTextOutW(dc, x_end + v.char_w / 2, y + win::px(h, 2), Gdi::ETO_OPTIONS(0), None, PCWSTR(lt.as_ptr()), lt.len() as u32, None);
                }
            }
            Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(if line == caret_line { win::window_text() } else { muted }.colorref()));
            // Line numbers off: nothing to draw (DrawTextW must not be given no text).
            if !n.is_empty() {
                let mut nn = n.clone();
                phai_ui::win::draw_text_w(dc, &mut nn, &mut nr, Gdi::DT_RIGHT | Gdi::DT_SINGLELINE);
            }
            // A breakpoint: a red dot in the margin; the debugger's place: a yellow arrow.
            let d = v.line_h - win::px(h, 6);
            let (bx, by) = (win::px(h, 3), y + (v.line_h - d) / 2);
            // Changed since git's last commit: a bar at the text's edge - green added, blue
            // changed, a red notch where lines were taken out - as Visual Studio and Android Studio show.
            if let Some(ch) = v.changes.get(&line) {
                let gx = v.gutter() - win::px(h, 4);
                match ch {
                    Change::Added => win::fill(dc, &RECT { left: gx, top: y, right: gx + win::px(h, 3), bottom: y + v.line_h }, Color::rgb(0x2e, 0xa0, 0x43)),
                    Change::Modified => win::fill(dc, &RECT { left: gx, top: y, right: gx + win::px(h, 3), bottom: y + v.line_h }, Color::rgb(0x1c, 0x6f, 0xd1)),
                    Change::Removed => win::fill(dc, &RECT { left: gx, top: y + v.line_h - win::px(h, 3), right: gx + win::px(h, 6), bottom: y + v.line_h + win::px(h, 1) }, Color::rgb(0xd1, 0x34, 0x38)),
                    Change::Deleted => win::fill(dc, &RECT { left: gx, top: y, right: gx + win::px(h, 3), bottom: y + v.line_h }, Color::rgb(0xd1, 0x34, 0x38)),
                }
            }
            // A test: a green ▶ in the margin, where a breakpoint is not - a click runs it.
            if v.tests.iter().any(|(l, _)| *l == line) && !v.breakpoints.contains(&line) && !v.bookmarks.contains(&line) {
                let g = Color::rgb(0x2e, 0xa0, 0x43);
                let brush = Gdi::CreateSolidBrush(windows::Win32::Foundation::COLORREF(g.colorref()));
                let pen = Gdi::CreatePen(Gdi::PS_SOLID, 1, windows::Win32::Foundation::COLORREF(g.colorref()));
                let (ob, op) = (Gdi::SelectObject(dc, brush.into()), Gdi::SelectObject(dc, pen.into()));
                let pts = [windows::Win32::Foundation::POINT { x: bx + d / 6, y: by }, windows::Win32::Foundation::POINT { x: bx + d / 6, y: by + d }, windows::Win32::Foundation::POINT { x: bx + d / 6 + d * 5 / 6, y: by + d / 2 }];
                let _ = Gdi::Polygon(dc, &pts);
                Gdi::SelectObject(dc, ob);
                Gdi::SelectObject(dc, op);
                let _ = Gdi::DeleteObject(brush.into());
                let _ = Gdi::DeleteObject(pen.into());
            }
            // Code coverage: a bar at the very left, green where the tests ran the line, red not.
            if let Some(ran) = v.coverage.get(&line) {
                let c = if *ran { Color::rgb(0x2e, 0xa0, 0x43) } else { Color::rgb(0xd1, 0x34, 0x38) };
                win::fill(dc, &RECT { left: 0, top: y, right: win::px(h, 3), bottom: y + v.line_h }, c);
            }
            // A bookmark: a small flag in the margin, where a breakpoint is not.
            if v.bookmarks.contains(&line) && !v.breakpoints.contains(&line) {
                let accent = theme::current().var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f));
                let (fw, fh) = (d * 2 / 3, d);
                win::fill(dc, &RECT { left: bx + d / 6, top: by, right: bx + d / 6 + fw, bottom: by + fh }, accent);
                // Its notch at the bottom, as a ribbon.
                win::fill(dc, &RECT { left: bx + d / 6 + fw / 2 - fw / 6, top: by + fh - fh / 4, right: bx + d / 6 + fw / 2 + fw / 6, bottom: by + fh }, if dark { Color::rgb(0x1e, 0x1e, 0x1e) } else { win::window_background() });
            }
            if v.breakpoints.contains(&line) {
                let red = Color::rgb(0xe5, 0x14, 0x00);
                // Turned off: a ring of it, the margin showing through.
                let off = v.disabled.contains(&line);
                let inside = if off { if dark { Color::rgb(0x1e, 0x1e, 0x1e) } else { win::window_background() } } else { red };
                let brush = Gdi::CreateSolidBrush(windows::Win32::Foundation::COLORREF(inside.colorref()));
                let pen = Gdi::CreatePen(Gdi::PS_SOLID, if off { (d / 8).max(1) } else { 1 }, windows::Win32::Foundation::COLORREF(red.colorref()));
                let (ob, op) = (Gdi::SelectObject(dc, brush.into()), Gdi::SelectObject(dc, pen.into()));
                let _ = Gdi::Ellipse(dc, bx, by, bx + d, by + d);
                Gdi::SelectObject(dc, ob);
                Gdi::SelectObject(dc, op);
                let _ = Gdi::DeleteObject(brush.into());
                let _ = Gdi::DeleteObject(pen.into());
                // A condition on it: a white bar across, as Visual Studio marks one with a +.
                if v.conditional.contains(&line) && !off {
                    let t = (d / 6).max(2);
                    win::fill(dc, &RECT { left: bx + d / 4, top: by + d / 2 - t / 2, right: bx + d - d / 4, bottom: by + d / 2 - t / 2 + t }, Color::rgb(255, 255, 255));
                    win::fill(dc, &RECT { left: bx + d / 2 - t / 2, top: by + d / 4, right: bx + d / 2 - t / 2 + t, bottom: by + d - d / 4 }, Color::rgb(255, 255, 255));
                }
            }
            // A tracepoint: a red diamond, where a breakpoint is not.
            if v.tracepoints.contains(&line) && !v.breakpoints.contains(&line) {
                let red = Color::rgb(0xe5, 0x14, 0x00);
                let brush = Gdi::CreateSolidBrush(windows::Win32::Foundation::COLORREF(red.colorref()));
                let pen = Gdi::CreatePen(Gdi::PS_SOLID, 1, windows::Win32::Foundation::COLORREF(red.colorref()));
                let (ob, op) = (Gdi::SelectObject(dc, brush.into()), Gdi::SelectObject(dc, pen.into()));
                let (cx, cy) = (bx + d / 2, by + d / 2);
                let pts = [
                    windows::Win32::Foundation::POINT { x: cx, y: by },
                    windows::Win32::Foundation::POINT { x: bx + d, y: cy },
                    windows::Win32::Foundation::POINT { x: cx, y: by + d },
                    windows::Win32::Foundation::POINT { x: bx, y: cy },
                ];
                let _ = Gdi::Polygon(dc, &pts);
                Gdi::SelectObject(dc, ob);
                Gdi::SelectObject(dc, op);
                let _ = Gdi::DeleteObject(brush.into());
                let _ = Gdi::DeleteObject(pen.into());
            }
            if v.current == Some(line) {
                let arrow = Color::rgb(0xf2, 0xc8, 0x11);
                let brush = Gdi::CreateSolidBrush(windows::Win32::Foundation::COLORREF(arrow.colorref()));
                let pen = Gdi::CreatePen(Gdi::PS_SOLID, 1, windows::Win32::Foundation::COLORREF(Color::rgb(0x8a, 0x6d, 0x00).colorref()));
                let (ob, op) = (Gdi::SelectObject(dc, brush.into()), Gdi::SelectObject(dc, pen.into()));
                let m = by + d / 2;
                let pts = [
                    windows::Win32::Foundation::POINT { x: bx, y: m - d / 4 },
                    windows::Win32::Foundation::POINT { x: bx + d / 2, y: m - d / 4 },
                    windows::Win32::Foundation::POINT { x: bx + d / 2, y: by },
                    windows::Win32::Foundation::POINT { x: bx + d, y: m },
                    windows::Win32::Foundation::POINT { x: bx + d / 2, y: by + d },
                    windows::Win32::Foundation::POINT { x: bx + d / 2, y: m + d / 4 },
                    windows::Win32::Foundation::POINT { x: bx, y: m + d / 4 },
                ];
                let _ = Gdi::Polygon(dc, &pts);
                Gdi::SelectObject(dc, ob);
                Gdi::SelectObject(dc, op);
                let _ = Gdi::DeleteObject(brush.into());
                let _ = Gdi::DeleteObject(pen.into());
            }
        }
        Gdi::SelectClipRgn(dc, None);
        // Sticky scroll: the lines that open the items around the top line, held at the top.
        v.sticky.clear();
        if STICKY_SCROLL.load(std::sync::atomic::Ordering::Relaxed) && v.lang == Language::Rust && v.top > 0 {
            let first = v.line_of(v.top);
            let all = v.model.buf.text();
            let heads = crate::crumbs::header_lines(&all, first);
            let heads: Vec<usize> = heads.into_iter().rev().take(3).collect::<Vec<_>>().into_iter().rev().collect();
            let shade = bg.mix(muted, 0.08);
            let text_colour = theme::style("edit", State::default()).color_or(win::window_text());
            for (k, l) in heads.iter().enumerate() {
                let y = k as i32 * v.line_h;
                win::fill(dc, &RECT { left: 0, top: y, right: w, bottom: y + v.line_h }, shade);
                let s = v.model.buf.line(*l);
                Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(text_colour.colorref()));
                let text = wide_of(&s.replace('\t', &tab));
                let _ = Gdi::ExtTextOutW(dc, gutter - v.left, y + win::px(h, 2), Gdi::ETO_OPTIONS(0), None, PCWSTR(text.as_ptr()), text.len() as u32, None);
                win::fill(dc, &RECT { left: 0, top: y, right: gutter - v.char_w / 2, bottom: y + v.line_h }, shade);
                if prefs.line_numbers {
                    Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(muted.colorref()));
                    let mut n = wide_of(&(l + 1).to_string());
                    let mut nr = RECT { left: 0, top: y + win::px(h, 2), right: gutter - v.char_w * 3, bottom: y + v.line_h };
                    phai_ui::win::draw_text_w(dc, &mut n, &mut nr, Gdi::DT_RIGHT | Gdi::DT_SINGLELINE);
                }
            }
            if !heads.is_empty() {
                let y = heads.len() as i32 * v.line_h;
                win::fill(dc, &RECT { left: 0, top: y - 1, right: w, bottom: y }, muted.mix(bg, 0.5));
            }
            v.sticky = heads;
        }
        // The scroll bar map: where errors, warnings, changes, breakpoints and the caret are in the
        // whole file, along the right edge - a click there goes there.
        if SCROLL_MAP.load(std::sync::atomic::Ordering::Relaxed) {
            let mw = win::px(h, MAP_W);
            let strip = RECT { left: w - mw, top: 0, right: w, bottom: hgt };
            win::fill(dc, &strip, bg.mix(muted, 0.08));
            let rows = v.row_count().max(1) as i32;
            let at_line = |l: usize| -> i32 { (v.row_of(l) as i32 * hgt / rows).min(hgt - 2) };
            for (l, c) in v.changes.iter() {
                let colour = match c {
                    Change::Added => Color::rgb(0x2e, 0xa0, 0x43),
                    Change::Modified => Color::rgb(0x1c, 0x6f, 0xd1),
                    Change::Removed | Change::Deleted => Color::rgb(0xd1, 0x34, 0x38),
                };
                let y = at_line(*l);
                win::fill(dc, &RECT { left: w - mw, top: y, right: w - mw + win::px(h, 3), bottom: y + win::px(h, 3) }, colour);
            }
            for l in &v.breakpoints {
                let y = at_line(*l);
                win::fill(dc, &RECT { left: w - mw / 2 - win::px(h, 2), top: y, right: w - mw / 2 + win::px(h, 2), bottom: y + win::px(h, 4) }, Color::rgb(0xe5, 0x14, 0x00));
            }
            for m in &v.marks {
                let l = v.model.buf.position(m.from.min(v.model.buf.len())).0;
                let y = at_line(l);
                let colour = if m.error { Color::rgb(0xe5, 0x14, 0x00) } else { Color::rgb(0xbf, 0x88, 0x03) };
                win::fill(dc, &RECT { left: w - win::px(h, 4), top: y, right: w, bottom: y + win::px(h, 3) }, colour);
            }
            let y = at_line(caret_line);
            win::fill(dc, &RECT { left: w - mw, top: y, right: w, bottom: y + win::px(h, 2) }, muted);
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
    // Esc ends a snippet being filled in.
    if vk == 0x1B && v.model.snippet.take().is_some() {
        return true;
    }
    // Several carets: Esc leaves the main one, Delete works at each, a move leaves the main one.
    if !v.model.extra.is_empty() {
        match vk {
            0x1B => {
                v.model.extra.clear();
                moved(h);
                return true;
            }
            0x2E => {
                v.model.delete_all();
                changed(h);
                return true;
            }
            0x21..=0x28 => v.model.extra.clear(),
            _ => {}
        }
    }
    // Esc closes a peek.
    if vk == 0x1B && v.peeking {
        hide_tip(v);
        return true;
    }
    if vk == 0x1B && v.list.is_none() && v.find.is_none() {
        notify(h, ESCAPE);
    }
    // Ctrl+.: Quick Actions.
    if vk == 0xBE && ctrl {
        notify(h, ACTIONS);
        return true;
    }
    if vk == 0x20 && ctrl && unsafe { km::GetKeyState(0x10) } < 0 {
        notify(h, SIGNATURE);
        return true;
    }
    if vk == 0x20 && ctrl {
        notify(h, COMPLETE);
        return true;
    }
    if vk == 0x7B && !ctrl {
        notify(h, if shift { REFERENCES } else { DEFINITION });
        return true;
    }
    // Ctrl+/ (and Ctrl+K on the numeric pad's /): comment the lines, or take the comment off.
    if ctrl && (vk == 0xBF || vk == 0x6F) {
        v.model.toggle_comment("//");
        changed(h);
        return true;
    }
    if vk == 0x71 && !shift && !ctrl {
        notify(h, RENAME);
        return true;
    }
    if vk == 0x78 && !shift && !ctrl {
        toggle_line(h, v, v.model.buf.position(v.model.caret).0);
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
        let up = matches!(m, Move::Up | Move::PageUp(_));
        v.model.move_caret(m, shift);
        // Up or down into a folded block: past it, not into it.
        let (line, _) = v.model.buf.position(v.model.caret);
        if matches!(m, Move::Up | Move::Down | Move::PageUp(_) | Move::PageDown(_)) && v.is_hidden(line) {
            if let Some(&(a, z)) = v.folded.iter().find(|(a, z)| line > *a && line <= *z) {
                let to = if up { a } else { (z + 1).min(v.model.buf.line_count() - 1) };
                let to = if !up && v.is_hidden(to) { a } else { to };
                let s = v.model.buf.line(to);
                v.model.caret = v.model.buf.line_start(to) + s.len().min(v.model.caret - v.model.buf.line_start(line)).min(s.len());
                while !s.is_char_boundary(v.model.caret - v.model.buf.line_start(to)) {
                    v.model.caret -= 1;
                }
                if !shift {
                    v.model.anchor = v.model.caret;
                }
            }
        }
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
                (ID_CASE | ID_WORD | ID_REGEX, 0) => {
                    if let Some(f) = v.find.as_ref() {
                        let on = |id| unsafe { wm::GetDlgItem(Some(f.panel), id) }.map(win::is_checked).unwrap_or(false);
                        v.opts = crate::model::FindOpts { case: on(ID_CASE), word: on(ID_WORD), regex: on(ID_REGEX) };
                    }
                    v.hits_for = usize::MAX;
                    refresh_hits(v);
                    say_count(v);
                    unsafe {
                        let _ = Gdi::InvalidateRect(Some(h), None, false);
                    }
                }
                (ID_NEXT, 0) => find_step(h, v, true),
                (ID_PREV, 0) => find_step(h, v, false),
                (ID_CLOSE, 0) => close_find(h, v),
                (ID_REPLACE, 0) => {
                    let with = v.find.as_ref().map(|f| win::text_of(f.with)).unwrap_or_default();
                    let q = v.query.clone();
                    v.model.replace_one_with(&q, &with, v.opts);
                    changed(h);
                    say_count(v);
                }
                (ID_ALL, 0) => {
                    let with = v.find.as_ref().map(|f| win::text_of(f.with)).unwrap_or_default();
                    let q = v.query.clone();
                    let n = v.model.replace_all_with(&q, &with, v.opts);
                    changed(h);
                    if let Some(f) = v.find.as_ref() {
                        win::set_text(f.count, &format!("{n} ✓"));
                    }
                }
                _ => {}
            }
            LRESULT(0)
        }
        // Alt+Up and Alt+Down: the lines move, as in Visual Studio.
        // Shift+Alt+.: the next place of the chosen words gets a caret too.
        wm::WM_SYSKEYDOWN if w.0 == 0xBE && unsafe { km::GetKeyState(0x10) } < 0 => {
            if v.model.add_next_match() {
                moved(h);
            }
            LRESULT(0)
        }
        // Alt+F12: Peek Definition.
        wm::WM_SYSKEYDOWN if w.0 == 0x7B => {
            notify(h, PEEK);
            LRESULT(0)
        }
        wm::WM_SYSKEYDOWN if matches!(w.0, 0x26 | 0x28) => {
            v.list = None;
            if v.model.move_lines(w.0 == 0x26) {
                changed(h);
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
            // Several carets: what is typed goes to each.
            if !v.model.extra.is_empty() {
                match unit {
                    0x08 => v.model.backspace_all(),
                    0x0D => v.model.type_all("\n"),
                    0x09 => v.model.type_all(&crate::prefs::indent()),
                    u if u < 0x20 || ctrl && u < 0x80 => return LRESULT(0),
                    0xD800..=0xDBFF => {
                        v.high_surrogate = Some(unit);
                        return LRESULT(0);
                    }
                    0xDC00..=0xDFFF => match v.high_surrogate.take() {
                        Some(hi) => v.model.type_all(&String::from_utf16_lossy(&[hi, unit])),
                        None => return LRESULT(0),
                    },
                    u => v.model.type_all(&String::from_utf16_lossy(&[u])),
                }
                changed(h);
                return LRESULT(0);
            }
            match unit {
                0x08 => {
                    if v.lang == Language::Text || !v.model.backspace_pair() {
                        v.model.backspace();
                    }
                    changed(h);
                }
                0x09 => {
                    let shift = unsafe { km::GetKeyState(0x10) } < 0;
                    if !shift && v.model.next_snippet_field() {
                        // A snippet's next field.
                    } else if v.model.caret != v.model.anchor && v.model.selected_text().contains('\n') || shift {
                        v.model.indent_lines(shift);
                    } else if v.lang == Language::Rust && v.model.expand_snippet() {
                        // A snippet's word before the caret: its code.
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
                    let mut s = String::from_utf16_lossy(&[u]);
                    // `ui::` is `ui.`: the first `:` taken back, a `.` typed instead.
                    if s == ":" && v.lang == Language::Rust && v.model.caret == v.model.anchor && v.model.colon_after_variable() {
                        v.model.backspace();
                        s = ".".to_string();
                    }
                    let before_colon = {
                        let (line, col) = v.model.buf.position(v.model.caret);
                        v.model.buf.line(line)[..col].ends_with(':')
                    };
                    // A bracket or a quote comes with its closer, as in Visual Studio Code.
                    let paired = v.lang != Language::Text && s.chars().count() == 1 && v.model.type_paired(s.chars().next().unwrap_or(' '));
                    if !paired {
                        v.model.type_text(&s);
                    }
                    changed(h);
                    // A member or a path asks for completions; typing on narrows the open list.
                    if s == "(" || s == "," {
                        notify(h, SIGNATURE);
                    } else if s == ")" {
                        if let Some(v) = view(h) {
                            hide_tip(v);
                        }
                    }
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
            // Rest the pointer and a hover is asked; move it and any tip goes - not a peek.
            if v.tip.is_some() && !v.peeking {
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
            if !v.peeking {
                hide_tip(v);
            }
            LRESULT(0)
        }
        wm::WM_LBUTTONDOWN | wm::WM_MOUSEMOVE if msg == wm::WM_LBUTTONDOWN || v.dragging => {
            v.list = None;
            if msg == wm::WM_LBUTTONDOWN && v.peeking {
                hide_tip(v);
            }
            let (x, y) = win::point_of(l);
            if msg == wm::WM_LBUTTONDOWN {
                let mut r = RECT::default();
                unsafe {
                    let _ = wm::GetClientRect(h, &mut r);
                }
                // On the scroll bar map: that part of the file shown.
                if SCROLL_MAP.load(std::sync::atomic::Ordering::Relaxed) && x >= r.right - win::px(h, MAP_W) {
                    let rows = v.row_count();
                    let page = v.visible_lines(h);
                    let row = (y.max(0) as usize * rows) / (r.bottom.max(1) as usize);
                    v.top = row.saturating_sub(page / 2).min(rows.saturating_sub(page));
                    update_scroll(h);
                    caret_in_place(h);
                    unsafe {
                        let _ = Gdi::InvalidateRect(Some(h), None, false);
                    }
                    return LRESULT(0);
                }
                // On a sticky line: there.
                let k = (y / v.line_h.max(1)) as usize;
                if let Some(&line) = v.sticky.get(k) {
                    v.model.caret = v.model.buf.line_start(line);
                    v.model.anchor = v.model.caret;
                    v.model.extra.clear();
                    v.top = v.row_of(line);
                    moved(h);
                    return LRESULT(0);
                }
            }
            unsafe {
                let dc = Gdi::GetDC(Some(h));
                let old = Gdi::SelectObject(dc, v.font.into());
                let at = v.offset_at(dc, x, y);
                Gdi::SelectObject(dc, old);
                Gdi::ReleaseDC(Some(h), dc);
                if msg == wm::WM_LBUTTONDOWN && x < v.line_h + win::px(h, 4) {
                    // A click in the margin: a breakpoint on that line, or none - on a test's ▶, the test run.
                    let line = v.line_of(v.top + (y / v.line_h.max(1)).max(0) as usize).min(v.model.buf.line_count().saturating_sub(1));
                    if !v.breakpoints.contains(&line) {
                        if let Some((_, name)) = v.tests.iter().find(|(l, _)| *l == line) {
                            v.clicked_test = Some(name.clone());
                            notify(h, RUN_TEST);
                            return LRESULT(0);
                        }
                    }
                    toggle_line(h, v, line);
                    return LRESULT(0);
                } else if msg == wm::WM_LBUTTONDOWN && lens_at(h, v, x, y).is_some() {
                    // A click on a code lens: its references, as Shift+F12 lists them.
                    if let Some(at) = lens_at(h, v, x, y) {
                        let _ = km::SetFocus(Some(h));
                        let line = v.model.buf.line(at.0 as usize);
                        v.model.caret = v.model.buf.line_start(at.0 as usize) + crate::lsp::from_lsp(&line, at.1);
                        v.model.anchor = v.model.caret;
                        moved(h);
                        notify(h, REFERENCES);
                    }
                    return LRESULT(0);
                } else if msg == wm::WM_LBUTTONDOWN && x >= v.gutter() - v.char_w * 3 && x < v.gutter() - v.char_w / 2 {
                    // A click on a folding box: the block folds or opens.
                    let line = v.line_of(v.top + (y / v.line_h.max(1)).max(0) as usize);
                    if v.regions.iter().any(|(a, _)| *a == line) {
                        toggle_fold(h, line);
                        return LRESULT(0);
                    }
                } else if msg == wm::WM_LBUTTONDOWN && km::GetKeyState(0x11) < 0 && km::GetKeyState(0x12) < 0 {
                    // Ctrl+Alt+click: one more caret there.
                    let _ = km::SetFocus(Some(h));
                    v.model.extra.push((v.model.caret, v.model.anchor));
                    v.model.caret = at;
                    v.model.anchor = at;
                    moved(h);
                    return LRESULT(0);
                } else if msg == wm::WM_LBUTTONDOWN && km::GetKeyState(0x12) < 0 {
                    // Alt+drag: a box - a caret and selection on every line it covers.
                    let _ = km::SetFocus(Some(h));
                    km::SetCapture(h);
                    v.dragging = true;
                    v.model.extra.clear();
                    let (line, col) = v.model.buf.position(at);
                    let chars = v.model.buf.line(line)[..col].chars().count();
                    v.box_from = Some((line, chars));
                    v.model.caret = at;
                    v.model.anchor = at;
                } else if msg == wm::WM_MOUSEMOVE && v.box_from.is_some() {
                    let from = v.box_from.unwrap_or((0, 0));
                    let (line, _) = v.model.buf.position(at);
                    // The column under the pointer, even past a short line's end.
                    let col = ((x - v.gutter() + v.left) as f32 / v.char_w.max(1) as f32).round().max(0.0) as usize;
                    v.model.box_select(from, (line, col));
                } else if msg == wm::WM_LBUTTONDOWN && km::GetKeyState(0x11) < 0 {
                    // Ctrl+click: to where the thing clicked is defined, as F12.
                    let _ = km::SetFocus(Some(h));
                    v.model.caret = at;
                    v.model.anchor = at;
                    moved(h);
                    notify(h, DEFINITION);
                    return LRESULT(0);
                } else if msg == wm::WM_LBUTTONDOWN {
                    let _ = km::SetFocus(Some(h));
                    km::SetCapture(h);
                    v.dragging = true;
                    v.model.extra.clear();
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
            v.box_from = None;
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
            v.top = (v.top as i64 + lines as i64).clamp(0, v.row_count().saturating_sub(1) as i64) as usize;
            update_scroll(h);
            caret_in_place(h);
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
            caret_in_place(h);
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

#[cfg(test)]
mod wrap_tests {
    use super::wrap_points;

    #[test]
    fn long_lines_wrap_after_a_space_that_fits() {
        assert_eq!(wrap_points("short", 20, 4), vec![0]);
        let s = "let total = add(total, items[i]) + more;";
        let at = wrap_points(s, 20, 4);
        assert_eq!(at, vec![0, 12, 23]);
        assert_eq!((&s[..12], &s[12..23], &s[23..]), ("let total = ", "add(total, ", "items[i]) + more;"));
        assert_eq!(wrap_points(&"x".repeat(45), 20, 4), vec![0, 20, 40], "no space: where it runs out");
        assert_eq!(wrap_points("นนน", 2, 4), vec![0, 6]);
        // A Thai mark sits on its letter: no column of its own.
        assert_eq!(wrap_points("น้น้น้", 2, 4), vec![0, 12]);
    }
}
