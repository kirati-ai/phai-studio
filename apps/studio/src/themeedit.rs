//! The Theme Editor, as DevExpress' Skin Editor: a project's theme changed by choosing, not by
//! writing CSS.
//!
//! - At the top, a gallery of ready themes - one click makes the whole palette, light or dark.
//! - On the left, every element: the Palette (the `:root` colours every control takes), then
//!   each kind of control by group, each with its states (hover, pressed, focus...), and a box to
//!   search them.
//! - In the middle, the chosen element's properties: a colour opens the colour tray; bold is set
//!   in this file, × takes it out (the theme it starts from shows again).
//! - On the right, every kind of control drawn as the theme now says; the form being designed
//!   changes too. Undo and Redo step through the changes; Save writes `theme.css` - only what
//!   was changed, the rest of the file as it was - and Cancel puts the theme back.

use std::path::{Path, PathBuf};

use phai_ui::controls;
use phai_ui::text::{t, tf};
use phai_ui::theme::{self, Color, Query, State};
use phai_ui::theme_edit;
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HFONT};
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::colortray;

/// The variables of the standard themes, and what each colours.
pub const VARS: [(&str, &str); 15] = [
    ("--accent", "Accent: what is chosen, the default button, ticks"),
    ("--accent-hover", "Accent under the pointer"),
    ("--accent-pressed", "Accent while pressed"),
    ("--accent-tint", "Light accent behind a hovered or chosen row"),
    ("--accent-tint-pressed", "Light accent while pressed"),
    ("--surface", "The window's background"),
    ("--raised", "Boxes and buttons: what is typed in or pressed"),
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
pub const PROPS: [&str; 10] = ["background", "background-image", "box-shadow", "color", "border-color", "border-width", "border-radius", "font-weight", "accent-color", "icon-color"];

/// Each palette colour's short name; what it colours is its tooltip.
fn var_name(i: usize) -> &'static str {
    [t("Accent"), t("Accent, pointed at"), t("Accent, pressed"), t("Tint"), t("Tint, pressed"), t("Background"), t("Boxes and buttons"), t("Words"), t("Quiet words"), t("Borders"), t("Dividing lines"), t("Disabled background"), t("Disabled border"), t("Disabled words"), t("Icons")][i]
}

/// What each kind is called in the tree.
fn kind_name(k: usize) -> &'static str {
    [t("Window"), t("Label"), t("Button"), t("Check box"), t("Option"), t("Text box"), t("Drop-down"), t("List"), t("Tree view"), t("Tabs"), t("Group"), t("Date"), t("Progress bar"), t("Slider"), t("Status bar"), t("Toolbar"), t("Menu bar")][k]
}

fn state_name(s: usize) -> &'static str {
    [t("Normal"), t("Under the pointer"), t("Pressed"), t("Focused"), t("Disabled"), t("Chosen"), t("Ticked")][s]
}

fn prop_name(p: usize) -> &'static str {
    [t("Background"), t("Gradient"), t("Shadow"), t("Words"), t("Border colour"), t("Border width"), t("Corner radius"), t("Boldness"), t("Accent (ticks, bars)"), t("Icon colour")][p]
}

/// The tree's groups: their names and kinds, by place in `KINDS`.
fn groups() -> [(&'static str, &'static [usize]); 4] {
    [(t("Common"), &[0, 1, 2, 10]), (t("Editors"), &[5, 6, 11, 3, 4, 13]), (t("Lists"), &[7, 8, 9]), (t("Bars"), &[17 - 1, 15, 14, 12])]
}

/// A ready theme: its name, accent, light or dark, and its own background if it has one.
pub struct Preset {
    pub name: &'static str,
    pub accent: u32,
    pub dark: bool,
    pub surface: Option<u32>,
}

pub const PRESETS: [Preset; 13] = [
    Preset { name: "Bamboo", accent: 0x12855f, dark: false, surface: None },
    Preset { name: "Ocean", accent: 0x1c6fd1, dark: false, surface: None },
    Preset { name: "Royal", accent: 0x6b3fc8, dark: false, surface: None },
    Preset { name: "Sunset", accent: 0xc8561a, dark: false, surface: None },
    Preset { name: "Rose", accent: 0xc2185b, dark: false, surface: None },
    Preset { name: "Teal", accent: 0x00838f, dark: false, surface: None },
    Preset { name: "Graphite", accent: 0x4a5568, dark: false, surface: None },
    Preset { name: "Paper", accent: 0x8a5a2b, dark: false, surface: Some(0xfbf8f2) },
    Preset { name: "Night", accent: 0x2fb487, dark: true, surface: None },
    Preset { name: "Midnight", accent: 0x4f9cf5, dark: true, surface: Some(0x1b2230) },
    Preset { name: "Violet", accent: 0xa78bfa, dark: true, surface: None },
    Preset { name: "Ember", accent: 0xf08a3c, dark: true, surface: None },
    Preset { name: "Slate", accent: 0x94a3b8, dark: true, surface: Some(0x1e2329) },
];

