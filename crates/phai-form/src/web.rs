//! A form for the web: `src/web/<form>_ui.rs` from the `.phf` - the same `<Form>Ui` struct
//! and `<Form>Events` trait as on Windows, made of `phai-web`'s controls - and the page around the
//! program: `index.html`, the theme as the browser's CSS, and what makes it an app a phone
//! installs (a manifest, a service worker that keeps it working offline, its icons).
//!
//! The program's own code (`<form>.rs`) is the same file for both: a project names `phai-web`
//! `phai-ui` when it builds for `wasm32`, so `phai_ui::ctl::TextBox` is the page's text box.

use std::fmt::Write;

use crate::codegen::combo_items;
use crate::events::{self as ev, Args, Source};
use crate::model::{snake, Control, FormFile, HasProps, Kind};

fn lit(s: &str) -> String {
    format!("{s:?}")
}

/// Whether a control of `kind` is on the page; what is not is left out of the web `Ui`.
pub fn on_web(kind: Kind) -> bool {
    !matches!(
        kind,
        Kind::OpenFile | Kind::SaveFile | Kind::FolderDialog | Kind::ColorDialog | Kind::FontDialog | Kind::ErrorProvider | Kind::NotifyIcon | Kind::PrintDocument | Kind::BindingSource | Kind::UserControl | Kind::Split
    )
}

/// What of `f` a browser does not have - shown to the person, each a sentence's key and the
/// control or part it is about.
pub fn left_out(f: &FormFile) -> Vec<String> {
    let mut out = Vec::new();
    for c in &f.control {
        if !on_web(c.kind) {
            out.push(format!("{} ({})", c.id, c.kind.name()));
        }
    }
    out
}

/// The grids bound to a table: on the web they ask the data server.
fn bound(f: &FormFile) -> Vec<&Control> {
    f.control.iter().filter(|c| c.kind == Kind::Grid && c.bind.is_some()).collect()
}

/// A bound grid's name on the data server: `main_window/customers`.
pub fn binding_name(f: &FormFile, c: &Control) -> String {
    format!("{}/{}", snake(&f.form.name), c.id)
}

/// `data-server/src/main.rs`: the program's data server, serving the tables of every bound grid
/// of `forms` (`None` when no grid is bound) - [`phai_data::server::run`] with the project's own
/// `data_sources.rs`.
pub fn data_server_main(forms: &[&FormFile]) -> Option<String> {
    let mut tables = Vec::new();
    for f in forms {
        for c in bound(f) {
            let Some(b) = &c.bind else { continue };
            let list = |v: &[String]| v.iter().map(|x| lit(x)).collect::<Vec<_>>().join(", ");
            tables.push(format!("        ({}, {}, {}, &[{}], &[{}]),", lit(&binding_name(f, c)), lit(&b.database), lit(&b.table), list(&b.columns), list(&b.keys)));
        }
    }
    if tables.is_empty() {
        return None;
    }
    Some(format!(
        r#"//! Made by Phai Studio from the forms' bound grids - do not change this file; it is written again
//! with each build for the web. The web app's tables, served to it over HTTP: run it beside the
//! database, behind the web server's HTTPS at /phai-data/.
//!
//! cargo run --release      (in this folder)
//! PHAI_DB_PASSWORD   the database's password, as the program on Windows takes it
//! PHAI_DATA_TOKEN    the access key the app asks for once on each device (needed off this computer)
//! PHAI_DATA_ADDR     where it listens, 127.0.0.1:8791 when unset

#[path = "../../src/data_sources.rs"]
mod data_sources;

fn main() {{
    let tables: &[phai_data::server::Table] = &[
{}
    ];
    if let Err(e) = phai_data::server::run(data_sources::connection(), tables) {{
        eprintln!("{{e}}");
        std::process::exit(1);
    }}
}}
"#,
        tables.join("
")
    ))
}

/// `data-server/Cargo.toml`: a package of its own (its own workspace), needing phai-data alone -
/// `phai_data` how the project names it.
pub fn data_server_toml(package: &str, phai_data: &str) -> String {
    format!(
        r#"[package]
name = "{package}-data-server"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
phai-data = {phai_data}

[workspace]
"#
    )
}

/// The tab control the pages belong to: the form's first.
fn tabs_id(f: &FormFile) -> Option<&str> {
    f.control.iter().find(|c| c.kind == Kind::Tabs).map(|c| c.id.as_str())
}

fn place(c: &Control, f: &FormFile) -> String {
    let [x, y, w, h] = c.rect();
    let dock = match c.dock.as_deref() {
        Some(d) if d != "none" => d,
        _ if c.kind == Kind::Statusbar => "bottom",
        _ => "none",
    };
    format!(
        "Place {{ x: {x}, y: {y}, w: {w}, h: {h}, anchor: {}, dock: {}, within: ({}, {}) }}",
        lit(c.anchor.as_deref().unwrap_or("")),
        lit(dock),
        f.form.size[0],
        f.form.size[1]
    )
}

/// Whether `f` stacks its controls on a screen `width` wide (PhoneLayout = stack, narrower than
/// the form).
pub fn stacks(f: &FormFile, width: i32) -> bool {
    f.form.word("phone") != Some("zoom") && width < f.form.size[0]
}

/// Where each control of `f` goes when stacked on a screen `width` x `height`, as the page's
/// CSS puts it (`.phai-stacked`): in reading order, as wide as the screen less its margins,
/// tall enough for a finger. `None`: not shown that way (hidden, a component, another page, a
/// frame its controls are stacked out of). `top`: where the first one starts (below a toolbar).
pub fn stacked_rects(f: &FormFile, width: i32, height: i32, top: i32) -> Vec<Option<[i32; 4]>> {
    const PAD: i32 = 12;
    const GAP: i32 = 10;
    let mut order: Vec<(i64, usize)> = Vec::new();
    for (i, c) in f.control.iter().enumerate() {
        let gone = c.visible == Some(false) || c.kind.is_component() || c.page.is_some_and(|p| p != 0) || (matches!(c.kind, Kind::Panel | Kind::Flow | Kind::Table | Kind::Split) && !c.events.contains_key("paint"));
        if gone {
            continue;
        }
        let [x, y, _, _] = c.rect();
        let key = match c.dock.as_deref() {
            Some("top") => -1_000_000 + y as i64,
            Some("bottom") => 100_000_000,
            _ if c.kind == Kind::Statusbar => 100_000_000,
            _ => y.clamp(0, 9_999) as i64 * 10_000 + x.clamp(0, 9_999) as i64,
        };
        order.push((key, i));
    }
    order.sort();
    let mut out = vec![None; f.control.len()];
    let mut y = top + PAD;
    for (key, i) in order {
        let c = &f.control[i];
        let [_, _, _, h] = c.rect();
        if key == 100_000_000 {
            // The status bar: along the bottom, the whole width.
            out[i] = Some([0, height - h, width, h]);
            continue;
        }
        let h = match c.kind {
            Kind::Button | Kind::Edit | Kind::Combo | Kind::Date | Kind::Checkbox | Kind::Radio | Kind::Number | Kind::Masked => h.max(44),
            Kind::Textarea | Kind::List | Kind::Grid | Kind::ListBox | Kind::CheckedList => h.max(180),
            Kind::Group => 24,
            _ => h,
        };
        out[i] = Some([PAD, y, width - 2 * PAD, h]);
        // A label sits close over what it names.
        y += h + if c.kind == Kind::Label { GAP - 6 } else { GAP };
    }
    out
}

/// The browser event a control's own method (`on_click`, `on_change`, `on_select`) listens to.
fn own_event(c: &Control) -> Option<&'static str> {
    match c.kind {
        Kind::Button | Kind::Checkbox | Kind::Radio | Kind::Link => Some("click"),
        Kind::Edit | Kind::Textarea | Kind::Rich | Kind::Masked | Kind::Number | Kind::Slider | Kind::Grid => Some("input"),
        Kind::Date | Kind::Calendar | Kind::Combo | Kind::CheckedList | Kind::ListBox => Some("change"),
        Kind::List | Kind::Tabs => Some("click"),
        // A tree raises `change` when another item is chosen - its AfterSelect.
        Kind::Tree => Some("change"),
        _ => None,
    }
}

