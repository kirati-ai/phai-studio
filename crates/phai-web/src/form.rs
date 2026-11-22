//! A form on the page: its window (the whole screen for the main one, a box over it for another),
//! its controls placed where the designer put them - anchored and docked as on Windows - and
//! their events. The generated `<form>_ui.rs` calls these; a program's own code seldom does.

use std::cell::RefCell;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlInputElement, HtmlSelectElement};

use crate::events::{KeyEventArgs, KeyPressEventArgs, MouseButton, MouseEventArgs};
use crate::win::{self, el, keep, make, HWND};

/// What a dialog answers - WinForms' DialogResult.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogResult {
    #[default]
    None,
    Ok,
    Cancel,
    Yes,
    No,
    Retry,
    Abort,
    Ignore,
}

/// Where a control goes in its window, in the designer's pixels, and what it holds on to.
#[derive(Clone, Copy, Debug)]
pub struct Place {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    /// `left`, `top`, `right`, `bottom` - the edges it keeps its distance to; `all`.
    pub anchor: &'static str,
    /// `top`, `bottom`, `left`, `right`, `fill` or `none`.
    pub dock: &'static str,
    /// The window's size in the designer, which the distances are measured in.
    pub within: (i32, i32),
}

/// The CSS that puts a control at `p`.
pub fn place_css(p: &Place) -> String {
    let (fw, fh) = p.within;
    match p.dock {
        "top" => format!("left:0;right:0;top:0;height:{}px;", p.h),
        "bottom" => format!("left:0;right:0;bottom:0;height:{}px;", p.h),
        "left" => format!("left:0;top:0;bottom:0;width:{}px;", p.w),
        "right" => format!("right:0;top:0;bottom:0;width:{}px;", p.w),
        "fill" => "left:0;top:0;right:0;bottom:0;".to_string(),
        _ => {
            let a = p.anchor;
            let has = |e: &str| a == "all" || a.split_whitespace().any(|x| x == e);
            // WinForms' default is left and top.
            let (l, r) = (has("left") || !has("right"), has("right"));
            let (t, b) = (has("top") || !has("bottom"), has("bottom"));
            let right = fw - p.x - p.w;
            let bottom = fh - p.y - p.h;
            let h = match (l, r) {
                (true, true) => format!("left:{}px;right:{}px;", p.x, right),
                (false, true) => format!("right:{right}px;width:{}px;", p.w),
                _ => format!("left:{}px;width:{}px;", p.x, p.w),
            };
            let v = match (t, b) {
                (true, true) => format!("top:{}px;bottom:{}px;", p.y, bottom),
                (false, true) => format!("bottom:{bottom}px;height:{}px;", p.h),
                _ => format!("top:{}px;height:{}px;", p.y, p.h),
            };
            h + &v
        }
    }
}

/// A window on the page: its element, the area its controls go in, its size in the designer.
type Shown = (HWND, HWND, (i32, i32));

thread_local! {
    /// Each window's design size and the area its controls go in, to shrink it onto a phone.
    static WINDOWS: RefCell<Vec<Shown>> = const { RefCell::new(Vec::new()) };
    static STARTED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// The areas whose controls stack on a narrow screen (PhoneLayout = stack).
    static STACKED: RefCell<Vec<HWND>> = const { RefCell::new(Vec::new()) };
}

/// The window's controls one under another when the screen is narrower than the form - a phone -
/// rather than the whole form shrunk.
pub fn stack_when_narrow(area: HWND) {
    STACKED.with(|s| s.borrow_mut().push(area));
    let size = WINDOWS.with(|w| w.borrow().iter().find(|x| x.1 == area).map(|x| x.2));
    if let Some(size) = size {
        fit(area, size);
    }
}

/// Where a control comes when stacked: in the order a page is read - top to bottom, then left
/// to right; what is docked at the top first, at the bottom last.
pub fn stack_order(p: &Place) -> i32 {
    match p.dock {
        "top" => -1_000_000 + p.y,
        "bottom" => 100_000_000,
        _ => p.y.clamp(0, 9_999) * 10_000 + p.x.clamp(0, 9_999),
    }
}

