//! How fast the data grid is with many rows: filling it, a page down drawn on the screen, sorting
//! and finding. `BENCH_ROWS=100000 cargo run --release -p phai-ui --example bench_grid` prints the
//! times and closes; the window shows while it runs (painting is part of what is measured).

use phai_ui::grid::{self, Align, Column, Summary};
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

fn main() {
    let n: usize = std::env::var("BENCH_ROWS").ok().and_then(|v| v.parse().ok()).unwrap_or(100_000);
    win::init();
    let class = win::register("PhaiBenchGrid", Some(proc_), None);
    let title = win::wide("Grid bench");
    unsafe {
        let h = wm::CreateWindowExW(Default::default(), windows::core::PCWSTR(class.as_ptr()), windows::core::PCWSTR(title.as_ptr()), wm::WS_OVERLAPPEDWINDOW | wm::WS_VISIBLE, 40, 40, 1200, 800, None, None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(h, 0, false);
        let g = grid::grid(h, 10, font, &[("Invoice", 100), ("Customer", 220), ("Status", 100), ("Amount", 120), ("Date", 120), ("Note", 300)]);
        win::move_to(g, 10, 10, 1160, 740);
        grid::set_column(g, 3, Column { align: Align::Right, format: "n2".into(), summary: Summary::Sum, ..Column::default() });
        grid::set_show_footer(g, true);
        settle(h);

        let t = Instant::now();
        let statuses = ["Paid", "Due", "Void"];
        let rows: Vec<Vec<String>> = (0..n)
            .map(|i| vec![format!("INV-{i:07}"), format!("Customer {}", (i * 7919) % 5000), statuses[i % 3].to_string(), format!("{}.{:02}", (i * 37) % 100_000, i % 100), format!("2026-{:02}-{:02}", i % 12 + 1, i % 28 + 1), format!("A note for row {i}")])
            .collect();
        let made = ms(t);

        let t = Instant::now();
        grid::grid_fill(g, &rows);
        let fill = ms(t);
        let t = Instant::now();
        settle(h);
        let first_paint = ms(t);

        // A page down, drawn - as a person paging through.
        let pages = 60;
        let t = Instant::now();
        for _ in 0..pages {
            wm::SendMessageW(g, wm::WM_KEYDOWN, Some(WPARAM(0x22 /* VK_NEXT */)), Some(LPARAM(0)));
            settle(h);
        }
        let page = ms(t) / pages as f64;

        // Straight to the end and back.
        let t = Instant::now();
        wm::SendMessageW(g, wm::WM_KEYDOWN, Some(WPARAM(0x23 /* VK_END */)), Some(LPARAM(0)));
        settle(h);
        wm::SendMessageW(g, wm::WM_KEYDOWN, Some(WPARAM(0x24 /* VK_HOME */)), Some(LPARAM(0)));
        settle(h);
        let ends = ms(t);

        let t = Instant::now();
        grid::sort_by(g, 1, true);
        settle(h);
        let sort = ms(t);

        let t = Instant::now();
        grid::find(g, "Customer 42");
        settle(h);
        let find = ms(t);

        println!("rows {n}: make data {made:.0} ms | fill {fill:.0} ms | first paint {first_paint:.1} ms | page down {page:.1} ms each | end+home {ends:.1} ms | sort {sort:.0} ms | find {find:.0} ms");
        // A moment for the memory to be read from outside.
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let _ = wm::DestroyWindow(h);
    }
}