fn rgb(v: u32) -> Color {
    Color::rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

fn hex(c: Color) -> String {
    colortray::hex(c)
}

/// Every palette colour of a ready theme: the accent's family made from its accent, the rest
/// from the standard theme it starts from (its own background, and colours that go with it,
/// when it has one).
pub fn preset_vars(p: &Preset) -> Vec<(&'static str, String)> {
    let base = if p.dark { theme::Theme::dark() } else { theme::Theme::light() };
    let accent = rgb(p.accent);
    let (white, black) = (Color::rgb(255, 255, 255), Color::rgb(0, 0, 0));
    let surface = p.surface.map(rgb).or_else(|| base.var_color("--surface")).unwrap_or(if p.dark { Color::rgb(0x20, 0x20, 0x20) } else { white });
    VARS.iter()
        .map(|(name, _)| {
            let c = match *name {
                "--accent" => Some(accent),
                "--accent-hover" => Some(if p.dark { accent.mix(white, 0.14) } else { accent.mix(black, 0.12) }),
                "--accent-pressed" => Some(if p.dark { accent.mix(black, 0.16) } else { accent.mix(black, 0.25) }),
                "--accent-tint" => Some(accent.mix(surface, if p.dark { 0.8 } else { 0.9 })),
                "--accent-tint-pressed" => Some(accent.mix(surface, if p.dark { 0.72 } else { 0.8 })),
                "--surface" => Some(surface),
                // Boxes a shade off the background: white on light, lifted on dark.
                "--raised" if p.surface.is_some() || p.dark => Some(if p.dark { surface.mix(white, 0.07) } else { white }),
                // A background of its own: lines and quiet colours leaning to it.
                "--border" | "--rule" | "--disabled-bg" | "--disabled-border" if p.surface.is_some() => {
                    let fallback = if p.dark { white } else { black };
                    let t = match *name {
                        "--border" => 0.2,
                        "--rule" => 0.12,
                        "--disabled-bg" => 0.04,
                        _ => 0.1,
                    };
                    Some(surface.mix(fallback, t))
                }
                _ => base.var_color(name),
            };
            (*name, c.map(hex).unwrap_or_default())
        })
        .collect()
}

const ID_SAVE: i32 = 1;
const ID_CANCEL: i32 = 2;
const ID_UNDO: i32 = 3;
const ID_REDO: i32 = 4;
const ID_TREE: i32 = 10;
const ID_SEARCH: i32 = 11;
const ID_GALLERY: i32 = 12;
const ID_VAR_SWATCH: i32 = 100;
const ID_VAR_EDIT: i32 = 200;
const ID_PROP_SWATCH: i32 = 300;
const ID_PROP_EDIT: i32 = 400;
const ID_VAR_RESET: i32 = 500;
const ID_PROP_RESET: i32 = 600;

/// What a tree item stands for: the Palette, a group (nothing), or a kind in a state.
const PALETTE: isize = 0;
const GROUP: isize = -1;

fn element(kind: usize, state: usize) -> isize {
    1000 + kind as isize * 10 + state as isize
}

struct Row {
    label: HWND,
    swatch: HWND,
    edit: HWND,
    reset: HWND,
}

struct Editor {
    hwnd: HWND,
    path: PathBuf,
    /// The file as it was opened, for Cancel.
    original: String,
    /// The file as it is now.
    css: String,
    undo: Vec<String>,
    redo: Vec<String>,
    font: HFONT,
    bold: HFONT,
    tree: HWND,
    search: HWND,
    gallery: HWND,
    heading: HWND,
    selector_label: HWND,
    vars: Vec<Row>,
    props: Vec<Row>,
    status: HWND,
    undo_button: HWND,
    redo_button: HWND,
    /// What the layout moves as the window is sized: the heading over the gallery, the hint, the
    /// preview's frame, Save and Cancel.
    themes_label: HWND,
    hint: HWND,
    preview_frame: HWND,
    save: HWND,
    cancel: HWND,
    /// The element chosen: the Palette, or (kind, state).
    on: Option<(usize, usize)>,
    /// Filling the boxes: their changes are not the person's.
    filling: bool,
    saved: bool,
}

fn is_colour(p: &str) -> bool {
    !matches!(p, "border-width" | "border-radius" | "font-weight" | "background-image" | "box-shadow")
}

impl Editor {
    fn dir(&self) -> PathBuf {
        self.path.parent().map(Path::to_path_buf).unwrap_or_default()
    }

    /// The selector of the element chosen: `button`, `button:hover`.
    fn selector(&self) -> String {
        let (k, s) = self.on.unwrap_or((2, 0));
        if s == 0 {
            KINDS[k].to_string()
        } else {
            format!("{}:{}", KINDS[k], STATES[s])
        }
    }

    /// A change by the person: the file before it kept for Undo.
    fn change(&mut self, css: String) {
        if css == self.css {
            return;
        }
        let before = self.css.clone();
        if self.apply(css) {
            self.undo.push(before);
            self.redo.clear();
            self.say_steps();
        }
    }

    fn say_steps(&self) {
        win::enable(self.undo_button, !self.undo.is_empty());
        win::enable(self.redo_button, !self.redo.is_empty());
    }

    /// The file's words changed: read as a theme, used everywhere, the boxes filled again.
    fn apply(&mut self, css: String) -> bool {
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
                unsafe {
                    let _ = Gdi::InvalidateRect(Some(self.gallery), None, false);
                }
                true
            }
            Err(e) => {
                win::set_text(self.status, &e.message);
                false
            }
        }
    }

    /// The theme's colours now, for the colour tray.
    fn theme_colours(&self) -> Vec<(String, Color)> {
        let cur = theme::current();
        VARS.iter().filter_map(|(n, _)| cur.var_color(n).map(|c| (n.to_string(), c))).collect()
    }

    /// Every box from the theme as it is now - the one being typed in left alone.
    fn fill(&mut self) {
        self.filling = true;
        let focus = unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetFocus() };
        let cur = theme::current();
        let own_vars = theme_edit::vars(&self.css);
        for (i, (name, what)) in VARS.iter().enumerate() {
            let r = &self.vars[i];
            let v = cur.var(name).unwrap_or_default();
            if r.edit != focus {
                win::set_text(r.edit, &v);
            }
            let set = own_vars.iter().any(|(k, _)| k == name);
            win::set_font(r.label, if set { self.bold } else { self.font });
            let _ = what;
            win::set_text(r.label, var_name(i));
            win::enable(r.reset, set);
            let _ = win::set_own_style(r.swatch, &format!("background: {v}"));
        }
        match self.on {
            None => {
                win::set_text(self.heading, t("Palette"));
                win::set_text(self.selector_label, t("The colours every control takes - :root"));
            }
            Some((k, s)) => {
                win::set_text(self.heading, &format!("{}  ›  {}", kind_name(k), state_name(s)));
                let sel = self.selector();
                win::set_text(self.selector_label, &sel);
                let st = State { hover: s == 1, pressed: s == 2, focus: s == 3, disabled: s == 4, selected: s == 5, checked: s == 6 };
                let look = cur.style(&Query { kind: KINDS[k], classes: &[], id: None, state: st });
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
                        "background-image" => look.gradient.as_ref().map(|g| g.css()),
                        "box-shadow" => look.shadow.flatten().map(|s| s.css()),
                        _ => look.font_weight.map(|n| n.to_string()),
                    }
                    .unwrap_or_default();
                    if r.edit != focus {
                        win::set_text(r.edit, &v);
                    }
                    let set = theme_edit::prop(&self.css, &sel, p).is_some();
                    win::set_font(r.label, if set { self.bold } else { self.font });
                    win::set_text(r.label, prop_name(i));
                    win::enable(r.reset, set);
                    if is_colour(p) {
                        let _ = win::set_own_style(r.swatch, &format!("background: {}", if v.is_empty() { "transparent".into() } else { v }));
                    }
                }
            }
        }
        self.filling = false;
    }

    /// The Palette's rows, or the chosen control's.
    fn show_rows(&self) {
        let palette = self.on.is_none();
        for r in &self.vars {
            for h in [r.label, r.swatch, r.edit, r.reset] {
                win::show(h, palette);
            }
        }
        for (i, r) in self.props.iter().enumerate() {
            for h in [r.label, r.edit, r.reset] {
                win::show(h, !palette);
            }
            win::show(r.swatch, !palette && is_colour(PROPS[i]));
        }
    }

    fn choose(&mut self, data: isize) {
        self.on = match data {
            PALETTE => None,
            GROUP => return,
            d => Some((((d - 1000) / 10) as usize % KINDS.len(), ((d - 1000) % 10) as usize % STATES.len())),
        };
        self.show_rows();
        self.fill();
    }

    /// The tree again, only what has the words searched for.
    fn fill_tree(&mut self) {
        let q = win::text_of(self.search).trim().to_lowercase();
        let has = |s: &str| q.is_empty() || s.to_lowercase().contains(&q);
        win::tree_clear(self.tree);
        if has(t("Palette")) || has("palette") || has("colour") || has("root") {
            win::tree_add(self.tree, 0, t("Palette"), -1, PALETTE);
        }
        for (group, kinds) in groups() {
            let shown: Vec<usize> = kinds.iter().copied().filter(|k| has(kind_name(*k)) || has(KINDS[*k]) || has(group)).collect();
            if shown.is_empty() {
                continue;
            }
            let g = win::tree_add(self.tree, 0, group, -1, GROUP);
            for k in shown {
                let item = win::tree_add(self.tree, g, kind_name(k), -1, element(k, 0));
                for s in 1..STATES.len() {
                    win::tree_add(self.tree, item, state_name(s), -1, element(k, s));
                }
            }
            win::tree_expand(self.tree, g);
        }
    }

    fn set_var(&mut self, i: usize, value: &str) {
        let v = value.trim();
        if theme::parse_color(v).is_none() {
            return;
        }
        let css = theme_edit::set_var(&self.css, VARS[i].0, v);
        self.change(css);
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
        self.change(css);
    }

    /// A ready theme: its base and every palette colour.
    fn use_preset(&mut self, k: usize) {
        let p = &PRESETS[k];
        let mut css = theme_edit::set_base(&self.css, if p.dark { "phai:dark" } else { "phai:light" });
        // A copy of the standard light theme from before `--raised`: its white boxes take the
        // palette's, or a dark theme would keep them white.
        for sel in ["button", "checkbox, radio", "edit", "combo", "date", "calendar"] {
            if theme_edit::prop(&css, sel, "background").is_some_and(|v| v.eq_ignore_ascii_case("#ffffff")) {
                css = theme_edit::set_prop(&css, sel, "background", "var(--raised)");
            }
        }
        for (name, v) in preset_vars(p) {
            if !v.is_empty() {
                css = theme_edit::set_var(&css, name, &v);
            }
        }
        self.change(css);
        win::set_text(self.status, &tf("Theme \"{0}\" - every colour can still be changed on the left", &[&t(p.name)]));
    }

    /// The colour tray under a swatch, starting at what its box says.
    fn tray(&self, swatch: HWND, edit: HWND) -> Option<String> {
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetWindowRect(swatch, &mut r);
        }
        colortray::choose(self.hwnd, r, &win::text_of(edit), &self.theme_colours())
    }

    fn command(&mut self, id: i32, code: u16) {
        if self.filling {
            return;
        }
        match (id, code) {
            (ID_SAVE, 0) => match std::fs::write(&self.path, &self.css) {
                Ok(()) => {
                    self.saved = true;
                    unsafe {
                        let _ = wm::DestroyWindow(self.hwnd);
                    }
                }
                Err(e) => win::set_text(self.status, &e.to_string()),
            },
            (ID_CANCEL, 0) => unsafe {
                let _ = wm::DestroyWindow(self.hwnd);
            },
            (ID_UNDO, 0) => {
                if let Some(before) = self.undo.pop() {
                    self.redo.push(self.css.clone());
                    self.apply(before);
                    self.say_steps();
                }
            }
            (ID_REDO, 0) => {
                if let Some(after) = self.redo.pop() {
                    self.undo.push(self.css.clone());
                    self.apply(after);
                    self.say_steps();
                }
            }
            (ID_SEARCH, 0x0300) => self.fill_tree(),
            (i, 0) if (ID_VAR_SWATCH..ID_VAR_SWATCH + VARS.len() as i32).contains(&i) => {
                let k = (i - ID_VAR_SWATCH) as usize;
                if let Some(c) = self.tray(self.vars[k].swatch, self.vars[k].edit) {
                    self.set_var(k, &c);
                }
            }
            (i, 0) if (ID_VAR_RESET..ID_VAR_RESET + VARS.len() as i32).contains(&i) => {
                let k = (i - ID_VAR_RESET) as usize;
                let css = theme_edit::remove_prop(&self.css, ":root", VARS[k].0);
                self.change(css);
            }
            // A colour typed: taken as soon as it reads as one.
            (i, 0x0300) if (ID_VAR_EDIT..ID_VAR_EDIT + VARS.len() as i32).contains(&i) => {
                let k = (i - ID_VAR_EDIT) as usize;
                let text = win::text_of(self.vars[k].edit);
                if theme::parse_color(text.trim()).is_some() && cur_var(VARS[k].0).as_deref() != Some(text.trim()) {
                    self.set_var(k, &text);
                }
            }
            (i, 0) if (ID_PROP_SWATCH..ID_PROP_SWATCH + PROPS.len() as i32).contains(&i) => {
                let k = (i - ID_PROP_SWATCH) as usize;
                if let Some(c) = self.tray(self.props[k].swatch, self.props[k].edit) {
                    self.set_prop(k, &c);
                }
            }
            (i, 0) if (ID_PROP_RESET..ID_PROP_RESET + PROPS.len() as i32).contains(&i) => {
                self.set_prop((i - ID_PROP_RESET) as usize, "");
            }
            // A property typed: taken when the box is left, as a half-typed number is no value.
            (i, 0x0200 /* EN_KILLFOCUS */) if (ID_PROP_EDIT..ID_PROP_EDIT + PROPS.len() as i32).contains(&i) => {
                let k = (i - ID_PROP_EDIT) as usize;
                let text = win::text_of(self.props[k].edit);
                let sel = self.selector();
                let now = theme_edit::prop(&self.css, &sel, PROPS[k]).unwrap_or_default();
                if !text.trim().is_empty() && text.trim() != now.trim_end_matches("px") && text.trim() != now {
                    self.set_prop(k, &text);
                }
            }
            _ => {}
        }
    }
}

