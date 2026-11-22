//! Diagnostic Tools, as Visual Studio's: while the program runs, how much memory it holds and how
//! much of the processors it takes, sampled twice a second and drawn as two graphs.

use std::collections::VecDeque;

use phai_ui::theme::{self, Color, State};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, FILETIME, HANDLE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX};
use windows::Win32::System::Threading::{GetExitCodeProcess, GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::WindowsAndMessaging as wm;

/// How many samples a graph keeps: a minute at two a second.
const KEEP: usize = 120;

/// One running program, watched.
pub struct Sampler {
    process: HANDLE,
    pub pid: u32,
    last_cpu: u64,
    last_at: std::time::Instant,
    cpus: f32,
    pub cpu: VecDeque<f32>,
    pub mem: VecDeque<u64>,
    pub ended: bool,
}

fn ticks(f: FILETIME) -> u64 {
    ((f.dwHighDateTime as u64) << 32) | f.dwLowDateTime as u64
}

impl Sampler {
    /// Starts watching process `pid`; none when it cannot be opened.
    pub fn new(pid: u32) -> Option<Sampler> {
        let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ, false, pid) }.ok()?;
        let cpus = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1) as f32;
        let mut s = Sampler { process, pid, last_cpu: 0, last_at: std::time::Instant::now(), cpus, cpu: VecDeque::new(), mem: VecDeque::new(), ended: false };
        s.last_cpu = s.cpu_time().unwrap_or(0);
        Some(s)
    }

    fn cpu_time(&self) -> Option<u64> {
        let (mut c, mut e, mut k, mut u) = (FILETIME::default(), FILETIME::default(), FILETIME::default(), FILETIME::default());
        unsafe { GetProcessTimes(self.process, &mut c, &mut e, &mut k, &mut u) }.ok()?;
        Some(ticks(k) + ticks(u))
    }

    /// One sample: the processors' share since the last (all of them together 100%) and the
    /// private memory. False once the program has ended.
    pub fn sample(&mut self) -> bool {
        if self.ended {
            return false;
        }
        let mut code = 0u32;
        if unsafe { GetExitCodeProcess(self.process, &mut code) }.is_ok() && code != 259 /* STILL_ACTIVE */ {
            self.ended = true;
            return false;
        }
        let now = std::time::Instant::now();
        let cpu = self.cpu_time().unwrap_or(self.last_cpu);
        let wall = now.duration_since(self.last_at).as_secs_f32().max(0.001);
        let share = ((cpu.saturating_sub(self.last_cpu)) as f32 / 10_000_000.0 / wall / self.cpus * 100.0).clamp(0.0, 100.0);
        self.last_cpu = cpu;
        self.last_at = now;
        let mut m = PROCESS_MEMORY_COUNTERS_EX { cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32, ..Default::default() };
        let got = unsafe { GetProcessMemoryInfo(self.process, &mut m as *mut _ as *mut _, m.cb) };
        let mem = if got.is_ok() { m.PrivateUsage as u64 } else { 0 };
        self.cpu.push_back(share);
        self.mem.push_back(mem);
        while self.cpu.len() > KEEP {
            self.cpu.pop_front();
            self.mem.pop_front();
        }
        true
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.process);
        }
    }
}

/// The graphs' window, a tab of the bottom panel.
pub fn view(parent: HWND, id: i32) -> HWND {
    let class = win::register("PhaiDiagnostics", Some(proc_), None);
    unsafe { wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_CLIPSIBLINGS, 0, 0, 10, 10, Some(parent), Some(wm::HMENU(id as isize as *mut _)), Some(win::instance()), None).unwrap_or_default() }
}

fn mb(b: u64) -> String {
    format!("{:.1} MB", b as f64 / 1_048_576.0)
}