/// The browser event for a table event of a control (`events = { key_down = ... }`).
fn hook_event(name: &str) -> Option<&'static str> {
    Some(match name {
        "click" => "click",
        "double_click" => "dblclick",
        "enter" => "focusin",
        "leave" => "focusout",
        "mouse_enter" => "mouseenter",
        "mouse_leave" => "mouseleave",
        "key_down" => "keydown",
        "key_up" => "keyup",
        "key_press" => "keypress",
        "mouse_down" => "mousedown",
        "mouse_up" => "mouseup",
        "mouse_move" => "mousemove",
        "mouse_wheel" => "wheel",
        "paint" => "paint",
        "checked_changed" => "change",
        "text_changed" => "input",
        _ => return None,
    })
}

fn glyph_code(name: Option<&str>) -> u16 {
    name.and_then(phai_ui::icon::glyph::by_name).unwrap_or(0)
}

/// A menu's items as `phai_ui::menu::Item` literals.
fn items_code(items: &[crate::model::MenuItem]) -> String {
    let all: Vec<String> = items
        .iter()
        .map(|it| {
            if it.separator {
                return "phai_ui::menu::Item { separator: true, ..Default::default() }".to_string();
            }
            format!(
                "phai_ui::menu::Item {{ id: {}, label: t({}), key: {}, glyph: {}, checked: {}, check_on_click: {}, disabled: {}, separator: false, items: {} }}",
                lit(it.id.as_deref().unwrap_or("")),
                lit(it.label.as_deref().unwrap_or("")),
                lit(it.key.as_deref().unwrap_or("")),
                glyph_code(it.glyph.as_deref()),
                it.checked,
                it.check_on_click,
                it.disabled,
                items_code(&it.items)
            )
        })
        .collect();
    format!("vec![{}]", all.join(", "))
}

/// The ids of the form's menu items and toolbar buttons, each once.
fn item_ids(f: &FormFile) -> Vec<String> {
    let mut v: Vec<String> = f.menu.iter().flat_map(|m| m.all_items()).filter_map(|it| it.id.clone()).collect();
    for t in f.toolbar.iter().filter(|t| !t.separator) {
        if !v.contains(&t.id) {
            v.push(t.id.clone());
        }
    }
    // A context menu's items too.
    for c in f.control.iter().filter(|c| c.kind == Kind::ContextMenu) {
        for (id, _) in c.menu_items() {
            if !id.is_empty() && !v.contains(&id) {
                v.push(id);
            }
        }
    }
    v
}

/// The toolbar's height in the designer: Windows' own.
pub const TOOLBAR_HEIGHT: i32 = 34;

