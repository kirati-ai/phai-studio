//! **Phai Studio** - design Windows programs in Rust: drag controls onto a form, and get real
//! Win32 Rust that is small, needs no runtime and shows Thai right.
//!
//! The window: a toolbox on the left, the form on its canvas in the middle, the chosen control's
//! properties on the right. Saving writes the `.phaiform`, the `<name>_ui.rs` generated from it,
//! and - once - the programmer's `<name>.rs`.
//!
//! `phai-studio [form.phaiform] [--dark] [--thai]`; `PHAI_SHOT=out.png` photographs it and ends.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod canvas;
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
const ID_THEME_NEW: i32 = 130;
const ID_THEME_IMPORT: i32 = 131;
const ID_THEME_NONE: i32 = 132;

const ID_TOOLBAR: i32 = 200;
const ID_TOOLBOX: i32 = 201;
const ID_CANVAS: i32 = 202;
const ID_PROPS: i32 = 203;
const ID_STATUS: i32 = 204;

pub struct Doc {
    pub form: FormFile,
    pub path: Option<PathBuf>,
    pub dirty: bool,
}

pub struct Studio {
    pub hwnd: HWND,
    pub font: HFONT,
    pub toolbar: HWND,
    pub toolbox: HWND,
    pub status: HWND,
    pub canvas: Option<Canvas>,
    pub props: Option<Props>,
    layout: Layout,
    pub doc: Doc,
    undo: Vec<FormFile>,
    redo: Vec<FormFile>,
    pub selected: Option<usize>,
    pub tool: Option<Kind>,
}

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
    }
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
        (ID_PREVIEW, t("Preview"), glyph::PLAY),
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
        let sub = |label: &str, m: HMENU| {
            let w = wide(label);
            let _ = wm::AppendMenuW(bar, wm::MF_POPUP, m.0 as usize, PCWSTR(w.as_ptr()));
        };
        let file = wm::CreatePopupMenu().unwrap_or_default();
        add(file, ID_NEW, t("New"), "Ctrl+N");
        add(file, ID_OPEN, t("Open..."), "Ctrl+O");
        add(file, ID_SAVE, t("Save"), "Ctrl+S");
        add(file, ID_SAVE_AS, t("Save as..."), "");
        sep(file);
        add(file, ID_EXIT, t("Exit"), "");
        sub(t("&File"), file);
        let edit = wm::CreatePopupMenu().unwrap_or_default();
        add(edit, ID_UNDO, t("Undo"), "Ctrl+Z");
        add(edit, ID_REDO, t("Redo"), "Ctrl+Y");
        sep(edit);
        add(edit, ID_DELETE, t("Delete"), "Del");
        sub(t("&Edit"), edit);
        let view = wm::CreatePopupMenu().unwrap_or_default();
        add(view, ID_PREVIEW, t("Preview"), "F5");
        sep(view);
        add(view, ID_LIGHT, t("Light"), "");
        add(view, ID_DARK, t("Dark"), "");
        sep(view);
        add(view, ID_THAI, "ไทย", "");
        add(view, ID_ENGLISH, "English", "");
        wm::CheckMenuItem(view, if theme::current().is_dark() { ID_DARK } else { ID_LIGHT } as u32, wm::MF_CHECKED.0);
        wm::CheckMenuItem(view, if words::is_thai() { ID_THAI } else { ID_ENGLISH } as u32, wm::MF_CHECKED.0);
        sub(t("&View"), view);
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
        self.doc.dirty = true;
        self.say_title();
    }

    /// A property typed into its box: the model takes it now, the canvas shows it now.
    pub fn apply(&mut self, key: Key, text: &str) {
        let f = &mut self.doc.form;
        let num = text.trim().parse::<i32>().ok();
        let list = || text.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect::<Vec<_>>();
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
                    Key::Event => c.set_handler((!text.trim().is_empty()).then(|| text.trim().to_string())),
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

    pub fn select(&mut self, i: Option<usize>) {
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
        let Some(i) = self.selected else { return };
        self.edit(|f| {
            f.control.remove(i);
        });
        self.selected = None;
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
        select_row(self.toolbox, 0);
    }

    // --- saying things --------------------------------------------------------------------------

    fn say_title(&self) {
        let name = self.doc.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| t("Untitled").to_string());
        win::set_text(self.hwnd, &format!("{}{name} - Phai Studio", if self.doc.dirty { "• " } else { "" }));
    }

    pub fn say_selection(&self) {
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
        self.doc = Doc { form, path, dirty: false };
        self.undo.clear();
        self.redo.clear();
        self.selected = None;
        self.load_theme();
        self.rebuild();
        self.refresh_props();
        self.say_title();
        self.say_selection();
    }

    pub fn open_path(&mut self, path: &Path) {
        let name = path.display().to_string();
        match std::fs::read_to_string(path).map_err(|e| e.to_string()).and_then(|src| FormFile::parse(&src).map_err(|e| e.to_string())) {
            Ok(f) => self.set_doc(f, Some(path.to_path_buf())),
            Err(e) => {
                win::message(Some(self.hwnd), &format!("{}\n\n{e}", tf("{0} cannot be opened", &[&name])), "Phai Studio", wm::MB_ICONWARNING);
            }
        }
    }

    /// Writes the form, its generated code and - the first time - the programmer's file.
    fn save(&mut self, choose: bool) -> bool {
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

    // --- commands -----------------------------------------------------------------------------

    fn command(&mut self, id: i32) {
        match id {
            ID_NEW => {
                if self.may_leave() {
                    self.set_doc(FormFile::new("MainWindow"), None);
                }
            }
            ID_OPEN => {
                if self.may_leave() {
                    if let Some(p) = file_dialog(self.hwnd, false, "Phai form (*.phaiform)", "*.phaiform", "") {
                        self.open_path(&p);
                    }
                }
            }
            ID_SAVE => {
                self.save(false);
            }
            ID_SAVE_AS => {
                self.save(true);
            }
            ID_EXIT => {
                if self.may_leave() {
                    unsafe {
                        let _ = wm::DestroyWindow(self.hwnd);
                    }
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
            ID_THEME_NONE => {
                self.edit(|f| f.form.theme = None);
                self.load_theme();
                self.refresh_props();
            }
            _ => {}
        }
    }

    fn relabel(&mut self) {
        win::set_toolbar(self.toolbar, &toolbar_items());
        fill_toolbox(self.toolbox);
        unsafe {
            let old = wm::GetMenu(self.hwnd);
            let _ = wm::SetMenu(self.hwnd, Some(build_menu()));
            let _ = wm::DestroyMenu(old);
        }
        if let Some(p) = self.props.as_mut() {
            p.relabel();
        }
        controls::status_text(self.status, 0, t("Ready"));
        self.say_title();
        self.rebuild();
    }
}

/// Where the theme is for the generated code: beside the form for `include_str!`, and from the
/// crate's `Cargo.toml` for a debug build to watch.
fn theme_options(form: &Path, theme_file: Option<&str>) -> codegen::Options {
    let Some(th) = theme_file else { return codegen::Options::default() };
    let dir = form.with_file_name("");
    let watch = dir.ancestors().find(|d| d.join("Cargo.toml").exists()).and_then(|root| dir.join(th).strip_prefix(root).ok().map(|p| p.to_string_lossy().replace('\\', "/")));
    codegen::Options { theme_include: Some(th.replace('\\', "/")), theme_watch: watch }
}

/// The toolbox's rows: the pointer, then every kind.
fn fill_toolbox(h: HWND) {
    use windows::Win32::UI::Controls as cc;
    unsafe {
        wm::SendMessageW(h, cc::LVM_DELETEALLITEMS, None, None);
    }
    win::list_add(h, &[t("Pointer")]);
    for k in Kind::ALL {
        win::list_add(h, &[kind_label(k)]);
    }
    select_row(h, 0);
}

fn select_row(h: HWND, row: usize) {
    use windows::Win32::UI::Controls as cc;
    unsafe {
        let clear = cc::LVITEMW { stateMask: cc::LVIS_SELECTED, state: cc::LIST_VIEW_ITEM_STATE_FLAGS(0), ..Default::default() };
        wm::SendMessageW(h, cc::LVM_SETITEMSTATE, Some(WPARAM(usize::MAX)), Some(LPARAM(&clear as *const _ as isize)));
        let set = cc::LVITEMW { stateMask: cc::LVIS_SELECTED, state: cc::LVIS_SELECTED, ..Default::default() };
        wm::SendMessageW(h, cc::LVM_SETITEMSTATE, Some(WPARAM(row)), Some(LPARAM(&set as *const _ as isize)));
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
        wm::WM_COMMAND => {
            s.command(win::loword(w.0) as i32);
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            use windows::Win32::UI::Controls as cc;
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.hwndFrom == s.toolbox && hdr.code == cc::LVN_ITEMCHANGED {
                let it = unsafe { &*(l.0 as *const cc::NMLISTVIEW) };
                if it.uNewState & cc::LVIS_SELECTED.0 != 0 {
                    s.tool = if it.iItem <= 0 { None } else { Kind::ALL.get(it.iItem as usize - 1).copied() };
                }
            }
            win::def(hwnd, msg, w, l)
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
            if s.may_leave() {
                unsafe {
                    let _ = wm::DestroyWindow(hwnd);
                }
            }
            LRESULT(0)
        }
        wm::WM_DESTROY => {
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
        (b'S', true) => Some(ID_SAVE),
        // In a box Ctrl+Z is the box's own undo.
        (b'Z', true) if !in_box => Some(ID_UNDO),
        (b'Y', true) if !in_box => Some(ID_REDO),
        (0x74, _) => Some(ID_PREVIEW),
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
        let toolbox = win::list(hwnd, ID_TOOLBOX, font, &[("", 150)]);
        // One column and no heading: a plain list of what can be drawn.
        let style = wm::GetWindowLongW(toolbox, wm::GWL_STYLE);
        wm::SetWindowLongW(toolbox, wm::GWL_STYLE, style | 0x4000 /* LVS_NOCOLUMNHEADER */ | 0x0004 /* LVS_SINGLESEL */);
        fill_toolbox(toolbox);
        let status = controls::status_bar(hwnd, ID_STATUS, font);
        controls::status_parts(status, &[260]);
        controls::status_text(status, 0, t("Ready"));
        let dpi = win::dpi(hwnd);
        let status_h = phai_ui::layout::unscale(controls::status_height(status), dpi);
        let mut layout = Layout::new((1100, 700));
        layout
            .add(toolbar, Item { bounds: Rect::new(0, 0, 0, win::TOOLBAR_HEIGHT), dock: Dock::Top, anchor: Anchor::default() })
            .add(status, Item { bounds: Rect::new(0, 0, 0, status_h), dock: Dock::Bottom, anchor: Anchor::default() })
            .add(toolbox, Item { bounds: Rect::new(0, 0, 150, 0), dock: Dock::Left, anchor: Anchor::default() });
        let studio_box = Box::new(Studio {
            hwnd,
            font,
            toolbar,
            toolbox,
            status,
            canvas: None,
            props: None,
            layout,
            doc: Doc { form: FormFile::new("MainWindow"), path: None, dirty: false },
            undo: vec![],
            redo: vec![],
            selected: None,
            tool: None,
        });
        STUDIO.store(Box::into_raw(studio_box), Ordering::Relaxed);
        let s = studio().unwrap();
        let props = Props::new(hwnd, ID_PROPS, font);
        let canvas = Canvas::new(hwnd, ID_CANVAS);
        s.layout.add(props.hwnd, Item { bounds: Rect::new(0, 0, 260, 0), dock: Dock::Right, anchor: Anchor::default() });
        s.layout.add(canvas.hwnd, Item { bounds: Rect::default(), dock: Dock::Fill, anchor: Anchor::default() });
        s.props = Some(props);
        s.canvas = Some(canvas);
        win::theme_frame(hwnd);
        let mut r = RECT { left: 0, top: 0, right: win::px(hwnd, 1100), bottom: win::px(hwnd, 700) };
        let st = wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32;
        let _ = windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(st), true, Default::default(), dpi as u32);
        let _ = wm::SetWindowPos(hwnd, None, 0, 0, r.right - r.left, r.bottom - r.top, wm::SWP_NOMOVE | wm::SWP_NOZORDER);
        match args.iter().skip(1).find(|a| !a.starts_with("--")) {
            Some(p) => s.open_path(Path::new(p)),
            None => s.set_doc(FormFile::new("MainWindow"), None),
        }
        let _ = wm::ShowWindow(hwnd, wm::SW_SHOW);
        s.layout.apply(hwnd);
        s.rebuild();
        s.refresh_props();
        // `--selftest out.phaiform`: draws, moves and sizes a control with the mouse messages a
        // person's hand would send, then saves - the designer checked end to end.
        if let Some(i) = args.iter().position(|a| a == "--selftest") {
            selftest(Path::new(args.get(i + 1).map(String::as_str).unwrap_or("selftest.phaiform")));
        }
        // The loop: shortcuts first, then Tab between boxes, then everything else.
        if std::env::var("PHAI_SHOT").is_ok() {
            win::run_loop(Some(hwnd));
            return;
        }
        let mut msg = wm::MSG::default();
        while wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
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
