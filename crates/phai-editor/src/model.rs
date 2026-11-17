//! What editing does, apart from how it looks: a caret and a selection over a [`Buffer`], the
//! commands Monaco's keyboard gives (by character, word, line, page; with Shift to select), typing
//! that keeps the line's indent, and undo that takes back a run of typing at once.
//!
//! The caret never stops inside a character, nor between a Thai consonant and the vowel or tone
//! mark written over or under it: those marks join the character before, as a reader sees them.

use crate::buffer::Buffer;

/// Marks that sit on the character before: Thai vowels and tones above and below, combining
/// accents, variation selectors and the zero-width joiner.
pub fn joins_previous(c: char) -> bool {
    matches!(c as u32, 0x0E31 | 0x0E34..=0x0E3A | 0x0E47..=0x0E4E | 0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x20D0..=0x20FF | 0xFE00..=0xFE0F | 0x200D)
}

/// The next place the caret may stop after byte `at` in `s`.
pub fn next_stop(s: &str, at: usize) -> usize {
    let mut it = s[at..].char_indices().skip(1);
    for (i, c) in it.by_ref() {
        if !joins_previous(c) {
            return at + i;
        }
    }
    s.len()
}

/// The place before byte `at` in `s` where the caret may stop.
pub fn prev_stop(s: &str, at: usize) -> usize {
    let mut i = at;
    while i > 0 {
        i -= 1;
        while !s.is_char_boundary(i) {
            i -= 1;
        }
        let c = s[i..].chars().next().unwrap_or(' ');
        if !joins_previous(c) {
            return i;
        }
    }
    0
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[derive(Clone, Debug, PartialEq)]
struct Edit {
    at: usize,
    removed: String,
    inserted: String,
    /// Where the caret and anchor were before.
    caret_before: (usize, usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    Left,
    Right,
    WordLeft,
    WordRight,
    Up,
    Down,
    Home,
    End,
    Top,
    Bottom,
    PageUp(usize),
    PageDown(usize),
}

pub struct Model {
    pub buf: Buffer,
    /// Byte offset of the caret.
    pub caret: usize,
    /// The other end of the selection; equal to `caret` when nothing is selected.
    pub anchor: usize,
    /// The column a run of Up and Down keeps to, in characters.
    goal: Option<usize>,
    undo: Vec<Edit>,
    redo: Vec<Edit>,
    /// The undo step typing is adding to, if any.
    typing: bool,
    pub dirty: bool,
    /// More carets, each with its selection's other end: (caret, anchor). Typing goes to all.
    pub extra: Vec<(usize, usize)>,
}

impl Model {
    pub fn new(text: &str) -> Model {
        Model { buf: Buffer::new(text), caret: 0, anchor: 0, goal: None, undo: vec![], redo: vec![], typing: false, dirty: false, extra: vec![] }
    }

    pub fn selection(&self) -> (usize, usize) {
        (self.caret.min(self.anchor), self.caret.max(self.anchor))
    }

    pub fn selected_text(&self) -> String {
        let (a, b) = self.selection();
        self.buf.slice(a, b)
    }

    /// Replaces the selection with `text`; `typing` joins it to the undo step before.
    fn replace(&mut self, text: &str, typing: bool) {
        let (a, b) = self.selection();
        let before = (self.caret, self.anchor);
        let removed = self.buf.delete(a, b - a);
        self.buf.insert(a, text);
        let joined = typing && self.typing && removed.is_empty() && self.undo.last().is_some_and(|e| e.at + e.inserted.len() == a && !text.contains('\n'));
        if joined {
            self.undo.last_mut().unwrap().inserted.push_str(text);
        } else {
            self.undo.push(Edit { at: a, removed, inserted: text.to_string(), caret_before: before });
        }
        self.typing = typing;
        self.redo.clear();
        self.caret = a + text.len();
        self.anchor = self.caret;
        self.goal = None;
        self.dirty = true;
    }

    /// Typed text. A new line keeps the indent of the line it breaks; a `}` typed at the start of
    /// an indented line takes one level of indent back.
    pub fn type_text(&mut self, text: &str) {
        if text == "\n" {
            let (line, col) = self.buf.position(self.selection().0);
            let current = self.buf.line(line);
            let mut indent: String = current.chars().take_while(|c| *c == ' ' || *c == '\t').collect();
            // What is before the caret decides: a line broken before its `{` is not opening one.
            let before = current[..col].trim_end();
            if before.ends_with('{') || before.ends_with('(') || before.ends_with('[') {
                indent.push_str("    ");
            }
            if indent.len() > col {
                indent.truncate(col);
            }
            self.replace(&format!("\n{indent}"), false);
            return;
        }
        if text == "}" && self.caret == self.anchor {
            let (line, col) = self.buf.position(self.caret);
            let before = &self.buf.line(line)[..col];
            if before.len() >= 4 && before.chars().all(|c| c == ' ') {
                self.anchor = self.caret - 4;
            }
        }
        self.replace(text, true);
    }

    /// Text put in all at once - pasted, or a tab's spaces.
    pub fn insert(&mut self, text: &str) {
        self.replace(text, false);
    }

    /// Ctrl+/: the chosen lines (or the caret's) commented with `mark`, or - when every one that is
    /// not blank already is - uncommented. The comment goes at the lines' least indent, as Visual
    /// Studio puts it; one undo step, and the selection keeps to the same lines.
    pub fn toggle_comment(&mut self, mark: &str) {
        let (a, b) = self.selection();
        let first = self.buf.position(a).0;
        let (mut last, last_col) = self.buf.position(b);
        if last > first && last_col == 0 {
            last -= 1;
        }
        let lines: Vec<String> = (first..=last).map(|i| self.buf.line(i)).collect();
        let filled: Vec<&String> = lines.iter().filter(|l| !l.trim().is_empty()).collect();
        if filled.is_empty() {
            return;
        }
        let commented = filled.iter().all(|l| l.trim_start().starts_with(mark));
        let indent = filled.iter().map(|l| l.len() - l.trim_start().len()).min().unwrap_or(0);
        let start = self.buf.line_start(first);
        let end = self.buf.line_start(last) + lines.last().map(|l| l.len()).unwrap_or(0);
        let mut spans: Vec<(usize, usize, String)> = Vec::new();
        for (k, l) in lines.iter().enumerate() {
            if l.trim().is_empty() {
                continue;
            }
            let at = self.buf.line_start(first + k);
            if commented {
                let i = l.len() - l.trim_start().len();
                let gone = if l[i + mark.len()..].starts_with(' ') { mark.len() + 1 } else { mark.len() };
                spans.push((at + i, at + i + gone, String::new()));
            } else {
                spans.push((at + indent, at + indent, format!("{mark} ")));
            }
        }
        let refs: Vec<(usize, usize, &str)> = spans.iter().map(|(a, b, t)| (*a, *b, t.as_str())).collect();
        let whole = self.caret != self.anchor;
        self.edit_many(&refs);
        if whole {
            let delta: isize = spans.iter().map(|(a, b, t)| t.len() as isize - (b - a) as isize).sum();
            self.anchor = start;
            self.caret = ((end as isize) + delta).max(start as isize) as usize;
        }
    }

    /// Several changes at once - a rename - as one undo step: `(from, to, new text)` in byte
    /// offsets of the text as it is, not overlapping. The caret keeps its place among them.
    pub fn edit_many(&mut self, spans: &[(usize, usize, &str)]) {
        if spans.is_empty() {
            return;
        }
        let mut spans = spans.to_vec();
        spans.sort_by_key(|s| s.0);
        let lo = spans[0].0;
        let hi = spans.iter().map(|s| s.1).max().unwrap_or(lo).min(self.buf.len());
        let old = self.buf.slice(lo, hi);
        let mut new = String::with_capacity(old.len());
        let mut at = lo;
        let mut caret = self.caret;
        let shift = |c: usize, a: usize, b: usize, t: &str| -> usize {
            if c >= b {
                c + t.len() - (b - a)
            } else if c > a {
                a + t.len().min(c - a)
            } else {
                c
            }
        };
        for &(a, b, t) in spans.iter().rev() {
            caret = shift(caret, a, b, t);
        }
        for &(a, b, t) in &spans {
            if a < at || b > hi {
                continue;
            }
            new.push_str(&old[at - lo..a - lo]);
            new.push_str(t);
            at = b;
        }
        new.push_str(&old[at - lo..]);
        self.anchor = lo;
        self.caret = hi;
        self.replace(&new, false);
        self.caret = caret.min(self.buf.len());
        self.anchor = self.caret;
    }

    pub fn backspace(&mut self) {
        if self.caret == self.anchor {
            if self.caret == 0 {
                return;
            }
            let (line, col) = self.buf.position(self.caret);
            let s = self.buf.line(line);
            // Within leading spaces, back one indent level at a time.
            let before = &s[..col];
            if col > 0 && before.chars().all(|c| c == ' ') {
                let to = (col - 1) / 4 * 4;
                self.anchor = self.caret - (col - to);
            } else if col == 0 {
                self.anchor = self.caret - 1;
            } else {
                self.anchor = self.buf.line_start(line) + prev_stop(&s, col);
            }
        }
        self.replace("", false);
    }

    pub fn delete_forward(&mut self) {
        if self.caret == self.anchor {
            if self.caret >= self.buf.len() {
                return;
            }
            let (line, col) = self.buf.position(self.caret);
            let s = self.buf.line(line);
            self.anchor = if col >= s.len() { self.caret + 1 } else { self.buf.line_start(line) + next_stop(&s, col) };
        }
        self.replace("", false);
    }

    /// Moves the caret; with `select` the anchor stays and the selection grows.
    pub fn move_caret(&mut self, m: Move, select: bool) {
        let (line, col) = self.buf.position(self.caret);
        let s = self.buf.line(line);
        let start = self.buf.line_start(line);
        let chars_before = s[..col].chars().filter(|c| !joins_previous(*c)).count();
        let collapsed = self.caret == self.anchor;
        let to = match m {
            // Without Shift, Left and Right first close a selection on its side.
            Move::Left if !select && !collapsed => self.selection().0,
            Move::Right if !select && !collapsed => self.selection().1,
            Move::Left => {
                if col == 0 {
                    self.caret.saturating_sub(1)
                } else {
                    start + prev_stop(&s, col)
                }
            }
            Move::Right => {
                if col >= s.len() {
                    (self.caret + 1).min(self.buf.len())
                } else {
                    start + next_stop(&s, col)
                }
            }
            Move::WordLeft => {
                if col == 0 {
                    self.caret.saturating_sub(1)
                } else {
                    let chars: Vec<(usize, char)> = s[..col].char_indices().collect();
                    let mut k = chars.len();
                    while k > 0 && chars[k - 1].1.is_whitespace() {
                        k -= 1;
                    }
                    let word = k > 0 && is_word(chars[k - 1].1);
                    while k > 0 && !chars[k - 1].1.is_whitespace() && is_word(chars[k - 1].1) == word {
                        k -= 1;
                    }
                    start + chars.get(k).map(|(i, _)| *i).unwrap_or(0)
                }
            }
            Move::WordRight => {
                if col >= s.len() {
                    (self.caret + 1).min(self.buf.len())
                } else {
                    let rest: Vec<(usize, char)> = s[col..].char_indices().collect();
                    let mut k = 0;
                    let word = is_word(rest[0].1);
                    while k < rest.len() && !rest[k].1.is_whitespace() && is_word(rest[k].1) == word {
                        k += 1;
                    }
                    while k < rest.len() && rest[k].1.is_whitespace() {
                        k += 1;
                    }
                    start + col + rest.get(k).map(|(i, _)| *i).unwrap_or(s.len() - col)
                }
            }
            Move::Up | Move::Down | Move::PageUp(_) | Move::PageDown(_) => {
                let goal = *self.goal.get_or_insert(chars_before);
                let target = match m {
                    Move::Up => line.checked_sub(1),
                    Move::Down => (line + 1 < self.buf.line_count()).then_some(line + 1),
                    Move::PageUp(n) => Some(line.saturating_sub(n)),
                    Move::PageDown(n) => Some((line + n).min(self.buf.line_count() - 1)),
                    _ => None,
                };
                let to = match target {
                    None if m == Move::Up => 0,
                    None => self.buf.len(),
                    Some(t) => {
                        let ts = self.buf.line(t);
                        let mut at = 0;
                        for _ in 0..goal {
                            if at >= ts.len() {
                                break;
                            }
                            at = next_stop(&ts, at);
                        }
                        self.buf.line_start(t) + at
                    }
                };
                self.caret = to;
                if !select {
                    self.anchor = to;
                }
                self.typing = false;
                return;
            }
            Move::Home => {
                // First to the line's first word, then to its very start.
                let indent = s.len() - s.trim_start().len();
                if col == indent {
                    start
                } else {
                    start + indent
                }
            }
            Move::End => start + s.len(),
            Move::Top => 0,
            Move::Bottom => self.buf.len(),
        };
        self.caret = to;
        if !select {
            self.anchor = to;
        }
        self.goal = None;
        self.typing = false;
    }

    pub fn select_all(&mut self) {
        self.anchor = 0;
        self.caret = self.buf.len();
    }

    /// The word around byte `at` - a double-click.
    pub fn select_word(&mut self, at: usize) {
        let (line, col) = self.buf.position(at);
        let s = self.buf.line(line);
        let start = self.buf.line_start(line);
        let mut a = col;
        while a > 0 {
            let p = prev_stop(&s, a);
            if !s[p..].chars().next().is_some_and(is_word) {
                break;
            }
            a = p;
        }
        let mut b = col;
        while b < s.len() && s[b..].chars().next().is_some_and(is_word) {
            b = next_stop(&s, b);
        }
        self.anchor = start + a;
        self.caret = start + b;
    }

    pub fn undo(&mut self) {
        let Some(e) = self.undo.pop() else { return };
        self.buf.delete(e.at, e.inserted.len());
        self.buf.insert(e.at, &e.removed);
        (self.caret, self.anchor) = e.caret_before;
        self.redo.push(e);
        self.typing = false;
        self.dirty = true;
    }

    pub fn redo(&mut self) {
        let Some(e) = self.redo.pop() else { return };
        self.buf.delete(e.at, e.removed.len());
        self.buf.insert(e.at, &e.inserted);
        self.caret = e.at + e.inserted.len();
        self.anchor = self.caret;
        self.undo.push(e);
        self.typing = false;
        self.dirty = true;
    }

    /// The whole text made `new` - Format Document - as one undo step: only the part between what
    /// stays the same at the start and the end is replaced, so the caret keeps its place.
    pub fn replace_whole(&mut self, new: &str) -> bool {
        let old = self.buf.text();
        if old == new {
            return false;
        }
        let pre = old.bytes().zip(new.bytes()).take_while(|(a, b)| a == b).count();
        let pre = (0..=pre).rev().find(|i| old.is_char_boundary(*i) && new.is_char_boundary(*i)).unwrap_or(0);
        let max_suf = (old.len() - pre).min(new.len() - pre);
        let suf = old.bytes().rev().zip(new.bytes().rev()).take(max_suf).take_while(|(a, b)| a == b).count();
        let suf = (0..=suf).rev().find(|i| old.is_char_boundary(old.len() - i) && new.is_char_boundary(new.len() - i)).unwrap_or(0);
        let span = (pre, old.len() - suf, &new[pre..new.len() - suf]);
        self.edit_many(&[span]);
        true
    }

    /// Every caret with its selection, the main one first: (caret, anchor).
    pub fn carets(&self) -> Vec<(usize, usize)> {
        std::iter::once((self.caret, self.anchor)).chain(self.extra.iter().copied()).collect()
    }

    /// One change at every caret, as one undo step: `change` gives each caret's span to replace
    /// (from, to) and its text; afterwards each caret sits after its new text.
    fn at_every_caret(&mut self, change: impl Fn(&Model, usize, usize) -> (usize, usize, String)) {
        let all = self.carets();
        let mut spans: Vec<(usize, usize, String, usize)> = all.iter().enumerate().map(|(k, &(c, a))| {
            let (from, to, text) = change(self, c, a);
            (from, to, text, k)
        }).collect();
        spans.sort_by_key(|s| s.0);
        // Spans that run into each other: the later one goes.
        let mut kept: Vec<(usize, usize, String, usize)> = Vec::new();
        for s in spans {
            if kept.last().is_some_and(|l| s.0 < l.1) {
                continue;
            }
            kept.push(s);
        }
        let refs: Vec<(usize, usize, &str)> = kept.iter().map(|(a, b, t, _)| (*a, *b, t.as_str())).collect();
        self.edit_many(&refs);
        // Where each caret is now: after its text, every earlier change counted.
        let mut shift: isize = 0;
        let mut placed: Vec<(usize, usize)> = vec![(0, 0); all.len()];
        let mut got = vec![false; all.len()];
        for (a, b, t, k) in &kept {
            let at = (*a as isize + shift) as usize + t.len();
            placed[*k] = (at, at);
            got[*k] = true;
            shift += t.len() as isize - (*b - *a) as isize;
        }
        let main = if got[0] { placed[0] } else { (self.caret, self.caret) };
        self.caret = main.0.min(self.buf.len());
        self.anchor = self.caret;
        self.extra = (1..all.len()).filter(|k| got[*k]).map(|k| placed[k]).collect();
        self.typing = false;
    }

    /// Typed with several carets: the text at each.
    pub fn type_all(&mut self, text: &str) {
        let t = text.to_string();
        self.at_every_caret(|_, c, a| (c.min(a), c.max(a), t.clone()));
    }

    /// Backspace with several carets: each selection, or the character before each caret.
    pub fn backspace_all(&mut self) {
        self.at_every_caret(|m, c, a| {
            if c != a {
                return (c.min(a), c.max(a), String::new());
            }
            let (line, col) = m.buf.position(c);
            let s = m.buf.line(line);
            let from = if col > 0 { m.buf.line_start(line) + prev_stop(&s, col) } else { c.saturating_sub(1) };
            (from, c, String::new())
        });
    }

    /// Delete with several carets: each selection, or the character after each caret.
    pub fn delete_all(&mut self) {
        self.at_every_caret(|m, c, a| {
            if c != a {
                return (c.min(a), c.max(a), String::new());
            }
            let (line, col) = m.buf.position(c);
            let s = m.buf.line(line);
            let to = if col < s.len() { m.buf.line_start(line) + next_stop(&s, col) } else { (c + 1).min(m.buf.len()) };
            (c, to, String::new())
        });
    }

    /// Shift+Alt+.: the next place the selection's words are, chosen too, with a caret of its own.
    pub fn add_next_match(&mut self) -> bool {
        let (a, b) = self.selection();
        if a == b {
            return false;
        }
        let word = self.buf.slice(a, b);
        let text = self.buf.text();
        let last = self.carets().iter().map(|(c, an)| (*c).max(*an)).max().unwrap_or(b);
        let found = text[last..].find(&word).map(|i| last + i).or_else(|| text.find(&word));
        match found {
            Some(at) if !self.carets().iter().any(|(c, an)| (*c).min(*an) == at) => {
                // The newest becomes the main caret, as Visual Studio does.
                self.extra.push((self.caret, self.anchor));
                self.anchor = at;
                self.caret = at + word.len();
                true
            }
            _ => false,
        }
    }

    /// Alt+drag: a box from (line, column) to (line, column), byte columns clamped to each line -
    /// a caret and selection on every line of it, the last line's the main one.
    pub fn box_select(&mut self, from: (usize, usize), to: (usize, usize)) {
        let (l0, l1) = (from.0.min(to.0), from.0.max(to.0));
        let mut sels = Vec::new();
        for l in l0..=l1 {
            let s = self.buf.line(l);
            let clamp = |chars: usize| s.char_indices().nth(chars).map(|(i, _)| i).unwrap_or(s.len());
            let start = self.buf.line_start(l);
            sels.push((start + clamp(to.1), start + clamp(from.1)));
        }
        let main = if to.0 >= from.0 { sels.pop() } else { Some(sels.remove(0)) };
        if let Some((c, a)) = main {
            self.caret = c;
            self.anchor = a;
        }
        self.extra = sels;
    }

    /// Tab after a snippet's word: the word made its code, at the line's indent, the caret where
    /// the snippet says - one undo step. False when the word before the caret is no snippet's.
    pub fn expand_snippet(&mut self) -> bool {
        if self.caret != self.anchor {
            return false;
        }
        let (line, col) = self.buf.position(self.caret);
        let text = self.buf.line(line);
        let before = &text[..col];
        let start = before.char_indices().rev().take_while(|(_, c)| c.is_alphanumeric() || *c == '_').last().map(|(i, _)| i).unwrap_or(col);
        let word = &before[start..];
        // Only a word standing alone: `foo.fn` or `xfor` is not a snippet.
        if word.is_empty() || before[..start].chars().last().is_some_and(|c| c == '.' || c == ':' || c.is_alphanumeric()) {
            return false;
        }
        let Some((_, body)) = SNIPPETS.iter().find(|(k, _)| *k == word) else { return false };
        let indent: String = text.chars().take_while(|c| *c == ' ' || *c == '\t').collect();
        let body = body.replace('\n', &format!("\n{indent}"));
        let at = body.find("$0").unwrap_or(body.len());
        let body = body.replace("$0", "");
        let from = self.buf.line_start(line) + start;
        self.anchor = from;
        self.insert(&body);
        self.caret = from + at;
        self.anchor = self.caret;
        true
    }

    /// The lines the selection (or the caret) is on: a selection ending at a line's start leaves
    /// that line out.
    fn chosen_lines(&self) -> (usize, usize) {
        let (a, b) = self.selection();
        let first = self.buf.position(a).0;
        let (last, last_col) = self.buf.position(b);
        (first, if last > first && last_col == 0 { last - 1 } else { last })
    }

    /// Replaces `from..to` with `text` as one undo step, then puts the caret and anchor where
    /// asked; undo brings back the selection as it was.
    fn replace_range(&mut self, from: usize, to: usize, text: &str, caret: usize, anchor: usize) {
        let before = (self.caret, self.anchor);
        self.anchor = from;
        self.caret = to;
        self.replace(text, false);
        if let Some(e) = self.undo.last_mut() {
            e.caret_before = before;
        }
        self.caret = caret.min(self.buf.len());
        self.anchor = anchor.min(self.buf.len());
    }

    /// Alt+Up and Alt+Down: the chosen lines swap places with the line above or below; the
    /// selection goes with them. False at the top or bottom, where nothing moves.
    pub fn move_lines(&mut self, up: bool) -> bool {
        let (first, last) = self.chosen_lines();
        let count = self.buf.line_count();
        if (up && first == 0) || (!up && last + 1 >= count) {
            return false;
        }
        let block_start = self.buf.line_start(first);
        let block_end = self.buf.line_start(last) + self.buf.line(last).len();
        let block = self.buf.slice(block_start, block_end);
        let (caret, anchor) = (self.caret, self.anchor);
        if up {
            let other = self.buf.line(first - 1);
            let from = self.buf.line_start(first - 1);
            let shift = other.len() + 1;
            self.replace_range(from, block_end, &format!("{block}
{other}"), caret - shift, anchor - shift);
        } else {
            let other = self.buf.line(last + 1);
            let to = self.buf.line_start(last + 1) + other.len();
            let shift = other.len() + 1;
            self.replace_range(block_start, to, &format!("{other}
{block}"), caret + shift, anchor + shift);
        }
        true
    }

    /// Ctrl+D: the chosen lines again below them - or, when a part of one line is chosen, that
    /// part again after itself - and the copy chosen, as Visual Studio does.
    pub fn duplicate(&mut self) {
        let (a, b) = self.selection();
        let (first, last) = self.chosen_lines();
        if a != b && self.buf.position(a).0 == self.buf.position(b).0 {
            let part = self.buf.slice(a, b);
            self.replace_range(b, b, &part, b + part.len(), b);
            return;
        }
        let end = self.buf.line_start(last) + self.buf.line(last).len();
        let block = self.buf.slice(self.buf.line_start(first), end);
        let shift = block.len() + 1;
        let (caret, anchor) = (self.caret, self.anchor);
        self.replace_range(end, end, &format!("
{block}"), caret + shift, anchor + shift);
    }

    /// Indents the selected lines one level (`out` false) or takes one level off them.
    pub fn indent_lines(&mut self, out: bool) {
        let (a, b) = self.selection();
        let (first, _) = self.buf.position(a);
        let (last, last_col) = self.buf.position(b);
        let last = if last > first && last_col == 0 { last - 1 } else { last };
        let start = self.buf.line_start(first);
        let end = self.buf.line_start(last) + self.buf.line(last).len();
        let text = self.buf.slice(start, end);
        let changed: Vec<String> = text
            .split('\n')
            .map(|l| {
                if out {
                    let n = l.chars().take(4).take_while(|c| *c == ' ').count();
                    l[n..].to_string()
                } else if l.is_empty() {
                    String::new()
                } else {
                    format!("    {l}")
                }
            })
            .collect();
        let joined = changed.join("\n");
        self.anchor = start;
        self.caret = end;
        self.replace(&joined, false);
        self.anchor = start;
        self.caret = start + joined.len();
    }
}

/// The bracket beside the caret at `at` (after it first, then before it) and the one that
/// matches it, as byte offsets - brackets in strings, character literals and comments left out.
pub fn matching_brace(text: &str, at: usize) -> Option<(usize, usize)> {
    let b = text.as_bytes();
    let code = code_brackets(text);
    matching_in(b, &code, at)
}

/// The places that can fold: each `{` whose `}` is on a later line, as (its line, the `}`'s
/// line), by first line; of several on one line, the one reaching furthest.
pub fn fold_regions(text: &str) -> Vec<(usize, usize)> {
    let b = text.as_bytes();
    let starts: Vec<usize> = std::iter::once(0).chain(text.match_indices('\n').map(|(i, _)| i + 1)).collect();
    let line_of = |at: usize| starts.partition_point(|s| *s <= at) - 1;
    let mut open: Vec<usize> = Vec::new();
    let mut out: std::collections::BTreeMap<usize, usize> = std::collections::BTreeMap::new();
    for p in code_brackets(text) {
        match b[p] {
            b'{' => open.push(p),
            b'}' => {
                if let Some(o) = open.pop() {
                    let (a, z) = (line_of(o), line_of(p));
                    if z > a {
                        let e = out.entry(a).or_insert(z);
                        *e = (*e).max(z);
                    }
                }
            }
            _ => {}
        }
    }
    out.into_iter().collect()
}

/// The offsets of the brackets that are code - not in strings, characters or comments - in order.
fn code_brackets(text: &str) -> Vec<usize> {
    let b = text.as_bytes();
    let mut code: Vec<usize> = Vec::new();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'/' if b.get(i + 1) == Some(&b'/') => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                i += 2;
                while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                i += 2;
                continue;
            }
            b'"' => {
                i += 1;
                while i < b.len() && b[i] != b'"' {
                    i += if b[i] == b'\\' { 2 } else { 1 };
                }
            }
            // 'x' and '\n' are characters; 'a in a lifetime is not.
            b'\'' if b.get(i + 2) == Some(&b'\'') && b.get(i + 1) != Some(&b'\\') => i += 2,
            b'\'' if b.get(i + 1) == Some(&b'\\') => {
                i += 2;
                while i < b.len() && b[i] != b'\'' {
                    i += 1;
                }
            }
            b'(' | b')' | b'[' | b']' | b'{' | b'}' => code.push(i),
            _ => {}
        }
        i += 1;
    }
    code
}

fn matching_in(b: &[u8], code: &[usize], at: usize) -> Option<(usize, usize)> {
    let open = |c: u8| matches!(c, b'(' | b'[' | b'{');
    let pair = |c: u8| match c {
        b'(' => b')',
        b'[' => b']',
        b'{' => b'}',
        b')' => b'(',
        b']' => b'[',
        _ => b'{',
    };
    let k = code.iter().position(|&p| p == at).or_else(|| code.iter().position(|&p| p + 1 == at))?;
    let c = b[code[k]];
    let mut depth = 0i32;
    if open(c) {
        for &p in &code[k + 1..] {
            if b[p] == c {
                depth += 1;
            } else if b[p] == pair(c) {
                if depth == 0 {
                    return Some((code[k], p));
                }
                depth -= 1;
            }
        }
    } else {
        for &p in code[..k].iter().rev() {
            if b[p] == c {
                depth += 1;
            } else if b[p] == pair(c) {
                if depth == 0 {
                    return Some((p, code[k]));
                }
                depth -= 1;
            }
        }
    }
    None
}

/// Rust code snippets, as Visual Studio's: a short word, then Tab, and the code is there - `$0`
/// where the caret goes.
pub const SNIPPETS: &[(&str, &str)] = &[
    ("fn", "fn $0() {\n}"),
    ("pfn", "pub fn $0() {\n}"),
    ("if", "if $0 {\n}"),
    ("ifl", "if let Some($0) = {\n}"),
    ("ife", "if $0 {\n} else {\n}"),
    ("for", "for $0 in  {\n}"),
    ("while", "while $0 {\n}"),
    ("loop", "loop {\n    $0\n}"),
    ("match", "match $0 {\n    _ => {}\n}"),
    ("struct", "struct $0 {\n}"),
    ("enum", "enum $0 {\n}"),
    ("impl", "impl $0 {\n}"),
    ("test", "#[test]\nfn $0() {\n}"),
    ("tests", "#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn $0() {\n    }\n}"),
    ("main", "fn main() {\n    $0\n}"),
    ("pln", "println!(\"{}\", $0);"),
    ("ok", "Ok($0)"),
    ("derive", "#[derive(Debug, Clone$0)]"),
];

/// The find bar's choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FindOpts {
    /// Upper and lower case kept apart.
    pub case: bool,
    /// Only where the words stand alone - no letter, digit or _ either side.
    pub word: bool,
    /// The query is a regular expression.
    pub regex: bool,
}

fn regex_of(query: &str, o: FindOpts) -> Option<regex::Regex> {
    let pattern = if o.regex { query.to_string() } else { regex::escape(query) };
    let pattern = if o.word { format!(r"\b(?:{pattern})\b") } else { pattern };
    regex::RegexBuilder::new(&pattern).case_insensitive(!o.case).build().ok()
}

/// Every match of `query` in `text` with these choices; a regular expression that does not read
/// matches nothing.
pub fn find_matches(text: &str, query: &str, o: FindOpts) -> Vec<(usize, usize)> {
    if query.is_empty() {
        return vec![];
    }
    if !o.regex && !o.word {
        return find_all(text, query, o.case);
    }
    match regex_of(query, o) {
        Some(re) => re.find_iter(text).filter(|m| m.end() > m.start()).map(|m| (m.start(), m.end())).collect(),
        None => vec![],
    }
}

/// Every place `query` is in `text`, as byte ranges; `case` false finds it in any case.
pub fn find_all(text: &str, query: &str, case: bool) -> Vec<(usize, usize)> {
    if query.is_empty() {
        return vec![];
    }
    if !case {
        // Lower case the same length as the text (ASCII, Thai, most): offsets carry over, and the
        // plain search is quick.
        let (lt, lq) = (text.to_lowercase(), query.to_lowercase());
        if lt.len() == text.len() && lt.char_indices().map(|(i, _)| i).eq(text.char_indices().map(|(i, _)| i)) {
            return find_all(&lt, &lq, true).into_iter().filter(|(a, b)| text.is_char_boundary(*a) && text.is_char_boundary(*b)).collect();
        }
    }
    let mut out = Vec::new();
    if case {
        let mut from = 0;
        while let Some(i) = text[from..].find(query) {
            out.push((from + i, from + i + query.len()));
            from += i + query.len().max(1);
        }
    } else {
        // Compare character by character, so offsets stay those of the text itself.
        let q: Vec<char> = query.chars().flat_map(char::to_lowercase).collect();
        let chars: Vec<(usize, char)> = text.char_indices().collect();
        let mut k = 0;
        while k < chars.len() {
            let mut j = k;
            let mut m = 0;
            let mut low = Vec::new();
            while j < chars.len() && low.len() < q.len() {
                low.extend(chars[j].1.to_lowercase());
                j += 1;
            }
            while m < q.len() && m < low.len() && low[m] == q[m] {
                m += 1;
            }
            if m == q.len() && low.len() == q.len() {
                let end = chars.get(j).map(|c| c.0).unwrap_or(text.len());
                out.push((chars[k].0, end));
                k = j;
            } else {
                k += 1;
            }
        }
    }
    out
}

impl Model {
    /// Chooses the next place `query` is after the caret (or before it, `forward` false),
    /// going round the end. False when it is nowhere.
    pub fn find(&mut self, query: &str, case: bool, forward: bool) -> bool {
        self.find_with(query, FindOpts { case, ..Default::default() }, forward)
    }

    /// Find with the find bar's choices: case, whole word, regular expression.
    pub fn find_with(&mut self, query: &str, o: FindOpts, forward: bool) -> bool {
        let text = self.buf.text();
        let hits = find_matches(&text, query, o);
        if hits.is_empty() {
            return false;
        }
        let (a, b) = self.selection();
        let hit = if forward { hits.iter().find(|h| h.0 >= b).or(hits.first()) } else { hits.iter().rev().find(|h| h.1 <= a).or(hits.last()) };
        let &(s, e) = hit.unwrap();
        self.anchor = s;
        self.caret = e;
        true
    }

    /// Replaces the chosen text if it is `query`, then finds the next.
    pub fn replace_one(&mut self, query: &str, with: &str, case: bool) -> bool {
        self.replace_one_with(query, with, FindOpts { case, ..Default::default() })
    }

    /// Replace with the find bar's choices; a regular expression's `$1` takes what its group found.
    pub fn replace_one_with(&mut self, query: &str, with: &str, o: FindOpts) -> bool {
        let (a, b) = self.selection();
        let text = self.buf.text();
        // The chosen words are one of the matches: replaced.
        if a != b && find_matches(&text, query, o).contains(&(a, b)) {
            let new = if o.regex { regex_of(query, o).map(|re| re.replace(&text[a..b], with).to_string()).unwrap_or_else(|| with.to_string()) } else { with.to_string() };
            self.insert(&new);
        }
        self.find_with(query, o, true)
    }

    /// Replaces every `query` with `with` in one undo step; returns how many.
    pub fn replace_all(&mut self, query: &str, with: &str, case: bool) -> usize {
        self.replace_all_with(query, with, FindOpts { case, ..Default::default() })
    }

    pub fn replace_all_with(&mut self, query: &str, with: &str, o: FindOpts) -> usize {
        let text = self.buf.text();
        let hits = find_matches(&text, query, o);
        if hits.is_empty() {
            return 0;
        }
        let re = if o.regex { regex_of(query, o) } else { None };
        let mut out = String::with_capacity(text.len());
        let mut at = 0;
        for (s, e) in &hits {
            out.push_str(&text[at..*s]);
            match &re {
                Some(re) => out.push_str(&re.replace(&text[*s..*e], with)),
                None => out.push_str(with),
            }
            at = *e;
        }
        out.push_str(&text[at..]);
        self.anchor = 0;
        self.caret = self.buf.len();
        self.insert(&out);
        self.caret = 0;
        self.anchor = 0;
        hits.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at_end(text: &str) -> Model {
        let mut m = Model::new(text);
        m.caret = text.len();
        m.anchor = m.caret;
        m
    }

    #[test]
    fn several_carets_type_together() {
        let mut m = Model::new("let a = 1;\nlet b = 2;\nlet c = 3;");
        // A box from "a" to "c": one caret on each line, before the letter.
        m.box_select((0, 4), (2, 4));
        assert_eq!(m.carets().len(), 3);
        m.type_all("my_");
        assert_eq!(m.buf.text(), "let my_a = 1;\nlet my_b = 2;\nlet my_c = 3;");
        m.backspace_all();
        assert_eq!(m.buf.text(), "let mya = 1;\nlet myb = 2;\nlet myc = 3;");
        m.undo();
        assert_eq!(m.buf.text(), "let my_a = 1;\nlet my_b = 2;\nlet my_c = 3;", "one undo step for all");
        // Shift+Alt+.: the next "let" chosen too, then typed over.
        let mut m = Model::new("let x;\nlet y;");
        m.anchor = 0;
        m.caret = 3;
        assert!(m.add_next_match());
        m.type_all("var");
        assert_eq!(m.buf.text(), "var x;\nvar y;");
        m.delete_all();
        assert_eq!(m.buf.text(), "varx;\nvary;");
    }

    #[test]
    fn blocks_over_several_lines_can_fold() {
        let t = "fn a() {\n    if x { y }\n    if z {\n        w\n    }\n}\nfn b() {}\n";
        assert_eq!(fold_regions(t), vec![(0, 5), (2, 4)], "a one-line block does not fold");
    }

    #[test]
    fn brackets_match_past_strings_and_comments() {
        let t = "fn f(a: [u8; 2]) { let s = \"(}\"; // )\n g('(', a[0]) }";
        let open = t.find('{').unwrap();
        assert_eq!(matching_brace(t, open), Some((open, t.len() - 1)), "the string's and the comment's brackets are passed over");
        assert_eq!(matching_brace(t, t.len()), Some((open, t.len() - 1)), "the caret just after a bracket");
        let call = t.find("g(").unwrap() + 1;
        assert_eq!(matching_brace(t, call).map(|(_, b)| &t[b..]), Some(") }"), "'(' is a character");
        assert_eq!(matching_brace(t, 1), None, "no bracket beside the caret");
        assert_eq!(matching_brace("a[0", 1), None, "none to match");
    }

    #[test]
    fn a_whole_new_text_is_one_undo_step_and_keeps_the_caret() {
        let mut m = Model::new("fn main(){\nlet  x=1;\n}");
        m.caret = 21; // before the last brace
        m.anchor = 21;
        assert!(m.replace_whole("fn main() {\n    let x = 1;\n}"));
        assert_eq!(m.buf.text(), "fn main() {\n    let x = 1;\n}");
        assert_eq!(&m.buf.text()[m.caret..], "}", "still before the brace");
        assert!(!m.replace_whole("fn main() {\n    let x = 1;\n}"), "nothing to change");
        m.undo();
        assert_eq!(m.buf.text(), "fn main(){\nlet  x=1;\n}");
    }

    #[test]
    fn alt_arrows_move_lines_and_ctrl_d_doubles_them() {
        let mut m = Model::new("a
bb
ccc");
        m.caret = 3; // in "bb"
        m.anchor = 3;
        assert!(m.move_lines(true));
        assert_eq!(m.buf.text(), "bb
a
ccc");
        assert_eq!(m.caret, 1, "the caret went up with its line");
        assert!(!m.move_lines(true), "the top line goes no higher");
        assert!(m.move_lines(false) && m.move_lines(false));
        assert_eq!(m.buf.text(), "a
ccc
bb");
        assert!(!m.move_lines(false));
        m.undo();
        assert_eq!(m.buf.text(), "a
bb
ccc");
        assert_eq!(m.caret, 3, "undo puts the caret back where it was before that move");
        // Two lines chosen move together.
        let mut m = Model::new("1
2
3
4");
        m.anchor = 2;
        m.caret = 6; // the start of "4": that line is not chosen
        assert!(m.move_lines(true));
        assert_eq!(m.buf.text(), "2
3
1
4");
        assert_eq!(m.selected_text(), "2
3
");
        // Ctrl+D: a line, then a part of one.
        let mut m = Model::new("let x = 1;
end");
        m.caret = 4;
        m.anchor = 4;
        m.duplicate();
        assert_eq!(m.buf.text(), "let x = 1;
let x = 1;
end");
        assert_eq!(m.caret, 15, "the caret is on the copy");
        m.anchor = 15;
        m.caret = 16;
        m.duplicate();
        assert_eq!(m.buf.text(), "let x = 1;
let xx = 1;
end");
        assert_eq!(m.selected_text(), "x");
    }

    #[test]
    fn the_caret_steps_over_thai_marks_as_one_character() {
        let s = "สวัสดี";
        let stops: Vec<usize> = std::iter::successors(Some(0), |&i| (i < s.len()).then(|| next_stop(s, i))).collect();
        // ส ว ั ส ด ี  -> stops before ส, ว, ส, ด and at the end: ั and ี join the letter before.
        assert_eq!(stops.len(), 5, "{stops:?}");
        assert_eq!(prev_stop(s, s.len()), "สวัส".len());
        let mut m = at_end("กี่");
        m.backspace();
        assert_eq!(m.buf.text(), "", "ก with its vowel and tone goes as one");
    }

    #[test]
    fn typing_keeps_the_indent_and_one_undo_takes_back_a_word() {
        let mut m = at_end("fn main() {");
        m.type_text("\n");
        assert_eq!(m.buf.text(), "fn main() {\n    ");
        for c in "hello".chars() {
            m.type_text(&c.to_string());
        }
        m.type_text("\n");
        m.type_text("}");
        assert_eq!(m.buf.text(), "fn main() {\n    hello\n}");
        m.undo();
        m.undo();
        assert_eq!(m.buf.text(), "fn main() {\n    hello", "the closing brace and the new line came back separately");
        m.undo();
        assert_eq!(m.buf.text(), "fn main() {\n    ", "hello went at once");
        m.redo();
        assert_eq!(m.buf.text(), "fn main() {\n    hello");
    }

    #[test]
    fn a_new_line_in_front_of_code_does_not_indent_it() {
        let mut m = Model::new("fn main() {");
        m.type_text("
");
        assert_eq!(m.buf.text(), "
fn main() {", "broken at the start, the brace was not before the caret");
        let mut m = Model::new("    let x = 1;");
        m.caret = 2;
        m.anchor = 2;
        m.type_text("
");
        assert_eq!(m.buf.text(), "  
    let x = 1;", "broken inside the indent, the code keeps its whole indent");
    }

    #[test]
    fn moves_by_word_line_and_page_keep_their_column() {
        let mut m = Model::new("let value = 42;\nx\nlet other = 7;");
        m.move_caret(Move::WordRight, false);
        assert_eq!(m.caret, 4);
        m.move_caret(Move::WordRight, true);
        assert_eq!(m.selected_text(), "value ");
        m.move_caret(Move::End, false);
        m.move_caret(Move::Down, false);
        assert_eq!(m.buf.position(m.caret), (1, 1), "a short line holds the caret at its end");
        m.move_caret(Move::Down, false);
        assert_eq!(m.buf.position(m.caret), (2, 14), "and the next long one gives the column back");
        m.move_caret(Move::Home, false);
        assert_eq!(m.buf.position(m.caret), (2, 0));
    }

    #[test]
    fn backspace_takes_indent_a_level_at_a_time_and_selections_go_whole() {
        let mut m = at_end("        ");
        m.backspace();
        assert_eq!(m.buf.text(), "    ");
        let mut m = Model::new("abc def");
        m.anchor = 1;
        m.caret = 5;
        m.backspace();
        assert_eq!(m.buf.text(), "aef");
    }

    #[test]
    fn lines_indent_and_unindent_together() {
        let mut m = Model::new("a\nb\nc");
        m.select_all();
        m.indent_lines(false);
        assert_eq!(m.buf.text(), "    a\n    b\n    c");
        m.indent_lines(true);
        assert_eq!(m.buf.text(), "a\nb\nc");
    }

    #[test]
    fn find_goes_round_and_ignores_case_when_asked() {
        assert_eq!(find_all("Ab ab AB", "ab", false).len(), 3);
        assert_eq!(find_all("Ab ab AB", "ab", true), vec![(3, 5)]);
        assert_eq!(find_all("ไทย ไทย", "ไทย", true).len(), 2);
        let mut m = Model::new("one two one");
        assert!(m.find("one", true, true));
        assert_eq!(m.selection(), (0, 3));
        assert!(m.find("one", true, true));
        assert_eq!(m.selection(), (8, 11));
        assert!(m.find("one", true, true), "and round again");
        assert_eq!(m.selection(), (0, 3));
        assert!(!m.find("three", true, true));
    }

    #[test]
    fn a_snippet_word_and_tab_make_its_code() {
        let mut m = Model::new("fn main() {\n    for\n}");
        m.caret = "fn main() {\n    for".len();
        m.anchor = m.caret;
        assert!(m.expand_snippet());
        assert_eq!(m.buf.text(), "fn main() {\n    for  in  {\n    }\n}", "at the line's indent");
        assert_eq!(&m.buf.text()[..m.caret], "fn main() {\n    for ", "the caret where the snippet says");
        m.undo();
        assert_eq!(m.buf.text(), "fn main() {\n    for\n}", "one undo step");
        let mut m = Model::new("x.fn");
        m.caret = 4;
        m.anchor = 4;
        assert!(!m.expand_snippet(), "a method name is not a snippet");
    }

    #[test]
    fn whole_words_and_regular_expressions() {
        let o = |case, word, regex| FindOpts { case, word, regex };
        assert_eq!(find_matches("sum summary sum_x Sum", "sum", o(false, true, false)), vec![(0, 3), (18, 21)], "alone only, any case");
        assert_eq!(find_matches("a1 b22 c333", r"[a-z]\d{2,}", o(true, false, true)), vec![(3, 6), (7, 11)]);
        assert!(find_matches("x", "(", o(false, false, true)).is_empty(), "a broken expression finds nothing");
        let mut m = Model::new("let a1 = b2;");
        assert_eq!(m.replace_all_with(r"([a-z])(\d)", "$2$1", o(true, false, true)), 2);
        assert_eq!(m.buf.text(), "let 1a = 2b;", "groups taken into the replacement");
    }

    #[test]
    fn replace_one_and_all_with_one_undo() {
        let mut m = Model::new("let a = a + a;");
        assert_eq!(m.replace_all("a", "b", true), 3);
        assert_eq!(m.buf.text(), "let b = b + b;");
        m.undo();
        assert_eq!(m.buf.text(), "let a = a + a;", "all of it back at once");
        let mut m = Model::new("x x");
        m.find("x", true, true);
        m.replace_one("x", "yy", true);
        assert_eq!(m.buf.text(), "yy x");
        assert_eq!(m.selected_text(), "x", "and the next one chosen");
    }

    #[test]
    fn double_click_takes_the_word() {
        let mut m = Model::new("call some_fn(now)");
        m.select_word(7);
        assert_eq!(m.selected_text(), "some_fn");
    }

    #[test]
    fn many_edits_are_one_undo_step_and_the_caret_keeps_its_place() {
        let mut m = Model::new("go(); go(); go();");
        m.caret = 7; // in the second `go`
        m.anchor = 7;
        m.edit_many(&[(12, 14, "run"), (0, 2, "run"), (6, 8, "run")]);
        assert_eq!(m.buf.text(), "run(); run(); run();");
        assert_eq!(m.caret, 8);
        m.undo();
        assert_eq!(m.buf.text(), "go(); go(); go();");
    }

    #[test]
    fn ctrl_slash_comments_and_uncomments_lines() {
        let mut m = Model::new("fn a() {\n    let x = 1;\n\n    go();\n}\n");
        m.anchor = m.buf.line_start(1);
        m.caret = m.buf.line_start(4);
        m.toggle_comment("//");
        assert_eq!(m.buf.text(), "fn a() {\n    // let x = 1;\n\n    // go();\n}\n");
        assert_eq!(m.selected_text(), "    // let x = 1;\n\n    // go();");
        m.toggle_comment("//");
        assert_eq!(m.buf.text(), "fn a() {\n    let x = 1;\n\n    go();\n}\n");
        // One line, the caret anywhere in it; one undo step back.
        let mut m = Model::new("x();\n");
        m.caret = 2;
        m.anchor = 2;
        m.toggle_comment("//");
        assert_eq!(m.buf.text(), "// x();\n");
        m.undo();
        assert_eq!(m.buf.text(), "x();\n");
    }
}
