//! Themes written as CSS: a small, strict part of it, enough to say what every control looks like
//! in every state. The way Qt (QSS) and GTK theme native widgets - the controls stay Win32, the
//! sheet only says their colours, borders and corners.
//!
//! ```css
//! :root { --accent: #12855f; }
//! button { background: #fff; border-color: #d4d4d4; border-radius: 4px; }
//! button:hover { background: #e7f3ef; border-color: var(--accent); }
//! button.primary { background: var(--accent); color: #fff; }
//! #save { font-weight: 600; }
//! ```
//!
//! **Selectors**: a control kind (`button`, `checkbox`, `window`...) or `*`, then any `.class`,
//! one `#id`, and states `:hover :pressed :focus :disabled :selected :checked`; several joined by
//! commas. No descendants, no `>` - a Win32 window is flat, and a rule that depends on where a
//! control sits is a rule nobody can predict. **Values**: `#rgb`, `#rrggbb`, `#rrggbbaa`,
//! `rgb(r, g, b)`, `transparent`, `white`, `black`, numbers with or without `px`/`pt`, and
//! `var(--name)` / `var(--name, fallback)` from `:root`. The later and more specific rule wins,
//! as in a browser. Anything the sheet cannot be read as is an error with its line, never a
//! silently ignored rule: a theme that half-applies looks like a bug in the control.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b, a: 255 }
    }

    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };

    /// A GDI `COLORREF` (0x00bbggrr).
    pub fn colorref(self) -> u32 {
        self.r as u32 | (self.g as u32) << 8 | (self.b as u32) << 16
    }

    /// Perceived lightness, 0-255: tells a dark theme from a light one.
    pub fn luma(self) -> u8 {
        ((self.r as u32 * 299 + self.g as u32 * 587 + self.b as u32 * 114) / 1000) as u8
    }

    pub fn is_dark(self) -> bool {
        self.luma() < 128
    }
}

/// What a control is doing right now.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub hover: bool,
    pub pressed: bool,
    pub focus: bool,
    pub disabled: bool,
    pub selected: bool,
    pub checked: bool,
}

impl State {
    fn has(&self, s: Pseudo) -> bool {
        match s {
            Pseudo::Hover => self.hover,
            Pseudo::Pressed => self.pressed,
            Pseudo::Focus => self.focus,
            Pseudo::Disabled => self.disabled,
            Pseudo::Selected => self.selected,
            Pseudo::Checked => self.checked,
        }
    }
}

/// How one control looks in one state - what the matching rules add up to. `None` is "the sheet
/// says nothing"; [`Style::or`] fills it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style {
    pub background: Option<Color>,
    pub color: Option<Color>,
    pub border_color: Option<Color>,
    /// Design pixels at 96 DPI.
    pub border_width: Option<i32>,
    pub border_radius: Option<i32>,
    pub font_weight: Option<i32>,
    /// Points.
    pub font_size: Option<i32>,
    /// The tick of a checkbox, the dot of a radio, a selected tab's line.
    pub accent_color: Option<Color>,
    /// Toolbar glyphs.
    pub icon_color: Option<Color>,
}

