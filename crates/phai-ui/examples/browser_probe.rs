//! The browser control in a window that is never shown: told an address while WebView2 is still
//! starting (as a program's `loaded` does), it must open that address once it is ready.
//!
//!   cargo run -p phai-ui --features browser --example browser_probe -- <url> [seconds] [trusted host]
//!
//! Needs WebView2Loader.dll beside the example. Prints what the control says it shows at the end.

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging as wm;

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe { wm::DefWindowProcW(h, msg, w, l) }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let url = args.get(1).cloned().unwrap_or_else(|| "about:blank".into());
    let seconds: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(8);

    let class = phai_ui::win::register("PhaiBrowserProbe", Some(proc_), None);
    let parent = unsafe {
        // WS_OVERLAPPEDWINDOW without WS_VISIBLE, far off the screen: nothing is ever drawn on it.
        wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_OVERLAPPEDWINDOW, -32000, -32000, 800, 600, None, None, Some(phai_ui::win::instance()), None).unwrap()
    };
    let page = phai_ui::browser::browser(parent, 1, "");
    // A third argument: the host whose self-made certificate is accepted.
    if let Some(host) = args.get(3) {
        phai_ui::browser::allow_certificate_errors_for(page, host);
    }
    unsafe {
        let _ = wm::MoveWindow(page, 0, 0, 800, 600, false);
    }
    // At once, before the control has had its start message: the race a form's loaded event runs.
    phai_ui::browser::navigate(page, &url);

    let until = std::time::Instant::now() + std::time::Duration::from_secs(seconds);
    let mut msg = wm::MSG::default();
    while std::time::Instant::now() < until {
        unsafe {
            while wm::PeekMessageW(&mut msg, None, 0, 0, wm::PM_REMOVE).as_bool() {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    println!("ready={} url={}", phai_ui::browser::ready(page), phai_ui::browser::url(page));
    unsafe {
        let _ = wm::DestroyWindow(parent);
    }
}
