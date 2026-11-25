//! A chart on a window, as WinForms' Chart control: columns, bars, lines, areas, a pie, a doughnut
//! or candlesticks, drawn with Windows' GDI+ so lines and circles are smooth. Its data is a table:
//! the first row names the columns - what the categories are, then each series - and each row
//! after it is a category and its values (`Month, Sales, Cost` / `Jan, 120, 80`). Candlesticks
//! take `Day, Open, High, Low, Close`. Points can be added while the program runs ([`add_point`]),
//! the oldest dropped past [`set_max_points`] - prices as they come in.
//!
//! The mouse over it shows the values there: a line down the chart and a box with each series.

use std::sync::OnceLock;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HDC, HFONT};
use windows::Win32::Graphics::GdiPlus as gp;
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::theme::{self, Color};
use crate::win;

/// What the chart draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Type {
    /// Upright bars, each series side by side.
    #[default]
    Column,
    /// Bars lying across.
    Bar,
    Line,
    /// A line with the room under it filled.
    Area,
    /// The first series' values as slices of a circle.
    Pie,
    /// A pie with a hole, its total in the middle.
    Doughnut,
    /// Open, high, low and close for each category - a market's days.
    Candlestick,
}

impl Type {
    /// By its `.phf` name: `column`, `bar`, `line`, `area`, `pie`, `doughnut`, `candlestick`.
    pub fn from_name(s: &str) -> Type {
        match s.trim().to_ascii_lowercase().as_str() {
            "bar" => Type::Bar,
            "line" => Type::Line,
            "area" => Type::Area,
            "pie" => Type::Pie,
            "doughnut" | "donut" => Type::Doughnut,
            "candlestick" | "candle" | "stock" => Type::Candlestick,
            _ => Type::Column,
        }
    }

    fn round(self) -> bool {
        matches!(self, Type::Pie | Type::Doughnut)
    }
}

/// The chart's table: the columns' names, then a category and its values for each row.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Data {
    /// What the categories are (`Month`), then each series' name.
    pub headers: Vec<String>,
    pub labels: Vec<String>,
    /// For each category, its value in each series (NaN where a cell is empty or not a number).
    pub values: Vec<Vec<f64>>,
}

impl Data {
    /// From lines of text, a comma (or a tab) between cells, the first line the headings. Numbers
    /// may have thousands commas inside quotes: `"1,200"`.
    pub fn parse(text: &str) -> Data {
        let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
        let mut d = Data::default();
        let Some(head) = lines.next() else { return d };
        d.headers = cells(head);
        for l in lines {
            let mut c = cells(l);
            if c.is_empty() {
                continue;
            }
            let label = c.remove(0);
            d.labels.push(label);
            d.values.push(c.iter().map(|v| number(v)).collect());
        }
        d
    }

    /// The series' names: the headings after the first, or "Series 1"... where there are none.
    pub fn series(&self) -> Vec<String> {
        let n = self.values.iter().map(Vec::len).max().unwrap_or(0).max(self.headers.len().saturating_sub(1));
        (0..n).map(|i| self.headers.get(i + 1).filter(|s| !s.is_empty()).cloned().unwrap_or_else(|| format!("Series {}", i + 1))).collect()
    }

    fn value(&self, row: usize, series: usize) -> f64 {
        self.values.get(row).and_then(|r| r.get(series)).copied().unwrap_or(f64::NAN)
    }
}

/// A line's cells: split at commas or tabs, outside double quotes, trimmed.
fn cells(line: &str) -> Vec<String> {
    let tab = line.contains('\t');
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    for ch in line.chars() {
        match ch {
            '"' => quoted = !quoted,
            '\t' if tab && !quoted => out.push(std::mem::take(&mut cur).trim().to_string()),
            ',' if !tab && !quoted => out.push(std::mem::take(&mut cur).trim().to_string()),
            _ => cur.push(ch),
        }
    }
    out.push(cur.trim().to_string());
    out
}

fn number(s: &str) -> f64 {
    s.trim().replace([',', '฿', '$', '%', ' '], "").parse::<f64>().unwrap_or(f64::NAN)
}

struct Chart {
    kind: Type,
    title: String,
    data: Data,
    legend: bool,
    max_points: usize,
    font: HFONT,
    bold: HFONT,
    /// The category (or slice) under the mouse.
    hot: Option<usize>,
    tracking: bool,
    /// Where each category's middle was drawn, and each slice's angles, for the mouse.
    xs: Vec<i32>,
    plot: RECT,
    slices: Vec<(f32, f32)>,
    center: (f32, f32, f32),
}

fn state<'a>(h: HWND) -> Option<&'a mut Chart> {
    unsafe { win::state::<Chart>(h) }
}

fn repaint(h: HWND) {
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

/// A chart control, empty until [`set_data`] (or [`set_data_text`]) gives it a table.
pub fn chart(parent: HWND, id: i32, font: HFONT) -> HWND {
    startup();
    let class = win::register("PhaiChart", Some(proc_), None);
    let h = unsafe {
        wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPSIBLINGS, 0, 0, 10, 10, Some(parent), Some(wm::HMENU(id as isize as *mut _)), Some(win::instance()), None).unwrap_or_default()
    };
    let bold = win::font(h, 0, true);
    win::set_state(h, Box::new(Chart { kind: Type::Column, title: String::new(), data: Data::default(), legend: true, max_points: 0, font, bold, hot: None, tracking: false, xs: vec![], plot: RECT::default(), slices: vec![], center: (0.0, 0.0, 0.0) }));
    h
}

/// What it draws - see [`Type::from_name`] for the names.
pub fn set_type(h: HWND, kind: &str) {
    if let Some(c) = state(h) {
        c.kind = Type::from_name(kind);
        c.hot = None;
        repaint(h);
    }
}

