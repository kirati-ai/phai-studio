//! Phai's setup program. Its own file carries what it installs (see `phai-pack`): it reads itself,
//! asks where (`%LOCALAPPDATA%\Programs\<name>` by default - for this user, so no administrator is
//! needed), writes the files, makes Start menu and desktop shortcuts, and puts the program in
//! Windows' list of apps with an uninstaller.
//!
//! `setup.exe [/S] [/D=folder] [/NOSHORTCUTS] [/NOREG]` installs; the uninstaller it leaves is
//! the same program, run as `uninstall.exe /uninstall [/S]`.
#![windows_subsystem = "windows"]

use std::path::{Path, PathBuf};

use phai_pack::Manifest;
use phai_ui::win;
use windows::core::{Interface, HSTRING, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, IPersistFile, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
use windows::Win32::System::Registry as reg;
use windows::Win32::UI::Shell::{IShellLinkW, SHGetKnownFolderPath, ShellLink, FOLDERID_CommonPrograms, FOLDERID_Desktop, FOLDERID_ProgramFiles, FOLDERID_Programs, FOLDERID_PublicDesktop, KF_FLAG_DEFAULT};
use windows::Win32::UI::WindowsAndMessaging as wm;

fn thai() -> bool {
    unsafe { windows::Win32::Globalization::GetUserDefaultUILanguage() == 0x041E }
}

/// The words, Thai on a Thai Windows.
fn tr(en: &'static str) -> &'static str {
    if !thai() {
        return en;
    }
    match en {
        "Install {0} {1}" => "ติดตั้ง {0} {1}",
        "Installing for everyone needs an administrator." => "การติดตั้งให้ทุกคนในเครื่องต้องใช้สิทธิ์ผู้ดูแลระบบ",
        "From {0}" => "จาก {0}",
        "Install to:" => "ติดตั้งที่:",
        "A shortcut in the Start menu" => "ทางลัดในเมนู Start",
        "A shortcut on the desktop" => "ทางลัดบนเดสก์ท็อป",
        "Install" => "ติดตั้ง",
        "Cancel" => "ยกเลิก",
        "Installing..." => "กำลังติดตั้ง...",
        "{0} is installed." => "ติดตั้ง {0} เรียบร้อยแล้ว",
        "Start {0} now" => "เปิด {0} เลย",
        "Finish" => "เสร็จสิ้น",
        "Setup" => "ติดตั้งโปรแกรม",
        "This setup carries nothing to install." => "ตัวติดตั้งนี้ไม่มีโปรแกรมให้ติดตั้ง",
        "Remove {0} from this computer?" => "ถอนการติดตั้ง {0} ออกจากเครื่องนี้?",
        "{0} has been removed." => "ถอนการติดตั้ง {0} แล้ว",
        "Could not install: {0}" => "ติดตั้งไม่สำเร็จ: {0}",
        _ => en,
    }
}

fn fill(t: &str, args: &[&str]) -> String {
    let mut s = t.to_string();
    for (i, a) in args.iter().enumerate() {
        s = s.replace(&format!("{{{i}}}"), a);
    }
    s
}

fn known_folder(id: &windows::core::GUID) -> Option<PathBuf> {
    unsafe {
        let p = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None).ok()?;
        let s = p.to_string().ok();
        windows::Win32::System::Com::CoTaskMemFree(Some(p.0 as _));
        s.map(PathBuf::from)
    }
}

fn shortcut_paths(m: &Manifest) -> (Option<PathBuf>, Option<PathBuf>) {
    let name = format!("{}.lnk", m.safe_name());
    // For everyone: the Start menu and desktop every user sees.
    let (programs, desktop) = if m.all_users { (&FOLDERID_CommonPrograms, &FOLDERID_PublicDesktop) } else { (&FOLDERID_Programs, &FOLDERID_Desktop) };
    (known_folder(programs).map(|d| d.join(&name)), known_folder(desktop).map(|d| d.join(&name)))
}

/// Where the list of apps keeps it: this user's part of the registry, or the computer's.
fn root(m: &Manifest) -> reg::HKEY {
    if m.all_users {
        reg::HKEY_LOCAL_MACHINE
    } else {
        reg::HKEY_CURRENT_USER
    }
}

/// Whether this program runs as an administrator.
fn elevated() -> bool {
    unsafe { windows::Win32::UI::Shell::IsUserAnAdmin().as_bool() }
}

/// For everyone, and not an administrator yet: run again asking Windows for one (its own
/// prompt, answered by the person). True when this one should stop.
fn elevate(me: &Path, args: &[String]) -> bool {
    let params: Vec<String> = args.iter().skip(1).map(|a| if a.contains(' ') { format!("\"{a}\"") } else { a.clone() }).collect();
    let r = unsafe { windows::Win32::UI::Shell::ShellExecuteW(None, &HSTRING::from("runas"), &HSTRING::from(me.as_os_str()), &HSTRING::from(params.join(" ")), PCWSTR::null(), wm::SW_SHOWNORMAL) };
    r.0 as isize > 32
}

fn make_shortcut(at: &Path, target: &Path, dir: &Path, about: &str) -> Result<(), String> {
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).map_err(|e| e.to_string())?;
        link.SetPath(&HSTRING::from(target.as_os_str())).map_err(|e| e.to_string())?;
        link.SetWorkingDirectory(&HSTRING::from(dir.as_os_str())).map_err(|e| e.to_string())?;
        link.SetDescription(&HSTRING::from(about)).map_err(|e| e.to_string())?;
        let file: IPersistFile = link.cast().map_err(|e| e.to_string())?;
        file.Save(&HSTRING::from(at.as_os_str()), true).map_err(|e| e.to_string())
    }
}

