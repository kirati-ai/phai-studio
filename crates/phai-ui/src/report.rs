//! Reports - FastReport's and DevExpress XtraReports' banded reports, drawn by Phai. A report is
//! bands one under another: the report header once at the start, the page header at the top of
//! every page, a group header and footer around the rows of each group, the detail band once per
//! row, the page footer at the bottom of every page, the report footer once at the end. Each band
//! holds items - words, a row's field, a total, a line, a box, a picture - placed in hundredths of
//! an inch from the band's corner, as [`crate::print`] measures the page.
//!
//! [`Report::layout`] lays the rows out on pages (no drawing, so it is tested as it is);
//! [`Report::preview`], [`Report::print`] and [`Report::export_pdf`] draw them through a
//! [`PrintDocument`]. A report's look is CSS, as a form's is: `label.page_header { font-weight:
//! 700 }`, `band.detail.odd { background: #f5f7fa }` - see [`REPORT_CSS`].

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use windows::Win32::Foundation::{COLORREF, HWND, RECT};
use windows::Win32::Graphics::Gdi::{self, HDC};

use crate::print::{Align, Bounds, PrintDocument, PrintPageEventArgs};
use crate::theme::{self, Color, Query, Theme};
use crate::win::wide;

/// Where a band prints.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BandKind {
    /// Once, before everything.
    ReportHeader,
    /// At the top of every page.
    PageHeader,
    /// Before the rows of each group of its level.
    GroupHeader,
    /// Once for every row.
    #[default]
    Detail,
    /// After the rows of each group of its level.
    GroupFooter,
    /// At the bottom of every page.
    PageFooter,
    /// Once, after everything.
    ReportFooter,
}

impl BandKind {
    pub const ALL: [BandKind; 7] = [BandKind::ReportHeader, BandKind::PageHeader, BandKind::GroupHeader, BandKind::Detail, BandKind::GroupFooter, BandKind::PageFooter, BandKind::ReportFooter];

    /// Its name in the file and in CSS: `page_header`.
    pub fn word(self) -> &'static str {
        match self {
            BandKind::ReportHeader => "report_header",
            BandKind::PageHeader => "page_header",
            BandKind::GroupHeader => "group_header",
            BandKind::Detail => "detail",
            BandKind::GroupFooter => "group_footer",
            BandKind::PageFooter => "page_footer",
            BandKind::ReportFooter => "report_footer",
        }
    }

    pub fn from_word(w: &str) -> Option<BandKind> {
        BandKind::ALL.into_iter().find(|k| k.word() == w)
    }

    /// As the designer captions it.
    pub fn caption(self) -> &'static str {
        match self {
            BandKind::ReportHeader => "Report Header",
            BandKind::PageHeader => "Page Header",
            BandKind::GroupHeader => "Group Header",
            BandKind::Detail => "Detail",
            BandKind::GroupFooter => "Group Footer",
            BandKind::PageFooter => "Page Footer",
            BandKind::ReportFooter => "Report Footer",
        }
    }
}

/// What an item is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemKind {
    /// Words; `{page}`, `{pages}`, `{date}`, `{time}`, `{row}` and `{column}` filled in.
    #[default]
    Label,
    /// A column of the row.
    Field,
    /// A column summed, counted, averaged, its least or most - over the group, the page or the
    /// whole report, as its band says.
    Total,
    /// A line from the top-left corner of its bounds to the bottom-right (a flat one when its
    /// height is 0).
    Line,
    /// The edge of a box.
    Box,
    /// A picture from a file.
    Picture,
}

impl ItemKind {
    pub const ALL: [ItemKind; 6] = [ItemKind::Label, ItemKind::Field, ItemKind::Total, ItemKind::Line, ItemKind::Box, ItemKind::Picture];

    pub fn word(self) -> &'static str {
        match self {
            ItemKind::Label => "label",
            ItemKind::Field => "field",
            ItemKind::Total => "total",
            ItemKind::Line => "line",
            ItemKind::Box => "box",
            ItemKind::Picture => "picture",
        }
    }

    pub fn from_word(w: &str) -> Option<ItemKind> {
        ItemKind::ALL.into_iter().find(|k| k.word() == w)
    }
}

/// What a total does with its column.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Func {
    #[default]
    Sum,
    Count,
    Avg,
    Min,
    Max,
}

impl Func {
    pub const ALL: [Func; 5] = [Func::Sum, Func::Count, Func::Avg, Func::Min, Func::Max];

    pub fn word(self) -> &'static str {
        match self {
            Func::Sum => "sum",
            Func::Count => "count",
            Func::Avg => "avg",
            Func::Min => "min",
            Func::Max => "max",
        }
    }

    pub fn from_word(w: &str) -> Option<Func> {
        Func::ALL.into_iter().find(|k| k.word() == w)
    }
}