/// The words over it; empty for none.
pub fn set_title(h: HWND, title: &str) {
    if let Some(c) = state(h) {
        c.title = title.to_string();
        repaint(h);
    }
}

/// Whether each series' colour and name shows over it (the slices' beside a pie).
pub fn set_legend(h: HWND, on: bool) {
    if let Some(c) = state(h) {
        c.legend = on;
        repaint(h);
    }
}

/// Its table, as rows of cells - the first row the headings.
pub fn set_data(h: HWND, rows: &[Vec<String>]) {
    let text: Vec<String> = rows.iter().map(|r| r.iter().map(|c| if c.contains(',') { format!("\"{c}\"") } else { c.clone() }).collect::<Vec<_>>().join(",")).collect();
    set_data_text(h, &text.join("\n"));
}

/// Its table as text: a line a row, commas (or tabs) between the cells, the first line the
/// headings.
pub fn set_data_text(h: HWND, text: &str) {
    if let Some(c) = state(h) {
        c.data = Data::parse(text);
        c.hot = None;
        repaint(h);
    }
}

/// The table it shows now.
pub fn data(h: HWND) -> Data {
    state(h).map(|c| c.data.clone()).unwrap_or_default()
}

/// One more category at the end - a day, a second - with its values; the first dropped when there
/// are more than [`set_max_points`] allows.
pub fn add_point(h: HWND, label: &str, values: &[f64]) {
    if let Some(c) = state(h) {
        c.data.labels.push(label.to_string());
        c.data.values.push(values.to_vec());
        if c.max_points > 0 {
            while c.data.labels.len() > c.max_points {
                c.data.labels.remove(0);
                c.data.values.remove(0);
            }
        }
        repaint(h);
    }
}

/// How many categories it keeps as points are added; 0 keeps them all.
pub fn set_max_points(h: HWND, n: usize) {
    if let Some(c) = state(h) {
        c.max_points = n;
    }
}

/// Category `k` shown as the mouse over it would - its values in a box; `None` for none. A
/// program can point at the row chosen in a grid, say.
pub fn highlight(h: HWND, k: Option<usize>) {
    if let Some(c) = state(h) {
        c.hot = k.filter(|k| *k < c.data.labels.len());
        repaint(h);
    }
}

/// Every category taken out; the headings stay.
pub fn clear(h: HWND) {
    if let Some(c) = state(h) {
        c.data.labels.clear();
        c.data.values.clear();
        c.hot = None;
        repaint(h);
    }
}

// --- GDI+ ----------------------------------------------------------------------------------------

/// GDI+ started once for the program.
fn startup() {
    static TOKEN: OnceLock<usize> = OnceLock::new();
    TOKEN.get_or_init(|| {
        let mut token = 0usize;
        let input = gp::GdiplusStartupInput { GdiplusVersion: 1, ..Default::default() };
        unsafe {
            let _ = gp::GdiplusStartup(&mut token, &input, std::ptr::null_mut());
        }
        token
    });
}

fn argb(c: Color, alpha: u8) -> u32 {
    ((alpha as u32) << 24) | ((c.r as u32) << 16) | ((c.g as u32) << 8) | c.b as u32
}

/// A GDI+ surface over a DC, smooth, freed when dropped.
struct Canvas(*mut gp::GpGraphics);

impl Canvas {
    fn on(dc: HDC) -> Canvas {
        let mut g = std::ptr::null_mut();
        unsafe {
            gp::GdipCreateFromHDC(dc, &mut g);
            gp::GdipSetSmoothingMode(g, gp::SmoothingModeAntiAlias);
        }
        Canvas(g)
    }

    fn fill_rect(&self, x: f32, y: f32, w: f32, h: f32, c: u32) {
        unsafe {
            let mut b = std::ptr::null_mut();
            gp::GdipCreateSolidFill(c, &mut b);
            gp::GdipFillRectangle(self.0, b as *mut gp::GpBrush, x, y, w, h);
            gp::GdipDeleteBrush(b as *mut gp::GpBrush);
        }
    }

    /// A rectangle with its corners rounded by `r` - only the top ones when `top_only`.
    fn round_rect(&self, x: f32, y: f32, w: f32, h: f32, r: f32, c: u32, top_only: bool) {
        let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
        if r < 0.5 {
            return self.fill_rect(x, y, w, h, c);
        }
        unsafe {
            let mut p = std::ptr::null_mut();
            gp::GdipCreatePath(gp::FillModeAlternate, &mut p);
            let d = r * 2.0;
            gp::GdipAddPathArc(p, x, y, d, d, 180.0, 90.0);
            gp::GdipAddPathArc(p, x + w - d, y, d, d, 270.0, 90.0);
            if top_only {
                gp::GdipAddPathLine(p, x + w, y + r, x + w, y + h);
                gp::GdipAddPathLine(p, x + w, y + h, x, y + h);
            } else {
                gp::GdipAddPathArc(p, x + w - d, y + h - d, d, d, 0.0, 90.0);
                gp::GdipAddPathArc(p, x, y + h - d, d, d, 90.0, 90.0);
            }
            gp::GdipClosePathFigure(p);
            let mut b = std::ptr::null_mut();
            gp::GdipCreateSolidFill(c, &mut b);
            gp::GdipFillPath(self.0, b as *mut gp::GpBrush, p);
            gp::GdipDeleteBrush(b as *mut gp::GpBrush);
            gp::GdipDeletePath(p);
        }
    }

