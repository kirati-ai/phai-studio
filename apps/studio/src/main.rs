//! **Phai Studio** - design Windows programs in Rust: drag controls onto a form, and get real
//! Win32 Rust that is small, needs no runtime and shows Thai right.
//!
//! The window: a toolbox on the left, the form on its canvas in the middle, the chosen control's
//! properties on the right. Saving writes the `.phaiform`, the `<name>_ui.rs` generated from it,
//! and - once - the programmer's `<name>.rs`.
//!
//! `phai-studio [form.phaiform] [--dark] [--thai]`; `PHAI_SHOT=out.png` photographs it and ends.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bottom;
mod canvas;
mod explorer;
mod cargo;
mod project;
mod preview;
mod props;
mod text;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicPtr, Ordering};

use phai_form::codegen;
use phai_form::{snake, FormFile, Kind};
use phai_ui::controls;
use phai_ui::icon::glyph;
use phai_ui::layout::{Anchor, Dock, Item, Layout, Rect};
use phai_ui::text::{self as words, t, tf};
use phai_ui::theme::{self, Theme};
use phai_ui::win::{self, wide};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HFONT};
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HMENU};

use canvas::Canvas;
use props::{Key, Props};

const ID_NEW: i32 = 100;
const ID_OPEN: i32 = 101;
const ID_SAVE: i32 = 102;
const ID_SAVE_AS: i32 = 103;
const ID_EXIT: i32 = 104;
const ID_UNDO: i32 = 110;
const ID_REDO: i32 = 111;
const ID_DELETE: i32 = 112;
const ID_PREVIEW: i32 = 120;
const ID_LIGHT: i32 = 121;
const ID_DARK: i32 = 122;
const ID_THAI: i32 = 123;
const ID_ENGLISH: i32 = 124;
const ID_LANG: i32 = 125;
const ID_THEME_TOGGLE: i32 = 126;
const ID_NEW_PROJECT: i32 = 105;
const ID_BUILD: i32 = 140;
const ID_RUN: i32 = 141;
const ID_STOP: i32 = 142;
const ID_CLOSE: i32 = 106;
const ID_DEBUG: i32 = 143;
const ID_VIEW_TOOLBOX: i32 = 180;
const ID_VIEW_EXPLORER: i32 = 181;
const ID_VIEW_PROPS: i32 = 182;
const ID_VIEW_OUTPUT: i32 = 183;
const ID_SPLIT_SIDE: i32 = 207;
const ID_SPLIT_BOTTOM: i32 = 208;
/// File > Recent: `ID_RECENT + n`, newest first.
const ID_RECENT: i32 = 160;
/// File > New from template: `ID_TEMPLATE + n` in `Template::ALL` order.
const ID_TEMPLATE: i32 = 170;
const RECENT_MAX: usize = 9;
const ID_RELEASE: i32 = 144;
const ID_EDIT_MENUS: i32 = 113;
const ID_CUT: i32 = 115;
const ID_COPY: i32 = 116;
const ID_PASTE: i32 = 117;
const ID_DUPLICATE: i32 = 118;
const ID_SELECT_ALL: i32 = 119;
const ID_FRONT: i32 = 190;
const ID_BACK: i32 = 191;
const ID_LOCK: i32 = 192;
const ID_TAB_ORDER: i32 = 193;
/// Format menu: `ID_FORMAT + Op as index` in `FORMATS` order.
const ID_FORMAT: i32 = 150;
const FORMATS: [phai_form::arrange::Op; 11] = {
    use phai_form::arrange::Op::*;
    [Lefts, Centers, Rights, Tops, Middles, Bottoms, SameWidth, SameHeight, SameSize, SpaceAcross, SpaceDown]
};
const ID_EDIT_TOOLBAR: i32 = 114;
/// Toolbox buttons: the pointer, then each kind in `Kind::ALL` order.
const ID_TOOL: i32 = 300;
const ID_THEME_NEW: i32 = 130;
const ID_THEME_IMPORT: i32 = 131;
/// For the properties pane's Theme "…".
pub const ID_THEME_IMPORT_PUB: i32 = ID_THEME_IMPORT;
const ID_THEME_NONE: i32 = 132;

const ID_TOOLBAR: i32 = 200;
const ID_TOOLBOX: i32 = 201;
const ID_CANVAS: i32 = 202;
const ID_PROPS: i32 = 203;
const ID_STATUS: i32 = 204;
const ID_ERRORS: i32 = 205;
const ID_TABS: i32 = 206;
/// The error list's height when it shows, design pixels.
const ERRORS_HEIGHT: i32 = 140;

/// The docked panes the View menu shows and hides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pane {
    Toolbox,
    Explorer,
    Properties,
    Output,
}

pub struct Doc {
    pub form: FormFile,
    pub path: Option<PathBuf>,
    pub dirty: bool,
    /// A code file's editor, when this tab holds code rather than a form.
    pub code: Option<HWND>,
    /// The file had `\r\n` line ends; they are written back so.
    pub crlf: bool,
}

impl Doc {
    pub fn new(form: FormFile, path: Option<PathBuf>) -> Doc {
        Doc { form, path, dirty: false, code: None, crlf: false }
    }
}

/// An open form that is not the one on the canvas: everything it needs to come back as it was.
struct Tab {
    doc: Doc,
    undo: Vec<FormFile>,
    redo: Vec<FormFile>,
    selected: Option<usize>,
}

impl Tab {
    fn empty() -> Tab {
        Tab { doc: Doc::new(FormFile::new("MainWindow"), None), undo: vec![], redo: vec![], selected: None }
    }

    fn label(&self) -> String {
        let name = self.doc.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| t("Untitled").to_string());
        if self.doc.dirty {
            format!("{name} •")
        } else {
            name
        }
    }
}

pub struct Studio {
    pub hwnd: HWND,
    pub font: HFONT,
    pub toolbar: HWND,
    pub toolbox: HWND,
    pub status: HWND,
    pub errors: HWND,
    bottom: Option<Box<bottom::Bottom>>,
    /// Build with `--release`.
    release: bool,
    diagnostics: Vec<cargo::Diagnostic>,
    building: bool,
    running: Option<std::process::Child>,
    /// rust-analyzer for the project of the code being edited.
    lsp: Option<phai_editor::lsp::Lsp>,
    /// Requests sent to it, by id: which editor asked, and whether for a hover (else completions).
    asked: std::collections::HashMap<i64, (HWND, bool)>,
    pub canvas: Option<Canvas>,
    side: HWND,
    side_split: HWND,
    bottom_split: HWND,
    /// Design pixels: the side panes' width and the bottom pane's height.
    side_w: i32,
    bottom_h: i32,
    toolbox_on: bool,
    bottom_on: bool,
    pub props: Option<Props>,
    pub explorer: Option<explorer::Explorer>,
    tabbar: HWND,
    /// Every open form; the active one's slot is stale - its contents live in the fields below.
    tabs: Vec<Tab>,
    active: usize,
    layout: Layout,
    pub doc: Doc,
    undo: Vec<FormFile>,
    redo: Vec<FormFile>,
    pub selected: Option<usize>,
    /// Chosen besides `selected`, in the order chosen.
    pub also: Vec<usize>,
    pub tool: Option<Kind>,
}

static BUILD_TEST: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static STUDIO: AtomicPtr<Studio> = AtomicPtr::new(std::ptr::null_mut());

/// The one Studio of this process - its window's thread is the only one that asks.
pub fn studio() -> Option<&'static mut Studio> {
    unsafe { STUDIO.load(Ordering::Relaxed).as_mut() }
}

/// What the toolbox and the properties call a kind.
pub fn kind_label(k: Kind) -> &'static str {
    match k {
        Kind::Label => t("Label"),
        Kind::Edit => t("Text box"),
        Kind::Button => t("Button"),
        Kind::Checkbox => t("Check box"),
        Kind::Radio => t("Option"),
        Kind::Combo => t("Drop-down"),
        Kind::List => t("List"),
        Kind::Group => t("Group"),
        Kind::Tabs => t("Tabs"),
        Kind::Statusbar => t("Status bar"),
        Kind::Textarea => t("Text area"),
        Kind::Date => t("Date"),
        Kind::Number => t("Number"),
        Kind::Progress => t("Progress bar"),
        Kind::Slider => t("Slider"),
    }
}

/// The toolbox: the pointer, then every kind, as icons; the names are the tooltips.
fn toolbox_items() -> Vec<(i32, &'static str, u16)> {
    let mut v = vec![(ID_TOOL, t("Pointer"), glyph::POINTER)];
    for (i, k) in Kind::ALL.iter().enumerate() {
        let g = match k {
            Kind::Label => glyph::FONT,
            Kind::Edit => glyph::RENAME,
            Kind::Button => glyph::BUTTON,
            Kind::Checkbox => glyph::CHECKBOX,
            Kind::Radio => glyph::RADIO,
            Kind::Combo => glyph::DROPDOWN,
            Kind::List => glyph::LIST,
            Kind::Group => glyph::GROUP,
            Kind::Tabs => glyph::TABS,
            Kind::Statusbar => glyph::STATUS,
            Kind::Textarea => glyph::TEXTAREA,
            Kind::Date => glyph::CALENDAR,
            Kind::Number => glyph::NUMBER,
            Kind::Progress => glyph::PROGRESS,
            Kind::Slider => glyph::SLIDER,
        };
        v.push((ID_TOOL + 1 + i as i32, kind_label(*k), g));
    }
    v
}

fn toolbar_items() -> Vec<(i32, &'static str, u16)> {
    vec![
        (ID_NEW, t("New"), glyph::NEW),
        (ID_OPEN, t("Open"), glyph::OPEN),
        (ID_SAVE, t("Save"), glyph::SAVE),
        (0, "", 0),
        (ID_UNDO, t("Undo"), glyph::UNDO),
        (ID_REDO, t("Redo"), glyph::REDO),
        (ID_DELETE, t("Delete"), glyph::DELETE),
        (0, "", 0),
        (ID_FORMAT, t("Align lefts"), glyph::ALIGN_LEFT),
        (ID_FORMAT + 1, t("Align centers"), glyph::ALIGN_CENTER),
        (ID_FORMAT + 2, t("Align rights"), glyph::ALIGN_RIGHT),
        (0, "", 0),
        (ID_BUILD, t("Build"), glyph::BUILD),
        (ID_RUN, t("Run"), glyph::PLAY),
        (ID_PREVIEW, t("Preview"), glyph::MONITOR),
        (0, "", 0),
        (ID_THEME_TOGGLE, t("Theme"), glyph::BRIGHTNESS),
        (ID_LANG, t("Language"), glyph::LANGUAGE),
    ]
}