fn uninstall_key(m: &Manifest) -> String {
    format!("Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Phai-{}", m.safe_name())
}

fn set_str(key: reg::HKEY, name: &str, value: &str) {
    let v: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();
    let bytes = unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 2) };
    unsafe {
        let _ = reg::RegSetValueExW(key, &HSTRING::from(name), None, reg::REG_SZ, Some(bytes));
    }
}

fn set_dword(key: reg::HKEY, name: &str, value: u32) {
    unsafe {
        let _ = reg::RegSetValueExW(key, &HSTRING::from(name), None, reg::REG_DWORD, Some(&value.to_le_bytes()));
    }
}

/// The program in Windows' list of apps, removed by its uninstaller.
fn register(m: &Manifest, dir: &Path, size: u64) -> Result<(), String> {
    let mut key = reg::HKEY::default();
    let r = unsafe { reg::RegCreateKeyExW(root(m), &HSTRING::from(uninstall_key(m)), None, PCWSTR::null(), reg::REG_OPTION_NON_VOLATILE, reg::KEY_WRITE, None, &mut key, None) };
    if r.is_err() {
        return Err(format!("registry: {r:?}"));
    }
    let uninstall = dir.join("uninstall.exe");
    set_str(key, "DisplayName", &m.name);
    set_str(key, "DisplayVersion", &m.version);
    set_str(key, "Publisher", &m.publisher);
    set_str(key, "InstallLocation", &dir.to_string_lossy());
    set_str(key, "DisplayIcon", &dir.join(&m.exe).to_string_lossy());
    set_str(key, "UninstallString", &format!("\"{}\" /uninstall", uninstall.display()));
    set_str(key, "QuietUninstallString", &format!("\"{}\" /uninstall /S", uninstall.display()));
    set_dword(key, "EstimatedSize", (size / 1024) as u32);
    set_dword(key, "NoModify", 1);
    set_dword(key, "NoRepair", 1);
    unsafe {
        let _ = reg::RegCloseKey(key);
    }
    Ok(())
}

struct Options {
    dir: PathBuf,
    start_menu: bool,
    desktop: bool,
    registry: bool,
}

