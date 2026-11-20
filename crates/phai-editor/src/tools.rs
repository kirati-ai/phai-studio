//! Editing tools of Visual Studio's Edit menu, as plain text work: Expand Selection (each time a
//! little more - the word, what is between the brackets around it, the brackets too, then the
//! next pair out) and Surround With (lines put in an `if`, a `for`, a block...).

/// The pairs Expand Selection grows to.
const PAIRS: [(u8, u8); 4] = [(b'(', b')'), (b'[', b']'), (b'{', b'}'), (b'"', b'"')];

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The selection `a..b` (byte offsets) grown one step in `text`.
pub fn expand(text: &str, a: usize, b: usize) -> (usize, usize) {
    let (a, b) = (a.min(text.len()), b.min(text.len()));
    // Nothing chosen: the word at the caret, when there is one.
    if a == b {
        let start = text[..a].char_indices().rev().take_while(|(_, c)| is_word(*c)).last().map(|(i, _)| i).unwrap_or(a);
        let end = text[a..].char_indices().find(|(_, c)| !is_word(*c)).map(|(i, _)| a + i).unwrap_or(text.len());
        if end > start {
            return (start, end);
        }
    }
    // The smallest pair around the selection: its inside, unless that is chosen already - then
    // the pair with it.
    let bytes = text.as_bytes();
    let mut best: Option<(usize, usize)> = None;
    for (open, close) in PAIRS {
        // Outward from the selection: the nearest unmatched open before it...
        let mut depth = 0i32;
        let mut i = a;
        let mut start = None;
        while i > 0 {
            i -= 1;
            let c = bytes[i];
            if open == close {
                if c == open {
                    start = Some(i);
                    break;
                }
                continue;
            }
            if c == close {
                depth += 1;
            } else if c == open {
                if depth == 0 {
                    start = Some(i);
                    break;
                }
                depth -= 1;
            }
        }
        let Some(s) = start else { continue };
        // ...and its close after it.
        let mut depth = 0i32;
        let mut end = None;
        for (j, &c) in bytes.iter().enumerate().skip(b) {
            if open == close {
                if c == close {
                    end = Some(j);
                    break;
                }
                continue;
            }
            if c == open {
                depth += 1;
            } else if c == close {
                if depth == 0 {
                    end = Some(j);
                    break;
                }
                depth -= 1;
            }
        }
        let Some(e) = end else { continue };
        let inside = (s + 1, e);
        let pick = if (a, b) == inside || (a <= inside.0 && b >= inside.1) { (s, e + 1) } else { inside };
        if pick.0 <= a && pick.1 >= b && (pick.0, pick.1) != (a, b) && best.is_none_or(|(x, y)| y - x > pick.1 - pick.0) {
            best = Some(pick);
        }
    }
    if let Some(p) = best {
        return p;
    }
    // No pair: the whole lines, then everything.
    let line_start = text[..a].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = text[b..].find('\n').map(|i| b + i).unwrap_or(text.len());
    if (line_start, line_end) != (a, b) {
        return (line_start, line_end);
    }
    (0, text.len())
}

/// What Surround With can put lines in: its name, the line before them (`{0}` the placeholder
/// to type over), and whether `}` closes it.
pub const SURROUNDS: [(&str, &str); 7] = [
    ("if", "if condition {"),
    ("if let", "if let Some(value) = option {"),
    ("for", "for item in items {"),
    ("while", "while condition {"),
    ("loop", "loop {"),
    ("unsafe", "unsafe {"),
    ("{ }", "{"),
];

/// `lines` (whole lines, no last line end) put in `head`'s block: each indented one more `unit`,
/// the head and the closing brace at the first line's indent.
pub fn surround(lines: &str, head: &str, unit: &str) -> String {
    let indent: String = lines.lines().find(|l| !l.trim().is_empty()).map(|l| l.chars().take_while(|c| c.is_whitespace()).collect()).unwrap_or_default();
    let mut out = format!("{indent}{head}\n");
    for l in lines.lines() {
        if l.trim().is_empty() {
            out.push('\n');
        } else {
            out.push_str(unit);
            out.push_str(l);
            out.push('\n');
        }
    }
    out.push_str(&indent);
    out.push('}');
    out
}