/// `src/web/<form>_ui.rs`.
pub fn generate(f: &FormFile) -> String {
    let name = &f.form.name;
    let file = snake(name);
    let ui = format!("{name}Ui");
    let events = format!("{name}Events");
    let handlers = f.handlers();
    let mut o = String::new();
    let w = &mut o;
    let _ = writeln!(w, "// Made by Phai Studio from {file}.phf for the web - do not change this file; change the form.");
    let _ = writeln!(w, "// The same {ui} and {events} as {file}_ui.rs, on a web page.");
    let _ = writeln!(w);
    let _ = writeln!(w, "#![allow(dead_code, unused_imports, unused_variables, unused_mut, clippy::all)]");
    let _ = writeln!(w);
    let _ = writeln!(w, "use std::cell::{{OnceCell, RefCell}};");
    let _ = writeln!(w, "use std::rc::{{Rc, Weak}};");
    let _ = writeln!(w, "use phai_ui::form::{{self, Place}};");
    let _ = writeln!(w, "use phai_ui::text::t;");
    let _ = writeln!(w, "use phai_ui::win::HWND;");
    let _ = writeln!(w);
    let _ = writeln!(w, "/// Every English sentence this window shows.");
    let _ = writeln!(w, "pub const TEXTS: &[&str] = &[");
    for t in f.texts() {
        let _ = writeln!(w, "    {},", lit(&t));
    }
    let _ = writeln!(w, "];");
    let _ = writeln!(w);

    let shown: Vec<&Control> = f.control.iter().filter(|c| on_web(c.kind)).collect();
    let _ = writeln!(w, "/// The window and every control in it, by the ids the form gives them.");
    let _ = writeln!(w, "pub struct {ui} {{");
    let _ = writeln!(w, "    pub hwnd: HWND,");
    let _ = writeln!(w, "    area: HWND,");
    if !f.toolbar.is_empty() {
        let _ = writeln!(w, "    pub toolbar: HWND,");
    }
    for c in &shown {
        let _ = writeln!(w, "    pub {}: phai_ui::ctl::{},", c.id, crate::codegen::ctl_type(c.kind));
    }
    // A bound grid's table, on the program's data server.
    for c in bound(f) {
        let _ = writeln!(w, "    pub {}_data: phai_ui::data::Binding,", c.id);
    }
    let _ = writeln!(w, "}}");
    let _ = writeln!(w);

    // The trait: the same methods as on Windows, so the program's one impl serves both.
    let _ = writeln!(w, "/// What the window tells the program. Every method does nothing until the program says otherwise.");
    let _ = writeln!(w, "pub trait {events} {{");
    let _ = writeln!(w, "    /// The window is made and about to show.");
    let _ = writeln!(w, "    fn loaded(&mut self, ui: &{ui}) {{}}");
    for h in &handlers {
        let _ = writeln!(w, "    fn {h}(&mut self, ui: &{ui}) {{}}");
    }
    for (h, a) in f.event_methods().into_iter().filter(|(h, _)| !handlers.contains(h)) {
        let _ = writeln!(w, "    fn {h}(&mut self, ui: &{ui}{}) {{}}", a.param().map(|p| format!(", e: &{p}")).unwrap_or_default());
    }
    let _ = writeln!(w, "}}");
    let _ = writeln!(w);
    let _ = writeln!(w, "struct Host<E> {{");
    let _ = writeln!(w, "    ui: {ui},");
    let _ = writeln!(w, "    events: E,");
    let _ = writeln!(w, "}}");
    let _ = writeln!(w);
    let _ = writeln!(w, "type Me<E> = Rc<OnceCell<Weak<RefCell<Host<E>>>>>;");
    let _ = writeln!(w);
    let _ = writeln!(w, "/// A method of the program's, run when the page says - unless one is running already.");
    let _ = writeln!(w, "fn call<E: {events} + 'static>(me: &Me<E>, f: impl Fn(&mut E, &{ui}) + 'static) -> impl FnMut() + 'static {{");
    let _ = writeln!(w, "    let me = me.clone();");
    let _ = writeln!(w, "    move || {{");
    let _ = writeln!(w, "        if let Some(h) = me.get().and_then(|w| w.upgrade()) {{");
    let _ = writeln!(w, "            if let Ok(mut h) = h.try_borrow_mut() {{");
    let _ = writeln!(w, "                let Host {{ ui, events }} = &mut *h;");
    let _ = writeln!(w, "                f(events, ui);");
    let _ = writeln!(w, "            }}");
    let _ = writeln!(w, "        }}");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w, "}}");
    let _ = writeln!(w);
    let _ = writeln!(w, "/// The same, given what happened.");
    let _ = writeln!(w, "fn call_with<E: {events} + 'static, A>(me: &Me<E>, f: impl Fn(&mut E, &{ui}, &A) + 'static) -> impl FnMut(&A) + 'static {{");
    let _ = writeln!(w, "    let me = me.clone();");
    let _ = writeln!(w, "    move |a: &A| {{");
    let _ = writeln!(w, "        if let Some(h) = me.get().and_then(|w| w.upgrade()) {{");
    let _ = writeln!(w, "            if let Ok(mut h) = h.try_borrow_mut() {{");
    let _ = writeln!(w, "                let Host {{ ui, events }} = &mut *h;");
    let _ = writeln!(w, "                f(events, ui, a);");
    let _ = writeln!(w, "            }}");
    let _ = writeln!(w, "        }}");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w, "}}");
    let _ = writeln!(w);

    let ids = item_ids(f);
    if !ids.is_empty() {
        let _ = writeln!(w, "/// A menu item or toolbar button chosen: its method.");
        let _ = writeln!(w, "fn chosen<E: {events} + 'static>(me: &Me<E>) -> impl FnMut(&String) + 'static {{");
        let _ = writeln!(w, "    call_with(me, |e: &mut E, ui: &{ui}, id: &String| match id.as_str() {{");
        for id in ids.iter().filter(|id| handlers.contains(id)) {
            let _ = writeln!(w, "        {} => e.{id}(ui),", lit(id));
        }
        let _ = writeln!(w, "        _ => {{}}");
        let _ = writeln!(w, "    }})");
        let _ = writeln!(w, "}}");
        let _ = writeln!(w);
    }
    let _ = writeln!(w, "impl {ui} {{");
    if !ids.is_empty() {
        let _ = writeln!(w, "    /// Whether menu item (or toolbar button) `item` is ticked - its id in the form.");
        let _ = writeln!(w, "    pub fn item_checked(&self, item: &str) -> bool {{");
        let _ = writeln!(w, "        phai_ui::menu::item_checked(self.hwnd, item)");
        let _ = writeln!(w, "    }}");
        let _ = writeln!(w);
        let _ = writeln!(w, "    /// Ticks menu item (and toolbar button) `item`, or takes the tick away - Checked.");
        let _ = writeln!(w, "    pub fn set_item_checked(&self, item: &str, on: bool) {{");
        let _ = writeln!(w, "        phai_ui::menu::set_item_checked(self.hwnd, item, on)");
        let _ = writeln!(w, "    }}");
        let _ = writeln!(w);
        let _ = writeln!(w, "    /// Menu item (and toolbar button) `item` usable or greyed - Enabled.");
        let _ = writeln!(w, "    pub fn set_item_enabled(&self, item: &str, on: bool) {{");
        let _ = writeln!(w, "        phai_ui::menu::set_item_enabled(self.hwnd, item, on)");
        let _ = writeln!(w, "    }}");
        let _ = writeln!(w);
    }
    let _ = writeln!(w, "    /// Puts the window on the page; the page runs it.");
    let _ = writeln!(w, "    pub fn run<E: {events} + 'static>(events: E) {{");
    let _ = writeln!(w, "        Self::make(true, events);");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    let _ = writeln!(w, "    pub fn create<E: {events} + 'static>(events: E) -> HWND {{");
    let _ = writeln!(w, "        Self::make(true, events)");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    let _ = writeln!(w, "    /// Shows this window over the page - as `Show()`.");
    let _ = writeln!(w, "    pub fn show<E: {events} + 'static>(_owner: HWND, events: E) -> HWND {{");
    let _ = writeln!(w, "        Self::make(false, events)");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    let _ = writeln!(w, "    /// Shows this window over the page. A page cannot wait for it: the answer is `None`");
    let _ = writeln!(w, "    /// at once, and the window's own buttons do the work.");
    let _ = writeln!(w, "    pub fn show_dialog<E: {events} + 'static>(_owner: HWND, events: E) -> form::DialogResult {{");
    let _ = writeln!(w, "        Self::make(false, events);");
    let _ = writeln!(w, "        form::DialogResult::None");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    for c in bound(f) {
        let id = &c.id;
        let _ = writeln!(w, "    /// {id}'s rows read again from the data server - the grid filled when they come.");
        let _ = writeln!(w, "    pub fn reload_{id}(&self) -> Result<(), String> {{");
        let _ = writeln!(w, "        self.{id}_data.load_into(self.{id}.hwnd());");
        let _ = writeln!(w, "        Ok(())");
        let _ = writeln!(w, "    }}");
        let _ = writeln!(w);
        let _ = writeln!(w, "    /// What {id} holds written back through the data server, then read again. On the web the");
        let _ = writeln!(w, "    /// answer comes later: 0 here, a message if it fails.");
        let _ = writeln!(w, "    pub fn save_{id}(&self) -> Result<usize, String> {{");
        let _ = writeln!(w, "        self.{id}_data.save_from(self.{id}.hwnd());");
        let _ = writeln!(w, "        Ok(0)");
        let _ = writeln!(w, "    }}");
        let _ = writeln!(w);
    }
    let _ = writeln!(w, "    /// Closes the window.");
    let _ = writeln!(w, "    pub fn close(&self) {{");
    let _ = writeln!(w, "        form::close(self.hwnd);");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    let _ = writeln!(w, "    fn make<E: {events} + 'static>(main: bool, events: E) -> HWND {{");
    let _ = writeln!(w, "        form::start();");
    let _ = writeln!(w, "        let (hwnd, area) = form::window({}, t({}), ({}, {}), main);", lit(name), lit(&f.form.title), f.form.size[0], f.form.size[1]);
    // PhoneLayout: stacked on a narrow screen unless the form says zoom.
    if f.form.word("phone") != Some("zoom") {
        let _ = writeln!(w, "        form::stack_when_narrow(area);");
    }
    let tabs = tabs_id(f);
    for c in &shown {
        let text = c.text.as_deref().unwrap_or("");
        let words = if text.is_empty() || matches!(c.kind, Kind::Edit | Kind::Textarea | Kind::Rich | Kind::Masked | Kind::Number) { lit(text) } else { format!("t({})", lit(text)) };
        // A context menu has no element: it opens over the control that has it.
        if c.kind == Kind::ContextMenu {
            let _ = writeln!(w, "        let {} = HWND::default();", c.id);
            continue;
        }
        let _ = writeln!(w, "        let {} = form::control(area, {}, {}, {}, {words}, {});", c.id, lit(c.kind.name()), lit(&c.id), lit(c.class.as_deref().unwrap_or("")), place(c, f));
        match c.kind {
            Kind::Tree => {
                let lines = c.items.iter().map(|s| lit(s)).collect::<Vec<_>>().join(", ");
                let _ = writeln!(w, "        phai_ui::tree::set_lines({}, &[{lines}]);", c.id);
                if c.flag("expand_all", false) {
                    let _ = writeln!(w, "        phai_ui::ctl::TreeView({}).expand_all();", c.id);
                }
            }
            Kind::Combo => {
                let items = combo_items(c).iter().map(|s| format!("t({})", lit(s))).collect::<Vec<_>>().join(", ");
                let _ = writeln!(w, "        form::set_items({}, &[{items}]);", c.id);
                if !text.is_empty() {
                    let _ = writeln!(w, "        phai_ui::ctl::Control::set_text(phai_ui::ctl::ComboBox({}), t({}));", c.id, lit(text));
                }
            }
            Kind::ListBox | Kind::CheckedList => {
                let items = c.items.iter().map(|s| format!("t({})", lit(s))).collect::<Vec<_>>().join(", ");
                let _ = writeln!(w, "        form::set_items({}, &[{items}]);", c.id);
                if c.word("selection_mode") == Some("multi") || c.kind == Kind::CheckedList {
                    let _ = writeln!(w, "        form::set_multi({});", c.id);
                }
            }
            Kind::List | Kind::Grid => {
                let cols = c.columns.iter().map(|(s, n)| format!("(t({}), {n})", lit(s))).collect::<Vec<_>>().join(", ");
                let _ = writeln!(w, "        form::set_columns({}, &[{cols}]);", c.id);
            }
            Kind::Tabs => {
                let pages = c.tabs.iter().map(|s| format!("t({})", lit(s))).collect::<Vec<_>>().join(", ");
                let _ = writeln!(w, "        form::set_tabs({}, area, &[{pages}]);", c.id);
            }
            Kind::Edit if c.password == Some(true) => {
                let _ = writeln!(w, "        form::set_password({});", c.id);
            }
            Kind::Radio => {
                let _ = writeln!(w, "        form::set_radio_group({}, {}, {});", c.id, lit(name), c.group.unwrap_or(0));
            }
            Kind::Number | Kind::Slider => {
                let (lo, hi) = (c.min.unwrap_or(0), c.max.unwrap_or(100));
                let _ = writeln!(w, "        form::set_range({}, {lo}, {hi});", c.id);
                if let Some(v) = c.value {
                    let _ = writeln!(w, "        phai_ui::ctl::NumericUpDown({}).set_value({v});", c.id);
                }
            }
            Kind::Progress => {
                if let Some(v) = c.value {
                    let _ = writeln!(w, "        phai_ui::ctl::ProgressBar({}).set_value({v});", c.id);
                }
            }
            Kind::Picture => {
                if let Some(img) = &c.image {
                    let _ = writeln!(w, "        phai_ui::ctl::PictureBox({}).load({});", c.id, lit(img));
                }
            }
            _ => {}
        }
        if matches!(c.kind, Kind::Checkbox | Kind::Radio) && c.checked == Some(true) {
            let _ = writeln!(w, "        form::set_checked({}, true);", c.id);
        }
        if let (Some(p), Some(t)) = (c.page, tabs) {
            if c.kind != Kind::Tabs {
                let _ = writeln!(w, "        form::set_page({}, {}, {p});", c.id, lit(t));
            }
        }
        if let Some(tip) = c.word("tooltip").filter(|s| !s.is_empty()) {
            let _ = writeln!(w, "        phai_ui::ctl::Control::set_tooltip(phai_ui::ctl::Label({}), t({}));", c.id, lit(tip));
        }
        if let Some(p) = c.word("placeholder").filter(|s| !s.is_empty()) {
            let _ = writeln!(w, "        if let Some(e) = phai_ui::win::el({}) {{ let _ = e.set_attribute(\"placeholder\", t({})); }}", c.id, lit(p));
        }
        if c.flag("read_only", false) {
            let _ = writeln!(w, "        phai_ui::ctl::TextBox({}).set_read_only(true);", c.id);
        }
        if c.enabled == Some(false) {
            let _ = writeln!(w, "        phai_ui::ctl::Control::set_enabled(phai_ui::ctl::Label({}), false);", c.id);
        }
        if c.visible == Some(false) {
            let _ = writeln!(w, "        phai_ui::ctl::Control::set_visible(phai_ui::ctl::Label({}), false);", c.id);
        }
    }
    let _ = writeln!(w, "        let me: Me<E> = Rc::new(OnceCell::new());");
    if !f.menu.is_empty() {
        let menus = f.menu.iter().map(|m| format!("phai_ui::menu::Menu {{ label: t({}), items: {} }}", lit(&m.label), items_code(&m.items))).collect::<Vec<_>>().join(", ");
        let _ = writeln!(w, "        let menus = vec![{menus}];");
        let _ = writeln!(w, "        phai_ui::menu::shortcuts(hwnd, &menus, chosen(&me));");
        let _ = writeln!(w, "        phai_ui::menu::menu_bar(hwnd, menus, chosen(&me));");
    }
    if !f.toolbar.is_empty() {
        let tools = f.toolbar.iter().filter(|t| !t.separator).map(|t| format!("phai_ui::menu::Tool {{ id: {}, label: t({}), glyph: {}, toggle: {} }}", lit(&t.id), lit(&t.label), glyph_code(Some(&t.glyph)), t.toggle)).collect::<Vec<_>>().join(", ");
        let _ = writeln!(w, "        let toolbar = phai_ui::menu::toolbar(hwnd, area, vec![{tools}], {TOOLBAR_HEIGHT}, chosen(&me));");
    }
    // Timers tick into the program once it is there.
    for c in shown.iter().filter(|c| c.kind == Kind::Timer) {
        let tick = c.handler().map(|h| format!("call(&me, |e: &mut E, ui: &{ui}| e.{h}(ui))")).unwrap_or_else(|| "|| {}".to_string());
        let _ = writeln!(w, "        let {} = form::timer({}, {}, {tick});", c.id, c.value.unwrap_or(1000).max(1), c.enabled != Some(false));
    }
    let _ = writeln!(w, "        let ui = {ui} {{");
    let _ = writeln!(w, "            hwnd,");
    let _ = writeln!(w, "            area,");
    if !f.toolbar.is_empty() {
        let _ = writeln!(w, "            toolbar,");
    }
    for c in &shown {
        if c.kind == Kind::Timer {
            let _ = writeln!(w, "            {},", c.id);
        } else {
            let _ = writeln!(w, "            {}: phai_ui::ctl::{}({}),", c.id, crate::codegen::ctl_type(c.kind), c.id);
        }
    }
    for c in bound(f) {
        let _ = writeln!(w, "            {}_data: phai_ui::data::Binding::new({}),", c.id, lit(&binding_name(f, c)));
    }
    let _ = writeln!(w, "        }};");
    // The controls' methods.
    for c in &shown {
        if c.kind == Kind::Timer {
            continue;
        }
        // Save or Reload for a bound grid, then the button's own method.
        let bound_button = if let Some((grid, what)) = c.bind_action.as_deref().and_then(|a| a.split_once(':')).filter(|(g, _)| f.control.iter().any(|x| x.id == *g && x.kind == Kind::Grid && x.bind.is_some())) {
            let run = if what == "save" { format!("ui.save_{grid}().map(|_| ())") } else { format!("ui.reload_{grid}()") };
            let after = c.handler().map(|h| format!(" e.{h}(ui);")).unwrap_or_default();
            let _ = writeln!(w, "        form::on(ui.{}.hwnd(), \"click\", call(&me, |e: &mut E, ui: &{ui}| {{ if let Err(m) = {run} {{ phai_ui::win::message(&m); }}{after} }}));", c.id);
            true
        } else {
            false
        };
        if let (Some(h), Some(on), false) = (c.handler(), own_event(c), bound_button) {
            if c.kind == Kind::Link {
                let _ = writeln!(w, "        form::on_link(ui.{}.hwnd(), call(&me, |e: &mut E, ui: &{ui}| e.{h}(ui)));", c.id);
            } else {
                let _ = writeln!(w, "        form::on(ui.{}.hwnd(), {}, call(&me, |e: &mut E, ui: &{ui}| e.{h}(ui)));", c.id, lit(on));
            }
        }
        for (e, h) in &c.events {
            let Some(spec) = ev::find(c.kind, e) else { continue };
            let Some(browser) = hook_event(e) else { continue };
            if !matches!(spec.source, Source::Hook(_) | Source::Command(_)) && e != "text_changed" {
                continue;
            }
            match spec.args {
                Args::None => {
                    let _ = writeln!(w, "        form::on(ui.{}.hwnd(), {}, call(&me, |e: &mut E, ui: &{ui}| e.{h}(ui)));", c.id, lit(browser));
                }
                Args::Key => {
                    let _ = writeln!(w, "        form::on_key(ui.{}.hwnd(), {}, call_with(&me, |e: &mut E, ui: &{ui}, a: &phai_ui::events::KeyEventArgs| e.{h}(ui, a)));", c.id, lit(browser));
                }
                Args::KeyPress => {
                    let _ = writeln!(w, "        form::on_key_press(ui.{}.hwnd(), call_with(&me, |e: &mut E, ui: &{ui}, a: &phai_ui::events::KeyPressEventArgs| e.{h}(ui, a)));", c.id);
                }
                Args::Mouse if e == "mouse_wheel" => {
                    let _ = writeln!(w, "        form::on_wheel(ui.{}.hwnd(), call_with(&me, |e: &mut E, ui: &{ui}, a: &phai_ui::events::MouseEventArgs| e.{h}(ui, a)));", c.id);
                }
                Args::Paint => {
                    let _ = writeln!(w, "        phai_ui::paint::on_paint(ui.{}.hwnd(), call_with(&me, |e: &mut E, ui: &{ui}, a: &phai_ui::paint::PaintEventArgs| e.{h}(ui, a)));", c.id);
                }
                Args::Mouse => {
                    let _ = writeln!(w, "        form::on_mouse(ui.{}.hwnd(), {}, call_with(&me, |e: &mut E, ui: &{ui}, a: &phai_ui::events::MouseEventArgs| e.{h}(ui, a)));", c.id, lit(browser));
                }
                _ => {}
            }
        }
    }
    // Context menus: a right-click (a long press on a phone) on the control that has one.
    for c in shown.iter().filter(|c| c.kind != Kind::ContextMenu) {
        let Some(menu) = c.context_menu.as_ref().and_then(|m| f.control.iter().find(|x| x.kind == Kind::ContextMenu && &x.id == m)) else { continue };
        let items = menu
            .menu_items()
            .iter()
            .map(|(id, label)| if id.is_empty() { "phai_ui::menu::Item { separator: true, ..Default::default() }".to_string() } else { format!("phai_ui::menu::Item {{ id: {}, label: t({}), ..Default::default() }}", lit(id), lit(label)) })
            .collect::<Vec<_>>()
            .join(", ");
        let opening = menu.events.get("opening").map(|h| format!("call(&me, |e: &mut E, ui: &{ui}| e.{h}(ui))")).unwrap_or_else(|| "|| {}".to_string());
        let _ = writeln!(w, "        phai_ui::menu::context_menu(hwnd, ui.{}.hwnd(), vec![{items}], {opening}, chosen(&me));", c.id);
    }
    let _ = writeln!(w, "        let host = Rc::new(RefCell::new(Host {{ ui, events }}));");
    let _ = writeln!(w, "        let _ = me.set(Rc::downgrade(&host));");
    let _ = writeln!(w, "        {{");
    let _ = writeln!(w, "            let mut h = host.borrow_mut();");
    let _ = writeln!(w, "            let Host {{ ui, events }} = &mut *h;");
    for c in bound(f) {
        let _ = writeln!(w, "            let _ = ui.reload_{}();", c.id);
    }
    let _ = writeln!(w, "            events.loaded(ui);");
    let _ = writeln!(w, "        }}");
    if let Some(h) = f.form.events.get("shown") {
        let _ = writeln!(w, "        let mut shown = call(&me, |e: &mut E, ui: &{ui}| e.{h}(ui));");
        let _ = writeln!(w, "        form::after_shown(move || shown());");
    }
    let _ = writeln!(w, "        // The page keeps the window's program as long as it is open.");
    let _ = writeln!(w, "        std::mem::forget(host);");
    let _ = writeln!(w, "        hwnd");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w, "}}");
    o
}

