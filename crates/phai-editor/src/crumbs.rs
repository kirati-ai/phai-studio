//! The breadcrumb over the editor, as Visual Studio's navigation bar and Android Studio's
//! breadcrumbs: where the caret is - the module, the `impl`, the function - found by reading the
//! braces above it. Strings and comments are passed over; a signature over several lines takes the
//! brace that ends it.

/// What a line declares that a `{` opens: `fn name`, `impl Thing`, `mod name`, ...
fn declared(line: &str) -> Option<String> {
    let mut t = line.trim_start();
    loop {
        let before = t;
        for word in ["pub(crate) ", "pub(super) ", "pub ", "async ", "unsafe ", "const ", "extern \"C\" ", "default "] {
            t = t.strip_prefix(word).unwrap_or(t);
        }
        if t == before {
            break;
        }
    }
    let name = |rest: &str| rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect::<String>();
    for (word, kind) in [("fn ", "fn"), ("mod ", "mod"), ("struct ", "struct"), ("enum ", "enum"), ("trait ", "trait")] {
        if let Some(rest) = t.strip_prefix(word) {
            let n = name(rest);
            return (!n.is_empty()).then(|| format!("{kind} {n}"));
        }
    }
    if t.starts_with("impl ") || t.starts_with("impl<") {
        let head = t.split('{').next().unwrap_or(t).trim();
        // `impl<T> Foo<T> for Bar` read as `impl Foo for Bar`, generics left out.
        let mut out = String::new();
        let mut depth = 0;
        for c in head.chars() {
            match c {
                '<' => depth += 1,
                '>' => depth -= 1,
                _ if depth == 0 => out.push(c),
                _ => {}
            }
        }
        let out = out.split(" where ").next().unwrap_or(&out).split_whitespace().collect::<Vec<_>>().join(" ");
        return Some(out);
    }
    None
}

/// The line with strings, characters and the comment at its end taken out.
fn code_of(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '/' if chars.peek() == Some(&'/') => break,
            '"' => {
                while let Some(d) = chars.next() {
                    if d == '\\' {
                        chars.next();
                    } else if d == '"' {
                        break;
                    }
                }
            }
            '\'' => {
                // A character ('{') - not a lifetime ('a).
                let rest: String = chars.clone().take(3).collect();
                if rest.starts_with("\\") || rest.chars().nth(1) == Some('\'') {
                    for d in chars.by_ref() {
                        if d == '\'' {
                            break;
                        }
                    }
                }
            }
            _ => out.push(c),
        }
    }
    out
}

/// Where line `line` (from 0) of `text` is: the items around it, outermost first.
pub fn scope_at(text: &str, line: usize) -> Vec<String> {
    let mut stack: Vec<Option<String>> = Vec::new();
    let mut pending: Option<String> = None;
    for (i, l) in text.lines().enumerate() {
        if i > line {
            break;
        }
        let code = code_of(l);
        if let Some(d) = declared(&code) {
            pending = Some(d);
        }
        for c in code.chars() {
            match c {
                '{' => stack.push(pending.take()),
                '}' if i < line => {
                    stack.pop();
                }
                ';' => pending = None,
                _ => {}
            }
        }
    }
    stack.into_iter().flatten().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = r#"mod shop {
    pub struct Cart {
        items: Vec<u32>,
    }

    impl<T: Clone> Iterator for Cart where T: Copy {
        fn next(&mut self) -> Option<u32> {
            let s = "{ not a block";
            if true { // {
                None
            } else {
                Some(1)
            }
        }

        pub async fn long(
            a: u32,
        ) -> u32 {
            a
        }
    }
}
fn after() {}
"#;

    #[test]
    fn the_caret_s_place_read_from_the_braces() {
        assert_eq!(scope_at(SRC, 2), vec!["mod shop", "struct Cart"]);
        assert_eq!(scope_at(SRC, 9), vec!["mod shop", "impl Iterator for Cart", "fn next"]);
        assert_eq!(scope_at(SRC, 11), vec!["mod shop", "impl Iterator for Cart", "fn next"]);
        assert_eq!(scope_at(SRC, 18), vec!["mod shop", "impl Iterator for Cart", "fn long"], "a signature over lines");
        assert_eq!(scope_at(SRC, 4), vec!["mod shop"]);
        assert_eq!(scope_at(SRC, 22), vec!["fn after"]);
    }
}