/// Once, before the first window: Thai when the page asks for it (`?lang=th`) or the browser
/// does, and the page following its size.
pub fn start() {
    if STARTED.with(|s| s.replace(true)) {
        return;
    }
    let search = win::window().location().search().unwrap_or_default();
    let thai = if search.contains("lang=th") {
        true
    } else if search.contains("lang=en") {
        false
    } else {
        win::browser_thai()
    };
    crate::text::set_thai(thai);
    let fit = Closure::<dyn FnMut()>::new(fit_all);
    let _ = win::window().add_event_listener_with_callback("resize", fit.as_ref().unchecked_ref());
    fit.forget();
}

/// Every window as big as the page lets it be: shrunk to the screen's width when its design is
/// wider (a phone), its anchored controls stretched when the screen is bigger.
pub(crate) fn fit_all() {
    let all = WINDOWS.with(|w| w.borrow().clone());
    for (_, area, size) in all {
        fit(area, size);
    }
}

fn fit(area: HWND, (dw, dh): (i32, i32)) {
    let (Some(area), Some(holder)) = (el(area), el(area).and_then(|a| a.parent_element()).map(|p| p.unchecked_into::<HtmlElement>())) else { return };
    let (cw, ch) = (holder.client_width().max(1) as f64, holder.client_height().max(1) as f64);
    // A page not shown yet (a tab behind another) has no width: fitted when it shows, as the
    // browser then says it was resized.
    if cw < 64.0 {
        return;
    }
    let s = area.style();
    let stacked = STACKED.with(|st| st.borrow().iter().any(|h| el(*h).is_some_and(|e| e == area)));
    if stacked && cw < dw as f64 {
        let _ = area.class_list().add_1("phai-stacked");
        let _ = s.set_property("zoom", "1");
        let _ = s.set_property("width", &format!("{}px", cw.floor()));
        let _ = s.remove_property("height");
        return;
    }
    let _ = area.class_list().remove_1("phai-stacked");
    let zoom = (cw / dw.max(1) as f64).min(1.0);
    let _ = s.set_property("zoom", &format!("{zoom}"));
    let _ = s.set_property("width", &format!("{}px", (cw / zoom).floor()));
    let _ = s.set_property("height", &format!("{}px", (ch / zoom).max(dh as f64).floor()));
}

/// A window: the main one fills the page, another stands over it until it closes. Its title in
/// a bar along the top. The area its controls go in.
pub fn window(name: &str, title: &str, size: (i32, i32), main: bool) -> (HWND, HWND) {
    start();
    let doc = win::document();
    let root = make("div");
    root.set_class_name(if main { "phai-window phai-main" } else { "phai-window phai-dialog" });
    root.set_attribute("data-form", name).ok();
    let bar = make("div");
    bar.set_class_name("phai-titlebar");
    let words = make("span");
    words.set_text_content(Some(title));
    let _ = bar.append_child(&words);
    let body = make("div");
    body.set_class_name("phai-body");
    let area = make("div");
    area.set_class_name("phai-area");
    let _ = body.append_child(&area);
    let _ = root.append_child(&bar);
    let _ = root.append_child(&body);
    let root_h = keep(root.clone());
    if !main {
        // Another window: a box over the page, its own close button.
        let shade = make("div");
        shade.set_class_name("phai-shade");
        let close = make("button");
        close.set_class_name("phai-close");
        close.set_text_content(Some("\u{2715}"));
        close.set_attribute("aria-label", crate::text::t("Close")).ok();
        let _ = bar.append_child(&close);
        let s = root.style();
        let _ = s.set_property("width", &format!("min({}px, 100vw)", size.0));
        let _ = s.set_property("height", &format!("min({}px, 100dvh)", size.1 + 40));
        let _ = shade.append_child(&root);
        let shade_h = keep(shade.clone());
        let _ = doc.body().map(|b| b.append_child(&shade));
        on_el(&close, "click", move || {
            win::forget(shade_h);
        });
    } else if let Some(b) = doc.body() {
        let _ = b.append_child(&root);
        doc.set_title(title);
    }
    let area_h = keep(area);
    WINDOWS.with(|w| w.borrow_mut().push((root_h, area_h, size)));
    fit(area_h, size);
    (root_h, area_h)
}

