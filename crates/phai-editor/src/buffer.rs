//! The text of a file as a *piece table* - Monaco's idea (its piece tree is the same pieces kept
//! in a balanced tree). The file as read is never copied or changed: an edit only appends what
//! was typed to a second buffer and rewrites a short list of pieces pointing into the two. Opening
//! a large file costs one read; typing in it costs nothing that grows with the file.
//!
//! Each piece knows where its line breaks are, so finding a line walks the pieces, not the text.
//! Offsets are bytes of UTF-8 and always fall between characters.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Source {
    Original,
    Added,
}

#[derive(Clone, Debug)]
struct Piece {
    source: Source,
    start: usize,
    len: usize,
    /// Byte offsets of each `\n` inside the piece, from its start.
    breaks: Vec<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct Buffer {
    original: String,
    added: String,
    pieces: Vec<Piece>,
    len: usize,
}

fn breaks_of(s: &str) -> Vec<usize> {
    s.bytes().enumerate().filter(|(_, b)| *b == b'\n').map(|(i, _)| i).collect()
}

impl Buffer {
    /// A buffer holding `text`; line ends are taken as `\n` (see [`normalize`]).
    pub fn new(text: &str) -> Buffer {
        let mut b = Buffer { original: text.to_string(), added: String::new(), pieces: vec![], len: text.len() };
        if !text.is_empty() {
            b.pieces.push(Piece { source: Source::Original, start: 0, len: text.len(), breaks: breaks_of(text) });
        }
        b
    }