    fn line(&self, pts: &[gp::PointF], c: u32, width: f32) {
        if pts.len() < 2 {
            return;
        }
        unsafe {
            let mut pen = std::ptr::null_mut();
            gp::GdipCreatePen1(c, width, gp::UnitPixel, &mut pen);
            gp::GdipSetPenLineJoin(pen, gp::LineJoinRound);
            gp::GdipDrawLines(self.0, pen, pts.as_ptr(), pts.len() as i32);
            gp::GdipDeletePen(pen);
        }
    }

    /// The room between `pts` and the line at `base` filled, darker near the line than at the base.
    fn area(&self, pts: &[gp::PointF], base: f32, top: f32, c: Color, strong: u8) {
        if pts.len() < 2 {
            return;
        }
        let mut poly = pts.to_vec();
        poly.push(gp::PointF { X: pts[pts.len() - 1].X, Y: base });
        poly.push(gp::PointF { X: pts[0].X, Y: base });
        unsafe {
            let rect = gp::RectF { X: pts[0].X, Y: top - 1.0, Width: (pts[pts.len() - 1].X - pts[0].X).max(1.0), Height: (base - top).max(1.0) + 2.0 };
            let mut b = std::ptr::null_mut();
            gp::GdipCreateLineBrushFromRect(&rect, argb(c, strong), argb(c, 8), gp::LinearGradientModeVertical, gp::WrapModeTile, &mut b);
            gp::GdipFillPolygon(self.0, b as *mut gp::GpBrush, poly.as_ptr(), poly.len() as i32, gp::FillModeAlternate);
            gp::GdipDeleteBrush(b as *mut gp::GpBrush);
        }
    }

    fn dot(&self, x: f32, y: f32, r: f32, c: u32) {
        unsafe {
            let mut b = std::ptr::null_mut();
            gp::GdipCreateSolidFill(c, &mut b);
            gp::GdipFillEllipse(self.0, b as *mut gp::GpBrush, x - r, y - r, r * 2.0, r * 2.0);
            gp::GdipDeleteBrush(b as *mut gp::GpBrush);
        }
    }

    fn pie(&self, x: f32, y: f32, d: f32, start: f32, sweep: f32, c: u32, edge: u32, edge_w: f32) {
        unsafe {
            let mut b = std::ptr::null_mut();
            gp::GdipCreateSolidFill(c, &mut b);
            gp::GdipFillPie(self.0, b as *mut gp::GpBrush, x, y, d, d, start, sweep);
            gp::GdipDeleteBrush(b as *mut gp::GpBrush);
            if edge_w > 0.0 {
                let mut pen = std::ptr::null_mut();
                gp::GdipCreatePen1(edge, edge_w, gp::UnitPixel, &mut pen);
                gp::GdipSetPenLineJoin(pen, gp::LineJoinRound);
                gp::GdipDrawPie(self.0, pen, x, y, d, d, start, sweep);
                gp::GdipDeletePen(pen);
            }
        }
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        unsafe {
            gp::GdipDeleteGraphics(self.0);
        }
    }
}

// --- drawing -------------------------------------------------------------------------------------

/// The series' colours: the theme's accent first, then colours apart from it and each other.
pub fn palette() -> Vec<Color> {
    let accent = theme::current().var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f));
    let mut out = vec![accent];
    for c in [(0x3b, 0x82, 0xf6), (0xf5, 0x9e, 0x0b), (0xef, 0x44, 0x44), (0x8b, 0x5c, 0xf6), (0x06, 0xb6, 0xd4), (0xec, 0x48, 0x99), (0x84, 0xcc, 0x16), (0x64, 0x74, 0x8b)] {
        out.push(Color::rgb(c.0, c.1, c.2));
    }
    out
}

/// A round step for an axis of `range` cut in about `parts`: 1, 2, 2.5 or 5 times a power of ten.
pub fn nice_step(range: f64, parts: f64) -> f64 {
    if !(range > 0.0) || !range.is_finite() {
        return 1.0;
    }
    let raw = range / parts.max(1.0);
    let p = 10f64.powf(raw.log10().floor());
    let f = raw / p;
    let m = if f <= 1.0 {
        1.0
    } else if f <= 2.0 {
        2.0
    } else if f <= 2.5 {
        2.5
    } else if f <= 5.0 {
        5.0
    } else {
        10.0
    };
    m * p
}

/// A number as an axis or a tip shows it: thousands grouped, the decimals the step needs.
pub fn show_number(v: f64, step: f64) -> String {
    // As many decimals as the step has: 2.5 one, 0.25 two, 250 none.
    let decimals = if step == 0.0 { if v.fract().abs() > 1e-9 { 2 } else { 0 } } else { (0..6).find(|d| { let m = step * 10f64.powi(*d as i32); (m - m.round()).abs() < 1e-6 * m.abs().max(1.0) }).unwrap_or(6) };
    let s = format!("{:.*}", decimals, v.abs());
    let (int, frac) = match s.split_once('.') {
        Some((i, f)) => (i.to_string(), Some(f.to_string())),
        None => (s.clone(), None),
    };
    let mut out = String::new();
    for (i, ch) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    if let Some(f) = frac {
        out = format!("{out}.{f}");
    }
    if v < 0.0 && out.chars().any(|c| c.is_ascii_digit() && c != '0') {
        out.insert(0, '-');
    }
    out
}

fn text(dc: HDC, font: HFONT, s: &str, r: &mut RECT, c: Color, flags: Gdi::DRAW_TEXT_FORMAT) {
    if s.is_empty() {
        return;
    }
    unsafe {
        let old = Gdi::SelectObject(dc, font.into());
        Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        Gdi::SetTextColor(dc, COLORREF(c.colorref()));
        let mut w: Vec<u16> = s.encode_utf16().collect();
        win::draw_text_w(dc, &mut w, r, flags | Gdi::DT_NOPREFIX);
        Gdi::SelectObject(dc, old);
    }
}

