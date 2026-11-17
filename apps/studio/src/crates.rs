//! Manage Crates, as Visual Studio's NuGet Package Manager: the project's crates, and a search of
//! crates.io to add more - name, newest version, how many downloads, what it is. Add and Remove
//! are Cargo's own `cargo add` and `cargo remove`. The search goes through Windows' own curl.exe.

use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use phai_ui::text::{t, tf};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::WindowsAndMessaging as wm;

const ID_QUERY: i32 = 11;
const ID_FOUND: i32 = 12;
const ID_INSTALLED: i32 = 13;
const ID_SEARCH: i32 = 1;
const ID_CLOSE: i32 = 2;
const ID_ADD: i32 = 20;
const ID_REMOVE: i32 = 21;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// A crate on crates.io.
#[derive(Clone, Debug, PartialEq)]
pub struct Found {
    pub name: String,
    pub version: String,
    pub downloads: u64,
    pub about: String,
}

/// crates.io's answer to a search, read.
pub fn found_of(json: &str) -> Vec<Found> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(json) else { return vec![] };
    v["crates"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|c| {
                    Some(Found {
                        name: c["name"].as_str()?.to_string(),
                        version: c["max_stable_version"].as_str().or(c["max_version"].as_str()).unwrap_or("").to_string(),
                        downloads: c["downloads"].as_u64().unwrap_or(0),
                        about: c["description"].as_str().unwrap_or("").split_whitespace().collect::<Vec<_>>().join(" "),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The `[dependencies]` of a Cargo.toml: (name, what it asks for - a version or a path).
pub fn installed_of(toml: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut inside = false;
    for l in toml.lines().map(str::trim) {
        if l.starts_with('[') {
            inside = l == "[dependencies]";
            continue;
        }
        if !inside || l.is_empty() || l.starts_with('#') {
            continue;
        }
        let Some((name, rest)) = l.split_once('=') else { continue };
        let rest = rest.trim();
        let what = if rest.starts_with('"') {
            rest.trim_matches('"').to_string()
        } else if let Some(i) = rest.find("version") {
            rest[i..].split('"').nth(1).unwrap_or("").to_string()
        } else if rest.contains("path") {
            t("this computer").to_string()
        } else if rest.contains("workspace") {
            t("the workspace's").to_string()
        } else {
            rest.to_string()
        };
        out.push((name.trim().to_string(), what));
    }
    out
}

/// A count as people read it: 1.2M, 34K.
fn short(n: u64) -> String {
    match n {
        n if n >= 1_000_000 => format!("{:.1}M", n as f64 / 1e6),
        n if n >= 1_000 => format!("{}K", n / 1000),
        n => n.to_string(),
    }
}

/// crates.io searched for `query`: the best matches.
pub fn search(query: &str) -> Result<Vec<Found>, String> {
    let q: String = query.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c.to_string() } else { format!("%{:02X}", c as u32 & 0xFF) }).collect();
    let url = format!("https://crates.io/api/v1/crates?q={q}&per_page=40");
    let out = Command::new("curl.exe")
        .args(["-s", "-f", "--max-time", "20", "-A", "phai-studio (crate search)", &url])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("curl: {e}"))?;
    if !out.status.success() {
        return Err(t("crates.io could not be reached.").to_string());
    }
    Ok(found_of(&String::from_utf8_lossy(&out.stdout)))
}

struct Dialog {
    project: PathBuf,
    cargo: Option<PathBuf>,
    query: HWND,
    found: HWND,
    installed: HWND,
    labels: Vec<HWND>,
    buttons: Vec<(i32, HWND)>,
    say: HWND,
    results: Vec<Found>,
    deps: Vec<(String, String)>,
    changed: bool,
    done: bool,
}

fn state<'a>(h: HWND) -> Option<&'a mut Dialog> {
    unsafe { win::state::<Dialog>(h) }
}

/// Opens Manage Crates for the project in `project`; whether its crates changed.
pub fn run(owner: HWND, project: &Path, cargo: Option<PathBuf>) -> bool {
    win::init();
    let class = win::register("PhaiCrates", Some(proc_), None);
    let name = crate::cargo::package_name(project).unwrap_or_default();
    let title = win::wide(&tf("Manage Crates - {0}", &[&name]));
    unsafe {
        let hwnd = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_THICKFRAME | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let bold = win::font(hwnd, 0, true);
        let query = win::edit(hwnd, "", ID_QUERY, font, false);
        let found = win::list(hwnd, ID_FOUND, font, &[(t("Crate"), 170), (t("Version"), 80), (t("Downloads"), 80), (t("Description"), 420)]);
        let installed = win::list(hwnd, ID_INSTALLED, font, &[(t("Crate"), 170), (t("Version"), 140)]);
        let labels = vec![win::label(hwnd, t("Browse crates.io"), 0, bold), win::label(hwnd, t("Installed"), 0, bold)];
        let say = win::label(hwnd, "", 0, font);
        win::set_css(say, "muted", "");
        let mut buttons = Vec::new();
        for (id, label) in [(ID_SEARCH, t("Search")), (ID_ADD, t("Add")), (ID_REMOVE, t("Remove")), (ID_CLOSE, t("Close"))] {
            buttons.push((id, win::button(hwnd, label, id, font, id == ID_SEARCH)));
        }
        win::set_state(hwnd, Box::new(Dialog { project: project.to_path_buf(), cargo, query, found, installed, labels, buttons, say, results: vec![], deps: vec![], changed: false, done: false }));
        refresh_installed(hwnd);
        let (w, h) = (win::px(hwnd, 900), win::px(hwnd, 600));
        let mut r = RECT::default();
        let _ = wm::GetWindowRect(owner, &mut r);
        let _ = wm::SetWindowPos(hwnd, None, r.left + (r.right - r.left - w) / 2, r.top + (r.bottom - r.top - h) / 2, w, h, wm::SWP_NOZORDER);
        layout(hwnd);
        win::theme_frame(hwnd);
        // `PHAI_CRATES_TEST=name`: searched, the first found added, then removed - for testing.
        if let Ok(q) = std::env::var("PHAI_CRATES_TEST") {
            win::set_text(query, &q);
            do_search(hwnd);
            let first = state(hwnd).and_then(|d| d.results.first().cloned());
            println!("found: {}", state(hwnd).map(|d| d.results.iter().take(3).map(|f| format!("{} {}", f.name, f.version)).collect::<Vec<_>>().join(", ")).unwrap_or_default());
            if let Some(f) = first {
                cargo_run(hwnd, &["add", &f.name]);
                println!("after add: {:?}", state(hwnd).map(|d| d.deps.iter().map(|x| x.0.clone()).collect::<Vec<_>>()));
                cargo_run(hwnd, &["remove", &f.name]);
                println!("after remove: {:?}", state(hwnd).map(|d| d.deps.iter().map(|x| x.0.clone()).collect::<Vec<_>>()));
            }
            let _ = wm::DestroyWindow(hwnd);
            return true;
        }
        win::enable(owner, false);
        win::show_top(hwnd);
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(query));
        let mut msg = wm::MSG::default();
        while state(hwnd).is_some_and(|d| !d.done) {
            if !wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
                wm::PostQuitMessage(msg.wParam.0 as i32);
                break;
            }
            if !wm::IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
        }
        let changed = state(hwnd).is_some_and(|d| d.changed);
        win::enable(owner, true);
        let _ = wm::SetForegroundWindow(owner);
        let _ = wm::DestroyWindow(hwnd);
        changed
    }
}

