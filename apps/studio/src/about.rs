//! Help > About: Phai Studio's icon, name and version; links to its website, its license terms,
//! its source and its issues; who makes it; the tools it works with and their versions (Rust,
//! Cargo, rust-analyzer, Git, Windows) - found in the background, copied in one click for a
//! problem report - and a button that looks for a newer Phai Studio.

use phai_ui::text::{t, tf};
use phai_ui::win;
use std::os::windows::process::CommandExt;
use std::path::Path;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::WindowsAndMessaging as wm;

const ID_OK: i32 = 1;
const ID_COPY: i32 = 3;
const ID_UPDATES: i32 = 4;
/// The links, in their order down the right.
const ID_LINKS: i32 = 10;
/// The versions found, handed from their thread: a `Box<Vec<(String, String)>>` in the LPARAM.
const WM_VERSIONS: u32 = wm::WM_APP + 1;

const REPO: &str = env!("CARGO_PKG_REPOSITORY");
const W: i32 = 640;
const M: i32 = 20;
const ICON: i32 = 48;

/// The links on the right: what they say, where they go.
fn links() -> [(&'static str, String); 4] {
    [
        ("phai-studio.monrak.net", crate::SITE.to_string()),
        (t("License terms"), format!("{REPO}/blob/main/LICENSE")),
        (t("Source code"), REPO.to_string()),
        (t("Report a problem"), format!("{REPO}/issues")),
    ]
}

struct About {
    list: HWND,
    icon: Option<wm::HICON>,
    /// Name and version of each tool, as the list shows them.
    rows: Vec<(String, String)>,
    updates: bool,
    done: bool,
}

fn state<'a>(h: HWND) -> Option<&'a mut About> {
    unsafe { win::state::<About>(h) }
}

/// Shows About over `owner` until it is closed; true when Check for updates was pressed.
pub fn show(owner: HWND) -> bool {
    let hwnd = make(owner);
    unsafe {
        win::enable(owner, false);
        win::show_top(hwnd);
        let mut msg = wm::MSG::default();
        // `PHAI_ASK_SHOT=<file.png>`: photographed once the versions are in, then closed.
        if let Some(to) = std::env::var_os("PHAI_ASK_SHOT") {
            for _ in 0..300 {
                while wm::PeekMessageW(&mut msg, None, 0, 0, wm::PM_REMOVE).as_bool() {
                    let _ = wm::TranslateMessage(&msg);
                    wm::DispatchMessageW(&msg);
                }
                if state(hwnd).is_some_and(|a| !a.rows.iter().any(|(_, v)| v == "...")) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            let _ = phai_ui::shot::save(hwnd, Path::new(&to));
            if let Some(a) = state(hwnd) {
                a.done = true;
            }
        }
        while state(hwnd).is_some_and(|a| !a.done) {
            if !wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
                wm::PostQuitMessage(msg.wParam.0 as i32);
                break;
            }
            if !wm::IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
        }
        let updates = state(hwnd).is_some_and(|a| a.updates);
        win::enable(owner, true);
        let _ = wm::SetForegroundWindow(owner);
        let _ = wm::DestroyWindow(hwnd);
        updates
    }
}

/// The window, filled, not yet shown; the versions arrive a moment later.
fn make(owner: HWND) -> HWND {
    win::init();
    let class = win::register("PhaiAbout", Some(proc_), None);
    let title = win::wide(t("About Phai Studio"));
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_DLGMODALFRAME, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let bold = win::font(hwnd, 0, true);
        let px = |v| win::px(hwnd, v);
        let at = |h: HWND, x: i32, y: i32, w: i32, ht: i32| win::move_to(h, px(x), px(y), px(w), px(ht));
        // The name beside the icon, the version under it.
        let x = M + ICON + 16;
        let name = win::label(hwnd, "Phai Studio", 0, win::font(hwnd, 16, false));
        at(name, x, M - 4, 300, 36);
        let version = win::label(hwnd, &tf("Version {0}", &[&env!("CARGO_PKG_VERSION")]), 0, font);
        at(version, x, M + 32, 300, 22);
        // The links, down the right.
        let lx = W - M - 190;
        for (i, (words, url)) in links().iter().enumerate() {
            let l = phai_ui::extra::link_label(hwnd, &format!("<a href=\"{url}\">{words}</a>"), ID_LINKS + i as i32, font);
            at(l, lx, M - 2 + i as i32 * 22, 190, 22);
        }
        // Words that run over more than one line: a label that wraps, from the top.
        let wrapping = |h: HWND| {
            let style = wm::GetWindowLongW(h, wm::GWL_STYLE);
            wm::SetWindowLongW(h, wm::GWL_STYLE, style & !0x200 /* SS_CENTERIMAGE */);
            h
        };
        let what = win::label(hwnd, t("A free RAD studio for Windows programs in Rust: draw the forms, style them with CSS, write what happens in Rust."), 0, font);
        wrapping(what);
        at(what, M, 104, W - 2 * M, 44);
        let who = win::label(hwnd, t("Copyright (c) 2026 Monrak Net (Kirati Kittikhun)"), 0, font);
        at(who, M, 152, W - 2 * M, 22);
        // The tools, and the buttons beside them.
        let works = win::label(hwnd, t("Works with:"), 0, bold);
        at(works, M, 186, 300, 22);
        let lw = W - 2 * M;
        let list = win::list(hwnd, 0, font, &[(t("Tool"), 180), (t("Version"), lw - 186)]);
        win::grid_lines(list);
        at(list, M, 210, lw, 172);
        // Under the list: copy what it says, look for a newer Phai Studio.
        let bw = 160;
        let copy = win::button(hwnd, t("Copy info"), ID_COPY, font, false);
        phai_ui::props::tooltip(copy, t("Every version here, for a problem report"));
        at(copy, M, 392, bw, 30);
        let updates = win::button(hwnd, t("Check for updates"), ID_UPDATES, font, false);
        at(updates, M + bw + 10, 392, bw, 30);
        // The license, then OK.
        let license = win::label(hwnd, t("Free to use for any work, programs you sell included. Phai Studio itself is under the PolyForm Perimeter License 1.0.1: it may not be made into a product that competes with it. The libraries that go into your programs are MIT. The Phai Studio name and bamboo logo are Monrak Net's."), 0, font);
        win::set_css(license, "muted", "");
        wrapping(license);
        at(license, M, 436, W - 2 * M, 66);
        let foot = 512;
        let ok = win::button(hwnd, t("OK"), ID_OK, font, true);
        at(ok, W - M - 100, foot, 100, 30);
        let rows: Vec<(String, String)> = tool_names().iter().map(|n| (n.to_string(), "...".to_string())).collect();
        for (n, v) in &rows {
            win::list_add(list, &[n, v]);
        }
        let icon = phai_ui::icon::app_icon(px(ICON));
        win::set_state(hwnd, Box::new(About { list, icon, rows, updates: false, done: false }));
        let mut r = RECT { left: 0, top: 0, right: px(W), bottom: px(foot + 30 + M) };
        let style = wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32;
        let exs = wm::GetWindowLongW(hwnd, wm::GWL_EXSTYLE) as u32;
        let _ = windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(style), false, wm::WINDOW_EX_STYLE(exs), win::dpi(hwnd) as u32);
        let _ = wm::SetWindowPos(hwnd, None, 0, 0, r.right - r.left, r.bottom - r.top, wm::SWP_NOMOVE | wm::SWP_NOZORDER);
        win::center_on(hwnd, owner);
        win::theme_frame(hwnd);
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(ok));
        // The versions: each asks a program, so not on the window's thread.
        let to = hwnd.0 as isize;
        std::thread::spawn(move || {
            let found = Box::into_raw(Box::new(versions()));
            // About closed meanwhile: nobody to take them.
            if wm::PostMessageW(Some(HWND(to as *mut _)), WM_VERSIONS, WPARAM(0), LPARAM(found as isize)).is_err() {
                drop(Box::from_raw(found));
            }
        });
        hwnd
    }
}