/// The brackets of a line - `(`, `[`, `{` and their closers, not in strings, characters or the
/// comment at its end - each with how deep it is, from `depth` at the line's start; and the depth
/// at its end. A closer is as deep as its opener: the pair the same colour.
pub fn brackets(line: &str, depth: i32) -> (Vec<(usize, i32)>, i32) {
    let mut out = Vec::new();
    let mut d = depth;
    let mut chars = line.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match c {
            '/' if chars.peek().is_some_and(|(_, n)| *n == '/') => break,
            '"' => {
                while let Some((_, n)) = chars.next() {
                    if n == '\\' {
                        chars.next();
                    } else if n == '"' {
                        break;
                    }
                }
            }
            '\'' => {
                // A character ('{', '\''), not a lifetime ('a).
                let rest: Vec<char> = line[i + 1..].chars().take(3).collect();
                if rest.first() == Some(&'\\') || rest.get(1) == Some(&'\'') {
                    for (_, n) in chars.by_ref() {
                        if n == '\'' {
                            break;
                        }
                    }
                }
            }
            '(' | '[' | '{' => {
                out.push((i, d));
                d += 1;
            }
            ')' | ']' | '}' => {
                d = (d - 1).max(0);
                out.push((i, d));
            }
            _ => {}
        }
    }
    (out, d)
}

/// Compare Files: which lines of `a` are not in `b` (each marked [`Change::Deleted`]) and which of
/// `b` are new or changed - by their longest common run of lines. Files too big to compare line
/// by line (more than 4 million pairs) are marked from where they first differ.
/// Lines (from 0) and how each changed.
pub type Changes = Vec<(usize, crate::view::Change)>;

pub fn diff_lines(a: &str, b: &str) -> (Changes, Changes) {
    use crate::view::Change;
    let (la, lb): (Vec<&str>, Vec<&str>) = (a.lines().collect(), b.lines().collect());
    let (n, m) = (la.len(), lb.len());
    let mut left = Vec::new();
    let mut right = Vec::new();
    if n * m > 4_000_000 {
        let same = la.iter().zip(&lb).take_while(|(x, y)| x == y).count();
        left.extend((same..n).map(|i| (i, Change::Deleted)));
        right.extend((same..m).map(|i| (i, Change::Added)));
        return (left, right);
    }
    // lcs[i][j]: the longest common run of a[i..] and b[j..].
    let mut lcs = vec![0u32; (n + 1) * (m + 1)];
    let at = |i: usize, j: usize| i * (m + 1) + j;
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[at(i, j)] = if la[i] == lb[j] { lcs[at(i + 1, j + 1)] + 1 } else { lcs[at(i + 1, j)].max(lcs[at(i, j + 1)]) };
        }
    }
    let (mut i, mut j) = (0, 0);
    while i < n || j < m {
        if i < n && j < m && la[i] == lb[j] {
            i += 1;
            j += 1;
        } else if j < m && (i == n || lcs[at(i, j + 1)] >= lcs[at(i + 1, j)]) {
            // A line of b's own: new - or changed, when a line of a's own stands across from it.
            let changed = i < n && lcs[at(i + 1, j)] == lcs[at(i, j + 1)];
            right.push((j, if changed { Change::Modified } else { Change::Added }));
            j += 1;
        } else {
            left.push((i, Change::Deleted));
            i += 1;
        }
    }
    (left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_selection_grows_word_inside_pair_and_out() {
        let t = "let x = add(total, items[i + 1]);";
        let at = t.find("i +").unwrap();
        let w = expand(t, at, at);
        assert_eq!(&t[w.0..w.1], "i");
        let s = expand(t, w.0, w.1);
        assert_eq!(&t[s.0..s.1], "i + 1");
        let s = expand(t, s.0, s.1);
        assert_eq!(&t[s.0..s.1], "[i + 1]");
        let s = expand(t, s.0, s.1);
        assert_eq!(&t[s.0..s.1], "total, items[i + 1]");
        let s = expand(t, s.0, s.1);
        assert_eq!(&t[s.0..s.1], "(total, items[i + 1])");
        let s = expand(t, s.0, s.1);
        assert_eq!(&t[s.0..s.1], t, "the line");
    }

    #[test]
    fn brackets_coloured_by_depth_strings_passed_over() {
        let (b, end) = brackets("f(a[1], \"(\", '{') { // (", 0);
        assert_eq!(b, vec![(1, 0), (3, 1), (5, 1), (16, 0), (18, 0)]);
        assert_eq!(end, 1);
        assert_eq!(brackets("}", 1), (vec![(0, 0)], 0));
    }

    #[test]
    fn two_files_compared_line_by_line() {
        use crate::view::Change;
        let (l, r) = diff_lines("a\nb\nc\nd", "a\nB\nc\nd\ne");
        assert_eq!(l, vec![(1, Change::Deleted)]);
        assert_eq!(r, vec![(1, Change::Modified), (4, Change::Added)]);
        let (l, r) = diff_lines("same", "same");
        assert!(l.is_empty() && r.is_empty());
    }

    #[test]
    fn lines_surrounded_at_their_indent() {
        let got = surround("    a();\n\n    b();", "if condition {", "    ");
        assert_eq!(got, "    if condition {\n        a();\n\n        b();\n    }");
    }
}