/// The window `root` gone - Close().
pub fn close(root: HWND) {
    let Some(r) = el(root) else { return };
    WINDOWS.with(|w| w.borrow_mut().retain(|x| x.0 != root));
    match r.parent_element() {
        Some(p) if p.class_name() == "phai-shade" => p.remove(),
        _ => r.remove(),
    }
}

/// The element of a control of `kind` (the theme's names: `button`, `edit`...).
fn element(kind: &str, text: &str) -> HtmlElement {
    let input = |ty: &str| {
        let e = make("input");
        e.set_attribute("type", ty).ok();
        e
    };
    match kind {
        "edit" | "masked" => {
            let e = input("text");
            e.unchecked_ref::<HtmlInputElement>().set_value(text);
            e
        }
        "number" => {
            let e = input("number");
            e.unchecked_ref::<HtmlInputElement>().set_value(if text.is_empty() { "0" } else { text });
            e
        }
        "textarea" | "richtext" => {
            let e = make("textarea");
            e.unchecked_ref::<web_sys::HtmlTextAreaElement>().set_value(text);
            e
        }
        "button" => {
            let e = make("button");
            e.set_text_content(Some(text));
            e
        }
        "checkbox" | "radio" => {
            let e = make("label");
            let i = input(kind);
            let words = make("span");
            words.set_text_content(Some(text));
            let _ = e.append_child(&i);
            let _ = e.append_child(&words);
            e
        }
        "combo" => make("select"),
        "listbox" | "checkedlist" => {
            let e = make("select");
            e.set_attribute("size", "4").ok();
            e
        }
        "date" | "calendar" => input("date"),
        "slider" => input("range"),
        "progress" => {
            let e = make("progress");
            e.set_attribute("max", "100").ok();
            e.set_attribute("value", "0").ok();
            e
        }
        "group" => {
            let e = make("fieldset");
            let l = make("legend");
            l.set_text_content(Some(text));
            let _ = e.append_child(&l);
            e
        }
        "link" => {
            let e = make("a");
            e.set_attribute("href", "#").ok();
            e.set_text_content(Some(text));
            e
        }
        "picture" => make("img"),
        "statusbar" => {
            let e = make("div");
            let part = make("span");
            part.set_text_content(Some(text));
            let _ = e.append_child(&part);
            e
        }
        "list" | "grid" => {
            let e = make("div");
            let t = make("table");
            let _ = t.append_child(&make("thead"));
            let _ = t.append_child(&make("tbody"));
            let _ = e.append_child(&t);
            e
        }
        "tabs" | "tree" => make("div"),
        _ => {
            let e = make("div");
            e.set_text_content(Some(text));
            e
        }
    }
}

/// A control of `kind` in window area `area`: its id and class for the theme, its words, its place.
pub fn control(area: HWND, kind: &str, id: &str, class: &str, text: &str, p: Place) -> HWND {
    let e = element(kind, text);
    // The theme's own kind for a number box, a calendar, a list box...
    let theme_kind = match kind {
        "number" | "masked" | "textarea" | "richtext" => "edit",
        "listbox" | "checkedlist" | "grid" => "list",
        other => other,
    };
    let mut classes = format!("phai phai-{theme_kind} phai-k-{kind}");
    for c in class.split_whitespace() {
        classes.push(' ');
        classes.push_str(c);
    }
    e.set_class_name(&classes);
    if !id.is_empty() {
        e.set_id(id);
    }
    e.style().set_css_text(&format!("{}order:{};", place_css(&p), stack_order(&p)));
    if let Some(a) = el(area) {
        let _ = a.append_child(&e);
    }
    let h = keep(e);
    if matches!(kind, "list" | "grid") {
        crate::ctl::rows_choosable(h);
    }
    if kind == "grid" {
        crate::ctl::rows_editable(h);
    }
    if kind == "tree" {
        crate::tree::make_tree(h);
    }
    h
}

