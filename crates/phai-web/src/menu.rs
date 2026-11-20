//! A window's menu bar and toolbar on the page - the same items as on Windows: menus within
//! menus, ticks a click turns over, greyed items, shortcut keys, toolbar buttons that stay down.
//! A tap opens a menu on a phone as a click does with the mouse.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

use crate::win::{self, el, make, HWND};

/// One item of a menu, as the form has it.
#[derive(Clone, Debug, Default)]
pub struct Item {
    /// The program's method for it; empty for a separator or a menu of its own.
    pub id: &'static str,
    pub label: &'static str,
    /// Its shortcut, as `Ctrl+S`.
    pub key: &'static str,
    /// A Segoe icon's code, 0 for none.
    pub glyph: u16,
    pub checked: bool,
    pub check_on_click: bool,
    pub disabled: bool,
    pub separator: bool,
    /// A menu of its own that opens beside it.
    pub items: Vec<Item>,
}

/// A menu of the bar: its words and its items.
#[derive(Clone, Debug, Default)]
pub struct Menu {
    pub label: &'static str,
    pub items: Vec<Item>,
}

/// A toolbar button.
#[derive(Clone, Debug, Default)]
pub struct Tool {
    pub id: &'static str,
    pub label: &'static str,
    pub glyph: u16,
    /// It stays down until clicked again, ticked with the menu item of the same id.
    pub toggle: bool,
}

type Chosen = Rc<RefCell<dyn FnMut(&String)>>;

/// The words with Windows' `&` (the underlined letter) taken out; `&&` stays one `&`.
pub fn plain(label: &str) -> String {
    label.replace("&&", "\u{1}").replace('&', "").replace('\u{1}', "&")
}

/// Whether Windows' icon font is here - not on a phone, so a toolbar shows words there.
fn icon_font() -> bool {
    thread_local! {
        static HAS: std::cell::OnceCell<bool> = const { std::cell::OnceCell::new() };
    }
    HAS.with(|h| {
        *h.get_or_init(|| {
            let Some(body) = win::document().body() else { return false };
            let probe = |font: &str| {
                let s = make("span");
                s.style().set_css_text(&format!("position:absolute;visibility:hidden;font-size:40px;font-family:{font}"));
                s.set_text_content(Some("\u{E721}\u{E74E}"));
                let _ = body.append_child(&s);
                let w = s.offset_width();
                s.remove();
                w
            };
            probe("'Segoe Fluent Icons','Segoe MDL2 Assets',monospace") != probe("monospace")
        })
    })
}

fn root_of(window: HWND) -> Option<HtmlElement> {
    el(window)
}

/// Every element of item `id` in `window` - its menu item and its toolbar button.
fn parts(window: HWND, id: &str) -> Vec<Element> {
    let Some(r) = root_of(window) else { return vec![] };
    let Ok(list) = r.query_selector_all(&format!("[data-item=\"{id}\"]")) else { return vec![] };
    (0..list.length()).filter_map(|i| list.item(i)).filter_map(|n| n.dyn_into::<Element>().ok()).collect()
}

/// Whether item `id` is ticked - its menu item or its toolbar button.
pub fn item_checked(window: HWND, id: &str) -> bool {
    parts(window, id).iter().any(|e| e.class_list().contains("checked"))
}

/// Item `id` ticked or not, in the menu and on the toolbar alike.
pub fn set_item_checked(window: HWND, id: &str, on: bool) {
    for e in parts(window, id) {
        let _ = e.class_list().toggle_with_force("checked", on);
        if e.tag_name() == "BUTTON" {
            let _ = e.set_attribute("aria-pressed", if on { "true" } else { "false" });
        }
    }
}

/// Item `id` usable or greyed, in the menu and on the toolbar alike.
pub fn set_item_enabled(window: HWND, id: &str, on: bool) {
    for e in parts(window, id) {
        let _ = e.class_list().toggle_with_force("disabled", !on);
        if on {
            let _ = e.remove_attribute("disabled");
        } else {
            let _ = e.set_attribute("disabled", "");
        }
    }
}

fn enabled(window: HWND, id: &str) -> bool {
    parts(window, id).iter().all(|e| !e.class_list().contains("disabled"))
}

/// Item `id` chosen: its tick turned over when it ticks itself, then the program told.
fn choose(window: HWND, id: &str, toggles: bool, chosen: &Chosen) {
    if !enabled(window, id) {
        return;
    }
    close_all();
    if toggles {
        let on = !item_checked(window, id);
        set_item_checked(window, id, on);
    }
    if let Ok(mut f) = chosen.try_borrow_mut() {
        f(&id.to_string());
    }
}

