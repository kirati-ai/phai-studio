//! The properties pane, as Visual Studio's: an object chooser (its name in bold, its kind beside),
//! a row of icons - Categorized, Alphabetical, Properties, Events - and a grid of name and value,
//! in categories that fold, or alphabetical. A value set apart from its default shows in bold;
//! Location, Size and Font fold open into their parts. Each value has the editor its kind
//! wants: a box for words and numbers, a list for a choice, a click for true or false, a picker
//! for anchors, Windows' colour dialog for colours, "…" for lists and files. Below, what the chosen
//! property is for.

use phai_form::props::{self as table, Spec, Ty};
use phai_form::{anchor_text, parse_anchor, Anchors, HasProps, Kind, DOCKS};
use phai_ui::text::{t, tf};
use phai_ui::theme::{Color, State};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HFONT};
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HMENU};

use crate::studio;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    FormName,
    FormTitle,
    FormWidth,
    FormHeight,
    FormTheme,
    FormFont,
    FormStart,
    FormState,
    Id,
    Text,
    X,
    Y,
    W,
    H,
    Class,
    Event,
    Items,
    Font,
    Bold,
    BackColor,
    ForeColor,
    BorderColor,
    Radius,
    Enabled,
    Visible,
    Locked,
    Image,
    Cols,
    ContextMenu,
    Checked,
    Password,
    Group,
    Dock,
    Anchor,
    Min,
    Max,
    Value,
    /// "x, y" - folds open into X and Y.
    Location,
    /// "w, h" - folds open into Width and Height.
    Size,
    /// The form's "w, h".
    FormSize,
    /// "Segoe UI, 9pt, Bold" - folds open into Font size and Bold.
    FontAll,
    /// A control's property from phai-form's table, by its name.
    More(&'static str),
    /// The form's, from the table.
    FormMore(&'static str),
    /// A control's event from phai-form's events table, by its name.
    On(&'static str),
    /// The form's event.
    FormOn(&'static str),
    /// The Style tab's state, shown on the canvas - not the model's.
    StyleState,
    /// A CSS property of the chosen control in a state (0 normal, then `phai_form::STATES`).
    StyleProp(&'static str, u8),
    /// The chosen control's transition: milliseconds, and its curve.
    StyleTransition,
    StyleEasing,
    /// A property of the chosen menu, menu item or toolbar button, by its name.
    Bar(&'static str),
    /// Its look moved into the theme as a class.
    StyleToTheme,
}

/// The CSS properties the Style tab shows, in its order.
pub const STYLE_PROPS: [&str; 10] = ["background", "background-image", "box-shadow", "color", "border-color", "border-width", "border-radius", "font-weight", "accent-color", "icon-color"];

/// The Style tab's states: normal, then the model's.
pub fn style_states() -> Vec<&'static str> {
    let mut v = vec!["normal"];
    v.extend(phai_form::STATES);
    v
}

/// A control's declaration of `prop` in a state's CSS (`background: red; color: white`).
pub fn decl_of(decls: &str, prop: &str) -> Option<String> {
    decls.split(';').filter_map(|d| d.split_once(':')).find(|(k, _)| k.trim() == prop).map(|(_, v)| v.trim().to_string())
}

/// `decls` with `prop` set to `value`, or taken out for `None`.
pub fn with_decl(decls: &str, prop: &str, value: Option<&str>) -> String {
    let mut out: Vec<String> = decls.split(';').map(str::trim).filter(|d| !d.is_empty() && d.split_once(':').is_none_or(|(k, _)| k.trim() != prop)).map(str::to_string).collect();
    if let Some(v) = value {
        out.push(format!("{prop}: {v}"));
    }
    out.join("; ")
}

/// The Events tab's category of an event, as Visual Studio groups them.
fn event_category(name: &str) -> &'static str {
    match name {
        "enter" | "leave" => t("Focus"),
        "key_down" | "key_up" | "key_press" => t("Key"),
        "resize" => t("Layout"),
        "shown" | "closing" | "closed" | "activated" | "deactivate" => t("Behavior"),
        n if n.starts_with("mouse_") => t("Mouse"),
        _ => t("Action"),
    }
}

/// The editor a table property's kind wants.
fn ed_of(s: &Spec, buttons: &[String]) -> Ed {
    match s.ty {
        Ty::Bool => Ed::Bool,
        Ty::Int(..) => Ed::Number,
        Ty::Choice(c) => Ed::Choice(c.iter().map(|x| x.to_string()).collect(), false),
        Ty::Color => Ed::Color,
        Ty::Button => {
            let mut v = vec![String::new()];
            v.extend(buttons.iter().cloned());
            Ed::Choice(v, false)
        }
        _ => Ed::Text,
    }
}

/// A table value as the grid shows it: True and False as Visual Studio writes them.
fn shown(s: &Spec, v: Option<&phai_form::toml::Value>) -> String {
    let text = v.map(table::text_of).unwrap_or_else(|| s.default.to_string());
    match (s.ty, text.as_str()) {
        (Ty::Bool, "true") => "True".into(),
        (Ty::Bool, "false") => "False".into(),
        _ => text,
    }
}

/// `#rrggbb`.
fn hex(c: Color) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
}

/// How a value is edited.
#[derive(Clone, Debug, PartialEq)]
enum Ed {
    Text,
    Number,
    Bool,
    /// One of these; `free` lets other words be typed too.
    Choice(Vec<String>, bool),
    /// "…" opens a dialog.
    Dialog,
    Anchor,
    Color,
    /// Shown, not changed here.
    ReadOnly,
}

#[derive(Clone, Debug)]
struct Prop {
    category: &'static str,
    key: Key,
    name: &'static str,
    value: String,
    ed: Ed,
    help: std::borrow::Cow<'static, str>,
    /// The composite property this one is a part of - shown only when that is open.
    parent: Option<Key>,
    /// Set apart from its default: shown in bold, as Visual Studio shows it.
    set: bool,
}

/// Whether a value is other than what a new control has.
fn is_set(key: Key, value: &str) -> bool {
    match key {
        Key::FormName | Key::FormTitle | Key::FormSize | Key::FormWidth | Key::FormHeight | Key::Id | Key::Location | Key::Size | Key::X | Key::Y | Key::W | Key::H => true,
        Key::Bold | Key::Checked | Key::Password | Key::Locked => value == "True",
        Key::Enabled | Key::Visible => value == "False",
        Key::Group | Key::Min | Key::Value => value != "0",
        Key::Max => value != "100",
        Key::Dock => value != "none",
        Key::Anchor => value != "left top",
        Key::FontAll => value.contains("Bold") || !value.ends_with(", 9pt"),
        _ => !value.trim().is_empty(),
    }
}

/// The two numbers of "a, b" - a Location or a Size typed in.
pub fn pair(text: &str) -> Option<(i32, i32)> {
    let mut it = text.split([',', ';', 'x', '×', ' ']).map(str::trim).filter(|s| !s.is_empty()).map(|s| s.parse::<i32>());
    match (it.next(), it.next(), it.next()) {
        (Some(Ok(a)), Some(Ok(b)), None) => Some((a, b)),
        _ => None,
    }
}

/// The size and boldness of "Segoe UI, 10pt, Bold" - a Font typed in. A bare number is points.
pub fn font_spec(text: &str) -> Option<(Option<i32>, bool)> {
    let mut size = None;
    let mut bold = false;
    for part in text.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let low = part.to_lowercase();
        if low == "bold" {
            bold = true;
        } else if let Ok(n) = low.trim_end_matches("pt").trim().parse::<i32>() {
            if !(6..=48).contains(&n) {
                return None;
            }
            size = Some(n);
        }
    }
    Some((size, bold))
}

/// A row of the grid as shown: a category's heading, or a property.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Line {
    Heading(&'static str),
    Prop(usize),
}

pub struct Props {
    pub hwnd: HWND,
    /// Where the line between the names and the values is: thousandths of the width, dragged.
    share: i32,
    /// The line being dragged.
    dragging: bool,
    chooser: HWND,
    /// Categorized, Alphabetical | Properties, Events.
    bar: HWND,
    /// The name shown when the pane is about a code file.
    heading: HWND,
    grid: HWND,
    help_name: HWND,
    help_text: HWND,
    mode: usize,
    /// The Style tab's state: 0 normal, then `phai_form::STATES`.
    style_state: usize,
    /// The canvas control shown in that state.
    previewing: Option<HWND>,
    alphabetical: bool,
    folded: Vec<&'static str>,
    /// The composite properties folded open.
    expanded: Vec<Key>,
    props: Vec<Prop>,
    lines: Vec<Line>,
    chosen: Option<usize>,
    top: usize,
    /// The in-place editor and the property it edits.
    editor: Option<(HWND, usize)>,
    font: HFONT,
    bold: HFONT,
    filling: bool,
}

const ID_CHOOSER: i32 = 3300;
const ID_BAR: i32 = 3301;
const ID_SORT_CAT: i32 = 3302;
const ID_SORT_AZ: i32 = 3303;
const ID_MODE_PROPS: i32 = 3305;
const ID_MODE_EVENTS: i32 = 3306;
const ID_MODE_STYLE: i32 = 3307;

fn bar_items() -> Vec<(i32, &'static str, u16)> {
    vec![
        (ID_SORT_CAT, t("Categorized"), 0xE8A9),
        (ID_SORT_AZ, t("Alphabetical"), 0xE8CB),
        (0, "", 0),
        (ID_MODE_PROPS, t("Properties"), 0xE713),
        (ID_MODE_EVENTS, t("Events"), 0xE945),
        (ID_MODE_STYLE, t("Style"), 0xE790),
    ]
}
const ID_EDITOR: i32 = 3304;
/// Posted to the pane to fill itself again after an edit, once the editor's message is done.
const WM_REFILL: u32 = wm::WM_APP + 0x0D01;

fn yes_no(b: bool) -> String {
    if b { "True" } else { "False" }.to_string()
}

/// Every class the project's theme sheet names - `button.primary` gives `primary`.
fn theme_classes() -> Vec<String> {
    let Some(s) = studio() else { return vec![] };
    let (Some(path), Some(theme)) = (s.doc.path.as_ref(), s.doc.form.form.theme.as_ref()) else { return vec!["primary".into(), "muted".into()] };
    let css = std::fs::read_to_string(path.with_file_name("").join(theme)).unwrap_or_default();
    let mut out: Vec<String> = vec!["primary".into(), "muted".into()];
    for word in css.split(|c: char| !(c.is_alphanumeric() || c == '-' || c == '_' || c == '.')) {
        if let Some(cls) = word.split('.').nth(1) {
            if !cls.is_empty() && !out.iter().any(|o| o == cls) && cls.chars().next().is_some_and(|c| c.is_alphabetic()) {
                out.push(cls.to_string());
            }
        }
    }
    out
}

/// The methods the programmer's file already has - for an event to point at.
fn user_methods() -> Vec<String> {
    let Some(s) = studio() else { return vec![] };
    let Some(path) = s.doc.path.as_ref() else { return vec![] };
    let user = path.with_file_name(format!("{}.rs", phai_form::snake(&s.doc.form.form.name)));
    let src = std::fs::read_to_string(user).unwrap_or_default();
    src.split("fn ").skip(1).filter_map(|r| r.split('(').next()).map(|n| n.trim().to_string()).filter(|n| !n.is_empty() && n.chars().all(|c| c.is_alphanumeric() || c == '_') && n != "loaded").collect()
}

impl Props {
    pub fn new(parent: HWND, id: i32, font: HFONT) -> Props {
        let class = win::register("PhaiStudioProps", Some(proc_), None);
        let hwnd = unsafe {
            wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPCHILDREN | wm::WS_CLIPSIBLINGS, 0, 0, 10, 10, Some(parent), Some(HMENU(id as isize as *mut _)), Some(win::instance()), None).unwrap_or_default()
        };
        // The chooser draws each object as Visual Studio does: its name in bold, its kind beside.
        const CBS_DROPDOWNLIST: u32 = 0x0003;
        const CBS_OWNERDRAWFIXED: u32 = 0x0010;
        const CBS_HASSTRINGS: u32 = 0x0200;
        let chooser = win::child(hwnd, "COMBOBOX", "", wm::WINDOW_STYLE(CBS_DROPDOWNLIST | CBS_OWNERDRAWFIXED | CBS_HASSTRINGS) | wm::WS_TABSTOP | wm::WS_VSCROLL, Default::default(), ID_CHOOSER, font);
        win::register_kind(chooser, win::Kind::Combo);
        let bar = win::toolbar(hwnd, ID_BAR, &bar_items());
        // Small icons with room above and below, as Visual Studio's Properties row.
        win::toolbar_compact(bar, 14, 22, &bar_items());
        let heading = win::label(hwnd, "", 0, font);
        let grid_class = win::register_with(win::ClassStyle::DoubleClicks, "PhaiPropGrid", Some(grid_proc), None);
        let grid = unsafe {
            wm::CreateWindowExW(Default::default(), PCWSTR(grid_class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPCHILDREN | wm::WS_VSCROLL, 0, 0, 10, 10, Some(hwnd), Some(HMENU(3310 as *mut _)), Some(win::instance()), None).unwrap_or_default()
        };
        let bold = win::font(hwnd, 0, true);
        let help_name = win::label(hwnd, "", 0, bold);
        let help_text = win::child(hwnd, "STATIC", "", wm::WINDOW_STYLE(0x80), Default::default(), 0, font);
        win::register_kind(help_text, win::Kind::Label);
        win::set_css(help_text, "muted", "");
        // As it was left last time.
        let share = crate::setting_pub("props_split").and_then(|v| v.parse::<i32>().ok()).filter(|v| (150..=850).contains(v)).unwrap_or(450);
        let p = Props { hwnd, share, dragging: false, chooser, bar, heading, grid, help_name, help_text, mode: 0, style_state: 0, previewing: None, alphabetical: false, folded: vec![], expanded: vec![], props: vec![], lines: vec![], chosen: None, top: 0, editor: None, font, bold, filling: false };
        p.say_bar();
        p
    }

    /// The properties of what is chosen, from the model. Several controls chosen: the
    /// properties they all have, each value shown where all agree and left empty where they
    /// differ - as Visual Studio does. A value typed in then goes to all of them.
    fn gather(&self) -> Vec<Prop> {
        let Some(s) = studio() else { return vec![] };
        let mut v = self.gather_one(s.selected);
        let chosen = s.selection();
        if self.mode == 0 && chosen.len() > 1 {
            v.retain(|p| p.key != Key::Id);
            for j in chosen.into_iter().filter(|j| Some(*j) != s.selected) {
                let other = self.gather_one(Some(j));
                v.retain(|p| other.iter().any(|q| q.key == p.key));
                for p in v.iter_mut() {
                    if other.iter().any(|q| q.key == p.key && q.value != p.value) {
                        p.value.clear();
                        p.set = false;
                    }
                }
            }
        }
        v
    }

    /// The properties of a menu, menu item or toolbar button - or of the menu bar or toolbar.
    fn gather_bar(&self, b: &crate::BarSel) -> Vec<Prop> {
        let Some(s) = studio() else { return vec![] };
        let f = &s.doc.form;
        let p = |category: &'static str, key, name: &'static str, value: String, ed, help: &'static str| {
            let set = !value.is_empty() && value != "False";
            Prop { category: t(category), key, name: t(name), value, ed, help: t(help).into(), parent: None, set }
        };
        let mut glyphs: Vec<String> = vec![String::new()];
        glyphs.extend(phai_ui::icon::glyph::NAMES.iter().map(|(n, _)| n.to_string()).filter(|n| !n.starts_with("LAY_")));
        let keys: Vec<String> = ["", "Ctrl+N", "Ctrl+O", "Ctrl+S", "Ctrl+Shift+S", "Ctrl+P", "Ctrl+Z", "Ctrl+Y", "Ctrl+X", "Ctrl+C", "Ctrl+V", "Ctrl+A", "Ctrl+F", "Ctrl+H", "F1", "F2", "F5", "Del", "Alt+F4"].iter().map(|k| k.to_string()).collect();
        let yes = |b: bool| if b { "True".to_string() } else { "False".to_string() };
        match b {
            crate::BarSel::MenuStrip => vec![
                p("Design", Key::Bar("name"), "(Name)", "menuStrip".into(), Ed::ReadOnly, "The form's menu bar: click a menu on the form to design it."),
                p("Data", Key::Bar("count"), "Items", tf("{0} menus", &[&f.menu.len()]), Ed::ReadOnly, "Its menus, designed on the form."),
            ],
            crate::BarSel::ToolStrip => vec![
                p("Design", Key::Bar("name"), "(Name)", "toolStrip".into(), Ed::ReadOnly, "The form's toolbar: click a button on the form to design it."),
                p("Data", Key::Bar("count"), "Items", tf("{0} buttons", &[&f.toolbar.len()]), Ed::ReadOnly, "Its buttons and separators, designed on the form."),
            ],
            crate::BarSel::Menu(i) => {
                let Some(m) = f.menu.get(*i) else { return vec![] };
                vec![p("Appearance", Key::Bar("text"), "Text", m.label.clone(), Ed::Text, "The menu's words on the bar; & before the underlined letter (Alt+it opens it).")]
            }
            crate::BarSel::MenuItem(path) => {
                let Some(it) = crate::menu_item(f, path) else { return vec![] };
                if it.separator {
                    return vec![p("Design", Key::Bar("name"), "(Name)", t("Separator").into(), Ed::ReadOnly, "A line between groups of items.")];
                }
                let sub = !it.items.is_empty();
                vec![
                    p("Design", Key::Bar("id"), "(Name)", it.id.clone().unwrap_or_default(), if sub { Ed::ReadOnly } else { Ed::Text }, "Its Click method's name - double-click it on the form to write it. An item that opens a menu has none."),
                    p("Appearance", Key::Bar("text"), "Text", it.label.clone().unwrap_or_default(), Ed::Text, "Its words; & before the underlined letter."),
                    p("Appearance", Key::Bar("image"), "Image", it.glyph.clone().unwrap_or_default(), Ed::Choice(glyphs, false), "A picture beside its words."),
                    p("Misc", Key::Bar("shortcut"), "ShortcutKeys", it.key.clone().unwrap_or_default(), Ed::Choice(keys, true), "The keys that choose it from anywhere in the window, e.g. Ctrl+S."),
                    p("Behavior", Key::Bar("checked"), "Checked", yes(it.checked), Ed::Bool, "A tick beside it."),
                    p("Behavior", Key::Bar("check_on_click"), "CheckOnClick", yes(it.check_on_click), Ed::Bool, "A click ticks it, or takes the tick away."),
                    p("Behavior", Key::Bar("enabled"), "Enabled", yes(!it.disabled), Ed::Bool, "Whether it can be chosen."),
                ]
            }
            crate::BarSel::Tool(i) => {
                let Some(tool) = f.toolbar.get(*i) else { return vec![] };
                if tool.separator {
                    return vec![p("Design", Key::Bar("name"), "(Name)", t("Separator").into(), Ed::ReadOnly, "A line between groups of buttons.")];
                }
                vec![
                    p("Design", Key::Bar("id"), "(Name)", tool.id.clone(), Ed::Text, "Its Click method's name - double-click it on the form to write it."),
                    p("Appearance", Key::Bar("text"), "Text", tool.label.clone(), Ed::Text, "Its words: the tooltip shown when the pointer rests on it."),
                    p("Appearance", Key::Bar("image"), "Image", tool.glyph.clone(), Ed::Choice(glyphs[1..].to_vec(), false), "Its picture."),
                    p("Behavior", Key::Bar("check_on_click"), "CheckOnClick", yes(tool.toggle), Ed::Bool, "It stays down when clicked, and comes up at the next click."),
                    p("Behavior", Key::Bar("checked"), "Checked", yes(tool.checked), Ed::Bool, "Down when the window opens (with CheckOnClick)."),
                    p("Behavior", Key::Bar("enabled"), "Enabled", yes(!tool.disabled), Ed::Bool, "Whether it can be clicked."),
                ]
            }
        }
    }

    /// The properties of one control - or the form's, for `None`.
    fn gather_one(&self, sel: Option<usize>) -> Vec<Prop> {
        let Some(s) = studio() else { return vec![] };
        let f = &s.doc.form;
        let p = |category, key, name, value: String, ed, help: &'static str| Prop { category, key, name, value, ed, help: help.into(), parent: None, set: false };
        let part = |parent: Key, category, key, name, value: String, ed, help: &'static str| Prop { category, key, name, value, ed, help: help.into(), parent: Some(parent), set: false };
        let sizes: Vec<String> = ["", "8", "9", "10", "11", "12", "14", "16", "18", "20", "24", "28", "36"].iter().map(|s| s.to_string()).collect();
        let bools = Ed::Bool;
        // A part of the menu bar or toolbar: its own properties, as a ToolStripItem's.
        if let Some(b) = s.bar_sel.clone() {
            if self.mode == 1 {
                // Its one event: Click, its id's method.
                let id = match &b {
                    crate::BarSel::MenuItem(path) => crate::menu_item(&s.doc.form, path).filter(|it| !it.separator && it.items.is_empty()).and_then(|it| it.id.clone()),
                    crate::BarSel::Tool(i) => s.doc.form.toolbar.get(*i).filter(|t0| !t0.separator).map(|t0| t0.id.clone()),
                    _ => None,
                };
                return id.map(|id| vec![Prop { category: t("Action"), key: Key::Bar("click"), name: t("Click"), value: id, ed: Ed::ReadOnly, help: t("The method a click runs - double-click the item on the form to write it.").into(), parent: None, set: true }]).unwrap_or_default();
            }
            if self.mode != 0 {
                return vec![];
            }
            let mut v = self.gather_bar(&b);
            // Enabled is True unless set otherwise: shown plain then, as a default.
            for q in v.iter_mut().filter(|q| q.key == Key::Bar("enabled")) {
                q.set = q.value == "False";
            }
            return v;
        }
        if self.mode == 2 {
            return self.gather_style(sel);
        }
        if self.mode == 1 {
            // Events: the control's, or the form's own and its menus' and toolbar's.
            let methods = user_methods();
            let row = |key, name: &'static str, help: &'static str, value: Option<&String>| {
                let ev_name = match key {
                    Key::On(n) | Key::FormOn(n) => n,
                    _ => "",
                };
                Prop { category: event_category(ev_name), key, name, value: value.cloned().unwrap_or_default(), ed: Ed::Choice(methods.clone(), true), help: t(help).into(), parent: None, set: value.is_some() }
            };
            return match sel.and_then(|i| f.control.get(i)) {
                Some(c) => {
                    let mut v = Vec::new();
                    if let Some(e) = c.kind.event() {
                        let name = match e {
                            "on_click" => t("Click"),
                            "on_change" => t("Change"),
                            "on_tick" => t("Tick"),
                            _ => t("Select"),
                        };
                        v.push(p(t("Action"), Key::Event, name, c.handler().unwrap_or_default().to_string(), Ed::Choice(methods.clone(), true), t("The method that runs when this happens. Double-click the control to write it.")));
                    }
                    for e in phai_form::events::of_kind(c.kind) {
                        v.push(row(Key::On(e.name), e.label, e.help, c.events.get(e.name)));
                    }
                    v
                }
                None => {
                    let mut v = vec![p(t("Behavior"), Key::Event, "Load", "loaded".into(), Ed::ReadOnly, t("Runs once the window is made, before it shows."))];
                    for e in phai_form::events::FORM {
                        v.push(row(Key::FormOn(e.name), e.label, e.help, f.form.events.get(e.name)));
                    }
                    for h in f.handlers() {
                        v.push(p(t("Menus and toolbars"), Key::Event, Box::leak(h.clone().into_boxed_str()), h, Ed::ReadOnly, t("A menu item, toolbar button or control runs this method.")));
                    }
                    v
                }
            };
        }
        let Some(c) = sel.and_then(|i| f.control.get(i)) else {
            return [
                p(t("Design"), Key::FormName, t("Name"), f.form.name.clone(), Ed::Text, t("The window's name in code: a struct, and its files' names.")),
                p(t("Appearance"), Key::FormTitle, t("Title"), f.form.title.clone(), Ed::Text, t("The words in the title bar - an English key, translated by the program.")),
                p(t("Appearance"), Key::FormFont, t("Font size"), f.form.font_size.map(|n| n.to_string()).unwrap_or_default(), Ed::Choice(sizes, true), t("Points for every control's words; empty is Windows' own 9.")),
                p(t("Appearance"), Key::FormTheme, t("Theme"), f.form.theme.clone().unwrap_or_default(), Ed::Dialog, t("The CSS file every control takes its colours from.")),
                p(t("Layout"), Key::FormSize, t("Size"), format!("{}, {}", f.form.size[0], f.form.size[1]), Ed::Text, t("The inside of the window, at 100% scale.")),
                part(Key::FormSize, t("Layout"), Key::FormWidth, t("Width"), f.form.size[0].to_string(), Ed::Number, t("The inside of the window, at 100% scale.")),
                part(Key::FormSize, t("Layout"), Key::FormHeight, t("Height"), f.form.size[1].to_string(), Ed::Number, t("The inside of the window, at 100% scale.")),
                p(t("Layout"), Key::FormStart, t("Start position"), if f.form.start.as_deref() == Some("windows") { t("Windows default") } else { t("Center screen") }.to_string(), Ed::Choice(vec![t("Center screen").to_string(), t("Windows default").to_string()], false), t("Where the window first appears: the middle of the screen, or where Windows puts it.")),
                p(t("Layout"), Key::FormState, t("Window state"), if f.form.maximized == Some(true) { t("Maximized") } else { t("Normal") }.to_string(), Ed::Choice(vec![t("Normal").to_string(), t("Maximized").to_string()], false), t("Maximized: the window opens filling the screen.")),
            ]
            .into_iter()
            .map(|mut q| {
                q.set = is_set(q.key, &q.value);
                q
            })
            .chain(table::FORM.iter().map(|sp| {
                let buttons: Vec<String> = f.control.iter().filter(|c| c.kind == Kind::Button).map(|c| c.id.clone()).collect();
                let v = f.form.prop(sp.name);
                Prop { category: t(sp.category), key: Key::FormMore(sp.name), name: sp.label, value: shown(sp, v), ed: ed_of(sp, &buttons), help: t(sp.help).into(), parent: None, set: v.is_some() }
            }))
            .collect();
        };
        // A context menu: its name and its items.
        if c.kind == Kind::ContextMenu {
            return [
                p(t("Design"), Key::Id, t("Id"), c.id.clone(), Ed::Text, t("The control's name in code, and its #id in CSS.")),
                p(t("Data"), Key::Items, t("Items"), c.items.join("\n"), Ed::Dialog, t("One a line: method | label - the method runs when it is chosen; - alone a separator.")),
            ]
            .into_iter()
            .map(|mut q| {
                q.set = is_set(q.key, &q.value);
                q
            })
            .collect();
        }
        // A timer: its name, how often, whether it runs from the start.
        if c.kind == Kind::Timer {
            return [
                p(t("Design"), Key::Id, t("Id"), c.id.clone(), Ed::Text, t("The control's name in code, and its #id in CSS.")),
                p(t("Behavior"), Key::Value, t("Interval"), c.value.unwrap_or(1000).to_string(), Ed::Number, t("Milliseconds between ticks - 1000 is a second.")),
                p(t("Behavior"), Key::Enabled, t("Enabled"), yes_no(c.enabled != Some(false)), bools.clone(), t("Whether it ticks from when the window opens.")),
            ]
            .into_iter()
            .map(|mut q| {
                q.set = is_set(q.key, &q.value);
                q
            })
            .collect();
        }
        let [x, y, w, h] = c.rect();
        let docked = c.dock.is_some();
        let has_text = c.kind.has_text() || c.kind == Kind::Statusbar;
        let colour = |o: &Option<String>| o.clone().unwrap_or_default();
        let mut v = vec![
            p(t("Design"), Key::Id, t("Id"), c.id.clone(), Ed::Text, t("The control's name in code, and its #id in CSS.")),
            p(t("Design"), Key::Class, t("Class"), c.class.clone().unwrap_or_default(), Ed::Choice(theme_classes(), true), t("CSS classes from the theme, like primary. Several, space between.")),
            p(t("Design"), Key::Locked, t("Locked"), yes_no(c.locked == Some(true)), bools.clone(), t("Locked: the mouse and the arrows do not move or size it in the designer.")),
        ];
        if has_text {
            v.push(p(t("Appearance"), Key::Text, t("Text"), c.text.clone().unwrap_or_default(), Ed::Text, t("The words it shows - an English key, translated by the program.")));
        }
        v.extend([
            p(t("Appearance"), Key::FontAll, t("Font"), format!("Segoe UI, {}pt{}", c.font_size.or(f.form.font_size).unwrap_or(9), if c.bold == Some(true) { ", Bold" } else { "" }), Ed::Text, t("Its words' size and weight - Segoe UI, 9pt is Windows' own.")),
            part(Key::FontAll, t("Appearance"), Key::Font, t("Font size"), c.font_size.map(|n| n.to_string()).unwrap_or_default(), Ed::Choice(sizes, true), t("Points for this control's words, over the form's.")),
            part(Key::FontAll, t("Appearance"), Key::Bold, t("Bold"), yes_no(c.bold == Some(true)), bools.clone(), t("Its words in bold.")),
            p(t("Appearance"), Key::BackColor, t("Back colour"), colour(&c.back_color), Ed::Color, t("Its background, over the theme - written into CSS as #id { background }.")),
            p(t("Appearance"), Key::ForeColor, t("Fore colour"), colour(&c.fore_color), Ed::Color, t("Its words' colour, over the theme - #id { color }.")),
            p(t("Appearance"), Key::BorderColor, t("Border colour"), colour(&c.border_color), Ed::Color, t("Its border, over the theme - #id { border-color }.")),
            p(t("Appearance"), Key::Radius, t("Corner radius"), c.radius.map(|n| n.to_string()).unwrap_or_default(), Ed::Number, t("How round its corners are, pixels - #id { border-radius }.")),
            p(t("Behavior"), Key::Enabled, t("Enabled"), yes_no(c.enabled != Some(false)), bools.clone(), t("Whether it can be used, or shows greyed.")),
            p(t("Behavior"), Key::Visible, t("Visible"), yes_no(c.visible != Some(false)), bools.clone(), t("Whether it shows when the window opens.")),
        ]);
        if matches!(c.kind, Kind::Checkbox | Kind::Radio) {
            v.push(p(t("Behavior"), Key::Checked, t("Checked"), yes_no(c.checked == Some(true)), bools.clone(), t("Ticked when the window opens.")));
        }
        if c.kind == Kind::Radio {
            v.push(p(t("Behavior"), Key::Group, t("Group"), c.group.unwrap_or(0).to_string(), Ed::Number, t("Options with the same group number exclude each other.")));
        }
        if c.kind == Kind::Edit {
            v.push(p(t("Behavior"), Key::Password, t("Password"), yes_no(c.password == Some(true)), bools.clone(), t("Shows dots instead of what is typed.")));
        }
        if matches!(c.kind, Kind::Number | Kind::Progress | Kind::Slider) {
            v.push(p(t("Data"), Key::Min, t("Minimum"), c.min.unwrap_or(0).to_string(), Ed::Number, t("The smallest value.")));
            v.push(p(t("Data"), Key::Max, t("Maximum"), c.max.unwrap_or(100).to_string(), Ed::Number, t("The largest value.")));
            v.push(p(t("Data"), Key::Value, t("Value"), c.value.unwrap_or(0).to_string(), Ed::Number, t("The value when the window opens.")));
        }
        // A context menu for any control, when the form has one.
        let menus: Vec<String> = f.control.iter().filter(|m| m.kind == Kind::ContextMenu).map(|m| m.id.clone()).collect();
        if !menus.is_empty() && !c.kind.is_component() {
            let mut choices = vec![String::new()];
            choices.extend(menus);
            v.push(p(t("Behavior"), Key::ContextMenu, t("Context menu"), c.context_menu.clone().unwrap_or_default(), Ed::Choice(choices, false), t("The menu a right-click on it opens.")));
        }
        if c.kind == Kind::Table {
            v.push(p(t("Layout"), Key::Cols, t("Columns"), c.cols.unwrap_or(2).to_string(), Ed::Number, t("How many equal columns the controls in it are laid in.")));
        }
        if c.kind == Kind::Picture {
            v.push(p(t("Appearance"), Key::Image, t("Image"), c.image.clone().unwrap_or_default(), Ed::Text, t("The picture's file: PNG, JPEG, BMP or GIF - from the project's folder, or a full path.")));
        }
        if matches!(c.kind, Kind::Combo | Kind::List | Kind::Tabs | Kind::Tree | Kind::Grid | Kind::ListBox | Kind::CheckedList) {
            let items = match c.kind {
                Kind::Combo | Kind::Tree | Kind::ListBox | Kind::CheckedList => c.items.join("\n"),
                Kind::Tabs => c.tabs.join("\n"),
                _ => c.columns.iter().map(|(n, w)| format!("{n}:{w}")).collect::<Vec<_>>().join("\n"),
            };
            v.push(p(t("Data"), Key::Items, if matches!(c.kind, Kind::List | Kind::Grid) { t("Columns") } else { t("Items") }, items, Ed::Dialog, t("One a line; a list's columns as Name:width.")));
        }
        if !docked {
            v.extend([
                p(t("Layout"), Key::Location, t("Location"), format!("{x}, {y}"), Ed::Text, t("Its top-left corner from the form's, at 100% scale.")),
                part(Key::Location, t("Layout"), Key::X, "X", x.to_string(), Ed::Number, t("From the form's left, at 100% scale.")),
                part(Key::Location, t("Layout"), Key::Y, "Y", y.to_string(), Ed::Number, t("From the form's top, at 100% scale.")),
            ]);
        }
        v.extend([
            p(t("Layout"), Key::Size, t("Size"), format!("{w}, {h}"), Ed::Text, t("Its width and height, at 100% scale.")),
            part(Key::Size, t("Layout"), Key::W, t("Width"), w.to_string(), Ed::Number, t("At 100% scale.")),
            part(Key::Size, t("Layout"), Key::H, t("Height"), h.to_string(), Ed::Number, t("At 100% scale.")),
            p(t("Layout"), Key::Dock, t("Dock"), c.dock.clone().unwrap_or_else(|| "none".into()), Ed::Choice(DOCKS.iter().map(|d| d.to_string()).collect(), false), t("Fills an edge of the form, or all that is left.")),
        ]);
        if !docked {
            v.push(p(t("Layout"), Key::Anchor, t("Anchor"), c.anchor.clone().unwrap_or_else(|| "left top".into()), Ed::Anchor, t("The edges it keeps its distance to when the window grows.")));
        }
        for q in v.iter_mut() {
            q.set = is_set(q.key, &q.value);
        }
        // What the theme gives it, where the control sets nothing of its own: plain, not bold.
        let theme = sel.and_then(|i| s.canvas.as_ref().and_then(|cv| cv.control_window(i))).and_then(|h| win::kind_of(h).map(|k| win::style_of(h, k, State::default())));
        if let Some(st) = &theme {
            for q in v.iter_mut().filter(|q| !q.set) {
                let from_theme = match q.key {
                    Key::BackColor => st.background.map(hex),
                    Key::ForeColor => st.color.map(hex),
                    Key::BorderColor => st.border_color.map(hex),
                    Key::Radius => st.border_radius.map(|n| n.to_string()),
                    _ => None,
                };
                if let Some(x) = from_theme {
                    q.value = x;
                }
            }
        }
        for sp in table::of_kind(c.kind) {
            let val = c.prop(sp.name);
            let mut value = shown(sp, val);
            if val.is_none() {
                let from_theme = theme.as_ref().and_then(|st| match sp.name {
                    "border_width" => st.border_width.map(|n| n.to_string()),
                    "accent_color" => st.accent_color.map(hex),
                    "icon_color" => st.icon_color.map(hex),
                    _ => None,
                });
                if let Some(x) = from_theme {
                    value = x;
                }
            }
            v.push(Prop { category: t(sp.category), key: Key::More(sp.name), name: sp.label, value, ed: ed_of(sp, &[]), help: t(sp.help).into(), parent: None, set: val.is_some() });
        }
        v
    }

    /// The Style tab: the chosen control's look in the chosen state - each value as it is drawn,
    /// where it comes from below, set ones in bold - its transition, and a way into the theme.
    fn gather_style(&self, sel: Option<usize>) -> Vec<Prop> {
        let Some(s) = studio() else { return vec![] };
        let f = &s.doc.form;
        let row = |category: &'static str, key, name: &'static str, value: String, ed, help: std::borrow::Cow<'static, str>, set| Prop { category, key, name, value, ed, help, parent: None, set };
        let Some((i, c)) = sel.and_then(|i| f.control.get(i).map(|c| (i, c))) else {
            return vec![row(t("Style"), Key::StyleState, t("Choose a control"), String::new(), Ed::ReadOnly, t("The Style tab shows a control's look: choose one on the form.").into(), false)];
        };
        let states = style_states();
        let st = self.style_state.min(states.len() - 1);
        let mut v = vec![row(t("State"), Key::StyleState, t("State"), states[st].to_string(), Ed::Choice(states.iter().map(|x| x.to_string()).collect(), false), t("The state to see and change; the form shows the control in it.").into(), st != 0)];
        let Some(h) = s.canvas.as_ref().and_then(|cv| cv.control_window(i)) else { return v };
        let Some(kind) = win::kind_of(h) else { return v };
        let state = State { hover: states[st] == "hover", pressed: states[st] == "pressed", focus: states[st] == "focus", disabled: states[st] == "disabled", checked: states[st] == "checked", selected: false };
        let look = win::style_of(h, kind, state);
        let origins = win::explain_of(h, kind, state);
        let own_css = c.own_css().unwrap_or_default();
        let mine = if st == 0 { own_css } else { c.states.get(states[st]).cloned().unwrap_or_default() };
        for prop in STYLE_PROPS {
            let value = match prop {
                "background" => look.background.map(hex),
                "color" => look.color.map(hex),
                "border-color" => look.border_color.map(hex),
                "accent-color" => look.accent_color.map(hex),
                "icon-color" => look.icon_color.map(hex),
                "border-width" => look.border_width.map(|n| n.to_string()),
                "border-radius" => look.border_radius.map(|n| n.to_string()),
                "background-image" => look.gradient.as_ref().map(|g| g.css()),
                "box-shadow" => look.shadow.flatten().map(|s| s.css()),
                _ => look.font_weight.map(|w| if w >= 600 { "bold".into() } else { "normal".into() }),
            }
            .unwrap_or_default();
            let help: std::borrow::Cow<'static, str> = match origins.iter().find(|o| o.property == prop) {
                Some(o) if o.source == "own" => tf("Set on this control: {0}", &[&o.selector]).into(),
                Some(o) if o.source.is_empty() => tf("From {0} in the theme, line {1}", &[&o.selector, &o.line]).into(),
                Some(o) => tf("From {0} in {1}, line {2}", &[&o.selector, &o.source, &o.line]).into(),
                None => t("Nothing sets it: Windows' own.").into(),
            };
            let ed = match prop {
                "border-width" | "border-radius" => Ed::Number,
                "background-image" | "box-shadow" => Ed::Text,
                "font-weight" => Ed::Choice(vec![String::new(), "normal".into(), "bold".into()], false),
                _ => Ed::Color,
            };
            let set = decl_of(&mine, prop).is_some() || (st == 0 && prop == "font-weight" && c.bold == Some(true));
            v.push(row(t("Look in this state"), Key::StyleProp(prop, st as u8), prop, value, ed, help, set));
        }
        if st == 0 {
            let (ms, easing) = c.prop("transition").and_then(|v| v.as_str()).map(|t| {
                let ms = t.split_whitespace().find_map(|w| w.strip_suffix("ms").and_then(|n| n.parse::<i32>().ok()).or_else(|| w.strip_suffix('s').and_then(|n| n.parse::<f32>().ok()).map(|n| (n * 1000.0) as i32)));
                let e = t.split_whitespace().find(|w| phai_ui::theme::Easing::parse(w).is_some()).unwrap_or("ease").to_string();
                (ms.map(|m| m.to_string()).unwrap_or_default(), e)
            }).unwrap_or_default();
            let set = !ms.is_empty();
            v.push(row(t("Transition"), Key::StyleTransition, t("Duration (ms)"), ms, Ed::Number, t("Its colours move to a new state's over this many milliseconds - 150 is quick; empty, at once.").into(), set));
            v.push(row(t("Transition"), Key::StyleEasing, t("Easing"), if easing.is_empty() { "ease".into() } else { easing }, Ed::Choice(["ease", "linear", "ease-in", "ease-out", "ease-in-out"].iter().map(|x| x.to_string()).collect(), false), t("How the change moves: ease starts and ends gently, linear evenly.").into(), set));
        }
        v.push(row(t("Theme"), Key::StyleToTheme, t("Send to theme"), t("Make a class...").into(), Ed::Dialog, t("This control's look, every state, becomes a class in the theme's CSS that other controls can take; the control takes the class.").into(), false));
        v
    }

    /// The lines shown: headings and their properties, folded ones left out - or every property by name.
    fn arrange(&mut self) {
        self.lines.clear();
        if self.alphabetical {
            let mut idx: Vec<usize> = (0..self.props.len()).filter(|i| self.props[*i].parent.is_none()).collect();
            idx.sort_by_key(|i| self.props[*i].name.to_lowercase());
            for i in idx {
                self.push_with_parts(i);
            }
            return;
        }
        let mut cats: Vec<&'static str> = Vec::new();
        for p in &self.props {
            if !cats.contains(&p.category) {
                cats.push(p.category);
            }
        }
        for c in cats {
            self.lines.push(Line::Heading(c));
            if !self.folded.contains(&c) {
                let mine: Vec<usize> = (0..self.props.len()).filter(|i| self.props[*i].category == c && self.props[*i].parent.is_none()).collect();
                for i in mine {
                    self.push_with_parts(i);
                }
            }
        }
    }

    /// A property's line, and its parts' lines below it when it is folded open.
    fn push_with_parts(&mut self, i: usize) {
        self.lines.push(Line::Prop(i));
        let key = self.props[i].key;
        if self.expanded.contains(&key) {
            for (k, q) in self.props.iter().enumerate() {
                if q.parent == Some(key) {
                    self.lines.push(Line::Prop(k));
                }
            }
        }
    }

    fn has_parts(&self, i: usize) -> bool {
        let key = self.props[i].key;
        self.props.iter().any(|q| q.parent == Some(key))
    }

    /// The left margin, where a category's or a composite's box is.
    fn gutter(&self) -> i32 {
        win::px(self.grid, 16)
    }

    /// The icons' pressed state as the pane is now.
    fn say_bar(&self) {
        for (id, on) in [(ID_SORT_CAT, !self.alphabetical), (ID_SORT_AZ, self.alphabetical), (ID_MODE_PROPS, self.mode == 0), (ID_MODE_EVENTS, self.mode == 1), (ID_MODE_STYLE, self.mode == 2)] {
            win::toolbar_check(self.bar, id, on);
        }
    }

    /// Fills everything from the model.
    pub fn refresh(&mut self) {
        let Some(s) = studio() else { return };
        self.close_editor(false);
        self.filling = true;
        // A retheme makes the bar's buttons again; their pressed state with them.
        self.say_bar();
        // A code file - or nothing open - has no properties to list.
        let code = s.doc.code.is_some() || s.nothing;
        for h in [self.chooser, self.bar, self.grid, self.help_name, self.help_text] {
            win::show(h, !code);
        }
        win::show(self.heading, code);
        if code {
            win::set_text(self.heading, &s.doc.path.as_ref().filter(|_| !s.nothing).and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
            self.filling = false;
            self.layout();
            return;
        }
        let f = &s.doc.form;
        unsafe {
            wm::SendMessageW(self.chooser, 0x014B /* CB_RESETCONTENT */, None, None);
            let add = |s: String| {
                let w = win::wide(&s);
                wm::SendMessageW(self.chooser, 0x0143 /* CB_ADDSTRING */, None, Some(LPARAM(w.as_ptr() as isize)));
            };
            // As Visual Studio: the name, then its type in code - `name  phai_ui::ctl::TextBox`.
            add(format!("{}\t{}Ui", f.form.name, f.form.name));
            for c in &f.control {
                add(format!("{}\tphai_ui::ctl::{}", c.id, phai_form::codegen::ctl_type(c.kind)));
            }
            // A part of the menu bar or toolbar chosen: it too, last, as Visual Studio lists a
            // ToolStripButton.
            let bar = s.bar_sel.as_ref().map(|b| match b {
                crate::BarSel::MenuStrip => "menuStrip\tMenuStrip".to_string(),
                crate::BarSel::ToolStrip => "toolStrip\tToolStrip".to_string(),
                crate::BarSel::Menu(i) => format!("{}\tToolStripMenuItem", f.menu.get(*i).map(|m| phai_form::bars::shown(&m.label)).unwrap_or_default()),
                crate::BarSel::MenuItem(path) => format!("{}\tToolStripMenuItem", crate::menu_item(f, path).map(|it| it.id.clone().unwrap_or_else(|| phai_form::bars::shown(it.label.as_deref().unwrap_or("")))).unwrap_or_default()),
                crate::BarSel::Tool(i) => format!("{}\tToolStripButton", f.toolbar.get(*i).map(|t0| if t0.separator { "separator".to_string() } else { t0.id.clone() }).unwrap_or_default()),
            });
            let bar_row = bar.is_some();
            if let Some(b) = bar {
                add(b);
            }
            let at = if bar_row { f.control.len() + 1 } else if s.selection().len() > 1 { usize::MAX } else { s.selected.map(|i| i + 1).unwrap_or(0) };
            wm::SendMessageW(self.chooser, 0x014E /* CB_SETCURSEL */, Some(WPARAM(at)), None);
        }
        let keep = self.chosen.and_then(|l| self.lines.get(l)).and_then(|l| if let Line::Prop(i) = l { self.props.get(*i).map(|p| p.key) } else { None });
        self.props = self.gather();
        self.sync_preview();
        for q in self.props.iter_mut().filter(|q| q.key == Key::Event) {
            q.set = !q.value.is_empty();
        }
        self.arrange();
        self.chosen = keep.and_then(|k| self.lines.iter().position(|l| matches!(l, Line::Prop(i) if self.props[*i].key == k)));
        self.say_help();
        self.filling = false;
        self.layout();
        self.update_scroll();
        unsafe {
            let _ = Gdi::InvalidateRect(Some(self.grid), None, false);
        }
    }

    fn say_help(&self) {
        match self.chosen.and_then(|l| self.lines.get(l)) {
            Some(Line::Prop(i)) => {
                win::set_text(self.help_name, self.props[*i].name);
                win::set_text(self.help_text, &self.props[*i].help);
            }
            _ => {
                win::set_text(self.help_name, "");
                win::set_text(self.help_text, "");
            }
        }
    }

    pub fn layout(&self) {
        let h = self.hwnd;
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(h, &mut r);
        }
        let px = |v| win::px(h, v);
        let (m, row, gap) = (px(8), px(23), px(6));
        let width = r.right - r.left;
        let mut y = m / 2;
        if unsafe { wm::IsWindowVisible(self.chooser) }.as_bool() {
            win::move_to(self.chooser, m / 2, y, width - m, px(300));
            y += row + px(2);
            win::move_to(self.bar, 0, y + px(2), width, win::toolbar_height(self.bar));
            y += win::toolbar_height(self.bar) + px(4);
        } else {
            win::move_to(self.heading, m, y, width - 2 * m, row);
            y += row + gap;
        }
        let help_h = px(64);
        let grid_h = (r.bottom - y - help_h - gap).max(0);
        win::move_to(self.grid, 0, y, width, grid_h);
        y += grid_h + gap;
        win::move_to(self.help_name, m, y, width - 2 * m, row);
        win::move_to(self.help_text, m, y + row, width - 2 * m, help_h - row);
    }

    /// The Style tab shows the chosen control in its state; the control before goes back. Again
    /// after the canvas is built anew, its windows being new.
    pub fn sync_preview(&mut self) {
        let Some(s) = studio() else { return };
        let want = (self.mode == 2 && self.style_state > 0).then(|| s.selected.and_then(|i| s.canvas.as_ref().and_then(|cv| cv.control_window(i)))).flatten();
        if self.previewing != want {
            if let Some(old) = self.previewing.take() {
                win::set_preview_state(old, None);
            }
        }
        if let Some(h) = want {
            let st = style_states()[self.style_state.min(5)];
            win::set_preview_state(h, Some(State { hover: st == "hover", pressed: st == "pressed", focus: st == "focus", disabled: st == "disabled", checked: st == "checked", selected: false }));
            self.previewing = Some(h);
        }
    }

    /// The Style tab on state `n` (0 normal, then hover, pressed, focus, disabled, checked).
    pub fn set_style_state(&mut self, n: usize) {
        self.style_state = n;
        self.set_mode(2);
    }

    pub fn set_mode(&mut self, mode: usize) {
        self.mode = mode;
        self.say_bar();
        self.refresh();
    }

    pub fn relabel(&mut self) {
        win::set_toolbar(self.bar, &bar_items());
        self.say_bar();
        self.refresh();
    }

    /// Visual Studio's rows: close, a little more than the words' height.
    fn row_h(&self) -> i32 {
        win::measure(self.grid, self.font, "Ag").1 + win::px(self.grid, 5)
    }

    fn visible_rows(&self) -> usize {
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(self.grid, &mut r);
        }
        (r.bottom / self.row_h().max(1)).max(1) as usize
    }

    fn update_scroll(&mut self) {
        let page = self.visible_rows();
        self.top = self.top.min(self.lines.len().saturating_sub(page));
        unsafe {
            let si = wm::SCROLLINFO { cbSize: std::mem::size_of::<wm::SCROLLINFO>() as u32, fMask: wm::SIF_ALL, nMin: 0, nMax: self.lines.len().max(1) as i32 - 1, nPage: page as u32, nPos: self.top as i32, nTrackPos: 0 };
            windows::Win32::UI::Controls::SetScrollInfo(self.grid, wm::SB_VERT, &si, true);
        }
    }

    /// The name column's width.
    fn split(&self) -> i32 {
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(self.grid, &mut r);
        }
        r.right * self.share / 1000
    }

    /// Whether `x` is on the line between the names and the values - where it is dragged.
    fn on_split(&self, x: i32) -> bool {
        (x - self.split()).abs() <= win::px(self.grid, 4)
    }

    /// The line dragged to `x`: kept between 15% and 85% of the width.
    fn drag_split(&mut self, x: i32) {
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(self.grid, &mut r);
        }
        if r.right <= 0 {
            return;
        }
        self.share = ((x as i64 * 1000 / r.right as i64) as i32).clamp(150, 850);
        unsafe {
            let _ = Gdi::InvalidateRect(Some(self.grid), None, false);
        }
    }

    fn line_rect(&self, line: usize) -> RECT {
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(self.grid, &mut r);
        }
        let rh = self.row_h();
        let y = (line as i32 - self.top as i32) * rh;
        RECT { left: 0, top: y, right: r.right, bottom: y + rh }
    }

    fn paint(&self) {
        let h = self.grid;
        unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            let bg = win::window_background();
            win::fill(dc, &r, bg);
            let rule = phai_ui::theme::style("group", State::default()).border_or(Color::rgb(0xe5, 0xe5, 0xe5));
            let tint = phai_ui::theme::style("list", State { selected: true, ..Default::default() }).background_or(Color::rgb(0xe7, 0xf3, 0xef));
            let head_bg = phai_ui::theme::style("tabs", State::default()).background_or(bg);
            let muted = phai_ui::theme::current().var_color("--muted").unwrap_or(Color::rgb(0x5f, 0x5f, 0x5f));
            let split = self.split();
            let pad = win::px(h, 8);
            let gutter = self.gutter();
            // Visual Studio's grey margin down the left, where the boxes that fold are.
            win::fill(dc, &RECT { right: gutter, ..r }, head_bg);
            let fold_box = |dc: Gdi::HDC, lr: &RECT, open: bool| {
                let s = win::px(h, 9);
                let (cx, cy) = (gutter / 2, (lr.top + lr.bottom) / 2);
                let b = RECT { left: cx - s / 2, top: cy - s / 2, right: cx - s / 2 + s, bottom: cy - s / 2 + s };
                win::round_rect(dc, b, bg, muted, 0);
                win::fill(dc, &RECT { left: b.left + 2, top: cy, right: b.right - 2, bottom: cy + 1 }, win::window_text());
                if !open {
                    win::fill(dc, &RECT { left: cx, top: b.top + 2, right: cx + 1, bottom: b.bottom - 2 }, win::window_text());
                }
            };
            for (k, line) in self.lines.iter().enumerate().skip(self.top).take(self.visible_rows() + 1) {
                let lr = self.line_rect(k);
                match line {
                    Line::Heading(name) => {
                        win::fill(dc, &lr, head_bg);
                        fold_box(dc, &lr, !self.folded.contains(name));
                        let mut tr = RECT { left: gutter + pad / 2, ..lr };
                        let old = Gdi::SelectObject(dc, self.bold.into());
                        draw(dc, name, &mut tr, win::window_text(), Gdi::DT_LEFT);
                        Gdi::SelectObject(dc, old);
                    }
                    Line::Prop(i) => {
                        let p = &self.props[*i];
                        let name_r = RECT { left: gutter, right: split, ..lr };
                        if self.chosen == Some(k) {
                            win::fill(dc, &name_r, tint);
                        }
                        if self.has_parts(*i) {
                            fold_box(dc, &lr, self.expanded.contains(&p.key));
                        }
                        let old = Gdi::SelectObject(dc, self.font.into());
                        let indent = if p.parent.is_some() { win::px(h, 12) } else { 0 };
                        let mut tr = RECT { left: gutter + pad / 2 + indent, right: split - pad / 2, ..lr };
                        draw(dc, p.name, &mut tr, win::window_text(), Gdi::DT_LEFT);
                        // A value set apart from its default, in bold.
                        if p.set {
                            Gdi::SelectObject(dc, self.bold.into());
                        }
                        let mut vr = RECT { left: split + pad / 2, right: lr.right - pad / 2, ..lr };
                        match &p.ed {
                            Ed::Color => {
                                // A swatch before the colour's words.
                                if let Some(c) = phai_ui::theme::parse_color(&p.value) {
                                    let sw = RECT { left: vr.left, top: lr.top + pad / 2, right: vr.left + win::px(h, 18), bottom: lr.bottom - pad / 2 };
                                    win::round_rect(dc, sw, c, rule, win::px(h, 3));
                                }
                                vr.left += win::px(h, 24);
                                draw(dc, &p.value, &mut vr, win::window_text(), Gdi::DT_LEFT);
                            }
                            Ed::Dialog => {
                                let first = p.value.lines().next().unwrap_or_default();
                                let more = if p.value.lines().count() > 1 { "  …" } else { "" };
                                let mut words = RECT { right: vr.right - win::px(h, 26), ..vr };
                                draw(dc, &format!("{first}{more}"), &mut words, win::window_text(), Gdi::DT_LEFT);
                                let b = RECT { left: vr.right - win::px(h, 24), top: lr.top + pad / 3, right: vr.right, bottom: lr.bottom - pad / 3 };
                                win::round_rect(dc, b, bg, rule, win::px(h, 4));
                                let mut br = b;
                                draw(dc, "…", &mut br, win::window_text(), Gdi::DT_CENTER);
                            }
                            Ed::ReadOnly => draw(dc, &p.value, &mut vr, muted, Gdi::DT_LEFT),
                            _ => draw(dc, &p.value, &mut vr, win::window_text(), Gdi::DT_LEFT),
                        }
                        Gdi::SelectObject(dc, old);
                        win::fill(dc, &RECT { left: split, right: split + 1, ..lr }, rule);
                    }
                }
                win::fill(dc, &RECT { left: gutter, top: lr.bottom - 1, ..lr }, rule);
            }
            let _ = Gdi::EndPaint(h, &ps);
        }
    }

    fn line_at(&self, y: i32) -> Option<usize> {
        let k = self.top + (y / self.row_h().max(1)).max(0) as usize;
        (k < self.lines.len()).then_some(k)
    }

    fn click(&mut self, x: i32, y: i32, double: bool) {
        let Some(k) = self.line_at(y) else { return };
        self.close_editor(true);
        match self.lines[k] {
            Line::Heading(name) => {
                if let Some(at) = self.folded.iter().position(|f| *f == name) {
                    self.folded.remove(at);
                } else {
                    self.folded.push(name);
                }
                self.arrange();
                self.update_scroll();
            }
            Line::Prop(i) if self.has_parts(i) && (x < self.gutter() || (double && x < self.split())) => {
                let key = self.props[i].key;
                if let Some(at) = self.expanded.iter().position(|e| *e == key) {
                    self.expanded.remove(at);
                } else {
                    self.expanded.push(key);
                }
                self.chosen = Some(k);
                self.say_help();
                self.arrange();
                self.update_scroll();
            }
            Line::Prop(i) if double && x < self.split() && matches!(self.props[i].key, Key::On(_) | Key::FormOn(_) | Key::Event) => {
                // As Visual Studio: a double-click on an event writes its method and goes to it.
                self.chosen = Some(k);
                let key = self.props[i].key;
                if let Some(s) = studio() {
                    unsafe {
                        let _ = wm::PostMessageW(Some(s.hwnd), crate::WM_EVENT_METHOD, WPARAM(event_key_index(key)), LPARAM(0));
                    }
                }
            }
            Line::Prop(i) => {
                self.chosen = Some(k);
                self.say_help();
                // A click on a value edits it; on a name, a double-click does.
                if x >= self.split() || double {
                    self.edit(k, i);
                }
            }
        }
        unsafe {
            let _ = Gdi::InvalidateRect(Some(self.grid), None, false);
        }
    }

    /// Opens the editor a property's kind wants, over its value.
    fn edit(&mut self, line: usize, i: usize) {
        let p = self.props[i].clone();
        let lr = self.line_rect(line);
        let split = self.split();
        let cell = RECT { left: split + 1, ..lr };
        match p.ed {
            Ed::ReadOnly => {}
            Ed::Bool => {
                // As Visual Studio does on a double-click: the other value, at once.
                self.commit(p.key, &yes_no(p.value != "True"));
            }
            Ed::Text | Ed::Number => self.open_box(cell, i, &p.value),
            Ed::Color => self.pick_colour(i, &p.value),
            Ed::Choice(items, free) => {
                const CBS_DROPDOWN: u32 = 0x0002;
                const CBS_DROPDOWNLIST: u32 = 0x0003;
                let style = if free { CBS_DROPDOWN } else { CBS_DROPDOWNLIST };
                let refs: Vec<&str> = items.iter().map(String::as_str).collect();
                let combo = win::combo(self.grid, ID_EDITOR, self.font, &refs);
                unsafe {
                    let old = wm::GetWindowLongW(combo, wm::GWL_STYLE) as u32;
                    wm::SetWindowLongW(combo, wm::GWL_STYLE, ((old & !0x3) | style) as i32);
                }
                win::move_to(combo, cell.left, cell.top, cell.right - cell.left, win::px(self.grid, 300));
                if free {
                    win::set_text(combo, &p.value);
                }
                let at = items.iter().position(|s| *s == p.value).unwrap_or(usize::MAX);
                unsafe {
                    wm::SendMessageW(combo, 0x014E /* CB_SETCURSEL */, Some(WPARAM(at)), None);
                    let _ = km::SetFocus(Some(combo));
                    wm::SendMessageW(combo, 0x014F /* CB_SHOWDROPDOWN */, Some(WPARAM(1)), None);
                }
                self.editor = Some((combo, i));
            }
            Ed::Dialog => self.open_dialog(i),
            Ed::Anchor => self.pick_anchor(i, cell),
        }
    }

    fn open_box(&mut self, cell: RECT, i: usize, value: &str) {
        let e = win::edit(self.grid, value, ID_EDITOR, self.font, false);
        win::move_to(e, cell.left, cell.top, cell.right - cell.left, cell.bottom - cell.top);
        unsafe {
            let _ = windows::Win32::UI::Shell::SetWindowSubclass(e, Some(box_keys), 11, 0);
            let _ = km::SetFocus(Some(e));
            wm::SendMessageW(e, 0x00B1 /* EM_SETSEL */, Some(WPARAM(0)), Some(LPARAM(-1)));
        }
        self.editor = Some((e, i));
    }

    /// Ends the in-place editor; `keep` puts what it holds into the model.
    fn close_editor(&mut self, keep: bool) {
        let Some((h, i)) = self.editor.take() else { return };
        let text = win::text_of(h);
        unsafe {
            let _ = wm::DestroyWindow(h);
        }
        if keep {
            if let Some(p) = self.props.get(i) {
                if text != p.value {
                    let key = p.key;
                    self.commit(key, &text);
                }
            }
        }
    }

    /// One change to the model, one undo step; the pane fills itself again afterwards.
    fn commit(&mut self, key: Key, value: &str) {
        let Some(s) = studio() else { return };
        if key == Key::StyleState {
            self.style_state = style_states().iter().position(|x| *x == value).unwrap_or(0);
            unsafe {
                let _ = wm::PostMessageW(Some(self.hwnd), WM_REFILL, WPARAM(0), LPARAM(0));
            }
            return;
        }
        s.checkpoint();
        s.apply(key, value);
        unsafe {
            let _ = wm::PostMessageW(Some(self.hwnd), WM_REFILL, WPARAM(0), LPARAM(0));
        }
    }

    /// The colour dialog, beside the click that opened it.
    fn pick_colour(&mut self, i: usize, value: &str) {
        if let Some(hex) = phai_ui::components::choose_colour(self.hwnd, value) {
            let key = self.props[i].key;
            self.commit(key, &hex);
        }
    }

    fn open_dialog(&mut self, i: usize) {
        let Some(s) = studio() else { return };
        let p = self.props[i].clone();
        match p.key {
            Key::FormTheme => s.command(crate::ID_THEME_IMPORT_PUB),
            Key::StyleToTheme => {
                let suggested = s.selected.and_then(|i| s.doc.form.control.get(i)).map(|c| c.id.replace('_', "-")).unwrap_or_default();
                if let Some(class) = phai_ui::dialogs::ask(Some(self.hwnd), t("Send to theme"), t("The class's name, as CSS writes it - letters, digits and -:"), &suggested, false) {
                    s.send_style_to_theme(class.trim());
                    unsafe {
                        let _ = wm::PostMessageW(Some(self.hwnd), WM_REFILL, WPARAM(0), LPARAM(0));
                    }
                }
            }
            // A grid's columns: the column editor.
            Key::Items if s.selected.and_then(|i| s.doc.form.control.get(i)).is_some_and(|c| c.kind == phai_form::Kind::Grid) => {
                s.edit_grid_columns();
                unsafe {
                    let _ = wm::PostMessageW(Some(self.hwnd), WM_REFILL, WPARAM(0), LPARAM(0));
                }
            }
            Key::Items => {
                if let Some(text) = phai_ui::dialogs::ask_lines(Some(self.hwnd), p.name, t("One a line; a list's columns as Name:width."), &p.value) {
                    self.commit(p.key, &text);
                }
            }
            _ => {}
        }
    }

    fn pick_anchor(&mut self, i: usize, cell: RECT) {
        let a = parse_anchor(Some(&self.props[i].value)).unwrap_or_default();
        let mut pt = windows::Win32::Foundation::POINT { x: cell.left, y: cell.bottom };
        unsafe {
            let _ = Gdi::ClientToScreen(self.grid, &mut pt);
        }
        if let Some(chosen) = anchor_picker(self.hwnd, pt.x, pt.y, a) {
            let key = self.props[i].key;
            self.commit(key, &anchor_text(chosen).unwrap_or_else(|| "left top".into()));
        }
    }

    fn command(&mut self, id: i32, code: u16) {
        let Some(s) = studio() else { return };
        if self.filling {
            return;
        }
        match (id, code) {
            (ID_MODE_PROPS, 0) | (ID_MODE_EVENTS, 0) | (ID_MODE_STYLE, 0) => {
                self.mode = match id {
                    ID_MODE_EVENTS => 1,
                    ID_MODE_STYLE => 2,
                    _ => 0,
                };
                self.say_bar();
                self.refresh();
            }
            (ID_CHOOSER, 1) => {
                let k = win::combo_selected(self.chooser);
                // The menu bar's or toolbar's own row (last): it stays chosen.
                if k as usize > s.doc.form.control.len() {
                    return;
                }
                s.select(if k <= 0 { None } else { Some(k as usize - 1) });
            }
            (ID_SORT_CAT, 0) | (ID_SORT_AZ, 0) => {
                self.alphabetical = id == ID_SORT_AZ;
                self.say_bar();
                self.arrange();
                self.chosen = None;
                self.update_scroll();
                unsafe {
                    let _ = Gdi::InvalidateRect(Some(self.grid), None, false);
                }
            }
            _ => {}
        }
    }
}

