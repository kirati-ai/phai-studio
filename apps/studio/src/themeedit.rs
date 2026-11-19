//! The Theme Editor, as DevExpress' Skin Editor: a project's theme changed by choosing, not by
//! writing CSS. Two tabs - Colours, the `:root` variables every control takes its colours from,
//! and Controls, every kind of control in every state with every property - beside a preview of
//! every kind of control, which changes as a value does; the form being designed changes too.
//! Save writes `theme.css` - only what was changed, the rest of the file as it was - and Cancel
//! puts the theme back.

use std::path::{Path, PathBuf};

use phai_ui::controls;
use phai_ui::text::{t, tf};
use phai_ui::theme::{self, Color, Query, State};
use phai_ui::theme_edit;
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::HFONT;
use windows::Win32::UI::WindowsAndMessaging as wm;

/// The variables of the standard themes, and what each colours.
pub const VARS: [(&str, &str); 14] = [
    ("--accent", "Accent: what is chosen, the default button, ticks"),
    ("--accent-hover", "Accent under the pointer"),
    ("--accent-pressed", "Accent while pressed"),
    ("--accent-tint", "Light accent behind a hovered or chosen row"),
    ("--accent-tint-pressed", "Light accent while pressed"),
    ("--surface", "The window's background"),
    ("--text", "Words"),
    ("--muted", "Quiet words: group titles, the status bar"),
    ("--border", "Borders of buttons and boxes"),
    ("--rule", "Thin lines that divide"),
    ("--disabled-bg", "Background of what cannot be used"),
    ("--disabled-border", "Border of what cannot be used"),
    ("--disabled-text", "Words of what cannot be used"),
    ("--icon", "Toolbar icons"),
];

/// Every kind of control the theme draws, as its selector.
pub const KINDS: [&str; 17] = ["window", "label", "button", "checkbox", "radio", "edit", "combo", "list", "tree", "tabs", "group", "date", "progress", "slider", "statusbar", "toolbar", "menubar"];

/// The states a rule can be for; the first is none.
pub const STATES: [&str; 7] = ["", "hover", "pressed", "focus", "disabled", "selected", "checked"];

/// The properties of a control's rule.
pub const PROPS: [&str; 8] = ["background", "color", "border-color", "border-width", "border-radius", "font-weight", "accent-color", "icon-color"];

const ID_SAVE: i32 = 1;
const ID_CANCEL: i32 = 2;
const ID_TABS: i32 = 10;
const ID_BASE: i32 = 11;
const ID_KIND: i32 = 12;
const ID_STATE: i32 = 13;
const ID_VAR_SWATCH: i32 = 100;
const ID_VAR_EDIT: i32 = 200;
const ID_PROP_SWATCH: i32 = 300;
const ID_PROP_EDIT: i32 = 400;

struct Row {
    label: HWND,
    swatch: HWND,
    edit: HWND,
}

struct Editor {
    hwnd: HWND,
    path: PathBuf,
    /// The file as it was opened, for Cancel.
    original: String,
    /// The file as it is now.
    css: String,
    font: HFONT,
    bold: HFONT,
    tabs: HWND,
    base_label: HWND,
    base: HWND,
    vars: Vec<Row>,
    kind_label: HWND,
    kind: HWND,
    state_label: HWND,
    state: HWND,
    props: Vec<Row>,
    status: HWND,
    /// Filling the boxes: their changes are not the person's.
    filling: bool,
    saved: bool,
}

fn is_colour(p: &str) -> bool {
    !matches!(p, "border-width" | "border-radius" | "font-weight")
}

fn hex(c: Color) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
}

impl Editor {
    fn dir(&self) -> PathBuf {
        self.path.parent().map(Path::to_path_buf).unwrap_or_default()
    }

    /// The selector the Controls tab is on: `button`, `button:hover`.
    fn selector(&self) -> String {
        let kind = KINDS[win::combo_selected(self.kind).max(0) as usize % KINDS.len()];
        let state = STATES[win::combo_selected(self.state).max(0) as usize % STATES.len()];
        if state.is_empty() {
            kind.to_string()
        } else {
            format!("{kind}:{state}")
        }
    }

    /// The file's words changed: read as a theme, used everywhere, the boxes filled again.
    fn apply(&mut self, css: String) {
        match theme::parse_in(&css, &self.dir()) {
            Ok(th) => {
                self.css = css;
                theme::set(th);
                win::set_text(self.status, "");
                win::retheme(self.hwnd);
                if let Some(s) = crate::studio() {
                    s.after_theme_pub();
                }
                self.fill();
            }
            Err(e) => win::set_text(self.status, &e.message),
        }
    }