fn on_click(e: &HtmlElement, f: impl FnMut(web_sys::MouseEvent) + 'static) {
    let c = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(f);
    let _ = e.add_event_listener_with_callback("click", c.as_ref().unchecked_ref());
    c.forget();
}

thread_local! {
    /// The menus open now, outermost first.
    static OPEN: RefCell<Vec<HtmlElement>> = const { RefCell::new(Vec::new()) };
}

/// Every open menu closed.
pub fn close_all() {
    OPEN.with(|o| {
        for m in o.borrow_mut().drain(..) {
            m.remove();
        }
    });
    if let Ok(list) = win::document().query_selector_all(".phai-menu-top.open") {
        for i in 0..list.length() {
            if let Some(e) = list.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                let _ = e.class_list().remove_1("open");
            }
        }
    }
}

/// The open menus deeper than `depth` closed.
fn close_from(depth: usize) {
    OPEN.with(|o| {
        let mut o = o.borrow_mut();
        while o.len() > depth {
            if let Some(m) = o.pop() {
                m.remove();
            }
        }
    });
}

/// A menu of `items` opened at (`x`, `y`) on the page, kept inside the screen.
fn open_menu(window: HWND, items: &[Item], x: f64, y: f64, depth: usize, chosen: &Chosen) {
    close_from(depth);
    let menu = make("div");
    menu.set_class_name("phai-dropdown");
    menu.set_attribute("role", "menu").ok();
    for it in items {
        if it.separator {
            let hr = make("div");
            hr.set_class_name("phai-menu-sep");
            let _ = menu.append_child(&hr);
            continue;
        }
        let row = make("div");
        row.set_class_name("phai-menu-item");
        row.set_attribute("role", "menuitem").ok();
        if !it.id.is_empty() {
            row.set_attribute("data-item", it.id).ok();
        }
        // The tick or the item's picture, its words, its key or the arrow to its menu.
        let mark = make("span");
        mark.set_class_name("phai-menu-mark");
        if it.glyph != 0 && icon_font() {
            mark.set_text_content(Some(&char::from_u32(it.glyph as u32).unwrap_or(' ').to_string()));
            mark.class_list().add_1("phai-glyph").ok();
        }
        let words = make("span");
        words.set_class_name("phai-menu-label");
        words.set_text_content(Some(&plain(it.label)));
        let key = make("span");
        key.set_class_name("phai-menu-key");
        key.set_text_content(Some(if it.items.is_empty() { it.key } else { "\u{203A}" }));
        let _ = row.append_child(&mark);
        let _ = row.append_child(&words);
        let _ = row.append_child(&key);
        let _ = menu.append_child(&row);
        // The tick and the grey as the window has them now (the toolbar's button may have
        // changed them).
        let now_checked = !it.id.is_empty() && (item_checked(window, it.id) || (it.checked && parts(window, it.id).is_empty()));
        if now_checked {
            row.class_list().add_1("checked").ok();
        }
        let now_disabled = it.disabled || (!it.id.is_empty() && !enabled(window, it.id));
        if now_disabled {
            row.class_list().add_1("disabled").ok();
        }
        if !it.items.is_empty() {
            let sub = it.items.clone();
            let chosen = chosen.clone();
            let r = row.clone();
            let open_sub = move || {
                let b = r.get_bounding_client_rect();
                open_menu(window, &sub, b.right() - 4.0, b.top(), depth + 1, &chosen);
            };
            let o1 = open_sub.clone();
            on_click(&row, move |e| {
                e.stop_propagation();
                o1();
            });
            let c = Closure::<dyn FnMut()>::new(open_sub);
            let _ = row.add_event_listener_with_callback("mouseenter", c.as_ref().unchecked_ref());
            c.forget();
        } else if !it.id.is_empty() {
            let (id, toggles, chosen) = (it.id, it.check_on_click, chosen.clone());
            on_click(&row, move |e| {
                e.stop_propagation();
                choose(window, id, toggles, &chosen);
            });
            let c = Closure::<dyn FnMut()>::new(move || close_from(depth + 1));
            let _ = row.add_event_listener_with_callback("mouseenter", c.as_ref().unchecked_ref());
            c.forget();
        }
    }
    let Some(body) = win::document().body() else { return };
    let _ = body.append_child(&menu);
    // Inside the screen: moved left (or beside its parent's other side) when it would run off.
    let (vw, vh) = (win::window().inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(800.0), win::window().inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(600.0));
    let b = menu.get_bounding_client_rect();
    let left = if x + b.width() > vw { (vw - b.width() - 4.0).max(0.0) } else { x };
    let top = if y + b.height() > vh { (vh - b.height() - 4.0).max(0.0) } else { y };
    let s = menu.style();
    let _ = s.set_property("left", &format!("{left}px"));
    let _ = s.set_property("top", &format!("{top}px"));
    OPEN.with(|o| o.borrow_mut().push(menu));
}