/// An event's key as a number to post: the control's events from 1, the form's from 1000, 0 the
/// control's one `on_...`.
pub fn event_key_index(key: Key) -> usize {
    match key {
        Key::On(n) => 1 + phai_form::events::CONTROL.iter().position(|e| e.name == n).unwrap_or(0),
        Key::FormOn(n) => 1000 + phai_form::events::FORM.iter().position(|e| e.name == n).unwrap_or(0),
        _ => 0,
    }
}

/// The key [`event_key_index`] posted.
pub fn event_key_of(i: usize) -> Key {
    match i {
        0 => Key::Event,
        1000.. => Key::FormOn(phai_form::events::FORM.get(i - 1000).map(|e| e.name).unwrap_or("shown")),
        _ => Key::On(phai_form::events::CONTROL.get(i - 1).map(|e| e.name).unwrap_or("click")),
    }
}

fn draw(dc: Gdi::HDC, text: &str, r: &mut RECT, colour: Color, align: Gdi::DRAW_TEXT_FORMAT) {
    if text.is_empty() {
        return;
    }
    let mut w: Vec<u16> = text.encode_utf16().collect();
    unsafe {
        Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(colour.colorref()));
        Gdi::DrawTextW(dc, &mut w, r, align | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX | Gdi::DT_END_ELLIPSIS);
    }
}