fn text_height(dc: HDC, font: HFONT) -> i32 {
    let mut r = RECT::default();
    text(dc, font, "Ag", &mut r, Color::rgb(0, 0, 0), Gdi::DT_CALCRECT | Gdi::DT_SINGLELINE);
    r.bottom - r.top
}

fn text_width(dc: HDC, font: HFONT, s: &str) -> i32 {
    let mut r = RECT::default();
    text(dc, font, s, &mut r, Color::rgb(0, 0, 0), Gdi::DT_CALCRECT | Gdi::DT_SINGLELINE);
    r.right - r.left
}

/// The colours it is drawn in: what is behind it, the words, quiet words, lines.
#[derive(Clone, Copy)]
struct Look {
    back: Color,
    text: Color,
    muted: Color,
    grid: Color,
    axis: Color,
}

fn look(h: HWND) -> Look {
    let cur = theme::current();
    let back = win::window_background_for(h);
    let text = win::window_text_for(h);
    let muted = cur.var_color("--muted").unwrap_or(text.mix(back, 0.45));
    let line = cur.var_color("--border").unwrap_or(text.mix(back, 0.8));
    Look { back, text, muted, grid: line.mix(back, 0.45), axis: line }
}

fn paint(h: HWND, dc: HDC, r: RECT) {
    let Some(c) = state(h) else { return };
    let lk = look(h);
    win::fill(dc, &r, lk.back);
    let px = |v: i32| win::px(h, v);
    let pad = px(12);
    let mut top = r.top + pad;
    // The title, then the legend under it (beside a pie, on its right).
    if !c.title.is_empty() {
        let mut tr = RECT { left: r.left + pad, top, right: r.right - pad, bottom: top + px(24) };
        text(dc, c.bold, &c.title, &mut tr, lk.text, Gdi::DT_SINGLELINE | Gdi::DT_END_ELLIPSIS | Gdi::DT_VCENTER);
        top += px(28);
    }
    let pal = palette();
    let series = c.data.series();
    let n = c.data.labels.len();
    c.xs.clear();
    c.slices.clear();
    if n == 0 || series.is_empty() {
        let mut tr = RECT { left: r.left, top, right: r.right, bottom: r.bottom };
        text(dc, c.font, crate::text::t("No data yet"), &mut tr, lk.muted, Gdi::DT_CENTER | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE);
        return;
    }
    let colour = |i: usize| pal[i % pal.len()];
    if c.kind.round() {
        paint_round(h, dc, c, r, top, lk, &colour);
        return;
    }
    if c.legend && (series.len() > 1 || c.kind == Type::Candlestick) {
        let names: Vec<(String, Color)> = if c.kind == Type::Candlestick {
            vec![(crate::text::t("Up").to_string(), up_colour()), (crate::text::t("Down").to_string(), down_colour())]
        } else {
            series.iter().enumerate().map(|(i, s)| (s.clone(), colour(i))).collect()
        };
        let mut x = r.left + pad;
        let sw = px(10);
        for (name, col) in names {
            let w = text_width(dc, c.font, &name);
            if x + sw + px(6) + w > r.right - pad {
                break;
            }
            let cy = top + px(9);
            let canvas = Canvas::on(dc);
            canvas.round_rect(x as f32, (cy - sw / 2) as f32, sw as f32, sw as f32, px(3) as f32, argb(col, 255), false);
            drop(canvas);
            let mut tr = RECT { left: x + sw + px(6), top, right: x + sw + px(6) + w + 2, bottom: top + px(18) };
            text(dc, c.font, &name, &mut tr, lk.muted, Gdi::DT_SINGLELINE | Gdi::DT_VCENTER);
            x += sw + px(6) + w + px(18);
        }
        top += px(26);
    }
    // The values' range, and the axis' round steps.
    let ser: Vec<usize> = if c.kind == Type::Candlestick { vec![0, 1, 2, 3] } else { (0..series.len()).collect() };
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for row in 0..n {
        for &s in &ser {
            let v = c.data.value(row, s);
            if v.is_finite() {
                lo = lo.min(v);
                hi = hi.max(v);
            }
        }
    }
    if !lo.is_finite() {
        lo = 0.0;
        hi = 1.0;
    }
    // Bars and areas stand on zero; lines and candles fit their values.
    if matches!(c.kind, Type::Column | Type::Bar | Type::Area) {
        lo = lo.min(0.0);
        hi = hi.max(0.0);
    }
    if hi - lo < 1e-12 {
        hi += 1.0;
        lo -= if lo == 0.0 { 0.0 } else { 1.0 };
    }
    // As many steps as there is room for: about one a 30 pixels up the chart (across, for bars).
    let label_h = px(20);
    let room = if c.kind == Type::Bar { (r.right - r.left - 2 * pad) * 2 / 3 / px(90) } else { (r.bottom - pad - label_h - top - px(6)) / px(30) };
    let step = nice_step(hi - lo, room.clamp(2, 6) as f64);
    let lo = (lo / step).floor() * step;
    let hi = (hi / step).ceil() * step;
    let ticks: Vec<f64> = (0..=(((hi - lo) / step).round() as usize).min(50)).map(|i| lo + i as f64 * step).collect();
    let tick_w = ticks.iter().map(|t| text_width(dc, c.font, &show_number(*t, step))).max().unwrap_or(0);
    if c.kind == Type::Bar {
        paint_bars(h, dc, c, r, top, lk, &colour, &series, lo, hi, step, &ticks);
        return;
    }
    let plot = RECT { left: r.left + pad + tick_w + px(8), top: top + px(6), right: r.right - pad, bottom: r.bottom - pad - label_h };
    if plot.right - plot.left < px(20) || plot.bottom - plot.top < px(20) {
        return;
    }
    c.plot = plot;
    let y_of = |v: f64| plot.bottom as f32 - ((v - lo) / (hi - lo)) as f32 * (plot.bottom - plot.top) as f32;
    // The lines across at each step, and their numbers.
    for t in &ticks {
        let y = y_of(*t).round() as i32;
        win::fill(dc, &RECT { left: plot.left, top: y, right: plot.right, bottom: y + 1 }, if t.abs() < step * 1e-9 { lk.axis } else { lk.grid });
        let mut tr = RECT { left: r.left + pad, top: y - px(9), right: plot.left - px(8), bottom: y + px(9) };
        text(dc, c.font, &show_number(*t, step), &mut tr, lk.muted, Gdi::DT_RIGHT | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE);
    }
    win::fill(dc, &RECT { left: plot.left, top: plot.bottom, right: plot.right, bottom: plot.bottom + 1 }, lk.axis);
    let slot = (plot.right - plot.left) as f32 / n as f32;
    let x_mid = |i: usize| plot.left as f32 + slot * (i as f32 + 0.5);
    c.xs = (0..n).map(|i| x_mid(i).round() as i32).collect();
    // The categories' names under it - as many as fit.
    let widest = c.data.labels.iter().map(|l| text_width(dc, c.font, l)).max().unwrap_or(0) + px(10);
    let every = ((widest as f32 / slot).ceil() as usize).max(1);
    for i in (0..n).step_by(every) {
        let x = x_mid(i).round() as i32;
        let mut tr = RECT { left: x - widest / 2 - px(20), top: plot.bottom + px(4), right: x + widest / 2 + px(20), bottom: plot.bottom + label_h };
        text(dc, c.font, &c.data.labels[i], &mut tr, lk.muted, Gdi::DT_CENTER | Gdi::DT_SINGLELINE);
    }
    // The one under the mouse, behind the data.
    if let Some(k) = c.hot.filter(|k| *k < n) {
        let x = x_mid(k);
        let canvas = Canvas::on(dc);
        if matches!(c.kind, Type::Column | Type::Candlestick) {
            canvas.fill_rect(x - slot / 2.0, plot.top as f32, slot, (plot.bottom - plot.top) as f32, argb(lk.text, 14));
        } else {
            canvas.fill_rect(x - 0.5, plot.top as f32, 1.0, (plot.bottom - plot.top) as f32, argb(lk.muted, 160));
        }
    }
    let canvas = Canvas::on(dc);
    match c.kind {
        Type::Column => {
            let group = slot * if n > 60 { 0.9 } else { 0.72 };
            let s_n = series.len() as f32;
            let gap = if s_n > 1.0 { (group * 0.06).min(px(3) as f32) } else { 0.0 };
            let w = ((group - gap * (s_n - 1.0)) / s_n).max(1.0);
            let zero = y_of(0.0f64.max(lo).min(hi));
            for i in 0..n {
                let x0 = x_mid(i) - group / 2.0;
                for s in 0..series.len() {
                    let v = c.data.value(i, s);
                    if !v.is_finite() {
                        continue;
                    }
                    let y = y_of(v);
                    let (top_y, height) = if y < zero { (y, zero - y) } else { (zero, y - zero) };
                    let alpha = if c.hot.is_some_and(|k| k != i) { 150 } else { 255 };
                    let x = x0 + s as f32 * (w + gap);
                    if v >= 0.0 {
                        canvas.round_rect(x, top_y, w, height.max(1.0), (w * 0.22).min(px(5) as f32), argb(colour(s), alpha), true);
                    } else {
                        canvas.fill_rect(x, top_y, w, height.max(1.0), argb(colour(s), alpha));
                    }
                }
            }
        }
        Type::Line | Type::Area => {
            for s in 0..series.len() {
                let pts: Vec<gp::PointF> = (0..n).filter_map(|i| {
                    let v = c.data.value(i, s);
                    v.is_finite().then(|| gp::PointF { X: x_mid(i), Y: y_of(v) })
                }).collect();
                if c.kind == Type::Area {
                    canvas.area(&pts, y_of(0.0f64.max(lo)), plot.top as f32, colour(s), if series.len() > 1 { 70 } else { 110 });
                }
                canvas.line(&pts, argb(colour(s), 255), win::px(h, 2).max(2) as f32 + 0.5);
                if slot >= px(16) as f32 {
                    for p in &pts {
                        canvas.dot(p.X, p.Y, px(4) as f32, argb(lk.back, 255));
                        canvas.dot(p.X, p.Y, px(3) as f32 - 0.5, argb(colour(s), 255));
                    }
                }
            }
            if let Some(k) = c.hot.filter(|k| *k < n) {
                for s in 0..series.len() {
                    let v = c.data.value(k, s);
                    if v.is_finite() {
                        canvas.dot(x_mid(k), y_of(v), px(6) as f32, argb(colour(s), 60));
                        canvas.dot(x_mid(k), y_of(v), px(4) as f32, argb(colour(s), 255));
                    }
                }
            }
        }
        Type::Candlestick => {
            let w = (slot * 0.62).max(1.0);
            for i in 0..n {
                let (o, hgh, low, cl) = (c.data.value(i, 0), c.data.value(i, 1), c.data.value(i, 2), c.data.value(i, 3));
                if ![o, hgh, low, cl].iter().all(|v| v.is_finite()) {
                    continue;
                }
                let col = if cl >= o { up_colour() } else { down_colour() };
                let x = x_mid(i);
                canvas.fill_rect(x - 0.6, y_of(hgh), 1.2_f32.max(px(1) as f32), (y_of(low) - y_of(hgh)).max(1.0), argb(col, 255));
                let (a, b) = (y_of(o.max(cl)), y_of(o.min(cl)));
                canvas.round_rect(x - w / 2.0, a, w, (b - a).max(1.5), (w * 0.15).min(px(2) as f32), argb(col, 255), false);
            }
        }
        _ => {}
    }
    drop(canvas);
    if let Some(k) = c.hot.filter(|k| *k < n) {
        let rows: Vec<(String, String, Color)> = if c.kind == Type::Candlestick {
            let names = ["Open", "High", "Low", "Close"];
            let col = if c.data.value(k, 3) >= c.data.value(k, 0) { up_colour() } else { down_colour() };
            (0..4).map(|s| (c.data.headers.get(s + 1).cloned().unwrap_or_else(|| crate::text::t(names[s]).to_string()), show_number(c.data.value(k, s), step.min(1.0) / 10.0), col)).collect()
        } else {
            series.iter().enumerate().map(|(s, name)| (name.clone(), number_or_dash(c.data.value(k, s), step), colour(s))).collect()
        };
        tip(h, dc, c, lk, x_mid(k) as i32, plot, &c.data.labels[k], &rows);
    }
}