    /// Every box from the theme as it is now - the one being typed in left alone.
    fn fill(&mut self) {
        self.filling = true;
        let focus = unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetFocus() };
        let cur = theme::current();
        let own_vars = theme_edit::vars(&self.css);
        for (i, (name, _)) in VARS.iter().enumerate() {
            let r = &self.vars[i];
            let v = cur.var(name).unwrap_or_default();
            if r.edit != focus {
                win::set_text(r.edit, &v);
            }
            let set = own_vars.iter().any(|(k, _)| k == name);
            win::set_font(r.label, if set { self.bold } else { self.font });
            win::set_text(r.label, t(VARS[i].1));
            let _ = win::set_own_style(r.swatch, &format!("background: {v}"));
        }
        let sel = self.selector();
        let (kind, state) = sel.split_once(':').unwrap_or((&sel, ""));
        let st = State { hover: state == "hover", pressed: state == "pressed", focus: state == "focus", disabled: state == "disabled", selected: state == "selected", checked: state == "checked" };
        let look = cur.style(&Query { kind, classes: &[], id: None, state: st });
        for (i, p) in PROPS.iter().enumerate() {
            let r = &self.props[i];
            let v = match *p {
                "background" => look.background.map(hex),
                "color" => look.color.map(hex),
                "border-color" => look.border_color.map(hex),
                "accent-color" => look.accent_color.map(hex),
                "icon-color" => look.icon_color.map(hex),
                "border-width" => look.border_width.map(|n| n.to_string()),
                "border-radius" => look.border_radius.map(|n| n.to_string()),
                _ => look.font_weight.map(|n| n.to_string()),
            }
            .unwrap_or_default();
            if r.edit != focus {
                win::set_text(r.edit, &v);
            }
            let set = theme_edit::prop(&self.css, &sel, p).is_some();
            win::set_font(r.label, if set { self.bold } else { self.font });
            win::set_text(r.label, p);
            if is_colour(p) {
                let _ = win::set_own_style(r.swatch, &format!("background: {}", if v.is_empty() { "transparent".into() } else { v }));
            }
        }
        unsafe {
            wm::SendMessageW(self.base, 0x014E /* CB_SETCURSEL */, Some(WPARAM((theme_edit::base(&self.css) == Some("phai:dark")) as usize)), None);
        }
        self.filling = false;
    }

    /// The Colours tab or the Controls tab.
    fn show_page(&self, page: usize) {
        for r in &self.vars {
            for h in [r.label, r.swatch, r.edit] {
                win::show(h, page == 0);
            }
        }
        win::show(self.base_label, page == 0);
        win::show(self.base, page == 0);
        for (i, r) in self.props.iter().enumerate() {
            for h in [r.label, r.edit] {
                win::show(h, page == 1);
            }
            win::show(r.swatch, page == 1 && is_colour(PROPS[i]));
        }
        for h in [self.kind_label, self.kind, self.state_label, self.state] {
            win::show(h, page == 1);
        }
    }

    fn set_var(&mut self, i: usize, value: &str) {
        let v = value.trim();
        if theme::parse_color(v).is_none() {
            return;
        }
        let css = theme_edit::set_var(&self.css, VARS[i].0, v);
        self.apply(css);
    }

    fn set_prop(&mut self, i: usize, value: &str) {
        let p = PROPS[i];
        let v = value.trim();
        let sel = self.selector();
        let css = if v.is_empty() {
            theme_edit::remove_prop(&self.css, &sel, p)
        } else {
            let v = if matches!(p, "border-width" | "border-radius") && v.parse::<i32>().is_ok() { format!("{v}px") } else { v.to_string() };
            if theme::declarations_style(&format!("{p}: {v}")).is_err() {
                return;
            }
            theme_edit::set_prop(&self.css, &sel, p, &v)
        };
        self.apply(css);
    }

    fn command(&mut self, id: i32, code: u16) {
        if self.filling {
            return;
        }
        match (id, code) {
            (ID_SAVE, 0) => {
                match std::fs::write(&self.path, &self.css) {
                    Ok(()) => {
                        self.saved = true;
                        unsafe {
                            let _ = wm::DestroyWindow(self.hwnd);
                        }
                    }
                    Err(e) => win::set_text(self.status, &e.to_string()),
                }
            }
            (ID_CANCEL, 0) => unsafe {
                let _ = wm::DestroyWindow(self.hwnd);
            },
            (ID_TABS, _) => self.show_page(code as usize),
            (ID_BASE, 1) => {
                let to = if win::combo_selected(self.base) == 1 { "phai:dark" } else { "phai:light" };
                let css = theme_edit::set_base(&self.css, to);
                self.apply(css);
            }
            (ID_KIND, 1) | (ID_STATE, 1) => self.fill(),
            (i, 0) if (ID_VAR_SWATCH..ID_VAR_SWATCH + VARS.len() as i32).contains(&i) => {
                let k = (i - ID_VAR_SWATCH) as usize;
                if let Some(c) = choose_colour(self.hwnd, &win::text_of(self.vars[k].edit)) {
                    self.set_var(k, &c);
                }
            }
            (i, 0x0300) if (ID_VAR_EDIT..ID_VAR_EDIT + VARS.len() as i32).contains(&i) => {
                let k = (i - ID_VAR_EDIT) as usize;
                let text = win::text_of(self.vars[k].edit);
                self.set_var(k, &text);
            }
            (i, 0) if (ID_PROP_SWATCH..ID_PROP_SWATCH + PROPS.len() as i32).contains(&i) => {
                let k = (i - ID_PROP_SWATCH) as usize;
                if let Some(c) = choose_colour(self.hwnd, &win::text_of(self.props[k].edit)) {
                    self.set_prop(k, &c);
                }
            }
            // A property typed: taken when the box is left, as a half-typed number is no value.
            (i, 0x0200 /* EN_KILLFOCUS */) if (ID_PROP_EDIT..ID_PROP_EDIT + PROPS.len() as i32).contains(&i) => {
                let k = (i - ID_PROP_EDIT) as usize;
                let text = win::text_of(self.props[k].edit);
                let sel = self.selector();
                let now = theme_edit::prop(&self.css, &sel, PROPS[k]).unwrap_or_default();
                if text.trim() != now.trim_end_matches("px") && text.trim() != now {
                    self.set_prop(k, &text);
                }
            }
            _ => {}
        }
    }
}