fn props_of<'a>(grid: HWND) -> Option<&'a mut Props> {
    studio().and_then(|s| s.props.as_mut()).filter(|p| p.grid == grid || p.hwnd == grid)
}

/// The value box keeps Enter (keep) and Escape (drop) for itself.
unsafe extern "system" fn box_keys(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, _: usize) -> LRESULT {
    use windows::Win32::UI::Shell::DefSubclassProc;
    match msg {
        wm::WM_GETDLGCODE => LRESULT(0x0004),
        wm::WM_KEYDOWN if w.0 == 0x0D || w.0 == 0x1B => {
            let grid = unsafe { wm::GetParent(h).unwrap_or_default() };
            if let Some(p) = props_of(grid) {
                p.close_editor(w.0 == 0x0D);
            }
            LRESULT(0)
        }
        wm::WM_CHAR if w.0 == 0x0D || w.0 == 0x1B => LRESULT(0),
        _ => unsafe { DefSubclassProc(h, msg, w, l) },
    }
}

extern "system" fn grid_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(p) = props_of(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_PAINT => {
            p.paint();
            LRESULT(0)
        }
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_SIZE => {
            p.update_scroll();
            LRESULT(0)
        }
        // The line between the names and the values, dragged as Visual Studio's.
        wm::WM_SETCURSOR if win::loword(l.0 as usize) == 1 /* HTCLIENT */ => {
            let mut pt = windows::Win32::Foundation::POINT::default();
            unsafe {
                let _ = wm::GetCursorPos(&mut pt);
                let _ = Gdi::ScreenToClient(h, &mut pt);
            }
            if p.dragging || p.on_split(pt.x) {
                unsafe {
                    let _ = wm::SetCursor(wm::LoadCursorW(None, wm::IDC_SIZEWE).ok());
                }
                return LRESULT(1);
            }
            win::def(h, msg, w, l)
        }
        wm::WM_LBUTTONDOWN if p.on_split(win::point_of(l).0) => {
            p.close_editor(true);
            p.dragging = true;
            unsafe {
                let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetCapture(h);
            }
            LRESULT(0)
        }
        // A double-click on the line: back to where it starts.
        wm::WM_LBUTTONDBLCLK if p.on_split(win::point_of(l).0) => {
            p.share = 450;
            crate::remember_setting_pub("props_split", "450");
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        wm::WM_MOUSEMOVE if p.dragging => {
            p.drag_split(win::point_of(l).0);
            LRESULT(0)
        }
        wm::WM_LBUTTONUP if p.dragging => {
            p.dragging = false;
            unsafe {
                let _ = windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
            }
            crate::remember_setting_pub("props_split", &p.share.to_string());
            LRESULT(0)
        }
        wm::WM_CAPTURECHANGED if p.dragging => {
            p.dragging = false;
            LRESULT(0)
        }
        wm::WM_LBUTTONDOWN | wm::WM_LBUTTONDBLCLK => {
            let (x, y) = win::point_of(l);
            p.click(x, y, msg == wm::WM_LBUTTONDBLCLK);
            LRESULT(0)
        }
        wm::WM_MOUSEWHEEL => {
            let delta = (w.0 >> 16) as i16 as i32;
            p.close_editor(true);
            p.top = (p.top as i64 - (delta / 40) as i64).max(0) as usize;
            p.update_scroll();
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        wm::WM_VSCROLL => {
            let mut si = wm::SCROLLINFO { cbSize: std::mem::size_of::<wm::SCROLLINFO>() as u32, fMask: wm::SIF_ALL, ..Default::default() };
            unsafe {
                let _ = wm::GetScrollInfo(h, wm::SB_VERT, &mut si);
            }
            let pos = match wm::SCROLLBAR_COMMAND(win::loword(w.0) as i32) {
                wm::SB_LINEUP => si.nPos - 1,
                wm::SB_LINEDOWN => si.nPos + 1,
                wm::SB_PAGEUP => si.nPos - si.nPage as i32,
                wm::SB_PAGEDOWN => si.nPos + si.nPage as i32,
                wm::SB_THUMBTRACK | wm::SB_THUMBPOSITION => si.nTrackPos,
                _ => si.nPos,
            };
            p.close_editor(true);
            p.top = pos.max(0) as usize;
            p.update_scroll();
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        // The in-place editor speaks: a choice made, a box left.
        wm::WM_COMMAND if win::loword(w.0) as i32 == ID_EDITOR => {
            match win::hiword(w.0) {
                1 /* CBN_SELCHANGE */ => {
                    // A list's choice is the value at once; a box that can be typed in waits.
                    let free = p.editor.as_ref().is_some_and(|(e, _)| unsafe { wm::GetWindowLongW(*e, wm::GWL_STYLE) } as u32 & 0x3 == 0x2);
                    if let Some((e, i)) = p.editor {
                        if !free {
                            let k = win::combo_selected(e).max(0) as usize;
                            let value = if let Ed::Choice(items, _) = &p.props[i].ed { items.get(k).cloned().unwrap_or_default() } else { String::new() };
                            let key = p.props[i].key;
                            p.editor = None;
                            unsafe {
                                let _ = wm::DestroyWindow(e);
                            }
                            p.commit(key, &value);
                        } else {
                            let k = win::combo_selected(e).max(0) as usize;
                            if let Ed::Choice(items, _) = &p.props[i].ed {
                                if let Some(v) = items.get(k) {
                                    win::set_text(e, v);
                                }
                            }
                        }
                    }
                }
                0x0200 /* EN_KILLFOCUS */ | 10 /* CBN_KILLFOCUS */ | 8 /* CBN_CLOSEUP */ if win::hiword(w.0) != 8 => p.close_editor(true),
                _ => {}
            }
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}

/// The chooser's rows: the name in bold, then the kind in the muted colour - as Visual Studio's.
fn draw_choice(l: LPARAM) {
    let d = unsafe { &*(l.0 as *const windows::Win32::UI::Controls::DRAWITEMSTRUCT) };
    if d.itemID == u32::MAX {
        return;
    }
    let Some(p) = studio().and_then(|s| s.props.as_ref()) else { return };
    let text = unsafe {
        let n = wm::SendMessageW(d.hwndItem, 0x0149 /* CB_GETLBTEXTLEN */, Some(WPARAM(d.itemID as usize)), None).0.max(0) as usize;
        let mut buf = vec![0u16; n + 1];
        wm::SendMessageW(d.hwndItem, 0x0148 /* CB_GETLBTEXT */, Some(WPARAM(d.itemID as usize)), Some(LPARAM(buf.as_mut_ptr() as isize)));
        String::from_utf16_lossy(&buf[..n])
    };
    let (name, kind) = text.split_once('\t').unwrap_or((&text, ""));
    let selected = d.itemState.0 & 0x0001 != 0 && d.itemState.0 & 0x1000 == 0; // ODS_SELECTED, not ODS_COMBOBOXEDIT
    let bg = if selected { phai_ui::theme::style("list", State { selected: true, ..Default::default() }).background_or(Color::rgb(0xe7, 0xf3, 0xef)) } else { win::window_background() };
    let muted = phai_ui::theme::current().var_color("--muted").unwrap_or(Color::rgb(0x5f, 0x5f, 0x5f));
    let dc = d.hDC;
    win::fill(dc, &d.rcItem, bg);
    unsafe {
        let pad = win::px(d.hwndItem, 4);
        let old = Gdi::SelectObject(dc, p.bold.into());
        let mut r = RECT { left: d.rcItem.left + pad, ..d.rcItem };
        draw(dc, name, &mut r, win::window_text(), Gdi::DT_LEFT);
        let (w, _) = win::measure(d.hwndItem, p.bold, name);
        Gdi::SelectObject(dc, p.font.into());
        let mut r = RECT { left: d.rcItem.left + pad + w + pad * 2, ..d.rcItem };
        draw(dc, kind, &mut r, muted, Gdi::DT_LEFT);
        Gdi::SelectObject(dc, old);
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_MEASUREITEM => {
            let m = unsafe { &mut *(l.0 as *mut windows::Win32::UI::Controls::MEASUREITEMSTRUCT) };
            m.itemHeight = win::px(h, 20) as u32;
            LRESULT(1)
        }
        wm::WM_DRAWITEM if w.0 as i32 == ID_CHOOSER => {
            draw_choice(l);
            LRESULT(1)
        }
        wm::WM_COMMAND => {
            if let Some(p) = studio().and_then(|s| s.props.as_mut()).filter(|p| p.hwnd == h) {
                p.command(win::loword(w.0) as i32, win::hiword(w.0));
            }
            LRESULT(0)
        }
        WM_REFILL => {
            if let Some(p) = studio().and_then(|s| s.props.as_mut()).filter(|p| p.hwnd == h) {
                p.refresh();
            }
            LRESULT(0)
        }
        wm::WM_SIZE => {
            if let Some(p) = studio().and_then(|s| s.props.as_ref()).filter(|p| p.hwnd == h) {
                p.layout();
            }
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}

// --- the anchor picker: a box with an arm to each edge, as Visual Studio's ---------------------

struct Picker {
    anchors: Anchors,
    done: bool,
    keep: bool,
}

fn arm_rects(h: HWND) -> [RECT; 4] {
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let (cx, cy) = (r.right / 2, r.bottom / 2);
    let (len, th, gap) = (r.right / 2 - win::px(h, 14), win::px(h, 8), win::px(h, 16));
    [
        RECT { left: cx - gap - len, top: cy - th / 2, right: cx - gap, bottom: cy + th / 2 },
        RECT { left: cx - th / 2, top: cy - gap - len / 2, right: cx + th / 2, bottom: cy - gap },
        RECT { left: cx + gap, top: cy - th / 2, right: cx + gap + len, bottom: cy + th / 2 },
        RECT { left: cx - th / 2, top: cy + gap, right: cx + th / 2, bottom: cy + gap + len / 2 },
    ]
}

extern "system" fn picker_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(pk) = (unsafe { win::state::<Picker>(h) }) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let dc = Gdi::BeginPaint(h, &mut ps);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            win::fill(dc, &r, win::window_background());
            let accent = phai_ui::theme::current().var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f));
            let off = phai_ui::theme::style("group", State::default()).border_or(Color::rgb(0xd4, 0xd4, 0xd4));
            let (cx, cy, s) = (r.right / 2, r.bottom / 2, win::px(h, 12));
            win::round_rect(dc, RECT { left: cx - s, top: cy - s, right: cx + s, bottom: cy + s }, win::window_background(), win::window_text(), win::px(h, 3));
            let on = [pk.anchors.left, pk.anchors.top, pk.anchors.right, pk.anchors.bottom];
            for (k, a) in arm_rects(h).iter().enumerate() {
                win::fill(dc, a, if on[k] { accent } else { off });
            }
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        wm::WM_LBUTTONDOWN => {
            let (x, y) = win::point_of(l);
            let grow = win::px(h, 8);
            for (k, a) in arm_rects(h).iter().enumerate() {
                if x >= a.left - grow && x < a.right + grow && y >= a.top - grow && y < a.bottom + grow {
                    match k {
                        0 => pk.anchors.left = !pk.anchors.left,
                        1 => pk.anchors.top = !pk.anchors.top,
                        2 => pk.anchors.right = !pk.anchors.right,
                        _ => pk.anchors.bottom = !pk.anchors.bottom,
                    }
                    unsafe {
                        let _ = Gdi::InvalidateRect(Some(h), None, false);
                    }
                }
            }
            LRESULT(0)
        }
        wm::WM_KEYDOWN if w.0 == 0x0D || w.0 == 0x1B => {
            pk.keep = w.0 == 0x0D;
            pk.done = true;
            LRESULT(0)
        }
        wm::WM_ACTIVATE if win::loword(w.0) == 0 => {
            // Clicked away: what was picked stands.
            pk.keep = true;
            pk.done = true;
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}

/// Shows the picker below `(x, y)` until it loses the focus; the anchors then, or `None` on Escape.
fn anchor_picker(owner: HWND, x: i32, y: i32, start: Anchors) -> Option<Anchors> {
    let class = win::register("PhaiAnchorPicker", Some(picker_proc), None);
    unsafe {
        let top = wm::GetAncestor(owner, wm::GA_ROOT);
        let (w, h) = (win::px(owner, 150), win::px(owner, 120));
        let p = wm::CreateWindowExW(wm::WS_EX_TOOLWINDOW, PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_POPUP | wm::WS_BORDER, x, y, w, h, Some(top), None, Some(win::instance()), None).ok()?;
        win::set_state(p, Box::new(Picker { anchors: start, done: false, keep: false }));
        let _ = wm::ShowWindow(p, wm::SW_SHOW);
        let _ = km::SetFocus(Some(p));
        let mut msg = wm::MSG::default();
        while win::state::<Picker>(p).is_some_and(|s| !s.done) && wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = wm::TranslateMessage(&msg);
            wm::DispatchMessageW(&msg);
        }
        let out = win::state::<Picker>(p).and_then(|s| s.keep.then_some(s.anchors));
        drop(win::take_state::<Picker>(p));
        let _ = wm::DestroyWindow(p);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composites_typed_in_whole_are_read() {
        assert_eq!(pair("95, 67"), Some((95, 67)));
        assert_eq!(pair("362 x 23"), Some((362, 23)));
        assert_eq!(pair("1, 2, 3"), None);
        assert_eq!(pair("a, 2"), None);
        assert_eq!(font_spec("Segoe UI, 10pt, Bold"), Some((Some(10), true)));
        assert_eq!(font_spec("12"), Some((Some(12), false)));
        assert_eq!(font_spec("Segoe UI, 99pt"), None);
        assert!(is_set(Key::FontAll, "Segoe UI, 9pt, Bold"));
        assert!(!is_set(Key::FontAll, "Segoe UI, 9pt"));
        assert!(!is_set(Key::Enabled, "True") && is_set(Key::Enabled, "False"));
    }
}