/// The lines of a drop-down or a list box.
pub fn set_items(h: HWND, items: &[&str]) {
    let Some(e) = el(h) else { return };
    if let Some(s) = e.dyn_ref::<HtmlSelectElement>() {
        s.set_length(0);
        for it in items {
            let o = win::make("option");
            o.set_text_content(Some(it));
            let _ = s.append_child(&o);
        }
        s.set_selected_index(-1);
    }
}

/// A list box that takes several lines.
pub fn set_multi(h: HWND) {
    if let Some(s) = el(h).and_then(|e| e.dyn_into::<HtmlSelectElement>().ok()) {
        s.set_multiple(true);
    }
}

/// A text box that hides what is typed.
pub fn set_password(h: HWND) {
    if let Some(i) = el(h).and_then(|e| e.dyn_into::<HtmlInputElement>().ok()) {
        i.set_type("password");
    }
}

/// The input inside a check box or a radio, or the element itself.
pub(crate) fn input_of(h: HWND) -> Option<HtmlInputElement> {
    let e = el(h)?;
    if let Ok(i) = e.clone().dyn_into::<HtmlInputElement>() {
        return Some(i);
    }
    e.query_selector("input").ok().flatten().and_then(|i| i.dyn_into().ok())
}

/// Radios of one group in one window choose among themselves.
pub fn set_radio_group(h: HWND, window: &str, group: i32) {
    if let Some(i) = input_of(h) {
        i.set_name(&format!("{window}-{group}"));
    }
}

pub fn set_checked(h: HWND, on: bool) {
    if let Some(i) = input_of(h) {
        i.set_checked(on);
    }
}

/// A slider's or a number box's range.
pub fn set_range(h: HWND, min: i32, max: i32) {
    if let Some(i) = input_of(h) {
        i.set_min(&min.to_string());
        i.set_max(&max.to_string());
    }
}

/// A list's or a grid's headings.
pub fn set_columns(h: HWND, columns: &[(&str, i32)]) {
    let Some(head) = el(h).and_then(|e| e.query_selector("thead").ok().flatten()) else { return };
    let row = win::make("tr");
    for (c, w) in columns {
        let th = win::make("th");
        th.set_text_content(Some(c));
        if *w > 0 {
            let _ = th.style().set_property("width", &format!("{w}px"));
        }
        let _ = row.append_child(&th);
    }
    head.set_inner_html("");
    let _ = head.append_child(&row);
}

/// The tabs of a tab control: the controls on page `n` show while its tab is chosen.
pub fn set_tabs(h: HWND, area: HWND, tabs: &[&str]) {
    let Some(e) = el(h) else { return };
    for (i, t) in tabs.iter().enumerate() {
        let b = win::make("button");
        b.set_class_name(if i == 0 { "phai-tab selected" } else { "phai-tab" });
        b.set_text_content(Some(t));
        let _ = e.append_child(&b);
        on_el(&b, "click", move || select_tab(h, area, i));
    }
}

/// Page `i` of tab control `h` shown, the others' controls hidden.
pub fn select_tab(h: HWND, area: HWND, i: usize) {
    let Some(e) = el(h) else { return };
    let Some(a) = el(area).or_else(|| e.parent_element().map(|p| p.unchecked_into())) else { return };
    let tabs = e.children();
    for k in 0..tabs.length() {
        if let Some(t) = tabs.item(k) {
            t.set_class_name(if k as usize == i { "phai-tab selected" } else { "phai-tab" });
        }
    }
    let id = e.id();
    if let Ok(list) = a.query_selector_all(&format!("[data-tabs=\"{id}\"]")) {
        for k in 0..list.length() {
            if let Some(c) = list.item(k).and_then(|n| n.dyn_into::<HtmlElement>().ok()) {
                let page = c.get_attribute("data-page").and_then(|p| p.parse::<usize>().ok()).unwrap_or(0);
                let _ = c.style().set_property("visibility", if page == i { "visible" } else { "hidden" });
            }
        }
    }
    e.set_attribute("data-selected", &i.to_string()).ok();
}

