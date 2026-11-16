//! What a `.phaiform` holds, read and written as TOML (see `docs/phaiform.md`). Every mistake is
//! an error with its line - a field nobody knows, a wrong value, two controls with one id -
//! because a form that half-loads is a form whose missing half someone loses on the next save.

use serde::{Deserialize, Serialize};

pub const FORMAT: u32 = 1;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FormFile {
    pub format: u32,
    pub form: Form,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub menu: Vec<Menu>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub toolbar: Vec<Tool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub control: Vec<Control>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Form {
    pub name: String,
    pub title: String,
    pub size: [i32; 2],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_size: Option<[i32; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Menu {
    pub label: String,
    #[serde(default)]
    pub items: Vec<MenuItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct MenuItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub separator: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Tool {
    pub id: String,
    pub label: String,
    pub glyph: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Label,
    Edit,
    Button,
    Checkbox,
    Radio,
    Combo,
    List,
    Group,
    Tabs,
    Statusbar,
}

impl Kind {
    pub const ALL: [Kind; 10] = [Kind::Label, Kind::Edit, Kind::Button, Kind::Checkbox, Kind::Radio, Kind::Combo, Kind::List, Kind::Group, Kind::Tabs, Kind::Statusbar];

    pub fn name(self) -> &'static str {
        match self {
            Kind::Label => "label",
            Kind::Edit => "edit",
            Kind::Button => "button",
            Kind::Checkbox => "checkbox",
            Kind::Radio => "radio",
            Kind::Combo => "combo",
            Kind::List => "list",
            Kind::Group => "group",
            Kind::Tabs => "tabs",
            Kind::Statusbar => "statusbar",
        }
    }

    /// The size a control gets when it is dropped with a click rather than drawn, in design
    /// pixels - Windows' own sizes.
    pub fn default_size(self) -> (i32, i32) {
        match self {
            Kind::Label => (60, 23),
            Kind::Edit => (150, 23),
            Kind::Button => (75, 23),
            Kind::Checkbox | Kind::Radio => (110, 20),
            Kind::Combo => (150, 23),
            Kind::List => (240, 150),
            Kind::Group => (240, 120),
            Kind::Tabs => (240, 30),
            Kind::Statusbar => (0, 26),
        }
    }

    /// The handler a control of this kind has, if any.
    pub fn event(self) -> Option<&'static str> {
        match self {
            Kind::Button | Kind::Checkbox | Kind::Radio => Some("on_click"),
            Kind::Edit | Kind::Combo | Kind::Tabs => Some("on_change"),
            Kind::List => Some("on_select"),
            _ => None,
        }
    }

    pub fn has_text(self) -> bool {
        matches!(self, Kind::Label | Kind::Edit | Kind::Button | Kind::Checkbox | Kind::Radio | Kind::Group)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Control {
    pub kind: Kind,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounds: Option<[i32; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dock: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub columns: Vec<(String, i32)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tabs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_click: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_change: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_select: Option<String>,
}

impl Control {
    /// A new control of `kind` at `bounds`, with nothing else set.
    pub fn new(kind: Kind, id: &str, bounds: [i32; 4]) -> Control {
        let text = match kind {
            Kind::Label => Some("Label".to_string()),
            Kind::Button => Some("Button".to_string()),
            Kind::Checkbox => Some("Check box".to_string()),
            Kind::Radio => Some("Option".to_string()),
            Kind::Group => Some("Group".to_string()),
            _ => None,
        };
        let mut c = Control {
            kind,
            id: id.to_string(),
            text,
            bounds: Some(bounds),
            anchor: None,
            dock: None,
            class: None,
            enabled: None,
            visible: None,
            checked: None,
            group: None,
            password: None,
            items: vec![],
            columns: vec![],
            tabs: vec![],
            page: None,
            parts: vec![],
            on_click: None,
            on_change: None,
            on_select: None,
        };
        match kind {
            Kind::Combo => c.items = vec!["One".into(), "Two".into()],
            Kind::List => c.columns = vec![("Name".into(), 120), ("Value".into(), 100)],
            Kind::Tabs => c.tabs = vec!["General".into(), "Advanced".into()],
            Kind::Statusbar => {
                c.dock = Some("bottom".into());
                c.text = Some("Ready".into());
            }
            _ => {}
        }
        c
    }

    pub fn rect(&self) -> [i32; 4] {
        self.bounds.unwrap_or([0, 0, 0, 0])
    }

    /// The handler name of this control's one event, whichever field holds it.
    pub fn handler(&self) -> Option<&str> {
        match self.kind.event()? {
            "on_click" => self.on_click.as_deref(),
            "on_change" => self.on_change.as_deref(),
            _ => self.on_select.as_deref(),
        }
    }

    pub fn set_handler(&mut self, name: Option<String>) {
        match self.kind.event() {
            Some("on_click") => self.on_click = name,
            Some("on_change") => self.on_change = name,
            Some(_) => self.on_select = name,
            None => {}
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Anchors {
    pub left: bool,
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
}

/// `"left top right"`, `"all"`, `"none"`; nothing is `left top`.
pub fn parse_anchor(s: Option<&str>) -> Result<Anchors, String> {
    let Some(s) = s else { return Ok(Anchors { left: true, top: true, ..Default::default() }) };
    let mut a = Anchors::default();
    for w in s.split_whitespace() {
        match w {
            "left" => a.left = true,
            "top" => a.top = true,
            "right" => a.right = true,
            "bottom" => a.bottom = true,
            "all" => a = Anchors { left: true, top: true, right: true, bottom: true },
            "none" => {}
            _ => return Err(format!("anchor {w:?}: left, top, right, bottom, all or none")),
        }
    }
    Ok(a)
}

pub fn anchor_text(a: Anchors) -> Option<String> {
    match (a.left, a.top, a.right, a.bottom) {
        (true, true, false, false) => None,
        (true, true, true, true) => Some("all".into()),
        (false, false, false, false) => Some("none".into()),
        _ => {
            let words: Vec<&str> = [(a.left, "left"), (a.top, "top"), (a.right, "right"), (a.bottom, "bottom")].iter().filter(|(on, _)| *on).map(|(_, w)| *w).collect();
            Some(words.join(" "))
        }
    }
}

pub const DOCKS: [&str; 6] = ["none", "top", "bottom", "left", "right", "fill"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for FormError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line == 0 {
            write!(f, "{}", self.message)
        } else {
            write!(f, "line {}: {}", self.line, self.message)
        }
    }
}

impl std::error::Error for FormError {}

fn is_snake(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') && !s.starts_with(|c: char| c.is_ascii_digit())
}

fn is_pascal(s: &str) -> bool {
    s.starts_with(|c: char| c.is_ascii_uppercase()) && s.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Rust's keywords, which cannot be field or method names.
const KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "gen",
];

/// The line in `src` where `needle` appears, for errors found after parsing - the last place when
/// it starts with [`LAST`], as a second control with a taken id is the one in the wrong.
fn line_of(src: &str, needle: &str) -> usize {
    let found = match needle.strip_prefix(LAST) {
        Some(n) => src.rfind(n),
        None => src.find(needle),
    };
    found.map(|i| src[..i].matches('\n').count() + 1).unwrap_or(0)
}

const LAST: &str = "\u{1}";

impl FormFile {
    /// A new empty form.
    pub fn new(name: &str) -> FormFile {
        FormFile { format: FORMAT, form: Form { name: name.to_string(), title: name.to_string(), size: [480, 320], min_size: None, theme: None }, menu: vec![], toolbar: vec![], control: vec![] }
    }

    pub fn parse(src: &str) -> Result<FormFile, FormError> {
        let f: FormFile = toml::from_str(src).map_err(|e| FormError { line: e.span().map(|s| src[..s.start.min(src.len())].matches('\n').count() + 1).unwrap_or(0), message: e.message().to_string() })?;
        let problems = f.check();
        if let Some(p) = problems.into_iter().next() {
            let line = p.1.as_deref().map(|n| line_of(src, n)).unwrap_or(0);
            return Err(FormError { line, message: p.0 });
        }
        Ok(f)
    }

    pub fn to_toml(&self) -> String {
        let body = toml::to_string(self).unwrap_or_default();
        format!("# Phai Studio form - docs/phaiform.md\n{body}")
    }

    /// Everything wrong with the form, each with text to find its line by. Empty when it is good.
    pub fn check(&self) -> Vec<(String, Option<String>)> {
        let mut out = Vec::new();
        if self.format != FORMAT {
            out.push((format!("format {} - this Phai Studio reads format {FORMAT}", self.format), Some("format".into())));
        }
        if !is_pascal(&self.form.name) {
            out.push((format!("form name {:?} must be PascalCase, like MainWindow", self.form.name), Some("name".into())));
        }
        if self.form.size[0] < 50 || self.form.size[1] < 30 {
            out.push(("form size is too small".into(), Some("size".into())));
        }
        let mut ids = std::collections::HashSet::new();
        let mut handlers = std::collections::HashSet::new();
        for c in &self.control {
            let at = Some(format!("id = \"{}\"", c.id));
            if !is_snake(&c.id) || KEYWORDS.contains(&c.id.as_str()) {
                out.push((format!("id {:?}: snake_case letters, digits and _, not a Rust keyword", c.id), at.clone()));
            }
            if !ids.insert(c.id.clone()) {
                out.push((format!("two controls are called {:?}", c.id), Some(format!("{LAST}id = \"{}\"", c.id))));
            }
            if let Err(e) = parse_anchor(c.anchor.as_deref()) {
                out.push((e, at.clone()));
            }
            if let Some(d) = &c.dock {
                if !DOCKS.contains(&d.as_str()) {
                    out.push((format!("dock {d:?}: none, top, bottom, left, right or fill"), at.clone()));
                }
            }
            let docked = c.dock.as_deref().is_some_and(|d| d != "none");
            if !docked && c.bounds.is_none() {
                out.push((format!("{} has no bounds", c.id), at.clone()));
            }
            if let Some([_, _, w, h]) = c.bounds {
                if w < 0 || h < 0 {
                    out.push((format!("{} has a negative size", c.id), at.clone()));
                }
            }
            for h in [&c.on_click, &c.on_change, &c.on_select].into_iter().flatten() {
                if !is_snake(h) || KEYWORDS.contains(&h.as_str()) {
                    out.push((format!("handler {h:?}: snake_case, not a Rust keyword"), Some(h.clone())));
                }
                handlers.insert(h.clone());
            }
            let wrong = |field: &str, ok: bool| (!ok).then(|| (format!("{field} does not belong on a {}", c.kind.name()), at.clone()));
            out.extend(wrong("items", c.items.is_empty() || c.kind == Kind::Combo));
            out.extend(wrong("columns", c.columns.is_empty() || c.kind == Kind::List));
            out.extend(wrong("tabs", c.tabs.is_empty() || c.kind == Kind::Tabs));
            out.extend(wrong("parts", c.parts.is_empty() || c.kind == Kind::Statusbar));
            out.extend(wrong("password", c.password.is_none() || c.kind == Kind::Edit));
            out.extend(wrong("checked", c.checked.is_none() || matches!(c.kind, Kind::Checkbox | Kind::Radio)));
            out.extend(wrong("group", c.group.is_none() || c.kind == Kind::Radio));
            if let Some(e) = c.kind.event() {
                for (name, v) in [("on_click", &c.on_click), ("on_change", &c.on_change), ("on_select", &c.on_select)] {
                    if v.is_some() && name != e {
                        out.push((format!("a {} has {e}, not {name}", c.kind.name()), at.clone()));
                    }
                }
            } else if c.on_click.is_some() || c.on_change.is_some() || c.on_select.is_some() {
                out.push((format!("a {} has no events", c.kind.name()), at.clone()));
            }
        }
        for m in &self.menu {
            for it in &m.items {
                if it.separator {
                    continue;
                }
                match (&it.id, &it.label) {
                    (Some(id), Some(_)) => {
                        if !is_snake(id) || KEYWORDS.contains(&id.as_str()) {
                            out.push((format!("menu id {id:?}: snake_case, not a Rust keyword"), Some(format!("id = \"{id}\""))));
                        }
                        if !ids.insert(id.clone()) && !handlers.contains(id) {
                            out.push((format!("two things are called {id:?}"), Some(format!("id = \"{id}\""))));
                        }
                    }
                    _ => out.push((format!("a menu item in {:?} needs an id and a label", m.label), Some(m.label.clone()))),
                }
            }
        }
        for t in &self.toolbar {
            if phai_ui::icon::glyph::by_name(&t.glyph).is_none() {
                out.push((format!("glyph {:?} is not one phai-ui knows", t.glyph), Some(t.glyph.clone())));
            }
            if !is_snake(&t.id) {
                out.push((format!("toolbar id {:?}: snake_case", t.id), Some(t.id.clone())));
            }
        }
        out
    }

    /// An id for a new control of `kind`: `button1`, `button2`...
    pub fn next_id(&self, kind: Kind) -> String {
        (1..).map(|n| format!("{}{n}", kind.name())).find(|id| self.control.iter().all(|c| &c.id != id)).unwrap()
    }

    /// Every handler the form names, in order, once each: menu and toolbar ids, then controls'.
    pub fn handlers(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut add = |s: &str| {
            if !out.iter().any(|x| x == s) {
                out.push(s.to_string());
            }
        };
        for m in &self.menu {
            for it in &m.items {
                if let Some(id) = &it.id {
                    add(id);
                }
            }
        }
        for t in &self.toolbar {
            add(&t.id);
        }
        for c in &self.control {
            if let Some(h) = c.handler() {
                add(h);
            }
        }
        out
    }

    /// Every English sentence the form shows, for the project's Thai table and its test.
    pub fn texts(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut add = |s: &str| {
            if !s.is_empty() && !out.iter().any(|x| x == s) {
                out.push(s.to_string());
            }
        };
        add(&self.form.title);
        for m in &self.menu {
            add(&m.label);
            for it in &m.items {
                if let Some(l) = &it.label {
                    add(l);
                }
            }
        }
        for t in &self.toolbar {
            add(&t.label);
        }
        for c in &self.control {
            if c.kind != Kind::Edit {
                if let Some(t) = &c.text {
                    add(t);
                }
            }
            c.items.iter().for_each(|s| add(s));
            c.columns.iter().for_each(|(s, _)| add(s));
            c.tabs.iter().for_each(|s| add(s));
        }
        out
    }
}

/// `MainWindow` -> `main_window`.
pub fn snake(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    pub const EXAMPLE: &str = include_str!("../../../apps/sample/src/main_window.phaiform");

    #[test]
    fn the_example_form_reads_and_writes_back_the_same() {
        let f = FormFile::parse(EXAMPLE).unwrap();
        assert_eq!(f.form.name, "MainWindow");
        assert!(f.control.iter().any(|c| c.kind == Kind::List));
        let again = FormFile::parse(&f.to_toml()).unwrap();
        assert_eq!(again, f, "saving loses nothing");
    }

    #[test]
    fn mistakes_are_errors_on_their_line() {
        let base = "format = 1\n[form]\nname = \"Main\"\ntitle = \"x\"\nsize = [300, 200]\n";
        let err = |extra: &str| FormFile::parse(&format!("{base}{extra}")).unwrap_err();
        let e = err("[[control]]\nkind = \"button\"\nid = \"ok\"\nbounds = [1, 2, 3, 4]\ncolour = \"red\"\n");
        assert_eq!(e.line, 10, "{e}");
        assert!(e.message.contains("colour"), "{e}");
        let e = err("[[control]]\nkind = \"knob\"\nid = \"k\"\nbounds = [1, 2, 3, 4]\n");
        assert!(e.message.contains("knob"), "{e}");
        let e = err("[[control]]\nkind = \"label\"\nid = \"a\"\nbounds = [1, 2, 3, 4]\n[[control]]\nkind = \"label\"\nid = \"a\"\nbounds = [1, 2, 3, 4]\n");
        assert!(e.message.contains("two controls"), "{e}");
        assert_eq!(e.line, 12, "the second one is the mistake");
        let e = err("[[control]]\nkind = \"label\"\nid = \"fn\"\nbounds = [1, 2, 3, 4]\n");
        assert!(e.message.contains("keyword"), "{e}");
        let e = err("[[control]]\nkind = \"label\"\nid = \"a\"\nbounds = [1, 2, 3, 4]\non_click = \"go\"\n");
        assert!(e.message.contains("no events"), "{e}");
        let e = err("[[control]]\nkind = \"edit\"\nid = \"a\"\nbounds = [1, 2, 3, 4]\nanchor = \"middle\"\n");
        assert!(e.message.contains("middle"), "{e}");
        let e = err("[[control]]\nkind = \"edit\"\nid = \"a\"\n");
        assert!(e.message.contains("bounds"), "{e}");
    }

    #[test]
    fn anchors_read_and_write() {
        assert_eq!(parse_anchor(None).unwrap(), Anchors { left: true, top: true, right: false, bottom: false });
        let a = parse_anchor(Some("right bottom")).unwrap();
        assert_eq!(anchor_text(a).as_deref(), Some("right bottom"));
        assert_eq!(anchor_text(parse_anchor(Some("left top")).unwrap()), None, "the default is not written");
        assert_eq!(anchor_text(parse_anchor(Some("all")).unwrap()).as_deref(), Some("all"));
    }

    #[test]
    fn names_ids_handlers_and_texts() {
        assert_eq!(snake("MainWindow"), "main_window");
        assert_eq!(snake("Form2"), "form2");
        let mut f = FormFile::new("Main");
        f.control.push(Control::new(Kind::Button, "button1", [0, 0, 75, 23]));
        assert_eq!(f.next_id(Kind::Button), "button2");
        assert_eq!(f.next_id(Kind::Edit), "edit1");
        f.control[0].set_handler(Some("go".into()));
        assert_eq!(f.handlers(), vec!["go".to_string()]);
        assert!(f.texts().contains(&"Button".to_string()));
    }
}