fn refresh_installed(h: HWND) {
    let Some(d) = state(h) else { return };
    d.deps = installed_of(&std::fs::read_to_string(d.project.join("Cargo.toml")).unwrap_or_default());
    unsafe {
        wm::SendMessageW(d.installed, cc::LVM_DELETEALLITEMS, None, None);
    }
    for (n, v) in &d.deps {
        win::list_add(d.installed, &[n, v]);
    }
}

fn do_search(h: HWND) {
    let Some(d) = state(h) else { return };
    let q = win::text_of(d.query);
    if q.trim().is_empty() {
        return;
    }
    win::set_text(d.say, t("Searching crates.io..."));
    unsafe {
        let _ = windows::Win32::Graphics::Gdi::UpdateWindow(d.say);
    }
    match search(q.trim()) {
        Ok(found) => {
            unsafe {
                wm::SendMessageW(d.found, cc::LVM_DELETEALLITEMS, None, None);
            }
            for f in &found {
                win::list_add(d.found, &[&f.name, &f.version, &short(f.downloads), &f.about]);
            }
            if !found.is_empty() {
                win::list_select(d.found, 0);
            }
            win::set_text(d.say, &tf("{0} crates found", &[&found.len()]));
            d.results = found;
        }
        Err(e) => win::set_text(d.say, &e),
    }
}

fn chosen(list: HWND) -> Option<usize> {
    let i = unsafe { wm::SendMessageW(list, cc::LVM_GETNEXTITEM, Some(WPARAM(usize::MAX)), Some(LPARAM(2))).0 };
    (i >= 0).then_some(i as usize)
}

