//! The code editor's preferences, as Tools > Options > Text Editor sets them: how wide a tab is
//! and whether Tab types spaces, line numbers, reference counts, the bracket highlight, the font.

use std::sync::RwLock;

#[derive(Clone, Debug, PartialEq)]
pub struct Prefs {
    /// Columns a tab stands for, and a level of indent.
    pub tab_size: usize,
    /// Tab and a new indent type spaces; off, a tab character.
    pub spaces: bool,
    pub line_numbers: bool,
    /// Reference counts after a function's line (code lenses).
    pub lenses: bool,
    /// The bracket by the caret and its match, boxed.
    pub braces: bool,
    /// The code's font: a fixed-width one.
    pub font: String,
}

impl Default for Prefs {
    fn default() -> Prefs {
        Prefs { tab_size: 4, spaces: true, line_numbers: true, lenses: true, braces: true, font: "Cascadia Mono".to_string() }
    }
}

static PREFS: RwLock<Option<Prefs>> = RwLock::new(None);

pub fn get() -> Prefs {
    PREFS.read().ok().and_then(|p| p.clone()).unwrap_or_default()
}

/// New preferences; open editors follow when they next paint (the font when they are made again).
pub fn set(p: Prefs) {
    if let Ok(mut w) = PREFS.write() {
        *w = Some(Prefs { tab_size: p.tab_size.clamp(1, 16), ..p });
    }
}

/// One level of indent: `tab_size` spaces, or a tab.
pub fn indent() -> String {
    let p = get();
    if p.spaces {
        " ".repeat(p.tab_size)
    } else {
        "\t".to_string()
    }
}

/// A tab drawn: as many spaces as it stands for.
pub fn tab_spaces() -> String {
    " ".repeat(get().tab_size)
}