    fn piece_text(&self, p: &Piece) -> &str {
        let s = match p.source {
            Source::Original => &self.original,
            Source::Added => &self.added,
        };
        &s[p.start..p.start + p.len]
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// All of it.
    pub fn text(&self) -> String {
        let mut s = String::with_capacity(self.len);
        for p in &self.pieces {
            s.push_str(self.piece_text(p));
        }
        s
    }

    /// The piece holding byte `offset` and where in it: `(index, inside)`. At a piece boundary it
    /// is the start of the later piece; at the end, one past the last.
    fn find(&self, offset: usize) -> (usize, usize) {
        let mut at = 0;
        for (i, p) in self.pieces.iter().enumerate() {
            if offset < at + p.len {
                return (i, offset - at);
            }
            at += p.len;
        }
        (self.pieces.len(), 0)
    }

    /// Splits the piece holding `offset` so a piece starts exactly there; returns its index.
    fn split_at(&mut self, offset: usize) -> usize {
        let (i, inside) = self.find(offset);
        if i == self.pieces.len() || inside == 0 {
            return i;
        }
        let p = self.pieces[i].clone();
        let first = Piece { source: p.source, start: p.start, len: inside, breaks: p.breaks.iter().copied().filter(|b| *b < inside).collect() };
        let second = Piece { source: p.source, start: p.start + inside, len: p.len - inside, breaks: p.breaks.iter().copied().filter(|b| *b >= inside).map(|b| b - inside).collect() };
        self.pieces[i] = first;
        self.pieces.insert(i + 1, second);
        i + 1
    }

    /// Puts `text` at byte `offset`.
    pub fn insert(&mut self, offset: usize, text: &str) {
        if text.is_empty() {
            return;
        }
        let offset = offset.min(self.len);
        let start = self.added.len();
        self.added.push_str(text);
        let i = self.split_at(offset);
        // Typing on: the piece before ends where this text begins in the added buffer - grow it.
        if i > 0 {
            let prev = &mut self.pieces[i - 1];
            if prev.source == Source::Added && prev.start + prev.len == start {
                let base = prev.len;
                prev.breaks.extend(breaks_of(text).into_iter().map(|b| b + base));
                prev.len += text.len();
                self.len += text.len();
                return;
            }
        }
        self.pieces.insert(i, Piece { source: Source::Added, start, len: text.len(), breaks: breaks_of(text) });
        self.len += text.len();
    }

    /// Removes `len` bytes from `offset`; returns what was there.
    pub fn delete(&mut self, offset: usize, len: usize) -> String {
        let offset = offset.min(self.len);
        let end = (offset + len).min(self.len);
        if end <= offset {
            return String::new();
        }
        let removed = self.slice(offset, end);
        let a = self.split_at(offset);
        let b = self.split_at(end);
        self.pieces.drain(a..b);
        self.len -= end - offset;
        removed
    }

    /// Bytes `from..to`.
    pub fn slice(&self, from: usize, to: usize) -> String {
        let mut out = String::new();
        let mut at = 0;
        for p in &self.pieces {
            let (ps, pe) = (at, at + p.len);
            if pe > from && ps < to {
                let s = self.piece_text(p);
                out.push_str(&s[from.max(ps) - ps..to.min(pe) - ps]);
            }
            at = pe;
            if at >= to {
                break;
            }
        }
        out
    }

    pub fn line_count(&self) -> usize {
        1 + self.pieces.iter().map(|p| p.breaks.len()).sum::<usize>()
    }

    /// The byte offset where line `line` (from 0) starts.
    pub fn line_start(&self, line: usize) -> usize {
        if line == 0 {
            return 0;
        }
        let mut seen = 0;
        let mut at = 0;
        for p in &self.pieces {
            if seen + p.breaks.len() >= line {
                return at + p.breaks[line - seen - 1] + 1;
            }
            seen += p.breaks.len();
            at += p.len;
        }
        self.len
    }

    /// Line `line` without its `\n`.
    pub fn line(&self, line: usize) -> String {
        let start = self.line_start(line);
        let end = if line + 1 < self.line_count() { self.line_start(line + 1) - 1 } else { self.len };
        self.slice(start, end.max(start))
    }

    /// The line holding byte `offset`, and the byte column in it.
    pub fn position(&self, offset: usize) -> (usize, usize) {
        let offset = offset.min(self.len);
        let mut line = 0;
        let mut at = 0;
        let mut line_start = 0;
        for p in &self.pieces {
            for b in &p.breaks {
                if at + b < offset {
                    line += 1;
                    line_start = at + b + 1;
                } else {
                    return (line, offset - line_start);
                }
            }
            at += p.len;
        }
        (line, offset - line_start)
    }

    /// The byte offset of `(line, column)`, the column held inside the line.
    pub fn offset(&self, line: usize, column: usize) -> usize {
        let line = line.min(self.line_count() - 1);
        let start = self.line_start(line);
        let len = self.line(line).len();
        start + column.min(len)
    }
}

/// Text with every line end made `\n`, and whether it had `\r\n` - to write it back the same way.
pub fn normalize(text: &str) -> (String, bool) {
    let crlf = text.contains("\r\n");
    (text.replace("\r\n", "\n"), crlf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edits_lines_and_positions_on_a_small_text() {
        let mut b = Buffer::new("fn main() {\n    hello();\n}\n");
        assert_eq!(b.line_count(), 4);
        assert_eq!(b.line(1), "    hello();");
        b.insert(17, "w");
        assert_eq!(b.line(1), "    hwello();");
        let gone = b.delete(17, 1);
        assert_eq!(gone, "w");
        b.insert(b.len(), "// end");
        assert_eq!(b.line(3), "// end");
        assert_eq!(b.position(b.offset(1, 4)), (1, 4));
        assert_eq!(b.offset(9, 99), b.len(), "past the end is held at the end");
    }

    #[test]
    fn thai_is_whole_characters() {
        let mut b = Buffer::new("สวัสดี\nครับ");
        assert_eq!(b.line(0), "สวัสดี");
        let at = b.offset(1, "ครับ".len());
        b.insert(at, " เจ้านาย");
        assert_eq!(b.line(1), "ครับ เจ้านาย");
    }

    /// Thousands of random edits, each checked against a plain String doing the same.
    #[test]
    fn random_edits_agree_with_a_plain_string() {
        let mut seed: u64 = 0x5eed_1234;
        let mut next = move |n: usize| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed % n.max(1) as u64) as usize
        };
        let words = ["a", "bc", "\n", "ไทย", "x\ny", "\n\n", "fn ", "ก่อน"];
        let mut b = Buffer::new("start\nof the\ntext");
        let mut s = String::from("start\nof the\ntext");
        let boundary = |s: &str, mut i: usize| {
            while !s.is_char_boundary(i) {
                i -= 1;
            }
            i
        };
        for _ in 0..4000 {
            if next(3) < 2 || s.is_empty() {
                let at = boundary(&s, next(s.len() + 1));
                let w = words[next(words.len())];
                b.insert(at, w);
                s.insert_str(at, w);
            } else {
                let at = boundary(&s, next(s.len()));
                let end = boundary(&s, (at + next(8)).min(s.len()));
                assert_eq!(b.delete(at, end - at), &s[at..end]);
                s.replace_range(at..end, "");
            }
            assert_eq!(b.len(), s.len());
        }
        assert_eq!(b.text(), s);
        let lines: Vec<&str> = s.split('\n').collect();
        assert_eq!(b.line_count(), lines.len());
        for (i, l) in lines.iter().enumerate() {
            assert_eq!(&b.line(i), l, "line {i}");
            let start = b.line_start(i);
            assert_eq!(b.position(start), (i, 0));
        }
    }

    #[test]
    fn line_ends_are_remembered() {
        assert_eq!(normalize("a\r\nb"), ("a\nb".to_string(), true));
        assert_eq!(normalize("a\nb"), ("a\nb".to_string(), false));
    }
}
