//! The project explorer: the project's files as a tree, on the right above the properties, as
//! Visual Studio has its Solution Explorer. Double-click a form to open it in a tab; anything
//! else opens the way Windows opens it. Right-click for a new form.

use std::path::{Path, PathBuf};

use phai_form::codegen;
use phai_form::{snake, FormFile};
use phai_ui::icon::glyph;
use phai_ui::layout::{Anchor, Dock, Item, Layout, Rect};
use phai_ui::text::{t, tf};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::HFONT;
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HMENU};

use crate::studio;

const ID_TREE: i32 = 400;
const CMD_NEW_FORM: u32 = 1;
const CMD_SHOW: u32 = 2;
const CMD_REFRESH: u32 = 3;

/// Icons, by index: folder, file, form, Rust source, the project, the theme.
const ICONS: [u16; 6] = [glyph::FOLDER, glyph::DOCUMENT, glyph::FORM, glyph::CODE, glyph::LIBRARY, glyph::BRIGHTNESS];

pub struct Explorer {
    pub hwnd: HWND,
    pub tree: HWND,
    root: Option<PathBuf>,
    /// The path behind each item, by the number kept with it.
    entries: Vec<PathBuf>,
}

struct Side {
    layout: Layout,
}

/// The right-hand panel that holds the explorer and the properties.
pub fn side_panel(parent: HWND) -> HWND {
    let class = win::register("PhaiStudioSide", Some(side_proc), None);
    unsafe {
        let h = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPCHILDREN | wm::WS_CLIPSIBLINGS, 0, 0, 10, 10, Some(parent), Some(HMENU::default()), Some(win::instance()), None).unwrap_or_default();
        win::set_state(h, Box::new(Side { layout: Layout::new((280, 600)) }));
        h
    }
}

/// Stacks the panel: the heading, the tree, then the properties filling the rest.
pub fn set_side_parts(side: HWND, heading: HWND, tree: HWND, props: HWND) {
    if let Some(s) = unsafe { win::state::<Side>(side) } {
        s.layout.add(heading, Item { bounds: Rect::new(0, 0, 0, 30), dock: Dock::Top, anchor: Anchor::default() });
        s.layout.add(tree, Item { bounds: Rect::new(0, 0, 0, 240), dock: Dock::Top, anchor: Anchor::default() });
        s.layout.add(props, Item { bounds: Rect::default(), dock: Dock::Fill, anchor: Anchor::default() });
        s.layout.apply(side);
    }
}

extern "system" fn side_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_SIZE => {
            if let Some(s) = unsafe { win::state::<Side>(h) } {
                s.layout.apply(h);
            }
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            use windows::Win32::UI::Controls as cc;
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.idFrom == ID_TREE as usize {
                if let Some(e) = studio().and_then(|s| s.explorer.as_mut()) {
                    match hdr.code {
                        cc::NM_DBLCLK => {
                            e.open_selected();
                            return LRESULT(1);
                        }
                        cc::NM_RCLICK => {
                            e.menu(h);
                            return LRESULT(1);
                        }
                        _ => {}
                    }
                }
            }
            win::def(h, msg, w, l)
        }
        wm::WM_NCDESTROY => {
            drop(unsafe { win::take_state::<Side>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

impl Explorer {
    pub fn new(side: HWND, font: HFONT) -> Explorer {
        let hwnd = win::label(side, t("Project"), 0, font);
        win::set_css(hwnd, "muted", "");
        let tree = win::tree(side, ID_TREE, font);
        win::tree_icons(tree, &ICONS);
        Explorer { hwnd, tree, root: None, entries: vec![] }
    }

    pub fn relabel(&mut self) {
        let root = self.root.clone();
        self.fill(root);
    }

    /// Shows the project that holds `form` - or the form's folder, or nothing.
    pub fn show_project_of(&mut self, form: Option<&Path>) {
        let root = form.and_then(|f| crate::cargo::project_of(f).or_else(|| f.parent().map(Path::to_path_buf)));
        if root != self.root || self.entries.is_empty() {
            self.fill(root);
        }
    }

    fn fill(&mut self, root: Option<PathBuf>) {
        win::tree_clear(self.tree);
        self.entries.clear();
        self.root = root.clone();
        let Some(root) = root else {
            win::set_text(self.hwnd, t("Project"));
            win::tree_add(self.tree, 0, t("No project"), 1, -1);
            return;
        };
        let name = root.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        win::set_text(self.hwnd, &format!("{} · {name}", t("Project")));
        self.entries.push(root.clone());
        let top = win::tree_add(self.tree, 0, &name, 4, 0);
        self.add_folder(top, &root, 0);
        win::tree_expand(self.tree, top);
    }

    fn add_folder(&mut self, parent: isize, dir: &Path, depth: usize) {
        if depth > 6 {
            return;
        }
        let Ok(read) = std::fs::read_dir(dir) else { return };
        let mut items: Vec<(bool, String, PathBuf)> = read
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                // What the build makes and what git keeps are not the project's own files.
                if name.starts_with('.') || name == "target" || name == "Cargo.lock" {
                    return None;
                }
                Some((e.path().is_dir(), name, e.path()))
            })
            .collect();
        // Folders first, then files, each by name.
        items.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.to_lowercase().cmp(&b.1.to_lowercase())));
        for (is_dir, name, path) in items {
            let icon = if is_dir {
                0
            } else if name.ends_with(".phaiform") {
                2
            } else if name.ends_with(".rs") {
                3
            } else if name.ends_with(".css") {
                5
            } else {
                1
            };
            self.entries.push(path.clone());
            let item = win::tree_add(self.tree, parent, &name, icon, self.entries.len() as isize - 1);
            if is_dir {
                self.add_folder(item, &path, depth + 1);
                if name == "src" {
                    win::tree_expand(self.tree, item);
                }
            }
        }
    }

    fn selected_path(&self) -> Option<PathBuf> {
        let i = win::tree_selected(self.tree)?;
        self.entries.get(usize::try_from(i).ok()?).cloned()
    }

    fn open_selected(&mut self) {
        let Some(path) = self.selected_path() else { return };
        if path.is_dir() {
            return;
        }
        let Some(s) = studio() else { return };
        if path.extension().is_some_and(|e| e == "phaiform") {
            s.open_path(&path);
        } else {
            crate::open_with_windows(&path.to_string_lossy());
        }
    }

    fn menu(&mut self, owner: HWND) {
        let chosen = win::popup(owner, &[(CMD_NEW_FORM, t("New form...")), (0, ""), (CMD_SHOW, t("Show in Explorer")), (CMD_REFRESH, t("Refresh"))]);
        match chosen {
            CMD_NEW_FORM => {
                if let Some(root) = self.root.clone() {
                    if let Some(s) = studio() {
                        s.new_form(&root);
                    }
                }
            }
            CMD_SHOW => {
                if let Some(p) = self.selected_path().or_else(|| self.root.clone()) {
                    let dir = if p.is_dir() { p } else { p.parent().map(Path::to_path_buf).unwrap_or(p) };
                    crate::open_with_windows(&dir.to_string_lossy());
                }
            }
            CMD_REFRESH => {
                let root = self.root.clone();
                self.fill(root);
            }
            _ => {}
        }
    }
}