/// Control `h` on page `page` of the tab control with id `tabs`.
pub fn set_page(h: HWND, tabs: &str, page: usize) {
    if let Some(e) = el(h) {
        e.set_attribute("data-tabs", tabs).ok();
        e.set_attribute("data-page", &page.to_string()).ok();
        if page > 0 {
            let _ = e.style().set_property("visibility", "hidden");
        }
    }
}

fn on_el(e: &HtmlElement, event: &str, mut f: impl FnMut() + 'static) {
    let c = Closure::<dyn FnMut(web_sys::Event)>::new(move |_e: web_sys::Event| f());
    let _ = e.add_event_listener_with_callback(event, c.as_ref().unchecked_ref());
    c.forget();
}

/// `f` each time control `h` has browser event `event` (`click`, `input`, `change`...).
pub fn on(h: HWND, event: &str, f: impl FnMut() + 'static) {
    if let Some(e) = el(h) {
        let e = if event == "click" || event == "dblclick" { e } else { input_of(h).map(|i| i.unchecked_into()).unwrap_or(e) };
        on_el(&e, event, f);
    }
}

/// `f` with a link's click, the page staying where it is.
pub fn on_link(h: HWND, mut f: impl FnMut() + 'static) {
    if let Some(e) = el(h) {
        let c = Closure::<dyn FnMut(web_sys::Event)>::new(move |ev: web_sys::Event| {
            ev.prevent_default();
            f()
        });
        let _ = e.add_event_listener_with_callback("click", c.as_ref().unchecked_ref());
        c.forget();
    }
}

/// `f` with each key on control `h` - `keydown` / `keyup` as KeyDown / KeyUp.
pub fn on_key(h: HWND, event: &str, mut f: impl FnMut(&KeyEventArgs) + 'static) {
    let Some(e) = el(h) else { return };
    focusable(&e);
    let c = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |k: web_sys::KeyboardEvent| {
        let a = KeyEventArgs { key: crate::events::virtual_key(&k.key()), ctrl: k.ctrl_key(), shift: k.shift_key(), alt: k.alt_key(), ..Default::default() };
        f(&a);
        if a.handled() {
            k.prevent_default();
        }
    });
    let _ = e.add_event_listener_with_callback(event, c.as_ref().unchecked_ref());
    c.forget();
}

/// `f` with each letter typed on control `h` - KeyPress.
pub fn on_key_press(h: HWND, mut f: impl FnMut(&KeyPressEventArgs) + 'static) {
    let Some(e) = el(h) else { return };
    focusable(&e);
    let c = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |k: web_sys::KeyboardEvent| {
        let key = k.key();
        let mut chars = key.chars();
        let ch = match (chars.next(), chars.next()) {
            (Some(c), None) => c,
            _ if key == "Enter" => '\r',
            _ => return,
        };
        let a = KeyPressEventArgs { ch, ..Default::default() };
        f(&a);
        if a.handled() {
            k.prevent_default();
        }
    });
    let _ = e.add_event_listener_with_callback("keydown", c.as_ref().unchecked_ref());
    c.forget();
}

/// `f` with the mouse (or a finger) on control `h` - `mousedown`, `mouseup`, `mousemove`.
pub fn on_mouse(h: HWND, event: &str, mut f: impl FnMut(&MouseEventArgs) + 'static) {
    let Some(e) = el(h) else { return };
    let c = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |m: web_sys::MouseEvent| {
        let button = match m.button() {
            0 => MouseButton::Left,
            1 => MouseButton::Middle,
            2 => MouseButton::Right,
            _ => MouseButton::None,
        };
        f(&MouseEventArgs { x: m.offset_x(), y: m.offset_y(), button, ctrl: m.ctrl_key(), shift: m.shift_key(), delta: 0 });
    });
    let _ = e.add_event_listener_with_callback(event, c.as_ref().unchecked_ref());
    c.forget();
}

