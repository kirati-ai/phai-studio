//! Rust from a form: `<name>_ui.rs`, which Phai Studio writes whole on every save, and the first
//! `<name>.rs`, which is the programmer's from then on (see `docs/phaiform.md`).
//!
//! The generated file builds the window with phai-ui - the same calls the designer's canvas makes
//! (`live.rs`) - so what is designed is what runs. Handlers are methods of a trait whose default
//! bodies do nothing: a handler added in the form never breaks the programmer's code, and the
//! designer never has to read that code back.

use std::fmt::Write;

use crate::model::{parse_anchor, snake, Control, FormFile, Kind};

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
        let _ = writeln!(w, "    pub {}: HWND,", c.id);
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
    let _ = writeln!(w, "}}");
    let _ = writeln!(w);
    let _ = writeln!(w, "struct Host<E> {{");
    let _ = writeln!(w, "    ui: {ui},");
    let _ = writeln!(w, "    events: E,");
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
    let _ = writeln!(w, "        win::init();");
    if let Some(inc) = &opts.theme_include {
        let _ = writeln!(w, "        if let Ok(sheet) = theme::Theme::parse(include_str!({})) {{", lit(inc));
        let _ = writeln!(w, "            theme::set(sheet);");
        let _ = writeln!(w, "        }}");
    }
    let _ = writeln!(w, "        let class = win::register({}, Some(proc_::<E>), None);", lit(&format!("Phai.{name}")));
    let _ = writeln!(w, "        let title = wide(t({}));", lit(&f.form.title));
    let menu = if f.menu.is_empty() { "None" } else { "Some(Self::menu())" };
    let _ = writeln!(w, "        unsafe {{");
    let _ = writeln!(w, "            let hwnd = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_OVERLAPPEDWINDOW | wm::WS_CLIPCHILDREN, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, 100, 100, None, {menu}, Some(win::instance()), None).unwrap_or_default();");
    let _ = writeln!(w, "            let ui = Self::build(hwnd);");
    let _ = writeln!(w, "            let mut host = Box::new(Host {{ ui, events }});");
    let _ = writeln!(w, "            host.events.loaded(&host.ui);");
    let _ = writeln!(w, "            win::set_state(hwnd, host);");
    let _ = writeln!(w, "            win::theme_frame(hwnd);");
    let _ = writeln!(w, "            let style = wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32;");
    let _ = writeln!(w, "            let mut r = RECT {{ left: 0, top: 0, right: win::px(hwnd, {}), bottom: win::px(hwnd, {}) }};", f.form.size[0], f.form.size[1]);
    let _ = writeln!(w, "            let _ = windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(style), {}, Default::default(), win::dpi(hwnd) as u32);", !f.menu.is_empty());
    let _ = writeln!(w, "            let _ = wm::SetWindowPos(hwnd, None, 0, 0, r.right - r.left, r.bottom - r.top, wm::SWP_NOMOVE | wm::SWP_NOZORDER);");
    let _ = writeln!(w, "            let _ = wm::ShowWindow(hwnd, wm::SW_SHOW);");
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
    let _ = writeln!(w, "        let font = win::font(hwnd, 0, false);");
    let _ = writeln!(w, "        let dpi = win::dpi(hwnd);");
    let _ = writeln!(w, "        let mut layout = Layout::new(({}, {}));", f.form.size[0], f.form.size[1]);
    if !f.toolbar.is_empty() {
        let _ = writeln!(w, "        let toolbar = win::toolbar(hwnd, ID_TOOLBAR, &Self::toolbar_items());");
        let _ = writeln!(w, "        layout.add(toolbar, Item {{ bounds: Rect::new(0, 0, 0, win::TOOLBAR_HEIGHT), dock: Dock::Top, anchor: Anchor::default() }});");
    }
    for c in &f.control {
        let id = format!("CONTROL_{}", c.id.to_ascii_uppercase());
        let text = format!("t({})", lit(&text_of(c)));
        let make = match c.kind {
            Kind::Label => format!("win::label(hwnd, {text}, {id}, font)"),
            Kind::Edit => format!("win::edit(hwnd, {}, {id}, font, {})", lit(&text_of(c)), c.password.unwrap_or(false)),
            Kind::Button => format!("win::button(hwnd, {text}, {id}, font, false)"),
            Kind::Checkbox => format!("win::checkbox(hwnd, {text}, {id}, font, {})", c.checked.unwrap_or(false)),
            Kind::Radio => format!("win::radio(hwnd, {text}, {id}, font, {}, {})", c.group.unwrap_or(0), c.checked.unwrap_or(false)),
            Kind::Combo => format!("win::combo(hwnd, {id}, font, &[{}])", c.items.iter().map(|s| format!("t({})", lit(s))).collect::<Vec<_>>().join(", ")),
            Kind::List => format!("win::list(hwnd, {id}, font, &[{}])", c.columns.iter().map(|(s, n)| format!("(t({}), {n})", lit(s))).collect::<Vec<_>>().join(", ")),
            Kind::Group => format!("controls::group(hwnd, {text}, {id}, font)"),
            Kind::Tabs => format!("controls::tabs(hwnd, {id}, font, &[{}])", c.tabs.iter().map(|s| format!("t({})", lit(s))).collect::<Vec<_>>().join(", ")),
            Kind::Statusbar => format!("controls::status_bar(hwnd, {id}, font)"),
        };
        let _ = writeln!(w, "        let {} = {make};", c.id);
        let _ = writeln!(w, "        win::set_css({}, {}, {});", c.id, lit(c.class.as_deref().unwrap_or("")), lit(&c.id));
        if c.kind == Kind::Statusbar {
            if !c.parts.is_empty() {
                let _ = writeln!(w, "        controls::status_parts({}, &[{}]);", c.id, c.parts.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "));
            }
            if !text_of(c).is_empty() {
                let _ = writeln!(w, "        controls::status_text({}, 0, {text});", c.id);
            }
        }
        if c.enabled == Some(false) {
            let _ = writeln!(w, "        win::enable({}, false);", c.id);
        }
        if c.visible == Some(false) || c.page.is_some_and(|p| p > 0) {
            let _ = writeln!(w, "        win::show({}, false);", c.id);
        }
        let [x, y, cw, ch] = c.rect();
        let ch = if c.kind == Kind::Statusbar { format!("unscale(controls::status_height({}), dpi)", c.id) } else { ch.to_string() };
        let _ = writeln!(w, "        layout.add({}, Item {{ bounds: Rect::new({x}, {y}, {cw}, {ch}), dock: {}, anchor: {} }});", c.id, dock_code(c), anchor_code(c));
    }
    let _ = writeln!(w, "        {ui} {{");
    let _ = writeln!(w, "            hwnd,");
    let _ = writeln!(w, "            font,");
    if !f.toolbar.is_empty() {
        let _ = writeln!(w, "            toolbar,");
    }
    for c in &f.control {
        let _ = writeln!(w, "            {},", c.id);
    }
    let _ = writeln!(w, "            layout,");
    let _ = writeln!(w, "        }}");
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
                let _ = writeln!(w, "        controls::tabs_relabel(self.{}, &[{}]);", c.id, c.tabs.iter().map(|s| format!("t({})", lit(s))).collect::<Vec<_>>().join(", "));
            }
            Kind::Statusbar if !text_of(c).is_empty() => {
                let _ = writeln!(w, "        controls::status_text(self.{}, 0, t({}));", c.id, lit(&text_of(c)));
            }
            _ => {}
        }
    }
    let _ = writeln!(w, "    }}");

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
    if let Some([mw, mh]) = f.form.min_size {
        let _ = writeln!(w, "    if msg == wm::WM_GETMINMAXINFO {{");
        let _ = writeln!(w, "        let info = unsafe {{ &mut *(l.0 as *mut wm::MINMAXINFO) }};");
        let _ = writeln!(w, "        let style = unsafe {{ wm::GetWindowLongW(hwnd, wm::GWL_STYLE) }} as u32;");
        let _ = writeln!(w, "        let mut r = RECT {{ left: 0, top: 0, right: win::px(hwnd, {mw}), bottom: win::px(hwnd, {mh}) }};");
        let _ = writeln!(w, "        let _ = unsafe {{ windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(style), {}, Default::default(), win::dpi(hwnd) as u32) }};", !f.menu.is_empty());
        let _ = writeln!(w, "        info.ptMinTrackSize.x = r.right - r.left;");
        let _ = writeln!(w, "        info.ptMinTrackSize.y = r.bottom - r.top;");
        let _ = writeln!(w, "        return LRESULT(0);");
        let _ = writeln!(w, "    }}");
    }
    let _ = writeln!(w, "    let Some(host) = (unsafe {{ win::state::<Host<E>>(hwnd) }}) else {{ return win::def(hwnd, msg, w, l) }};");
    let _ = writeln!(w, "    match msg {{");
    let _ = writeln!(w, "        wm::WM_SIZE => {{");
    let _ = writeln!(w, "            host.ui.layout.apply(hwnd);");
    let _ = writeln!(w, "            LRESULT(0)");
    let _ = writeln!(w, "        }}");
    let _ = writeln!(w, "        wm::WM_COMMAND => {{");
    let _ = writeln!(w, "            let (id, code) = (win::loword(w.0) as i32, win::hiword(w.0));");
    let _ = writeln!(w, "            let Host {{ ui, events }} = host;");
    let _ = writeln!(w, "            match id {{");
    for h in handlers.iter().filter(|h| f.menu.iter().flat_map(|m| &m.items).any(|it| it.id.as_deref() == Some(h.as_str())) || f.toolbar.iter().any(|t| &t.id == *h)) {
        let _ = writeln!(w, "                {} => events.{h}(ui),", const_name(h));
    }
    for c in &f.control {
        let id = format!("CONTROL_{}", c.id.to_ascii_uppercase());
        let call = c.handler().map(|h| format!("events.{h}(ui)"));
        match c.kind {
            Kind::Button | Kind::Checkbox | Kind::Radio => {
                if let Some(call) = call {
                    let _ = writeln!(w, "                {id} if code == 0 => {call},");
                }
            }
            Kind::Edit => {
                if let Some(call) = call {
                    let _ = writeln!(w, "                {id} if code == 0x0300 => {call}, // EN_CHANGE");
                }
            }
            Kind::Combo => {
                if let Some(call) = call {
                    let _ = writeln!(w, "                {id} if code == 1 => {call}, // CBN_SELCHANGE");
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
    let lists: Vec<&Control> = f.control.iter().filter(|c| c.kind == Kind::List && c.on_select.is_some()).collect();
    if !lists.is_empty() {
        let _ = writeln!(w, "        wm::WM_NOTIFY => {{");
        let _ = writeln!(w, "            use windows::Win32::UI::Controls as cc;");
        let _ = writeln!(w, "            let hdr = unsafe {{ &*(l.0 as *const cc::NMHDR) }};");
        let _ = writeln!(w, "            if hdr.code == cc::LVN_ITEMCHANGED {{");
        let _ = writeln!(w, "                let it = unsafe {{ &*(l.0 as *const cc::NMLISTVIEW) }};");
        let _ = writeln!(w, "                let chosen = it.uNewState & cc::LVIS_SELECTED.0 != 0 && it.uOldState & cc::LVIS_SELECTED.0 == 0;");
        let _ = writeln!(w, "                let Host {{ ui, events }} = host;");
        for c in lists {
            let _ = writeln!(w, "                if chosen && hdr.idFrom == CONTROL_{} as usize {{", c.id.to_ascii_uppercase());
            let _ = writeln!(w, "                    events.{}(ui);", c.on_select.as_deref().unwrap_or_default());
            let _ = writeln!(w, "                }}");
        }
        let _ = writeln!(w, "            }}");
        let _ = writeln!(w, "            win::def(hwnd, msg, w, l)");
        let _ = writeln!(w, "        }}");
    }
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
    let _ = writeln!(w, "            unsafe {{ wm::PostQuitMessage(0) }};");
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
    for (i, h) in f.handlers().iter().enumerate() {
        if i > 0 {
            let _ = writeln!(w);
        }
        let _ = writeln!(w, "    fn {h}(&mut self, _ui: &{name}Ui) {{}}");
    }
    let _ = writeln!(w, "}}");
    o
}

/// A handler's empty method to add to the programmer's file, if it has none by that name yet -
/// found by its text only; the designer never parses the programmer's code. `None` when it is
/// there, or when the file has no `impl ...Events for` to put it in.
pub fn add_handler(user_src: &str, form_name: &str, handler: &str) -> Option<String> {
    if user_src.contains(&format!("fn {handler}(")) {
        return None;
    }
    let marker = format!("impl {form_name}Events for ");
    let start = user_src.find(&marker)?;
    // The impl's closing brace: the first `}` at the start of a line after it.
    let at = start + user_src[start..].find("\n}")?;
    Some(format!("{}\n\n    fn {handler}(&mut self, _ui: &{form_name}Ui) {{}}{}", &user_src[..at], &user_src[at..]))
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
            assert!(code.contains(&format!("pub {}: HWND,", c.id)), "{}", c.id);
        }
        assert!(!code.contains("include_str!"), "no theme, no include");
    }

    #[test]
    fn a_new_handler_goes_into_the_impl_once() {
        let f = FormFile::parse(FORM).unwrap();
        let stub = user_stub(&f);
        assert!(add_handler(&stub, "MainWindow", "save").is_none(), "already there");
        let more = add_handler(&stub, "MainWindow", "print").unwrap();
        assert!(more.contains("    fn print(&mut self, _ui: &MainWindowUi) {}\n}"), "{more}");
        assert!(add_handler(&more, "MainWindow", "print").is_none());
        assert!(add_handler("struct X;", "MainWindow", "print").is_none(), "nowhere to put it");
    }
}