/// Everything installed: files, the uninstaller, shortcuts, the entry in the list of apps.
fn install(me: &[u8], m: &Manifest, files: &[(String, Vec<u8>)], o: &Options) -> Result<(), String> {
    std::fs::create_dir_all(&o.dir).map_err(|e| format!("{}: {e}", o.dir.display()))?;
    let mut size = 0u64;
    for (path, data) in files {
        let at = o.dir.join(path.replace('/', "\\"));
        if let Some(p) = at.parent() {
            std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        std::fs::write(&at, data).map_err(|e| format!("{}: {e}", at.display()))?;
        size += data.len() as u64;
    }
    // The uninstaller: this program again, with only the list of what to take away.
    let mut listed = m.clone();
    listed.files = files.iter().map(|(p, _)| p.clone()).collect();
    let uninstaller = phai_pack::pack(phai_pack::strip(me), &listed, &[]);
    std::fs::write(o.dir.join("uninstall.exe"), &uninstaller).map_err(|e| e.to_string())?;
    let exe = o.dir.join(&m.exe);
    let (start, desk) = shortcut_paths(m);
    if o.start_menu {
        if let Some(p) = start {
            make_shortcut(&p, &exe, &o.dir, &m.name)?;
        }
    }
    if o.desktop {
        if let Some(p) = desk {
            make_shortcut(&p, &exe, &o.dir, &m.name)?;
        }
    }
    if o.registry {
        register(m, &o.dir, size + uninstaller.len() as u64)?;
    }
    Ok(())
}

/// The uninstaller's work: run from a copy of itself in TEMP, so the folder can go too.
fn uninstall(m: &Manifest, dir: &Path, quiet: bool, wait_for: Option<u32>) {
    if let Some(pid) = wait_for {
        // The copy waits for the uninstaller in the folder to end.
        unsafe {
            if let Ok(p) = windows::Win32::System::Threading::OpenProcess(windows::Win32::System::Threading::PROCESS_SYNCHRONIZE, false, pid) {
                windows::Win32::System::Threading::WaitForSingleObject(p, 10_000);
                let _ = windows::Win32::Foundation::CloseHandle(p);
            }
        }
    }
    for f in &m.files {
        let _ = std::fs::remove_file(dir.join(f.replace('/', "\\")));
    }
    let _ = std::fs::remove_file(dir.join("uninstall.exe"));
    // Folders left empty go; anything the person put there stays.
    let mut dirs: Vec<PathBuf> = m.files.iter().filter_map(|f| dir.join(f.replace('/', "\\")).parent().map(Path::to_path_buf)).collect();
    dirs.push(dir.to_path_buf());
    dirs.sort_by_key(|d| std::cmp::Reverse(d.components().count()));
    for d in dirs {
        let _ = std::fs::remove_dir(d);
    }
    let (start, desk) = shortcut_paths(m);
    for p in [start, desk].into_iter().flatten() {
        let _ = std::fs::remove_file(p);
    }
    unsafe {
        let _ = reg::RegDeleteTreeW(root(m), &HSTRING::from(uninstall_key(m)));
    }
    if !quiet {
        win::message(None, &fill(tr("{0} has been removed."), &[&m.name]), tr("Setup"), wm::MB_ICONINFORMATION);
    }
}

// --- the window ------------------------------------------------------------------------------

const ID_DIR: i32 = 10;
const ID_START: i32 = 11;
const ID_DESK: i32 = 12;
const ID_RUN: i32 = 13;
const ID_OK: i32 = 1;
const ID_CANCEL: i32 = 2;

struct Wizard {
    me: Vec<u8>,
    m: Manifest,
    files: Vec<(String, Vec<u8>)>,
    title: HWND,
    from: HWND,
    label: HWND,
    dir: HWND,
    start: HWND,
    desk: HWND,
    run: HWND,
    ok: HWND,
    cancel: HWND,
    installed: bool,
    done: bool,
}

fn layout(h: HWND) {
    let Some(s) = (unsafe { win::state::<Wizard>(h) }) else { return };
    let mut r = windows::Win32::Foundation::RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let px = |v| win::px(h, v);
    let (m, row) = (px(16), px(24));
    let w = r.right - 2 * m;
    win::move_to(s.title, m, m, w, px(32));
    win::move_to(s.from, m, m + px(36), w, row);
    // No publisher: the rest moves up into its line.
    let y = m + if s.m.publisher.is_empty() { px(52) } else { px(80) };
    win::move_to(s.label, m, y, w, row);
    win::move_to(s.dir, m, y + px(26), w, row);
    win::move_to(s.start, m, y + px(66), w, row);
    win::move_to(s.desk, m, y + px(94), w, row);
    win::move_to(s.run, m, y + px(66), w, row);
    let (bw, bh) = (px(90), px(28));
    win::move_to(s.cancel, r.right - m - bw, r.bottom - m - bh, bw, bh);
    win::move_to(s.ok, r.right - m - bw * 2 - px(8), r.bottom - m - bh, bw, bh);
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(s) = (unsafe { win::state::<Wizard>(h) }) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            layout(h);
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            match win::loword(w.0) as i32 {
                ID_OK if s.installed => {
                    if win::is_checked(s.run) {
                        let dir = PathBuf::from(win::text_of(s.dir));
                        let _ = std::process::Command::new(dir.join(&s.m.exe)).current_dir(&dir).spawn();
                    }
                    s.done = true;
                }
                ID_OK => {
                    let o = Options { dir: PathBuf::from(win::text_of(s.dir).trim()), start_menu: win::is_checked(s.start), desktop: win::is_checked(s.desk), registry: true };
                    win::set_text(s.title, tr("Installing..."));
                    win::enable(s.ok, false);
                    match install(&s.me, &s.m, &s.files, &o) {
                        Ok(()) => {
                            s.installed = true;
                            win::set_text(s.title, &fill(tr("{0} is installed."), &[&s.m.name]));
                            for c in [s.label, s.dir, s.start, s.desk, s.cancel] {
                                win::show(c, false);
                            }
                            win::show(s.run, true);
                            win::set_text(s.ok, tr("Finish"));
                            win::enable(s.ok, true);
                        }
                        Err(e) => {
                            win::enable(s.ok, true);
                            win::set_text(s.title, &fill(tr("Install {0} {1}"), &[&s.m.name, &s.m.version]));
                            win::message(Some(h), &fill(tr("Could not install: {0}"), &[&e]), tr("Setup"), wm::MB_ICONWARNING);
                        }
                    }
                }
                ID_CANCEL => s.done = true,
                _ => {}
            }
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            s.done = true;
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}

fn wizard(me: Vec<u8>, m: Manifest, files: Vec<(String, Vec<u8>)>, dir: PathBuf) {
    win::init();
    let class = win::register("PhaiSetup", Some(proc_), None);
    let caption = HSTRING::from(fill(tr("Install {0} {1}"), &[&m.name, &m.version]));
    unsafe {
        let h = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), &caption, wm::WS_OVERLAPPED | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_MINIMIZEBOX, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, 100, 100, None, None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(h, 0, false);
        let big = win::font(h, 14, false);
        let title = win::label(h, &fill(tr("Install {0} {1}"), &[&m.name, &m.version]), 0, big);
        let from = win::label(h, &if m.publisher.is_empty() { String::new() } else { fill(tr("From {0}"), &[&m.publisher]) }, 0, font);
        let label = win::label(h, tr("Install to:"), 0, font);
        let dir_box = win::edit(h, &dir.to_string_lossy(), ID_DIR, font, false);
        let start = win::checkbox(h, tr("A shortcut in the Start menu"), ID_START, font, true);
        let desk = win::checkbox(h, tr("A shortcut on the desktop"), ID_DESK, font, m.desktop);
        let run = win::checkbox(h, &fill(tr("Start {0} now"), &[&m.name]), ID_RUN, font, true);
        win::show(run, false);
        let ok = win::button(h, tr("Install"), ID_OK, font, true);
        let cancel = win::button(h, tr("Cancel"), ID_CANCEL, font, false);
        win::set_state(h, Box::new(Wizard { me, m, files, title, from, label, dir: dir_box, start, desk, run, ok, cancel, installed: false, done: false }));
        let mut r = windows::Win32::Foundation::RECT { left: 0, top: 0, right: win::px(h, 520), bottom: win::px(h, 300) };
        let _ = wm::AdjustWindowRect(&mut r, wm::WS_OVERLAPPED | wm::WS_CAPTION | wm::WS_SYSMENU, false);
        let (w, hh) = (r.right - r.left, r.bottom - r.top);
        let (sw, sh) = (wm::GetSystemMetrics(wm::SM_CXSCREEN), wm::GetSystemMetrics(wm::SM_CYSCREEN));
        let _ = wm::SetWindowPos(h, None, (sw - w) / 2, (sh - hh) / 2, w, hh, wm::SWP_NOZORDER);
        win::theme_frame(h);
        win::show_top(h);
        // `PHAI_SHOT=out.png`: photographed, then ends - no keys or mouse taken.
        if std::env::var_os("PHAI_SHOT").is_some() {
            win::run_loop(Some(h));
            return;
        }
        let mut msg = wm::MSG::default();
        while win::state::<Wizard>(h).is_some_and(|s| !s.done) && wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if !wm::IsDialogMessageW(h, &msg).as_bool() {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
        }
    }
}

