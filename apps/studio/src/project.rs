//! A new project: everything a Phai program needs, in a folder the person picked - Cargo.toml,
//! the manifest build script, main.rs, a first form with its generated code, the standard theme,
//! and a Thai table whose test already passes.

use std::path::{Path, PathBuf};

use phai_form::codegen;
use phai_form::{Control, FormFile, Kind};
use windows::Win32::Foundation::HWND;

/// Where phai-ui is on this machine, for the new project's Cargo.toml - until it is published:
/// installed, beside Phai Studio (`phai-ui\` in its folder); else the source it was built from.
pub fn phai_ui_path() -> PathBuf {
    if let Some(installed) = std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("phai-ui"))).filter(|d| d.join("Cargo.toml").is_file()) {
        return installed;
    }
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("crates").join("phai-ui");
    here.canonicalize().map(|p| PathBuf::from(p.to_string_lossy().trim_start_matches(r"\\?\"))).unwrap_or(here)
}

/// Where phai-data is on this machine, as [`phai_ui_path`] finds phai-ui: beside Phai Studio when
/// installed, else the source it was built from.
pub fn phai_data_path() -> PathBuf {
    if let Some(installed) = std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("phai-data"))).filter(|d| d.join("Cargo.toml").is_file()) {
        return installed;
    }
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("crates").join("phai-data");
    here.canonicalize().map(|p| PathBuf::from(p.to_string_lossy().trim_start_matches(r"\\?\"))).unwrap_or(here)
}

/// How a project names phai-data - the same way as phai-ui.
pub fn phai_data_dependency() -> String {
    library_dependency(&phai_data_path())
}

/// Whether this Phai Studio was installed from the Microsoft Store: an MSIX package lives in
/// `WindowsApps`, a folder no one writes in, whose name changes with every update.
pub fn from_store(exe: &Path) -> bool {
    exe.components().any(|c| c.as_os_str().eq_ignore_ascii_case("WindowsApps"))
}

/// How a project names one of the Phai libraries, `local` its copy on this machine. That copy
/// while it is there; from the repository Phai Studio was built from when it is not - or when
/// Phai Studio came from the Store, whose folder moves with each update, or the setting
/// `library_from_git` is `1`. From git, pinned to this Phai Studio's release (`tag = "v0.2.1"`),
/// so the project builds the same next year.
pub fn library_dependency(local: &Path) -> String {
    let repo = env!("CARGO_PKG_REPOSITORY");
    let exe = std::env::current_exe().unwrap_or_default();
    let want_git = !repo.is_empty() && (from_store(&exe) || crate::setting_pub("library_from_git").as_deref() == Some("1"));
    library_line(local, repo, env!("CARGO_PKG_VERSION"), want_git)
}

/// The dependency's line: `{ path = ".." }` or `{ git = "..", tag = "v.." }`.
pub fn library_line(local: &Path, repo: &str, version: &str, want_git: bool) -> String {
    if !want_git && (local.join("Cargo.toml").is_file() || repo.is_empty()) {
        format!("{{ path = {:?} }}", local.to_string_lossy().replace('\\', "/"))
    } else {
        format!("{{ git = {repo:?}, tag = \"v{version}\" }}")
    }
}

/// A project's Cargo.toml with phai-data among its dependencies (unchanged when it is there).
pub fn with_phai_data(cargo_toml: &str) -> String {
    if cargo_toml.lines().any(|l| l.trim_start().starts_with("phai-data")) {
        return cargo_toml.to_string();
    }
    let mut out = String::new();
    let mut added = false;
    for l in cargo_toml.lines() {
        out.push_str(l);
        out.push(char::from(10));
        if !added && l.trim() == "[dependencies]" {
            out.push_str(&format!("phai-data = {}", phai_data_dependency()));
            out.push(char::from(10));
            added = true;
        }
    }
    if !added {
        out.push_str(&format!("{}[dependencies]{}phai-data = {}{}", char::from(10), char::from(10), phai_data_dependency(), char::from(10)));
    }
    out
}

/// `src/data_sources.rs`: where the program's data is - the server of the tables its forms show,
/// as WinForms keeps a connection string. The password is never written here: it comes from the
/// PHAI_DB_PASSWORD environment variable, so the code can be shared and kept in Git.
pub fn data_sources_rs(c: &phai_data::Connection) -> String {
    let server = if c.kind == phai_data::Kind::Sqlite {
        String::new()
    } else {
        format!(
            "        port: {},\n        user: {:?}.into(),\n        // The password from the environment, never from the code.\n        password: std::env::var(\"PHAI_DB_PASSWORD\").unwrap_or_default(),\n        database: {:?}.into(),\n        ssl: {},\n        trust_cert: {},\n",
            c.port, c.user, c.database, c.ssl, c.trust_cert
        )
    };
    format!(
        "//! Where this program's data is: the database server the forms' tables come from.\n//! Phai Studio wrote it when a table was first put on a form; it is yours to change.\n\n/// The connection every bound grid uses.\npub fn connection() -> phai_data::Connection {{\n    phai_data::Connection {{\n        kind: phai_data::Kind::{:?},\n        host: {:?}.into(),\n{server}        ..Default::default()\n    }}\n}}\n",
        c.kind, c.host
    )
}

/// The project's Thai table (`src/text.rs`) with these sentences added - the ones it does not
/// have yet - so its test that every sentence has Thai still passes after Phai Studio adds
/// words to a form. A sentence with no Thai yet is given its English, to change later.
pub fn add_thai(src: &Path, pairs: &[(&str, &str)]) -> std::io::Result<()> {
    let path = src.join("text.rs");
    let Ok(text) = std::fs::read_to_string(&path) else { return Ok(()) };
    let Some(start) = text.find("pub static THAI") else { return Ok(()) };
    let Some(end) = text[start..].find("];").map(|i| start + i) else { return Ok(()) };
    let mut add = String::new();
    for (en, th) in pairs {
        let key = format!("({en:?},");
        if !text.contains(&key) && !add.contains(&key) {
            add.push_str(&format!("    ({en:?}, {th:?}),"));
            add.push(char::from(10));
        }
    }
    if add.is_empty() {
        return Ok(());
    }
    std::fs::write(&path, format!("{}{add}{}", &text[..end], &text[end..]))
}

/// A folder name as a Cargo package name: lowercase letters, digits, `-` and `_`.
/// The project's forms still called by the older name, `.phaiform`: in `src` and in the project's
/// own folder.
pub fn old_forms(root: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = [root.join("src"), root.to_path_buf()]
        .iter()
        .flat_map(|d| std::fs::read_dir(d).into_iter().flatten().flatten().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case(phai_form::OLD_FORM_EXT)))
        .collect();
    out.sort();
    out
}

/// Each of `forms` renamed to `.phf`: with `git mv` when git knows the file, so its history goes
/// with it, else a plain rename (never over a `.phf` already there); its `_ui.rs` written again,
/// as it names the form's file. What was renamed, old and new, until the first that could not be.
pub fn rename_forms(forms: &[PathBuf]) -> (Vec<(PathBuf, PathBuf)>, Option<String>) {
    let mut done = vec![];
    for old in forms {
        let new = old.with_extension(phai_form::FORM_EXT);
        if new.exists() {
            return (done, Some(format!("{} already exists", new.display())));
        }
        let dir = old.parent().unwrap_or(Path::new("."));
        let name = |p: &Path| p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let tracked = crate::gitpane::git(dir, &["ls-files", "--error-unmatch", &name(old)]).0;
        let moved = if tracked {
            let (ok, said) = crate::gitpane::git(dir, &["mv", &name(old), &name(&new)]);
            if ok { Ok(()) } else { Err(said.trim().to_string()) }
        } else {
            std::fs::rename(old, &new).map_err(|e| e.to_string())
        };
        if let Err(e) = moved {
            return (done, Some(format!("{}: {e}", old.display())));
        }
        if let Some(f) = std::fs::read_to_string(&new).ok().and_then(|t| FormFile::parse(&t).ok()) {
            let ui = new.with_file_name(format!("{}_ui.rs", crate::snake(&f.form.name)));
            if ui.exists() {
                let _ = std::fs::write(&ui, codegen::generate(&f, &crate::theme_options(&new, f.form.theme.as_deref())));
            }
        }
        done.push((old.clone(), new));
    }
    (done, None)
}

pub fn package_name(folder: &str) -> String {
    let mut s: String = folder.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' }).collect();
    s = s.trim_matches('_').to_string();
    if s.is_empty() || s.starts_with(|c: char| c.is_ascii_digit()) {
        s = format!("app_{s}");
    }
    s
}

/// The first form: a window that says hello, with a button and a status bar.
pub fn first_form() -> FormFile {
    let mut f = FormFile::new("MainWindow");
    f.form.title = "My program".into();
    f.form.size = [480, 320];
    f.form.theme = Some("../theme.css".into());
    // A new program opens filling the screen; restored, it sits in the middle of it.
    f.form.maximized = Some(true);
    let mut hello = Control::new(Kind::Label, "hello", [11, 11, 300, 23]);
    hello.text = Some("Hello".into());
    let mut ok = Control::new(Kind::Button, "ok", [394, 260, 75, 23]);
    ok.text = Some("OK".into());
    ok.class = Some("primary".into());
    ok.anchor = Some("right bottom".into());
    ok.on_click = Some("ok_clicked".into());
    let mut status = Control::new(Kind::Statusbar, "status", [0, 0, 0, 26]);
    status.text = Some("Ready".into());
    f.control = vec![hello, ok, status];
    f
}

/// A new project's Cargo.toml naming, beside phai-ui, the Phai `libraries` a template needs
/// (`phai-data`, `phai-web`) as this Phai Studio has them.
pub fn cargo_toml_with(name: &str, wanted: &[&str]) -> String {
    let mut toml = cargo_toml(name);
    let extra: String = libraries().iter().filter(|(n, _)| wanted.contains(n)).map(|(n, local)| format!("{n} = {}\n", library_dependency(local))).collect();
    if let Some(at) = toml.find("\nwindows = ") {
        toml.insert_str(at + 1, &extra);
    }
    toml
}

fn cargo_toml(name: &str) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

# Made with Phai Studio.

[dependencies]
phai-ui = {ui}
windows = {{ version = "0.62", features = [
    "Win32_Foundation",
    "Win32_Graphics_Gdi",
    "Win32_UI_Controls",
    "Win32_UI_HiDpi",
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_UI_WindowsAndMessaging",
] }}

# Thin LTO: a release built in half the time and rebuilt in a quarter of it, for 7% more size
# (measured: 66 s and 8 s for 523 KB with lto = true and codegen-units = 1; 30 s and 2 s for 563 KB).
[profile.release]
opt-level = "s"
lto = "thin"
panic = "abort"
strip = true
"#,
        ui = phai_ui_dependency()
    )
}

