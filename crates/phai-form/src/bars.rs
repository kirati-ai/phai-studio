//! A form's menus and toolbar as text a person edits in one box - shorter and faster than a tree
//! of dialogs, and exact:
//!
//! ```text
//! &File
//!   new_customer | New customer | Ctrl+N
//!   -
//!   exit | Exit
//! &Help
//!   about | About
//! ```
//!
//! A line at the left starts a menu; an indented line is an item, `id | label | key` (the key is
//! optional), and `-` a separator. The toolbar is one button a line: `id | label | GLYPH`.

use crate::model::{Menu, MenuItem, Tool};

/// The menus as text.
pub fn menus_to_text(menus: &[Menu]) -> String {
    let mut out = String::new();
    for m in menus {
        out.push_str(&m.label);
        out.push('\n');
        for it in &m.items {
            if it.separator {
                out.push_str("  -\n");
                continue;
            }
            out.push_str("  ");
            out.push_str(it.id.as_deref().unwrap_or(""));
            out.push_str(" | ");
            out.push_str(it.label.as_deref().unwrap_or(""));
            if let Some(k) = &it.key {
                out.push_str(" | ");
                out.push_str(k);
            }
            out.push('\n');
        }
    }
    out
}

/// Text back to menus; an error names its line.
pub fn menus_from_text(text: &str) -> Result<Vec<Menu>, String> {
    let mut menus: Vec<Menu> = Vec::new();
    for (n, raw) in text.lines().enumerate() {
        let line = n + 1;
        if raw.trim().is_empty() {
            continue;
        }
        let indented = raw.starts_with(' ') || raw.starts_with('\t');
        if !indented {
            menus.push(Menu { label: raw.trim().to_string(), items: vec![] });
            continue;
        }
        let Some(m) = menus.last_mut() else { return Err(format!("line {line}: an item before any menu - start with a menu name at the left")) };
        let t = raw.trim();
        if t == "-" {
            m.items.push(MenuItem { separator: true, ..Default::default() });
            continue;
        }
        let parts: Vec<&str> = t.split('|').map(str::trim).collect();
        if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(format!("line {line}: an item is  id | label  or  id | label | key"));
        }
        m.items.push(MenuItem { id: Some(parts[0].to_string()), label: Some(parts[1].to_string()), key: parts.get(2).filter(|k| !k.is_empty()).map(|k| k.to_string()), separator: false });
    }
    Ok(menus)
}

/// The toolbar as text.
pub fn toolbar_to_text(tools: &[Tool]) -> String {
    tools.iter().map(|t| format!("{} | {} | {}\n", t.id, t.label, t.glyph)).collect()
}

/// Text back to a toolbar; an error names its line.
pub fn toolbar_from_text(text: &str) -> Result<Vec<Tool>, String> {
    let mut out = Vec::new();
    for (n, raw) in text.lines().enumerate() {
        let t = raw.trim();
        if t.is_empty() {
            continue;
        }
        let parts: Vec<&str> = t.split('|').map(str::trim).collect();
        if parts.len() != 3 || parts.iter().any(|p| p.is_empty()) {
            return Err(format!("line {}: a button is  id | label | GLYPH", n + 1));
        }
        if phai_ui::icon::glyph::by_name(parts[2]).is_none() {
            let names: Vec<&str> = phai_ui::icon::glyph::NAMES.iter().map(|(n, _)| *n).collect();
            return Err(format!("line {}: {} is not a glyph - one of {}", n + 1, parts[2], names.join(", ")));
        }
        out.push(Tool { id: parts[0].to_string(), label: parts[1].to_string(), glyph: parts[2].to_string() });
    }
    Ok(out)
}

/// A menu label as shown: `&File` is "File" with the F underlined - the underline is Windows'.
pub fn shown(label: &str) -> String {
    label.replace("&&", "\u{1}").replace('&', "").replace('\u{1}', "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menus_go_to_text_and_back_unchanged() {
        let text = "&File\n  new_customer | New customer | Ctrl+N\n  -\n  exit | Exit\n&Help\n  about | About\n";
        let menus = menus_from_text(text).unwrap();
        assert_eq!(menus.len(), 2);
        assert_eq!(menus[0].items[0].key.as_deref(), Some("Ctrl+N"));
        assert!(menus[0].items[1].separator);
        assert_eq!(menus_to_text(&menus), text);
    }

    #[test]
    fn mistakes_name_their_line() {
        assert!(menus_from_text("  exit | Exit").unwrap_err().starts_with("line 1"));
        assert!(menus_from_text("&File\n  exit").unwrap_err().starts_with("line 2"));
        assert_eq!(menus_from_text("\n\n").unwrap(), vec![]);
    }

    #[test]
    fn the_toolbar_goes_to_text_and_back_and_knows_its_glyphs() {
        let text = "save | Save | SAVE\nnew_customer | New customer | NEW\n";
        let tools = toolbar_from_text(text).unwrap();
        assert_eq!(toolbar_to_text(&tools), text);
        let e = toolbar_from_text("save | Save | DISK").unwrap_err();
        assert!(e.contains("DISK") && e.contains("SAVE"), "{e}");
        assert!(toolbar_from_text("save | Save").is_err());
    }

    #[test]
    fn ampersands_mark_the_key_letter() {
        assert_eq!(shown("&File"), "File");
        assert_eq!(shown("Save && quit"), "Save & quit");
    }
}