/// Windows' colour dialog, starting at `start`; the colour chosen as `#rrggbb`.
pub fn choose_colour(owner: HWND, start: &str) -> Option<String> {
    use windows::Win32::UI::Controls::Dialogs::{ChooseColorW, CC_FULLOPEN, CC_RGBINIT, CHOOSECOLORW};
    static mut CUSTOM: [windows::Win32::Foundation::COLORREF; 16] = [windows::Win32::Foundation::COLORREF(0x00FF_FFFF); 16];
    let first = theme::parse_color(start).map(|c| c.colorref()).unwrap_or(0x00FF_FFFF);
    let mut cc = CHOOSECOLORW { lStructSize: std::mem::size_of::<CHOOSECOLORW>() as u32, hwndOwner: owner, rgbResult: windows::Win32::Foundation::COLORREF(first), lpCustColors: &raw mut CUSTOM as *mut _, Flags: CC_FULLOPEN | CC_RGBINIT, ..Default::default() };
    if !unsafe { ChooseColorW(&mut cc) }.as_bool() {
        return None;
    }
    let v = cc.rgbResult.0;
    Some(format!("#{:02x}{:02x}{:02x}", v & 0xFF, (v >> 8) & 0xFF, (v >> 16) & 0xFF))
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(ed) = (unsafe { win::state::<Editor>(h) }) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_COMMAND => {
            ed.command(win::loword(w.0) as i32, win::hiword(w.0));
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            unsafe {
                let _ = wm::DestroyWindow(h);
            }
            LRESULT(0)
        }
        wm::WM_DESTROY => {
            // Closed without saving: the theme it had.
            if !ed.saved {
                if let Ok(th) = theme::parse_in(&ed.original, &ed.dir()) {
                    theme::set(th);
                    if let Some(s) = crate::studio() {
                        s.after_theme_pub();
                    }
                }
            }
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}

/// The controls a theme is for, one of each kind, in a group on the right - each drawn as the
/// theme now says.
fn preview(parent: HWND, f: HFONT, x: i32, y: i32) {
    let px = |v| win::px(parent, v);
    let at = |h: HWND, dx: i32, dy: i32, w: i32, ht: i32| win::move_to(h, px(x + dx), px(y + dy), px(w), px(ht));
    let g = controls::group(parent, t("Preview"), 0, f);
    at(g, 0, 0, 460, 548);
    at(win::label(parent, t("A label"), 0, f), 16, 28, 200, 23);
    let muted = win::label(parent, t("Quiet words"), 0, f);
    win::set_css(muted, "muted", "");
    at(muted, 230, 28, 200, 23);
    at(win::button(parent, t("Button"), 0, f, false), 16, 60, 100, 28);
    at(win::button(parent, t("Default"), 0, f, true), 126, 60, 100, 28);
    let off = win::button(parent, t("Disabled"), 0, f, false);
    win::enable(off, false);
    at(off, 236, 60, 100, 28);
    let cb = win::checkbox(parent, t("Check box"), 0, f, true);
    at(cb, 16, 100, 140, 24);
    at(win::radio(parent, t("Option"), 0, f, 9, true), 166, 100, 120, 24);
    at(win::radio(parent, t("Other"), 0, f, 9, false), 296, 100, 120, 24);
    at(win::edit(parent, t("Words typed in"), 0, f, false), 16, 134, 200, 23);
    at(win::combo(parent, 0, f, &[t("Drop-down"), t("Other")]), 226, 134, 200, 23);
    let list = win::list(parent, 0, f, &[(t("Name"), 120), (t("Value"), 90)]);
    for (a, b) in [("Somchai", "081"), ("Monrak", "02"), ("Phai", "1")] {
        win::list_add(list, &[a, b]);
    }
    win::list_select(list, 1);
    at(list, 16, 168, 230, 110);
    let tree = win::tree(parent, 0, f);
    win::tree_items(tree, &["Thailand", "  Bangkok", "  Chiang Mai"]);
    phai_ui::props::expand_all(tree, true);
    at(tree, 256, 168, 170, 110);
    at(controls::tabs(parent, 0, f, &[t("General"), t("Advanced")]), 16, 290, 410, 30);
    at(win::date(parent, 0, f), 16, 332, 200, 23);
    at(win::number(parent, 0, f, 0, 100, 42), 226, 332, 100, 23);
    at(win::progress(parent, 0, 0, 100, 60), 16, 370, 410, 18);
    let sl = win::slider(parent, 0, 0, 100, 30);
    phai_ui::props::tick_frequency(sl, 10);
    at(sl, 16, 398, 410, 34);
    let status = controls::status_bar(parent, 0, f);
    controls::status_parts(status, &[120]);
    controls::status_text(status, 0, t("Ready"));
    at(status, 16, 500, 428, 26);
    // The group's frame behind what it gathers, or it paints over them.
    unsafe {
        let _ = wm::SetWindowPos(g, Some(wm::HWND_BOTTOM), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOACTIVATE);
    }
}

/// For checking: the open editor's Controls tab on `kind` and `state` (`""` normal), `prop` set to
/// `value` as typed, then Save when `save`.
pub fn test_set(editor: HWND, kind: &str, state: &str, prop: &str, value: &str, save: bool) {
    let Some(e) = (unsafe { win::state::<Editor>(editor) }) else { return };
    unsafe {
        wm::SendMessageW(e.kind, 0x014E, Some(WPARAM(KINDS.iter().position(|k| *k == kind).unwrap_or(0))), None);
        wm::SendMessageW(e.state, 0x014E, Some(WPARAM(STATES.iter().position(|s| *s == state).unwrap_or(0))), None);
    }
    if let Some(i) = PROPS.iter().position(|p| *p == prop) {
        e.set_prop(i, value);
    }
    if save {
        e.command(ID_SAVE, 0);
    }
}

/// Opens the editor on the theme file `path` and waits until it is closed.
pub fn open(owner: HWND, path: &Path) {
    let Ok(css) = std::fs::read_to_string(path) else { return };
    let class = win::register("PhaiThemeEditor", Some(proc_), None);
    let title = win::wide(&tf("Theme Editor - {0}", &[&path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()]));
    let (style, ex) = phai_ui::form::frame("fixed_dialog", false, false, true, false);
    let hwnd = unsafe { wm::CreateWindowExW(ex, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), style, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default() };
    let font = win::font(hwnd, 0, false);
    let bold = win::font(hwnd, 0, true);
    let px = |v| win::px(hwnd, v);
    let at = |h: HWND, x: i32, y: i32, w: i32, ht: i32| win::move_to(h, px(x), px(y), px(w), px(ht));
    let tabs = controls::tabs(hwnd, ID_TABS, font, &[t("Colours"), t("Controls")]);
    at(tabs, 11, 11, 430, 30);
    let base_label = win::label(hwnd, t("Starts from"), 0, font);
    at(base_label, 11, 52, 220, 23);
    let base = win::combo(hwnd, ID_BASE, font, &[t("Light"), t("Dark")]);
    at(base, 281, 52, 160, 23);
    let mut vars = Vec::new();
    for (i, _) in VARS.iter().enumerate() {
        let y = 86 + i as i32 * 32;
        let label = win::label(hwnd, "", 0, font);
        at(label, 11, y, 265, 23);
        let swatch = win::button(hwnd, "", ID_VAR_SWATCH + i as i32, font, false);
        at(swatch, 281, y, 36, 25);
        let edit = win::edit(hwnd, "", ID_VAR_EDIT + i as i32, font, false);
        at(edit, 323, y, 118, 25);
        vars.push(Row { label, swatch, edit });
    }
    let kind_label = win::label(hwnd, t("Control"), 0, font);
    at(kind_label, 11, 52, 120, 23);
    let kind = win::combo(hwnd, ID_KIND, font, &KINDS);
    phai_ui::props::max_drop_down_items(kind, 17);
    at(kind, 141, 52, 300, 23);
    let state_label = win::label(hwnd, t("State"), 0, font);
    at(state_label, 11, 86, 120, 23);
    let names: Vec<&str> = STATES.iter().map(|s| if s.is_empty() { t("normal") } else { s }).collect();
    let state = win::combo(hwnd, ID_STATE, font, &names);
    at(state, 141, 86, 300, 23);
    let mut props = Vec::new();
    for (i, _) in PROPS.iter().enumerate() {
        let y = 128 + i as i32 * 32;
        let label = win::label(hwnd, "", 0, font);
        at(label, 11, y, 180, 23);
        let swatch = win::button(hwnd, "", ID_PROP_SWATCH + i as i32, font, false);
        at(swatch, 281, y, 36, 25);
        let edit = win::edit(hwnd, "", ID_PROP_EDIT + i as i32, font, false);
        at(edit, 323, y, 118, 25);
        props.push(Row { label, swatch, edit });
    }
    let hint = win::label(hwnd, t("Bold: set in this file. Empty: the theme it starts from."), 0, font);
    win::set_css(hint, "muted", "");
    at(hint, 11, 548, 430, 23);
    preview(hwnd, font, 460, 11);
    let status = win::label(hwnd, "", 0, font);
    at(status, 11, 590, 600, 23);
    let save = win::button(hwnd, t("Save"), ID_SAVE, font, true);
    at(save, 740, 586, 90, 28);
    let cancel = win::button(hwnd, t("Cancel"), ID_CANCEL, font, false);
    at(cancel, 840, 586, 90, 28);
    let mut ed = Box::new(Editor { hwnd, path: path.to_path_buf(), original: css.clone(), css, font, bold, tabs, base_label, base, vars, kind_label, kind, state_label, state, props, status, filling: false, saved: false });
    unsafe {
        wm::SendMessageW(kind, 0x014E, Some(WPARAM(2)), None);
        wm::SendMessageW(state, 0x014E, Some(WPARAM(0)), None);
    }
    ed.show_page(0);
    ed.fill();
    let _ = ed.tabs;
    win::set_state(hwnd, ed);
    win::theme_frame(hwnd);
    unsafe {
        let style = wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32;
        let exs = wm::GetWindowLongW(hwnd, wm::GWL_EXSTYLE) as u32;
        let mut r = windows::Win32::Foundation::RECT { left: 0, top: 0, right: px(942), bottom: px(626) };
        let _ = windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(style), false, wm::WINDOW_EX_STYLE(exs), win::dpi(hwnd) as u32);
        let _ = wm::SetWindowPos(hwnd, None, 0, 0, r.right - r.left, r.bottom - r.top, wm::SWP_NOMOVE | wm::SWP_NOZORDER);
    }
    win::center_on(hwnd, owner);
    if std::env::var_os("PHAI_SHOT").is_some() {
        // A picture of the editor, for checking: the page `PHAI_THEME_PAGE` names.
        if let Ok(p) = std::env::var("PHAI_THEME_PAGE") {
            let page = p.parse::<usize>().unwrap_or(0);
            controls::tabs_select(tabs, page);
            if let Some(e) = unsafe { win::state::<Editor>(hwnd) } {
                e.show_page(page);
            }
        }
        win::show_top(hwnd);
        return;
    }
    win::show_top(hwnd);
    win::modal_loop(hwnd, owner);
}
