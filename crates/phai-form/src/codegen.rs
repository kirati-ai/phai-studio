//! Rust from a form: `<name>_ui.rs`, which Phai Studio writes whole on every save, and the first
//! `<name>.rs`, which is the programmer's from then on (see `docs/phaiform.md`).
//!
//! The generated file builds the window with phai-ui - the same calls the designer's canvas makes
//! (`live.rs`) - so what is designed is what runs. Handlers are methods of a trait whose default
//! bodies do nothing: a handler added in the form never breaks the programmer's code, and the
//! designer never has to read that code back.

use std::fmt::Write;

use crate::model::{parse_anchor, snake, Control, FormFile, HasProps, Kind};
use crate::events::{self as ev, Args, Source};
use crate::props;

/// Where the theme is, if the form has one.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// For `include_str!`, relative to the generated file: the theme built into the program.
    pub theme_include: Option<String>,
    /// Relative to the crate's `Cargo.toml`: the file a debug build watches, so a saved change to
    /// the theme shows in the running program.
    pub theme_watch: Option<String>,
}

/// The id a control's window gets: 2000 and up, in form order.
fn control_id(i: usize) -> i32 {
    2000 + i as i32
}

fn const_name(s: &str) -> String {
    format!("ID_{}", s.to_ascii_uppercase())
}

fn lit(s: &str) -> String {
    format!("{s:?}")
}

fn anchor_code(c: &Control) -> String {
    let a = parse_anchor(c.anchor.as_deref()).unwrap_or_default();
    format!("Anchor {{ left: {}, top: {}, right: {}, bottom: {} }}", a.left, a.top, a.right, a.bottom)
}

fn dock_code(c: &Control) -> &'static str {
    match c.dock.as_deref() {
        Some("top") => "Dock::Top",
        Some("bottom") => "Dock::Bottom",
        Some("left") => "Dock::Left",
        Some("right") => "Dock::Right",
        Some("fill") => "Dock::Fill",
        _ => "Dock::None",
    }
}

fn text_of(c: &Control) -> String {
    c.text.clone().unwrap_or_default()
}

/// The calls that give control `c` its table properties (`props::CONTROL`) - `ui.` before its
/// name when `on_ui` - only those of words when `words_only` (the Thai or English again).
fn prop_calls(c: &Control, on_ui: bool, words_only: bool) -> Vec<String> {
    let name = if on_ui { format!("self.{}.hwnd()", c.id) } else { c.id.clone() };
    props::of_kind(c.kind)
        .filter(|s| s.how == props::How::Call && (!words_only || s.ty == props::Ty::Words))
        .filter_map(|s| c.prop(s.name).map(|v| format!("phai_ui::props::{}({name}, {});", s.name, props::rust_literal(s, v))))
        .collect()
}

/// A combo's items in the order shown: sorted when it is `sorted`.
pub fn combo_items(c: &Control) -> Vec<String> {
    let mut items = c.items.clone();
    if c.flag("sorted", false) {
        items.sort_by_key(|s| s.to_lowercase());
    }
    items
}

/// The page of the tabs shown first: the first tabs control's SelectedIndex.
pub fn first_page(f: &FormFile) -> usize {
    f.control.iter().find(|c| c.kind == Kind::Tabs).and_then(|c| c.int("selected_index")).unwrap_or(0).max(0) as usize
}

/// The controls Tab goes through, in its order, when any has a TabIndex; empty otherwise.
pub fn tab_order(f: &FormFile) -> Vec<usize> {
    let focusable = |c: &Control| props::find(c.kind, "tab_index").is_some();
    if !f.control.iter().any(|c| focusable(c) && c.int("tab_index").is_some()) {
        return vec![];
    }
    let mut v: Vec<usize> = (0..f.control.len()).filter(|i| focusable(&f.control[*i])).collect();
    v.sort_by_key(|i| (f.control[*i].int("tab_index").unwrap_or(i32::MAX), *i));
    v
}

/// The `phai_ui::ctl` type of a control of `kind` - its field's type in the generated struct.
pub fn ctl_type(kind: Kind) -> &'static str {
    match kind {
        Kind::Label => "Label",
        Kind::Edit | Kind::Textarea => "TextBox",
        Kind::Rich => "RichTextBox",
        Kind::Button => "Button",
        Kind::Checkbox => "CheckBox",
        Kind::Radio => "RadioButton",
        Kind::Combo => "ComboBox",
        Kind::List => "ListView",
        Kind::Grid => "Grid",
        Kind::Tree => "TreeView",
        Kind::Picture => "PictureBox",
        Kind::Date => "DateTimePicker",
        Kind::Number => "NumericUpDown",
        Kind::Slider => "TrackBar",
        Kind::Progress => "ProgressBar",
        Kind::Group => "GroupBox",
        Kind::Tabs => "TabControl",
        Kind::Flow => "FlowPanel",
        Kind::Table => "TablePanel",
        Kind::Statusbar => "StatusBar",
        Kind::Timer => "Timer",
        Kind::ContextMenu => "ContextMenu",
    }
}

/// `phai_ui::events`' name for an event code.
fn event_const(code: u16) -> &'static str {
    use phai_ui::events as e;
    match code {
        e::ENTER => "ENTER",
        e::LEAVE => "LEAVE",
        e::KEY_DOWN => "KEY_DOWN",
        e::KEY_UP => "KEY_UP",
        e::KEY_PRESS => "KEY_PRESS",
        e::MOUSE_ENTER => "MOUSE_ENTER",
        e::MOUSE_LEAVE => "MOUSE_LEAVE",
        e::MOUSE_DOWN => "MOUSE_DOWN",
        e::MOUSE_UP => "MOUSE_UP",
        e::MOUSE_MOVE => "MOUSE_MOVE",
        e::CLICK => "CLICK",
        e::DOUBLE_CLICK => "DOUBLE_CLICK",
        e::DRAG_DROP => "DRAG_DROP",
        _ => "RESIZE",
    }
}

/// The call of a method given the event's arguments from `l` (a `WM_EVENT`'s) - or none.
fn event_call(h: &str, a: Args) -> String {
    match a.param() {
        Some(p) => format!("events.{h}(ui, unsafe {{ phai_ui::events::args::<{p}>(l) }})"),
        None => format!("events.{h}(ui)"),
    }
}

