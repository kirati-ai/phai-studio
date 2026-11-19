//! Changing a theme file's words as a person would, for the Theme Editor: a `:root` variable set,
//! the built-in theme it starts from swapped, one property of one rule set - everything else in
//! the file, its comments and its order, left as it was.

/// The `:root` variables the file itself sets, in order: `("--accent", "#12855f")`.
pub fn vars(css: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (sel, body, _) in blocks(css) {
        if sel.trim() == ":root" {
            for d in body.split(';') {
                if let Some((k, v)) = d.split_once(':') {
                    let k = name_of(k);
                    if k.starts_with("--") {
                        out.push((k, strip(v).trim().to_string()));
                    }
                }
            }
        }
    }
    out
}

/// The built-in theme the file starts from - `phai:light` or `phai:dark` - if it imports one.
pub fn base(css: &str) -> Option<&'static str> {
    for line in css.lines() {
        let l = line.trim();
        if l.starts_with("@import") {
            if l.contains("phai:dark") {
                return Some("phai:dark");
            }
            if l.contains("phai:light") {
                return Some("phai:light");
            }
        }
    }
    None
}

/// The file starting from `phai:light` or `phai:dark` instead: its `@import` line changed, or one
/// put first.
pub fn set_base(css: &str, to: &str) -> String {
    let mut done = false;
    let mut out: Vec<String> = css
        .lines()
        .map(|line| {
            let l = line.trim();
            if !done && l.starts_with("@import") && (l.contains("phai:light") || l.contains("phai:dark")) {
                done = true;
                format!("@import \"{to}\";")
            } else {
                line.to_string()
            }
        })
        .collect();
    if !done {
        out.insert(0, format!("@import \"{to}\";"));
    }
    let mut s = out.join("\n");
    if css.ends_with('\n') {
        s.push('\n');
    }
    s
}

/// The file with `:root`'s `name` set to `value`: the declaration changed where it is, or added to
/// the first `:root` block, or a `:root` block made after the `@import`s.
pub fn set_var(css: &str, name: &str, value: &str) -> String {
    set_prop(css, ":root", name, value)
}

/// The file with `prop` of the rule `selector` (as written, `button`) set to `value`: changed
/// where it is, added to that rule, or a new rule at the end (`:root`: after the `@import`s).
pub fn set_prop(css: &str, selector: &str, prop: &str, value: &str) -> String {
    for (sel, body, at) in blocks(css) {
        if sel.trim() != selector {
            continue;
        }
        // The declaration itself, comments and spacing round it kept.
        let mut offset = 0;
        for d in body.split(';') {
            if let Some((k, v)) = d.split_once(':') {
                if name_of(k) == prop {
                    let vstart = at + offset + k.len() + 1;
                    let lead = v.len() - v.trim_start().len();
                    let core = strip(v).trim_end();
                    let (from, to) = (vstart + lead, vstart + lead + core.trim_start().len());
                    return format!("{}{value}{}", &css[..from], &css[to..]);
                }
            }
            offset += d.len() + 1;
        }
        // Not there: added before the block's end.
        let end = at + body.len();
        let before = css[..end].trim_end_matches([' ', '\t', '\n', '\r']);
        let sep = if before.ends_with(';') || before.ends_with('{') { "" } else { ";" };
        let indent = if body.contains('\n') { "\n  " } else { " " };
        let tail = if body.contains('\n') { "\n" } else { " " };
        return format!("{before}{sep}{indent}{prop}: {value};{tail}{}", &css[end..]);
    }
    let rule = format!("{selector} {{ {prop}: {value}; }}");
    if selector == ":root" {
        // After the imports, so the variable recolours what they say.
        let mut lines: Vec<&str> = css.lines().collect();
        let at = lines.iter().rposition(|l| l.trim_start().starts_with("@import")).map(|i| i + 1).unwrap_or(0);
        lines.insert(at, &rule);
        let mut s = lines.join("\n");
        if css.ends_with('\n') || css.is_empty() {
            s.push('\n');
        }
        return s;
    }
    let mut s = css.to_string();
    if !s.is_empty() && !s.ends_with('\n') {
        s.push('\n');
    }
    s.push_str(&rule);
    s.push('\n');
    s
}

/// The file with `prop` taken out of the rule `selector` - the theme it starts from says it then.
/// A rule left with nothing in it goes too.
pub fn remove_prop(css: &str, selector: &str, prop: &str) -> String {
    for (sel, body, at) in blocks(css) {
        if sel.trim() != selector {
            continue;
        }
        let mut offset = 0;
        for d in body.split(';') {
            if d.split_once(':').is_some_and(|(k, _)| name_of(k) == prop) {
                let from = at + offset;
                let to = (at + offset + d.len() + 1).min(at + body.len());
                // The declaration with the space before it, comments after it kept.
                let lead = d.len() - d.trim_start().len();
                let comment = d.find("/*").map(|i| d[i..].to_string()).unwrap_or_default();
                let mut s = format!("{}{}{}", &css[..from + lead.min(d.len())], comment, &css[to..]);
                if comment.is_empty() {
                    s = format!("{}{}", css[..from].trim_end_matches([' ', '\t']), &css[to..]);
                }
                // Nothing left in it: the rule goes.
                if let Some((_, b, at2)) = blocks(&s).into_iter().find(|(x, _, _)| x.trim() == selector) {
                    if b.trim().is_empty() {
                        let start = s[..at2 - 1].rfind(['}', ';']).map(|p| p + 1).unwrap_or(0);
                        let end = at2 + b.len() + 1;
                        let rest = s[end..].strip_prefix('\n').unwrap_or(&s[end..]);
                        return format!("{}{}", s[..start].trim_end_matches([' ', '\t']), if start == 0 { rest.to_string() } else { format!("\n{rest}").replacen("\n\n", "\n", 1) });
                    }
                }
                return s;
            }
            offset += d.len() + 1;
        }
    }
    css.to_string()
}