impl Style {
    pub fn background_or(&self, c: Color) -> Color {
        self.background.unwrap_or(c)
    }
    pub fn color_or(&self, c: Color) -> Color {
        self.color.unwrap_or(c)
    }
    pub fn border_or(&self, c: Color) -> Color {
        self.border_color.unwrap_or(c)
    }
    pub fn accent_or(&self, c: Color) -> Color {
        self.accent_color.unwrap_or(c)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pseudo {
    Hover,
    Pressed,
    Focus,
    Disabled,
    Selected,
    Checked,
}

#[derive(Clone, Debug, PartialEq)]
struct Selector {
    kind: Option<String>,
    classes: Vec<String>,
    id: Option<String>,
    states: Vec<Pseudo>,
}

impl Selector {
    /// (ids, classes and states, kinds) - compared as a tuple, as CSS does.
    fn specificity(&self) -> (u8, u8, u8) {
        (self.id.is_some() as u8, (self.classes.len() + self.states.len()) as u8, self.kind.is_some() as u8)
    }

    fn matches(&self, q: &Query) -> bool {
        self.kind.as_deref().is_none_or(|k| k == q.kind)
            && self.classes.iter().all(|c| q.classes.contains(&c.as_str()))
            && self.id.as_deref().is_none_or(|id| Some(id) == q.id)
            && self.states.iter().all(|s| q.state.has(*s))
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Value {
    Color(Color),
    Number(i32),
}

#[derive(Clone, Debug)]
struct Rule {
    selectors: Vec<Selector>,
    decls: Vec<(Prop, Value)>,
    order: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Prop {
    Background,
    Color,
    BorderColor,
    BorderWidth,
    BorderRadius,
    FontWeight,
    FontSize,
    AccentColor,
    IconColor,
}

impl Prop {
    fn parse(name: &str) -> Option<Prop> {
        Some(match name {
            "background" | "background-color" => Prop::Background,
            "color" => Prop::Color,
            "border-color" => Prop::BorderColor,
            "border-width" => Prop::BorderWidth,
            "border-radius" => Prop::BorderRadius,
            "font-weight" => Prop::FontWeight,
            "font-size" => Prop::FontSize,
            "accent-color" => Prop::AccentColor,
            "icon-color" => Prop::IconColor,
            _ => return None,
        })
    }

    fn is_color(self) -> bool {
        matches!(self, Prop::Background | Prop::Color | Prop::BorderColor | Prop::AccentColor | Prop::IconColor)
    }
}

/// Which control is asking: its kind, its classes, its id and what it is doing.
#[derive(Clone, Copy, Debug)]
pub struct Query<'a> {
    pub kind: &'a str,
    pub classes: &'a [&'a str],
    pub id: Option<&'a str>,
    pub state: State,
}

impl<'a> Query<'a> {
    pub fn kind(kind: &'a str) -> Query<'a> {
        Query { kind, classes: &[], id: None, state: State::default() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ThemeError {}

#[derive(Clone, Debug)]
pub struct Theme {
    rules: Vec<Rule>,
    vars: HashMap<String, String>,
}

pub const LIGHT_CSS: &str = include_str!("../themes/light.css");
pub const DARK_CSS: &str = include_str!("../themes/dark.css");

impl Theme {
    pub fn light() -> Theme {
        Theme::parse(LIGHT_CSS).expect("the built-in light theme reads")
    }

    pub fn dark() -> Theme {
        Theme::parse(DARK_CSS).expect("the built-in dark theme reads")
    }

    /// Reads a sheet. Every rule, every value and every `var()` is checked here, so a theme that
    /// loads is a theme that applies. `@import "phai:light";` and `"phai:dark"` bring in a
    /// built-in theme to change only part of; a file path needs [`load`].
    pub fn parse(css: &str) -> Result<Theme, ThemeError> {
        Theme::parse_with(css, &|name| Err(format!("{name:?}: only phai:light and phai:dark can be imported here")))
    }

    /// As [`Theme::parse`], with `files` reading any `@import` that is not built in.
    pub fn parse_with(css: &str, files: &dyn Fn(&str) -> Result<String, String>) -> Result<Theme, ThemeError> {
        let mut vars = HashMap::new();
        let mut raw = Vec::new();
        collect(css, "", files, 0, &mut vars, &mut raw)?;
        // Every value is read now, with every sheet's variables: a `--accent` set after an
        // `@import` recolours what the imported sheet said, as in a browser.
        let mut rules = Vec::new();
        for (selectors, decls, source) in raw {
            let mut out = Vec::new();
            for (name, value, line) in decls {
                let err = |message: String| ThemeError { line, message: if source.is_empty() { message } else { format!("{source}: {message}") } };
                let prop = Prop::parse(&name).ok_or_else(|| err(format!("unknown property {name}")))?;
                let text = resolve_vars(&value, &vars, line, 0).map_err(|e| err(e.message))?;
                let value = parse_value(prop, &text).ok_or_else(|| err(format!("{name}: cannot read {text:?}")))?;
                out.push((prop, value));
            }
            let order = rules.len();
            rules.push(Rule { selectors, decls: out, order });
        }
        Ok(Theme { rules, vars })
    }

    /// A `:root` variable as a colour - for drawing that is not one control's, like a rule line.
    pub fn var_color(&self, name: &str) -> Option<Color> {
        let raw = self.vars.get(name)?;
        parse_color(&resolve_vars(raw, &self.vars, 0, 0).ok()?)
    }

    /// What the sheet says about this control in this state.
    pub fn style(&self, q: &Query) -> Style {
        let mut hits: Vec<((u8, u8, u8), usize, &Rule)> = Vec::new();
        for r in &self.rules {
            if let Some(spec) = r.selectors.iter().filter(|s| s.matches(q)).map(|s| s.specificity()).max() {
                hits.push((spec, r.order, r));
            }
        }
        hits.sort_by_key(|(spec, order, _)| (*spec, *order));
        let mut s = Style::default();
        for (_, _, r) in hits {
            for (p, v) in &r.decls {
                match (p, v) {
                    (Prop::Background, Value::Color(c)) => s.background = Some(*c),
                    (Prop::Color, Value::Color(c)) => s.color = Some(*c),
                    (Prop::BorderColor, Value::Color(c)) => s.border_color = Some(*c),
                    (Prop::AccentColor, Value::Color(c)) => s.accent_color = Some(*c),
                    (Prop::IconColor, Value::Color(c)) => s.icon_color = Some(*c),
                    (Prop::BorderWidth, Value::Number(n)) => s.border_width = Some(*n),
                    (Prop::BorderRadius, Value::Number(n)) => s.border_radius = Some(*n),
                    (Prop::FontWeight, Value::Number(n)) => s.font_weight = Some(*n),
                    (Prop::FontSize, Value::Number(n)) => s.font_size = Some(*n),
                    _ => {}
                }
            }
        }
        s
    }

    /// The window background, which decides light or dark for everything Windows draws itself.
    pub fn is_dark(&self) -> bool {
        self.style(&Query::kind("window")).background.is_some_and(|c| c.is_dark())
    }
}

type RawRule = (Vec<Selector>, Vec<(String, String, usize)>, String);

/// One sheet's imports, variables and rules, in order, values still unread.
fn collect(css: &str, source: &str, files: &dyn Fn(&str) -> Result<String, String>, depth: usize, vars: &mut HashMap<String, String>, raw: &mut Vec<RawRule>) -> Result<(), ThemeError> {
    let named = |e: ThemeError| if source.is_empty() { e } else { ThemeError { line: e.line, message: format!("{source}: {}", e.message) } };
    if depth > 8 {
        return Err(ThemeError { line: 0, message: "@import goes too deep - sheets that import each other?".into() });
    }
    let mut css = strip_comments(css).map_err(named)?;
    // `@import "name";` first; each becomes spaces so line numbers stay true.
    while let Some(i) = css.find("@import") {
        let line = line_at(&css, i);
        let end = css[i..].find(';').map(|e| i + e).ok_or_else(|| named(ThemeError { line, message: "@import without ;".into() }))?;
        let what = css[i + 7..end].trim().trim_matches(|c| c == '"' || c == '\'').to_string();
        let text = match what.as_str() {
            "phai:light" => LIGHT_CSS.to_string(),
            "phai:dark" => DARK_CSS.to_string(),
            _ => files(&what).map_err(|m| named(ThemeError { line, message: m }))?,
        };
        collect(&text, &what, files, depth + 1, vars, raw)?;
        let blank: String = css[i..=end].chars().map(|c| if c == '\n' { '\n' } else { ' ' }).collect();
        css.replace_range(i..=end, &blank);
    }
    let blocks = split_blocks(&css).map_err(named)?;
    for b in &blocks {
        if b.selectors.trim() == ":root" {
            for (name, value, line) in declarations(&b.body, b.body_line).map_err(named)? {
                if !name.starts_with("--") {
                    return Err(named(ThemeError { line, message: format!("only --variables belong in :root, not {name}") }));
                }
                vars.insert(name, value);
            }
        }
    }
    for b in &blocks {
        if b.selectors.trim() == ":root" {
            continue;
        }
        let selectors = b.selectors.split(',').map(|s| parse_selector(s.trim(), b.line)).collect::<Result<Vec<_>, _>>().map_err(named)?;
        raw.push((selectors, declarations(&b.body, b.body_line).map_err(named)?, source.to_string()));
    }
    Ok(())
}

// --- the theme in use ---------------------------------------------------------------------------

static CURRENT: RwLock<Option<Arc<Theme>>> = RwLock::new(None);

/// The theme every control draws with; the built-in light one until [`set`] is called.
pub fn current() -> Arc<Theme> {
    if let Some(t) = CURRENT.read().unwrap_or_else(|e| e.into_inner()).as_ref() {
        return t.clone();
    }
    let t = Arc::new(Theme::light());
    *CURRENT.write().unwrap_or_else(|e| e.into_inner()) = Some(t.clone());
    t
}

/// Switches every window's theme. Open windows repaint with it after [`crate::win::retheme`].
pub fn set(theme: Theme) {
    *CURRENT.write().unwrap_or_else(|e| e.into_inner()) = Some(Arc::new(theme));
}

/// Reads a theme from a `.css` file. An error names the file and the line.
/// `@import "other.css";` inside it is read beside it.
pub fn load(path: &std::path::Path) -> Result<Theme, ThemeError> {
    let css = std::fs::read_to_string(path).map_err(|e| ThemeError { line: 0, message: format!("{}: {e}", path.display()) })?;
    let dir = path.parent().map(|d| d.to_path_buf()).unwrap_or_default();
    let files = move |name: &str| std::fs::read_to_string(dir.join(name)).map_err(|e| format!("@import {name:?}: {e}"));
    Theme::parse_with(&css, &files).map_err(|e| ThemeError { line: e.line, message: format!("{}: {}", path.display(), e.message) })
}

/// The standard sheet to start a theme from: every control, every state, every property, said
/// what each is for. It is the light theme itself, so the two can never drift apart. Phai Studio
/// copies it into a project as `theme.css`.
pub const STANDARD_CSS: &str = LIGHT_CSS;

/// `current().style(...)` for a control with no classes or id.
pub fn style(kind: &str, state: State) -> Style {
    current().style(&Query { kind, classes: &[], id: None, state })
}

// --- reading ----------------------------------------------------------------------------------

fn line_at(s: &str, byte: usize) -> usize {
    s[..byte.min(s.len())].matches('\n').count() + 1
}

/// Comments become spaces, newlines kept, so every line number still points at the source.
fn strip_comments(css: &str) -> Result<String, ThemeError> {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(i) = rest.find("/*") {
        out.push_str(&rest[..i]);
        let after = &rest[i + 2..];
        let Some(end) = after.find("*/") else {
            return Err(ThemeError { line: line_at(css, css.len() - rest.len() + i), message: "a comment that never ends".into() });
        };
        for c in after[..end].chars().chain("    ".chars()) {
            out.push(if c == '\n' { '\n' } else { ' ' });
        }
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    Ok(out)
}

struct Block {
    selectors: String,
    body: String,
    line: usize,
    body_line: usize,
}

fn split_blocks(css: &str) -> Result<Vec<Block>, ThemeError> {
    let mut out = Vec::new();
    let mut pos = 0;
    while pos < css.len() {
        let rest = &css[pos..];
        let Some(open) = rest.find('{') else {
            if !rest.trim().is_empty() {
                let skip = rest.len() - rest.trim_start().len();
                return Err(ThemeError { line: line_at(css, pos + skip), message: format!("{:?} has no {{ ... }}", rest.trim()) });
            }
            break;
        };
        let Some(close) = rest[open..].find('}') else {
            return Err(ThemeError { line: line_at(css, pos + open), message: "a { that is never closed".into() });
        };
        let close = open + close;
        let selectors = &rest[..open];
        if selectors.contains('}') {
            return Err(ThemeError { line: line_at(css, pos), message: "a } with no {".into() });
        }
        let body = &rest[open + 1..close];
        if body.contains('{') {
            return Err(ThemeError { line: line_at(css, pos + open), message: "rules do not nest here".into() });
        }
        let skip = selectors.len() - selectors.trim_start().len();
        if selectors.trim().is_empty() {
            return Err(ThemeError { line: line_at(css, pos + open), message: "a block with no selector".into() });
        }
        out.push(Block { selectors: selectors.to_string(), body: body.to_string(), line: line_at(css, pos + skip), body_line: line_at(css, pos + open) });
        pos += close + 1;
    }
    Ok(out)
}

/// `name: value;` pairs with the line each is on.
fn declarations(body: &str, first_line: usize) -> Result<Vec<(String, String, usize)>, ThemeError> {
    let mut out = Vec::new();
    let mut offset = 0;
    for part in body.split(';') {
        let line = first_line + body[..offset].matches('\n').count() + part[..part.len() - part.trim_start().len()].matches('\n').count();
        offset += part.len() + 1;
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        let Some((name, value)) = p.split_once(':') else {
            return Err(ThemeError { line, message: format!("{p:?} is not name: value") });
        };
        let (name, value) = (name.trim().to_ascii_lowercase(), value.trim().to_string());
        if name.is_empty() || value.is_empty() {
            return Err(ThemeError { line, message: format!("{p:?} is not name: value") });
        }
        out.push((name, value, line));
    }
    Ok(out)
}

fn is_ident(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') && !s.starts_with(|c: char| c.is_ascii_digit())
}

fn parse_selector(s: &str, line: usize) -> Result<Selector, ThemeError> {
    let bad = |why: &str| ThemeError { line, message: format!("selector {s:?}: {why}") };
    if s.is_empty() {
        return Err(bad("empty"));
    }
    if s.contains(char::is_whitespace) || s.contains('>') || s.contains('+') || s.contains('~') {
        return Err(bad("one control only - no descendants or combinators"));
    }
    let mut sel = Selector { kind: None, classes: vec![], id: None, states: vec![] };
    // Split before every . # : keeping the mark.
    let mut parts: Vec<&str> = Vec::new();
    let mut start = 0;
    for (i, c) in s.char_indices() {
        if i > 0 && matches!(c, '.' | '#' | ':') {
            parts.push(&s[start..i]);
            start = i;
        }
    }
    parts.push(&s[start..]);
    for (i, p) in parts.iter().enumerate() {
        if let Some(c) = p.strip_prefix('.') {
            if !is_ident(c) {
                return Err(bad("a class is letters, digits, - and _"));
            }
            sel.classes.push(c.to_string());
        } else if let Some(id) = p.strip_prefix('#') {
            if !is_ident(id) || sel.id.is_some() {
                return Err(bad("one #id of letters, digits, - and _"));
            }
            sel.id = Some(id.to_string());
        } else if let Some(st) = p.strip_prefix(':') {
            sel.states.push(match st {
                "hover" => Pseudo::Hover,
                "pressed" | "active" => Pseudo::Pressed,
                "focus" => Pseudo::Focus,
                "disabled" => Pseudo::Disabled,
                "selected" => Pseudo::Selected,
                "checked" => Pseudo::Checked,
                _ => return Err(bad("states are :hover :pressed :focus :disabled :selected :checked")),
            });
        } else if i == 0 {
            if *p != "*" {
                if !is_ident(p) {
                    return Err(bad("a control kind is letters, digits, - and _"));
                }
                sel.kind = Some(p.to_ascii_lowercase());
            }
        } else {
            return Err(bad("cannot read it"));
        }
    }
    Ok(sel)
}

fn resolve_vars(raw: &str, vars: &HashMap<String, String>, line: usize, depth: usize) -> Result<String, ThemeError> {
    if depth > 8 {
        return Err(ThemeError { line, message: "variables that refer to each other in a circle".into() });
    }
    let Some(i) = raw.find("var(") else { return Ok(raw.to_string()) };
    let after = &raw[i + 4..];
    let Some(end) = after.find(')') else {
        return Err(ThemeError { line, message: format!("{raw:?}: var( is never closed") });
    };
    let inner = &after[..end];
    let (name, fallback) = match inner.split_once(',') {
        Some((n, f)) => (n.trim(), Some(f.trim())),
        None => (inner.trim(), None),
    };
    let value = match (vars.get(name), fallback) {
        (Some(v), _) => resolve_vars(v, vars, line, depth + 1)?,
        (None, Some(f)) => f.to_string(),
        (None, None) => return Err(ThemeError { line, message: format!("{name} is not set in :root") }),
    };
    let joined = format!("{}{}{}", &raw[..i], value, &after[end + 1..]);
    resolve_vars(&joined, vars, line, depth + 1)
}

fn parse_value(prop: Prop, text: &str) -> Option<Value> {
    let t = text.trim();
    if prop.is_color() {
        return parse_color(t).map(Value::Color);
    }
    if prop == Prop::FontWeight {
        match t {
            "normal" => return Some(Value::Number(400)),
            "bold" => return Some(Value::Number(700)),
            _ => {}
        }
    }
    let n = t.strip_suffix("px").or_else(|| t.strip_suffix("pt")).unwrap_or(t).trim();
    n.parse::<i32>().ok().filter(|n| *n >= 0).map(Value::Number)
}

pub fn parse_color(t: &str) -> Option<Color> {
    let t = t.trim().to_ascii_lowercase();
    match t.as_str() {
        "transparent" => return Some(Color::TRANSPARENT),
        "white" => return Some(Color::rgb(255, 255, 255)),
        "black" => return Some(Color::rgb(0, 0, 0)),
        _ => {}
    }
    if let Some(hex) = t.strip_prefix('#') {
        let d = |s: &str| u8::from_str_radix(s, 16).ok();
        return match hex.len() {
            3 => {
                let v: Vec<u8> = hex.chars().map(|c| c.to_digit(16).map(|n| n as u8 * 17)).collect::<Option<_>>()?;
                Some(Color::rgb(v[0], v[1], v[2]))
            }
            6 => Some(Color::rgb(d(&hex[0..2])?, d(&hex[2..4])?, d(&hex[4..6])?)),
            8 => Some(Color { r: d(&hex[0..2])?, g: d(&hex[2..4])?, b: d(&hex[4..6])?, a: d(&hex[6..8])? }),
            _ => None,
        };
    }
    if let Some(inner) = t.strip_prefix("rgb(").and_then(|r| r.strip_suffix(')')) {
        let v: Vec<u8> = inner.split(',').map(|p| p.trim().parse::<u8>().ok()).collect::<Option<_>>()?;
        if v.len() == 3 {
            return Some(Color::rgb(v[0], v[1], v[2]));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q<'a>(kind: &'a str, classes: &'a [&'a str], id: Option<&'a str>, state: State) -> Query<'a> {
        Query { kind, classes, id, state }
    }

    #[test]
    fn colours_read_every_way_the_sheet_may_write_them() {
        assert_eq!(parse_color("#12855f"), Some(Color::rgb(0x12, 0x85, 0x5f)));
        assert_eq!(parse_color("#fff"), Some(Color::rgb(255, 255, 255)));
        assert_eq!(parse_color("rgb(1, 2, 3)"), Some(Color::rgb(1, 2, 3)));
        assert_eq!(parse_color("#00000080").map(|c| c.a), Some(0x80));
        assert_eq!(parse_color("transparent"), Some(Color::TRANSPARENT));
        assert_eq!(parse_color("#12"), None);
        assert_eq!(parse_color("green"), None, "only the few names listed, so a typo is an error");
    }

    #[test]
    fn the_more_specific_rule_wins_then_the_later_one() {
        let t = Theme::parse(
            "button { background: #111; color: #222; }
             button:hover { background: #333; }
             button.primary { background: #444; }
             #save { background: #555; }
             button { color: #666; }",
        )
        .unwrap();
        let plain = t.style(&q("button", &[], None, State::default()));
        assert_eq!(plain.background, Some(Color::rgb(0x11, 0x11, 0x11)));
        assert_eq!(plain.color, Some(Color::rgb(0x66, 0x66, 0x66)), "same specificity: the later rule");
        let hover = State { hover: true, ..Default::default() };
        assert_eq!(t.style(&q("button", &[], None, hover)).background, Some(Color::rgb(0x33, 0x33, 0x33)));
        assert_eq!(t.style(&q("button", &["primary"], None, State::default())).background, Some(Color::rgb(0x44, 0x44, 0x44)));
        assert_eq!(t.style(&q("button", &["primary"], Some("save"), hover)).background, Some(Color::rgb(0x55, 0x55, 0x55)), "an id beats classes and states");
        assert_eq!(t.style(&q("checkbox", &[], None, State::default())).background, None, "another kind is untouched");
    }

    #[test]
    fn variables_come_from_root_with_fallbacks_and_may_be_used_before_they_are_set() {
        let t = Theme::parse(
            "button { background: var(--accent); color: var(--missing, #fff); }
             :root { --green: #12855f; --accent: var(--green); }",
        )
        .unwrap();
        let s = t.style(&Query::kind("button"));
        assert_eq!(s.background, Some(Color::rgb(0x12, 0x85, 0x5f)));
        assert_eq!(s.color, Some(Color::rgb(255, 255, 255)));
        assert_eq!(t.var_color("--accent"), Some(Color::rgb(0x12, 0x85, 0x5f)));
    }

    #[test]
    fn comma_selectors_star_and_numbers() {
        let t = Theme::parse("button, checkbox:checked { border-radius: 4px; font-size: 9pt; font-weight: bold } * { border-width: 1 }").unwrap();
        assert_eq!(t.style(&Query::kind("button")).border_radius, Some(4));
        assert_eq!(t.style(&Query::kind("checkbox")).border_radius, None);
        let checked = State { checked: true, ..Default::default() };
        let s = t.style(&q("checkbox", &[], None, checked));
        assert_eq!((s.border_radius, s.font_size, s.font_weight, s.border_width), (Some(4), Some(9), Some(700), Some(1)));
    }

    #[test]
    fn a_mistake_is_an_error_on_its_line_never_a_rule_quietly_dropped() {
        let err = |css: &str| Theme::parse(css).unwrap_err();
        assert_eq!(err("button {\n  colour: #fff;\n}").line, 2);
        assert!(err("button {\n  colour: #fff;\n}").message.contains("unknown property"));
        assert_eq!(err("\n\nbutton { color: #ff; }").line, 3);
        assert!(err("button:hovered { color: #fff }").message.contains("states"));
        assert!(err("window button { color: #fff }").message.contains("descendants"));
        assert!(err("button { color: var(--nope) }").message.contains("--nope"));
        assert!(err("button { color: #fff").message.contains("never closed"));
        assert!(err(":root { --a: var(--b); --b: var(--a); } button { color: var(--a) }").message.contains("circle"));
        assert!(err("/* open").message.contains("comment"));
    }

    #[test]
    fn comments_keep_line_numbers() {
        let e = Theme::parse("/* one\n two */\nbutton { bogus: 1 }").unwrap_err();
        assert_eq!(e.line, 3);
    }

    #[test]
    fn the_built_in_themes_read_and_say_light_and_dark() {
        let light = Theme::light();
        let dark = Theme::dark();
        assert!(!light.is_dark());
        assert!(dark.is_dark());
        // Every control the library draws is styled by both, so neither falls back to a guess.
        for kind in ["window", "button", "checkbox", "radio", "edit", "label", "list", "group", "statusbar", "toolbar", "tabs", "combo"] {
            for t in [&light, &dark] {
                let s = t.style(&Query::kind(kind));
                assert!(s.background.is_some() || s.color.is_some() || s.border_color.is_some() || s.icon_color.is_some(), "{kind} has no style");
            }
        }
        let primary = light.style(&q("button", &["primary"], None, State::default()));
        assert_eq!(primary.background, Some(Color::rgb(0x12, 0x85, 0x5f)), "Phai green");
    }

    #[test]
    fn an_import_brings_a_built_in_theme_and_later_variables_recolour_it() {
        let t = Theme::parse("@import \"phai:light\";\n:root { --accent: #ff0000; }\nbutton { border-radius: 12px; }").unwrap();
        let primary = t.style(&Query { kind: "button", classes: &["primary"], id: None, state: State::default() });
        assert_eq!(primary.background, Some(Color::rgb(255, 0, 0)), "the imported rule takes the new --accent");
        assert_eq!(primary.border_radius, Some(12), "and the sheet's own rule is added");
        assert!(!t.is_dark());
        let e = Theme::parse("@import \"mine.css\";").unwrap_err();
        assert!(e.message.contains("mine.css"), "{e}");
        let e = Theme::parse_with("@import \"a.css\";", &|_| Ok("button {\n  bogus: 1; }".into())).unwrap_err();
        assert!(e.message.starts_with("a.css:") && e.line == 2, "an imported sheet's mistake names it: {e}");
        let e = Theme::parse_with("@import \"a.css\";", &|_| Ok("@import \"a.css\";".into())).unwrap_err();
        assert!(e.message.contains("too deep"), "{e}");
    }

    #[test]
    fn the_standard_sheet_names_every_kind_and_every_property() {
        for word in ["window", "label", "button", "checkbox", "radio", "edit", "combo", "list", "group", "tabs", "toolbar", "statusbar", "menubar", "canvas", ":hover", ":pressed", ":focus", ":disabled", ":selected", ":checked", "background", "border-radius", "accent-color", "icon-color", "@import"] {
            assert!(STANDARD_CSS.contains(word), "the standard sheet should say {word}");
        }
        assert!(!Theme::parse(STANDARD_CSS).unwrap().is_dark());
    }
}