/// `<name>_ui.rs`.
pub fn generate(f: &FormFile, opts: &Options) -> String {
    let name = &f.form.name;
    let file = snake(name);
    let ui = format!("{name}Ui");
    let events = format!("{name}Events");
    let handlers = f.handlers();
    let mut o = String::new();
    let w = &mut o;

    let _ = writeln!(w, "// Made by Phai Studio from {file}.phaiform - do not change this file; change the form.");
    let _ = writeln!(w, "// Everything here is written again on every save.");
    let _ = writeln!(w);
    let _ = writeln!(w, "#![allow(dead_code, unused_imports, unused_variables, clippy::all)]");
    let _ = writeln!(w);
    for u in [
        "phai_ui::controls",
        "phai_ui::layout::{scale, unscale, Anchor, Dock, Item, Layout, Rect}",
        "phai_ui::text::t",
        "phai_ui::theme",
        "phai_ui::win::{self, wide}",
        "windows::core::PCWSTR",
        "windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM}",
        "windows::Win32::Graphics::Gdi::{self, HFONT}",
        "windows::Win32::UI::WindowsAndMessaging as wm",
    ] {
        let _ = writeln!(w, "use {u};");
    }
    let _ = writeln!(w);
    let _ = writeln!(w, "/// Every English sentence this window shows - for the project's Thai table and its test.");
    let _ = writeln!(w, "pub const TEXTS: &[&str] = &[");
    for t in f.texts() {
        let _ = writeln!(w, "    {},", lit(&t));
    }
    let _ = writeln!(w, "];");
    let _ = writeln!(w);

    // Ids.
    for (i, h) in handlers.iter().enumerate() {
        let _ = writeln!(w, "const {}: i32 = {};", const_name(h), 1000 + i);
    }
    let _ = writeln!(w, "const ID_TOOLBAR: i32 = 1999;");
    for (i, c) in f.control.iter().enumerate() {
        let _ = writeln!(w, "const CONTROL_{}: i32 = {};", c.id.to_ascii_uppercase(), control_id(i));
    }
    let _ = writeln!(w);

    // The struct.
    let _ = writeln!(w, "/// The window and every control in it, by the ids the form gives them.");
    let _ = writeln!(w, "pub struct {ui} {{");
    let _ = writeln!(w, "    pub hwnd: HWND,");
    let _ = writeln!(w, "    pub font: HFONT,");
    if !f.toolbar.is_empty() {
        let _ = writeln!(w, "    pub toolbar: HWND,");
    }
    for c in &f.control {
        let _ = writeln!(w, "    pub {}: phai_ui::ctl::{},", c.id, ctl_type(c.kind));
    }
    for c in f.control.iter().filter(|c| c.bind.is_some()) {
        let _ = writeln!(w, "    /// The table {} shows, and what it held when last read.", c.id);
        let _ = writeln!(w, "    pub {}_data: phai_data::Binding,", c.id);
    }
    let _ = writeln!(w, "    layout: Layout,");
    let _ = writeln!(w, "}}");
    let _ = writeln!(w);

    // The trait.
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
    let _ = writeln!(w, "    /// The program's main window: the program ends with it.");
    let _ = writeln!(w, "    main: bool,");
    let _ = writeln!(w, "}}");
    let _ = writeln!(w);

    // impl
    let _ = writeln!(w, "impl {ui} {{");
    let _ = writeln!(w, "    /// Makes the window and runs it until it is closed.");
    let _ = writeln!(w, "    pub fn run<E: {events} + 'static>(events: E) {{");
    let _ = writeln!(w, "        let hwnd = Self::create(events);");
    let _ = writeln!(w, "        win::run_loop(Some(hwnd));");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    let _ = writeln!(w, "    /// Makes the window and shows it; the caller runs the message loop.");
    let _ = writeln!(w, "    pub fn create<E: {events} + 'static>(events: E) -> HWND {{");
    let _ = writeln!(w, "        Self::make(None, events)");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    let _ = writeln!(w, "    /// Shows this window beside `owner` (another window of the program) and returns at once,");
    let _ = writeln!(w, "    /// as `Show()` in WinForms. Closing it leaves the program running.");
    let _ = writeln!(w, "    pub fn show<E: {events} + 'static>(owner: HWND, events: E) -> HWND {{");
    let _ = writeln!(w, "        Self::make(Some(owner), events)");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    let _ = writeln!(w, "    /// Shows this window over `owner` and waits until it is closed, `owner` taking no input");
    let _ = writeln!(w, "    /// meanwhile - as `ShowDialog()` in WinForms. What it answers: the DialogResult of the");
    let _ = writeln!(w, "    /// button that closed it, or `Cancel`.");
    let _ = writeln!(w, "    pub fn show_dialog<E: {events} + 'static>(owner: HWND, events: E) -> phai_ui::form::DialogResult {{");
    let _ = writeln!(w, "        let hwnd = Self::make(Some(owner), events);");
    let _ = writeln!(w, "        win::modal_loop(hwnd, owner);");
    let _ = writeln!(w, "        phai_ui::form::take_result(hwnd)");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    let _ = writeln!(w, "    fn make<E: {events} + 'static>(owner: Option<HWND>, events: E) -> HWND {{");
    let _ = writeln!(w, "        win::init();");
    if let Some(inc) = &opts.theme_include {
        let _ = writeln!(w, "        if let Ok(sheet) = theme::Theme::parse(include_str!({})) {{", lit(inc));
        let _ = writeln!(w, "            theme::set(sheet);");
        let _ = writeln!(w, "        }}");
    }
    if f.form.events.contains_key("double_click") {
        let _ = writeln!(w, "        let class = win::register_with(win::ClassStyle::DoubleClicks, {}, Some(proc_::<E>), None);", lit(&format!("Phai.{name}")));
    } else {
        let _ = writeln!(w, "        let class = win::register({}, Some(proc_::<E>), None);", lit(&format!("Phai.{name}")));
    }
    let _ = writeln!(w, "        let title = wide(t({}));", lit(&f.form.title));
    let menu = if f.menu.is_empty() { "None" } else { "Some(Self::menu())" };
    let fm = &f.form;
    let _ = writeln!(w, "        let (style, ex) = phai_ui::form::frame({}, {}, {}, {}, {});", lit(fm.word("border").unwrap_or("sizable")), fm.flag("maximize_box", true), fm.flag("minimize_box", true), fm.flag("control_box", true), fm.flag("top_most", false));
    let _ = writeln!(w, "        let main = owner.is_none();");
    let _ = writeln!(w, "        let over = owner;");
    if !fm.flag("show_in_taskbar", true) {
        let _ = writeln!(w, "        // ShowInTaskbar = false: owned by a hidden window, which Windows gives no button.");
        let _ = writeln!(w, "        let owner = owner.or_else(|| Some(phai_ui::form::hidden_owner()));");
    }
    let _ = writeln!(w, "        unsafe {{");
    let _ = writeln!(w, "            let hwnd = wm::CreateWindowExW(ex, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), style, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, 100, 100, owner, {menu}, Some(win::instance()), None).unwrap_or_default();");
    let (back, fore) = (fm.word("back_color"), fm.word("fore_color"));
    if back.is_some() || fore.is_some() {
        let opt = |v: Option<&str>| v.map(|s| format!("Some({})", lit(s))).unwrap_or_else(|| "None".into());
        let _ = writeln!(w, "            phai_ui::form::set_colors(hwnd, {}, {});", opt(back), opt(fore));
    }
    if let Some(img) = fm.word("background_image") {
        let _ = writeln!(w, "            phai_ui::form::set_background_image(hwnd, {}, {});", lit(img), lit(fm.word("background_image_layout").unwrap_or("tile")));
    }
    if fm.flag("auto_scroll", false) {
        let _ = writeln!(w, "            phai_ui::form::set_auto_scroll(hwnd, true);");
    }
    if fm.flag("allow_drop", false) || fm.events.contains_key("drag_drop") {
        let _ = writeln!(w, "            phai_ui::props::allow_drop(hwnd, true);");
    }
    let _ = writeln!(w, "            let ui = Self::build(hwnd);");
    for c in f.control.iter().filter(|c| c.bind.is_some()) {
        let _ = writeln!(w, "            if let Err(e) = ui.reload_{}() {{", c.id);
        let _ = writeln!(w, "                win::message(Some(hwnd), &e, t({}), wm::MB_ICONWARNING);", lit(&f.form.title));
        let _ = writeln!(w, "            }}");
    }
    let _ = writeln!(w, "            let mut host = Box::new(Host {{ ui, events, main }});");
    let _ = writeln!(w, "            host.events.loaded(&host.ui);");
    let _ = writeln!(w, "            win::set_state(hwnd, host);");
    let _ = writeln!(w, "            win::theme_frame(hwnd);");
    if let Some(icon) = fm.word("icon") {
        let _ = writeln!(w, "            phai_ui::form::set_icon(hwnd, include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), {})));", lit(&format!("/{}", icon.trim_start_matches(['/', '\\']).replace('\\', "/"))));
    }
    if let Some(o) = fm.int("opacity") {
        let _ = writeln!(w, "            phai_ui::form::set_opacity(hwnd, {o});");
    }
    let _ = writeln!(w, "            let style = wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32;");
    let _ = writeln!(w, "            let ex = wm::GetWindowLongW(hwnd, wm::GWL_EXSTYLE) as u32;");
    let _ = writeln!(w, "            let mut r = RECT {{ left: 0, top: 0, right: win::px(hwnd, {}), bottom: win::px(hwnd, {}) }};", f.form.size[0], f.form.size[1]);
    let _ = writeln!(w, "            let _ = windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(style), {}, wm::WINDOW_EX_STYLE(ex), win::dpi(hwnd) as u32);", !f.menu.is_empty());
    let _ = writeln!(w, "            let _ = wm::SetWindowPos(hwnd, None, 0, 0, r.right - r.left, r.bottom - r.top, wm::SWP_NOMOVE | wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);");
    let _ = writeln!(w, "            if let Some(o) = over {{");
    let _ = writeln!(w, "                win::center_on(hwnd, o);");
    if f.form.start.as_deref() != Some("windows") {
        let _ = writeln!(w, "            }} else {{");
        let _ = writeln!(w, "                win::center_on_screen(hwnd);");
    }
    let _ = writeln!(w, "            }}");
    if f.form.maximized == Some(true) {
        let _ = writeln!(w, "            win::show_maximized(hwnd);");
    } else {
        let _ = writeln!(w, "            win::show_top(hwnd);");
    }
    if f.form.events.contains_key("shown") {
        let _ = writeln!(w, "            let _ = wm::PostMessageW(Some(hwnd), phai_ui::events::WM_SHOWN, WPARAM(0), LPARAM(0));");
    }
    if let Some(watch) = &opts.theme_watch {
        let _ = writeln!(w, "            // A debug build follows the theme file as it is saved.");
        let _ = writeln!(w, "            #[cfg(debug_assertions)]");
        let _ = writeln!(w, "            win::watch_theme(hwnd, std::path::Path::new(concat!(env!(\"CARGO_MANIFEST_DIR\"), {})));", lit(&format!("/{watch}")));
    }
    let _ = writeln!(w, "            hwnd");
    let _ = writeln!(w, "        }}");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);

    if !f.menu.is_empty() {
        let _ = writeln!(w, "    fn menu() -> wm::HMENU {{");
        let _ = writeln!(w, "        unsafe {{");
        let _ = writeln!(w, "            let bar = wm::CreateMenu().unwrap_or_default();");
        for m in &f.menu {
            let _ = writeln!(w, "            let m = wm::CreatePopupMenu().unwrap_or_default();");
            for it in &m.items {
                if it.separator {
                    let _ = writeln!(w, "            let _ = wm::AppendMenuW(m, wm::MF_SEPARATOR, 0, None);");
                } else if let (Some(id), Some(label)) = (&it.id, &it.label) {
                    let key = it.key.as_deref().map(|k| format!(", {}", lit(&format!("\t{k}")))).unwrap_or_default();
                    let text = if key.is_empty() { format!("t({})", lit(label)) } else { format!("&format!(\"{{}}{{}}\", t({}){key})", lit(label)) };
                    let _ = writeln!(w, "            let s = wide({text});");
                    let _ = writeln!(w, "            let _ = wm::AppendMenuW(m, wm::MF_STRING, {} as usize, PCWSTR(s.as_ptr()));", const_name(id));
                }
            }
            let _ = writeln!(w, "            let s = wide(t({}));", lit(&m.label));
            let _ = writeln!(w, "            let _ = wm::AppendMenuW(bar, wm::MF_POPUP, m.0 as usize, PCWSTR(s.as_ptr()));");
        }
        let _ = writeln!(w, "            bar");
        let _ = writeln!(w, "        }}");
        let _ = writeln!(w, "    }}");
        let _ = writeln!(w);
    }
    if !f.toolbar.is_empty() {
        let _ = writeln!(w, "    fn toolbar_items() -> Vec<(i32, &'static str, u16)> {{");
        let _ = writeln!(w, "        vec![");
        for tool in &f.toolbar {
            let _ = writeln!(w, "            ({}, t({}), phai_ui::icon::glyph::{}),", const_name(&tool.id), lit(&tool.label), tool.glyph);
        }
        let _ = writeln!(w, "        ]");
        let _ = writeln!(w, "    }}");
        let _ = writeln!(w);
    }

    // build
    let _ = writeln!(w, "    fn build(hwnd: HWND) -> {ui} {{");
    let _ = writeln!(w, "        let font = win::font(hwnd, {}, false);", f.form.font_size.unwrap_or(0));
    let _ = writeln!(w, "        let dpi = win::dpi(hwnd);");
    // A form with nothing laid out yet (a new one, or only timers) changes nothing in it.
    let _ = writeln!(w, "        #[allow(unused_mut)]");
    let _ = writeln!(w, "        let mut layout = Layout::new(({}, {}));", f.form.size[0], f.form.size[1]);
    if let Some(p) = f.form.int("padding") {
        let _ = writeln!(w, "        layout.padding = {p};");
    }
    if !f.toolbar.is_empty() {
        let _ = writeln!(w, "        let toolbar = win::toolbar(hwnd, ID_TOOLBAR, &Self::toolbar_items());");
        let _ = writeln!(w, "        layout.add(toolbar, Item {{ bounds: Rect::new(0, 0, 0, win::TOOLBAR_HEIGHT), dock: Dock::Top, anchor: Anchor::default() }});");
    }
    for c in &f.control {
        let id = format!("CONTROL_{}", c.id.to_ascii_uppercase());
        let text = format!("t({})", lit(&text_of(c)));
        // A control with its own size gets a font of its own; the form's otherwise.
        let font = match c.font_size {
            Some(pt) => {
                let _ = writeln!(w, "        let font_{} = win::font(hwnd, {pt}, false);", c.id);
                format!("font_{}", c.id)
            }
            None => "font".to_string(),
        };
        let make = match c.kind {
            Kind::Label => format!("win::label(hwnd, {text}, {id}, {font})"),
            Kind::Edit => format!("win::edit(hwnd, {}, {id}, {font}, {})", lit(&text_of(c)), c.password.unwrap_or(false)),
            Kind::Button => format!("win::button(hwnd, {text}, {id}, {font}, false)"),
            Kind::Checkbox => format!("win::checkbox(hwnd, {text}, {id}, {font}, {})", c.checked.unwrap_or(false)),
            Kind::Radio => format!("win::radio(hwnd, {text}, {id}, {font}, {}, {})", c.group.unwrap_or(0), c.checked.unwrap_or(false)),
            Kind::Combo => format!("win::combo_with(hwnd, {id}, {font}, &[{}], {})", combo_items(c).iter().map(|s| format!("t({})", lit(s))).collect::<Vec<_>>().join(", "), c.flag("editable", false)),
            Kind::List => format!("win::list(hwnd, {id}, {font}, &[{}])", c.columns.iter().map(|(s, n)| format!("(t({}), {n})", lit(s))).collect::<Vec<_>>().join(", ")),
            Kind::Group => format!("controls::group(hwnd, {text}, {id}, {font})"),
            Kind::Tabs => format!("controls::tabs(hwnd, {id}, {font}, &[{}])", c.tabs.iter().map(|s| format!("t({})", lit(s))).collect::<Vec<_>>().join(", ")),
            Kind::Statusbar => format!("controls::status_bar(hwnd, {id}, {font})"),
            Kind::Textarea => format!("win::textarea_with(hwnd, {}, {id}, {font}, {}, {})", lit(&text_of(c)), c.flag("word_wrap", true), lit(c.word("scroll_bars").unwrap_or("vertical"))),
            Kind::Date => format!("win::date_with(hwnd, {id}, {font}, {}, {})", c.flag("show_check_box", false), c.flag("show_up_down", false)),
            Kind::Number => format!("win::number(hwnd, {id}, {font}, {}, {}, {})", c.min.unwrap_or(0), c.max.unwrap_or(100), c.value.unwrap_or(0)),
            Kind::Progress => format!("win::progress(hwnd, {id}, {}, {}, {})", c.min.unwrap_or(0), c.max.unwrap_or(100), c.value.unwrap_or(0)),
            Kind::Slider => format!("win::slider(hwnd, {id}, {}, {}, {})", c.min.unwrap_or(0), c.max.unwrap_or(100), c.value.unwrap_or(0)),
            Kind::Tree => format!("{{ let h = win::tree(hwnd, {id}, {font}); win::tree_items(h, &[{}]); h }}", c.items.iter().map(|s| lit(s)).collect::<Vec<_>>().join(", ")),
            Kind::Picture => format!("phai_ui::picture::picture(hwnd, {id}, {})", lit(c.image.as_deref().unwrap_or(""))),
            Kind::Flow | Kind::Table => format!("win::panel(hwnd, {id}, false)"),
            Kind::Rich => format!("phai_ui::grid::richtext(hwnd, {id}, {font}, {})", lit(&text_of(c))),
            Kind::Grid => {
                // A bound grid's headings that are its columns' own names: shown as they are.
                let raw = |i: usize, s: &String| c.bind.as_ref().is_some_and(|b| b.columns.get(i) == Some(s));
                format!("phai_ui::grid::grid(hwnd, {id}, {font}, &[{}])", c.columns.iter().enumerate().map(|(i, (s, n))| if raw(i, s) { format!("({}, {n})", lit(s)) } else { format!("(t({}), {n})", lit(s)) }).collect::<Vec<_>>().join(", "))
            }
            // A context menu is the window's own too: made when a right-click asks for it.
            Kind::ContextMenu => "hwnd".to_string(),
            // A timer is the window's own: its field is the window, its ticks come as WM_TIMER.
            Kind::Timer => format!("phai_ui::ctl::Timer::new(hwnd, {id}, {}, {})", c.value.unwrap_or(1000).max(1), c.enabled != Some(false)),
        };
        if c.kind.is_component() {
            let _ = writeln!(w, "        let {} = {make};", c.id);
            continue;
        }
        let _ = writeln!(w, "        let {} = {make};", c.id);
        let _ = writeln!(w, "        win::set_css({}, {}, {});", c.id, lit(c.class.as_deref().unwrap_or("")), lit(&c.id));
        if let Some(own) = c.own_css() {
            let _ = writeln!(w, "        let _ = win::set_own_style({}, {});", c.id, lit(&own));
        }
        if c.kind == Kind::Statusbar {
            if !c.parts.is_empty() {
                let _ = writeln!(w, "        controls::status_parts({}, &[{}]);", c.id, c.parts.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "));
            }
            if !text_of(c).is_empty() {
                let _ = writeln!(w, "        controls::status_text({}, 0, {text});", c.id);
            }
        }
        for call in prop_calls(c, false, false) {
            let _ = writeln!(w, "        {call}");
        }
        if c.kind == Kind::Tabs && first_page(f) > 0 {
            let _ = writeln!(w, "        controls::tabs_select({}, {});", c.id, first_page(f));
        }
        if c.enabled == Some(false) {
            let _ = writeln!(w, "        win::enable({}, false);", c.id);
        }
        if c.visible == Some(false) || c.page.is_some_and(|p| p != first_page(f)) {
            let _ = writeln!(w, "        win::show({}, false);", c.id);
        }
        let [x, y, cw, ch] = c.rect();
        let ch = if c.kind == Kind::Statusbar { format!("unscale(controls::status_height({}), dpi)", c.id) } else { ch.to_string() };
        let _ = writeln!(w, "        layout.add({}, Item {{ bounds: Rect::new({x}, {y}, {cw}, {ch}), dock: {}, anchor: {} }});", c.id, dock_code(c), anchor_code(c));
    }
    for c in &f.control {
        if let Some(b) = &c.bind {
            let list = |v: &[String]| v.iter().map(|x| lit(x)).collect::<Vec<_>>().join(", ");
            let _ = writeln!(w, "        let {}_data = phai_data::Binding::new(crate::data_sources::connection(), {}, {}, &[{}], &[{}]);", c.id, lit(&b.database), lit(&b.table), list(&b.columns), list(&b.keys));
        }
    }
    let preview = f.form.flag("key_preview", false) && ["key_down", "key_up", "key_press"].iter().any(|e| f.form.events.contains_key(*e));
    for (i, c) in f.control.iter().enumerate() {
        let hooked: Vec<&str> = c.events.keys().filter_map(|e| ev::find(c.kind, e)).filter_map(|s| if let Source::Hook(code) = s.source { Some(event_const(code)) } else { None }).collect();
        let keys = preview && props::find(c.kind, "tab_index").is_some();
        if hooked.is_empty() && !keys {
            continue;
        }
        let mask = if hooked.is_empty() { "0".to_string() } else { hooked.iter().map(|k| format!("phai_ui::events::bit(phai_ui::events::{k})")).collect::<Vec<_>>().join(" | ") };
        let _ = writeln!(w, "        phai_ui::events::hook({}, hwnd, {}, {mask}, {keys});", c.id, control_id(i));
    }
    let order = tab_order(f);
    if !order.is_empty() {
        let _ = writeln!(w, "        phai_ui::form::tab_order(&[{}]);", order.iter().map(|i| f.control[*i].id.clone()).collect::<Vec<_>>().join(", "));
    }
    let _ = writeln!(w, "        {ui} {{");
    let _ = writeln!(w, "            hwnd,");
    let _ = writeln!(w, "            font,");
    if !f.toolbar.is_empty() {
        let _ = writeln!(w, "            toolbar,");
    }
    for c in &f.control {
        if c.kind == Kind::Timer {
            let _ = writeln!(w, "            {},", c.id);
        } else {
            let _ = writeln!(w, "            {}: phai_ui::ctl::{}({}),", c.id, ctl_type(c.kind), c.id);
        }
    }
    for c in f.control.iter().filter(|c| c.bind.is_some()) {
        let _ = writeln!(w, "            {}_data,", c.id);
    }
    let _ = writeln!(w, "            layout,");
    let _ = writeln!(w, "        }}");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);

    // bound grids
    for c in f.control.iter().filter(|c| c.bind.is_some()) {
        let id = &c.id;
        let table = &c.bind.as_ref().map(|b| b.table.clone()).unwrap_or_default();
        let _ = writeln!(w, "    /// {id}'s rows read again from {table}.");
        let _ = writeln!(w, "    pub fn reload_{id}(&self) -> Result<(), String> {{");
        let _ = writeln!(w, "        let rows = self.{id}_data.load()?;");
        let _ = writeln!(w, "        phai_ui::grid::grid_fill(self.{id}, &rows);");
        let _ = writeln!(w, "        Ok(())");
        let _ = writeln!(w, "    }}");
        let _ = writeln!(w);
        let _ = writeln!(w, "    /// What {id} holds written back to {table} - changed cells, new rows, rows taken out -");
        let _ = writeln!(w, "    /// then read again. How many changes were made.");
        let _ = writeln!(w, "    pub fn save_{id}(&self) -> Result<usize, String> {{");
        let _ = writeln!(w, "        let n = self.{id}_data.save(&phai_ui::grid::grid_tagged_rows(self.{id}))?;");
        let _ = writeln!(w, "        self.reload_{id}()?;");
        let _ = writeln!(w, "        Ok(n)");
        let _ = writeln!(w, "    }}");
        let _ = writeln!(w);
    }

    // the form's methods, as a WinForms Form's
    let _ = writeln!(w, "    /// Closes the window, as the close box would - its FormClosing first.");
    let _ = writeln!(w, "    pub fn close(&self) {{");
    let _ = writeln!(w, "        unsafe {{");
    let _ = writeln!(w, "            let _ = wm::PostMessageW(Some(self.hwnd), wm::WM_CLOSE, WPARAM(0), LPARAM(0));");
    let _ = writeln!(w, "        }}");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    let _ = writeln!(w, "    /// Out of sight, still running; [`Self::show_window`] brings it back.");
    let _ = writeln!(w, "    pub fn hide(&self) {{");
    let _ = writeln!(w, "        win::show(self.hwnd, false);");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    let _ = writeln!(w, "    pub fn show_window(&self) {{");
    let _ = writeln!(w, "        win::show(self.hwnd, true);");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    let _ = writeln!(w, "    /// In front, taking the keys.");
    let _ = writeln!(w, "    pub fn activate(&self) {{");
    let _ = writeln!(w, "        unsafe {{");
    let _ = writeln!(w, "            let _ = wm::SetForegroundWindow(self.hwnd);");
    let _ = writeln!(w, "        }}");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    let _ = writeln!(w, "    pub fn center_to_screen(&self) {{");
    let _ = writeln!(w, "        win::center_on_screen(self.hwnd);");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    let _ = writeln!(w, "    /// The words in its title bar.");
    let _ = writeln!(w, "    pub fn title(&self) -> String {{");
    let _ = writeln!(w, "        win::text_of(self.hwnd)");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    let _ = writeln!(w, "    pub fn set_title(&self, s: &str) {{");
    let _ = writeln!(w, "        win::set_text(self.hwnd, s);");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    let _ = writeln!(w, "    /// A handle other threads use to run work on this window's thread - [`Self::begin_invoke`].");
    let _ = writeln!(w, "    pub fn invoker(&self) -> phai_ui::invoke::Invoker {{");
    let _ = writeln!(w, "        phai_ui::invoke::Invoker::new(self.hwnd)");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    let _ = writeln!(w, "    /// From any thread: `f` runs on the window's thread with the program's {name} and the");
    let _ = writeln!(w, "    /// controls, soon; returns at once - WinForms' BeginInvoke. `false` when the window is gone.");
    let _ = writeln!(w, "    pub fn begin_invoke<E: {events} + 'static>(inv: phai_ui::invoke::Invoker, f: impl FnOnce(&mut E, &{ui}) + Send + 'static) -> bool {{");
    let _ = writeln!(w, "        inv.post(Box::new(move |e, ui| {{");
    let _ = writeln!(w, "            if let (Some(e), Some(ui)) = (e.downcast_mut::<E>(), ui.downcast_ref::<{ui}>()) {{");
    let _ = writeln!(w, "                f(e, ui);");
    let _ = writeln!(w, "            }}");
    let _ = writeln!(w, "        }}))");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    let _ = writeln!(w, "    /// As [`Self::begin_invoke`], waiting until `f` has run - WinForms' Invoke.");
    let _ = writeln!(w, "    pub fn invoke<E: {events} + 'static>(inv: phai_ui::invoke::Invoker, f: impl FnOnce(&mut E, &{ui}) + Send + 'static) {{");
    let _ = writeln!(w, "        inv.send(Box::new(move |e, ui| {{");
    let _ = writeln!(w, "            if let (Some(e), Some(ui)) = (e.downcast_mut::<E>(), ui.downcast_ref::<{ui}>()) {{");
    let _ = writeln!(w, "                f(e, ui);");
    let _ = writeln!(w, "            }}");
    let _ = writeln!(w, "        }}))");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w);
    // relabel
    let _ = writeln!(w, "    /// Every word again in the language now chosen - after `phai_ui::text::set_thai`.");
    let _ = writeln!(w, "    pub fn relabel(&self) {{");
    let _ = writeln!(w, "        win::set_text(self.hwnd, t({}));", lit(&f.form.title));
    if !f.menu.is_empty() {
        let _ = writeln!(w, "        unsafe {{");
        let _ = writeln!(w, "            let old = wm::GetMenu(self.hwnd);");
        let _ = writeln!(w, "            let _ = wm::SetMenu(self.hwnd, Some(Self::menu()));");
        let _ = writeln!(w, "            let _ = wm::DestroyMenu(old);");
        let _ = writeln!(w, "        }}");
    }
    if !f.toolbar.is_empty() {
        let _ = writeln!(w, "        win::set_toolbar(self.toolbar, &Self::toolbar_items());");
    }
    for c in &f.control {
        match c.kind {
            Kind::Label | Kind::Button | Kind::Checkbox | Kind::Radio | Kind::Group => {
                let _ = writeln!(w, "        win::set_text(self.{}, t({}));", c.id, lit(&text_of(c)));
            }
            Kind::Tabs => {
                let _ = writeln!(w, "        controls::tabs_relabel(self.{}.hwnd(), &[{}]);", c.id, c.tabs.iter().map(|s| format!("t({})", lit(s))).collect::<Vec<_>>().join(", "));
            }
            Kind::Statusbar if !text_of(c).is_empty() => {
                let _ = writeln!(w, "        controls::status_text(self.{}, 0, t({}));", c.id, lit(&text_of(c)));
            }
            _ => {}
        }
        for call in prop_calls(c, true, true) {
            let _ = writeln!(w, "        {call}");
        }
    }
    let _ = writeln!(w, "    }}");

    // flow and table panels
    let panels: Vec<usize> = f.control.iter().enumerate().filter(|(_, c)| c.kind.arranges()).map(|(i, _)| i).collect();
    if !panels.is_empty() {
        let _ = writeln!(w);
        let _ = writeln!(w, "    /// Flow and table panels place the controls in them - after the layout placed the panels.");
        let _ = writeln!(w, "    pub fn arrange(&self) {{");
        let _ = writeln!(w, "        let dpi = win::dpi(self.hwnd);");
        let _ = writeln!(w, "        let px = |v: i32| phai_ui::layout::scale(v, dpi);");
        for p in panels {
            let c = &f.control[p];
            let kids = f.arranged_in(p);
            if kids.is_empty() {
                continue;
            }
            let _ = writeln!(w, "        {{");
            let _ = writeln!(w, "            let r = win::rect_in_parent(self.{});", c.id);
            let _ = writeln!(w, "            let area = Rect::new(r.left, r.top, r.right - r.left, r.bottom - r.top);");
            let sizes: Vec<String> = kids.iter().map(|k| {
                let [_, _, cw, ch] = f.control[*k].rect();
                format!("(px({cw}), px({ch}))")
            }).collect();
            let _ = writeln!(w, "            let sizes = [{}];", sizes.join(", "));
            let names: Vec<String> = kids.iter().map(|k| format!("self.{}.hwnd()", f.control[*k].id)).collect();
            let gap = c.int("gap").unwrap_or(6);
            if c.kind == Kind::Flow {
                let _ = writeln!(w, "            let places = phai_ui::layout::flow_with(area, &sizes, px({gap}), {}, {});", c.word("direction") == Some("top_down"), c.flag("wrap", true));
                let _ = writeln!(w, "            for (k, h) in [{}].into_iter().enumerate() {{", names.join(", "));
                let _ = writeln!(w, "                win::move_to(h, places[k].0, places[k].1, sizes[k].0, sizes[k].1);");
            } else {
                let _ = writeln!(w, "            let cells = phai_ui::layout::table(area, &sizes, {}, px({gap}));", c.cols.unwrap_or(2).max(1));
                let _ = writeln!(w, "            for (k, h) in [{}].into_iter().enumerate() {{", names.join(", "));
                let _ = writeln!(w, "                win::move_to(h, cells[k].x, cells[k].y, cells[k].w, cells[k].h);");
            }
            let _ = writeln!(w, "            }}");
            let _ = writeln!(w, "        }}");
        }
        let _ = writeln!(w, "    }}");
    }

    // tabs pages
    let tabs: Vec<&Control> = f.control.iter().filter(|c| c.kind == Kind::Tabs).collect();
    if !tabs.is_empty() {
        let _ = writeln!(w);
        let _ = writeln!(w, "    /// Shows the controls of the chosen tab's page and hides the others.");
        let _ = writeln!(w, "    fn show_page(&self, page: usize) {{");
        for c in f.control.iter().filter(|c| c.page.is_some()) {
            let _ = writeln!(w, "        win::show(self.{}, page == {});", c.id, c.page.unwrap_or(0));
        }
        let _ = writeln!(w, "    }}");
    }
    let _ = writeln!(w, "}}");
    let _ = writeln!(w);

    // proc
    let _ = writeln!(w, "extern \"system\" fn proc_<E: {events} + 'static>(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {{");
    let (min, max) = (f.form.size_of("min_size"), f.form.size_of("max_size"));
    if min.is_some() || max.is_some() {
        let pair = |s: Option<[i32; 2]>| s.map(|[a, b]| format!("Some(({a}, {b}))")).unwrap_or_else(|| "None".into());
        let _ = writeln!(w, "    if msg == wm::WM_GETMINMAXINFO {{");
        let _ = writeln!(w, "        phai_ui::form::min_max(hwnd, l, {}, {}, {});", pair(min), pair(max), !f.menu.is_empty());
        let _ = writeln!(w, "        return LRESULT(0);");
        let _ = writeln!(w, "    }}");
    }
    let _ = writeln!(w, "    let Some(host) = (unsafe {{ win::state::<Host<E>>(hwnd) }}) else {{ return win::def(hwnd, msg, w, l) }};");
    let _ = writeln!(w, "    match msg {{");
    let form_ev = |e: &str| f.form.events.get(e).cloned();
    let _ = writeln!(w, "        wm::WM_SIZE => {{");
    let _ = writeln!(w, "            host.ui.layout.apply(hwnd);");
    if let Some(h) = form_ev("resize") {
        let _ = writeln!(w, "            host.events.{h}(&host.ui);");
    }
    if f.control.iter().any(|c| c.kind.arranges() && !f.arranged_in(f.control.iter().position(|x| std::ptr::eq(x, c)).unwrap_or(0)).is_empty()) {
        let _ = writeln!(w, "            host.ui.arrange();");
    }
    let _ = writeln!(w, "            LRESULT(0)");
    let _ = writeln!(w, "        }}");
    let _ = writeln!(w, "        wm::WM_COMMAND => {{");
    let _ = writeln!(w, "            let (id, code) = (win::loword(w.0) as i32, win::hiword(w.0));");
    let _ = writeln!(w, "            let Host {{ ui, events, .. }} = host;");
    let _ = writeln!(w, "            match id {{");
    for (which, idc) in [("accept_button", 1), ("cancel_button", 2)] {
        if let Some(b) = f.form.word(which) {
            let what = if idc == 1 { "Enter" } else { "Escape" };
            let _ = writeln!(w, "                {idc} => phai_ui::form::click(ui.{b}.hwnd()), // {what}: {}", props::find_form(which).map(|s| s.label).unwrap_or_default());
        }
    }
    let in_context = |h: &str| f.control.iter().filter(|c| c.kind == Kind::ContextMenu).any(|c| c.menu_items().iter().any(|(id, _)| id == h));
    for h in handlers.iter().filter(|h| f.menu.iter().flat_map(|m| &m.items).any(|it| it.id.as_deref() == Some(h.as_str())) || f.toolbar.iter().any(|t| &t.id == *h) || in_context(h)) {
        let _ = writeln!(w, "                {} => events.{h}(ui),", const_name(h));
    }
    for c in &f.control {
        let id = format!("CONTROL_{}", c.id.to_ascii_uppercase());
        let call = c.handler().map(|h| format!("events.{h}(ui)"));
        match c.kind {
            Kind::Button if c.bind_action.is_some() => {
                // Save or Reload for a bound grid, then the button's own handler if it has one.
                let (grid, what) = c.bind_action.as_deref().and_then(|a| a.split_once(':')).unwrap_or(("", "reload"));
                let run = if what == "save" { format!("ui.save_{grid}().map(|_| ())") } else { format!("ui.reload_{grid}()") };
                let after = call.map(|c| format!(" {c};")).unwrap_or_default();
                let _ = writeln!(w, "                {id} if code == 0 => {{");
                let _ = writeln!(w, "                    if let Err(e) = {run} {{");
                let _ = writeln!(w, "                        win::message(Some(hwnd), &e, t({}), wm::MB_ICONWARNING);", lit(&f.form.title));
                let _ = writeln!(w, "                    }}{after}");
                let _ = writeln!(w, "                }}");
            }
            Kind::Button if c.word("dialog_result").is_some() => {
                // Clicked: its handler, then the window closes with its answer.
                let r = c.word("dialog_result").unwrap_or("none");
                let answer = format!("phai_ui::form::DialogResult::{}", r[..1].to_uppercase() + &r[1..]);
                let before = call.map(|c| format!("{c}; ")).unwrap_or_default();
                let _ = writeln!(w, "                {id} if code == 0 => {{ {before}phai_ui::form::close_with(hwnd, {answer}); }}");
            }
            Kind::Checkbox | Kind::Radio if c.events.contains_key("checked_changed") => {
                let changed = format!("events.{}(ui);", c.events["checked_changed"]);
                let first = call.map(|c| format!("{c}; ")).unwrap_or_default();
                let _ = writeln!(w, "                {id} if code == 0 => {{ {first}{changed} }}");
            }
            Kind::Button | Kind::Checkbox | Kind::Radio => {
                if let Some(call) = call {
                    let _ = writeln!(w, "                {id} if code == 0 => {call},");
                }
            }
            Kind::Edit | Kind::Textarea | Kind::Number | Kind::Date | Kind::Slider | Kind::Rich | Kind::Grid => {
                if let Some(call) = call {
                    let _ = writeln!(w, "                {id} if code == 0x0300 => {call}, // EN_CHANGE, win::VALUE_CHANGED");
                }
            }
            Kind::Combo => {
                if let Some(call) = call {
                    let _ = writeln!(w, "                {id} if code == 1 => {call}, // CBN_SELCHANGE");
                }
                if let Some(h) = c.events.get("text_changed") {
                    let _ = writeln!(w, "                {id} if code == 5 => events.{h}(ui), // CBN_EDITCHANGE");
                }
            }
            Kind::Tabs => {
                let after = call.map(|c| format!(" {c};")).unwrap_or_default();
                let _ = writeln!(w, "                {id} => {{ ui.show_page(code as usize);{after} }}");
            }
            _ => {}
        }
    }
    let _ = writeln!(w, "                _ => {{}}");
    let _ = writeln!(w, "            }}");
    let _ = writeln!(w, "            LRESULT(0)");
    let _ = writeln!(w, "        }}");
    // A right-click on a control with a context menu: the menu, its choice coming as WM_COMMAND.
    let with_menu: Vec<&Control> = f.control.iter().filter(|c| c.context_menu.is_some()).collect();
    if !with_menu.is_empty() {
        let _ = writeln!(w, "        wm::WM_CONTEXTMENU => {{");
        let _ = writeln!(w, "            let from = w.0 as isize;");
        let _ = writeln!(w, "            let (x, y) = (win::loword(l.0 as usize) as i16 as i32, win::hiword(l.0 as usize) as i16 as i32);");
        for c in with_menu {
            let Some(menu) = f.control.iter().find(|m| m.kind == Kind::ContextMenu && Some(&m.id) == c.context_menu.as_ref()) else { continue };
            let _ = writeln!(w, "            if from == host.ui.{}.hwnd().0 as isize {{", c.id);
            let _ = writeln!(w, "                unsafe {{");
            let _ = writeln!(w, "                    let m = wm::CreatePopupMenu().unwrap_or_default();");
            for (id, label) in menu.menu_items() {
                if id.is_empty() {
                    let _ = writeln!(w, "                    let _ = wm::AppendMenuW(m, wm::MF_SEPARATOR, 0, None);");
                } else {
                    let _ = writeln!(w, "                    let s = wide(t({}));", lit(&label));
                    let _ = writeln!(w, "                    let _ = wm::AppendMenuW(m, wm::MF_STRING, {} as usize, PCWSTR(s.as_ptr()));", const_name(&id));
                }
            }
            let _ = writeln!(w, "                    let _ = wm::TrackPopupMenu(m, wm::TPM_LEFTALIGN | wm::TPM_TOPALIGN, x, y, None, hwnd, None);");
            let _ = writeln!(w, "                    let _ = wm::DestroyMenu(m);");
            let _ = writeln!(w, "                }}");
            let _ = writeln!(w, "                return LRESULT(0);");
            let _ = writeln!(w, "            }}");
        }
        let _ = writeln!(w, "            win::def(hwnd, msg, w, l)");
        let _ = writeln!(w, "        }}");
    }
    let timers: Vec<&Control> = f.control.iter().filter(|c| c.kind == Kind::Timer && c.on_tick.is_some()).collect();
    if !timers.is_empty() {
        let _ = writeln!(w, "        wm::WM_TIMER => {{");
        let _ = writeln!(w, "            let Host {{ ui, events, .. }} = host;");
        let _ = writeln!(w, "            match w.0 as i32 {{");
        for c in timers {
            let _ = writeln!(w, "                CONTROL_{} => events.{}(ui),", c.id.to_ascii_uppercase(), c.on_tick.as_deref().unwrap_or_default());
        }
        let _ = writeln!(w, "                _ => {{}}");
        let _ = writeln!(w, "            }}");
        let _ = writeln!(w, "            LRESULT(0)");
        let _ = writeln!(w, "        }}");
    }
    // WM_NOTIFY: a tree's item chosen, a list's row chosen, a heading clicked, a grid's row changed.
    let notify: Vec<&Control> = f.control.iter().filter(|c| (matches!(c.kind, Kind::Tree | Kind::List) && c.on_select.is_some()) || c.events.contains_key("column_click") || c.events.contains_key("selection_changed")).collect();
    if !notify.is_empty() {
        let _ = writeln!(w, "        wm::WM_NOTIFY => {{");
        let _ = writeln!(w, "            use windows::Win32::UI::Controls as cc;");
        let _ = writeln!(w, "            let hdr = unsafe {{ &*(l.0 as *const cc::NMHDR) }};");
        let _ = writeln!(w, "            let Host {{ ui, events, .. }} = host;");
        for c in notify {
            let idc = format!("CONTROL_{}", c.id.to_ascii_uppercase());
            if c.kind == Kind::Tree {
                if let Some(h) = &c.on_select {
                    let _ = writeln!(w, "            if hdr.idFrom == {idc} as usize && hdr.code == (-451i32) as u32 {{ // TVN_SELCHANGEDW");
                    let _ = writeln!(w, "                events.{h}(ui);");
                    let _ = writeln!(w, "            }}");
                }
                continue;
            }
            let row_chosen = |h: &str| {
                format!("            if hdr.idFrom == {idc} as usize && hdr.code == cc::LVN_ITEMCHANGED {{\n                let it = unsafe {{ &*(l.0 as *const cc::NMLISTVIEW) }};\n                if it.uNewState & cc::LVIS_SELECTED.0 != 0 && it.uOldState & cc::LVIS_SELECTED.0 == 0 {{\n                    events.{h}(ui);\n                }}\n            }}")
            };
            if c.kind == Kind::List {
                if let Some(h) = &c.on_select {
                    let _ = writeln!(w, "{}", row_chosen(h));
                }
            }
            if let Some(h) = c.events.get("selection_changed") {
                let _ = writeln!(w, "{}", row_chosen(h));
            }
            if let Some(h) = c.events.get("column_click") {
                let _ = writeln!(w, "            if hdr.idFrom == {idc} as usize && hdr.code == cc::LVN_COLUMNCLICK {{");
                let _ = writeln!(w, "                let it = unsafe {{ &*(l.0 as *const cc::NMLISTVIEW) }};");
                let _ = writeln!(w, "                events.{h}(ui, &phai_ui::events::ItemEventArgs {{ index: it.iSubItem as usize }});");
                let _ = writeln!(w, "            }}");
            }
        }
        let _ = writeln!(w, "            win::def(hwnd, msg, w, l)");
        let _ = writeln!(w, "        }}");
    }
    // The controls' own events, and the form's keys first when it previews them.
    let mut arms: Vec<String> = Vec::new();
    for c in &f.control {
        for (e, h) in &c.events {
            if let Some(s) = ev::find(c.kind, e) {
                if let Source::Hook(code) = s.source {
                    arms.push(format!("                (CONTROL_{}, phai_ui::events::{}) => {},", c.id.to_ascii_uppercase(), event_const(code), event_call(h, s.args)));
                }
            }
        }
    }
    for k in ["key_down", "key_up", "key_press"] {
        if let (Some(h), Some(s)) = (form_ev(k), ev::find_form(k)) {
            arms.push(format!("                (0, phai_ui::events::{}) => {},", k.to_ascii_uppercase(), event_call(&h, s.args)));
        }
    }
    if !arms.is_empty() {
        let _ = writeln!(w, "        phai_ui::events::WM_EVENT => {{");
        let _ = writeln!(w, "            let Host {{ ui, events, .. }} = host;");
        let _ = writeln!(w, "            match phai_ui::events::decode(w) {{");
        for a in arms {
            let _ = writeln!(w, "{a}");
        }
        let _ = writeln!(w, "                _ => {{}}");
        let _ = writeln!(w, "            }}");
        let _ = writeln!(w, "            LRESULT(0)");
        let _ = writeln!(w, "        }}");
    }
    // The form's own.
    if let Some(h) = form_ev("shown") {
        let _ = writeln!(w, "        phai_ui::events::WM_SHOWN => {{");
        let _ = writeln!(w, "            host.events.{h}(&host.ui);");
        let _ = writeln!(w, "            LRESULT(0)");
        let _ = writeln!(w, "        }}");
    }
    if let Some(h) = form_ev("closing") {
        let _ = writeln!(w, "        wm::WM_CLOSE => {{");
        let _ = writeln!(w, "            let e = phai_ui::events::ClosingEventArgs::default();");
        let _ = writeln!(w, "            host.events.{h}(&host.ui, &e);");
        let _ = writeln!(w, "            if e.cancelled() {{");
        let _ = writeln!(w, "                return LRESULT(0);");
        let _ = writeln!(w, "            }}");
        let _ = writeln!(w, "            win::def(hwnd, msg, w, l)");
        let _ = writeln!(w, "        }}");
    }
    let (act, deact) = (form_ev("activated"), form_ev("deactivate"));
    if act.is_some() || deact.is_some() {
        let _ = writeln!(w, "        wm::WM_ACTIVATE => {{");
        if let Some(h) = act {
            let _ = writeln!(w, "            if win::loword(w.0) != 0 {{");
            let _ = writeln!(w, "                host.events.{h}(&host.ui);");
            let _ = writeln!(w, "            }}");
        }
        if let Some(h) = deact {
            let _ = writeln!(w, "            if win::loword(w.0) == 0 {{");
            let _ = writeln!(w, "                host.events.{h}(&host.ui);");
            let _ = writeln!(w, "            }}");
        }
        let _ = writeln!(w, "            win::def(hwnd, msg, w, l)");
        let _ = writeln!(w, "        }}");
    }
    // The form's own mouse: messages on the window itself, not on a control.
    let down = form_ev("mouse_down");
    let up = form_ev("mouse_up");
    let click = form_ev("click");
    if down.is_some() {
        let _ = writeln!(w, "        wm::WM_LBUTTONDOWN | wm::WM_RBUTTONDOWN | wm::WM_MBUTTONDOWN => {{");
        let _ = writeln!(w, "            host.events.{}(&host.ui, &phai_ui::events::mouse_of(msg, l));", down.as_deref().unwrap_or_default());
        let _ = writeln!(w, "            LRESULT(0)");
        let _ = writeln!(w, "        }}");
    }
    if up.is_some() || click.is_some() {
        let _ = writeln!(w, "        wm::WM_LBUTTONUP | wm::WM_RBUTTONUP | wm::WM_MBUTTONUP => {{");
        if let Some(h) = &up {
            let _ = writeln!(w, "            host.events.{h}(&host.ui, &phai_ui::events::mouse_of(msg, l));");
        }
        if let Some(h) = &click {
            let _ = writeln!(w, "            if msg == wm::WM_LBUTTONUP {{");
            let _ = writeln!(w, "                host.events.{h}(&host.ui);");
            let _ = writeln!(w, "            }}");
        }
        let _ = writeln!(w, "            LRESULT(0)");
        let _ = writeln!(w, "        }}");
    }
    if let Some(h) = form_ev("double_click") {
        let _ = writeln!(w, "        wm::WM_LBUTTONDBLCLK => {{");
        let _ = writeln!(w, "            host.events.{h}(&host.ui);");
        let _ = writeln!(w, "            LRESULT(0)");
        let _ = writeln!(w, "        }}");
    }
    if let Some(h) = form_ev("mouse_move") {
        let _ = writeln!(w, "        wm::WM_MOUSEMOVE => {{");
        let _ = writeln!(w, "            host.events.{h}(&host.ui, &phai_ui::events::mouse_of(msg, l));");
        let _ = writeln!(w, "            LRESULT(0)");
        let _ = writeln!(w, "        }}");
    }
    if f.form.flag("auto_scroll", false) {
        let _ = writeln!(w, "        wm::WM_VSCROLL | wm::WM_HSCROLL | wm::WM_MOUSEWHEEL if phai_ui::form::scroll(hwnd, msg, w) => {{");
        let _ = writeln!(w, "            // AutoScroll: the controls where the bars now say.");
        let _ = writeln!(w, "            host.ui.layout.apply(hwnd);");
        let _ = writeln!(w, "            LRESULT(0)");
        let _ = writeln!(w, "        }}");
    }
    if let Some(h) = form_ev("drag_drop") {
        let _ = writeln!(w, "        0x0233 => {{");
        let _ = writeln!(w, "            // WM_DROPFILES: files dropped on the window itself.");
        let _ = writeln!(w, "            host.events.{h}(&host.ui, &phai_ui::events::files_of(w));");
        let _ = writeln!(w, "            LRESULT(0)");
        let _ = writeln!(w, "        }}");
    }
    // The form's own keys, when it has the focus itself.
    for (k, m, maker) in [("key_down", "wm::WM_KEYDOWN", "key_of(w)"), ("key_up", "wm::WM_KEYUP", "key_of(w)"), ("key_press", "wm::WM_CHAR", "key_press_of(w)")] {
        if let Some(h) = form_ev(k) {
            let _ = writeln!(w, "        {m} => {{");
            let _ = writeln!(w, "            host.events.{h}(&host.ui, &phai_ui::events::{maker});");
            let _ = writeln!(w, "            LRESULT(0)");
            let _ = writeln!(w, "        }}");
        }
    }
    let _ = writeln!(w, "        phai_ui::invoke::WM_INVOKE => {{");
    let _ = writeln!(w, "            // Work handed over from another thread (begin_invoke, invoke).");
    let _ = writeln!(w, "            LRESULT(unsafe {{ phai_ui::invoke::run(l, &mut host.events, &host.ui) }})");
    let _ = writeln!(w, "        }}");
    let _ = writeln!(w, "        0x02E0 => {{");
    let _ = writeln!(w, "            // WM_DPICHANGED: the window, its font and its icons at the new scale.");
    let _ = writeln!(w, "            let r = unsafe {{ &*(l.0 as *const RECT) }};");
    let _ = writeln!(w, "            unsafe {{");
    let _ = writeln!(w, "                let _ = wm::SetWindowPos(hwnd, None, r.left, r.top, r.right - r.left, r.bottom - r.top, wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);");
    let _ = writeln!(w, "                let _ = Gdi::DeleteObject(host.ui.font.into());");
    let _ = writeln!(w, "            }}");
    let _ = writeln!(w, "            host.ui.font = win::font(hwnd, 0, false);");
    let _ = writeln!(w, "            win::set_font_all(hwnd, host.ui.font);");
    let _ = writeln!(w, "            win::retheme(hwnd);");
    let _ = writeln!(w, "            host.ui.layout.apply(hwnd);");
    let _ = writeln!(w, "            LRESULT(0)");
    let _ = writeln!(w, "        }}");
    let _ = writeln!(w, "        wm::WM_DESTROY => {{");
    if let Some(h) = form_ev("closed") {
        let _ = writeln!(w, "            host.events.{h}(&host.ui);");
    }
    let _ = writeln!(w, "            // The program ends with its main window; a window shown over another only closes.");
    let _ = writeln!(w, "            if host.main {{");
    let _ = writeln!(w, "                unsafe {{ wm::PostQuitMessage(0) }};");
    let _ = writeln!(w, "            }}");
    let _ = writeln!(w, "            LRESULT(0)");
    let _ = writeln!(w, "        }}");
    let _ = writeln!(w, "        wm::WM_NCDESTROY => {{");
    let _ = writeln!(w, "            if let Some(host) = unsafe {{ win::take_state::<Host<E>>(hwnd) }} {{");
    let _ = writeln!(w, "                unsafe {{");
    let _ = writeln!(w, "                    let _ = Gdi::DeleteObject(host.ui.font.into());");
    let _ = writeln!(w, "                }}");
    let _ = writeln!(w, "            }}");
    let _ = writeln!(w, "            win::def(hwnd, msg, w, l)");
    let _ = writeln!(w, "        }}");
    let _ = writeln!(w, "        _ => win::def(hwnd, msg, w, l),");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w, "}}");
    o
}