/// What a rule written `selector` in the file says its `prop` is, if it says.
pub fn prop(css: &str, selector: &str, prop: &str) -> Option<String> {
    blocks(css).into_iter().filter(|(s, _, _)| s.trim() == selector).flat_map(|(_, body, _)| body.split(';').filter_map(|d| d.split_once(':')).filter(|(k, _)| name_of(k) == prop).map(|(_, v)| strip(v).trim().to_string()).collect::<Vec<_>>()).last()
}

/// The text before a `/*` - a value with a comment after it.
fn strip(s: &str) -> &str {
    s.split("/*").next().unwrap_or(s)
}

/// A declaration's name, comments before or after it taken out.
fn name_of(k: &str) -> String {
    let mut out = String::new();
    let mut rest = k;
    while let Some(i) = rest.find("/*") {
        out.push_str(&rest[..i]);
        rest = rest[i + 2..].split_once("*/").map(|(_, r)| r).unwrap_or("");
    }
    out.push_str(rest);
    out.trim().to_string()
}

/// Every `selector { body }` of the file outside comments: the selector, the body, and where the
/// body starts in the file.
fn blocks(css: &str) -> Vec<(&str, &str, usize)> {
    // Comments blanked, places kept, so `{` and `}` in them do not count.
    let mut plain: Vec<u8> = css.as_bytes().to_vec();
    let mut i = 0;
    while let Some(s) = css[i..].find("/*") {
        let s = i + s;
        let e = css[s + 2..].find("*/").map(|e| s + 2 + e + 2).unwrap_or(css.len());
        for b in plain.iter_mut().take(e).skip(s) {
            if *b != b'\n' {
                *b = b' ';
            }
        }
        i = e;
    }
    let plain = String::from_utf8_lossy(&plain).to_string();
    let mut out = Vec::new();
    let mut pos = 0;
    while let Some(open) = plain[pos..].find('{') {
        let open = pos + open;
        let Some(close) = plain[open..].find('}') else { break };
        let close = open + close;
        let sel_start = plain[..open].rfind(['}', ';']).map(|p| p + 1).unwrap_or(0).max(pos);
        out.push((&css[sel_start..open], &css[open + 1..close], open + 1));
        pos = close + 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHEET: &str = "@import \"phai:light\";\n\n:root {\n  --accent: #12855f;   /* green */\n  --text: #1b1b1b;\n}\n\nbutton { border-radius: 4px; }\n";

    #[test]
    fn variables_are_read_and_changed_in_place() {
        assert_eq!(vars(SHEET), vec![("--accent".to_string(), "#12855f".to_string()), ("--text".to_string(), "#1b1b1b".to_string())]);
        let s = set_var(SHEET, "--accent", "#2563eb");
        assert!(s.contains("  --accent: #2563eb;   /* green */\n"), "{s}");
        assert_eq!(s.len(), SHEET.len(), "nothing else moved");
        let s = set_var(SHEET, "--muted", "#666666");
        assert!(s.contains("--text: #1b1b1b;\n  --muted: #666666;\n}"), "{s}");
        let s = set_var("@import \"phai:dark\";\nbutton { color: red; }\n", "--accent", "#f00");
        assert_eq!(s, "@import \"phai:dark\";\n:root { --accent: #f00; }\nbutton { color: red; }\n");
        assert!(crate::theme::Theme::parse(&set_var(SHEET, "--accent", "#2563eb")).is_ok());
    }

    #[test]
    fn the_base_and_a_rules_property() {
        assert_eq!(base(SHEET), Some("phai:light"));
        let d = set_base(SHEET, "phai:dark");
        assert!(d.starts_with("@import \"phai:dark\";\n"));
        assert_eq!(base(&d), Some("phai:dark"));
        assert_eq!(set_base("button { color: red; }", "phai:light"), "@import \"phai:light\";\nbutton { color: red; }");
        assert_eq!(prop(SHEET, "button", "border-radius").as_deref(), Some("4px"));
        let s = set_prop(SHEET, "button", "border-radius", "10px");
        assert!(s.contains("button { border-radius: 10px; }"), "{s}");
        let s = set_prop(SHEET, "edit", "border-radius", "6px");
        assert!(s.ends_with("edit { border-radius: 6px; }\n"), "{s}");
        let s = set_prop("button { color: red }", "button", "border-radius", "3px");
        assert_eq!(s, "button { color: red; border-radius: 3px; }");
        assert_eq!(prop("/* button { border-radius: 9px; } */ button { color: red; }", "button", "border-radius"), None, "comments do not count");
        let s = remove_prop("button { color: red; border-radius: 3px; }\n", "button", "border-radius");
        assert_eq!(s, "button { color: red; }\n");
        let s = remove_prop("a { color: red; }\nbutton { border-radius: 3px; }\nedit { color: blue; }\n", "button", "border-radius");
        assert_eq!(s, "a { color: red; }\nedit { color: blue; }\n", "an empty rule goes");
        assert!(crate::theme::Theme::parse(&remove_prop(SHEET, ":root", "--text")).is_ok());
    }
}
