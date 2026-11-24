//! New Project's templates: what each makes, in the groups New Project lists them by. The first
//! two are made in code (`project.rs`); the rest are folders of `apps/studio/templates/`, built into
//! Phai Studio by `build.rs` - their forms, code, words in Thai and a picture of them running - and
//! copied into the new project, each form's `_ui.rs` written from it there.

use std::path::{Path, PathBuf};

use phai_form::{codegen, FormFile};

include!(concat!(env!("OUT_DIR"), "/templates.rs"));

/// One template: its folder's name (`key`), the group it is listed under, its name and what it
/// makes (English keys, Thai in `text.rs`), its first form, and the Phai libraries it needs beyond
/// phai-ui.
pub struct Template {
    pub key: &'static str,
    pub group: &'static str,
    pub name: &'static str,
    pub about: &'static str,
    pub first: &'static str,
    pub libraries: &'static [&'static str],
}

const fn tpl(key: &'static str, group: &'static str, name: &'static str, about: &'static str, first: &'static str, libraries: &'static [&'static str]) -> Template {
    Template { key, group, name, about, first, libraries }
}

/// Every template, in the order New Project lists them.
pub const TEMPLATES: &[Template] = &[
    tpl("windows", "Basic", "Windows program", "A window with a menu, a toolbar and a status bar - the form opens filling the screen.", "main_window.phf", &[]),
    tpl("phone", "Basic", "Phone app (PWA)", "A phone-sized form, its controls one under another; Build > Run on Phone shows it on a phone at once.", "main_window.phf", &[]),
    tpl("mdi", "General", "Child forms program (MDI)", "A main window whose documents open inside it, with File, Window and Help menus.", "main_window.phf", &[]),
    tpl("dialog", "General", "Small dialog program", "One fixed-size window that converts lengths, a good first program.", "main_window.phf", &[]),
    tpl("tray", "General", "Tray program", "Starts hidden as an icon by the clock; its menu opens a small window or exits.", "main_window.phf", &[]),
    tpl("records", "Business", "Records with a database", "Customers kept in a SQLite file beside the program: search, edit, add and delete.", "main_window.phf", &["phai-data"]),
    tpl("login", "Business", "Sign in, then the main window", "A sign-in form comes first; the right password opens the main window, and Sign out returns to it.", "main_window.phf", &[]),
    tpl("report", "Business", "Report with print preview", "Sales in a grid, printed as a report grouped by branch with totals: preview, print or save as PDF.", "main_window.phf", &[]),
    tpl("admin", "Business", "Admin dashboard", "A back-office window: a dark side menu, a header with search, summary cards, goals, and pages of users, orders and settings.", "main_window.phf", &[]),
    tpl("explorer", "Familiar", "Explorer-style browser", "Folders in a tree on the left, the chosen folder's files with size and date on the right.", "main_window.phf", &[]),
    tpl("notepad", "Familiar", "Text editor (like Notepad)", "A Notepad-style editor: open, save, find, word wrap and font, asking to save changes.", "main_window.phf", &[]),
    tpl("wizard", "Familiar", "Step-by-step wizard", "Welcome, options, details and summary pages with Back, Next, Finish and Cancel.", "main_window.phf", &[]),
    tpl("game", "School", "Catch the ball (timer game)", "A simple timer game for schools: click the bouncing ball as often as you can in 30 seconds.", "main_window.phf", &[]),
];

/// The groups, in order.
pub const GROUPS: &[&str] = &["Basic", "General", "Business", "Familiar", "School"];

/// The templates whose folder is built in (the two made in code always are).
pub fn available() -> Vec<&'static Template> {
    TEMPLATES.iter().filter(|t| matches!(t.key, "windows" | "phone") || TEMPLATE_FILES.iter().any(|(k, _, _)| *k == t.key)).collect()
}

/// A template's files: (path inside the template, bytes).
fn files(key: &str) -> impl Iterator<Item = (&'static str, &'static [u8])> + '_ {
    TEMPLATE_FILES.iter().filter(move |(k, _, _)| *k == key).map(|(_, p, b)| (*p, *b))
}

/// The picture of a template running, when its folder has one.
pub fn preview(key: &str) -> Option<&'static [u8]> {
    files(key).find(|(p, _)| *p == "preview.png").map(|(_, b)| b)
}

/// Writes a new project made from template `key` into `dir`; returns its first form's path. `dir`
/// must have no Cargo.toml.
pub fn create(dir: &Path, key: &str) -> std::io::Result<PathBuf> {
    match key {
        "windows" => return crate::project::create_with(dir, false),
        "phone" => return crate::project::create_with(dir, true),
        _ => {}
    }
    let t = TEMPLATES.iter().find(|t| t.key == key).ok_or_else(|| std::io::Error::other(format!("no template {key}")))?;
    if dir.join("Cargo.toml").exists() {
        return Err(std::io::Error::other("this folder already has a Cargo.toml"));
    }
    let name = crate::project::package_name(&dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
    std::fs::create_dir_all(dir.join("src"))?;
    std::fs::write(dir.join("Cargo.toml"), crate::project::cargo_toml_with(&name, t.libraries))?;
    std::fs::write(dir.join("build.rs"), crate::project::BUILD_RS)?;
    std::fs::write(dir.join(".gitignore"), "/target\n")?;
    let mut theme = false;
    for (rel, bytes) in files(key) {
        if rel == "preview.png" {
            continue;
        }
        theme |= rel == "theme.css";
        let to = dir.join(rel.replace('/', "\\"));
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&to, bytes)?;
    }
    if !theme {
        std::fs::write(dir.join("theme.css"), phai_ui::theme::STANDARD_CSS)?;
    }
    // Each form's _ui.rs, as saving it in the designer writes it.
    for (rel, bytes) in files(key) {
        let Some(stem) = rel.strip_prefix("src/").and_then(|r| r.strip_suffix(".phf")) else { continue };
        let text = String::from_utf8_lossy(bytes);
        let form = FormFile::parse(&text).map_err(|e| std::io::Error::other(format!("{rel}: {}", e.message)))?;
        let opts = codegen::Options { theme_include: Some("../theme.css".into()), theme_watch: Some("theme.css".into()), form_file: Some(rel.to_string()) };
        std::fs::write(dir.join("src").join(format!("{stem}_ui.rs")), codegen::generate(&form, &opts))?;
    }
    Ok(dir.join("src").join(t.first))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every built-in template folder has its forms readable, a first form, main.rs and text.rs,
    /// and a picture - and every folder is one New Project lists.
    #[test]
    fn every_template_folder_is_whole() {
        let mut keys: Vec<&str> = TEMPLATE_FILES.iter().map(|(k, _, _)| *k).collect();
        keys.dedup();
        for key in keys {
            let t = TEMPLATES.iter().find(|t| t.key == key).unwrap_or_else(|| panic!("templates/{key} is not in TEMPLATES"));
            // The two made in code: only their picture is a file.
            if matches!(key, "windows" | "phone") {
                assert!(preview(key).is_some(), "{key}: no preview.png");
                continue;
            }
            let names: Vec<&str> = files(key).map(|(p, _)| p).collect();
            for must in ["src/main.rs", "src/text.rs", "preview.png"] {
                assert!(names.contains(&must), "{key}: no {must}");
            }
            assert!(names.contains(&format!("src/{}", t.first).as_str()), "{key}: no first form {}", t.first);
            for (rel, bytes) in files(key).filter(|(p, _)| p.ends_with(".phf")) {
                FormFile::parse(&String::from_utf8_lossy(bytes)).unwrap_or_else(|e| panic!("{key}/{rel}: {}", e.message));
            }
        }
    }
}
