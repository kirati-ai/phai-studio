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
    /// Points for every control's words; none is Windows' own (9 pt). A control may set its own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<i32>,
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
    Date,
    Number,
    Progress,
    Slider,
    Textarea,
    /// A tree of items, indented lines.
    Tree,
    /// A picture from a file.
    Picture,
    /// A component, not a control: something happens every so many milliseconds.
    Timer,
    /// A panel that lays the controls in it out left to right, row after row.
    Flow,
    /// A panel that lays the controls in it out in equal columns.
    Table,
    /// Text with many lines - Windows' RichEdit.
    #[serde(rename = "richtext")]
    Rich,
    /// A grid of cells to change in place: columns, then rows the program adds.
    Grid,
    /// A component: the menu a right-click on a control opens - its items `id | label`.
    #[serde(rename = "contextmenu")]
    ContextMenu,
}

impl Kind {
    pub const ALL: [Kind; 23] = [Kind::Label, Kind::Edit, Kind::Textarea, Kind::Rich, Kind::Button, Kind::Checkbox, Kind::Radio, Kind::Combo, Kind::List, Kind::Grid, Kind::Tree, Kind::Picture, Kind::Date, Kind::Number, Kind::Slider, Kind::Progress, Kind::Group, Kind::Tabs, Kind::Flow, Kind::Table, Kind::Statusbar, Kind::Timer, Kind::ContextMenu];

    /// A container that places the controls in it itself.
    pub fn arranges(self) -> bool {
        matches!(self, Kind::Flow | Kind::Table)
    }

    /// A component: nothing on the form itself - Visual Studio puts it in the tray below.
    pub fn is_component(self) -> bool {
        matches!(self, Kind::Timer | Kind::ContextMenu)
    }

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
            Kind::Date => "date",
            Kind::Number => "number",
            Kind::Progress => "progress",
            Kind::Slider => "slider",
            Kind::Tree => "tree",
            Kind::Picture => "picture",
            Kind::Timer => "timer",
            Kind::Flow => "flow",
            Kind::Table => "table",
            Kind::Rich => "richtext",
            Kind::Grid => "grid",
            Kind::ContextMenu => "contextmenu",
            Kind::Textarea => "textarea",
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
            Kind::Date => (150, 23),
            Kind::Number => (80, 23),
            Kind::Progress => (200, 20),
            Kind::Slider => (200, 30),
            Kind::Tree => (200, 150),
            Kind::Picture => (120, 90),
            Kind::Timer => (32, 32),
            Kind::Flow | Kind::Table => (240, 120),
            Kind::Rich => (240, 120),
            Kind::Grid => (300, 150),
            Kind::ContextMenu => (32, 32),
            Kind::Textarea => (240, 90),
        }
    }

    /// The handler a control of this kind has, if any.
    pub fn event(self) -> Option<&'static str> {
        match self {
            Kind::Button | Kind::Checkbox | Kind::Radio => Some("on_click"),
            Kind::Edit | Kind::Textarea | Kind::Combo | Kind::Tabs | Kind::Date | Kind::Number | Kind::Slider | Kind::Rich | Kind::Grid => Some("on_change"),
            Kind::List | Kind::Tree => Some("on_select"),
            Kind::Timer => Some("on_tick"),
            _ => None,
        }
    }

    pub fn has_text(self) -> bool {
        matches!(self, Kind::Label | Kind::Edit | Kind::Textarea | Kind::Rich | Kind::Button | Kind::Checkbox | Kind::Radio | Kind::Group)
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
    /// A picture's file, relative to the project (or the program) or absolute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// The context menu a right-click on this control opens: a contextmenu component's id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_menu: Option<String>,
    /// A table panel's number of columns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cols: Option<i32>,
    /// A timer's method, every interval.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_tick: Option<String>,
    /// Points for this control's words, over the form's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<i32>,
    /// Bold words - `#id { font-weight: bold }`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    /// Colours over the theme's, as CSS writes them (`#12855f`, `red`) - `#id { background }`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub back_color: Option<String>,
    /// `#id { color }`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fore_color: Option<String>,
    /// `#id { border-color }`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_color: Option<String>,
    /// Corner roundness, pixels - `#id { border-radius }`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<i32>,
    /// A number box's, slider's or progress bar's range and starting value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<i32>,
    /// Locked in the designer: not moved or sized by the mouse or the arrows. Design only - the
    /// program never sees it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
    /// A grid bound to a database table: its rows read when the window opens, written back by a
    /// button with `bind_action` - as a WinForms DataGridView with its BindingSource.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bind: Option<Bind>,
    /// A button that saves or reloads a bound grid: `"customers:save"`, `"customers:reload"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bind_action: Option<String>,
}