fn number_or_dash(v: f64, step: f64) -> String {
    if v.is_finite() {
        show_number(v, if v.fract().abs() > 1e-9 { step.min(0.01) } else { step.max(1.0) })
    } else {
        "-".into()
    }
}

fn up_colour() -> Color {
    Color::rgb(0x16, 0xa3, 0x4a)
}

fn down_colour() -> Color {
    Color::rgb(0xdc, 0x26, 0x26)
}

/// The box with the values under the mouse, beside the line at `x`, kept inside the chart.
fn tip(h: HWND, dc: HDC, c: &Chart, lk: Look, x: i32, plot: RECT, heading: &str, rows: &[(String, String, Color)]) {
    let px = |v: i32| win::px(h, v);
    let line_h = px(20);
    let names_w = rows.iter().map(|(n, _, _)| text_width(dc, c.font, n)).max().unwrap_or(0);
    let values_w = rows.iter().map(|(_, v, _)| text_width(dc, c.bold, v)).max().unwrap_or(0);
    let w = (px(10) + px(14) + names_w + px(16) + values_w + px(10)).max(text_width(dc, c.bold, heading) + px(20));
    let ht = px(8) + line_h * (rows.len() as i32 + 1) + px(6);
    let mut left = x + px(12);
    if left + w > plot.right {
        left = x - px(12) - w;
    }
    let left = left.max(plot.left);
    // Inside the chart: from the top of the drawing, moved up when it would run off the bottom.
    let mut all = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut all);
    }
    let top = (plot.top + px(4)).min(all.bottom - ht - px(2)).max(all.top + px(2));
    let canvas = Canvas::on(dc);
    canvas.round_rect(left as f32 + 1.0, top as f32 + 2.0, w as f32, ht as f32, px(6) as f32, argb(Color::rgb(0, 0, 0), 28), false);
    let card = if theme::current().is_dark() { lk.back.mix(Color::rgb(255, 255, 255), 0.08) } else { Color::rgb(255, 255, 255) };
    canvas.round_rect(left as f32, top as f32, w as f32, ht as f32, px(6) as f32, argb(lk.axis, 255), false);
    canvas.round_rect(left as f32 + 1.0, top as f32 + 1.0, w as f32 - 2.0, ht as f32 - 2.0, px(6) as f32 - 1.0, argb(card, 255), false);
    drop(canvas);
    let mut tr = RECT { left: left + px(10), top: top + px(5), right: left + w - px(8), bottom: top + px(5) + line_h };
    text(dc, c.bold, heading, &mut tr, lk.text, Gdi::DT_SINGLELINE | Gdi::DT_VCENTER | Gdi::DT_END_ELLIPSIS);
    for (i, (name, value, col)) in rows.iter().enumerate() {
        let y = top + px(5) + line_h * (i as i32 + 1);
        let canvas = Canvas::on(dc);
        canvas.dot((left + px(14)) as f32, (y + line_h / 2) as f32, px(4) as f32, argb(*col, 255));
        drop(canvas);
        let mut nr = RECT { left: left + px(24), top: y, right: left + w - px(10), bottom: y + line_h };
        text(dc, c.font, name, &mut nr, lk.muted, Gdi::DT_SINGLELINE | Gdi::DT_VCENTER);
        let mut vr = nr;
        text(dc, c.bold, value, &mut vr, lk.text, Gdi::DT_SINGLELINE | Gdi::DT_VCENTER | Gdi::DT_RIGHT);
    }
}

