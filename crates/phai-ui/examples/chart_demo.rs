//! Every kind of chart at once: `cargo run -p phai-ui --example chart_demo`. With
//! `PHAI_SHOT=out.png` it photographs itself and exits (`CHART_HOT=k` points the mouse at
//! category k of each chart first, to show the tip). `CHART_DARK=1` draws it dark.

#![windows_subsystem = "windows"]

use phai_ui::chart;
use phai_ui::win;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging as wm;

static CHARTS: std::sync::Mutex<Vec<isize>> = std::sync::Mutex::new(Vec::new());
const TIMER_LIVE: usize = 1;
const TIMER_SHOT: usize = 2;
const TIMER_HOT: usize = 3;

fn layout(h: HWND) {
    let mut r = windows::Win32::Foundation::RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let charts = CHARTS.lock().unwrap().clone();
    let (cols, rows) = (3, 3);
    let m = win::px(h, 10);
    let (cw, ch) = ((r.right - m) / cols, (r.bottom - m) / rows);
    for (i, c) in charts.iter().enumerate() {
        let (x, y) = (i as i32 % cols, i as i32 / cols);
        win::move_to(HWND(*c as *mut _), m + x * cw, m + y * ch, cw - m, ch - m);
    }
}

/// Category k of each chart pointed at, so the tips show in the picture.
fn point_at(k: usize) {
    for c in CHARTS.lock().unwrap().iter() {
        chart::highlight(HWND(*c as *mut _), Some(k));
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_SIZE => {
            layout(h);
            LRESULT(0)
        }
        wm::WM_TIMER if w.0 == TIMER_LIVE => {
            // A price that moves: one more point a tick, the last 40 kept.
            static mut T: f64 = 0.0;
            unsafe {
                T += 1.0;
                let v = 100.0 + (T * 0.35).sin() * 8.0 + (T * 0.11).cos() * 5.0;
                let live = HWND(CHARTS.lock().unwrap()[8] as *mut _);
                chart::add_point(live, &format!("{}", T as i32), &[v]);
            }
            LRESULT(0)
        }
        wm::WM_TIMER if w.0 == TIMER_HOT => {
            unsafe {
                let _ = wm::KillTimer(Some(h), TIMER_HOT);
            }
            if let Some(k) = std::env::var("CHART_HOT").ok().and_then(|v| v.parse::<usize>().ok()) {
                point_at(k);
            }
            LRESULT(0)
        }
        wm::WM_TIMER if w.0 == TIMER_SHOT => {
            let path = std::env::var("PHAI_SHOT").unwrap_or_default();
            let _ = phai_ui::shot::save(h, std::path::Path::new(&path));
            unsafe { wm::PostQuitMessage(0) };
            LRESULT(0)
        }
        wm::WM_DESTROY => {
            unsafe { wm::PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}

fn main() {
    win::init();
    if std::env::var_os("CHART_DARK").is_some() {
        phai_ui::theme::set(phai_ui::theme::Theme::dark());
    }
    let class = win::register("PhaiChartDemo", Some(proc_), None);
    let title = win::wide("Charts");
    unsafe {
        let h = wm::CreateWindowExW(Default::default(), windows::core::PCWSTR(class.as_ptr()), windows::core::PCWSTR(title.as_ptr()), wm::WS_OVERLAPPEDWINDOW, 40, 40, 1500, 980, None, None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(h, 0, false);
        let sales = "Month, Sales, Cost\nJan, 120, 80\nFeb, 150, 95\nMar, 135, 90\nApr, 180, 110\nMay, 210, 120\nJun, 195, 130";
        let make = |id: i32, kind: &str, title: &str, data: &str| {
            let c = chart::chart(h, id, font);
            chart::set_type(c, kind);
            chart::set_title(c, title);
            chart::set_data_text(c, data);
            CHARTS.lock().unwrap().push(c.0 as isize);
            c
        };
        make(10, "column", "Sales and cost by month", sales);
        make(11, "line", "Visitors this week", "Day, This week, Last week\nMon, 820, 700\nTue, 932, 760\nWed, 901, 880\nThu, 1290, 940\nFri, 1330, 1100\nSat, 1520, 1180\nSun, 1450, 1210");
        make(12, "area", "Orders", "Week, Orders\nW1, 32\nW2, 41\nW3, 38\nW4, 55\nW5, 61\nW6, 58\nW7, 72\nW8, 80");
        make(13, "pie", "Where customers come from", "Source, Customers\nSearch, 420\nFacebook, 310\nLINE, 260\nFriends, 120\nOther, 60");
        make(14, "doughnut", "Spending", "Item, Baht\nRent, 12000\nFood, 8000\nTravel, 3500\nBills, 2500");
        make(15, "bar", "Sales by branch", "สาขา, ยอดขาย\nเชียงใหม่, 48200\nกรุงเทพ, 91500\nขอนแก่น, 35100\nภูเก็ต, 52700\nหาดใหญ่, 28900");
        let mut candles = String::from("Day, Open, High, Low, Close\n");
        let mut p = 100.0f64;
        for d in 1..=30 {
            let o = p;
            let c = o + ((d as f64 * 1.7).sin() * 3.0) + ((d as f64 * 0.6).cos() * 1.5);
            let hi = o.max(c) + 1.0 + (d % 3) as f64;
            let lo = o.min(c) - 1.0 - (d % 4) as f64 * 0.5;
            candles.push_str(&format!("{d}, {o:.2}, {hi:.2}, {lo:.2}, {c:.2}\n"));
            p = c;
        }
        make(16, "candlestick", "PHAI - 30 days", &candles);
        make(17, "column", "Temperature (°C)", "City, Min, Max\nBKK, 25, 34\nCNX, 15, 29\nHKT, 24, 31\nKKC, 19, 33");
        let live = make(18, "line", "Live price", "Time, Price");
        chart::set_max_points(live, 40);
        for t in 0..40 {
            let t = t as f64 - 40.0;
            chart::add_point(live, &format!("{}", t as i32), &[100.0 + (t * 0.35).sin() * 8.0 + (t * 0.11).cos() * 5.0]);
        }
        layout(h);
        let _ = wm::ShowWindow(h, wm::SW_SHOW);
        win::theme_frame(h);
        // `CHART_HOT=k`: the mouse over category k of each once they are drawn, for the tips.
        wm::SetTimer(Some(h), TIMER_HOT, 300, None);
        if std::env::var_os("PHAI_SHOT").is_some() {
            wm::SetTimer(Some(h), TIMER_SHOT, 800, None);
        } else {
            wm::SetTimer(Some(h), TIMER_LIVE, 500, None);
        }
        win::run_loop(Some(h));
    }
}
