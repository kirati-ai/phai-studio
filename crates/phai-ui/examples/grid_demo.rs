//! A data grid with rows to filter, sort, total and export - the place to see the grid's features
//! at work. `cargo run -p phai-ui --example grid_demo` opens it; with `PHAI_SHOT=out.png` it
//! photographs itself and exits (`GRID_DEMO=filter` filters Status to Paid first,
//! `GRID_DEMO=popup` photographs the Status heading's filter box, `GRID_DEMO=detail` opens the
//! first row's details).

#![windows_subsystem = "windows"]

use phai_ui::grid::{self, Column, Summary};
use phai_ui::win;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging as wm;

const ID_GRID: i32 = 10;
const TIMER_SHOT: usize = 1;
const TIMER_POPUP: usize = 2;

static GRID: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);
static DETAIL: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

fn grid_of() -> HWND {
    HWND(GRID.load(std::sync::atomic::Ordering::SeqCst) as *mut _)
}

fn mode() -> String {
    std::env::var("GRID_DEMO").unwrap_or_default()
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_SIZE => {
            let m = win::px(h, 12);
            let (cw, ch) = (win::loword(l.0 as usize) as i32, win::hiword(l.0 as usize) as i32);
            let d = HWND(DETAIL.load(std::sync::atomic::Ordering::SeqCst) as *mut _);
            if d.is_invalid() {
                win::move_to(grid_of(), m, m, cw - 2 * m, ch - 2 * m);
            } else {
                let top = (ch - 3 * m) / 2;
                win::move_to(grid_of(), m, m, cw - 2 * m, top);
                win::move_to(d, m, 2 * m + top, cw - 2 * m, ch - 3 * m - top);
            }
            LRESULT(0)
        }
        wm::WM_TIMER if w.0 == TIMER_POPUP => {
            unsafe {
                let _ = wm::KillTimer(Some(h), TIMER_POPUP);
            }
            // The filter box of Status, photographed from inside its own loop, then closed.
            unsafe {
                let _ = wm::SetTimer(Some(h), TIMER_SHOT, 600, None);
            }
            let g = grid_of();
            let mut r = RECT::default();
            unsafe {
                let _ = wm::GetWindowRect(g, &mut r);
            }
            let x = r.left + win::px(h, 300);
            let values: Vec<(String, String)> = ["Due", "Paid", "Void"].iter().map(|s| (s.to_string(), s.to_string())).collect();
            let keep = vec!["Paid".to_string()];
            let _ = phai_ui::gridfilter::ask(g, RECT { left: x - win::px(h, 20), top: r.top, right: x, bottom: r.top + win::px(h, 24) }, values, Some(&keep));
            LRESULT(0)
        }
        wm::WM_TIMER if w.0 == TIMER_SHOT => {
            let path = std::env::var("PHAI_SHOT").unwrap_or_default();
            let target = if mode() == "popup" {
                unsafe { wm::FindWindowW(windows::core::w!("PhaiGridFilter"), None) }.unwrap_or(h)
            } else {
                h
            };
            let _ = phai_ui::shot::save(target, std::path::Path::new(&path));
            unsafe {
                wm::PostQuitMessage(0);
            }
            LRESULT(0)
        }
        wm::WM_DESTROY => {
            unsafe {
                wm::PostQuitMessage(0);
            }
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}

fn main() {
    win::init();
    let class = win::register("PhaiGridDemo", Some(proc_), None);
    let title = win::wide("Grid demo");
    let quiet = std::env::var_os("PHAI_SHOT").is_some();
    unsafe {
        let h = wm::CreateWindowExW(
            Default::default(),
            windows::core::PCWSTR(class.as_ptr()),
            windows::core::PCWSTR(title.as_ptr()),
            wm::WS_OVERLAPPEDWINDOW,
            40,
            40,
            1400,
            640,
            None,
            None,
            Some(win::instance()),
            None,
        )
        .unwrap_or_default();
        let font = win::font(h, 0, false);
        let g = grid::grid(h, ID_GRID, font, &[("Invoice", 90), ("Customer", 200), ("Status", 110), ("Amount", 120), ("Date", 120)]);
        GRID.store(g.0 as isize, std::sync::atomic::Ordering::SeqCst);
        let rows: Vec<Vec<String>> = [
            ["INV-001", "Monrak Net", "Paid", "12500", "2026-11-02"],
            ["INV-002", "Hoei Metal", "Due", "8400.5", "2026-11-05"],
            ["INV-003", "Siam Steel", "Paid", "1500", "2026-11-09"],
            ["INV-004", "บริษัท ไผ่ทอง จำกัด", "Void", "0", "2026-11-11"],
            ["INV-005", "Monrak Net", "Due", "22000", "2026-11-15"],
            ["INV-006", "Chiang Mai Foods", "Paid", "3200", "2026-11-18"],
        ]
        .iter()
        .map(|r| r.iter().map(|s| s.to_string()).collect())
        .collect();
        grid::set_column(g, 3, Column { align: grid::Align::Right, format: "n2".into(), summary: Summary::Sum, ..Column::default() });
        grid::set_column(g, 4, Column { format: "d".into(), ..Column::default() });
        grid::set_show_footer(g, true);
        grid::grid_fill(g, &rows);
        // GRID_DEMO=export: the rows written to grid_demo.xlsx and grid_demo.pdf beside PHAI_SHOT.
        if mode() == "export" {
            let shot = std::path::PathBuf::from(std::env::var("PHAI_SHOT").unwrap_or_default());
            let _ = grid::export_xlsx(g, &shot.with_extension("xlsx"));
            let pages = grid::export_pdf(g, &shot.with_extension("pdf"), "Invoices");
            println!("pdf pages: {pages}");
        }
        if mode() == "filter" {
            grid::set_column_filter(g, 2, Some(vec!["Paid".into()]));
        }
        // GRID_DEMO=detail: the invoices' lines under them, following the chosen invoice.
        if mode() == "detail" {
            let d = grid::grid(h, ID_GRID + 1, font, &[("Invoice", 90), ("Item", 220), ("Qty", 60), ("Price", 100)]);
            DETAIL.store(d.0 as isize, std::sync::atomic::Ordering::SeqCst);
            let lines: Vec<Vec<String>> = [
                ["INV-001", "Hosting, a year", "1", "9000"],
                ["INV-001", "Domain", "2", "1750"],
                ["INV-002", "Steel sheet 2 mm", "12", "700.04"],
                ["INV-003", "Support, an hour", "3", "500"],
            ]
            .iter()
            .map(|r| r.iter().map(|s| s.to_string()).collect())
            .collect();
            grid::set_column(d, 3, Column { align: grid::Align::Right, format: "n2".into(), ..Column::default() });
            grid::grid_fill(d, &lines);
            grid::set_master(d, g, 0, 0);
            win::list_select(g, 0);
        }
        let _ = wm::ShowWindow(h, if quiet { wm::SW_SHOWNOACTIVATE } else { wm::SW_SHOW });
        if quiet {
            let _ = wm::SetTimer(Some(h), if mode() == "popup" { TIMER_POPUP } else { TIMER_SHOT }, 700, None);
        }
        let mut msg = wm::MSG::default();
        while wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = wm::TranslateMessage(&msg);
            wm::DispatchMessageW(&msg);
        }
    }
}