/// A box that hears keys takes the focus - clicked, or by Tab - as on Windows; a box, a button or
/// a link has it already.
fn focusable(e: &web_sys::HtmlElement) {
    let own = matches!(e.tag_name().to_ascii_lowercase().as_str(), "input" | "textarea" | "select" | "button" | "a");
    if !own && !e.has_attribute("tabindex") {
        let _ = e.set_attribute("tabindex", "0");
    }
}

/// `f` with each turn of the mouse wheel over control `h` - MouseWheel: 120 a notch, more than 0
/// away from the person, as on Windows; the page does not scroll for it.
pub fn on_wheel(h: HWND, mut f: impl FnMut(&MouseEventArgs) + 'static) {
    let Some(e) = el(h) else { return };
    let c = Closure::<dyn FnMut(web_sys::WheelEvent)>::new(move |m: web_sys::WheelEvent| {
        m.prevent_default();
        f(&MouseEventArgs { x: m.offset_x(), y: m.offset_y(), button: MouseButton::None, ctrl: m.ctrl_key(), shift: m.shift_key(), delta: wheel_delta(m.delta_y(), m.delta_mode()) });
    });
    let _ = e.add_event_listener_with_callback("wheel", c.as_ref().unchecked_ref());
    c.forget();
}

/// A browser's wheel turn (`deltaY`, down more than 0, in pixels or lines) as Windows' delta.
pub fn wheel_delta(dy: f64, mode: u32) -> i32 {
    // A line is a third of a notch on Windows; pixels about 100 a notch.
    let notches = if mode == 0 { dy / 100.0 } else { dy / 3.0 };
    let d = -(notches * 120.0).round() as i32;
    if d == 0 && dy != 0.0 {
        if dy > 0.0 { -1 } else { 1 }
    } else {
        d
    }
}

/// `f` once the page has drawn the window - Shown.
pub fn after_shown(f: impl FnOnce() + 'static) {
    let c = Closure::once_into_js(f);
    let _ = win::window().set_timeout_with_callback_and_timeout_and_arguments_0(c.unchecked_ref(), 0);
}

/// A timer: `f` every `ms` milliseconds while it runs.
pub fn timer(ms: u32, on: bool, f: impl FnMut() + 'static) -> crate::ctl::Timer {
    crate::ctl::Timer::make(ms, on, Box::new(f))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: i32, y: i32, w: i32, h: i32, anchor: &'static str, dock: &'static str) -> Place {
        Place { x, y, w, h, anchor, dock, within: (480, 320) }
    }

    #[test]
    fn a_wheel_notch_is_windows_120() {
        assert_eq!(wheel_delta(100.0, 0), -120);
        assert_eq!(wheel_delta(-3.0, 1), 120);
        assert_eq!(wheel_delta(4.0, 0), -5);
    }

    #[test]
    fn anchors_become_css_distances() {
        assert_eq!(place_css(&p(11, 11, 300, 23, "", "none")), "left:11px;width:300px;top:11px;height:23px;");
        assert_eq!(place_css(&p(394, 260, 75, 23, "right bottom", "none")), "right:11px;width:75px;bottom:37px;height:23px;");
        assert_eq!(place_css(&p(11, 40, 458, 200, "all", "none")), "left:11px;right:11px;top:40px;bottom:80px;");
        assert_eq!(place_css(&p(0, 0, 0, 26, "", "bottom")), "left:0;right:0;bottom:0;height:26px;");
        assert_eq!(place_css(&p(0, 0, 0, 0, "", "fill")), "left:0;top:0;right:0;bottom:0;");
    }
}