/// The tools the list names, in its order.
fn tool_names() -> [&'static str; 6] {
    ["Phai Studio", t("phai-ui (in your programs)"), t("Rust compiler"), "Cargo", "rust-analyzer", "Git"]
}

/// Each tool's version, as its program says it - "not found" when it is not on this computer - and
/// Windows' last.
fn versions() -> Vec<(String, String)> {
    let cargo = crate::cargo::find_cargo();
    let rustc = cargo.as_ref().map(|c| c.with_file_name("rustc.exe")).filter(|p| p.is_file());
    let ask = |exe: Option<&Path>, arg: &str| -> String {
        exe.and_then(|e| std::process::Command::new(e).arg(arg).creation_flags(0x0800_0000).output().ok())
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| t("not found").to_string())
    };
    let ours = env!("CARGO_PKG_VERSION").to_string();
    let names = tool_names();
    let mut out = vec![
        (names[0].to_string(), ours.clone()),
        (names[1].to_string(), ours),
        (names[2].to_string(), ask(rustc.as_deref(), "-V")),
        (names[3].to_string(), ask(cargo.as_deref(), "-V")),
        (names[4].to_string(), ask(phai_editor::lsp::find_server().as_deref(), "--version")),
        (names[5].to_string(), ask(Some(Path::new("git.exe")), "--version")),
    ];
    out.push(("Windows".to_string(), windows_version()));
    out
}