/// How a new project names phai-ui: this machine's copy while Phai Studio runs beside its source,
/// else the repository Phai Studio was built from (`repository` in the workspace Cargo.toml) - so a
/// project made by someone who only downloaded Phai Studio builds anywhere.
pub fn phai_ui_dependency() -> String {
    library_dependency(&phai_ui_path())
}

/// The first line of the build.rs Phai Studio writes - how it knows a build.rs is its own, and
/// may bring it up to date.
pub const BUILD_RS_MARK: &str = "//! Made by Phai Studio.";

pub const BUILD_RS: &str = r#"//! Made by Phai Studio. Embeds a manifest that names Common Controls 6 - without it Windows hands
//! the program the old controls: buttons as in Windows 2000, toolbar icons in black squares - and
//! the program's icon, the .ico Cargo.toml names (`[package.metadata.phai] icon`), as Explorer,
//! the taskbar and shortcuts show it. Project > Properties sets the icon.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
        if let Some(icon) = icon_of_package() {
            println!("cargo:rerun-if-changed={icon}");
            match std::fs::read(&icon).map_err(|e| e.to_string()).and_then(|ico| res_of_ico(&ico)) {
                Ok(res) => {
                    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap_or_default()).join("icon.res");
                    if std::fs::write(&out, res).is_ok() {
                        // The linker takes a compiled resource file as it is.
                        println!("cargo:rustc-link-arg={}", out.display());
                    }
                }
                Err(e) => println!("cargo:warning=the program's icon {icon}: {e}"),
            }
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
}