/// One thing in a band. Lengths are hundredths of an inch; colours any CSS colour, empty for the
/// report's CSS.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Item {
    pub kind: ItemKind,
    /// Its name, for the designer and for CSS's `#name`.
    pub name: String,
    /// x, y (from the band's top-left corner), width, height.
    pub bounds: [i32; 4],
    /// A label's words.
    pub text: String,
    /// The column a field shows or a total counts.
    pub field: String,
    pub func: Func,
    /// Numbers written as `#,##0.00`, `0.0`, `฿#,##0` - empty: as they come.
    pub format: String,
    pub align: Align,
    /// The font's face, empty for Segoe UI.
    pub font: String,
    /// Points; 0 for the CSS's (10).
    pub size: i32,
    /// Over the CSS's `font-weight`.
    pub bold: Option<bool>,
    pub italic: bool,
    pub color: String,
    pub background: String,
    /// A box's or line's thickness, or a frame round words: hundredths of an inch, 0 for the CSS's.
    pub border: i32,
    /// CSS classes, space-separated.
    pub class: String,
    /// A picture's file.
    pub image: String,
}

/// A band and what is in it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Band {
    pub kind: BandKind,
    /// A group band's level: 0 the outermost group.
    pub group: usize,
    /// Hundredths of an inch.
    pub height: i32,
    /// A group header that starts a new page for each group.
    pub new_page: bool,
    pub items: Vec<Item>,
}

/// A whole report.
#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    /// In the printer's queue and the preview's title.
    pub title: String,
    pub landscape: bool,
    /// Left, top, right, bottom, hundredths of an inch.
    pub margins: [i32; 4],
    /// The column each level of groups breaks on, outermost first.
    pub groups: Vec<String>,
    pub bands: Vec<Band>,
    /// CSS over [`REPORT_CSS`].
    pub css: String,
}

impl Default for Report {
    fn default() -> Report {
        Report { title: "Report".into(), landscape: false, margins: [50; 4], groups: Vec::new(), bands: Vec::new(), css: String::new() }
    }
}

/// The rows a report shows: the columns' names and each row's values as text (`NULL` for none),
/// as phai-data's `Grid` holds them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Data {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Data {
    pub fn new(columns: Vec<String>, rows: Vec<Vec<String>>) -> Data {
        Data { columns, rows }
    }
    fn col(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c.eq_ignore_ascii_case(name))
    }
    fn value<'a>(&'a self, row: usize, name: &str) -> &'a str {
        let v = self.col(name).and_then(|c| self.rows.get(row)?.get(c)).map(String::as_str).unwrap_or("");
        if v == "NULL" {
            ""
        } else {
            v
        }
    }
}