/// `cargo add` or `cargo remove` in the project; what Cargo said shown, the crates again.
fn cargo_run(h: HWND, args: &[&str]) {
    let Some(d) = state(h) else { return };
    let Some(cargo) = d.cargo.clone() else {
        win::set_text(d.say, t("Rust is not installed"));
        return;
    };
    win::set_text(d.say, &format!("cargo {}...", args.join(" ")));
    unsafe {
        let _ = windows::Win32::Graphics::Gdi::UpdateWindow(d.say);
    }
    let out = Command::new(cargo).args(args).current_dir(&d.project).creation_flags(CREATE_NO_WINDOW).output();
    let said = match out {
        Ok(o) => {
            let words = String::from_utf8_lossy(&o.stderr);
            let line = words.lines().map(str::trim).filter(|l| !l.is_empty()).last().unwrap_or("").to_string();
            if o.status.success() {
                d.changed = true;
                line
            } else {
                words.lines().find(|l| l.contains("error")).unwrap_or(&line).trim().to_string()
            }
        }
        Err(e) => e.to_string(),
    };
    win::set_text(d.say, &said);
    refresh_installed(h);
}

fn layout(h: HWND) {
    let Some(d) = state(h) else { return };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let px = |v| win::px(h, v);
    let (m, row, bw, gap) = (px(11), px(26), px(96), px(6));
    let button = |id: i32| d.buttons.iter().find(|(i, _)| *i == id).map(|(_, b)| *b).unwrap_or_default();
    let left_w = (r.right - 3 * m) * 64 / 100;
    let x2 = 2 * m + left_w;
    let right_w = r.right - x2 - m;
    win::move_to(d.labels[0], m, m, left_w, row);
    win::move_to(d.labels[1], x2, m, right_w, row);
    let y = m + row;
    win::move_to(d.query, m, y, left_w - bw - gap, row);
    win::move_to(button(ID_SEARCH), m + left_w - bw, y, bw, row);
    let list_top = y + row + gap;
    let list_h = r.bottom - list_top - row * 2 - m - gap * 2;
    win::move_to(d.found, m, list_top, left_w, list_h);
    win::move_to(d.installed, x2, y, right_w, list_h + row + gap);
    let by = list_top + list_h + gap;
    win::move_to(button(ID_ADD), m, by, bw, row + px(2));
    win::move_to(button(ID_REMOVE), x2, by, bw, row + px(2));
    win::move_to(button(ID_CLOSE), r.right - m - bw, r.bottom - m - row, bw, row);
    win::move_to(d.say, m, r.bottom - m - row, r.right - bw - 3 * m, row);
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(d) = state(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            layout(h);
            LRESULT(0)
        }
        wm::WM_GETMINMAXINFO => {
            let mm = unsafe { &mut *(l.0 as *mut wm::MINMAXINFO) };
            mm.ptMinTrackSize = POINT { x: win::px(h, 640), y: win::px(h, 420) };
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            match win::loword(w.0) as i32 {
                ID_SEARCH => do_search(h),
                ID_ADD => {
                    if let Some(f) = chosen(d.found).and_then(|i| d.results.get(i).cloned()) {
                        cargo_run(h, &["add", &f.name]);
                    }
                }
                ID_REMOVE => {
                    if let Some((n, _)) = chosen(d.installed).and_then(|i| d.deps.get(i).cloned()) {
                        cargo_run(h, &["remove", &n]);
                    }
                }
                ID_CLOSE => d.done = true,
                _ => return win::def(h, msg, w, l),
            }
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.hwndFrom == d.found && hdr.code == cc::NM_DBLCLK {
                if let Some(f) = chosen(d.found).and_then(|i| d.results.get(i).cloned()) {
                    cargo_run(h, &["add", &f.name]);
                }
                return LRESULT(0);
            }
            win::def(h, msg, w, l)
        }
        wm::WM_CLOSE => {
            d.done = true;
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crates_io_answers_and_dependencies_are_read() {
        let json = r#"{"crates":[{"name":"regex","max_version":"1.11.1","max_stable_version":"1.11.1","downloads":412000000,"description":"An implementation of\n regular expressions"}],"meta":{"total":1}}"#;
        let f = found_of(json);
        assert_eq!(f[0].name, "regex");
        assert_eq!(f[0].about, "An implementation of regular expressions");
        assert_eq!(short(f[0].downloads), "412.0M");
        let deps = installed_of("[package]\nname = \"a\"\n\n[dependencies]\nregex = \"1\"\nserde = { version = \"1.0\", features = [\"derive\"] }\nphai-ui = { path = \"../ui\" }\n\n[dev-dependencies]\nx = \"1\"\n");
        assert_eq!(deps.len(), 3, "dev-dependencies are not the program's");
        assert_eq!(deps[1], ("serde".to_string(), "1.0".to_string()));
    }
}
