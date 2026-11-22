//! The form file, `.phf` (docs/phaiform.md): read, check, write, generate Rust from it, build it live.

pub mod arrange;
pub mod bars;
pub mod codegen;
pub mod events;
pub mod live;
pub mod model;
pub mod props;
pub mod report;
pub mod sample;
pub mod web;

pub use model::*;
pub use toml;

/// A form file's extension: `main_window.phf`.
pub const FORM_EXT: &str = "phf";
/// What form files were called before 0.2.6, still read: `main_window.phaiform`.
pub const OLD_FORM_EXT: &str = "phaiform";

/// Whether `path` is a form file - `.phf`, or the older `.phaiform`.
pub fn is_form(path: &std::path::Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case(FORM_EXT) || e.eq_ignore_ascii_case(OLD_FORM_EXT))
}

/// The form a file name names, without its extension: `main_window` for `main_window.phf` (or
/// `main_window.phaiform`); `None` for any other file.
pub fn form_stem(name: &str) -> Option<&str> {
    name.strip_suffix(".phf").or_else(|| name.strip_suffix(".phaiform"))
}

/// The form file of form `stem` in `dir`: the `.phf`, or the older `.phaiform` when only that is
/// there; the `.phf` to write when neither is.
pub fn form_path(dir: &std::path::Path, stem: &str) -> std::path::PathBuf {
    let new = dir.join(format!("{stem}.{FORM_EXT}"));
    let old = dir.join(format!("{stem}.{OLD_FORM_EXT}"));
    if !new.exists() && old.exists() { old } else { new }
}

#[cfg(test)]
mod ext_tests {
    use super::*;

    #[test]
    fn form_files_are_phf_and_the_older_phaiform() {
        assert!(is_form(std::path::Path::new("src/main_window.phf")));
        assert!(is_form(std::path::Path::new("src/main_window.phaiform")));
        assert!(!is_form(std::path::Path::new("src/main_window.rs")));
        assert_eq!(form_stem("main_window.phf"), Some("main_window"));
        assert_eq!(form_stem("main_window.phaiform"), Some("main_window"));
        assert_eq!(form_stem("main_window_ui.rs"), None);
        assert!(form_path(std::path::Path::new("no/such/folder"), "x").ends_with("x.phf"));
    }
}