fn cur_var(name: &str) -> Option<String> {
    theme::current().var(name)
}

/// Everything placed for the window's size: the gallery across the top, the tree down the left,
/// the properties in the middle, the preview's frame filling the rest, the buttons along the foot.
fn layout(h: HWND) {
    let Some(e) = (unsafe { win::state::<Editor>(h) }) else { return };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let px = |v| win::px(h, v);
    let (cw, ch) = (r.right, r.bottom);
    win::move_to(e.themes_label, px(10), px(8), cw - px(20), px(22));
    win::move_to(e.gallery, px(10), px(32), cw - px(20), px(TILE_H));
    let foot = ch - px(38);
    win::move_to(e.search, px(10), px(100), px(230), px(26));
    win::move_to(e.tree, px(10), px(132), px(230), foot - px(10) - px(132));
    win::move_to(e.hint, px(254), foot - px(36), px(380), px(22));
    // The preview's controls keep their places; its frame takes the room to the right and down.
    win::move_to(e.preview_frame, px(650), px(96), (cw - px(10) - px(650)).max(px(460)), (foot - px(10) - px(96)).max(px(548)));
    win::move_to(e.undo_button, px(10), foot, px(84), px(28));
    win::move_to(e.redo_button, px(100), foot, px(84), px(28));
    win::move_to(e.status, px(196), foot + px(4), cw - px(196) - px(220), px(22));
    win::move_to(e.save, cw - px(200), foot, px(90), px(28));
    win::move_to(e.cancel, cw - px(100), foot, px(90), px(28));
    unsafe {
        let _ = Gdi::InvalidateRect(Some(e.gallery), None, false);
    }
}