/// Bars lying across: the categories down the left, the values along the bottom.
#[allow(clippy::too_many_arguments)]
fn paint_bars(h: HWND, dc: HDC, c: &mut Chart, r: RECT, top: i32, lk: Look, colour: &dyn Fn(usize) -> Color, series: &[String], lo: f64, hi: f64, step: f64, ticks: &[f64]) {
    let px = |v: i32| win::px(h, v);
    let pad = px(12);
    let n = c.data.labels.len();
    let names_w = c.data.labels.iter().map(|l| text_width(dc, c.font, l)).max().unwrap_or(0).min((r.right - r.left) / 3);
    let plot = RECT { left: r.left + pad + names_w + px(10), top: top + px(4), right: r.right - pad - px(10), bottom: r.bottom - pad - px(20) };
    if plot.right - plot.left < px(20) || plot.bottom - plot.top < px(10) {
        return;
    }
    c.plot = plot;
    let x_of = |v: f64| plot.left as f32 + ((v - lo) / (hi - lo)) as f32 * (plot.right - plot.left) as f32;
    for t in ticks {
        let x = x_of(*t).round() as i32;
        win::fill(dc, &RECT { left: x, top: plot.top, right: x + 1, bottom: plot.bottom }, if t.abs() < step * 1e-9 { lk.axis } else { lk.grid });
        let mut tr = RECT { left: x - px(40), top: plot.bottom + px(4), right: x + px(40), bottom: plot.bottom + px(20) };
        text(dc, c.font, &show_number(*t, step), &mut tr, lk.muted, Gdi::DT_CENTER | Gdi::DT_SINGLELINE);
    }
    let slot = (plot.bottom - plot.top) as f32 / n as f32;
    let y_mid = |i: usize| plot.top as f32 + slot * (i as f32 + 0.5);
    c.xs = (0..n).map(|i| y_mid(i).round() as i32).collect();
    let every = ((text_height(dc, c.font) as f32 / slot).ceil() as usize).max(1);
    for i in (0..n).step_by(every) {
        let y = y_mid(i).round() as i32;
        let mut tr = RECT { left: r.left + pad, top: y - px(10), right: plot.left - px(10), bottom: y + px(10) };
        text(dc, c.font, &c.data.labels[i], &mut tr, lk.muted, Gdi::DT_RIGHT | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_END_ELLIPSIS);
    }
    let canvas = Canvas::on(dc);
    if let Some(k) = c.hot.filter(|k| *k < n) {
        canvas.fill_rect(plot.left as f32, y_mid(k) - slot / 2.0, (plot.right - plot.left) as f32, slot, argb(lk.text, 14));
    }
    let group = slot * 0.72;
    let s_n = series.len() as f32;
    let hgt = (group / s_n).max(1.0);
    let zero = x_of(0.0f64.max(lo).min(hi));
    for i in 0..n {
        let y0 = y_mid(i) - group / 2.0;
        for s in 0..series.len() {
            let v = c.data.value(i, s);
            if !v.is_finite() {
                continue;
            }
            let x = x_of(v);
            let (left, w) = if x < zero { (x, zero - x) } else { (zero, x - zero) };
            canvas.round_rect(left, y0 + s as f32 * hgt, w.max(1.0), (hgt - 1.0).max(1.0), (hgt * 0.25).min(px(4) as f32), argb(colour(s), 255), false);
        }
    }
    drop(canvas);
    if let Some(k) = c.hot.filter(|k| *k < n) {
        let rows: Vec<(String, String, Color)> = series.iter().enumerate().map(|(s, name)| (name.clone(), number_or_dash(c.data.value(k, s), step), colour(s))).collect();
        let at = (plot.left + plot.right) / 2;
        tip(h, dc, c, lk, at, plot, &c.data.labels[k], &rows);
    }
}