/// The window's menu bar, under its title: each menu opens with a click or a tap, and the next
/// one under the pointer while one is open. `chosen` is given each item's id.
pub fn menu_bar(window: HWND, menus: Vec<Menu>, chosen: impl FnMut(&String) + 'static) {
    let Some(root) = el(window) else { return };
    let chosen: Chosen = Rc::new(RefCell::new(chosen));
    let bar = make("div");
    bar.set_class_name("phai-menubar");
    bar.set_attribute("role", "menubar").ok();
    // Hidden items, so the ticks and greys have a place before a menu first opens.
    let state = make("div");
    state.set_class_name("phai-menu-state");
    fn keep_state(state: &HtmlElement, items: &[Item]) {
        for it in items {
            if !it.id.is_empty() {
                let s = make("span");
                s.set_attribute("data-item", it.id).ok();
                if it.checked {
                    s.class_list().add_1("checked").ok();
                }
                if it.disabled {
                    s.class_list().add_1("disabled").ok();
                }
                let _ = state.append_child(&s);
            }
            keep_state(state, &it.items);
        }
    }
    for m in &menus {
        keep_state(&state, &m.items);
    }
    let _ = bar.append_child(&state);
    for m in menus {
        let top = make("button");
        top.set_class_name("phai-menu-top");
        top.set_text_content(Some(&plain(m.label)));
        let items = m.items.clone();
        let (t, ch) = (top.clone(), chosen.clone());
        let open = move || {
            close_all();
            t.class_list().add_1("open").ok();
            let b = t.get_bounding_client_rect();
            open_menu(window, &items, b.left(), b.bottom(), 0, &ch);
        };
        let o1 = open.clone();
        let t2 = top.clone();
        on_click(&top, move |e| {
            e.stop_propagation();
            if t2.class_list().contains("open") {
                close_all();
            } else {
                o1();
            }
        });
        // With one open, the pointer moving onto another opens that one.
        let o2 = open;
        let t3 = top.clone();
        let c = Closure::<dyn FnMut()>::new(move || {
            let any_open = OPEN.with(|o| !o.borrow().is_empty());
            if any_open && !t3.class_list().contains("open") {
                o2();
            }
        });
        let _ = top.add_event_listener_with_callback("mouseenter", c.as_ref().unchecked_ref());
        c.forget();
        let _ = bar.append_child(&top);
    }
    // Under the title bar.
    let title = root.first_element_child();
    match title {
        Some(t) => {
            let _ = t.after_with_node_1(&bar);
        }
        None => {
            let _ = root.prepend_with_node_1(&bar);
        }
    }
    // The controls' area is shorter by the bar now.
    crate::form::fit_all();
    close_on_click_outside();
}

fn close_on_click_outside() {
    thread_local! {
        static DONE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }
    if DONE.with(|d| d.replace(true)) {
        return;
    }
    let c = Closure::<dyn FnMut(web_sys::Event)>::new(|e: web_sys::Event| {
        let inside = e.target().and_then(|t| t.dyn_into::<Element>().ok()).is_some_and(|t| t.closest(".phai-dropdown").ok().flatten().is_some());
        if !inside {
            close_all();
        }
    });
    let _ = win::document().add_event_listener_with_callback("click", c.as_ref().unchecked_ref());
    c.forget();
    let k = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(|e: web_sys::KeyboardEvent| {
        if e.key() == "Escape" {
            close_all();
        }
    });
    let _ = win::document().add_event_listener_with_callback("keydown", k.as_ref().unchecked_ref());
    k.forget();
}

/// The window's toolbar along the top of its controls' area, `height` designer pixels tall.
pub fn toolbar(window: HWND, area: HWND, tools: Vec<Tool>, height: i32, chosen: impl FnMut(&String) + 'static) -> HWND {
    let chosen: Chosen = Rc::new(RefCell::new(chosen));
    let bar = make("div");
    bar.set_class_name("phai-toolbar");
    bar.set_attribute("role", "toolbar").ok();
    bar.style().set_css_text(&format!("left:0;right:0;top:0;height:{height}px;"));
    let icons = icon_font();
    for t in tools {
        let b = make("button");
        b.set_class_name("phai-tool");
        b.set_attribute("data-item", t.id).ok();
        b.set_title(&plain(t.label));
        if t.toggle {
            b.set_attribute("aria-pressed", "false").ok();
        }
        if icons && t.glyph != 0 {
            let g = make("span");
            g.set_class_name("phai-glyph");
            g.set_text_content(Some(&char::from_u32(t.glyph as u32).unwrap_or(' ').to_string()));
            let _ = b.append_child(&g);
        }
        let w = make("span");
        w.set_text_content(Some(&plain(t.label)));
        let _ = b.append_child(&w);
        let (id, toggle, ch) = (t.id, t.toggle, chosen.clone());
        on_click(&b, move |_| choose(window, id, toggle, &ch));
        let _ = bar.append_child(&b);
    }
    if let Some(a) = el(area) {
        let _ = a.prepend_with_node_1(&bar);
    }
    // A tick the menu had from the start shows on the button too.
    if let Some(r) = root_of(window) {
        if let Ok(list) = r.query_selector_all(".phai-menu-state .checked") {
            for i in 0..list.length() {
                if let Some(id) = list.item(i).and_then(|n| n.dyn_into::<Element>().ok()).and_then(|e| e.get_attribute("data-item")) {
                    set_item_checked(window, &id, true);
                }
            }
        }
    }
    win::keep(bar)
}