/// The gallery's tiles: as wide as the window lets them be, within reason - device pixels.
fn tile_width(gallery: HWND) -> i32 {
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(gallery, &mut r);
    }
    let n = PRESETS.len() as i32;
    ((r.right - (n - 1) * win::px(gallery, TILE_GAP)) / n).clamp(win::px(gallery, 70), win::px(gallery, 150))
}

/// Windows' colour dialog, starting at `start`; the colour chosen as `#rrggbb` - the colour
/// tray's "More colours".
pub fn choose_colour(owner: HWND, start: &str) -> Option<String> {
    use windows::Win32::UI::Controls::Dialogs::{ChooseColorW, CC_FULLOPEN, CC_RGBINIT, CHOOSECOLORW};
    static mut CUSTOM: [COLORREF; 16] = [COLORREF(0x00FF_FFFF); 16];
    let first = theme::parse_color(start).map(|c| c.colorref()).unwrap_or(0x00FF_FFFF);
    let mut cc = CHOOSECOLORW { lStructSize: std::mem::size_of::<CHOOSECOLORW>() as u32, hwndOwner: owner, rgbResult: COLORREF(first), lpCustColors: &raw mut CUSTOM as *mut _, Flags: CC_FULLOPEN | CC_RGBINIT, ..Default::default() };
    if !unsafe { ChooseColorW(&mut cc) }.as_bool() {
        return None;
    }
    let v = cc.rgbResult.0;
    Some(format!("#{:02x}{:02x}{:02x}", v & 0xFF, (v >> 8) & 0xFF, (v >> 16) & 0xFF))
}