/// The first `<name>.rs`: the programmer's from then on. Phai Studio writes it once.
pub fn user_stub(f: &FormFile) -> String {
    let name = &f.form.name;
    let file = snake(name);
    let mut o = String::new();
    let w = &mut o;
    let _ = writeln!(w, "//! {name}: what its controls do. Phai Studio made this file once; it is yours.");
    let _ = writeln!(w, "//! The window itself is in {file}_ui.rs, which Phai Studio writes from {file}.phaiform.");
    let _ = writeln!(w);
    let _ = writeln!(w, "use crate::{file}_ui::{{{name}Events, {name}Ui}};");
    let _ = writeln!(w);
    let _ = writeln!(w, "#[derive(Default)]");
    let _ = writeln!(w, "pub struct {name} {{}}");
    let _ = writeln!(w);
    let _ = writeln!(w, "impl {name}Events for {name} {{");
    if f.handlers().is_empty() && f.event_methods().is_empty() {
        // A form with no events yet: the one every window has, so there is a place to start.
        let _ = writeln!(w, "    /// The window is made and about to show.");
        let _ = writeln!(w, "    fn loaded(&mut self, _ui: &{name}Ui) {{}}");
    }
    for (i, h) in f.handlers().iter().enumerate() {
        if i > 0 {
            let _ = writeln!(w);
        }
        let _ = writeln!(w, "    fn {h}(&mut self, _ui: &{name}Ui) {{}}");
    }
    let plain = f.handlers();
    for (h, a) in f.event_methods().into_iter().filter(|(h, _)| !plain.contains(h)) {
        let _ = writeln!(w);
        let _ = writeln!(w, "    fn {h}(&mut self, _ui: &{name}Ui{}) {{}}", a.param().map(|p| format!(", _e: &{p}")).unwrap_or_default());
    }
    let _ = writeln!(w, "}}");
    o
}