/// A pie or a doughnut of the first series, its slices named on the right.
fn paint_round(h: HWND, dc: HDC, c: &mut Chart, r: RECT, top: i32, lk: Look, colour: &dyn Fn(usize) -> Color) {
    let px = |v: i32| win::px(h, v);
    let pad = px(12);
    let n = c.data.labels.len();
    let values: Vec<f64> = (0..n).map(|i| c.data.value(i, 0).max(0.0)).map(|v| if v.is_finite() { v } else { 0.0 }).collect();
    let total: f64 = values.iter().sum();
    let legend_w = if c.legend { (c.data.labels.iter().map(|l| text_width(dc, c.font, l)).max().unwrap_or(0) + px(90)).min((r.right - r.left) / 2) } else { 0 };
    let avail_w = r.right - r.left - 2 * pad - legend_w;
    let avail_h = r.bottom - top - pad;
    let d = avail_w.min(avail_h).max(px(20)) as f32;
    let cx = (r.left + pad) as f32 + avail_w as f32 / 2.0;
    let cy = top as f32 + avail_h as f32 / 2.0;
    c.center = (cx, cy, d / 2.0);
    let canvas = Canvas::on(dc);
    let mut start = -90.0f32;
    for (i, v) in values.iter().enumerate() {
        let sweep = if total > 0.0 { (*v / total * 360.0) as f32 } else { 0.0 };
        c.slices.push((start, sweep));
        if sweep > 0.0 {
            let grow = if c.hot == Some(i) { px(5) as f32 } else { 0.0 };
            let (dx, dy) = {
                let mid = (start + sweep / 2.0).to_radians();
                (mid.cos() * grow, mid.sin() * grow)
            };
            canvas.pie(cx - d / 2.0 + dx, cy - d / 2.0 + dy, d, start, sweep, argb(colour(i), 255), argb(lk.back, 255), if n > 1 { px(2) as f32 } else { 0.0 });
        }
        start += sweep;
    }
    if c.kind == Type::Doughnut {
        let hole = d * 0.58;
        canvas.dot(cx, cy, hole / 2.0, argb(lk.back, 255));
    }
    drop(canvas);
    if c.kind == Type::Doughnut {
        let (heading, value) = match c.hot.filter(|k| *k < n) {
            Some(k) => (c.data.labels[k].clone(), show_number(values[k], 1.0)),
            None => (crate::text::t("Total").to_string(), show_number(total, 1.0)),
        };
        let mut tr = RECT { left: (cx - d * 0.28) as i32, top: (cy - px(22) as f32) as i32, right: (cx + d * 0.28) as i32, bottom: cy as i32 };
        text(dc, c.font, &heading, &mut tr, lk.muted, Gdi::DT_CENTER | Gdi::DT_BOTTOM | Gdi::DT_SINGLELINE | Gdi::DT_END_ELLIPSIS);
        let mut vr = RECT { left: (cx - d * 0.3) as i32, top: cy as i32, right: (cx + d * 0.3) as i32, bottom: (cy + px(26) as f32) as i32 };
        text(dc, c.bold, &value, &mut vr, lk.text, Gdi::DT_CENTER | Gdi::DT_TOP | Gdi::DT_SINGLELINE);
    }
    if c.legend {
        let x = r.right - pad - legend_w + px(10);
        let row_h = px(22).min((r.bottom - pad - top) / n.max(1) as i32).max(text_height(dc, c.font));
        let y0 = (cy as i32 - row_h * n as i32 / 2).max(top);
        for (i, label) in c.data.labels.iter().enumerate() {
            let y = y0 + row_h * i as i32;
            if y + row_h > r.bottom - pad {
                break;
            }
            let canvas = Canvas::on(dc);
            canvas.round_rect(x as f32, (y + row_h / 2 - px(5)) as f32, px(10) as f32, px(10) as f32, px(3) as f32, argb(colour(i), 255), false);
            drop(canvas);
            let pct = if total > 0.0 { values[i] / total * 100.0 } else { 0.0 };
            let mut nr = RECT { left: x + px(16), top: y, right: r.right - pad, bottom: y + row_h };
            text(dc, if c.hot == Some(i) { c.bold } else { c.font }, label, &mut nr, lk.text, Gdi::DT_SINGLELINE | Gdi::DT_VCENTER | Gdi::DT_END_ELLIPSIS);
            let mut vr = nr;
            text(dc, c.font, &format!("{pct:.0}%"), &mut vr, lk.muted, Gdi::DT_SINGLELINE | Gdi::DT_VCENTER | Gdi::DT_RIGHT);
        }
    }
}