// --- the gallery of ready themes ----------------------------------------------------------------

const TILE_H: i32 = 54;
const TILE_GAP: i32 = 6;

fn gallery_paint(h: HWND) {
    let cur = theme::current();
    let accent_now = cur.var_color("--accent");
    let dark_now = cur.is_dark();
    unsafe {
        let mut ps = Gdi::PAINTSTRUCT::default();
        let dc = Gdi::BeginPaint(h, &mut ps);
        let mut r = RECT::default();
        let _ = wm::GetClientRect(h, &mut r);
        win::fill(dc, &r, cur.var_color("--surface").unwrap_or(Color::rgb(255, 255, 255)));
        let font = win::font(h, 0, false);
        let old = Gdi::SelectObject(dc, font.into());
        Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        let px = |v| win::px(h, v);
        for (k, p) in PRESETS.iter().enumerate() {
            let vars = preset_vars(p);
            let get = |n: &str| vars.iter().find(|(k, _)| *k == n).and_then(|(_, v)| theme::parse_color(v)).unwrap_or(Color::rgb(128, 128, 128));
            let tw = tile_width(h);
            let x = k as i32 * (tw + px(TILE_GAP));
            let tile = RECT { left: x, top: 0, right: x + tw, bottom: px(TILE_H) };
            let on = accent_now == Some(rgb(p.accent)) && dark_now == p.dark;
            // Its frame: the accent when it is the theme now.
            win::fill(dc, &tile, if on { get("--accent") } else { get("--border") });
            let inner = RECT { left: tile.left + px(if on { 2 } else { 1 }), top: tile.top + px(if on { 2 } else { 1 }), right: tile.right - px(if on { 2 } else { 1 }), bottom: tile.bottom - px(if on { 2 } else { 1 }) };
            win::fill(dc, &inner, get("--surface"));
            // A bar of its accent, a row of its tint, and its name in its words.
            win::fill(dc, &RECT { bottom: inner.top + px(10), ..inner }, get("--accent"));
            let dot = |i: i32, c: Color| win::fill(dc, &RECT { left: inner.left + px(6 + i * 14), top: inner.top + px(16), right: inner.left + px(16 + i * 14), bottom: inner.top + px(26) }, c);
            dot(0, get("--accent"));
            dot(1, get("--accent-tint"));
            dot(2, get("--text"));
            Gdi::SetTextColor(dc, COLORREF(get("--text").colorref()));
            let mut tr = RECT { left: inner.left + px(6), top: inner.top + px(28), right: inner.right - px(2), bottom: inner.bottom };
            let mut w: Vec<u16> = t(p.name).encode_utf16().collect();
            Gdi::DrawTextW(dc, &mut w, &mut tr, Gdi::DT_LEFT | Gdi::DT_SINGLELINE | Gdi::DT_VCENTER | Gdi::DT_END_ELLIPSIS);
        }
        Gdi::SelectObject(dc, old);
        let _ = Gdi::EndPaint(h, &ps);
    }
}