/// `icon = "..."` under `[package.metadata.phai]` in Cargo.toml.
fn icon_of_package() -> Option<String> {
    let toml = std::fs::read_to_string("Cargo.toml").ok()?;
    let mut inside = false;
    for line in toml.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line == "[package.metadata.phai]";
        } else if inside {
            if let Some(v) = line.strip_prefix("icon").map(str::trim_start).and_then(|r| r.strip_prefix('=')) {
                let v = v.trim().trim_matches('"').trim();
                return (!v.is_empty()).then(|| v.to_string());
            }
        }
    }
    None
}

/// An .ico as a compiled resource file: each picture an RT_ICON, the set an RT_GROUP_ICON named 1.
fn res_of_ico(ico: &[u8]) -> Result<Vec<u8>, String> {
    let u16_at = |i: usize| ico.get(i..i + 2).map(|b| u16::from_le_bytes([b[0], b[1]])).ok_or("too short");
    let u32_at = |i: usize| ico.get(i..i + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).ok_or("too short");
    if u16_at(0)? != 0 || u16_at(2)? != 1 {
        return Err("not an .ico file".into());
    }
    let count = u16_at(4)? as usize;
    let mut res = Vec::new();
    // A resource: its header (type and name as numbers), then its data, each to 4 bytes.
    fn put(res: &mut Vec<u8>, kind: u16, name: u16, flags: u16, data: &[u8]) {
        for v in [data.len() as u32, 32] {
            res.extend_from_slice(&v.to_le_bytes());
        }
        for v in [0xFFFF, kind, 0xFFFF, name] {
            res.extend_from_slice(&(v as u16).to_le_bytes());
        }
        res.extend_from_slice(&0u32.to_le_bytes()); // data version
        res.extend_from_slice(&flags.to_le_bytes());
        res.extend_from_slice(&0x0409u16.to_le_bytes()); // language
        res.extend_from_slice(&0u32.to_le_bytes()); // version
        res.extend_from_slice(&0u32.to_le_bytes()); // characteristics
        res.extend_from_slice(data);
        while res.len() % 4 != 0 {
            res.push(0);
        }
    }
    // A resource file starts with an empty one.
    res.extend_from_slice(&[0, 0, 0, 0, 32, 0, 0, 0, 0xFF, 0xFF, 0, 0, 0xFF, 0xFF, 0, 0]);
    res.extend_from_slice(&[0u8; 16]);
    let mut group = Vec::new();
    group.extend_from_slice(&[0, 0, 1, 0]);
    group.extend_from_slice(&(count as u16).to_le_bytes());
    for k in 0..count {
        let e = 6 + k * 16;
        let (size, offset) = (u32_at(e + 8)? as usize, u32_at(e + 12)? as usize);
        let image = ico.get(offset..offset + size).ok_or("a picture past the file's end")?;
        put(&mut res, 3, (k + 1) as u16, 0x1010, image);
        group.extend_from_slice(&ico[e..e + 8]); // width, height, colours, reserved, planes, bits
        group.extend_from_slice(&(size as u32).to_le_bytes());
        group.extend_from_slice(&((k + 1) as u16).to_le_bytes());
    }
    put(&mut res, 14, 1, 0x1030, &group);
    Ok(res)
}
"#;