/// The category, or the slice, at a point in the window.
fn hit(c: &Chart, x: i32, y: i32) -> Option<usize> {
    if c.kind.round() {
        let (cx, cy, rad) = c.center;
        let (dx, dy) = (x as f32 - cx, y as f32 - cy);
        let dist = (dx * dx + dy * dy).sqrt();
        if dist > rad || (c.kind == Type::Doughnut && dist < rad * 0.58) {
            return None;
        }
        let mut a = dy.atan2(dx).to_degrees();
        while a < -90.0 {
            a += 360.0;
        }
        return c.slices.iter().position(|(s, w)| a >= *s && a < s + w);
    }
    if c.xs.is_empty() {
        return None;
    }
    let p = c.plot;
    if x < p.left || x > p.right || y < p.top || y > p.bottom {
        return None;
    }
    let along = if c.kind == Type::Bar { y } else { x };
    c.xs.iter().enumerate().min_by_key(|(_, m)| (**m - along).abs()).map(|(i, _)| i)
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_SIZE => {
            repaint(h);
            LRESULT(0)
        }
        wm::WM_PAINT => unsafe {
            let mut ps = Gdi::PAINTSTRUCT::default();
            let screen = Gdi::BeginPaint(h, &mut ps);
            let buf = win::off_screen(h, screen);
            let mut r = RECT::default();
            let _ = wm::GetClientRect(h, &mut r);
            paint(h, buf.dc, r);
            drop(buf);
            let _ = Gdi::EndPaint(h, &ps);
            LRESULT(0)
        },
        wm::WM_MOUSEMOVE => {
            let (x, y) = win::point_of(l);
            if let Some(c) = state(h) {
                if !c.tracking {
                    let mut tme = km::TRACKMOUSEEVENT { cbSize: std::mem::size_of::<km::TRACKMOUSEEVENT>() as u32, dwFlags: km::TME_LEAVE, hwndTrack: h, dwHoverTime: 0 };
                    unsafe {
                        let _ = km::TrackMouseEvent(&mut tme);
                    }
                    c.tracking = true;
                }
                let k = hit(c, x, y);
                if k != c.hot {
                    c.hot = k;
                    repaint(h);
                }
            }
            LRESULT(0)
        }
        0x02A3 /* WM_MOUSELEAVE */ => {
            if let Some(c) = state(h) {
                c.tracking = false;
                if c.hot.take().is_some() {
                    repaint(h);
                }
            }
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            if let Some(c) = unsafe { win::take_state::<Chart>(h) } {
                let _ = unsafe { Gdi::DeleteObject(c.bold.into()) };
            }
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_table_is_read_from_lines() {
        let d = Data::parse("Month, Sales, Cost\nJan, 120, 80\nFeb, \"1,200\", x\n\nMar, 90");
        assert_eq!(d.headers, ["Month", "Sales", "Cost"]);
        assert_eq!(d.labels, ["Jan", "Feb", "Mar"]);
        assert_eq!(d.values[1][0], 1200.0);
        assert!(d.values[1][1].is_nan(), "a cell that is not a number is a gap");
        assert_eq!(d.series(), ["Sales", "Cost"]);
        let t = Data::parse("วัน\tเปิด\tสูง\tต่ำ\tปิด\nจ.\t10\t12\t9\t11");
        assert_eq!(t.values[0], [10.0, 12.0, 9.0, 11.0]);
    }

    #[test]
    fn axis_steps_are_round() {
        assert_eq!(nice_step(100.0, 5.0), 20.0);
        assert_eq!(nice_step(1200.0, 5.0), 250.0);
        assert_eq!(nice_step(7.0, 5.0), 2.0);
        assert_eq!(nice_step(0.9, 5.0), 0.2);
        assert_eq!(show_number(12500.0, 2500.0), "12,500");
        assert_eq!(show_number(0.4, 0.2), "0.4");
        assert_eq!(show_number(-1500.0, 500.0), "-1,500");
        assert_eq!(show_number(102.5, 2.5), "102.5");
        assert_eq!(show_number(0.25, 0.25), "0.25");
    }

    #[test]
    fn types_by_name() {
        assert_eq!(Type::from_name("Line"), Type::Line);
        assert_eq!(Type::from_name("donut"), Type::Doughnut);
        assert_eq!(Type::from_name("nonsense"), Type::Column);
    }
}