/// How a report looks before its own CSS: black on white paper, the headers bold, the rows
/// striped lightly. Item kinds are the selectors' names (`label`, `field`, `total`, `line`,
/// `box`), each with its band's kind as a class (`.page_header`); bands are `band`, a detail row
/// `.odd` or `.even` too.
pub const REPORT_CSS: &str = "\
label, field, total { color: #1b1b1b; font-size: 10pt; }
line, box { color: #1b1b1b; border-color: #1b1b1b; border-width: 1px; }
label.report_header { font-size: 18pt; font-weight: 700; color: #0b3d6b; }
label.page_header { font-weight: 700; color: #374151; }
band.page_header { border-color: #9ca3af; border-width: 1px; }
label.group_header, field.group_header { font-weight: 700; font-size: 11pt; }
band.group_header { background: #eef2f7; }
band.detail.odd { background: #f7f9fb; }
total { font-weight: 700; }
label.page_footer, field.page_footer { color: #6b7280; font-size: 8pt; }
";

/// One thing drawn on a page, in hundredths of an inch from the paper's corner.
#[derive(Clone, Debug, PartialEq)]
pub enum Draw {
    Fill(Bounds, Color),
    Text { bounds: Bounds, text: String, align: Align, font: String, size: i32, bold: bool, italic: bool, color: Color },
    Line { from: (i32, i32), to: (i32, i32), color: Color, width: i32 },
    Rect { bounds: Bounds, color: Color, width: i32 },
    Image { bounds: Bounds, path: String },
}

/// One page, laid out.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Page {
    pub draws: Vec<Draw>,
}

impl Page {
    /// Every piece of words on the page, top to bottom - what a test reads.
    pub fn texts(&self) -> Vec<&str> {
        self.draws.iter().filter_map(|d| if let Draw::Text { text, .. } = d { Some(text.as_str()) } else { None }).collect()
    }
}

/// When the report is laid out: what `{date}` and `{time}` say.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Now {
    pub date: String,
    pub time: String,
}

impl Now {
    /// Today, from Windows' clock: `20/11/2026`, `14:05`.
    pub fn local() -> Now {
        let t = unsafe { windows::Win32::System::SystemInformation::GetLocalTime() };
        Now { date: format!("{}/{}/{}", t.wDay, t.wMonth, t.wYear), time: format!("{:02}:{:02}", t.wHour, t.wMinute) }
    }
}

const PAGES_MARK: &str = "\u{1}pages\u{1}";

/// A number as `format` writes it: `#,##0.00` two places and thousands parted, `0` none; what is
/// before and after the pattern kept (`฿#,##0`, `0.0 kg`). Not a number: as it is.
pub fn format_number(value: &str, format: &str) -> String {
    let Ok(n) = value.trim().parse::<f64>() else { return value.to_string() };
    if format.is_empty() {
        return plain(n);
    }
    let is_pat = |c: char| matches!(c, '#' | '0' | ',' | '.');
    let (Some(start), Some(end)) = (format.find(['#', '0']), format.rfind(['#', '0'])) else { return format!("{format}{}", plain(n)) };
    let mut start = start;
    while start > 0 && format[..start].ends_with(is_pat) {
        start -= 1;
    }
    let pat = &format[start..=end];
    let decimals = pat.split_once('.').map(|(_, d)| d.chars().filter(|c| *c == '0' || *c == '#').count()).unwrap_or(0);
    let thousands = pat.split_once('.').map(|(i, _)| i).unwrap_or(pat).contains(',');
    // Halves away from zero, as a spreadsheet rounds - not to even, as `{:.2}` does.
    let scale = 10f64.powi(decimals as i32);
    let fixed = format!("{:.*}", decimals, (n.abs() * scale).round() / scale);
    let (int, frac) = fixed.split_once('.').unwrap_or((&fixed, ""));
    let mut int_out = String::new();
    for (i, c) in int.chars().enumerate() {
        if thousands && i > 0 && (int.len() - i) % 3 == 0 {
            int_out.push(',');
        }
        int_out.push(c);
    }
    let sign = if n < 0.0 && fixed.chars().any(|c| c.is_ascii_digit() && c != '0') { "-" } else { "" };
    let num = if frac.is_empty() { int_out } else { format!("{int_out}.{frac}") };
    format!("{}{sign}{num}{}", &format[..start], &format[end + 1..])
}

/// A number with no format: whole when it is, else to six places with the zeros taken off.
fn plain(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        let s = format!("{n:.6}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Two values in order: as numbers when both are, else as words.
fn compare(a: &str, b: &str) -> std::cmp::Ordering {
    match (a.trim().parse::<f64>(), b.trim().parse::<f64>()) {
        (Ok(x), Ok(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
        _ => a.cmp(b),
    }
}

/// A total of `field` over `rows`.
fn total(data: &Data, rows: &[usize], field: &str, func: Func) -> Option<f64> {
    if func == Func::Count {
        return Some(if field.is_empty() { rows.len() } else { rows.iter().filter(|r| !data.value(**r, field).is_empty()).count() } as f64);
    }
    let nums: Vec<f64> = rows.iter().filter_map(|r| data.value(*r, field).trim().parse::<f64>().ok()).collect();
    if nums.is_empty() {
        return if func == Func::Sum { Some(0.0) } else { None };
    }
    Some(match func {
        Func::Sum => nums.iter().sum(),
        Func::Avg => nums.iter().sum::<f64>() / nums.len() as f64,
        Func::Min => nums.iter().cloned().fold(f64::INFINITY, f64::min),
        Func::Max => nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        Func::Count => unreachable!(),
    })
}

/// What a band is printed for: the row whose fields it shows and the rows its totals count.
struct Ctx<'a> {
    row: Option<usize>,
    rows: &'a [usize],
    /// The row's number in the report, from 1.
    number: usize,
}

struct Layout<'a> {
    report: &'a Report,
    data: &'a Data,
    theme: Theme,
    area: Bounds,
    now: &'a Now,
    pages: Vec<Page>,
    page: Page,
    y: i32,
    /// Rows printed on this page so far: what a page footer's totals count.
    page_rows: Vec<usize>,
    /// Nothing but the page header on the page yet.
    fresh: bool,
    /// Rows or groups on the page already: a group that starts a new page starts one then.
    rows_on_page: bool,
    started: bool,
    detail_count: usize,
    /// Drawn for the designer: fields as `[column]`, totals as `sum(column)`, words as written.
    design: bool,
}

impl<'a> Layout<'a> {
    fn band(&self, kind: BandKind, group: usize) -> Option<&'a Band> {
        self.report.bands.iter().find(|b| b.kind == kind && (!matches!(kind, BandKind::GroupHeader | BandKind::GroupFooter) || b.group == group))
    }

    fn footer_height(&self) -> i32 {
        self.band(BandKind::PageFooter, 0).map_or(0, |b| b.height)
    }

    fn start_page(&mut self) {
        self.started = true;
        self.y = self.area.y;
        self.page_rows.clear();
        if let Some(b) = self.band(BandKind::PageHeader, 0) {
            let all: Vec<usize> = Vec::new();
            self.draw_band(b, &Ctx { row: None, rows: &all, number: 0 }, self.y, &[]);
            self.y += b.height;
        }
        self.fresh = true;
        self.rows_on_page = false;
    }

    fn end_page(&mut self) {
        if let Some(b) = self.band(BandKind::PageFooter, 0) {
            let rows = self.page_rows.clone();
            let ctx = Ctx { row: rows.last().copied(), rows: &rows, number: 0 };
            self.draw_band(b, &ctx, self.area.bottom() - b.height, &[]);
        }
        self.pages.push(std::mem::take(&mut self.page));
        self.started = false;
    }

    /// `band` at the next place, on a new page when it does not fit on this one.
    fn put(&mut self, band: &Band, ctx: &Ctx, extra: &[&str]) {
        if !self.started {
            self.start_page();
        }
        let bottom = self.area.bottom() - self.footer_height();
        let breaks = band.kind == BandKind::GroupHeader && band.new_page && self.rows_on_page;
        if !self.fresh && (self.y + band.height > bottom || breaks) {
            self.end_page();
            self.start_page();
        }
        self.draw_band(band, ctx, self.y, extra);
        self.y += band.height;
        self.fresh = false;
        self.rows_on_page |= matches!(band.kind, BandKind::GroupHeader | BandKind::Detail | BandKind::GroupFooter);
    }

    fn style(&self, kind: &str, classes: &[&str], id: Option<&str>) -> theme::Style {
        self.theme.style(&Query { kind, classes, id, state: Default::default() })
    }

    fn draw_band(&mut self, band: &Band, ctx: &Ctx, top: i32, extra: &[&str]) {
        let word = band.kind.word();
        let mut classes = vec![word];
        classes.extend_from_slice(extra);
        let bs = self.style("band", &classes, None);
        let whole = Bounds { x: self.area.x, y: top, width: self.area.width, height: band.height };
        if let Some(bg) = bs.background.filter(|c| c.a > 0) {
            self.page.draws.push(Draw::Fill(whole, bg));
        }
        if let (Some(c), Some(w)) = (bs.border_color, bs.border_width.filter(|w| *w > 0)) {
            // A band's border is a rule under it, as a report's headers have.
            let y = whole.bottom() - 1;
            self.page.draws.push(Draw::Line { from: (whole.x, y), to: (whole.right(), y), color: c, width: px_to_units(w) });
        }
        for item in &band.items {
            let [x, y, w, h] = item.bounds;
            let b = Bounds { x: self.area.x + x, y: top + y, width: w, height: h };
            let mut cls: Vec<&str> = vec![word];
            cls.extend(item.class.split_whitespace());
            let id = (!item.name.is_empty()).then_some(item.name.as_str());
            let s = self.style(item.kind.word(), &cls, id);
            let color = theme::parse_color(&item.color).or(s.color).unwrap_or(Color::rgb(0x1b, 0x1b, 0x1b));
            let background = theme::parse_color(&item.background).or(s.background).filter(|c| c.a > 0);
            let border_color = s.border_color.or(theme::parse_color(&item.color)).unwrap_or(color);
            let border = if item.border > 0 { item.border } else { s.border_width.map(px_to_units).unwrap_or(0) };
            match item.kind {
                ItemKind::Line => {
                    let width = border.max(1);
                    self.page.draws.push(Draw::Line { from: (b.x, b.y), to: (b.right(), b.bottom()), color: theme::parse_color(&item.color).unwrap_or(border_color), width });
                    continue;
                }
                ItemKind::Box => {
                    if let Some(bg) = background {
                        self.page.draws.push(Draw::Fill(b, bg));
                    }
                    self.page.draws.push(Draw::Rect { bounds: b, color: theme::parse_color(&item.color).unwrap_or(border_color), width: border.max(1) });
                    continue;
                }
                ItemKind::Picture => {
                    self.page.draws.push(Draw::Image { bounds: b, path: item.image.clone() });
                    continue;
                }
                _ => {}
            }
            if let Some(bg) = background {
                self.page.draws.push(Draw::Fill(b, bg));
            }
            if border > 0 && item.border > 0 {
                self.page.draws.push(Draw::Rect { bounds: b, color: border_color, width: border });
            }
            let text = match item.kind {
                ItemKind::Field if self.design => format!("[{}]", item.field),
                ItemKind::Total if self.design => format!("{}({})", item.func.word(), item.field),
                _ if self.design => item.text.clone(),
                ItemKind::Field => match ctx.row {
                    Some(r) => format_value(self.data.value(r, &item.field), &item.format),
                    None => String::new(),
                },
                ItemKind::Total => total(self.data, ctx.rows, &item.field, item.func).map(|n| format_number(&plain(n), &item.format)).unwrap_or_default(),
                _ => self.fill(&item.text, ctx),
            };
            let size = if item.size > 0 { item.size } else { s.font_size.unwrap_or(10) };
            let bold = item.bold.unwrap_or(s.font_weight.unwrap_or(400) >= 600);
            self.page.draws.push(Draw::Text { bounds: b, text, align: item.align, font: item.font.clone(), size, bold, italic: item.italic, color });
        }
    }

    /// A label's words with its `{...}` filled in.
    fn fill(&self, text: &str, ctx: &Ctx) -> String {
        let mut out = String::new();
        let mut rest = text;
        while let Some(open) = rest.find('{') {
            out.push_str(&rest[..open]);
            let after = &rest[open + 1..];
            let Some(close) = after.find('}') else {
                out.push_str(&rest[open..]);
                rest = "";
                break;
            };
            let name = after[..close].trim();
            match name {
                "page" => out.push_str(&(self.pages.len() + 1).to_string()),
                "pages" => out.push_str(PAGES_MARK),
                "date" => out.push_str(&self.now.date),
                "time" => out.push_str(&self.now.time),
                "row" => out.push_str(&ctx.number.to_string()),
                "title" => out.push_str(&self.report.title),
                _ => match (ctx.row, self.data.col(name)) {
                    (Some(r), Some(_)) => out.push_str(self.data.value(r, name)),
                    (None, Some(_)) => {}
                    _ => out.push_str(&rest[open..open + close + 2]),
                },
            }
            rest = &after[close + 1..];
        }
        out.push_str(rest);
        out
    }

    /// The rows of `rows` at group `level` and below, each group between its header and footer.
    fn emit(&mut self, level: usize, rows: &[usize]) {
        if level >= self.report.groups.len() {
            let detail = self.band(BandKind::Detail, 0);
            for &r in rows {
                self.detail_count += 1;
                if let Some(b) = detail {
                    let n = self.detail_count;
                    let parity = if n.is_multiple_of(2) { "even" } else { "odd" };
                    self.put(b, &Ctx { row: Some(r), rows: std::slice::from_ref(&r), number: n }, &[parity]);
                }
                // Counted on the page it went on: put has begun a new one when it did not fit.
                self.page_rows.push(r);
            }
            return;
        }
        let field = self.report.groups[level].clone();
        let mut i = 0;
        while i < rows.len() {
            let v = self.data.value(rows[i], &field).to_string();
            let mut j = i + 1;
            while j < rows.len() && self.data.value(rows[j], &field) == v {
                j += 1;
            }
            let run = &rows[i..j];
            if let Some(b) = self.band(BandKind::GroupHeader, level) {
                self.put(b, &Ctx { row: Some(run[0]), rows: run, number: self.detail_count + 1 }, &[]);
            }
            self.emit(level + 1, run);
            if let Some(b) = self.band(BandKind::GroupFooter, level) {
                self.put(b, &Ctx { row: run.last().copied(), rows: run, number: self.detail_count }, &[]);
            }
            i = j;
        }
    }
}

/// A field's value: a number through `format` when it has one, else as it is.
fn format_value(v: &str, format: &str) -> String {
    if format.is_empty() {
        v.to_string()
    } else {
        format_number(v, format)
    }
}

/// CSS pixels at 96 DPI in hundredths of an inch.
fn px_to_units(px: i32) -> i32 {
    (px * 100 / 96).max(1)
}

impl Report {
    /// The report's look: [`REPORT_CSS`] and its own CSS over it - the built-in one alone when its
    /// own does not read.
    pub fn theme(&self) -> Result<Theme, theme::ThemeError> {
        Theme::parse(&format!("{REPORT_CSS}\n{}", self.css))
    }

    /// The bands of `kind` (of group `level`, for a group's).
    pub fn band(&self, kind: BandKind, level: usize) -> Option<&Band> {
        self.bands.iter().find(|b| b.kind == kind && b.group == level)
    }

    /// The rows sorted by the groups' columns - the order they print in; rows equal in them keep
    /// the order they came in.
    pub fn order(&self, data: &Data) -> Vec<usize> {
        let mut rows: Vec<usize> = (0..data.rows.len()).collect();
        if !self.groups.is_empty() {
            rows.sort_by(|a, b| {
                for g in &self.groups {
                    let o = compare(data.value(*a, g), data.value(*b, g));
                    if o != std::cmp::Ordering::Equal {
                        return o;
                    }
                }
                std::cmp::Ordering::Equal
            });
        }
        rows
    }

    /// `data` laid out on pages whose printable part is `area`, as it prints `now`.
    pub fn layout(&self, data: &Data, area: Bounds, now: &Now) -> Vec<Page> {
        let theme = self.theme().unwrap_or_else(|_| Theme::parse(REPORT_CSS).expect("the report's built-in CSS reads"));
        let mut l = Layout { report: self, data, theme, area, now, pages: Vec::new(), page: Page::default(), y: area.y, page_rows: Vec::new(), fresh: true, rows_on_page: false, started: false, detail_count: 0, design: false };
        let all = self.order(data);
        // The report header first on the first page, its page header under it.
        if let Some(b) = l.band(BandKind::ReportHeader, 0) {
            l.started = true;
            l.draw_band(b, &Ctx { row: all.first().copied(), rows: &all, number: 0 }, l.y, &[]);
            l.y += b.height;
            if let Some(h) = l.band(BandKind::PageHeader, 0) {
                l.draw_band(h, &Ctx { row: None, rows: &[], number: 0 }, l.y, &[]);
                l.y += h.height;
            }
            l.fresh = true;
        }
        l.emit(0, &all);
        if let Some(b) = l.band(BandKind::ReportFooter, 0) {
            l.put(b, &Ctx { row: all.last().copied(), rows: &all, number: l.detail_count }, &[]);
        }
        if !l.started {
            l.start_page();
        }
        l.end_page();
        let n = l.pages.len().to_string();
        for p in &mut l.pages {
            for d in &mut p.draws {
                if let Draw::Text { text, .. } = d {
                    if text.contains(PAGES_MARK) {
                        *text = text.replace(PAGES_MARK, &n);
                    }
                }
            }
        }
        l.pages
    }

    /// `band` as the designer shows it, `width` wide, its top-left corner at (0, 0): the CSS's
    /// look, with fields and totals named rather than filled.
    pub fn design_band(&self, band: &Band, width: i32) -> Vec<Draw> {
        let theme = self.theme().unwrap_or_else(|_| Theme::parse(REPORT_CSS).expect("the report's built-in CSS reads"));
        let (data, now) = (Data::default(), Now::default());
        let area = Bounds { x: 0, y: 0, width, height: band.height };
        let mut l = Layout { report: self, data: &data, theme, area, now: &now, pages: Vec::new(), page: Page::default(), y: 0, page_rows: Vec::new(), fresh: true, rows_on_page: false, started: true, detail_count: 0, design: true };
        l.draw_band(band, &Ctx { row: None, rows: &[], number: 0 }, 0, &[]);
        l.page.draws
    }

    /// A print document drawing this report's pages of `data`: laid out again on its first page,
    /// on the paper the printer has.
    pub fn document(&self, owner: HWND, data: Data) -> PrintDocument {
        let doc = PrintDocument::new(owner, 0, &self.title, self.landscape);
        let [l, t, r, b] = self.margins;
        doc.set_margins(l, t, r, b);
        let report = self.clone();
        let pages: Rc<RefCell<Vec<Page>>> = Rc::default();
        doc.set_painter(Box::new(move |e: &PrintPageEventArgs| {
            if e.page == 1 {
                *pages.borrow_mut() = report.layout(&data, e.margin_bounds, &Now::local());
            }
            let pages = pages.borrow();
            if let Some(p) = pages.get(e.page - 1) {
                draw(e.hdc(), &p.draws);
            }
            e.set_has_more_pages(e.page < pages.len());
        }));
        doc
    }

    /// The pages in a window first, its Print button printing them - waits until it closes.
    pub fn preview(&self, owner: HWND, data: Data) {
        self.document(owner, data).print_preview();
    }

    /// Windows' Print dialog, then the pages on the printer chosen: how many; 0 when cancelled.
    pub fn print(&self, owner: HWND, data: Data) -> usize {
        self.document(owner, data).print_dialog()
    }

    /// Printed on Windows' default printer, asking nothing: how many pages.
    pub fn print_default(&self, owner: HWND, data: Data) -> usize {
        self.document(owner, data).print()
    }

    /// Saved as a PDF at `path` through Windows' own "Microsoft Print to PDF": how many pages; 0
    /// when this Windows has not that printer.
    pub fn export_pdf(&self, owner: HWND, data: Data, path: &Path) -> usize {
        let pdf = crate::print::printers().into_iter().find(|p| p.contains("PDF")).unwrap_or_else(|| "Microsoft Print to PDF".into());
        self.document(owner, data).print_to_file(&pdf, path)
    }
}

/// `draws` on `dc`, whose units are hundredths of an inch - a printer's page, a preview's picture,
/// or a designer's canvas mapped so.
pub fn draw(dc: HDC, draws: &[Draw]) {
    unsafe {
        Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        for d in draws {
            match d {
                Draw::Fill(b, c) => {
                    let brush = Gdi::CreateSolidBrush(COLORREF(c.colorref()));
                    let _ = Gdi::FillRect(dc, &rect(*b), brush);
                    let _ = Gdi::DeleteObject(brush.into());
                }
                Draw::Text { bounds, text, align, font, size, bold, italic, color } => {
                    let face = if font.is_empty() { "Segoe UI" } else { font.as_str() };
                    let mut lf = Gdi::LOGFONTW { lfHeight: -((*size).max(1) * 100 / 72), lfWeight: if *bold { 700 } else { 400 }, lfItalic: *italic as u8, lfQuality: Gdi::CLEARTYPE_QUALITY, lfCharSet: Gdi::DEFAULT_CHARSET, ..Default::default() };
                    for (i, c) in face.encode_utf16().take(31).enumerate() {
                        lf.lfFaceName[i] = c;
                    }
                    let f = Gdi::CreateFontIndirectW(&lf);
                    let old = Gdi::SelectObject(dc, f.into());
                    Gdi::SetTextColor(dc, COLORREF(color.colorref()));
                    let a = match align {
                        Align::Left => Gdi::DT_LEFT,
                        Align::Center => Gdi::DT_CENTER,
                        Align::Right => Gdi::DT_RIGHT,
                    };
                    let mut w = wide(text);
                    let n = w.len().saturating_sub(1);
                    let mut r = rect(*bounds);
                    // A little room inside, as a report's text boxes have.
                    r.left += 2;
                    r.right -= 2;
                    crate::win::draw_text_w(dc, &mut w[..n], &mut r, a | Gdi::DT_WORDBREAK | Gdi::DT_NOPREFIX | Gdi::DT_NOCLIP);
                    Gdi::SelectObject(dc, old);
                    let _ = Gdi::DeleteObject(f.into());
                }
                Draw::Line { from, to, color, width } => {
                    let pen = Gdi::CreatePen(Gdi::PS_SOLID, *width, COLORREF(color.colorref()));
                    let old = Gdi::SelectObject(dc, pen.into());
                    let _ = Gdi::MoveToEx(dc, from.0, from.1, None);
                    let _ = Gdi::LineTo(dc, to.0, to.1);
                    Gdi::SelectObject(dc, old);
                    let _ = Gdi::DeleteObject(pen.into());
                }
                Draw::Rect { bounds, color, width } => {
                    let pen = Gdi::CreatePen(Gdi::PS_SOLID, *width, COLORREF(color.colorref()));
                    let old = Gdi::SelectObject(dc, pen.into());
                    let old_brush = Gdi::SelectObject(dc, Gdi::GetStockObject(Gdi::NULL_BRUSH));
                    let _ = Gdi::Rectangle(dc, bounds.x, bounds.y, bounds.right(), bounds.bottom());
                    Gdi::SelectObject(dc, old_brush);
                    Gdi::SelectObject(dc, old);
                    let _ = Gdi::DeleteObject(pen.into());
                }
                Draw::Image { bounds, path } => draw_image(dc, path, *bounds),
            }
        }
    }
}

fn rect(b: Bounds) -> RECT {
    RECT { left: b.x, top: b.y, right: b.right(), bottom: b.bottom() }
}

/// A picture stretched into `b`; a grey frame when it cannot be read.
fn draw_image(dc: HDC, path: &str, b: Bounds) {
    let found = crate::picture::resolve(path);
    let Some(img) = found.as_deref().and_then(crate::picture::load) else {
        unsafe {
            let pen = Gdi::CreatePen(Gdi::PS_DOT, 1, COLORREF(0x9c9c9c));
            let old = Gdi::SelectObject(dc, pen.into());
            let old_brush = Gdi::SelectObject(dc, Gdi::GetStockObject(Gdi::NULL_BRUSH));
            let _ = Gdi::Rectangle(dc, b.x, b.y, b.right(), b.bottom());
            Gdi::SelectObject(dc, old_brush);
            Gdi::SelectObject(dc, old);
            let _ = Gdi::DeleteObject(pen.into());
        }
        return;
    };
    unsafe {
        let mem = Gdi::CreateCompatibleDC(Some(dc));
        let old = Gdi::SelectObject(mem, img.bmp.into());
        Gdi::SetStretchBltMode(dc, Gdi::HALFTONE);
        let _ = Gdi::StretchBlt(dc, b.x, b.y, b.width, b.height, Some(mem), 0, 0, img.w, img.h, Gdi::SRCCOPY);
        Gdi::SelectObject(mem, old);
        let _ = Gdi::DeleteDC(mem);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(kind: ItemKind, x: i32, w: i32) -> Item {
        Item { kind, bounds: [x, 0, w, 20], ..Default::default() }
    }

    fn sales() -> (Report, Data) {
        let data = Data::new(
            vec!["region".into(), "name".into(), "amount".into()],
            vec![
                vec!["North".into(), "Somchai".into(), "100".into()],
                vec!["South".into(), "Monrak".into(), "250.5".into()],
                vec!["North".into(), "Kirati".into(), "50".into()],
                vec!["South".into(), "Nok".into(), "NULL".into()],
            ],
        );
        let field = |f: &str, x| Item { field: f.into(), ..item(ItemKind::Field, x, 200) };
        let total = |func, f: &str| Item { func, field: f.into(), format: "#,##0.00".into(), ..item(ItemKind::Total, 400, 200) };
        let report = Report {
            title: "Sales".into(),
            groups: vec!["region".into()],
            bands: vec![
                Band { kind: BandKind::ReportHeader, height: 50, items: vec![Item { text: "{title}".into(), ..item(ItemKind::Label, 0, 400) }], ..Default::default() },
                Band { kind: BandKind::PageHeader, height: 25, items: vec![Item { text: "Name".into(), ..item(ItemKind::Label, 0, 200) }], ..Default::default() },
                Band { kind: BandKind::GroupHeader, height: 25, items: vec![Item { text: "Region: {region}".into(), ..item(ItemKind::Label, 0, 300) }], ..Default::default() },
                Band { kind: BandKind::Detail, height: 20, items: vec![field("name", 0), Item { format: "#,##0.00".into(), ..field("amount", 400) }], ..Default::default() },
                Band { kind: BandKind::GroupFooter, height: 25, items: vec![total(Func::Sum, "amount")], ..Default::default() },
                Band { kind: BandKind::PageFooter, height: 20, items: vec![Item { text: "Page {page} of {pages}".into(), ..item(ItemKind::Label, 0, 300) }], ..Default::default() },
                Band { kind: BandKind::ReportFooter, height: 25, items: vec![total(Func::Sum, "amount"), Item { func: Func::Count, format: String::new(), ..total(Func::Count, "") }], ..Default::default() },
            ],
            ..Default::default()
        };
        (report, data)
    }

    fn now() -> Now {
        Now { date: "20/11/2026".into(), time: "09:30".into() }
    }

    #[test]
    fn rows_print_in_their_groups_with_totals() {
        let (report, data) = sales();
        let pages = report.layout(&data, Bounds { x: 50, y: 50, width: 750, height: 1000 }, &now());
        assert_eq!(pages.len(), 1);
        let t = pages[0].texts();
        assert_eq!(
            t,
            ["Sales", "Name", "Region: North", "Somchai", "100.00", "Kirati", "50.00", "150.00", "Region: South", "Monrak", "250.50", "Nok", "", "250.50", "400.50", "4", "Page 1 of 1"],
            "the report header first, then the page header, each group between its header and its total, the page footer last"
        );
    }

    #[test]
    fn rows_that_do_not_fit_go_on_the_next_page_under_its_header() {
        let (mut report, mut data) = sales();
        report.groups.clear();
        report.bands.retain(|b| !matches!(b.kind, BandKind::GroupHeader | BandKind::GroupFooter));
        data.rows = (1..=30).map(|i| vec!["N".into(), format!("row {i}"), i.to_string()]).collect();
        // 200 high: page header 25 + footer 20 leaves 155; the report header 50, then 5 rows of 20.
        let pages = report.layout(&data, Bounds { x: 0, y: 0, width: 700, height: 200 }, &now());
        let first = pages[0].texts();
        assert!(first.contains(&"row 5") && !first.contains(&"row 6"), "{first:?}");
        let second = pages[1].texts();
        assert_eq!(second[0], "Name", "each page begins with the page header");
        assert_eq!(second[1], "row 6");
        let last = pages.last().unwrap().texts();
        assert_eq!(last.last().copied(), Some(format!("Page {0} of {0}", pages.len()).as_str()));
        assert!(last.contains(&"465.00"), "the report's total, 1 + ... + 30: {last:?}");
    }

    #[test]
    fn a_page_footer_totals_its_own_page() {
        let (mut report, mut data) = sales();
        report.groups.clear();
        report.bands.retain(|b| matches!(b.kind, BandKind::Detail | BandKind::PageFooter));
        report.bands[1].items = vec![Item { func: Func::Sum, field: "amount".into(), ..item(ItemKind::Total, 0, 100) }];
        data.rows = (1..=10).map(|i| vec!["N".into(), format!("r{i}"), "1".into()]).collect();
        // 120 high, footer 20: five rows a page.
        let pages = report.layout(&data, Bounds { x: 0, y: 0, width: 700, height: 120 }, &now());
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].texts().last().copied(), Some("5"));
        assert_eq!(pages[1].texts().last().copied(), Some("5"));
    }

    #[test]
    fn a_group_can_start_a_new_page() {
        let (mut report, data) = sales();
        report.bands.iter_mut().find(|b| b.kind == BandKind::GroupHeader).unwrap().new_page = true;
        let pages = report.layout(&data, Bounds { x: 0, y: 0, width: 700, height: 1000 }, &now());
        assert_eq!(pages.len(), 2);
        assert!(pages[1].texts().contains(&"Region: South"));
    }

    #[test]
    fn numbers_take_their_format() {
        assert_eq!(format_number("1234567.891", "#,##0.00"), "1,234,567.89");
        assert_eq!(format_number("1234.5", "0"), "1235");
        assert_eq!(format_number("-1234.5", "฿#,##0.0"), "฿-1,234.5");
        assert_eq!(format_number("12", "0.0 kg"), "12.0 kg");
        assert_eq!(format_number("abc", "0.00"), "abc");
        assert_eq!(format_number("0.1", ""), "0.1");
        assert_eq!(plain(0.1 + 0.2), "0.3");
        assert_eq!(total(&sales().1, &[0, 1, 2, 3], "amount", Func::Avg), Some(400.5 / 3.0));
        assert_eq!(total(&sales().1, &[0, 1, 2, 3], "amount", Func::Count), Some(3.0), "NULL is not counted");
        assert_eq!(total(&sales().1, &[3], "amount", Func::Max), None);
    }

    #[test]
    fn the_look_comes_from_css() {
        let (mut report, data) = sales();
        report.css = "band.detail.odd { background: #ff0000; } label.page_header { color: #00ff00; }".into();
        let pages = report.layout(&data, Bounds { x: 0, y: 0, width: 700, height: 1000 }, &now());
        let fills = pages[0].draws.iter().filter(|d| matches!(d, Draw::Fill(_, c) if *c == Color::rgb(255, 0, 0))).count();
        assert_eq!(fills, 2, "rows 1 and 3 striped");
        assert!(pages[0].draws.iter().any(|d| matches!(d, Draw::Text { text, color, .. } if text == "Name" && *color == Color::rgb(0, 255, 0))));
        assert!(report.theme().is_ok());
        report.css = "label { colour: red }".into();
        assert!(report.theme().is_err(), "a sheet that does not read is told");
    }

    #[test]
    fn the_designer_sees_names_not_values() {
        let (report, _) = sales();
        let detail = report.bands.iter().find(|b| b.kind == BandKind::Detail).unwrap();
        let p = Page { draws: report.design_band(detail, 700) };
        assert_eq!(p.texts(), ["[name]", "[amount]"]);
        let footer = report.bands.iter().find(|b| b.kind == BandKind::ReportFooter).unwrap();
        assert_eq!(Page { draws: report.design_band(footer, 700) }.texts(), ["sum(amount)", "count()"]);
    }

    #[test]
    fn the_words_and_kinds_go_both_ways() {
        for k in BandKind::ALL {
            assert_eq!(BandKind::from_word(k.word()), Some(k));
        }
        for k in ItemKind::ALL {
            assert_eq!(ItemKind::from_word(k.word()), Some(k));
        }
        for k in Func::ALL {
            assert_eq!(Func::from_word(k.word()), Some(k));
        }
    }
}