/// A handler's empty method to add to the programmer's file, if it has none by that name yet -
/// found by its text only; the designer never parses the programmer's code. `None` when it is
/// there, or when the file has no `impl ...Events for` to put it in.
pub fn add_handler(user_src: &str, form_name: &str, handler: &str) -> Option<String> {
    add_handler_with(user_src, form_name, handler, None)
}

/// [`add_handler`] for an event whose method is given `param` (`phai_ui::events::KeyEventArgs`).
pub fn add_handler_with(user_src: &str, form_name: &str, handler: &str, param: Option<&str>) -> Option<String> {
    let extra = param.map(|p| format!(", _e: &{p}")).unwrap_or_default();
    if user_src.contains(&format!("fn {handler}(")) {
        return None;
    }
    let marker = format!("impl {form_name}Events for ");
    let start = user_src.find(&marker)?;
    // The impl's closing brace: the first `}` at the start of a line after it.
    let at = start + user_src[start..].find("\n}")?;
    // An empty line inside, indented, for the caret to land on and the code to start from.
    Some(format!("{}\n\n    fn {handler}(&mut self, _ui: &{form_name}Ui{extra}) {{\n        \n    }}{}", &user_src[..at], &user_src[at..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FORM: &str = include_str!("../../../apps/sample/src/main_window.phaiform");
    const GENERATED: &str = include_str!("../../../apps/sample/src/main_window_ui.rs");

    pub fn sample_options() -> Options {
        Options { theme_include: Some("../theme.css".into()), theme_watch: Some("theme.css".into()) }
    }

    /// The sample program builds from `main_window_ui.rs`, so the generator's output compiles; and
    /// that file must be exactly what the generator writes now. When the generator changes on
    /// purpose, write the file again: `cargo test -p phai-form -- --ignored regenerate`.
    #[test]
    fn the_sample_programs_generated_file_is_what_the_generator_writes() {
        let f = FormFile::parse(FORM).unwrap();
        let fresh = generate(&f, &sample_options());
        assert!(GENERATED.replace("\r\n", "\n") == fresh, "apps/sample/src/main_window_ui.rs is stale - run the ignored `regenerate` test");
    }

    #[test]
    #[ignore]
    fn regenerate() {
        let f = FormFile::parse(FORM).unwrap();
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../apps/sample/src/main_window_ui.rs");
        std::fs::write(path, generate(&f, &sample_options())).unwrap();
    }

    #[test]
    fn every_handler_is_a_trait_method_and_every_control_a_field() {
        let f = FormFile::parse(FORM).unwrap();
        let code = generate(&f, &Options::default());
        for h in f.handlers() {
            assert!(code.contains(&format!("fn {h}(&mut self, ui: &MainWindowUi) {{}}")), "{h}");
        }
        for c in &f.control {
            assert!(code.contains(&format!("pub {}: phai_ui::ctl::{},", c.id, ctl_type(c.kind))), "{}", c.id);
        }
        assert!(!code.contains("include_str!"), "no theme, no include");
    }

    #[test]
    fn a_new_handler_goes_into_the_impl_once() {
        let f = FormFile::parse(FORM).unwrap();
        let stub = user_stub(&f);
        assert!(add_handler(&stub, "MainWindow", "save").is_none(), "already there");
        let more = add_handler(&stub, "MainWindow", "print").unwrap();
        assert!(more.contains("    fn print(&mut self, _ui: &MainWindowUi) {\n        \n    }\n}"), "{more}");
        assert!(add_handler(&more, "MainWindow", "print").is_none());
        assert!(add_handler("struct X;", "MainWindow", "print").is_none(), "nowhere to put it");
    }
}
