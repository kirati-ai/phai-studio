//! How fast the code editor is with a long file: opening it, a page down drawn on the screen, going
//! to the end, and typing in the middle. `BENCH_LINES=100000 cargo run --release -p phai-editor
//! --example bench_editor` prints the times and closes; the window shows while it runs (painting
//! is part of what is measured).

use phai_ui::win;
use std::time::Instant;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::WindowsAndMessaging as wm;

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    win::def(h, msg, w, l)
}

/// Messages waiting handled, then the window drawn now.
fn settle(h: HWND) {
    unsafe {
        let mut msg = wm::MSG::default();
        while wm::PeekMessageW(&mut msg, None, 0, 0, wm::PM_REMOVE).as_bool() {
            let _ = wm::TranslateMessage(&msg);
            wm::DispatchMessageW(&msg);
        }
        let _ = Gdi::RedrawWindow(Some(h), None, None, Gdi::RDW_UPDATENOW | Gdi::RDW_ALLCHILDREN);
    }
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

/// Rust that looks like a real file: functions, comments, strings, nesting.
fn source(lines: usize) -> String {
    let block = [
        "/// Adds the totals of an order and says what it came to.",
        "pub fn total_{n}(items: &[(String, f64, u32)]) -> Result<f64, String> {",
        "    let mut sum = 0.0;",
        "    for (name, price, qty) in items {",
        "        if *qty == 0 {",
        "            return Err(format!(\"{name}: none ordered\"));",
        "        }",
        "        sum += price * *qty as f64; // ราคารวม",
        "    }",
        "    Ok(sum)",
        "}",
        "",
    ];
    let mut s = String::new();
    let mut i = 0;
    while i < lines {
        for l in block {
            s.push_str(&l.replace("{n}", &i.to_string()));
            s.push('\n');
            i += 1;
        }
    }
    s
}

fn main() {
    let n: usize = std::env::var("BENCH_LINES").ok().and_then(|v| v.parse().ok()).unwrap_or(10_000);
    win::init();
    let class = win::register("PhaiBenchEditor", Some(proc_), None);
    let title = win::wide("Editor bench");
    unsafe {
        let h = wm::CreateWindowExW(Default::default(), windows::core::PCWSTR(class.as_ptr()), windows::core::PCWSTR(title.as_ptr()), wm::WS_OVERLAPPEDWINDOW | wm::WS_VISIBLE, 40, 40, 1200, 900, None, None, Some(win::instance()), None).unwrap_or_default();
        settle(h);
        let text = source(n);

        let t = Instant::now();
        let ed = phai_editor::view::create(h, 10, &text, "bench.rs");
        win::move_to(ed, 0, 0, 1180, 850);
        let open = ms(t);
        let t = Instant::now();
        settle(h);
        let first_paint = ms(t);
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(ed));

        let pages = 60;
        let t = Instant::now();
        for _ in 0..pages {
            wm::SendMessageW(ed, wm::WM_KEYDOWN, Some(WPARAM(0x22 /* VK_NEXT */)), Some(LPARAM(0)));
            settle(h);
        }
        let page = ms(t) / pages as f64;

        // To the last line by the scroll bar, drawn.
        let t = Instant::now();
        wm::SendMessageW(ed, wm::WM_VSCROLL, Some(WPARAM(7 /* SB_BOTTOM */)), Some(LPARAM(0)));
        settle(h);
        let end = ms(t);

        // Typing in the middle of the file: back up a page, then letters one by one, each drawn.
        wm::SendMessageW(ed, wm::WM_VSCROLL, Some(WPARAM(6 /* SB_TOP */)), Some(LPARAM(0)));
        for _ in 0..(n / 2 / 40).max(1) {
            wm::SendMessageW(ed, wm::WM_KEYDOWN, Some(WPARAM(0x22)), Some(LPARAM(0)));
        }
        settle(h);
        let keys = 200;
        let t = Instant::now();
        for k in 0..keys {
            let c = if k % 20 == 19 { 0x0D } else { b'a' as usize + k % 26 };
            wm::SendMessageW(ed, wm::WM_CHAR, Some(WPARAM(c)), Some(LPARAM(0)));
            settle(h);
        }
        let key = ms(t) / keys as f64;

        println!("lines {n}: open {open:.0} ms | first paint {first_paint:.1} ms | page down {page:.1} ms each | to the end {end:.1} ms | a key typed {key:.1} ms each");
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let _ = wm::DestroyWindow(h);
    }
}