/// Which table a grid shows: its database (`main` for SQLite), the table, the columns in the
/// grid's order, and the primary key that finds a row again to change or delete it. The server
/// itself is the project's, in `src/data_sources.rs`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bind {
    #[serde(default)]
    pub database: String,
    pub table: String,
    pub columns: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keys: Vec<String>,
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
            font_size: None,
            bold: None,
            back_color: None,
            fore_color: None,
            border_color: None,
            radius: None,
            min: None,
            max: None,
            value: None,
            locked: None,
            bind: None,
            bind_action: None,
            image: None,
            on_tick: None,
            cols: None,
            context_menu: None,
        };
        match kind {
            Kind::Combo => c.items = vec!["One".into(), "Two".into()],
            Kind::Tree => c.items = vec!["Root".into(), "  First".into(), "  Second".into()],
            Kind::Timer => c.value = Some(1000),
            Kind::Table => c.cols = Some(2),
            Kind::Grid => c.columns = vec![("Name".into(), 140), ("Value".into(), 120)],
            Kind::ContextMenu => c.items = vec!["copy_clicked | Copy".into(), "paste_clicked | Paste".into()],
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

    /// The CSS declarations this control's own properties make - `background: #12855f; color: white`
    /// - as a `#id { ... }` rule would say them, over the theme. Nothing when it sets none.
    pub fn own_css(&self) -> Option<String> {
        let mut decls = Vec::new();
        for (name, v) in [("background", &self.back_color), ("color", &self.fore_color), ("border-color", &self.border_color)] {
            if let Some(v) = v {
                decls.push(format!("{name}: {v}"));
            }
        }
        if let Some(r) = self.radius {
            decls.push(format!("border-radius: {r}px"));
        }
        if self.bold == Some(true) {
            decls.push("font-weight: bold".into());
        }
        (!decls.is_empty()).then(|| decls.join("; "))
    }

    pub fn rect(&self) -> [i32; 4] {
        self.bounds.unwrap_or([0, 0, 0, 0])
    }

    /// The handler name of this control's one event, whichever field holds it.
    /// A context menu's items: (method, label), from its `id | label` lines; `-` a separator
    /// (an empty method).
    pub fn menu_items(&self) -> Vec<(String, String)> {
        self.items
            .iter()
            .map(|l| match l.split_once('|') {
                Some((id, label)) => (id.trim().to_string(), label.trim().to_string()),
                None if l.trim() == "-" => (String::new(), String::new()),
                None => (l.trim().to_string(), l.trim().to_string()),
            })
            .collect()
    }

    pub fn handler(&self) -> Option<&str> {
        match self.kind.event()? {
            "on_click" => self.on_click.as_deref(),
            "on_change" => self.on_change.as_deref(),
            "on_tick" => self.on_tick.as_deref(),
            _ => self.on_select.as_deref(),
        }
    }

    pub fn set_handler(&mut self, name: Option<String>) {
        match self.kind.event() {
            Some("on_click") => self.on_click = name,
            Some("on_change") => self.on_change = name,
            Some("on_tick") => self.on_tick = name,
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
        FormFile { format: FORMAT, form: Form { name: name.to_string(), title: name.to_string(), size: [480, 320], min_size: None, theme: None, font_size: None }, menu: vec![], toolbar: vec![], control: vec![] }
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
        let font_ok = |f: Option<i32>| f.is_none_or(|n| (6..=48).contains(&n));
        if !font_ok(self.form.font_size) {
            out.push(("font_size: 6 to 48 points".into(), Some("font_size".into())));
        }
        if let Some(c) = self.control.iter().find(|c| !font_ok(c.font_size)) {
            out.push((format!("{}: font_size 6 to 48 points", c.id), Some(format!("id = \"{}\"", c.id))));
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
            if !docked && c.bounds.is_none() && !c.kind.is_component() {
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
            out.extend(wrong("items", c.items.is_empty() || matches!(c.kind, Kind::Combo | Kind::Tree | Kind::ContextMenu)));
            if let Some(m) = &c.context_menu {
                if !self.control.iter().any(|x| x.kind == Kind::ContextMenu && &x.id == m) {
                    out.push((format!("{}: no context menu {m}", c.id), at.clone()));
                }
            }
            out.extend(wrong("image", c.image.is_none() || c.kind == Kind::Picture));
            out.extend(wrong("cols", c.cols.is_none() || c.kind == Kind::Table));
            out.extend(wrong("columns", c.columns.is_empty() || matches!(c.kind, Kind::List | Kind::Grid)));
            out.extend(wrong("tabs", c.tabs.is_empty() || c.kind == Kind::Tabs));
            out.extend(wrong("bind", c.bind.is_none() || c.kind == Kind::Grid));
            out.extend(wrong("bind_action", c.bind_action.is_none() || c.kind == Kind::Button));
            if let Some(b) = &c.bind {
                if b.table.is_empty() || b.columns.is_empty() {
                    out.push((format!("{}: bind needs a table and its columns", c.id), at.clone()));
                }
                if b.columns.len() != c.columns.len() {
                    out.push((format!("{}: bind has {} columns, the grid {}", c.id, b.columns.len(), c.columns.len()), at.clone()));
                }
                if let Some(k) = b.keys.iter().find(|k| !b.columns.contains(k)) {
                    out.push((format!("{}: the key {k} is not among the bound columns", c.id), at.clone()));
                }
            }
            if let Some(a) = &c.bind_action {
                let ok = a.split_once(':').is_some_and(|(g, what)| matches!(what, "save" | "reload") && self.control.iter().any(|x| x.id == g && x.bind.is_some()));
                if !ok {
                    out.push((format!("{}: bind_action {a:?} - a bound grid's id, then :save or :reload", c.id), at.clone()));
                }
            }
            out.extend(wrong("parts", c.parts.is_empty() || c.kind == Kind::Statusbar));
            out.extend(wrong("password", c.password.is_none() || c.kind == Kind::Edit));
            out.extend(wrong("checked", c.checked.is_none() || matches!(c.kind, Kind::Checkbox | Kind::Radio)));
            out.extend(wrong("group", c.group.is_none() || c.kind == Kind::Radio));
            let ranged = matches!(c.kind, Kind::Number | Kind::Progress | Kind::Slider);
            out.extend(wrong("min", c.min.is_none() || ranged));
            out.extend(wrong("max", c.max.is_none() || ranged));
            out.extend(wrong("value", c.value.is_none() || ranged || c.kind == Kind::Timer));
            if ranged && c.min.unwrap_or(0) > c.max.unwrap_or(100) {
                out.push((format!("{}: min is more than max", c.id), at.clone()));
            }
            for (field, v) in [("back_color", &c.back_color), ("fore_color", &c.fore_color), ("border_color", &c.border_color)] {
                if let Some(v) = v {
                    if phai_ui::theme::parse_color(v).is_none() {
                        out.push((format!("{field} {v:?} is not a colour - #rrggbb, rgb(...) or a CSS name"), at.clone()));
                    }
                }
            }
            if c.radius.is_some_and(|r| !(0..=50).contains(&r)) {
                out.push((format!("{}: radius 0 to 50", c.id), at.clone()));
            }
            if let Some(e) = c.kind.event() {
                for (name, v) in [("on_click", &c.on_click), ("on_change", &c.on_change), ("on_select", &c.on_select), ("on_tick", &c.on_tick)] {
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
            if c.kind == Kind::ContextMenu {
                for (id, _) in c.menu_items().into_iter().filter(|(id, _)| !id.is_empty()) {
                    add(&id);
                }
            }
        }
        out
    }

    /// The container a control lies in: the smallest group, tabs, flow or table, not docked,
    /// whose rectangle holds the control's - if any.
    pub fn container_of(&self, i: usize) -> Option<usize> {
        let c = self.control.get(i)?;
        if c.dock.as_deref().is_some_and(|d| d != "none") || c.kind.is_component() {
            return None;
        }
        let [x, y, w, h] = c.rect();
        self.control
            .iter()
            .enumerate()
            .filter(|(j, p)| *j != i && matches!(p.kind, Kind::Group | Kind::Tabs | Kind::Flow | Kind::Table) && p.dock.as_deref().is_none_or(|d| d == "none"))
            .filter(|(_, p)| {
                let [px, py, pw, ph] = p.rect();
                x >= px && y >= py && x + w <= px + pw && y + h <= py + ph && (pw, ph) != (w, h)
            })
            .min_by_key(|(_, p)| {
                let [_, _, pw, ph] = p.rect();
                pw as i64 * ph as i64
            })
            .map(|(j, _)| j)
    }

    /// The controls a flow or table panel lays out: those it directly holds, in the form's order.
    pub fn arranged_in(&self, panel: usize) -> Vec<usize> {
        (0..self.control.len()).filter(|i| self.container_of(*i) == Some(panel)).collect()
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
            // A bound grid's headings that are its table's own column names are shown as they are,
            // not translated; one renamed in the form is a sentence like any other.
            c.columns.iter().enumerate().filter(|(i, (s, _))| c.bind.as_ref().is_none_or(|b| b.columns.get(*i) != Some(s))).for_each(|(_, (s, _))| add(s));
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
    #[test]
    fn trees_pictures_and_timers_are_read_and_written() {
        let src = "format = 1
[form]
name = \"Main\"
title = \"T\"
size = [480, 320]

[[control]]
kind = \"tree\"
id = \"places\"
bounds = [11, 45, 200, 200]
items = [\"Thailand\", \"  Bangkok\"]
on_select = \"place_chosen\"

[[control]]
kind = \"picture\"
id = \"logo\"
bounds = [230, 45, 160, 120]
image = \"logo.png\"

[[control]]
kind = \"timer\"
id = \"clock\"
value = 1000
on_tick = \"clock_ticked\"
";
        let f = FormFile::parse(src).unwrap_or_else(|e| panic!("{e:?}"));
        assert_eq!(f.control[0].items, ["Thailand", "  Bangkok"], "a tree keeps its indent");
        assert_eq!(f.control[2].handler(), Some("clock_ticked"));
        assert_eq!(FormFile::parse(&f.to_toml()).unwrap_or_else(|e| panic!("{e:?}")), f);
    }

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

    #[test]
    fn a_controls_own_colours_are_css_and_checked() {
        let mut c = Control::new(Kind::Button, "ok", [0, 0, 75, 23]);
        assert_eq!(c.own_css(), None);
        c.back_color = Some("#12855f".into());
        c.fore_color = Some("white".into());
        c.radius = Some(6);
        c.bold = Some(true);
        assert_eq!(c.own_css().as_deref(), Some("background: #12855f; color: white; border-radius: 6px; font-weight: bold"));
        let mut f = FormFile::new("Main");
        f.control.push(c.clone());
        assert!(f.check().is_empty(), "{:?}", f.check());
        f.control[0].back_color = Some("blurple".into());
        assert!(f.check()[0].0.contains("blurple"));
        f.control[0].back_color = None;
        f.control[0].min = Some(1);
        assert!(f.check()[0].0.contains("min"), "a button has no range");
        let mut n = Control::new(Kind::Number, "count", [0, 0, 80, 23]);
        n.min = Some(10);
        n.max = Some(5);
        f.control = vec![n];
        assert!(f.check()[0].0.contains("more than max"));
    }
}