/// `MainWindow`-style names only: they become a struct and two file names.
pub fn is_form_name(s: &str) -> bool {
    s.starts_with(|c: char| c.is_ascii_uppercase()) && s.chars().all(|c| c.is_ascii_alphanumeric())
}

/// The lines that make a new form's two files part of the program: `mod x; mod x_ui;` after the
/// last `mod` line of `main.rs` (or at its top). `None` when they are there already.
pub fn add_mods(main_rs: &str, file: &str) -> Option<String> {
    let lines = [format!("mod {file};"), format!("mod {file}_ui;")];
    if lines.iter().all(|l| main_rs.lines().any(|x| x.trim() == l)) {
        return None;
    }
    let mut out: Vec<String> = main_rs.lines().map(str::to_string).collect();
    let at = out.iter().rposition(|l| l.trim_start().starts_with("mod ")).map(|i| i + 1).unwrap_or(0);
    for (k, l) in lines.iter().enumerate() {
        if !out.iter().any(|x| x.trim() == l) {
            out.insert(at + k, l.clone());
        }
    }
    Some(out.join("\n") + "\n")
}

impl crate::Studio {
    /// Right-click, New form: asks a name, writes the form and its code into `src`, makes them part
    /// of the program, and opens the form in a tab.
    pub fn new_form(&mut self, root: &Path) {
        let Some(name) = phai_ui::dialogs::ask(Some(self.hwnd), t("New form"), t("Name of the new form, like CustomerWindow"), "", false) else { return };
        let name = name.trim().to_string();
        if !is_form_name(&name) {
            win::message(Some(self.hwnd), &tf("{0} is not a form name - PascalCase letters and digits, like CustomerWindow", &[&name]), "Phai Studio", wm::MB_ICONWARNING);
            return;
        }
        let src = root.join("src");
        let file = snake(&name);
        let form_path = src.join(format!("{file}.phaiform"));
        if form_path.exists() {
            win::message(Some(self.hwnd), &tf("{0} already exists", &[&form_path.display()]), "Phai Studio", wm::MB_ICONWARNING);
            return;
        }
        let mut f = FormFile::new(&name);
        if root.join("theme.css").exists() {
            f.form.theme = Some("../theme.css".into());
        }
        let opts = crate::theme_options(&form_path, f.form.theme.as_deref());
        let wrote = std::fs::create_dir_all(&src)
            .and_then(|_| std::fs::write(&form_path, f.to_toml()))
            .and_then(|_| std::fs::write(src.join(format!("{file}_ui.rs")), codegen::generate(&f, &opts)))
            .and_then(|_| std::fs::write(src.join(format!("{file}.rs")), codegen::user_stub(&f)));
        if let Err(e) = wrote {
            win::message(Some(self.hwnd), &format!("{}\n\n{e}", tf("{0} cannot be made", &[&form_path.display()])), "Phai Studio", wm::MB_ICONWARNING);
            return;
        }
        let main_rs = src.join("main.rs");
        if let Ok(text) = std::fs::read_to_string(&main_rs) {
            if let Some(more) = add_mods(&text, &file) {
                let _ = std::fs::write(&main_rs, more);
            }
        }
        self.open_in_tab(f, Some(form_path));
        if let Some(e) = self.explorer.as_mut() {
            e.fill(Some(root.to_path_buf()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_forms_files_join_the_program_once() {
        let main = "//! x\n\nmod main_window;\nmod main_window_ui;\nmod text;\n\nfn main() {}\n";
        let more = add_mods(main, "customer_window").unwrap();
        assert!(more.contains("mod text;\nmod customer_window;\nmod customer_window_ui;\n\nfn main"), "{more}");
        assert!(add_mods(&more, "customer_window").is_none());
        assert_eq!(add_mods("fn main() {}\n", "a").unwrap(), "mod a;\nmod a_ui;\nfn main() {}\n");
    }

    #[test]
    fn form_names() {
        assert!(is_form_name("CustomerWindow"));
        assert!(!is_form_name("customer"));
        assert!(!is_form_name("Customer Window"));
        assert!(!is_form_name(""));
    }
}