/// A selector of the theme's (`button.primary:hover`, `edit, combo`) as the browser's.
fn selector(s: &str) -> String {
    s.split(',')
        .map(|one| {
            let one = one.trim();
            if one.starts_with(":root") {
                return one.to_string();
            }
            // The kind first, then classes, an id and states.
            let end = one.find(['.', '#', ':']).unwrap_or(one.len());
            let (kind, rest) = one.split_at(end);
            let rest = rest.replace(":pressed", ":active").replace(":selected", ".selected").replace(":checked", ":has(input:checked)").replace(":disabled", ".disabled");
            let rest = rest.replace(":focus", ":focus-within");
            match kind {
                "" | "*" => format!(".phai{rest}"),
                "window" => format!(".phai-window{rest}"),
                k => format!(".phai-{k}{rest}"),
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// A theme's declarations as the browser's: a border its width says, sizes in pixels, what only
/// Windows draws left out.
fn declarations(kind: &str, body: &str) -> String {
    let mut out = Vec::new();
    for d in body.split(';') {
        let Some((p, v)) = d.split_once(':') else { continue };
        let (p, v) = (p.trim(), v.trim());
        match p {
            "icon-color" => continue,
            // On Windows the tick's colour; a browser's check box takes the colour it is filled with.
            "accent-color" if matches!(kind, "checkbox" | "radio") => continue,
            "background" | "background-color" if matches!(kind, "checkbox" | "radio") => {
                out.push(format!("--phai-check: {v}"));
                continue;
            }
            "border-width" | "border-radius" => {
                let v = if v.chars().all(|c| c.is_ascii_digit()) { format!("{v}px") } else { v.to_string() };
                if p == "border-width" {
                    out.push("border-style: solid".to_string());
                }
                out.push(format!("{p}: {v}"));
                continue;
            }
            _ => {}
        }
        if p.starts_with("--") || !p.is_empty() {
            out.push(format!("{p}: {v}"));
        }
    }
    out.join("; ")
}

/// A theme file (with its `@import "phai:light"`) as the page's CSS.
/// The page's CSS for the forms' grids, as the Windows grid draws them: every other row's
/// colour, the headings' colours, each column's alignment, hidden columns left out.
pub fn grids_css(forms: &[&FormFile]) -> String {
    let mut out = String::new();
    for c in forms.iter().flat_map(|f| f.control.iter()).filter(|c| c.kind == Kind::Grid) {
        let id = &c.id;
        let colour = |name: &str| c.word(name).filter(|v| phai_ui::theme::parse_color(v).is_some()).map(str::to_string);
        if let Some(v) = colour("alternate_row_color") {
            out += &format!("#{id} tbody tr:nth-child(even) td {{ background: {v}; }}\n");
        }
        if let Some(v) = colour("header_back_color") {
            out += &format!("#{id} th {{ background: {v}; }}\n");
        }
        if let Some(v) = colour("header_fore_color") {
            out += &format!("#{id} th {{ color: {v}; }}\n");
        }
        if c.int("row_height").is_some_and(|h| h > 0) {
            out += &format!("#{id} td {{ height: {}px; }}\n", c.int("row_height").unwrap_or(0));
        }
        for (i, p) in c.column_props.iter().enumerate().filter(|(i, _)| *i < c.columns.len()) {
            let n = i + 1;
            if p.visible == Some(false) {
                out += &format!("#{id} th:nth-child({n}), #{id} td:nth-child({n}) {{ display: none; }}\n");
            }
            if let Some(a) = p.align.as_deref().filter(|a| matches!(*a, "center" | "right")) {
                out += &format!("#{id} th:nth-child({n}), #{id} td:nth-child({n}) {{ text-align: {a}; }}\n");
            }
        }
    }
    out
}

pub fn theme_css(theme: &str) -> String {
    let mut text = theme.to_string();
    for (key, sheet) in [("\"phai:light\"", phai_ui::theme::STANDARD_CSS), ("\"phai:dark\"", phai_ui::theme::DARK_CSS)] {
        let import = format!("@import {key};");
        if text.contains(&import) {
            text = text.replace(&import, sheet);
        }
    }
    // Comments out.
    let mut clean = String::new();
    let mut rest = text.as_str();
    while let Some(i) = rest.find("/*") {
        clean.push_str(&rest[..i]);
        rest = rest[i + 2..].find("*/").map(|j| &rest[i + 2 + j + 2..]).unwrap_or("");
    }
    clean.push_str(rest);
    let mut out = String::new();
    for rule in clean.split('}') {
        let Some((sel, body)) = rule.split_once('{') else { continue };
        let sel = sel.trim();
        if sel.is_empty() || sel.starts_with('@') {
            continue;
        }
        let first_kind = sel.split(',').next().unwrap_or("").trim();
        let kind = &first_kind[..first_kind.find(['.', '#', ':']).unwrap_or(first_kind.len())];
        let _ = writeln!(out, "{} {{ {} }}", selector(sel), declarations(kind, body));
    }
    out
}

/// How every control sits on the page, before the theme colours it.
pub const BASE_CSS: &str = r#"
* { box-sizing: border-box; }
html, body { margin: 0; height: 100%; overflow: hidden; }
body { font: 12px/1.3 "Segoe UI", "Leelawadee UI", "Noto Sans Thai", system-ui, sans-serif; -webkit-text-size-adjust: 100%; }
.phai-window { display: flex; flex-direction: column; background: var(--surface, #fff); color: var(--text, #1b1b1b); }
.phai-main { position: fixed; inset: 0; }
.phai-titlebar { flex: none; display: flex; align-items: center; gap: 8px; height: 40px; padding: 0 12px; padding-top: env(safe-area-inset-top); font-size: 14px; font-weight: 600; border-bottom: 1px solid var(--rule, #e5e5e5); }
.phai-titlebar span { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.phai-close { border: 0; background: transparent; font-size: 16px; width: 32px; height: 32px; border-radius: 4px; color: inherit; cursor: pointer; }
.phai-close:hover { background: var(--accent-tint, #e7f3ef); }
.phai-body { flex: 1; position: relative; overflow: auto; }
.phai-area { position: relative; }
.phai-shade { position: fixed; inset: 0; display: flex; align-items: center; justify-content: center; background: rgba(0, 0, 0, 0.3); }
.phai-dialog { position: relative; border-radius: 8px; overflow: hidden; box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3); }
.phai { position: absolute; z-index: 1; margin: 0; font: inherit; color: inherit; }
.phai-group, .phai-panel, .phai-k-tabs, .phai-k-flow, .phai-k-table { z-index: 0; }
.phai-label { display: flex; align-items: center; white-space: nowrap; overflow: hidden; }
.phai-button { border: 1px solid; cursor: pointer; padding: 0 8px; }
.phai-edit, .phai-combo, .phai-date, .phai-calendar, .phai-list { border: 1px solid; padding: 0 6px; outline: none; }
textarea.phai-edit { padding: 4px 6px; resize: none; }
.phai-checkbox, .phai-radio { display: flex; align-items: center; gap: 6px; background: transparent !important; border: 0 !important; cursor: pointer; }
.phai-checkbox input, .phai-radio input { margin: 0; width: 15px; height: 15px; accent-color: var(--phai-check, var(--accent, #12855f)); }
.phai-group { border: 1px solid; padding: 0 8px; pointer-events: none; }
.phai-group legend { padding: 0 4px; }
.phai-panel { border: 0; }
.phai-slider { accent-color: var(--accent, #12855f); }
.phai-progress { appearance: none; -webkit-appearance: none; border: 0; overflow: hidden; }
.phai-progress::-webkit-progress-bar { background: transparent; }
.phai-progress::-webkit-progress-value { background: var(--accent, #12855f); }
.phai-progress::-moz-progress-bar { background: var(--accent, #12855f); }
.phai-statusbar { display: flex; align-items: center; gap: 16px; padding: 0 12px; padding-bottom: env(safe-area-inset-bottom); border-top: 1px solid; }
.phai-link { display: flex; align-items: center; color: var(--accent, #12855f); }
.phai-picture { object-fit: contain; }
/* PhoneLayout = stack: on a screen narrower than the form, one control under another, as wide as
   the screen, in the order they read; big enough for a finger. */
.phai-area.phai-stacked { display: flex; flex-direction: column; gap: 10px; padding: 12px 12px calc(12px + env(safe-area-inset-bottom)); min-height: 100%; }
.phai-stacked > .phai { position: relative !important; inset: auto !important; width: auto !important; flex: none; }
.phai-stacked > .phai-button, .phai-stacked > .phai-edit, .phai-stacked > .phai-combo, .phai-stacked > .phai-date, .phai-stacked > .phai-checkbox, .phai-stacked > .phai-radio { min-height: 44px; font-size: 16px; }
.phai-stacked > textarea.phai-edit, .phai-stacked > .phai-k-list, .phai-stacked > .phai-k-grid, .phai-stacked > .phai-k-listbox { min-height: 180px; }
.phai-stacked > .phai-label { height: auto !important; white-space: normal; margin-bottom: -6px; }
.phai-stacked > .phai-group { height: auto !important; border-width: 0 0 1px; padding: 0; margin-top: 6px; }
.phai-stacked > .phai-panel:empty, .phai-stacked > .phai-k-flow:empty { display: none; }
.phai-stacked > .phai-statusbar { margin: auto -12px calc(-12px - env(safe-area-inset-bottom)); }
.phai-stacked > .phai-toolbar { position: relative !important; inset: auto !important; order: -2000000; flex: none; margin: -12px -12px 0; }
.phai-stacked .phai-checkbox input, .phai-stacked .phai-radio input { width: 22px; height: 22px; }
.phai-k-list, .phai-k-grid { overflow: auto; padding: 0; }
.phai-k-list table, .phai-k-grid table { border-collapse: collapse; width: 100%; }
.phai-k-list th, .phai-k-grid th { position: sticky; top: 0; text-align: left; font-weight: 400; padding: 3px 6px; background: var(--surface, #fff); border-bottom: 1px solid var(--rule, #e5e5e5); }
.phai-k-list td, .phai-k-grid td { padding: 3px 6px; white-space: nowrap; }
.phai-k-grid td { border: 1px solid var(--rule, #e5e5e5); }
.phai-k-list tr.selected, .phai-k-grid tr.selected { background: var(--accent-tint, #e7f3ef); }
.phai-k-tabs { display: flex; align-items: flex-end; gap: 2px; border-bottom: 1px solid var(--rule, #e5e5e5); }
.phai-tab { border: 0; background: transparent; padding: 6px 12px; font: inherit; color: var(--muted, #5f5f5f); border-bottom: 2px solid transparent; cursor: pointer; }
.phai-tab.selected { color: var(--text, #1b1b1b); border-bottom-color: var(--accent, #12855f); }
.phai-k-listbox, .phai-k-checkedlist { padding: 0; }
.phai-menubar { flex: none; display: flex; align-items: stretch; gap: 2px; padding: 0 4px; height: 28px; overflow-x: auto; scrollbar-width: none; border-bottom: 1px solid var(--rule, #e5e5e5); }
.phai-menu-top { border: 0; background: transparent; color: inherit; font: inherit; padding: 0 10px; border-radius: 4px; cursor: pointer; white-space: nowrap; }
.phai-menu-top:hover, .phai-menu-top.open { background: var(--accent-tint, #e7f3ef); }
.phai-menu-state { display: none; }
.phai-dropdown { position: fixed; z-index: 100; min-width: 180px; max-width: calc(100vw - 8px); max-height: calc(100dvh - 8px); overflow-y: auto; padding: 4px; background: var(--surface, #fff); color: var(--text, #1b1b1b); border: 1px solid var(--border, #d4d4d4); border-radius: 6px; box-shadow: 0 6px 20px rgba(0, 0, 0, 0.18); font-size: 12px; }
.phai-menu-item { display: flex; align-items: center; gap: 8px; min-height: 28px; padding: 0 10px 0 4px; border-radius: 4px; cursor: pointer; white-space: nowrap; }
.phai-menu-item:hover { background: var(--accent-tint, #e7f3ef); }
.phai-menu-item.disabled { color: var(--disabled-text, #a0a0a0); pointer-events: none; }
.phai-menu-mark { width: 18px; text-align: center; flex: none; }
.phai-menu-item.checked .phai-menu-mark::before { content: "\2713"; }
.phai-menu-item.checked .phai-menu-mark.phai-glyph { font-size: 0; }
.phai-menu-item.checked .phai-menu-mark.phai-glyph::before { font-size: 12px; font-family: system-ui, sans-serif; }
.phai-menu-label { flex: 1; }
.phai-menu-key { color: var(--muted, #5f5f5f); padding-left: 16px; }
.phai-menu-sep { height: 1px; margin: 4px 6px; background: var(--rule, #e5e5e5); }
.phai-glyph { font-family: "Segoe Fluent Icons", "Segoe MDL2 Assets"; font-size: 15px; line-height: 1; }
.phai-toolbar { position: absolute; z-index: 2; display: flex; align-items: center; gap: 2px; padding: 0 4px; overflow-x: auto; scrollbar-width: none; border-bottom: 1px solid var(--rule, #e5e5e5); }
.phai-tool { display: flex; align-items: center; gap: 6px; height: 28px; padding: 0 8px; border: 1px solid transparent; border-radius: 4px; background: transparent; color: inherit; font: inherit; cursor: pointer; white-space: nowrap; }
.phai-tool:hover { background: var(--accent-tint, #e7f3ef); }
.phai-tool .phai-glyph { color: var(--icon, #3c3c3c); }
.phai-tool.checked { background: var(--accent-tint-pressed, #d0e7df); border-color: var(--accent, #12855f); }
.phai-tool.disabled { color: var(--disabled-text, #a0a0a0); pointer-events: none; }
.phai-k-tree { overflow: auto; border: 1px solid; padding: 2px 0; outline: none; }
.phai-k-tree ul { list-style: none; margin: 0; padding: 0; }
.phai-k-tree ul ul { padding-left: 16px; }
.phai-node { display: flex; align-items: center; gap: 2px; min-height: 22px; padding: 0 6px 0 2px; white-space: nowrap; cursor: default; }
.phai-twist { width: 16px; flex: none; text-align: center; font-size: 10px; color: var(--muted, #5f5f5f); cursor: pointer; }
.phai-k-tree li.open > .phai-node .phai-twist::before { content: "BE"; }
.phai-k-tree li.closed > .phai-node .phai-twist::before { content: "B8"; }
.phai-k-tree li.closed > ul { display: none; }
.phai-k-tree li.selected > .phai-node { background: var(--accent-tint, #e7f3ef); }
.phai-k-tree:focus li.selected > .phai-node { background: var(--accent, #12855f); color: #fff; }
.phai-stacked > .phai-k-tree { min-height: 180px; }
@media (pointer: coarse) { .phai-node { min-height: 36px; font-size: 15px; } }
@media (pointer: coarse) { .phai-edit, .phai-combo, .phai-date { font-size: 16px; } .phai-menu-item { min-height: 40px; font-size: 15px; } .phai-menubar { height: 36px; } }
"#;

/// The page: the window's title, the theme, the program started.
/// `version` names this build on the program's files, so a browser never pairs a new page with
/// an old program it kept.
pub fn index_html(title: &str, package: &str, theme: &str, color: &str, version: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="th">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<meta name="theme-color" content="{color}">
<meta name="apple-mobile-web-app-capable" content="yes">
<meta name="mobile-web-app-capable" content="yes">
<title>{title}</title>
<link rel="manifest" href="manifest.webmanifest">
<link rel="icon" href="icon-192.png">
<link rel="apple-touch-icon" href="icon-192.png">
<style>{BASE_CSS}
{theme}</style>
</head>
<body>
<script type="module">
import start from "./pkg/{package}.js?v={version}";
start({{ module_or_path: "./pkg/{package}_bg.wasm?v={version}" }});
if ("serviceWorker" in navigator) {{
  navigator.serviceWorker.register("sw.js");
}}
</script>
</body>
</html>
"#,
        title = html_escape(title),
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// `s` as a JSON string.
fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// What a phone reads to install the page as an app. `short` is the name under its icon (the
/// first 12 letters of `name` when empty); `orientation` any, portrait or landscape.
pub fn manifest(name: &str, short: &str, color: &str, background: &str, orientation: &str) -> String {
    let short = json_str(&if short.trim().is_empty() { name.chars().take(12).collect::<String>() } else { short.trim().to_string() });
    let orientation = json_str(match orientation.trim() {
        "portrait" | "landscape" => orientation.trim(),
        _ => "any",
    });
    let (name, color, background) = (json_str(name), json_str(color), json_str(background));
    format!(
        r#"{{
  "name": {name},
  "short_name": {short},
  "start_url": ".",
  "scope": ".",
  "display": "standalone",
  "orientation": {orientation},
  "background_color": {background},
  "theme_color": {color},
  "icons": [
    {{ "src": "icon-192.png", "sizes": "192x192", "type": "image/png" }},
    {{ "src": "icon-512.png", "sizes": "512x512", "type": "image/png" }},
    {{ "src": "icon-512.png", "sizes": "512x512", "type": "image/png", "purpose": "maskable" }}
  ]
}}
"#
    )
}

/// Keeps the app's files so it opens without the network; `version` changes with each build, so
/// a phone takes the new one.
pub fn service_worker(package: &str, version: &str) -> String {
    format!(
        r#"const CACHE = "phai-{package}-{version}";
const FILES = [".", "index.html", "manifest.webmanifest", "icon-192.png", "icon-512.png", "pkg/{package}.js?v={version}", "pkg/{package}_bg.wasm?v={version}"];
self.addEventListener("install", (e) => {{
  e.waitUntil(caches.open(CACHE).then((c) => c.addAll(FILES)).then(() => self.skipWaiting()));
}});
self.addEventListener("activate", (e) => {{
  e.waitUntil(caches.keys().then((keys) => Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k)))).then(() => self.clients.claim()));
}});
self.addEventListener("fetch", (e) => {{
  if (e.request.method !== "GET") return;
  // The network first, so a new version shows at once; what was kept when there is none.
  e.respondWith(fetch(e.request, {{ cache: "no-cache" }}).then((r) => {{
    const copy = r.clone();
    caches.open(CACHE).then((c) => c.put(e.request, copy));
    return r;
  }}).catch(() => caches.match(e.request)));
}});
"#
    )
}

/// A variable's colour in the theme (`--accent`), as `#rrggbb`.
pub fn theme_var(theme: &str, name: &str) -> Option<String> {
    let full = theme.replace("@import \"phai:light\";", phai_ui::theme::STANDARD_CSS).replace("@import \"phai:dark\";", phai_ui::theme::DARK_CSS);
    // The last definition wins, as in CSS.
    let mut found = None;
    for line in full.split([';', '{', '}']) {
        if let Some((k, v)) = line.split_once(':') {
            if k.trim() == name {
                found = Some(v.trim().split("/*").next().unwrap_or("").trim().to_string());
            }
        }
    }
    found.filter(|v| v.starts_with('#'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_theme_becomes_the_browsers_css() {
        let css = theme_css(":root { --accent: #2563eb; }\nbutton.primary:hover { background: var(--accent); border-width: 1; }\n/* a note */\nedit, combo { border-color: red; }\ncheckbox:checked { background: var(--accent); accent-color: #fff; }\n#save { border-radius: 8; }");
        assert!(css.lines().any(|l| l == ":root { --accent: #2563eb }"), "{css}");
        assert!(css.contains(".phai-button.primary:hover { background: var(--accent); border-style: solid; border-width: 1px }"), "{css}");
        assert!(css.contains(".phai-edit, .phai-combo { border-color: red }"), "{css}");
        assert!(css.contains(".phai-checkbox:has(input:checked) { --phai-check: var(--accent) }"), "{css}");
        assert!(css.contains(".phai#save { border-radius: 8px }"), "{css}");
        assert!(!css.contains("a note"));
        let std = theme_css("@import \"phai:light\";");
        assert!(std.contains(".phai-statusbar"), "{std}");
        assert_eq!(theme_var("@import \"phai:light\";\n:root { --accent: #2563eb; }", "--accent").as_deref(), Some("#2563eb"));
        assert_eq!(theme_var("@import \"phai:light\";", "--accent").as_deref(), Some("#12855f"));
    }

    #[test]
    fn a_form_for_the_web_keeps_its_trait_and_leaves_out_what_a_browser_lacks() {
        let mut f = FormFile::new("MainWindow");
        f.form.title = "My program".into();
        f.form.size = [480, 320];
        let mut ok = Control::new(Kind::Button, "ok", [394, 260, 75, 23]);
        ok.text = Some("OK".into());
        ok.anchor = Some("right bottom".into());
        ok.on_click = Some("ok_clicked".into());
        let mut name = Control::new(Kind::Edit, "name", [11, 11, 200, 23]);
        name.events.insert("key_down".into(), "name_key".into());
        let tray = Control::new(Kind::NotifyIcon, "tray", [0, 0, 0, 0]);
        f.control = vec![ok, name, tray];
        f.form.events.insert("shown".into(), "begin".into());
        let code = generate(&f);
        assert!(code.contains("fn ok_clicked(&mut self, ui: &MainWindowUi) {}"), "{code}");
        assert!(code.contains("fn name_key(&mut self, ui: &MainWindowUi, e: &phai_ui::events::KeyEventArgs) {}"), "{code}");
        assert!(code.contains("form::control(area, \"button\", \"ok\", \"\", t(\"OK\"), Place { x: 394, y: 260, w: 75, h: 23, anchor: \"right bottom\", dock: \"none\", within: (480, 320) });"), "{code}");
        assert!(code.contains("form::on(ui.ok.hwnd(), \"click\", call(&me, |e: &mut E, ui: &MainWindowUi| e.ok_clicked(ui)));"), "{code}");
        assert!(code.contains("form::on_key(ui.name.hwnd(), \"keydown\""), "{code}");
        assert!(code.contains("form::after_shown"), "{code}");
        assert!(!code.contains("pub tray"), "{code}");
        assert_eq!(left_out(&f), vec!["tray (notifyicon)".to_string()]);
        // A menu within a menu, a tick a click turns over, a toolbar button that stays down.
        let mut g = f.clone();
        g.menu = crate::bars::menus_from_text("&View\n  word_wrap | Word wrap | checked | check_on_click\n  Zoom\n    zoom_in | Zoom in | Ctrl+= | SEARCH\n").unwrap();
        g.toolbar = vec![crate::model::Tool { id: "word_wrap".into(), label: "Wrap".into(), glyph: "SEARCH".into(), toggle: true, ..Default::default() }];
        let code = generate(&g);
        assert!(code.contains("phai_ui::menu::Item { id: \"word_wrap\", label: t(\"Word wrap\"), key: \"\", glyph: 0, checked: true, check_on_click: true"), "{code}");
        assert!(code.contains("label: t(\"Zoom\")") && code.contains("key: \"Ctrl+=\", glyph: 59169"), "{code}");
        assert!(code.contains("phai_ui::menu::toolbar(hwnd, area, vec![phai_ui::menu::Tool { id: \"word_wrap\", label: t(\"Wrap\"), glyph: 59169, toggle: true }], 34, chosen(&me));"), "{code}");
        assert!(code.contains("pub fn set_item_checked(&self, item: &str, on: bool)") && code.contains("pub toolbar: HWND,"), "{code}");
    }

    #[test]
    fn a_form_stacks_on_a_phone_in_reading_order() {
        let mut f = FormFile::parse(
            r#"format = 1

[form]
name = "MainWindow"
title = "Shop"
size = [600, 400]

[[control]]
kind = "button"
id = "ok"
text = "OK"
bounds = [500, 360, 75, 23]

[[control]]
kind = "label"
id = "name_label"
text = "Name"
bounds = [11, 20, 80, 23]

[[control]]
kind = "edit"
id = "name"
bounds = [100, 20, 300, 23]

[[control]]
kind = "statusbar"
id = "status"
dock = "bottom"
"#,
        )
        .unwrap();
        assert!(stacks(&f, 390) && !stacks(&f, 800));
        let r = stacked_rects(&f, 390, 844, 0);
        let (ok, label, name) = (r[0].unwrap(), r[1].unwrap(), r[2].unwrap());
        assert!(label[1] < name[1] && name[1] < ok[1], "read top to bottom: {r:?}");
        assert_eq!((name[0], name[2], name[3]), (12, 390 - 24, 44), "the width less the margins, a finger's height");
        assert_eq!(r[3].unwrap()[1] + r[3].unwrap()[3], 844, "the status bar along the bottom");
        f.form.set_prop("phone", Some(toml::Value::String("zoom".into())));
        assert!(!stacks(&f, 390));
        let code = generate(&f);
        assert!(!code.contains("stack_when_narrow"), "zoom: shrunk as before");
    }

    #[test]
    fn the_pwa_files_name_the_program() {
        let page = index_html("Shop <1>", "shop", ".phai-button { }", "#12855f", "7");
        assert!(page.contains("./pkg/shop.js?v=7") && page.contains("module_or_path: \"./pkg/shop_bg.wasm?v=7\""), "{page}");
        assert!(page.contains("./pkg/shop.js") && page.contains("Shop &lt;1&gt;") && page.contains("manifest.webmanifest"));
        let m = manifest("ร้านค้าของฉันทั้งหมดเลย", "", "#12855f", "#ffffff", "");
        assert!(m.contains("\"short_name\": \"ร้านค้าของฉั\"") && m.contains("standalone") && m.contains("\"orientation\": \"any\"") && !m.contains("\\u{"), "{m}");
        let m = manifest("Shop", "ร้าน", "#12855f", "#ffffff", "portrait");
        assert!(m.contains("\"short_name\": \"ร้าน\"") && m.contains("\"orientation\": \"portrait\""), "{m}");
        assert_eq!(json_str("a\"b\\c\n"), r#""a\"b\\c\u000a""#);
        assert!(service_worker("shop", "7").contains("pkg/shop_bg.wasm") && service_worker("shop", "7").contains("phai-shop-7"));
    }

    /// A grid looks in the browser as on Windows: every other row, the headings, each column's
    /// alignment, hidden columns.
    #[test]
    fn a_grid_s_look_reaches_the_page() {
        let mut f = FormFile::new("MainWindow");
        let mut g = Control::new(Kind::Grid, "orders", [0, 0, 300, 200]);
        g.columns = vec![("Name".into(), 100), ("Amount".into(), 80), ("Key".into(), 40)];
        g.column_props = vec![Default::default(), crate::ColumnProps { align: Some("right".into()), ..Default::default() }, crate::ColumnProps { visible: Some(false), ..Default::default() }];
        g.set_prop("alternate_row_color", Some(toml::Value::String("#f4f8f6".into())));
        g.set_prop("header_back_color", Some(toml::Value::String("#12855f".into())));
        f.control.push(g);
        let css = grids_css(&[&f]);
        assert!(css.contains("#orders tbody tr:nth-child(even) td { background: #f4f8f6; }"), "{css}");
        assert!(css.contains("#orders th { background: #12855f; }"), "{css}");
        assert!(css.contains("#orders th:nth-child(2), #orders td:nth-child(2) { text-align: right; }"), "{css}");
        assert!(css.contains("#orders th:nth-child(3), #orders td:nth-child(3) { display: none; }"), "{css}");
    }

    #[test]
    fn a_tree_and_a_context_menu_reach_the_page() {
        let mut f = FormFile::new("MainWindow");
        let mut tree = Control::new(Kind::Tree, "folders", [0, 0, 200, 300]);
        tree.items = vec!["Root".into(), "  Child".into()];
        tree.on_select = Some("folders_selected".into());
        tree.context_menu = Some("menu".into());
        let mut menu = Control::new(Kind::ContextMenu, "menu", [0, 0, 0, 0]);
        menu.items = vec!["rename | Rename".into(), "-".into(), "remove | Remove".into()];
        menu.events.insert("opening".into(), "menu_opening".into());
        f.control.push(tree);
        f.control.push(menu);
        assert!(left_out(&f).is_empty(), "{:?}", left_out(&f));
        let code = generate(&f);
        assert!(code.contains("phai_ui::tree::set_lines(folders, &[\"Root\", \"  Child\"]);"), "{code}");
        assert!(code.contains("form::on(ui.folders.hwnd(), \"change\""), "{code}");
        assert!(code.contains("let menu = HWND::default();"), "{code}");
        assert!(code.contains("phai_ui::menu::context_menu(hwnd, ui.folders.hwnd(), vec![phai_ui::menu::Item { id: \"rename\", label: t(\"Rename\"), ..Default::default() }, phai_ui::menu::Item { separator: true, ..Default::default() }"), "{code}");
        assert!(code.contains("e.menu_opening(ui)"), "{code}");
        assert!(code.contains("\"remove\" => e.remove(ui),"), "{code}");
    }

    #[test]
    fn a_bound_grid_asks_the_data_server_on_the_web() {
        let mut f = FormFile::new("MainWindow");
        let mut g = Control::new(Kind::Grid, "customers", [0, 0, 300, 200]);
        g.columns = vec![("Id".into(), 60), ("Name".into(), 120)];
        g.bind = Some(crate::model::Bind { database: "shop".into(), table: "customers".into(), columns: vec!["id".into(), "name".into()], keys: vec!["id".into()] });
        let mut save = Control::new(Kind::Button, "save", [0, 210, 75, 23]);
        save.bind_action = Some("customers:save".into());
        f.control.push(g);
        f.control.push(save);
        assert!(left_out(&f).is_empty(), "{:?}", left_out(&f));
        let code = generate(&f);
        assert!(code.contains("pub customers_data: phai_ui::data::Binding,"), "{code}");
        assert!(code.contains("customers_data: phai_ui::data::Binding::new(\"main_window/customers\"),"), "{code}");
        assert!(code.contains("pub fn save_customers(&self) -> Result<usize, String> {"), "{code}");
        assert!(code.contains("let _ = ui.reload_customers();"), "{code}");
        assert!(code.contains("form::on(ui.save.hwnd(), \"click\", call(&me, |e: &mut E, ui: &MainWindowUi| { if let Err(m) = ui.save_customers().map(|_| ())"), "{code}");
        let main = data_server_main(&[&f]).unwrap();
        assert!(main.contains("(\"main_window/customers\", \"shop\", \"customers\", &[\"id\", \"name\"], &[\"id\"]),"), "{main}");
        assert!(data_server_main(&[&FormFile::new("Empty")]).is_none());
    }
}
