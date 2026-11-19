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
//! optional), and `-` a separator. After the label, in any order: a key (`Ctrl+N`), a glyph's
//! name for a picture (`SAVE`), `checked`, `check_on_click`, `disabled`. A line of words alone,
//! with lines indented further under it, is a menu within the menu:
//!
//! ```text
//! &View
//!   word_wrap | Word wrap | checked | check_on_click
//!   Zoom
//!     zoom_in | Zoom in | Ctrl+=
//!     zoom_out | Zoom out | Ctrl+-
//! ```
//!
//! The toolbar is one button a line: `id | label | GLYPH`, and `toggle` for one that stays down.

use crate::model::{Menu, MenuItem, Tool};

fn items_to_text(items: &[MenuItem], depth: usize, out: &mut String) {
    let pad = "  ".repeat(depth);
    for it in items {
        out.push_str(&pad);
        if it.separator {
            out.push_str("-\n");
            continue;
        }
        if !it.items.is_empty() {
            out.push_str(it.label.as_deref().unwrap_or(""));
            out.push('\n');
            items_to_text(&it.items, depth + 1, out);
            continue;
        }
        out.push_str(it.id.as_deref().unwrap_or(""));
        out.push_str(" | ");
        out.push_str(it.label.as_deref().unwrap_or(""));
        for extra in [it.key.clone(), it.glyph.clone(), it.checked.then(|| "checked".into()), it.check_on_click.then(|| "check_on_click".into()), it.disabled.then(|| "disabled".into())].into_iter().flatten() {
            out.push_str(" | ");
            out.push_str(&extra);
        }
        out.push('\n');
    }
}

/// The menus as text.
pub fn menus_to_text(menus: &[Menu]) -> String {
    let mut out = String::new();
    for m in menus {
        out.push_str(&m.label);
        out.push('\n');
        items_to_text(&m.items, 1, &mut out);
    }
    out
}

/// One item's line, `id | label | ...`.
fn item_of(t: &str, line: usize) -> Result<MenuItem, String> {
    let parts: Vec<&str> = t.split('|').map(str::trim).collect();
    if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(format!("line {line}: an item is  id | label  or  id | label | key"));
    }
    let mut it = MenuItem { id: Some(parts[0].to_string()), label: Some(parts[1].to_string()), ..Default::default() };
    for p in parts[2..].iter().filter(|p| !p.is_empty()) {
        match *p {
            "checked" => it.checked = true,
            "check_on_click" => it.check_on_click = true,
            "disabled" => it.disabled = true,
            g if phai_ui::icon::glyph::by_name(g).is_some() => it.glyph = Some(g.to_string()),
            k => it.key = Some(k.to_string()),
        }
    }
    Ok(it)
}

/// The items at `items`' place in the tree: `path` the indexes of the menus within menus.
fn at_path<'a>(items: &'a mut Vec<MenuItem>, path: &[usize]) -> &'a mut Vec<MenuItem> {
    match path.split_first() {
        None => items,
        Some((i, rest)) => at_path(&mut items[*i].items, rest),
    }
}

/// Text back to menus; an error names its line.
pub fn menus_from_text(text: &str) -> Result<Vec<Menu>, String> {
    let mut menus: Vec<Menu> = Vec::new();
    // The menus within menus the lines are in: each one's indent and its place.
    let mut stack: Vec<(usize, usize)> = Vec::new();
    // Each menu within a menu: its line, its menu, its place - to find one left empty.
    let mut subs: Vec<(usize, usize, Vec<usize>)> = Vec::new();
    for (n, raw) in text.lines().enumerate() {
        let line = n + 1;
        if raw.trim().is_empty() {
            continue;
        }
        let indent = raw.chars().take_while(|c| *c == ' ' || *c == '\t').map(|c| if c == '\t' { 2 } else { 1 }).sum::<usize>();
        if indent == 0 {
            menus.push(Menu { label: raw.trim().to_string(), items: vec![] });
            stack.clear();
            continue;
        }
        let mi = menus.len().wrapping_sub(1);
        let Some(m) = menus.last_mut() else { return Err(format!("line {line}: an item before any menu - start with a menu name at the left")) };
        // Out of the menus within menus this line is no longer under.
        while stack.last().is_some_and(|(ind, _)| *ind >= indent) {
            stack.pop();
        }
        let path: Vec<usize> = stack.iter().map(|(_, i)| *i).collect();
        let list = at_path(&mut m.items, &path);
        let t = raw.trim();
        if t == "-" {
            list.push(MenuItem { separator: true, ..Default::default() });
            continue;
        }
        if !t.contains('|') {
            // Words alone: a menu within the menu, its items indented under it.
            list.push(MenuItem { label: Some(t.to_string()), ..Default::default() });
            let mut at = path.clone();
            at.push(list.len() - 1);
            subs.push((line, mi, at));
            stack.push((indent, list.len() - 1));
            continue;
        }
        list.push(item_of(t, line)?);
    }
    for (line, mi, path) in subs {
        let (last, up) = path.split_last().unwrap_or((&0, &[]));
        if at_path(&mut menus[mi].items, up)[*last].items.is_empty() {
            return Err(format!("line {line}: an item is  id | label  - or words alone with items indented under them"));
        }
    }
    Ok(menus)
}

/// The toolbar as text.
pub fn toolbar_to_text(tools: &[Tool]) -> String {
    tools.iter().map(|t| format!("{} | {} | {}{}\n", t.id, t.label, t.glyph, if t.toggle { " | toggle" } else { "" })).collect()
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
        let toggle = parts.len() == 4 && parts[3] == "toggle";
        if !(parts.len() == 3 || toggle) || parts.iter().any(|p| p.is_empty()) {
            return Err(format!("line {}: a button is  id | label | GLYPH  (and  | toggle  for one that stays down)", n + 1));
        }
        if phai_ui::icon::glyph::by_name(parts[2]).is_none() {
            let names: Vec<&str> = phai_ui::icon::glyph::NAMES.iter().map(|(n, _)| *n).collect();
            return Err(format!("line {}: {} is not a glyph - one of {}", n + 1, parts[2], names.join(", ")));
        }
        out.push(Tool { id: parts[0].to_string(), label: parts[1].to_string(), glyph: parts[2].to_string(), toggle });
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
    fn menus_within_menus_ticks_pictures_and_greyed_items() {
        let text = "&View\n  word_wrap | Word wrap | checked | check_on_click\n  Zoom\n    zoom_in | Zoom in | Ctrl+= | SEARCH\n    Very\n      zoom_max | Most | disabled\n  -\n  refresh | Refresh | F5\n&Help\n  about | About\n";
        let menus = menus_from_text(text).unwrap();
        let v = &menus[0];
        assert!(v.items[0].checked && v.items[0].check_on_click);
        assert_eq!(v.items[1].label.as_deref(), Some("Zoom"));
        assert_eq!(v.items[1].items[0].glyph.as_deref(), Some("SEARCH"));
        assert_eq!(v.items[1].items[0].key.as_deref(), Some("Ctrl+="));
        assert!(v.items[1].items[1].items[0].disabled);
        assert!(v.items[2].separator && v.items[3].key.as_deref() == Some("F5"), "back out of the menus within");
        assert_eq!(v.all_items().len(), 7);
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
        let text = "save | Save | SAVE\nnew_customer | New customer | NEW\nbold | Bold | FONT | toggle\n";
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