/// A shortcut as the form writes it (`Ctrl+Shift+S`, `F5`, `Del`) in one spelling, to compare.
pub fn normal_key(k: &str) -> String {
    let parts: Vec<String> = k.split('+').map(|p| p.trim().to_ascii_lowercase()).filter(|p| !p.is_empty()).collect();
    // A `+` itself: `Ctrl++`.
    let mut parts = parts;
    if k.ends_with("++") {
        parts.push("+".into());
    }
    let mut mods: Vec<&str> = Vec::new();
    let mut main = String::new();
    for p in &parts {
        match p.as_str() {
            "ctrl" | "control" => mods.push("ctrl"),
            "shift" => mods.push("shift"),
            "alt" => mods.push("alt"),
            "del" | "delete" => main = "delete".into(),
            "ins" | "insert" => main = "insert".into(),
            "esc" | "escape" => main = "escape".into(),
            "enter" | "return" => main = "enter".into(),
            "pgup" | "pageup" => main = "pageup".into(),
            "pgdn" | "pagedown" => main = "pagedown".into(),
            other => main = other.to_string(),
        }
    }
    mods.sort();
    mods.dedup();
    let mut s = mods.join("+");
    if !s.is_empty() {
        s.push('+');
    }
    s + &main
}

/// The same spelling for a key the browser hears.
fn key_of(e: &web_sys::KeyboardEvent) -> String {
    let mut mods = Vec::new();
    if e.alt_key() {
        mods.push("alt");
    }
    if e.ctrl_key() || e.meta_key() {
        mods.push("ctrl");
    }
    if e.shift_key() {
        mods.push("shift");
    }
    let key = e.key().to_ascii_lowercase();
    let main = match key.as_str() {
        "arrowleft" => "left".to_string(),
        "arrowright" => "right".to_string(),
        "arrowup" => "up".to_string(),
        "arrowdown" => "down".to_string(),
        " " => "space".to_string(),
        k => k.to_string(),
    };
    let mut s = mods.join("+");
    if !s.is_empty() {
        s.push('+');
    }
    s + &main
}

/// The menu items' shortcut keys heard anywhere on the page while `window` is open.
pub fn shortcuts(window: HWND, menus: &[Menu], chosen: impl FnMut(&String) + 'static) {
    let mut keys: Vec<(String, &'static str, bool)> = Vec::new();
    fn walk(items: &[Item], keys: &mut Vec<(String, &'static str, bool)>) {
        for it in items {
            if !it.key.is_empty() && !it.id.is_empty() {
                keys.push((normal_key(it.key), it.id, it.check_on_click));
            }
            walk(&it.items, keys);
        }
    }
    for m in menus {
        walk(&m.items, &mut keys);
    }
    if keys.is_empty() {
        return;
    }
    let chosen: Chosen = Rc::new(RefCell::new(chosen));
    let c = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
        // A window closed since does not hear them.
        if el(window).is_none_or(|r| !r.is_connected()) {
            return;
        }
        let k = key_of(&e);
        if let Some((_, id, toggles)) = keys.iter().find(|(n, _, _)| *n == k) {
            e.prevent_default();
            choose(window, id, *toggles, &chosen);
        }
    });
    let _ = win::document().add_event_listener_with_callback("keydown", c.as_ref().unchecked_ref());
    c.forget();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcuts_are_spelled_one_way() {
        assert_eq!(normal_key("Ctrl+S"), "ctrl+s");
        assert_eq!(normal_key("Shift+Ctrl+N"), "ctrl+shift+n");
        assert_eq!(normal_key("F5"), "f5");
        assert_eq!(normal_key("Del"), "delete");
        assert_eq!(normal_key("Ctrl+="), "ctrl+=");
        assert_eq!(normal_key("Ctrl++"), "ctrl++");
        assert_eq!(plain("&File"), "File");
        assert_eq!(plain("Save && quit"), "Save & quit");
    }
}
