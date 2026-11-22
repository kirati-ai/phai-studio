//! Colours for code: Rust, TOML and CSS split into tokens a line at a time, as Monaco's Monarch
//! tokenizer does, with the one state that crosses lines - inside a `/* */` comment - carried from
//! line to line.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Token {
    Plain,
    Keyword,
    Type,
    String,
    Number,
    Comment,
    Macro,
    Attribute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Rust,
    Toml,
    Css,
    Text,
}

impl Language {
    pub fn of(path: &str) -> Language {
        let lower = path.to_ascii_lowercase();
        if lower.ends_with(".rs") {
            Language::Rust
        } else if lower.ends_with(".toml") || lower.ends_with(".phf") || lower.ends_with(".phaiform") {
            Language::Toml
        } else if lower.ends_with(".css") {
            Language::Css
        } else {
            Language::Text
        }
    }
}

const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while",
];

/// The tokens of one line as `(byte start, byte end, token)`, and whether a block comment is still
/// open at its end. `in_comment` says whether one was open at its start.
pub fn line_tokens(lang: Language, line: &str, in_comment: bool) -> (Vec<(usize, usize, Token)>, bool) {
    let mut out = Vec::new();
    if lang == Language::Text {
        out.push((0, line.len(), Token::Plain));
        return (out, false);
    }
    let b = line.as_bytes();
    let mut i = 0;
    let mut open = in_comment;
    let line_comment: &[u8] = if lang == Language::Toml { b"#" } else { b"//" };
    while i < b.len() {
        let start = i;
        if open {
            match line[i..].find("*/") {
                Some(k) => {
                    i += k + 2;
                    open = false;
                }
                None => i = b.len(),
            }
            out.push((start, i, Token::Comment));
            continue;
        }
        let c = b[i];
        if b[i..].starts_with(line_comment) && !(lang == Language::Css && line_comment == b"//") {
            out.push((i, b.len(), Token::Comment));
            break;
        }
        if lang != Language::Toml && b[i..].starts_with(b"/*") {
            open = true;
            i += 2;
            out.push((start, i, Token::Comment));
            continue;
        }
        if c == b'"' || (c == b'\'' && lang != Language::Rust) {
            i += 1;
            while i < b.len() && b[i] != c {
                if b[i] == b'\\' {
                    i += 1;
                }
                i += 1;
            }
            i = (i + 1).min(b.len());
            out.push((start, i, Token::String));
            continue;
        }
        if lang == Language::Rust && c == b'#' && b.get(i + 1).is_some_and(|n| *n == b'[' || *n == b'!') {
            let end = line[i..].find(']').map(|k| i + k + 1).unwrap_or(b.len());
            out.push((start, end, Token::Attribute));
            i = end;
            continue;
        }
        if c.is_ascii_digit() || (lang == Language::Css && c == b'#' && b.get(i + 1).is_some_and(|n| n.is_ascii_hexdigit())) {
            i += 1;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'.') {
                i += 1;
            }
            out.push((start, i, Token::Number));
            continue;
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || (lang == Language::Css && b[i] == b'-')) {
                i += 1;
            }
            let word = &line[start..i];
            let token = match lang {
                Language::Rust if b.get(i) == Some(&b'!') => {
                    i += 1;
                    Token::Macro
                }
                Language::Rust if RUST_KEYWORDS.contains(&word) => Token::Keyword,
                Language::Rust if word.starts_with(|ch: char| ch.is_ascii_uppercase()) => Token::Type,
                Language::Toml if word == "true" || word == "false" => Token::Keyword,
                Language::Css if b.get(i) == Some(&b':') || word.starts_with("var") => Token::Keyword,
                _ => Token::Plain,
            };
            out.push((start, i, token));
            continue;
        }
        // Anything else - punctuation, spaces, Thai - one character as plain.
        i += line[i..].chars().next().map(char::len_utf8).unwrap_or(1);
        match out.last_mut() {
            Some(last) if last.2 == Token::Plain && last.1 == start => last.1 = i,
            _ => out.push((start, i, Token::Plain)),
        }
    }
    (out, open)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(lang: Language, line: &str) -> Vec<(String, Token)> {
        line_tokens(lang, line, false).0.into_iter().filter(|t| t.2 != Token::Plain).map(|(a, b, t)| (line[a..b].to_string(), t)).collect()
    }

    #[test]
    fn rust_keywords_types_strings_macros_and_comments() {
        let k = kinds(Language::Rust, r#"pub fn go(x: Option<u32>) { println!("ไทย {}", 42); } // done"#);
        assert!(k.contains(&("pub".into(), Token::Keyword)));
        assert!(k.contains(&("Option".into(), Token::Type)));
        assert!(k.contains(&("println!".into(), Token::Macro)));
        assert!(k.contains(&("\"ไทย {}\"".into(), Token::String)));
        assert!(k.contains(&("42".into(), Token::Number)));
        assert!(k.contains(&("// done".into(), Token::Comment)));
    }

    #[test]
    fn a_block_comment_carries_to_the_next_line() {
        let (_, open) = line_tokens(Language::Rust, "let a = 1; /* starts", false);
        assert!(open);
        let (t, open) = line_tokens(Language::Rust, "still */ let b", true);
        assert!(!open);
        assert_eq!(t[0].2, Token::Comment);
    }

    #[test]
    fn toml_and_css() {
        assert!(kinds(Language::Toml, r#"name = "x" # note"#).contains(&("# note".into(), Token::Comment)));
        assert!(kinds(Language::Css, "button { color: #12855f; }").contains(&("#12855f".into(), Token::Number)));
        assert_eq!(Language::of("a/b/main.rs"), Language::Rust);
        assert_eq!(Language::of("theme.css"), Language::Css);
    }
}
