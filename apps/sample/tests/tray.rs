//! `phai-sample --tray` run for real, told what to do by messages only - no mouse, no keys:
//! its window starts hidden, its icon comes back after Explorer's TaskbarCreated, the icon's
//! double-click and a click on its notice bring the window up, the close box only hides it, Exit
//! ends it.
//! `cargo test -p phai-sample --test tray -- --ignored`

use std::time::{Duration, Instant};

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::Shell as shell;
use windows::Win32::UI::WindowsAndMessaging as wm;

/// The program's window of class `class`, once it is there.
fn window_of(pid: u32, class: &str) -> Option<HWND> {
    struct Look {
        pid: u32,
        class: Vec<u16>,
        found: Option<HWND>,
    }
    unsafe extern "system" fn each(h: HWND, l: LPARAM) -> windows::core::BOOL {
        let look = unsafe { &mut *(l.0 as *mut Look) };
        let mut pid = 0;
        unsafe { wm::GetWindowThreadProcessId(h, Some(&mut pid)) };
        let mut buf = [0u16; 64];
        let n = unsafe { wm::GetClassNameW(h, &mut buf) } as usize;
        if pid == look.pid && buf[..n] == look.class[..] {
            look.found = Some(h);
            return false.into();
        }
        true.into()
    }
    let mut look = Look { pid, class: class.encode_utf16().collect(), found: None };
    let start = Instant::now();
    while look.found.is_none() && start.elapsed() < Duration::from_secs(10) {
        unsafe {
            let _ = wm::EnumWindows(Some(each), LPARAM(&mut look as *mut Look as isize));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    look.found
}

fn post(h: HWND, msg: u32, w: usize, l: isize) {
    unsafe {
        wm::PostMessageW(Some(h), msg, WPARAM(w), LPARAM(l)).unwrap();
    }
    std::thread::sleep(Duration::from_millis(400));
}

/// Whether window `h`'s icon `id` is in the corner: Windows changes only one that is there.
fn icon_there(h: HWND, id: u32) -> bool {
    let mut d = shell::NOTIFYICONDATAW { cbSize: std::mem::size_of::<shell::NOTIFYICONDATAW>() as u32, hWnd: h, uID: id, uFlags: shell::NIF_TIP, ..Default::default() };
    for (i, c) in "Phai Tray".encode_utf16().enumerate() {
        d.szTip[i] = c;
    }
    unsafe { shell::Shell_NotifyIconW(shell::NIM_MODIFY, &d).as_bool() }
}

fn visible(h: HWND) -> bool {
    unsafe { wm::IsWindowVisible(h).as_bool() }
}

#[test]
#[ignore = "runs the program; it puts an icon in the corner for a moment"]
fn the_tray_program_starts_hidden_and_answers_its_icon() {
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_phai-sample")).arg("--tray").spawn().unwrap();
    let h = window_of(child.id(), "Phai.TrayWindow").expect("its window is made");
    std::thread::sleep(Duration::from_millis(500));
    let ok = std::panic::catch_unwind(|| {
        assert!(!visible(h), "start_hidden: nothing on the screen");
        // Explorer restarted: its icons gone, then TaskbarCreated - the hidden window puts its
        // icon back.
        assert!(icon_there(h, 2003), "its icon is in the corner");
        let d = shell::NOTIFYICONDATAW { cbSize: std::mem::size_of::<shell::NOTIFYICONDATAW>() as u32, hWnd: h, uID: 2003, ..Default::default() };
        assert!(unsafe { shell::Shell_NotifyIconW(shell::NIM_DELETE, &d).as_bool() });
        assert!(!icon_there(h, 2003), "gone, as when Explorer stops");
        let created = unsafe { wm::RegisterWindowMessageW(windows::core::w!("TaskbarCreated")) };
        post(h, created, 0, 0);
        assert!(icon_there(h, 2003), "back after TaskbarCreated");
        // The ids its tray_window_ui.rs gives: the icon, Exit.
        let (tray, exit) = (2003, 1007);
        let wm_tray = wm::WM_APP + 0x3E5;
        post(h, wm_tray, tray, wm::WM_LBUTTONDBLCLK as isize);
        assert!(visible(h), "the icon's double-click opens it");
        post(h, wm::WM_CLOSE, 0, 0);
        assert!(!visible(h), "the close box only hides it");
        post(h, wm_tray, tray, (wm::WM_USER + 5) as isize);
        assert!(visible(h), "a click on its notice opens it");
        post(h, wm::WM_COMMAND, exit, 0);
    });
    let start = Instant::now();
    let mut ended = false;
    while start.elapsed() < Duration::from_secs(5) {
        if child.try_wait().unwrap().is_some() {
            ended = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    if !ended {
        let _ = child.kill();
    }
    let _ = child.wait();
    ok.unwrap();
    assert!(ended, "Exit ends the program");
}