fn build_menu() -> HMENU {
    unsafe {
        let bar = wm::CreateMenu().unwrap_or_default();
        let add = |m: HMENU, id: i32, label: &str, key: &str| {
            let text = if key.is_empty() { label.to_string() } else { format!("{label}\t{key}") };
            let w = wide(&text);
            let _ = wm::AppendMenuW(m, wm::MF_STRING, id as usize, PCWSTR(w.as_ptr()));
        };
        let sep = |m: HMENU| {
            let _ = wm::AppendMenuW(m, wm::MF_SEPARATOR, 0, None);
        };
        let sub_of = |parent: HMENU, label: &str, m: HMENU| {
            let w = wide(label);
            let _ = wm::AppendMenuW(parent, wm::MF_POPUP, m.0 as usize, PCWSTR(w.as_ptr()));
        };
        let sub = |label: &str, m: HMENU| {
            let w = wide(label);
            let _ = wm::AppendMenuW(bar, wm::MF_POPUP, m.0 as usize, PCWSTR(w.as_ptr()));
        };
        let file = wm::CreatePopupMenu().unwrap_or_default();
        add(file, ID_NEW, t("New"), "Ctrl+N");
        let tm = wm::CreatePopupMenu().unwrap_or_default();
        for (k, tpl) in phai_design::templates::Template::ALL.iter().enumerate() {
            add(tm, ID_TEMPLATE + k as i32, t(tpl.name()), "");
        }
        sub_of(file, t("New from template"), tm);
        add(file, ID_OPEN, t("Open..."), "Ctrl+O");
        add(file, ID_SAVE, t("Save"), "Ctrl+S");
        add(file, ID_SAVE_AS, t("Save as..."), "");
        add(file, ID_CLOSE, t("Close"), "Ctrl+W");
        sep(file);
        add(file, ID_NEW_PROJECT, t("New project..."), "");
        let recent = recent_files();
        if !recent.is_empty() {
            let rm = wm::CreatePopupMenu().unwrap_or_default();
            for (k, p) in recent.iter().enumerate() {
                add(rm, ID_RECENT + k as i32, &format!("&{} {}", k + 1, p.display()), "");
            }
            sub_of(file, t("Recent"), rm);
        }
        sep(file);
        add(file, ID_EXIT, t("Exit"), "");
        sub(t("&File"), file);
        let edit = wm::CreatePopupMenu().unwrap_or_default();
        add(edit, ID_UNDO, t("Undo"), "Ctrl+Z");
        add(edit, ID_REDO, t("Redo"), "Ctrl+Y");
        sep(edit);
        add(edit, ID_CUT, t("Cut"), "Ctrl+X");
        add(edit, ID_COPY, t("Copy"), "Ctrl+C");
        add(edit, ID_PASTE, t("Paste"), "Ctrl+V");
        add(edit, ID_DUPLICATE, t("Duplicate"), "Ctrl+D");
        add(edit, ID_DELETE, t("Delete"), "Del");
        sep(edit);
        add(edit, ID_SELECT_ALL, t("Select all"), "Ctrl+A");
        sep(edit);
        add(edit, ID_EDIT_MENUS, t("Menus..."), "");
        add(edit, ID_EDIT_TOOLBAR, t("Toolbar..."), "");
        sub(t("&Edit"), edit);
        let view = wm::CreatePopupMenu().unwrap_or_default();
        add(view, ID_PREVIEW, t("Preview"), "F4");
        sep(view);
        add(view, ID_VIEW_TOOLBOX, t("Toolbox"), "");
        add(view, ID_VIEW_EXPLORER, t("Project explorer"), "");
        add(view, ID_VIEW_PROPS, t("Properties"), "");
        add(view, ID_VIEW_OUTPUT, t("Errors and output"), "");
        if let Some(s) = studio() {
            let (e, p) = explorer::side_panes(s.side);
            for (id, on) in [(ID_VIEW_TOOLBOX, s.toolbox_on), (ID_VIEW_EXPLORER, e), (ID_VIEW_PROPS, p), (ID_VIEW_OUTPUT, s.bottom_on)] {
                if on {
                    wm::CheckMenuItem(view, id as u32, wm::MF_CHECKED.0);
                }
            }
        }
        sep(view);
        add(view, ID_LIGHT, t("Light"), "");
        add(view, ID_DARK, t("Dark"), "");
        sep(view);
        add(view, ID_THAI, "ไทย", "");
        add(view, ID_ENGLISH, "English", "");
        wm::CheckMenuItem(view, if theme::current().is_dark() { ID_DARK } else { ID_LIGHT } as u32, wm::MF_CHECKED.0);
        wm::CheckMenuItem(view, if words::is_thai() { ID_THAI } else { ID_ENGLISH } as u32, wm::MF_CHECKED.0);
        sub(t("&View"), view);
        let fmt = wm::CreatePopupMenu().unwrap_or_default();
        let names = [t("Align lefts"), t("Align centers"), t("Align rights"), t("Align tops"), t("Align middles"), t("Align bottoms"), t("Same width"), t("Same height"), t("Same size"), t("Space evenly across"), t("Space evenly down")];
        for (k, name) in names.iter().enumerate() {
            if k == 6 || k == 9 {
                sep(fmt);
            }
            add(fmt, ID_FORMAT + k as i32, name, "");
        }
        sep(fmt);
        add(fmt, ID_FRONT, t("Bring to front"), "");
        add(fmt, ID_BACK, t("Send to back"), "");
        sep(fmt);
        add(fmt, ID_LOCK, t("Lock controls"), "");
        add(fmt, ID_TAB_ORDER, t("Tab order"), "");
        if let Some(c) = studio().and_then(|s| s.canvas.as_ref()) {
            for (id, on) in [(ID_LOCK, c.locked), (ID_TAB_ORDER, c.tab_mode())] {
                if on {
                    wm::CheckMenuItem(fmt, id as u32, wm::MF_CHECKED.0);
                }
            }
        }
        sub(t("F&ormat"), fmt);
        let proj = wm::CreatePopupMenu().unwrap_or_default();
        add(proj, ID_BUILD, t("Build"), "Ctrl+B");
        add(proj, ID_RUN, t("Run"), "F5");
        add(proj, ID_STOP, t("Stop"), "Shift+F5");
        sep(proj);
        add(proj, ID_DEBUG, "Debug", "");
        add(proj, ID_RELEASE, "Release", "");
        let release = studio().is_some_and(|s| s.release);
        wm::CheckMenuItem(proj, if release { ID_RELEASE } else { ID_DEBUG } as u32, wm::MF_CHECKED.0);
        sub(t("&Project"), proj);
        let th = wm::CreatePopupMenu().unwrap_or_default();
        add(th, ID_THEME_NEW, t("New theme from the standard..."), "");
        add(th, ID_THEME_IMPORT, t("Import CSS..."), "");
        sep(th);
        add(th, ID_THEME_NONE, t("No theme"), "");
        sub(t("&Theme"), th);
        bar
    }
}

impl Studio {
    // --- editing the model -------------------------------------------------------------------