/// One graph in `r`: its title and the value now, a line of the samples, scaled to `top`.
fn graph(dc: Gdi::HDC, h: HWND, r: RECT, (title, now): (&str, &str), values: &[f32], top: f32, colour: Color) {
    let text = win::window_text();
    let muted = theme::current().var_color("--muted").unwrap_or(Color::rgb(0x80, 0x80, 0x80));
    let border = theme::style("group", State::default()).border_or(Color::rgb(0xd4, 0xd4, 0xd4));
    let row = win::px(h, 20);
    unsafe {
        Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(text.colorref()));
        let mut tr = RECT { bottom: r.top + row, ..r };
        let mut w: Vec<u16> = title.encode_utf16().collect();
        phai_ui::win::draw_text_w(dc, &mut w, &mut tr, Gdi::DT_LEFT | Gdi::DT_SINGLELINE | Gdi::DT_VCENTER);
        Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(muted.colorref()));
        let mut w: Vec<u16> = now.encode_utf16().collect();
        if !w.is_empty() {
            phai_ui::win::draw_text_w(dc, &mut w, &mut tr, Gdi::DT_RIGHT | Gdi::DT_SINGLELINE | Gdi::DT_VCENTER);
        }
    }
    let box_ = RECT { top: r.top + row + win::px(h, 2), ..r };
    win::fill(dc, &box_, border);
    let inner = RECT { left: box_.left + 1, top: box_.top + 1, right: box_.right - 1, bottom: box_.bottom - 1 };
    win::fill(dc, &inner, win::window_background());
    if values.len() < 2 || inner.right <= inner.left {
        return;
    }
    let (w, hgt) = ((inner.right - inner.left) as f32, (inner.bottom - inner.top - 2) as f32);
    let step = w / (KEEP - 1) as f32;
    let start = KEEP - values.len();
    let pts: Vec<windows::Win32::Foundation::POINT> = values
        .iter()
        .enumerate()
        .map(|(i, v)| windows::Win32::Foundation::POINT { x: inner.left + ((start + i) as f32 * step) as i32, y: inner.bottom - 1 - ((v / top.max(0.0001)).min(1.0) * hgt) as i32 })
        .collect();
    unsafe {
        let pen = Gdi::CreatePen(Gdi::PS_SOLID, win::px(h, 2).max(1), windows::Win32::Foundation::COLORREF(colour.colorref()));
        let old = Gdi::SelectObject(dc, pen.into());
        let _ = Gdi::Polyline(dc, &pts);
        Gdi::SelectObject(dc, old);
        let _ = Gdi::DeleteObject(pen.into());
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_SIZE => {
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            LRESULT(0)
        }
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let screen = Gdi::BeginPaint(h, &mut ps);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            let dc = Gdi::CreateCompatibleDC(Some(screen));
            let bmp = Gdi::CreateCompatibleBitmap(screen, r.right.max(1), r.bottom.max(1));
            let old_bmp = Gdi::SelectObject(dc, bmp.into());
            let font = win::font(h, 0, false);
            let old_font = Gdi::SelectObject(dc, font.into());
            win::fill(dc, &r, win::window_background());
            let m = win::px(h, 10);
            let half = (r.right - 3 * m) / 2;
            let accent = theme::current().var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f));
            let s = crate::studio().and_then(|s| s.sampler.as_ref());
            let (cpu, mem, state) = match s {
                Some(s) => (s.cpu.iter().copied().collect::<Vec<_>>(), s.mem.iter().copied().collect::<Vec<_>>(), if s.ended { phai_ui::text::t("The program has ended.") } else { "" }),
                None => (vec![], vec![], phai_ui::text::t("Run the program (F5 or Ctrl+F5) to see its memory and processor use here.")),
            };
            let top_mem = mem.iter().copied().max().unwrap_or(1).max(1) as f32 * 1.25;
            let mem_f: Vec<f32> = mem.iter().map(|b| *b as f32).collect();
            let area = RECT { left: m, top: m, right: m + half, bottom: r.bottom - m - win::px(h, 20) };
            graph(dc, h, area, (phai_ui::text::t("Process Memory"), &mem.last().map(|b| mb(*b)).unwrap_or_default()), &mem_f, top_mem, accent);
            let area2 = RECT { left: 2 * m + half, right: r.right - m, ..area };
            graph(dc, h, area2, (phai_ui::text::t("CPU (% of all processors)"), &cpu.last().map(|c| format!("{c:.0}%")).unwrap_or_default()), &cpu, 100.0, Color::rgb(0x2f, 0x6f, 0xc4));
            if !state.is_empty() {
                let mut sr = RECT { left: m, top: r.bottom - m - win::px(h, 20), right: r.right - m, bottom: r.bottom - m };
                Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(win::window_text().colorref()));
                let mut w: Vec<u16> = state.encode_utf16().collect();
                phai_ui::win::draw_text_w(dc, &mut w, &mut sr, Gdi::DT_LEFT | Gdi::DT_SINGLELINE | Gdi::DT_VCENTER);
            }
            let _ = Gdi::BitBlt(screen, 0, 0, r.right, r.bottom, Some(dc), 0, 0, Gdi::SRCCOPY);
            Gdi::SelectObject(dc, old_font);
            let _ = Gdi::DeleteObject(font.into());
            Gdi::SelectObject(dc, old_bmp);
            let _ = Gdi::DeleteObject(bmp.into());
            let _ = Gdi::DeleteDC(dc);
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn this_program_can_be_watched() {
        let mut s = Sampler::new(std::process::id()).expect("our own process opens");
        // Some work, so the processors' share is something.
        let mut x = 0u64;
        for i in 0..20_000_000u64 {
            x = x.wrapping_add(i * i);
        }
        assert!(x > 0);
        assert!(s.sample());
        assert!(s.mem[0] > 0, "a program holds some memory of its own");
        assert!(s.cpu[0] >= 0.0 && s.cpu[0] <= 100.0);
    }
}