/// Windows as Settings > System > About names it: "Windows 11 Home 24H2 (build 26100.2605)".
fn windows_version() -> String {
    use windows::Win32::System::Registry as reg;
    let get = |name: &str| -> Option<String> {
        let key = win::wide(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion");
        let value = win::wide(name);
        let mut buf = [0u16; 256];
        let mut size = (buf.len() * 2) as u32;
        let mut kind = reg::REG_VALUE_TYPE(0);
        let ok = unsafe { reg::RegGetValueW(reg::HKEY_LOCAL_MACHINE, PCWSTR(key.as_ptr()), PCWSTR(value.as_ptr()), reg::RRF_RT_REG_SZ | reg::RRF_RT_REG_DWORD, Some(&mut kind), Some(buf.as_mut_ptr() as *mut _), Some(&mut size)) };
        if ok.is_err() {
            return None;
        }
        if kind == reg::REG_DWORD {
            return Some(u32::from_le_bytes([buf[0] as u8, (buf[0] >> 8) as u8, buf[1] as u8, (buf[1] >> 8) as u8]).to_string());
        }
        Some(String::from_utf16_lossy(&buf[..(size as usize / 2)]).trim_end_matches('\0').to_string())
    };
    let build = get("CurrentBuild").unwrap_or_default();
    // Windows 11 still says "Windows 10" here: its builds start at 22000.
    let mut product = get("ProductName").unwrap_or_else(|| "Windows".into());
    if build.parse::<u32>().is_ok_and(|b| b >= 22000) {
        product = product.replace("Windows 10", "Windows 11");
    }
    let release = get("DisplayVersion").unwrap_or_default();
    let ubr = get("UBR").map(|u| format!(".{u}")).unwrap_or_default();
    format!("{product} {release} (build {build}{ubr})").replace("  ", " ")
}

/// Everything About says, as text for a problem report.
fn info_text(rows: &[(String, String)]) -> String {
    rows.iter().map(|(n, v)| format!("{n}: {v}")).collect::<Vec<_>>().join("\r\n") + "\r\n"
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(a) = state(h) else { return win::def(h, msg, w, l) };
    match msg {
        WM_VERSIONS => {
            let found = unsafe { Box::from_raw(l.0 as *mut Vec<(String, String)>) };
            a.rows = *found;
            unsafe {
                wm::SendMessageW(a.list, windows::Win32::UI::Controls::LVM_DELETEALLITEMS, None, None);
            }
            for (n, v) in &a.rows {
                win::list_add(a.list, &[n, v]);
            }
            LRESULT(0)
        }
        wm::WM_PAINT => {
            let mut ps = Gdi::PAINTSTRUCT::default();
            unsafe {
                let dc = Gdi::BeginPaint(h, &mut ps);
                if let Some(icon) = a.icon {
                    let (m, size) = (win::px(h, M), win::px(h, ICON));
                    let _ = wm::DrawIconEx(dc, m, m, icon, size, size, 0, None, wm::DI_NORMAL);
                }
                let _ = Gdi::EndPaint(h, &ps);
            }
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const windows::Win32::UI::Controls::NMHDR) };
            // NM_CLICK / NM_RETURN on a link: its address in the browser.
            let code = hdr.code as i32;
            let i = hdr.idFrom as i32 - ID_LINKS;
            if (code == -2 || code == -4) && (0..links().len() as i32).contains(&i) {
                crate::open_with_windows(&links()[i as usize].1);
                return LRESULT(0);
            }
            win::def(h, msg, w, l)
        }
        wm::WM_COMMAND => {
            match win::loword(w.0) as i32 {
                ID_OK | 2 /* IDCANCEL: Esc */ => a.done = true,
                ID_COPY => {
                    crate::copy_text(h, &info_text(&a.rows));
                }
                ID_UPDATES => {
                    a.updates = true;
                    a.done = true;
                }
                _ => return win::def(h, msg, w, l),
            }
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            a.done = true;
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            if let Some(icon) = a.icon {
                let _ = unsafe { wm::DestroyIcon(icon) };
            }
            drop(unsafe { win::take_state::<About>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_info_copied_is_one_line_a_tool() {
        let rows = vec![("Phai Studio".to_string(), "0.2.8".to_string()), ("Cargo".to_string(), "cargo 1.90.0".to_string())];
        assert_eq!(info_text(&rows), "Phai Studio: 0.2.8\r\nCargo: cargo 1.90.0\r\n");
    }

    #[test]
    fn windows_is_named_with_its_build() {
        let v = windows_version();
        assert!(v.starts_with("Windows"), "{v}");
        assert!(v.contains("(build "), "{v}");
    }
}