const MAIN_RS: &str = r#"//! Made with Phai Studio. `main_window.phf` is the window's design, `main_window_ui.rs`
//! what Phai Studio writes from it, and `main_window.rs` what its controls do.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod main_window;
mod main_window_ui;
mod text;

fn main() {
    text::load();
    main_window_ui::MainWindowUi::run(main_window::MainWindow::default());
}
"#;

const TEXT_RS: &str = r#"//! The program's words in Thai. The English is the key. The test fails on a sentence the program
//! shows with no Thai, so none is forgotten.

pub static THAI: &[(&str, &str)] = &[
    ("My program", "โปรแกรมของฉัน"),
    ("Hello", "สวัสดี"),
    ("Ready", "พร้อม"),
    ("Hello, {0}", "สวัสดี {0}"),
];

/// Thai when started with `--thai`, else English.
pub fn load() {
    phai_ui::text::add(THAI);
    phai_ui::text::set_thai(std::env::args().any(|a| a == "--thai"));
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_sentence_has_thai() {
        phai_ui::text::add(super::THAI);
        let mut used: Vec<String> = crate::main_window_ui::TEXTS.iter().map(|s| s.to_string()).collect();
        used.extend(phai_ui::text::scan(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src")), &["main_window_ui.rs"]));
        assert_eq!(phai_ui::text::problems(&used), Vec::<String>::new());
    }
}
"#;

const MAIN_WINDOW_RS: &str = r#"//! MainWindow: what its controls do. Phai Studio made this file once; it is yours.
//! The window itself is in main_window_ui.rs, which Phai Studio writes from main_window.phf.

use crate::main_window_ui::{MainWindowEvents, MainWindowUi};
use phai_ui::prelude::*;

#[derive(Default)]
pub struct MainWindow {
    clicks: u32,
}

impl MainWindowEvents for MainWindow {
    fn ok_clicked(&mut self, ui: &MainWindowUi) {
        self.clicks += 1;
        ui.status.set_part_text(0, &tf("Hello, {0}", &[&self.clicks]));
    }
}
"#;

/// The first form of a phone app (New Project > Phone app): phone-sized, its controls one under
/// another as wide as the screen, finger-sized - and the same `ok` and status bar as a Windows
/// program's, so the code made for it fits.
pub fn phone_form() -> FormFile {
    let mut f = FormFile::new("MainWindow");
    f.form.title = "My app".into();
    f.form.size = [390, 700];
    f.form.theme = Some("../theme.css".into());
    let mut hello = Control::new(Kind::Label, "hello", [16, 16, 358, 28]);
    hello.text = Some("Hello".into());
    let mut name_label = Control::new(Kind::Label, "name_label", [16, 56, 358, 22]);
    name_label.text = Some("Your name".into());
    let mut name = Control::new(Kind::Edit, "name", [16, 80, 358, 40]);
    name.anchor = Some("left top right".into());
    let mut ok = Control::new(Kind::Button, "ok", [16, 620, 358, 44]);
    ok.text = Some("OK".into());
    ok.class = Some("primary".into());
    ok.anchor = Some("left right bottom".into());
    ok.on_click = Some("ok_clicked".into());
    let mut status = Control::new(Kind::Statusbar, "status", [0, 0, 0, 26]);
    status.text = Some("Ready".into());
    f.control = vec![hello, name_label, name, ok, status];
    f
}

/// Writes a new project into `dir`; returns its first form's path. `dir` must have no Cargo.toml.
/// (New Project goes through `templates::create`; the tests make their projects with this.)
#[cfg(test)]
pub fn create(dir: &Path) -> std::io::Result<PathBuf> {
    create_with(dir, false)
}

/// As [`create`]; `phone`: a phone app's - its form phone-sized, the web app upright.
pub fn create_with(dir: &Path, phone: bool) -> std::io::Result<PathBuf> {
    if dir.join("Cargo.toml").exists() {
        return Err(std::io::Error::other("this folder already has a Cargo.toml"));
    }
    let name = package_name(&dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
    let src = dir.join("src");
    std::fs::create_dir_all(&src)?;
    let f = if phone { phone_form() } else { first_form() };
    let form_path = src.join("main_window.phf");
    let opts = codegen::Options { theme_include: Some("../theme.css".into()), theme_watch: Some("theme.css".into()), form_file: Some("src/main_window.phf".into()) };
    let files: [(PathBuf, String); 9] = [
        (dir.join("Cargo.toml"), cargo_toml(&name)),
        (dir.join("build.rs"), BUILD_RS.to_string()),
        (dir.join("theme.css"), phai_ui::theme::STANDARD_CSS.to_string()),
        (dir.join(".gitignore"), "/target\n".to_string()),
        (src.join("main.rs"), MAIN_RS.to_string()),
        (src.join("text.rs"), TEXT_RS.to_string()),
        (form_path.clone(), f.to_toml()),
        (src.join("main_window_ui.rs"), codegen::generate(&f, &opts)),
        (src.join("main_window.rs"), MAIN_WINDOW_RS.to_string()),
    ];
    for (path, text) in files {
        std::fs::write(path, text)?;
    }
    if phone {
        // The app as a phone shows it: upright, named as the form is.
        let toml = std::fs::read_to_string(dir.join("Cargo.toml"))?;
        let toml = crate::cargo::with_table_field(&toml, crate::cargo::WEB_APP_TABLE, "orientation", "portrait");
        std::fs::write(dir.join("Cargo.toml"), toml)?;
    }
    Ok(form_path)
}

/// Asks for a folder - Windows' own folder picker, where a new folder can be made too.
pub fn pick_folder(owner: HWND, title: &str) -> Option<PathBuf> {
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
    use windows::Win32::UI::Shell::{FileOpenDialog, IFileOpenDialog, FOS_PICKFOLDERS, SIGDN_FILESYSPATH};
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let dlg: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).ok()?;
        let opts = dlg.GetOptions().ok()?;
        dlg.SetOptions(opts | FOS_PICKFOLDERS).ok()?;
        let t = phai_ui::win::wide(title);
        let _ = dlg.SetTitle(windows::core::PCWSTR(t.as_ptr()));
        dlg.Show(Some(owner)).ok()?;
        let item = dlg.GetResult().ok()?;
        let name = item.GetDisplayName(SIGDN_FILESYSPATH).ok()?;
        let path = name.to_string().ok();
        CoTaskMemFree(Some(name.0 as *const _));
        path.map(PathBuf::from)
    }
}

/// The Phai libraries a project may name, each with where this Phai Studio has it.
fn libraries() -> [(&'static str, PathBuf); 3] {
    [("phai-ui", phai_ui_path()), ("phai-data", phai_data_path()), ("phai-web", crate::web::phai_web_path())]
}

/// A project's Cargo.toml with each Phai library whose `path` leads nowhere - Phai Studio
/// uninstalled, put in another folder, or the project copied from another machine - named again
/// by `line_for` (the name to its new `{ .. }`); `None` when every path is still good.
pub fn fixed_libraries(toml: &str, exists: impl Fn(&Path) -> bool, line_for: impl Fn(&str) -> Option<String>) -> Option<String> {
    let mut changed = false;
    let lines: Vec<String> = toml
        .lines()
        .map(|l| {
            let Some((name, value)) = l.split_once('=') else { return l.to_string() };
            let name = name.trim();
            let value = value.trim();
            if !["phai-ui", "phai-data", "phai-web"].contains(&name) || !value.starts_with('{') {
                return l.to_string();
            }
            let Some(rest) = value.split("path").nth(1) else { return l.to_string() };
            let Some(path) = rest.split('"').nth(1) else { return l.to_string() };
            if exists(&Path::new(path).join("Cargo.toml")) {
                return l.to_string();
            }
            match line_for(name) {
                Some(new) => {
                    changed = true;
                    format!("{name} = {new}")
                }
                None => l.to_string(),
            }
        })
        .collect();
    let mut out = lines.join("\n");
    if toml.ends_with('\n') {
        out.push('\n');
    }
    changed.then_some(out)
}

/// Mends a project's Cargo.toml before a build (see [`fixed_libraries`]); whether it changed.
pub fn mend_libraries(project: &Path) -> bool {
    let file = project.join("Cargo.toml");
    let Ok(toml) = std::fs::read_to_string(&file) else { return false };
    let libs = libraries();
    let fixed = fixed_libraries(&toml, |p| p.is_file(), |name| libs.iter().find(|(n, _)| *n == name).map(|(_, local)| library_dependency(local)));
    match fixed {
        Some(t) => std::fs::write(&file, t).is_ok(),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A project made before 0.2.6: its form called `.phaiform` is found, renamed `.phf` (a plain
    /// rename outside git), and its `_ui.rs` written again to name the new file.
    #[test]
    fn an_older_project_s_forms_become_phf() {
        let dir = std::env::temp_dir().join(format!("phai-phf-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let form = create(&dir).unwrap();
        assert!(form.ends_with("main_window.phf"));
        let old = form.with_extension("phaiform");
        std::fs::rename(&form, &old).unwrap();
        assert_eq!(old_forms(&dir), vec![old.clone()]);
        let (done, failed) = rename_forms(&old_forms(&dir));
        assert!(failed.is_none(), "{failed:?}");
        assert_eq!(done, vec![(old.clone(), form.clone())]);
        assert!(form.is_file() && !old.exists());
        assert!(old_forms(&dir).is_empty());
        let ui = std::fs::read_to_string(dir.join("src").join("main_window_ui.rs")).unwrap();
        assert!(ui.starts_with("// Made by Phai Studio from main_window.phf"), "{}", ui.lines().next().unwrap_or(""));
        assert!(ui.contains("/src/main_window.phf"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_phone_app_s_form_is_whole_and_phone_sized() {
        let f = phone_form();
        assert!(f.check().is_empty(), "{:?}", f.check());
        assert_eq!(f.form.size[0], 390);
        // The code made for a new project answers its OK: the same control as a Windows program's.
        assert!(f.control.iter().any(|c| c.id == "ok" && c.on_click.as_deref() == Some("ok_clicked")));
        let dir = std::env::temp_dir().join(format!("phai-phone-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        create_with(&dir, true).unwrap();
        let toml = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        assert!(toml.contains("[package.metadata.phai-web]
orientation = \"portrait\""), "{toml}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_library_path_that_leads_nowhere_is_named_again() {
        let toml = "[package]\nname = \"t\"\n\n[dependencies]\nphai-ui = { path = \"C:/Gone/phai-ui\" }\nphai-data = { path = \"C:/Here/phai-data\" }\nregex = \"1\"\n\n[target.'cfg(target_arch = \"wasm32\")'.dependencies]\nphai-web = { path = \"C:/Gone/phai-web\" }\n";
        let here = |p: &Path| p.starts_with("C:/Here");
        let new = |n: &str| Some(format!("{{ path = \"D:/now/{n}\" }}"));
        let out = fixed_libraries(toml, here, new).expect("mended");
        assert!(out.contains("phai-ui = { path = \"D:/now/phai-ui\" }"), "{out}");
        assert!(out.contains("phai-web = { path = \"D:/now/phai-web\" }"), "{out}");
        assert!(out.contains("phai-data = { path = \"C:/Here/phai-data\" }"), "a good path stays: {out}");
        assert!(out.contains("regex = \"1\"") && out.ends_with('\n'), "{out}");
        let fine = "[dependencies]\nphai-ui = { path = \"C:/Here/phai-ui\" }\nphai-ui-git = { git = \"x\" }\n";
        assert_eq!(fixed_libraries(fine, here, new), None, "nothing to mend");
    }

    #[test]
    fn a_store_install_takes_the_libraries_from_git_at_its_release() {
        assert!(from_store(Path::new(r"C:\Program Files\WindowsApps\Monrak.PhaiStudio_0.1.3.0_x64__abc\phai-studio.exe")));
        assert!(!from_store(Path::new(r"C:\Users\me\AppData\Local\Programs\Phai Studio\phai-studio.exe")));
        let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("crates").join("phai-ui");
        let repo = "https://github.com/kirati-ai/phai-studio";
        let git = "{ git = \"https://github.com/kirati-ai/phai-studio\", tag = \"v0.1.3\" }";
        assert!(library_line(&here, repo, "0.1.3", false).starts_with("{ path = "), "the copy on this machine");
        assert_eq!(library_line(&here, repo, "0.1.3", true), git, "asked for git");
        assert_eq!(library_line(Path::new("nowhere"), repo, "0.1.3", false), git, "no copy here");
    }

    #[test]
    fn folder_names_become_package_names() {
        assert_eq!(package_name("My App"), "my_app");
        assert_eq!(package_name("2026 report"), "app_2026_report");
        assert_eq!(package_name("ร้านค้า"), "app_");
    }

    #[test]
    fn a_new_project_has_everything_and_its_form_reads() {
        let dir = std::env::temp_dir().join(format!("phai-new-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let form = create(&dir).unwrap();
        for f in ["Cargo.toml", "build.rs", "theme.css", "src/main.rs", "src/text.rs", "src/main_window_ui.rs", "src/main_window.rs"] {
            assert!(dir.join(f).is_file(), "{f}");
        }
        let parsed = FormFile::parse(&std::fs::read_to_string(&form).unwrap()).unwrap();
        assert_eq!(parsed, first_form());
        assert!(create(&dir).is_err(), "never over a project");
        // Every word the first form shows has Thai, so the new project's own test passes.
        phai_ui::text::add(Box::leak(Box::new([("My program", "โปรแกรมของฉัน"), ("Hello", "สวัสดี"), ("Ready", "พร้อม")])));
        assert!(TEXT_RS.contains("\"My program\"") && TEXT_RS.contains("\"Hello\"") && TEXT_RS.contains("\"Ready\""));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Makes a project in `PHAI_NEW_DIR`, for building it by hand: `cargo test -p phai-studio -- --ignored make_project`.
    #[test]
    #[ignore]
    fn make_project() {
        let dir = PathBuf::from(std::env::var("PHAI_NEW_DIR").expect("PHAI_NEW_DIR"));
        std::fs::create_dir_all(&dir).unwrap();
        create(&dir).unwrap();
    }
}