fn main() {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }
    let args: Vec<String> = std::env::args().collect();
    let flag = |f: &str| args.iter().any(|a| a.eq_ignore_ascii_case(f));
    let quiet = flag("/S");
    let Ok(me_path) = std::env::current_exe() else { return };
    let me = std::fs::read(&me_path).unwrap_or_default();
    let uninstalling = flag("/uninstall");
    let (m, files) = match phai_pack::unpack(&me, !uninstalling) {
        Ok(x) => x,
        Err(_) => {
            if !quiet {
                win::message(None, tr("This setup carries nothing to install."), tr("Setup"), wm::MB_ICONWARNING);
            }
            return;
        }
    };
    // For everyone: an administrator is needed - asked for once, by Windows, before anything.
    if m.all_users && !elevated() && std::env::var_os("PHAI_SETUP_NO_ELEVATE").is_none() {
        if elevate(&me_path, &args) {
            return;
        }
        if !quiet {
            win::message(None, tr("Installing for everyone needs an administrator."), tr("Setup"), wm::MB_ICONWARNING);
        }
        return;
    }
    if uninstalling {
        // From the program's folder: a copy in TEMP does the work, so the folder can go too.
        let dir_arg = args.iter().position(|a| a.eq_ignore_ascii_case("/uninstall")).and_then(|i| args.get(i + 1)).filter(|a| !a.starts_with('/')).map(PathBuf::from);
        match dir_arg {
            Some(dir) => {
                let wait = args.iter().find_map(|a| a.strip_prefix("/WAIT=")).and_then(|p| p.parse().ok());
                uninstall(&m, &dir, quiet, wait);
            }
            None => {
                let dir = me_path.parent().map(Path::to_path_buf).unwrap_or_default();
                if !quiet && win::message(None, &fill(tr("Remove {0} from this computer?"), &[&m.name]), tr("Setup"), wm::MB_YESNO | wm::MB_ICONQUESTION) != wm::IDYES {
                    return;
                }
                let copy = std::env::temp_dir().join(format!("phai-uninstall-{}.exe", std::process::id()));
                if std::fs::copy(&me_path, &copy).is_ok() {
                    let mut c = std::process::Command::new(&copy);
                    c.arg("/uninstall").arg(&dir).arg(format!("/WAIT={}", std::process::id()));
                    if quiet {
                        c.arg("/S");
                    }
                    let _ = c.spawn();
                } else {
                    uninstall(&m, &dir, quiet, None);
                }
            }
        }
        return;
    }
    let default_dir = if m.all_users {
        known_folder(&FOLDERID_ProgramFiles).unwrap_or_else(|| PathBuf::from(r"C:\Program Files")).join(m.safe_name())
    } else {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join("Programs").join(m.safe_name())
    };
    let dir = args.iter().find_map(|a| a.strip_prefix("/D=")).map(PathBuf::from).unwrap_or(default_dir);
    if quiet {
        let o = Options { dir, start_menu: !flag("/NOSHORTCUTS"), desktop: m.desktop && !flag("/NOSHORTCUTS"), registry: !flag("/NOREG") };
        if let Err(e) = install(&me, &m, &files, &o) {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }
    wizard(me, m, files, dir);
}