extern "system" fn gallery_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_PAINT => {
            gallery_paint(h);
            LRESULT(0)
        }
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_LBUTTONDOWN => {
            let (x, _) = win::point_of(l);
            let step = tile_width(h) + win::px(h, TILE_GAP);
            let k = (x / step) as usize;
            if k < PRESETS.len() && x % step < tile_width(h) {
                if let Some(ed) = unsafe { wm::GetParent(h) }.ok().and_then(|p| unsafe { win::state::<Editor>(p) }) {
                    ed.use_preset(k);
                }
            }
            LRESULT(0)
        }
        wm::WM_SETCURSOR => {
            unsafe {
                let _ = wm::SetCursor(wm::LoadCursorW(None, wm::IDC_HAND).ok());
            }
            LRESULT(1)
        }
        _ => win::def(h, msg, w, l),
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(ed) = (unsafe { win::state::<Editor>(h) }) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            layout(h);
            LRESULT(0)
        }
        wm::WM_GETMINMAXINFO => {
            let mm = unsafe { &mut *(l.0 as *mut wm::MINMAXINFO) };
            mm.ptMinTrackSize = windows::Win32::Foundation::POINT { x: win::px(h, 1120), y: win::px(h, 700) };
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            ed.command(win::loword(w.0) as i32, win::hiword(w.0));
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            const TVN_SELCHANGEDW: u32 = (-451i32) as u32;
            if hdr.hwndFrom == ed.tree {
                if hdr.code == cc::NM_CUSTOMDRAW {
                    return win::tree_custom_draw(l);
                }
                if hdr.code == TVN_SELCHANGEDW {
                    if let Some(d) = win::tree_selected(ed.tree) {
                        ed.choose(d);
                    }
                }
            }
            win::def(h, msg, w, l)
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
fn preview(parent: HWND, f: HFONT, x: i32, y: i32) -> HWND {
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
    g
}

/// For checking: the open editor on `kind` and `state` (`""` normal), `prop` set to `value` as
/// typed, then Save when `save`.
pub fn test_set(editor: HWND, kind: &str, state: &str, prop: &str, value: &str, save: bool) {
    let Some(e) = (unsafe { win::state::<Editor>(editor) }) else { return };
    let k = KINDS.iter().position(|x| *x == kind).unwrap_or(2);
    let s = STATES.iter().position(|x| *x == state).unwrap_or(0);
    e.choose(element(k, s));
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
    let gallery_class = win::register("PhaiThemeGallery", Some(gallery_proc), None);
    let title = win::wide(&tf("Theme Editor - {0}", &[&path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()]));
    // Sizable, opened filling the screen.
    let (style, ex) = phai_ui::form::frame("sizable", true, true, true, false);
    let hwnd = unsafe { wm::CreateWindowExW(ex, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), style, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default() };
    let font = win::font(hwnd, 0, false);
    let bold = win::font(hwnd, 0, true);
    let px = |v| win::px(hwnd, v);
    let at = |h: HWND, x: i32, y: i32, w: i32, ht: i32| win::move_to(h, px(x), px(y), px(w), px(ht));
    // The ready themes along the top.
    let themes_label = win::label(hwnd, t("Ready themes - click one to start from it"), 0, bold);
    at(themes_label, 10, 8, 600, 22);
    let gallery = unsafe { wm::CreateWindowExW(Default::default(), PCWSTR(gallery_class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE, 0, 0, 10, 10, Some(hwnd), Some(wm::HMENU(ID_GALLERY as isize as *mut _)), Some(win::instance()), None).unwrap_or_default() };
    at(gallery, 10, 32, 1100, TILE_H);
    // The elements on the left.
    let search = win::edit(hwnd, "", ID_SEARCH, font, false);
    phai_ui::props::placeholder(search, t("Search elements"));
    at(search, 10, 100, 230, 26);
    let tree = win::tree(hwnd, ID_TREE, font);
    at(tree, 10, 132, 230, 498);
    // The chosen element's properties in the middle.
    let heading = win::label(hwnd, "", 0, bold);
    at(heading, 254, 100, 380, 24);
    let selector_label = win::label(hwnd, "", 0, font);
    win::set_css(selector_label, "muted", "");
    at(selector_label, 254, 124, 380, 20);
    let row = |label_x: i32, y: i32, swatch_id: i32, edit_id: i32, reset_id: i32| {
        let label = win::label(hwnd, "", 0, font);
        at(label, label_x, y, 170, 26);
        let swatch = win::button(hwnd, "", swatch_id, font, false);
        at(swatch, 428, y, 34, 26);
        let edit = win::edit(hwnd, "", edit_id, font, false);
        at(edit, 466, y, 136, 26);
        let reset = win::button(hwnd, "×", reset_id, font, false);
        phai_ui::props::tooltip(reset, t("Take it out of this file: the theme it starts from shows again"));
        at(reset, 606, y, 28, 26);
        Row { label, swatch, edit, reset }
    };
    let vars: Vec<Row> = (0..VARS.len()).map(|i| row(254, 152 + i as i32 * 32, ID_VAR_SWATCH + i as i32, ID_VAR_EDIT + i as i32, ID_VAR_RESET + i as i32)).collect();
    // What each palette colour colours, on its name and its swatch.
    for (i, r) in vars.iter().enumerate() {
        phai_ui::props::tooltip(r.label, t(VARS[i].1));
        phai_ui::props::tooltip(r.swatch, t(VARS[i].1));
    }
    let props: Vec<Row> = (0..PROPS.len()).map(|i| row(254, 152 + i as i32 * 32, ID_PROP_SWATCH + i as i32, ID_PROP_EDIT + i as i32, ID_PROP_RESET + i as i32)).collect();
    let hint = win::label(hwnd, t("Bold: set in this file. ×: take it out - the theme it starts from shows."), 0, font);
    win::set_css(hint, "muted", "");
    at(hint, 254, 604, 380, 22);
    // Every kind of control on the right.
    let preview_frame = preview(hwnd, font, 650, 96);
    // Below: Undo, Redo, what happened, Save, Cancel.
    let undo_button = win::button(hwnd, t("Undo"), ID_UNDO, font, false);
    at(undo_button, 10, 646, 84, 28);
    let redo_button = win::button(hwnd, t("Redo"), ID_REDO, font, false);
    at(redo_button, 100, 646, 84, 28);
    let status = win::label(hwnd, "", 0, font);
    at(status, 196, 650, 700, 22);
    let save = win::button(hwnd, t("Save"), ID_SAVE, font, true);
    at(save, 920, 646, 90, 28);
    let cancel = win::button(hwnd, t("Cancel"), ID_CANCEL, font, false);
    at(cancel, 1020, 646, 90, 28);
    let mut ed = Box::new(Editor { hwnd, path: path.to_path_buf(), original: css.clone(), css, undo: vec![], redo: vec![], font, bold, tree, search, gallery, heading, selector_label, vars, props, status, undo_button, redo_button, themes_label, hint, preview_frame, save, cancel, on: None, filling: false, saved: false });
    ed.fill_tree();
    ed.show_rows();
    ed.fill();
    ed.say_steps();
    win::set_state(hwnd, ed);
    win::theme_frame(hwnd);
    unsafe {
        let style = wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32;
        let exs = wm::GetWindowLongW(hwnd, wm::GWL_EXSTYLE) as u32;
        let mut r = RECT { left: 0, top: 0, right: px(1120), bottom: px(700) };
        let _ = windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(style), false, wm::WINDOW_EX_STYLE(exs), win::dpi(hwnd) as u32);
        let _ = wm::SetWindowPos(hwnd, None, 0, 0, r.right - r.left, r.bottom - r.top, wm::SWP_NOMOVE | wm::SWP_NOZORDER);
    }
    win::center_on(hwnd, owner);
    if std::env::var_os("PHAI_SHOT").is_some() {
        // A picture of the editor, for checking: `PHAI_THEME_PAGE` 1 the Button element.
        if std::env::var("PHAI_THEME_PAGE").ok().and_then(|p| p.parse::<usize>().ok()) == Some(1) {
            if let Some(e) = unsafe { win::state::<Editor>(hwnd) } {
                e.choose(element(2, 0));
            }
        }
        // `PHAI_THEME_PRESET=N`: ready theme N used (not saved); `PHAI_TRAY_SHOT`: the colour tray
        // of the first palette colour photographed.
        if let Some(e) = unsafe { win::state::<Editor>(hwnd) } {
            if let Some(k) = std::env::var("PHAI_THEME_PRESET").ok().and_then(|p| p.parse::<usize>().ok()).filter(|k| *k < PRESETS.len()) {
                e.use_preset(k);
            }
            if std::env::var_os("PHAI_TRAY_SHOT").is_some() {
                win::show_top(hwnd);
                let _ = e.tray(e.vars[0].swatch, e.vars[0].edit);
            }
        }
        // `PHAI_THEME_WIDE`: sized as a large screen fills it, to see the layout grow.
        if std::env::var_os("PHAI_THEME_WIDE").is_some() {
            unsafe {
                let _ = wm::SetWindowPos(hwnd, None, 0, 0, px(1500), px(900), wm::SWP_NOMOVE | wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
            }
        }
        win::show_top(hwnd);
        layout(hwnd);
        return;
    }
    win::show_maximized(hwnd);
    layout(hwnd);
    win::modal_loop(hwnd, owner);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_ready_theme_makes_a_palette_that_reads() {
        for p in &PRESETS {
            let vars = preset_vars(p);
            assert_eq!(vars.len(), VARS.len());
            for (name, v) in &vars {
                assert!(theme::parse_color(v).is_some(), "{} {name}: {v:?}", p.name);
            }
            // Words stand out from the background.
            let get = |n: &str| theme::parse_color(&vars.iter().find(|(k, _)| *k == n).unwrap().1).unwrap();
            assert!((get("--text").luma() as i32 - get("--surface").luma() as i32).abs() > 120, "{}", p.name);
            let mut css = theme_edit::set_base("", if p.dark { "phai:dark" } else { "phai:light" });
            for (name, v) in &vars {
                css = theme_edit::set_var(&css, name, v);
            }
            let th = theme::Theme::parse_with(&css, &|n| Ok(if n == "phai:dark" { theme::DARK_CSS.to_string() } else { theme::LIGHT_CSS.to_string() })).unwrap();
            assert_eq!(th.var_color("--accent"), Some(rgb(p.accent)), "{}", p.name);
        }
    }
}