    /// Remembers the form as it is, for Undo.
    pub fn checkpoint(&mut self) {
        self.undo.push(self.doc.form.clone());
        if self.undo.len() > 200 {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// One change, one undo step.
    pub fn edit(&mut self, change: impl FnOnce(&mut FormFile)) {
        self.checkpoint();
        change(&mut self.doc.form);
        self.changed();
    }

    fn changed(&mut self) {
        let was = self.doc.dirty;
        self.doc.dirty = true;
        self.say_title();
        if !was {
            self.refresh_tabs();
        }
    }

    /// A property typed into its box: the model takes it now, the canvas shows it now.
    pub fn apply(&mut self, key: Key, text: &str) {
        let f = &mut self.doc.form;
        let num = text.trim().parse::<i32>().ok();
        // A list is one item a line, as the "…" dialog writes it; commas also part, as typed in a box.
        let list = || text.split(['\n', ',']).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect::<Vec<_>>();
        let yes = text.trim().eq_ignore_ascii_case("true");
        // A colour as CSS writes it, or nothing; a word that is not one changes nothing.
        let colour = || -> Result<Option<String>, ()> {
            let t = text.trim();
            if t.is_empty() {
                Ok(None)
            } else if phai_ui::theme::parse_color(t).is_some() {
                Ok(Some(t.to_string()))
            } else {
                Err(())
            }
        };
        match (key, self.selected) {
            (Key::FormName, _) => f.form.name = text.trim().to_string(),
            (Key::FormTitle, _) => f.form.title = text.to_string(),
            (Key::FormWidth, _) => match num {
                Some(n) if n >= 120 => f.form.size[0] = n,
                _ => return,
            },
            (Key::FormHeight, _) => match num {
                Some(n) if n >= 80 => f.form.size[1] = n,
                _ => return,
            },
            (Key::FormFont, _) => {
                f.form.font_size = num.filter(|n| (6..=48).contains(n));
                if !text.trim().is_empty() && f.form.font_size.is_none() {
                    return;
                }
            }
            (Key::FormTheme, _) => {
                f.form.theme = (!text.trim().is_empty()).then(|| text.trim().to_string());
                self.changed();
                self.load_theme();
                return;
            }
            (_, None) => return,
            (k, Some(i)) => {
                let c = &mut f.control[i];
                let [x, y, w, h] = c.rect();
                match k {
                    Key::Id => c.id = text.trim().to_string(),
                    Key::Text => c.text = Some(text.to_string()),
                    Key::X => c.bounds = Some([num.unwrap_or(x), y, w, h]),
                    Key::Y => c.bounds = Some([x, num.unwrap_or(y), w, h]),
                    Key::W => c.bounds = Some([x, y, num.unwrap_or(w).max(1), h]),
                    Key::H => c.bounds = Some([x, y, w, num.unwrap_or(h).max(1)]),
                    Key::Class => c.class = (!text.trim().is_empty()).then(|| text.trim().to_string()),
                    Key::Font => {
                        c.font_size = num.filter(|n| (6..=48).contains(n));
                        if !text.trim().is_empty() && c.font_size.is_none() {
                            return;
                        }
                    }
                    Key::Event => c.set_handler((!text.trim().is_empty()).then(|| text.trim().to_string())),
                    Key::Bold => c.bold = yes.then_some(true),
                    Key::BackColor | Key::ForeColor | Key::BorderColor => {
                        let Ok(v) = colour() else { return };
                        match k {
                            Key::BackColor => c.back_color = v,
                            Key::ForeColor => c.fore_color = v,
                            _ => c.border_color = v,
                        }
                    }
                    Key::Radius => match num {
                        Some(n) if (0..=50).contains(&n) => c.radius = Some(n),
                        None if text.trim().is_empty() => c.radius = None,
                        _ => return,
                    },
                    Key::Enabled => c.enabled = (!yes).then_some(false),
                    Key::Visible => c.visible = (!yes).then_some(false),
                    Key::Checked => c.checked = yes.then_some(true),
                    Key::Password => c.password = yes.then_some(true),
                    Key::Group => c.group = num.filter(|n| *n != 0),
                    Key::Dock => {
                        let d = text.trim();
                        if !phai_form::DOCKS.contains(&d) {
                            return;
                        }
                        c.dock = (d != "none").then(|| d.to_string());
                        if c.dock.is_none() && c.bounds.is_none() {
                            let (dw, dh) = c.kind.default_size();
                            c.bounds = Some([11, 11, dw.max(20), dh]);
                        }
                    }
                    Key::Anchor => match phai_form::parse_anchor(Some(text.trim())) {
                        Ok(a) => c.anchor = phai_form::anchor_text(a),
                        Err(_) => return,
                    },
                    Key::Min | Key::Max | Key::Value => {
                        let Some(n) = num else { return };
                        let (mut lo, mut hi, mut v) = (c.min.unwrap_or(0), c.max.unwrap_or(100), c.value.unwrap_or(0));
                        match k {
                            Key::Min => lo = n,
                            Key::Max => hi = n,
                            _ => v = n,
                        }
                        if lo > hi {
                            return;
                        }
                        v = v.clamp(lo, hi);
                        c.min = (lo != 0).then_some(lo);
                        c.max = (hi != 100).then_some(hi);
                        c.value = (v != 0).then_some(v);
                    }
                    Key::Items => match c.kind {
                        Kind::Combo => c.items = list(),
                        Kind::Tabs => c.tabs = list(),
                        Kind::List => {
                            c.columns = list().into_iter().map(|s| match s.rsplit_once(':') {
                                Some((n, w)) => (n.trim().to_string(), w.trim().parse().unwrap_or(100)),
                                None => (s, 100),
                            }).collect()
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        self.changed();
        self.rebuild();
        self.say_selection();
    }

    pub fn rebuild(&mut self) {
        if let Some(c) = self.canvas.as_mut() {
            c.rebuild();
        }
    }

    pub fn refresh_props(&mut self) {
        if let Some(p) = self.props.as_mut() {
            p.refresh();
        }
    }

    /// Everything chosen, the first one first.
    pub fn selection(&self) -> Vec<usize> {
        self.selected.into_iter().chain(self.also.iter().copied()).collect()
    }

    /// Ctrl+click: in or out of the choice.
    pub fn toggle(&mut self, i: usize) {
        if self.selected == Some(i) {
            self.selected = if self.also.is_empty() { None } else { Some(self.also.remove(0)) };
        } else if let Some(k) = self.also.iter().position(|x| *x == i) {
            self.also.remove(k);
        } else if self.selected.is_none() {
            self.selected = Some(i);
        } else {
            self.also.push(i);
        }
        self.refresh_props();
        self.say_selection();
    }

    pub fn make_primary(&mut self, i: usize) {
        if let Some(k) = self.also.iter().position(|x| *x == i) {
            self.also.remove(k);
            if let Some(old) = self.selected.replace(i) {
                self.also.insert(0, old);
            }
            self.refresh_props();
        }
    }

    pub fn select_many(&mut self, all: &[usize]) {
        self.selected = all.first().copied();
        self.also = all.iter().skip(1).copied().collect();
        self.refresh_props();
        if let Some(c) = self.canvas.as_ref() {
            c.place_handles();
        }
        self.say_selection();
    }

    pub fn select(&mut self, i: Option<usize>) {
        self.also.clear();
        if self.selected != i {
            self.selected = i;
            self.refresh_props();
        }
        if let Some(c) = self.canvas.as_ref() {
            c.place_handles();
        }
        self.say_selection();
    }

    pub fn delete_selected(&mut self) {
        let mut chosen = self.selection();
        if chosen.is_empty() {
            return;
        }
        chosen.sort_unstable();
        self.edit(|f| {
            for i in chosen.iter().rev() {
                f.control.remove(*i);
            }
        });
        self.selected = None;
        self.also.clear();
        self.rebuild();
        self.refresh_props();
        self.say_selection();
    }

    fn undo_redo(&mut self, back: bool) {
        let (from, to) = if back { (&mut self.undo, &mut self.redo) } else { (&mut self.redo, &mut self.undo) };
        let Some(f) = from.pop() else { return };
        to.push(std::mem::replace(&mut self.doc.form, f));
        if self.selected.is_some_and(|i| i >= self.doc.form.control.len()) {
            self.selected = None;
        }
        let n = self.doc.form.control.len();
        self.also.retain(|i| *i < n);
        self.changed();
        self.rebuild();
        self.refresh_props();
        self.say_selection();
    }

    /// Double-click: give the control a handler if it has none, and put the empty method in the
    /// programmer's file if the form is saved and the method is not there yet.
    pub fn add_handler_for_selected(&mut self) {
        let Some(i) = self.selected else { return };
        let c = &self.doc.form.control[i];
        let Some(event) = c.kind.event() else { return };
        let name = c.handler().map(str::to_string).unwrap_or_else(|| {
            let verb = match event {
                "on_click" => "clicked",
                "on_change" => "changed",
                _ => "chosen",
            };
            format!("{}_{verb}", c.id)
        });
        if c.handler() != Some(name.as_str()) {
            let n = name.clone();
            self.edit(|f| f.control[i].set_handler(Some(n)));
            self.refresh_props();
        }
        let Some(path) = self.doc.path.clone() else { return };
        let user = path.with_file_name(format!("{}.rs", snake(&self.doc.form.form.name)));
        if let Ok(src) = std::fs::read_to_string(&user) {
            if let Some(more) = codegen::add_handler(&src, &self.doc.form.form.name, &name) {
                if std::fs::write(&user, more).is_ok() {
                    controls::status_text(self.status, 0, &tf("Added {0} to {1}", &[&format!("fn {name}"), &user.file_name().unwrap_or_default().to_string_lossy()]));
                }
            }
        }
    }

    pub fn toolbox_reset(&mut self) {
        win::toolbar_choose(self.toolbox, ID_TOOL);
    }

    /// The form's size as the canvas shows it now.
    pub fn drag_size(&self) -> [i32; 2] {
        self.canvas.as_ref().and_then(|c| c.shown_size()).unwrap_or(self.doc.form.form.size)
    }

    // --- clipboard and format ---------------------------------------------------------------------

    fn chosen_controls(&self) -> Vec<phai_form::Control> {
        self.selection().into_iter().filter_map(|i| self.doc.form.control.get(i).cloned()).collect()
    }

    fn copy(&mut self) {
        let chosen = self.chosen_controls();
        if !chosen.is_empty() {
            set_clipboard(self.hwnd, &phai_form::arrange::to_clipboard(&chosen));
        }
    }

    /// Adds controls read from `text` and chooses them.
    fn add_controls(&mut self, text: &str) {
        let Some(new) = phai_form::arrange::from_clipboard(text, &self.doc.form, canvas::GRID * 2) else { return };
        let first = self.doc.form.control.len();
        let count = new.len();
        self.edit(|f| f.control.extend(new));
        self.rebuild();
        self.select_many(&(first..first + count).collect::<Vec<_>>());
    }

    fn paste(&mut self) {
        if let Some(text) = get_clipboard(self.hwnd) {
            self.add_controls(&text);
        }
    }

    fn duplicate(&mut self) {
        let chosen = self.chosen_controls();
        if !chosen.is_empty() {
            self.add_controls(&phai_form::arrange::to_clipboard(&chosen));
        }
    }

    /// Format > Bring to front / Send to back. Windows stacks the first child made on top and
    /// visits it first with Tab, so the front is the start of the form's list, the back its end.
    fn restack(&mut self, front: bool) {
        let mut chosen = self.selection();
        if chosen.is_empty() {
            return;
        }
        chosen.sort_unstable();
        let mut moved: Vec<usize> = Vec::new();
        self.edit(|f| {
            let taken: Vec<phai_form::Control> = chosen.iter().rev().map(|i| f.control.remove(*i)).collect::<Vec<_>>().into_iter().rev().collect();
            let n = taken.len();
            if front {
                for (k, c) in taken.into_iter().enumerate() {
                    f.control.insert(k, c);
                }
                moved = (0..n).collect();
            } else {
                let start = f.control.len();
                f.control.extend(taken);
                moved = (start..start + n).collect();
            }
        });
        self.rebuild();
        self.select_many(&moved);
    }

    /// The toolbox double-clicked: the kind goes on the form at once, as in Visual Studio, at the
    /// margin or a little below and right of what is already there.
    fn add_at_margin(&mut self, kind: Kind) {
        let m = phai_design::metrics::MARGIN;
        let (w, h) = kind.default_size();
        let taken: Vec<(i32, i32)> = self.doc.form.control.iter().filter_map(|c| c.bounds).map(|b| (b[0], b[1])).collect();
        let mut at = (m, m + if self.doc.form.toolbar.is_empty() { 0 } else { phai_ui::win::TOOLBAR_HEIGHT });
        while taken.contains(&at) {
            at = (at.0 + canvas::GRID * 2, at.1 + canvas::GRID * 2);
        }
        self.edit(|f| {
            let id = f.next_id(kind);
            let mut c = phai_form::Control::new(kind, &id, [at.0, at.1, w, h]);
            if kind == Kind::Statusbar {
                c.bounds = Some([0, 0, 0, 26]);
            }
            f.control.push(c);
        });
        self.tool = None;
        self.toolbox_reset();
        self.rebuild();
        let last = self.doc.form.control.len() - 1;
        self.select(Some(last));
    }

    /// Lines the chosen controls up against the first one chosen.
    fn format(&mut self, op: phai_form::arrange::Op) {
        let chosen: Vec<usize> = self.selection().into_iter().filter(|i| self.doc.form.control[*i].dock.as_deref().is_none_or(|d| d == "none")).collect();
        if chosen.len() < 2 {
            return;
        }
        let mut rects: Vec<[i32; 4]> = chosen.iter().map(|i| self.doc.form.control[*i].rect()).collect();
        phai_form::arrange::arrange(&mut rects, 0, op);
        self.edit(|f| {
            for (i, r) in chosen.iter().zip(&rects) {
                f.control[*i].bounds = Some(*r);
            }
        });
        self.rebuild();
        self.refresh_props();
        self.say_selection();
    }

    // --- menus and toolbar -----------------------------------------------------------------------

    /// The form's menus as text in one box; asked again until it reads or is cancelled.
    pub fn edit_menus(&mut self) {
        let mut text = phai_form::bars::menus_to_text(&self.doc.form.menu);
        if text.is_empty() {
            text = "&File\n  exit | Exit\n".into();
        }
        loop {
            let Some(answer) = phai_ui::dialogs::ask_lines(Some(self.hwnd), t("Menus"), t("A menu at the left; its items indented: id | label | key. A line of - separates."), &text) else { return };
            match phai_form::bars::menus_from_text(&answer) {
                Ok(menus) => {
                    self.edit(|f| f.menu = menus);
                    self.rebuild();
                    return;
                }
                Err(e) => {
                    win::message(Some(self.hwnd), &e, t("Menus"), wm::MB_ICONWARNING);
                    text = answer;
                }
            }
        }
    }

    /// The form's toolbar as text: one button a line, id | label | GLYPH.
    pub fn edit_toolbar(&mut self) {
        let mut text = phai_form::bars::toolbar_to_text(&self.doc.form.toolbar);
        if text.is_empty() {
            text = "save | Save | SAVE\n".into();
        }
        loop {
            let Some(answer) = phai_ui::dialogs::ask_lines(Some(self.hwnd), t("Toolbar"), t("One button a line: id | label | GLYPH. The label is the tooltip."), &text) else { return };
            match phai_form::bars::toolbar_from_text(&answer) {
                Ok(tools) => {
                    self.edit(|f| f.toolbar = tools);
                    self.rebuild();
                    return;
                }
                Err(e) => {
                    win::message(Some(self.hwnd), &e, t("Toolbar"), wm::MB_ICONWARNING);
                    text = answer;
                }
            }
        }
    }

    // --- tabs -----------------------------------------------------------------------------------

    /// The form on the canvas, packed away into its tab.
    fn stash(&mut self) {
        let doc = std::mem::replace(&mut self.doc, Doc::new(FormFile::new("MainWindow"), None));
        self.tabs[self.active] = Tab { doc, undo: std::mem::take(&mut self.undo), redo: std::mem::take(&mut self.redo), selected: self.selected.take() };
    }

    /// Tab `i` onto the canvas.
    fn activate(&mut self, i: usize) {
        if i >= self.tabs.len() {
            return;
        }
        if i != self.active {
            self.stash();
            self.active = i;
        }
        let tab = std::mem::replace(&mut self.tabs[i], Tab::empty());
        self.doc = tab.doc;
        self.undo = tab.undo;
        self.redo = tab.redo;
        self.selected = tab.selected;
        self.after_switch();
    }

    /// Shows the active tab's editor and hides every other one; the canvas shows when it is a form.
    fn show_code(&mut self) {
        let active = self.doc.code;
        for t in &self.tabs {
            if let Some(h) = t.doc.code {
                win::show(h, false);
            }
        }
        if let Some(c) = self.canvas.as_ref() {
            win::show(c.surface, active.is_none());
            if let Some(h) = active {
                let mut r = RECT::default();
                unsafe {
                    let _ = wm::GetClientRect(c.hwnd, &mut r);
                }
                win::move_to(h, 0, 0, r.right, r.bottom);
                unsafe {
                    // Children made later sit lower: lift the editor over the form's windows.
                    let _ = wm::SetWindowPos(h, Some(wm::HWND_TOP), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_SHOWWINDOW);
                }
                unsafe {
                    let _ = km::SetFocus(Some(h));
                }
            }
        }
    }

    /// Opens a code file in an editor tab - or the tab it is already in.
    pub fn open_code(&mut self, path: &Path) {
        let same = |q: &Option<PathBuf>| q.as_ref().is_some_and(|q| q == path);
        if same(&self.doc.path) {
            return self.after_switch();
        }
        if let Some(i) = self.tabs.iter().enumerate().position(|(i, t)| i != self.active && same(&t.doc.path)) {
            return self.activate(i);
        }
        let Ok(raw) = std::fs::read_to_string(path) else {
            win::message(Some(self.hwnd), &tf("{0} cannot be opened", &[&path.display()]), "Phai Studio", wm::MB_ICONWARNING);
            return;
        };
        let (text, crlf) = phai_editor::buffer::normalize(&raw);
        let Some(parent) = self.canvas.as_ref().map(|c| c.hwnd) else { return };
        let h = phai_editor::view::create(parent, 900, &text, &path.to_string_lossy());
        let blank = self.doc.path.is_none() && !self.doc.dirty && self.doc.form.control.is_empty() && self.doc.code.is_none();
        if !blank {
            self.stash();
            self.tabs.push(Tab::empty());
            self.active = self.tabs.len() - 1;
        }
        self.doc = Doc { code: Some(h), crlf, ..Doc::new(FormFile::new("Code"), Some(path.to_path_buf())) };
        if path.extension().is_some_and(|e| e == "rs") {
            if let Some(l) = self.lsp_for(path) {
                l.did_open(path, &text);
            }
        }
        self.undo.clear();
        self.redo.clear();
        self.selected = None;
        self.after_switch();
    }

    /// rust-analyzer for the project holding `path`, started if it is not running for it already.
    fn lsp_for(&mut self, path: &Path) -> Option<&mut phai_editor::lsp::Lsp> {
        let root = cargo::project_of(path)?;
        if self.lsp.as_ref().is_none_or(|l| !phai_editor::lsp::same_path(&l.root, &root)) {
            self.lsp = phai_editor::lsp::Lsp::start(&root, self.hwnd);
            if self.lsp.is_none() {
                controls::status_text(self.status, 0, t("rust-analyzer is not installed: rustup component add rust-analyzer"));
            }
        }
        self.lsp.as_mut()
    }

    /// The editor of the open file at `path`, in any tab.
    fn editor_of(&self, path: &Path) -> Option<HWND> {
        let same = |p: &Option<PathBuf>| p.as_deref().is_some_and(|q| phai_editor::lsp::same_path(q, path));
        if same(&self.doc.path) {
            return self.doc.code;
        }
        self.tabs.iter().find(|t| same(&t.doc.path)).and_then(|t| t.doc.code)
    }

    /// A message from rust-analyzer.
    fn lsp_message(&mut self, msg: serde_json::Value) {
        // A request of ours answered.
        if let (Some(id), None) = (msg.get("id").and_then(|i| i.as_i64()), msg.get("method")) {
            if let Some((h, hover)) = self.asked.remove(&id) {
                let result = msg.get("result").cloned().unwrap_or_default();
                if hover {
                    if let Some(text) = phai_editor::lsp::hover_text(&result) {
                        phai_editor::view::show_tip(h, &text);
                    }
                } else {
                    phai_editor::view::show_completions(h, phai_editor::lsp::completions_of(&result, 300));
                }
            }
            return;
        }
        // The server asking something of us: it waits for an answer.
        if let (Some(id), Some(_)) = (msg.get("id"), msg.get("method")) {
            if let Some(l) = self.lsp.as_ref() {
                l.respond(id);
            }
            return;
        }
        if let Some((path, diags)) = phai_editor::lsp::diagnostics_of(&msg) {
            if let Some(h) = self.editor_of(&path) {
                phai_editor::view::set_diagnostics(h, &diags);
                if Some(h) == self.doc.code {
                    let (e, w) = phai_editor::view::diagnostic_counts(h);
                    controls::status_text(self.status, 0, &tf("{0} errors, {1} warnings", &[&e, &w]));
                }
            }
        }
    }

    /// The pointer rested in an editor: ask what is under it.
    pub fn lsp_hover(&mut self, h: HWND) {
        let (Some(path), Some(pos)) = (self.doc.path.clone(), phai_editor::view::hover_position(h)) else { return };
        if let Some(l) = self.lsp_for(&path) {
            let id = l.hover(&path, pos.0, pos.1);
            self.asked.insert(id, (h, true));
        }
    }

    /// Completions were asked for at the caret.
    pub fn lsp_complete(&mut self, h: HWND) {
        let Some(path) = self.doc.path.clone() else { return };
        let pos = phai_editor::view::caret_lsp(h);
        if let Some(l) = self.lsp_for(&path) {
            let id = l.completion(&path, pos.0, pos.1);
            self.asked.insert(id, (h, false));
        }
    }

    /// The editor said its text changed.
    pub fn code_changed(&mut self) {
        if let (Some(h), Some(path)) = (self.doc.code, self.doc.path.clone()) {
            if path.extension().is_some_and(|e| e == "rs") {
                let text = phai_editor::view::text(h);
                let version = phai_editor::view::version(h);
                if let Some(l) = self.lsp_for(&path) {
                    l.did_change(&path, version, &text);
                }
            }
        }
        if let Some(h) = self.doc.code {
            let dirty = phai_editor::view::is_dirty(h);
            if dirty != self.doc.dirty {
                self.doc.dirty = dirty;
                self.say_title();
                self.refresh_tabs();
            }
            let (l, c) = phai_editor::view::caret_position(h);
            controls::status_text(self.status, 1, &tf("Line {0}, column {1}", &[&l, &c]));
        }
    }

    /// Everything that shows the active form, again.
    fn after_switch(&mut self) {
        self.load_theme();
        if self.doc.code.is_none() {
            self.rebuild();
        }
        self.refresh_props();
        self.say_title();
        self.say_selection();
        self.refresh_tabs();
        let path = self.doc.path.clone();
        if let Some(e) = self.explorer.as_mut() {
            e.show_project_of(path.as_deref());
        }
        // Last, so nothing shown above leaves the form's surface over the editor.
        self.show_code();
    }

    fn refresh_tabs(&mut self) {
        let mut labels: Vec<String> = Vec::new();
        for (i, tab) in self.tabs.iter().enumerate() {
            labels.push(if i == self.active { Tab { doc: Doc { dirty: self.doc.dirty, ..Doc::new(FormFile::new("X"), self.doc.path.clone()) }, undo: vec![], redo: vec![], selected: None }.label() } else { tab.label() });
        }
        let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
        controls::tabs_relabel(self.tabbar, &refs);
        controls::tabs_select(self.tabbar, self.active);
    }

    /// A form in a tab of its own - or the tab it is already in. An untouched empty first tab is
    /// used rather than left behind.
    fn open_in_tab(&mut self, form: FormFile, path: Option<PathBuf>) {
        if let Some(p) = &path {
            let same = |q: &Option<PathBuf>| q.as_ref().is_some_and(|q| q == p);
            if same(&self.doc.path) {
                return self.after_switch();
            }
            if let Some(i) = self.tabs.iter().enumerate().position(|(i, t)| i != self.active && same(&t.doc.path)) {
                return self.activate(i);
            }
        }
        let blank = self.doc.path.is_none() && !self.doc.dirty && self.doc.form.control.is_empty();
        if !blank {
            self.stash();
            self.tabs.push(Tab::empty());
            self.active = self.tabs.len() - 1;
        }
        self.doc = Doc::new(form, path);
        self.undo.clear();
        self.redo.clear();
        self.selected = None;
        self.after_switch();
    }

    /// Closes the active tab, asking to save; the last one closed leaves an empty form.
    fn close_tab(&mut self) -> bool {
        if !self.may_leave() {
            return false;
        }
        if let Some(h) = self.doc.code.take() {
            if let (Some(l), Some(p)) = (self.lsp.as_ref(), self.doc.path.as_ref()) {
                l.did_close(p);
            }
            unsafe {
                let _ = wm::DestroyWindow(h);
            }
        }
        if self.tabs.len() == 1 {
            self.doc = Doc::new(FormFile::new("MainWindow"), None);
            self.undo.clear();
            self.redo.clear();
            self.selected = None;
            self.after_switch();
            return true;
        }
        self.tabs.remove(self.active);
        let next = self.active.min(self.tabs.len() - 1);
        self.active = next;
        let tab = std::mem::replace(&mut self.tabs[next], Tab::empty());
        self.doc = tab.doc;
        self.undo = tab.undo;
        self.redo = tab.redo;
        self.selected = tab.selected;
        self.after_switch();
        true
    }

    /// Asks about every unsaved tab before the window closes; false when one said Cancel.
    fn may_quit(&mut self) -> bool {
        for i in 0..self.tabs.len() {
            let dirty = if i == self.active { self.doc.dirty } else { self.tabs[i].doc.dirty };
            if dirty {
                self.activate(i);
                if !self.may_leave() {
                    return false;
                }
            }
        }
        true
    }

    // --- saying things --------------------------------------------------------------------------

    fn say_title(&self) {
        let name = self.doc.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| t("Untitled").to_string());
        win::set_text(self.hwnd, &format!("{}{name} - Phai Studio", if self.doc.dirty { "• " } else { "" }));
    }

    pub fn say_selection(&self) {
        if let Some(h) = self.doc.code {
            let (l, c) = phai_editor::view::caret_position(h);
            controls::status_text(self.status, 1, &tf("Line {0}, column {1}", &[&l, &c]));
            return;
        }
        let f = &self.doc.form;
        let text = match self.selected.and_then(|i| f.control.get(i)) {
            Some(c) => {
                let [x, y, w, h] = c.rect();
                match c.dock.as_deref() {
                    Some(d) => format!("{} · {d}", c.id),
                    None => format!("{} · {x}, {y} · {w} × {h}", c.id),
                }
            }
            None => format!("{} · {} × {}", f.form.name, f.form.size[0], f.form.size[1]),
        };
        controls::status_text(self.status, 1, &text);
    }

    // --- files --------------------------------------------------------------------------------

    /// Asks to save changes; false when the person cancelled what they were doing.
    fn may_leave(&mut self) -> bool {
        if !self.doc.dirty {
            return true;
        }
        let name = self.doc.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| t("Untitled").to_string());
        match win::message(Some(self.hwnd), &tf("Save changes to {0}?", &[&name]), "Phai Studio", wm::MB_YESNOCANCEL | wm::MB_ICONQUESTION) {
            wm::IDYES => self.save(false),
            wm::IDNO => true,
            _ => false,
        }
    }

    fn set_doc(&mut self, form: FormFile, path: Option<PathBuf>) {
        self.open_in_tab(form, path);
    }

    pub fn open_path(&mut self, path: &Path) {
        let name = path.display().to_string();
        match std::fs::read_to_string(path).map_err(|e| e.to_string()).and_then(|src| FormFile::parse(&src).map_err(|e| e.to_string())) {
            Ok(f) => {
                self.set_doc(f, Some(path.to_path_buf()));
                remember_recent(path);
                self.rebuild_menu();
            }
            Err(e) => {
                win::message(Some(self.hwnd), &format!("{}\n\n{e}", tf("{0} cannot be opened", &[&name])), "Phai Studio", wm::MB_ICONWARNING);
            }
        }
    }

    /// Writes the form, its generated code and - the first time - the programmer's file.
    fn save(&mut self, choose: bool) -> bool {
        // A code file: its text, with the line ends it came with.
        if let (Some(h), Some(path)) = (self.doc.code, self.doc.path.clone()) {
            let text = phai_editor::view::text(h);
            let text = if self.doc.crlf { text.replace('\n', "\r\n") } else { text };
            return match std::fs::write(&path, text) {
                Ok(()) => {
                    if let Some(l) = self.lsp.as_ref() {
                        l.did_save(&path);
                    }
                    phai_editor::view::set_clean(h);
                    self.doc.dirty = false;
                    self.say_title();
                    self.refresh_tabs();
                    controls::status_text(self.status, 0, &tf("Saved {0}", &[&path.file_name().unwrap_or_default().to_string_lossy()]));
                    true
                }
                Err(e) => {
                    win::message(Some(self.hwnd), &format!("{}\n\n{e}", tf("{0} cannot be saved", &[&path.display()])), "Phai Studio", wm::MB_ICONWARNING);
                    false
                }
            };
        }
        if let Some((message, _)) = self.doc.form.check().into_iter().next() {
            win::message(Some(self.hwnd), &message, t("The form has a mistake"), wm::MB_ICONWARNING);
            return false;
        }
        let path = match (&self.doc.path, choose) {
            (Some(p), false) => p.clone(),
            _ => match file_dialog(self.hwnd, true, "Phai form (*.phaiform)", "*.phaiform", &format!("{}.phaiform", snake(&self.doc.form.form.name))) {
                Some(p) => p,
                None => return false,
            },
        };
        let f = &self.doc.form;
        let ui_path = path.with_file_name(format!("{}_ui.rs", snake(&f.form.name)));
        let user_path = path.with_file_name(format!("{}.rs", snake(&f.form.name)));
        let opts = theme_options(&path, f.form.theme.as_deref());
        let wrote = std::fs::write(&path, f.to_toml()).and_then(|_| std::fs::write(&ui_path, codegen::generate(f, &opts))).and_then(|_| if user_path.exists() { Ok(()) } else { std::fs::write(&user_path, codegen::user_stub(f)) });
        match wrote {
            Ok(()) => {
                self.doc.path = Some(path.clone());
                self.doc.dirty = false;
                self.say_title();
                self.refresh_tabs();
                if let Some(e) = self.explorer.as_mut() {
                    e.show_project_of(Some(&path));
                }
                let file = |p: &Path| p.file_name().unwrap_or_default().to_string_lossy().to_string();
                controls::status_text(self.status, 0, &tf("Saved {0} and wrote {1}", &[&file(&path), &file(&ui_path)]));
                true
            }
            Err(e) => {
                win::message(Some(self.hwnd), &format!("{}\n\n{e}", tf("{0} cannot be saved", &[&path.display()])), "Phai Studio", wm::MB_ICONWARNING);
                false
            }
        }
    }

    // --- themes -------------------------------------------------------------------------------

    /// The form's theme, read beside the form and watched; the built-in one when it has none.
    fn load_theme(&mut self) {
        let file = match (&self.doc.path, &self.doc.form.form.theme) {
            (Some(p), Some(th)) => Some(p.with_file_name("").join(th)),
            _ => None,
        };
        match file {
            Some(f) if f.exists() => {
                win::watch_theme(self.hwnd, &f);
                controls::status_text(self.status, 0, &tf("Theme: {0}", &[&self.doc.form.form.theme.clone().unwrap_or_default()]));
            }
            _ => {
                win::unwatch_theme(self.hwnd);
                let dark = theme::current().is_dark();
                theme::set(if dark { Theme::dark() } else { Theme::light() });
                self.after_theme();
            }
        }
    }

    fn after_theme(&mut self) {
        unsafe {
            let old = wm::GetMenu(self.hwnd);
            let _ = wm::SetMenu(self.hwnd, Some(build_menu()));
            let _ = wm::DestroyMenu(old);
        }
        win::retheme(self.hwnd);
        self.rebuild();
    }

    fn theme_from_standard(&mut self) {
        let Some(path) = self.doc.path.clone() else {
            win::message(Some(self.hwnd), t("Save the form first, so its theme can sit beside it."), "Phai Studio", wm::MB_ICONINFORMATION);
            return;
        };
        let css = path.with_file_name("theme.css");
        if !css.exists() && std::fs::write(&css, theme::STANDARD_CSS).is_err() {
            return;
        }
        self.edit(|f| f.form.theme = Some("theme.css".into()));
        self.load_theme();
        self.refresh_props();
    }

    fn theme_import(&mut self) {
        let Some(path) = self.doc.path.clone() else {
            win::message(Some(self.hwnd), t("Save the form first, so its theme can sit beside it."), "Phai Studio", wm::MB_ICONINFORMATION);
            return;
        };
        let Some(chosen) = file_dialog(self.hwnd, false, "CSS (*.css)", "*.css", "") else { return };
        let dir = path.with_file_name("");
        // A sheet from elsewhere is copied in, so the project carries everything it is built from.
        let name = chosen.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "theme.css".into());
        if chosen.parent() != Some(dir.as_path()) && std::fs::copy(&chosen, dir.join(&name)).is_err() {
            return;
        }
        self.edit(|f| f.form.theme = Some(name));
        self.load_theme();
        self.refresh_props();
    }

    // --- projects, building and running --------------------------------------------------------

    fn new_project(&mut self) {
        let Some(dir) = project::pick_folder(self.hwnd, t("Choose an empty folder for the new project")) else { return };
        match project::create(&dir) {
            Ok(form) => {
                self.open_path(&form);
                controls::status_text(self.status, 0, &tf("New project in {0}", &[&dir.display()]));
            }
            Err(e) => {
                win::message(Some(self.hwnd), &format!("{}\n\n{e}", tf("{0} cannot be made", &[&dir.display()])), "Phai Studio", wm::MB_ICONWARNING);
            }
        }
    }

    /// Saves, then builds the project the form is in; `then_run` starts the program after.
    fn start_build(&mut self, then_run: bool) {
        if self.building {
            return;
        }
        let Some(path) = self.doc.path.clone() else {
            win::message(Some(self.hwnd), t("Save the form in a project first - or make one with New project."), "Phai Studio", wm::MB_ICONINFORMATION);
            return;
        };
        if self.doc.dirty && !self.save(false) {
            return;
        }
        let Some(dir) = cargo::project_of(&path) else {
            win::message(Some(self.hwnd), t("This form is not in a Cargo project - make one with New project."), "Phai Studio", wm::MB_ICONINFORMATION);
            return;
        };
        if then_run {
            self.stop();
        }
        self.building = true;
        controls::status_text(self.status, 0, t("Building..."));
        if let Some(b) = self.bottom.as_ref() {
            b.clear_output();
            b.show_tab(1);
        }
        self.show_errors(true);
        cargo::build(self.hwnd, dir, then_run, self.release);
    }

    fn stop(&mut self) {
        if let Some(mut child) = self.running.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn show_errors(&mut self, on: bool) {
        self.show_pane(Pane::Output, on);
    }

    /// The layout item of window `h`.
    fn item(&mut self, h: HWND) -> Option<&mut Item> {
        self.layout.items.iter_mut().find(|(w, _)| *w == h).map(|(_, i)| i)
    }

    /// Shows or hides a docked pane, as the View menu and the panes' close buttons do.
    pub fn show_pane(&mut self, pane: Pane, on: bool) {
        match pane {
            Pane::Toolbox => {
                self.toolbox_on = on;
                let tb = self.toolbox;
                if let Some(i) = self.item(tb) {
                    i.bounds.w = if on { win::TOOLBAR_HEIGHT + 4 } else { 0 };
                }
                win::show(tb, on);
            }
            Pane::Explorer | Pane::Properties => {
                let (mut e, mut p) = explorer::side_panes(self.side);
                if pane == Pane::Explorer {
                    e = on;
                } else {
                    p = on;
                }
                let any = explorer::show_side_panes(self.side, e, p);
                let (side, split, w) = (self.side, self.side_split, self.side_w);
                if let Some(i) = self.item(side) {
                    i.bounds.w = if any { w } else { 0 };
                }
                if let Some(i) = self.item(split) {
                    i.bounds.w = if any { 6 } else { 0 };
                }
                win::show(side, any);
                win::show(split, any);
            }
            Pane::Output => {
                self.bottom_on = on;
                let (bh, split, h) = (self.bottom.as_ref().map(|b| b.hwnd).unwrap_or_default(), self.bottom_split, self.bottom_h);
                if let Some(i) = self.item(bh) {
                    i.bounds.h = if on { h } else { 0 };
                }
                if let Some(i) = self.item(split) {
                    i.bounds.h = if on { 6 } else { 0 };
                }
                win::show(bh, on);
                win::show(split, on);
            }
        }
        self.layout.apply(self.hwnd);
        self.rebuild_menu();
    }

    /// A splitter dragged `d` device pixels: the pane beside it takes or gives the room.
    fn split(&mut self, id: i32, d: i32) {
        let d = phai_ui::layout::unscale(d, win::dpi(self.hwnd));
        if id == ID_SPLIT_SIDE {
            self.side_w = (self.side_w - d).clamp(180, 900);
            let (side, w) = (self.side, self.side_w);
            if let Some(i) = self.item(side) {
                i.bounds.w = w;
            }
        } else {
            self.bottom_h = (self.bottom_h - d).clamp(60, 900);
            let (bh, h) = (self.bottom.as_ref().map(|b| b.hwnd).unwrap_or_default(), self.bottom_h);
            if let Some(i) = self.item(bh) {
                i.bounds.h = h;
            }
        }
        self.layout.apply(self.hwnd);
    }

    fn build_done(&mut self, out: cargo::Outcome) {
        use windows::Win32::UI::Controls as cc;
        self.building = false;
        if out.no_cargo {
            controls::status_text(self.status, 0, t("Rust is not installed"));
            let answer = win::message(Some(self.hwnd), t("Building needs Rust, and it is not on this computer. Open rustup.rs to install it?"), "Phai Studio", wm::MB_YESNO | wm::MB_ICONINFORMATION);
            if answer == wm::IDYES {
                open_with_windows("https://rustup.rs");
            }
            return;
        }
        self.diagnostics = out.diagnostics.clone();
        if let Some(e) = &out.cargo_error {
            self.diagnostics.insert(0, cargo::Diagnostic { error: true, message: e.clone(), file: None, line: 0, column: 0 });
        }
        unsafe {
            wm::SendMessageW(self.errors, cc::LVM_DELETEALLITEMS, None, None);
        }
        for d in &self.diagnostics {
            let file = d.file.as_ref().and_then(|f| f.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            let line = if d.line > 0 { d.line.to_string() } else { String::new() };
            let first = d.message.lines().next().unwrap_or_default();
            win::list_add(self.errors, &[if d.error { t("Error") } else { t("Warning") }, first, &file, &line]);
        }
        let errors = self.diagnostics.iter().filter(|d| d.error).count();
        let warnings = self.diagnostics.len() - errors;
        // Problems show their list; a clean build leaves its output on screen.
        if let Some(b) = self.bottom.as_ref() {
            b.add_line(&format!("{} ({:.1} s)", if out.ok { t("Build succeeded") } else { t("Build failed") }, out.seconds));
            if !self.diagnostics.is_empty() {
                b.show_tab(0);
            }
        }
        if out.ok {
            controls::status_text(self.status, 0, &tf("Built in {0} s - {1} warnings", &[&format!("{:.1}", out.seconds), &warnings]));
            if out.then_run {
                match out.executable.as_deref().map(cargo::run) {
                    Some(Ok(child)) => {
                        self.running = Some(child);
                        controls::status_text(self.status, 0, &tf("Running {0}", &[&out.executable.as_ref().and_then(|e| e.file_name()).unwrap_or_default().to_string_lossy()]));
                    }
                    Some(Err(e)) => {
                        controls::status_text(self.status, 0, &e.to_string());
                    }
                    None => {
                        controls::status_text(self.status, 0, t("The project has no program to run"));
                    }
                }
            }
        } else {
            controls::status_text(self.status, 0, &tf("Build failed - {0} errors, {1} warnings", &[&errors.max(1), &warnings]));
        }
    }

    /// Double-click in the error list: the file, opened as Windows opens it.
    fn open_diagnostic(&mut self, row: usize) {
        let Some(d) = self.diagnostics.get(row) else { return };
        let Some(file) = d.file.clone() else { return };
        let (line, column) = (d.line as usize, d.column as usize);
        controls::status_text(self.status, 0, &format!("{}:{line}:{column}", file.display()));
        // Into Phai's own editor, at the line.
        self.open_code(&file);
        if let Some(h) = self.doc.code {
            phai_editor::view::go_to(h, line, column);
        }
    }

    // --- commands -----------------------------------------------------------------------------

    fn command(&mut self, id: i32) {
        match id {
            ID_NEW => self.open_in_tab(FormFile::new("MainWindow"), None),
            ID_OPEN => {
                if let Some(p) = file_dialog(self.hwnd, false, "Phai form (*.phaiform)", "*.phaiform", "") {
                    self.open_path(&p);
                }
            }
            ID_VIEW_TOOLBOX => self.show_pane(Pane::Toolbox, !self.toolbox_on),
            ID_VIEW_EXPLORER => {
                let on = !explorer::side_panes(self.side).0;
                self.show_pane(Pane::Explorer, on);
            }
            ID_VIEW_PROPS => {
                let on = !explorer::side_panes(self.side).1;
                self.show_pane(Pane::Properties, on);
            }
            ID_VIEW_OUTPUT => self.show_pane(Pane::Output, !self.bottom_on),
            ID_EDIT_MENUS => self.edit_menus(),
            ID_CUT => {
                self.copy();
                self.delete_selected();
            }
            ID_COPY => self.copy(),
            ID_PASTE => self.paste(),
            ID_DUPLICATE => self.duplicate(),
            ID_SELECT_ALL => {
                let all: Vec<usize> = (0..self.doc.form.control.len()).collect();
                self.select_many(&all);
            }
            f if (ID_FORMAT..ID_FORMAT + FORMATS.len() as i32).contains(&f) => self.format(FORMATS[(f - ID_FORMAT) as usize]),
            ID_EDIT_TOOLBAR => self.edit_toolbar(),
            ID_FRONT | ID_BACK => self.restack(id == ID_FRONT),
            ID_LOCK => {
                if let Some(c) = self.canvas.as_mut() {
                    c.locked = !c.locked;
                    let on = c.locked;
                    controls::status_text(self.status, 0, if on { t("Controls are locked") } else { t("Controls are unlocked") });
                }
                self.rebuild_menu();
            }
            ID_TAB_ORDER => {
                if let Some(c) = self.canvas.as_mut() {
                    let on = !c.tab_mode();
                    c.set_tab_mode(on);
                    if on {
                        controls::status_text(self.status, 0, t("Click the controls in the order Tab should visit them; Esc ends"));
                    }
                }
                self.rebuild_menu();
            }
            ID_CLOSE => {
                self.close_tab();
            }
            ID_TABS => {}
            t if (ID_TOOL..=ID_TOOL + Kind::ALL.len() as i32).contains(&t) => {
                self.tool = if t == ID_TOOL { None } else { Kind::ALL.get((t - ID_TOOL - 1) as usize).copied() };
            }
            ID_SAVE => {
                self.save(false);
            }
            ID_SAVE_AS => {
                self.save(true);
            }
            ID_EXIT => {
                if self.may_quit() {
                    unsafe {
                        let _ = wm::DestroyWindow(self.hwnd);
                    }
                }
            }
            ID_UNDO | ID_REDO if self.doc.code.is_some() => {
                if let Some(h) = self.doc.code {
                    phai_editor::view::undo_redo(h, id == ID_UNDO);
                }
            }
            ID_UNDO => self.undo_redo(true),
            ID_REDO => self.undo_redo(false),
            ID_DELETE => self.delete_selected(),
            ID_PREVIEW => preview::open(self.hwnd, &self.doc.form),
            ID_LIGHT | ID_DARK | ID_THEME_TOGGLE => {
                win::unwatch_theme(self.hwnd);
                let dark = if id == ID_THEME_TOGGLE { !theme::current().is_dark() } else { id == ID_DARK };
                theme::set(if dark { Theme::dark() } else { Theme::light() });
                self.after_theme();
            }
            ID_THAI | ID_ENGLISH | ID_LANG => {
                words::set_thai(if id == ID_LANG { !words::is_thai() } else { id == ID_THAI });
                self.relabel();
            }
            ID_THEME_NEW => self.theme_from_standard(),
            ID_THEME_IMPORT => self.theme_import(),
            ID_NEW_PROJECT => self.new_project(),
            ID_BUILD => self.start_build(false),
            ID_RUN => self.start_build(true),
            ID_STOP => self.stop(),
            n if (ID_TEMPLATE..ID_TEMPLATE + phai_design::templates::Template::ALL.len() as i32).contains(&n) => {
                let tpl = phai_design::templates::Template::ALL[(n - ID_TEMPLATE) as usize];
                let name = match tpl {
                    phai_design::templates::Template::Dialog => "AskDialog",
                    phai_design::templates::Template::MainWindow => "MainWindow",
                    phai_design::templates::Template::DataEntry => "DetailsWindow",
                };
                self.open_in_tab(tpl.make(name), None);
            }
            r if (ID_RECENT..ID_RECENT + RECENT_MAX as i32).contains(&r) => {
                if let Some(p) = recent_files().get((r - ID_RECENT) as usize).cloned() {
                    self.open_path(&p);
                }
            }
            ID_DEBUG | ID_RELEASE => {
                self.release = id == ID_RELEASE;
                controls::status_text(self.status, 2, if self.release { "Release" } else { "Debug" });
                unsafe {
                    let old = wm::GetMenu(self.hwnd);
                    let _ = wm::SetMenu(self.hwnd, Some(build_menu()));
                    let _ = wm::DestroyMenu(old);
                }
            }
            ID_THEME_NONE => {
                self.edit(|f| f.form.theme = None);
                self.load_theme();
                self.refresh_props();
            }
            _ => {}
        }
    }

    fn rebuild_menu(&self) {
        unsafe {
            let old = wm::GetMenu(self.hwnd);
            let _ = wm::SetMenu(self.hwnd, Some(build_menu()));
            let _ = wm::DestroyMenu(old);
        }
    }

    fn relabel(&mut self) {
        win::set_toolbar(self.toolbar, &toolbar_items());
        win::set_toolbar(self.toolbox, &toolbox_items());
        win::toolbar_choose(self.toolbox, self.tool.and_then(|k| Kind::ALL.iter().position(|x| *x == k)).map(|i| ID_TOOL + 1 + i as i32).unwrap_or(ID_TOOL));
        unsafe {
            let old = wm::GetMenu(self.hwnd);
            let _ = wm::SetMenu(self.hwnd, Some(build_menu()));
            let _ = wm::DestroyMenu(old);
        }
        if let Some(p) = self.props.as_mut() {
            p.relabel();
        }
        if let Some(e) = self.explorer.as_mut() {
            e.relabel();
        }
        if let Some(b) = self.bottom.as_ref() {
            b.relabel();
        }
        explorer::relabel_side(self.side);
        self.refresh_tabs();
        controls::status_text(self.status, 0, t("Ready"));
        self.say_title();
        self.rebuild();
    }
}

/// Where the theme is for the generated code: beside the form for `include_str!`, and from the
/// crate's `Cargo.toml` for a debug build to watch.
pub fn theme_options(form: &Path, theme_file: Option<&str>) -> codegen::Options {
    let Some(th) = theme_file else { return codegen::Options::default() };
    let dir = form.with_file_name("");
    let watch = dir.ancestors().find(|d| d.join("Cargo.toml").exists()).and_then(|root| dir.join(th).strip_prefix(root).ok().map(|p| p.to_string_lossy().replace('\\', "/")));
    codegen::Options { theme_include: Some(th.replace('\\', "/")), theme_watch: watch }
}

/// Where Phai Studio keeps what it remembers for this person: `%APPDATA%\Phai Studio`.
fn settings_dir() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("APPDATA")?).join("Phai Studio"))
}

/// A remembered setting, `name=value` lines in `settings.txt`.
fn setting(name: &str) -> Option<String> {
    let text = std::fs::read_to_string(settings_dir()?.join("settings.txt")).ok()?;
    text.lines().find_map(|l| l.strip_prefix(&format!("{name}=")).map(str::to_string))
}

fn remember_setting(name: &str, value: &str) {
    let Some(dir) = settings_dir() else { return };
    let path = dir.join("settings.txt");
    let mut lines: Vec<String> = std::fs::read_to_string(&path).unwrap_or_default().lines().filter(|l| !l.starts_with(&format!("{name}="))).map(str::to_string).collect();
    lines.push(format!("{name}={value}"));
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(path, lines.join("\n"));
}

/// The forms opened last, newest first, those still there.
fn recent_files() -> Vec<PathBuf> {
    let Some(dir) = settings_dir() else { return vec![] };
    std::fs::read_to_string(dir.join("recent.txt")).unwrap_or_default().lines().map(PathBuf::from).filter(|p| p.is_file()).take(RECENT_MAX).collect()
}

fn remember_recent(path: &Path) {
    let Some(dir) = settings_dir() else { return };
    let path = path.canonicalize().map(|p| PathBuf::from(p.to_string_lossy().trim_start_matches(r"\\?\"))).unwrap_or_else(|_| path.to_path_buf());
    let mut list = recent_files();
    list.retain(|p| p != &path);
    list.insert(0, path);
    list.truncate(RECENT_MAX);
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join("recent.txt"), list.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("\n"));
}

/// Puts `text` on the clipboard.
fn set_clipboard(owner: HWND, text: &str) {
    use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData};
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    let w: Vec<u16> = text.replace('\n', "\r\n").encode_utf16().chain(Some(0)).collect();
    unsafe {
        if OpenClipboard(Some(owner)).is_err() {
            return;
        }
        let _ = EmptyClipboard();
        if let Ok(mem) = GlobalAlloc(GMEM_MOVEABLE, w.len() * 2) {
            let p = GlobalLock(mem) as *mut u16;
            if !p.is_null() {
                std::ptr::copy_nonoverlapping(w.as_ptr(), p, w.len());
                let _ = GlobalUnlock(mem);
                let _ = SetClipboardData(13 /* CF_UNICODETEXT */, Some(windows::Win32::Foundation::HANDLE(mem.0)));
            }
        }
        let _ = CloseClipboard();
    }
}

/// The clipboard's text, if it holds some.
fn get_clipboard(owner: HWND) -> Option<String> {
    use windows::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
    use windows::Win32::Foundation::HGLOBAL;
    use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
    unsafe {
        OpenClipboard(Some(owner)).ok()?;
        let out = (|| {
            let h = GetClipboardData(13).ok()?;
            let mem = HGLOBAL(h.0);
            let p = GlobalLock(mem) as *const u16;
            if p.is_null() {
                return None;
            }
            let mut n = 0;
            while *p.add(n) != 0 {
                n += 1;
            }
            let s = String::from_utf16_lossy(std::slice::from_raw_parts(p, n));
            let _ = GlobalUnlock(mem);
            Some(s.replace("\r\n", "\n"))
        })();
        let _ = CloseClipboard();
        out
    }
}

/// A file or web address, opened by whatever Windows opens it with.
pub fn open_with_windows(what: &str) {
    let w = wide(what);
    unsafe {
        windows::Win32::UI::Shell::ShellExecuteW(None, windows::core::w!("open"), PCWSTR(w.as_ptr()), PCWSTR::null(), PCWSTR::null(), wm::SW_SHOWNORMAL);
    }
}

/// Asks for a file to open or a place to save.
fn file_dialog(owner: HWND, save: bool, description: &str, pattern: &str, suggested: &str) -> Option<PathBuf> {
    use windows::Win32::UI::Controls::Dialogs as dlg;
    let filter: Vec<u16> = [description, pattern, ""].iter().flat_map(|s| s.encode_utf16().chain(Some(0))).collect();
    let mut file = [0u16; 1024];
    for (i, u) in suggested.encode_utf16().take(1000).enumerate() {
        file[i] = u;
    }
    let ext = wide(pattern.trim_start_matches("*."));
    let mut ofn = dlg::OPENFILENAMEW {
        lStructSize: std::mem::size_of::<dlg::OPENFILENAMEW>() as u32,
        hwndOwner: owner,
        lpstrFilter: PCWSTR(filter.as_ptr()),
        lpstrFile: windows::core::PWSTR(file.as_mut_ptr()),
        nMaxFile: file.len() as u32,
        lpstrDefExt: PCWSTR(ext.as_ptr()),
        Flags: if save { dlg::OFN_OVERWRITEPROMPT | dlg::OFN_PATHMUSTEXIST } else { dlg::OFN_FILEMUSTEXIST | dlg::OFN_PATHMUSTEXIST },
        ..Default::default()
    };
    let ok = unsafe { if save { dlg::GetSaveFileNameW(&mut ofn) } else { dlg::GetOpenFileNameW(&mut ofn) } }.as_bool();
    ok.then(|| {
        let n = file.iter().position(|c| *c == 0).unwrap_or(0);
        PathBuf::from(String::from_utf16_lossy(&file[..n]))
    })
}

extern "system" fn proc_(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(s) = studio().filter(|s| s.hwnd == hwnd) else { return win::def(hwnd, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            s.layout.apply(hwnd);
            LRESULT(0)
        }
        phai_ui::controls::WM_SPLIT => {
            s.split(w.0 as i32, l.0 as i32);
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            let id = win::loword(w.0) as i32;
            if id == ID_TABS {
                s.activate(win::hiword(w.0) as usize);
            } else {
                s.command(id);
            }
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            use windows::Win32::UI::Controls as cc;
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.hwndFrom == s.toolbox && hdr.code == cc::NM_DBLCLK && s.doc.code.is_none() {
                let m = unsafe { &*(l.0 as *const cc::NMMOUSE) };
                let id = m.dwItemSpec as i32;
                if let Some(kind) = (id > ID_TOOL).then(|| Kind::ALL.get((id - ID_TOOL - 1) as usize).copied()).flatten() {
                    s.add_at_margin(kind);
                    return LRESULT(0);
                }
            }
            if hdr.hwndFrom == s.errors && hdr.code == cc::NM_DBLCLK {
                let it = unsafe { &*(l.0 as *const cc::NMITEMACTIVATE) };
                if it.iItem >= 0 {
                    s.open_diagnostic(it.iItem as usize);
                }
            }
            win::def(hwnd, msg, w, l)
        }
        phai_editor::lsp::WM_LSP => {
            let msg = unsafe { Box::from_raw(l.0 as *mut serde_json::Value) };
            s.lsp_message(*msg);
            LRESULT(0)
        }
        cargo::WM_BUILD_LINE => {
            let line = unsafe { Box::from_raw(l.0 as *mut String) };
            if let Some(b) = s.bottom.as_ref() {
                b.add_line(&line);
            }
            LRESULT(0)
        }
        cargo::WM_BUILD_DONE => {
            let out = unsafe { Box::from_raw(l.0 as *mut cargo::Outcome) };
            if BUILD_TEST.load(Ordering::Relaxed) && std::env::var_os("PHAI_SHOT").is_none() {
                println!("ok={} seconds={:.1} exe={:?} no_cargo={} cargo_error={:?}", out.ok, out.seconds, out.executable, out.no_cargo, out.cargo_error);
                for d in &out.diagnostics {
                    println!("{} {}:{}:{} {}", if d.error { "error" } else { "warning" }, d.file.as_ref().map(|f| f.display().to_string()).unwrap_or_default(), d.line, d.column, d.message);
                }
                std::process::exit(if out.ok { 0 } else { 2 });
            }
            s.build_done(*out);
            LRESULT(0)
        }
        win::WM_THEME_CHANGED => {
            if w.0 == 1 {
                s.after_theme();
            } else if let Some(e) = win::theme_error() {
                controls::status_text(s.status, 0, &e);
                win::message(Some(hwnd), &e, t("Theme file has a mistake"), wm::MB_ICONWARNING);
            }
            LRESULT(0)
        }
        0x02E0 => {
            // WM_DPICHANGED
            let r = unsafe { &*(l.0 as *const RECT) };
            unsafe {
                let _ = wm::SetWindowPos(hwnd, None, r.left, r.top, r.right - r.left, r.bottom - r.top, wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
                let _ = Gdi::DeleteObject(s.font.into());
            }
            s.font = win::font(hwnd, 0, false);
            win::set_font_all(hwnd, s.font);
            win::retheme(hwnd);
            s.layout.apply(hwnd);
            s.rebuild();
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            if s.may_quit() {
                unsafe {
                    let _ = wm::DestroyWindow(hwnd);
                }
            }
            LRESULT(0)
        }
        wm::WM_DESTROY => {
            s.stop();
            remember_setting("editor_font_pt", &phai_editor::view::font_size().to_string());
            unsafe { wm::PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => win::def(hwnd, msg, w, l),
    }
}

/// Shortcuts, before a box can take the key: Ctrl+N/O/S/Z/Y and F5.
fn shortcut(msg: &wm::MSG) -> Option<i32> {
    if msg.message != wm::WM_KEYDOWN {
        return None;
    }
    let ctrl = unsafe { km::GetKeyState(0x11) } < 0;
    let in_box = !unsafe { km::GetFocus() }.0.is_null() && win::kind_of(unsafe { km::GetFocus() }) == Some(win::Kind::Edit);
    match (msg.wParam.0 as u8, ctrl) {
        (b'N', true) => Some(ID_NEW),
        (b'O', true) => Some(ID_OPEN),
        (b'W', true) => Some(ID_CLOSE),
        (b'S', true) => Some(ID_SAVE),
        // In a box Ctrl+Z is the box's own undo.
        (b'Z', true) if !in_box => Some(ID_UNDO),
        (b'Y', true) if !in_box => Some(ID_REDO),
        (b'C', true) if !in_box => Some(ID_COPY),
        (b'X', true) if !in_box => Some(ID_CUT),
        (b'V', true) if !in_box => Some(ID_PASTE),
        (b'D', true) if !in_box => Some(ID_DUPLICATE),
        (b'A', true) if !in_box => Some(ID_SELECT_ALL),
        (b'B', true) => Some(ID_BUILD),
        (0x73, _) => Some(ID_PREVIEW),
        (0x74, _) if unsafe { km::GetKeyState(0x10) } < 0 => Some(ID_STOP),
        (0x74, _) => Some(ID_RUN),
        _ => None,
    }
}

/// Debug builds: a crash says what and where, with a backtrace, instead of vanishing.
#[cfg(debug_assertions)]
fn report_crashes() {
    use windows::Win32::System::Diagnostics::Debug::{AddVectoredExceptionHandler, EXCEPTION_POINTERS};
    unsafe extern "system" fn handler(info: *mut EXCEPTION_POINTERS) -> i32 {
        let rec = unsafe { &*(*info).ExceptionRecord };
        let code = rec.ExceptionCode.0 as u32;
        // Access violations, stack overflows and the like; not the exceptions Windows uses itself.
        if matches!(code, 0xC0000005 | 0xC00000FD | 0xC0000409 | 0xC000001D) {
            eprintln!("crash {code:#x} at {:?}
{}", rec.ExceptionAddress, std::backtrace::Backtrace::force_capture());
        }
        0 // EXCEPTION_CONTINUE_SEARCH
    }
    unsafe {
        AddVectoredExceptionHandler(1, Some(handler));
    }
}

/// `--selftest-multi out.phaiform`: chooses several controls with a band, lines them up, makes
/// a copy and saves - checked by what lands in the file.
fn selftest_multi(out: &Path) {
    let Some(s) = studio() else { return };
    let Some(surface) = s.canvas.as_ref().map(|c| c.surface) else { return };
    let at = |x: i32, y: i32| LPARAM(((win::px(surface, y) as isize) << 16) | (win::px(surface, x) as isize & 0xFFFF));
    let send = |msg: u32, l: LPARAM| unsafe {
        wm::SendMessageW(surface, msg, Some(WPARAM(1)), Some(l));
    };
    // A band over the two buttons at the bottom right of the sample form.
    send(wm::WM_LBUTTONDOWN, at(300, 298));
    send(wm::WM_MOUSEMOVE, at(470, 330));
    send(wm::WM_LBUTTONUP, at(470, 330));
    println!("chosen: {:?}", s.selection().iter().map(|i| s.doc.form.control[*i].id.clone()).collect::<Vec<_>>());
    s.format(phai_form::arrange::Op::SameWidth);
    s.duplicate();
    println!("after duplicate: {:?}", s.selection().iter().map(|i| s.doc.form.control[*i].id.clone()).collect::<Vec<_>>());
    s.doc.path = Some(out.to_path_buf());
    s.save(false);
}

/// See `--selftest` in `main`.
fn selftest(out: &Path) {
    let Some(s) = studio() else { return };
    let Some(surface) = s.canvas.as_ref().map(|c| c.surface) else { return };
    let at = |x: i32, y: i32| LPARAM(((win::px(surface, y) as isize) << 16) | (win::px(surface, x) as isize & 0xFFFF));
    let send = |msg: u32, l: LPARAM| unsafe {
        wm::SendMessageW(surface, msg, Some(WPARAM(1)), Some(l));
    };
    // Draw a button from (40, 300) to (140, 324).
    s.tool = Some(Kind::Button);
    send(wm::WM_LBUTTONDOWN, at(40, 300));
    send(wm::WM_MOUSEMOVE, at(100, 312));
    send(wm::WM_MOUSEMOVE, at(140, 324));
    send(wm::WM_LBUTTONUP, at(140, 324));
    // Move it 20 right, 8 down, by grabbing its middle.
    send(wm::WM_LBUTTONDOWN, at(90, 312));
    send(wm::WM_MOUSEMOVE, at(110, 320));
    send(wm::WM_LBUTTONUP, at(110, 320));
    // Widen it from its right-middle handle by 24.
    send(wm::WM_LBUTTONDOWN, at(160, 320));
    send(wm::WM_MOUSEMOVE, at(184, 320));
    send(wm::WM_LBUTTONUP, at(184, 320));
    // Give it words and a handler, as the properties pane would.
    s.apply(Key::Text, "Print");
    s.add_handler_for_selected();
    let path = out.to_path_buf();
    s.doc.path = Some(path);
    s.save(false);
}

fn main() {
    #[cfg(debug_assertions)]
    report_crashes();
    let args: Vec<String> = std::env::args().collect();
    words::add(text::THAI);
    phai_form::live::set_translate(false);
    if let Some(pt) = setting("editor_font_pt").and_then(|v| v.parse().ok()) {
        phai_editor::view::set_font_size(pt);
    }
    words::set_thai(args.iter().any(|a| a == "--thai"));
    if args.iter().any(|a| a == "--dark") {
        theme::set(Theme::dark());
    }
    win::init();
    let icon = phai_ui::icon::glyph_icon(glyph::APPS, 32, theme::Color::rgb(0x12, 0x85, 0x5f));
    let class = win::register("PhaiStudio", Some(proc_), icon);
    let title = wide("Phai Studio");
    unsafe {
        let hwnd = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_OVERLAPPEDWINDOW | wm::WS_CLIPCHILDREN, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, 100, 100, None, Some(build_menu()), Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let toolbar = win::toolbar(hwnd, ID_TOOLBAR, &toolbar_items());
        let toolbox = win::toolbar_column(hwnd, ID_TOOLBOX, &toolbox_items());
        let tabbar = controls::tabs(hwnd, ID_TABS, font, &[t("Untitled")]);
        let status = controls::status_bar(hwnd, ID_STATUS, font);
        let bottom = bottom::Bottom::new(hwnd, font, ID_ERRORS);
        let errors = bottom.errors;
        let bottom_hwnd = bottom.hwnd;
        let bottom_split = controls::splitter(hwnd, ID_SPLIT_BOTTOM, false);
        win::show(bottom_split, false);
        controls::status_parts(status, &[260, 80]);
        controls::status_text(status, 2, "Debug");
        controls::status_text(status, 0, t("Ready"));
        let dpi = win::dpi(hwnd);
        let status_h = phai_ui::layout::unscale(controls::status_height(status), dpi);
        let mut layout = Layout::new((1100, 700));
        layout
            .add(toolbar, Item { bounds: Rect::new(0, 0, 0, win::TOOLBAR_HEIGHT), dock: Dock::Top, anchor: Anchor::default() })
            .add(status, Item { bounds: Rect::new(0, 0, 0, status_h), dock: Dock::Bottom, anchor: Anchor::default() })
            .add(bottom_hwnd, Item { bounds: Rect::new(0, 0, 0, 0), dock: Dock::Bottom, anchor: Anchor::default() })
            .add(bottom_split, Item { bounds: Rect::new(0, 0, 0, 0), dock: Dock::Bottom, anchor: Anchor::default() })
            .add(toolbox, Item { bounds: Rect::new(0, 0, win::TOOLBAR_HEIGHT + 4, 0), dock: Dock::Left, anchor: Anchor::default() });
        let studio_box = Box::new(Studio {
            hwnd,
            font,
            toolbar,
            toolbox,
            status,
            errors,
            bottom: Some(bottom),
            release: false,
            diagnostics: vec![],
            building: false,
            running: None,
            lsp: None,
            asked: std::collections::HashMap::new(),
            canvas: None,
            side: HWND::default(),
            side_split: HWND::default(),
            bottom_split: HWND::default(),
            side_w: 280,
            bottom_h: ERRORS_HEIGHT,
            toolbox_on: true,
            bottom_on: false,
            props: None,
            explorer: None,
            tabbar,
            tabs: vec![Tab::empty()],
            active: 0,
            layout,
            doc: Doc::new(FormFile::new("MainWindow"), None),
            undo: vec![],
            redo: vec![],
            selected: None,
            also: vec![],
            tool: None,
        });
        STUDIO.store(Box::into_raw(studio_box), Ordering::Relaxed);
        let s = studio().unwrap();
        // The right side: the project's files above, the chosen thing's properties below.
        let side = explorer::side_panel(hwnd);
        let explorer = explorer::Explorer::new(side, font);
        let props = Props::new(side, ID_PROPS, font);
        explorer::set_side_parts(side, explorer.hwnd, explorer.tree, props.hwnd, font);
        let canvas = Canvas::new(hwnd, ID_CANVAS);
        let side_split = controls::splitter(hwnd, ID_SPLIT_SIDE, true);
        s.side = side;
        s.side_split = side_split;
        s.bottom_split = bottom_split;
        s.layout.add(side, Item { bounds: Rect::new(0, 0, 280, 0), dock: Dock::Right, anchor: Anchor::default() });
        s.layout.add(side_split, Item { bounds: Rect::new(0, 0, 6, 0), dock: Dock::Right, anchor: Anchor::default() });
        s.layout.add(tabbar, Item { bounds: Rect::new(0, 0, 0, 30), dock: Dock::Top, anchor: Anchor::default() });
        s.layout.add(canvas.hwnd, Item { bounds: Rect::default(), dock: Dock::Fill, anchor: Anchor::default() });
        s.props = Some(props);
        s.explorer = Some(explorer);
        win::toolbar_choose(toolbox, ID_TOOL);
        s.canvas = Some(canvas);
        win::theme_frame(hwnd);
        let mut r = RECT { left: 0, top: 0, right: win::px(hwnd, 1100), bottom: win::px(hwnd, 700) };
        let st = wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32;
        let _ = windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(st), true, Default::default(), dpi as u32);
        let _ = wm::SetWindowPos(hwnd, None, 0, 0, r.right - r.left, r.bottom - r.top, wm::SWP_NOMOVE | wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
        // Every form named on the command line, each in its tab; the first one shown. A value
        // after a `--flag` that takes one is not a form.
        let forms: Vec<&String> = args.iter().enumerate().skip(1).filter(|(i, a)| !a.starts_with("--") && !["--selftest", "--selftest-multi", "--tab", "--command", "--select", "--find"].contains(&args[i - 1].as_str())).map(|(_, a)| a).collect();
        if forms.is_empty() {
            s.set_doc(FormFile::new("MainWindow"), None);
        }
        for p in &forms {
            if p.ends_with(".phaiform") {
                s.open_path(Path::new(p));
            } else {
                s.open_code(Path::new(p));
            }
        }
        if forms.len() > 1 {
            s.activate(0);
        }
        // `--command N` (more than once) does what menu command N does - for checking by picture.
        for (i, a) in args.iter().enumerate() {
            if a == "--command" {
                if let Some(id) = args.get(i + 1).and_then(|n| n.parse::<usize>().ok()) {
                    let _ = wm::PostMessageW(Some(hwnd), wm::WM_COMMAND, WPARAM(id), LPARAM(0));
                }
            }
        }
        // `--select N` chooses control N; `--events` shows the Events view - for checking by picture.
        if let Some(n) = args.iter().position(|a| a == "--select").and_then(|i| args.get(i + 1)).and_then(|n| n.parse::<usize>().ok()) {
            s.select(Some(n));
        }
        if args.iter().any(|a| a == "--events") {
            if let Some(p) = s.props.as_mut() {
                p.set_mode(1);
            }
        }
        // `--find WORD` opens the code editor's find bar on it (with replace when `--replace`).
        if let (Some(q), Some(h)) = (args.iter().position(|a| a == "--find").and_then(|i| args.get(i + 1)), s.doc.code) {
            phai_editor::view::find_text(h, q, args.iter().any(|a| a == "--replace"));
        }
        // `--menus` opens the menu editor at once - for checking it by its picture.
        if args.iter().any(|a| a == "--menus") {
            let _ = wm::PostMessageW(Some(hwnd), wm::WM_COMMAND, WPARAM(ID_EDIT_MENUS as usize), LPARAM(0));
        }
        // `--tab N` shows tab N, the way a click on it does - for checking the switch.
        if let Some(n) = args.iter().position(|a| a == "--tab").and_then(|i| args.get(i + 1)).and_then(|n| n.parse::<usize>().ok()) {
            wm::SendMessageW(hwnd, wm::WM_COMMAND, Some(WPARAM((n << 16) | ID_TABS as usize)), Some(LPARAM(0)));
        }
        win::show_top(hwnd);
        s.layout.apply(hwnd);
        s.rebuild();
        s.refresh_props();
        // `--selftest out.phaiform`: draws, moves and sizes a control with the mouse messages a
        // `--snaptest`: drags the sample's Save button a little off its row and leaves it held,
        // so a picture shows the snapline it is drawn to.
        if args.iter().any(|a| a == "--snaptest") {
            if let Some(surface) = s.canvas.as_ref().map(|c| c.surface) {
                let at = |x: i32, y: i32| LPARAM(((win::px(surface, y) as isize) << 16) | (win::px(surface, x) as isize & 0xFFFF));
                wm::SendMessageW(surface, wm::WM_LBUTTONDOWN, Some(WPARAM(1)), Some(at(340, 312)));
                wm::SendMessageW(surface, wm::WM_MOUSEMOVE, Some(WPARAM(1)), Some(at(300, 290)));
                wm::SendMessageW(surface, wm::WM_MOUSEMOVE, Some(WPARAM(1)), Some(at(260, 315)));
            }
        }
        // person's hand would send, then saves - the designer checked end to end.
        if let Some(i) = args.iter().position(|a| a == "--selftest") {
            selftest(Path::new(args.get(i + 1).map(String::as_str).unwrap_or("selftest.phaiform")));
        }
        if let Some(i) = args.iter().position(|a| a == "--selftest-multi") {
            selftest_multi(Path::new(args.get(i + 1).map(String::as_str).unwrap_or("multi.phaiform")));
        }
        // `--codetest`: types into the open code file as a keyboard would - Thai, a new line, a
        // backspace - then saves it.
        if args.iter().any(|a| a == "--codetest") {
            if let Some(h) = s.doc.code {
                for u in "// ทดสอบ พิมพ์ไทยx".encode_utf16() {
                    wm::SendMessageW(h, wm::WM_CHAR, Some(WPARAM(u as usize)), Some(LPARAM(1)));
                }
                wm::SendMessageW(h, wm::WM_CHAR, Some(WPARAM(0x08)), Some(LPARAM(1)));
                wm::SendMessageW(h, wm::WM_CHAR, Some(WPARAM(0x0D)), Some(LPARAM(1)));
                s.save(false);
            }
        }
        // `--buildtest`: builds the open form's project the way Build does, prints what came of
        // it and ends - the Build command checked end to end.
        if args.iter().any(|a| a == "--buildtest") {
            BUILD_TEST.store(true, Ordering::Relaxed);
            s.start_build(false);
        }
        // The loop: shortcuts first, then Tab between boxes, then everything else.
        if std::env::var("PHAI_SHOT").is_ok() {
            win::run_loop(Some(hwnd));
            return;
        }
        let quiet = std::env::var_os("PHAI_SHOT").is_some();
        let mut msg = wm::MSG::default();
        while wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
            // Being photographed, a program takes no keys and no mouse: a person typing or moving the
            // mouse elsewhere must not reach it. Tests send theirs straight to the window, past this queue.
            if quiet && ((wm::WM_KEYFIRST..=wm::WM_KEYLAST).contains(&msg.message) || (wm::WM_MOUSEFIRST..=wm::WM_MOUSELAST).contains(&msg.message)) {
                continue;
            }
            if let Some(cmd) = shortcut(&msg) {
                if let Some(s) = studio() {
                    s.command(cmd);
                }
                continue;
            }
            if wm::IsDialogMessageW(hwnd, &msg).as_bool() {
                continue;
            }
            let _ = wm::TranslateMessage(&msg);
            wm::DispatchMessageW(&msg);
        }
    }
}
