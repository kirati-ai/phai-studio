//! The `.phaireport` file (docs/phaireport.md): a banded report - its bands, their items, where
//! its rows come from - as TOML, read, checked, written, and turned into Rust: `<name>.rs` with
//! `report()`, `data()`, `preview()`, `print()` and `export_pdf()`, which Phai Studio writes
//! whole on every save, as it writes a form's `_ui.rs`.
//!
//! ```toml
//! format = 1
//!
//! [report]
//! name = "SalesReport"
//! title = "Sales by region"
//! groups = ["region"]
//!
//! [data]
//! database = "main"
//! table = "orders"
//! columns = ["region", "customer", "amount"]
//!
//! [[band]]
//! kind = "detail"
//! height = 22
//! [[band.item]]
//! kind = "field"
//! name = "customer"
//! bounds = [0, 2, 300, 18]
//! field = "customer"
//! ```

use std::fmt::Write;

use phai_ui::print::Align;
use phai_ui::report::{self as rp, BandKind, Func, ItemKind};
use serde::{Deserialize, Serialize};

pub const FORMAT: u32 = 1;

/// A whole `.phaireport`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportFile {
    pub format: u32,
    pub report: ReportHead,
    #[serde(default, skip_serializing_if = "DataSpec::is_empty")]
    pub data: DataSpec,
    #[serde(default, rename = "band", skip_serializing_if = "Vec::is_empty")]
    pub bands: Vec<BandSpec>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportHead {
    /// PascalCase: `SalesReport`, its file `sales_report.phaireport`.
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub title: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub landscape: bool,
    /// Left, top, right, bottom, hundredths of an inch.
    #[serde(default = "margins", skip_serializing_if = "is_margins")]
    pub margins: [i32; 4],
    /// The columns its groups break on, outermost first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<String>,
    /// CSS over the report's built-in look (`phai_ui::report::REPORT_CSS`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub css: String,
}

/// Where the rows come from: a table of the project's database (`src/data_sources.rs`), or SQL
/// of its own.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataSpec {
    /// `main` for SQLite.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub database: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub table: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub columns: Vec<String>,
    /// SQL's WHERE: `amount > 0`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub filter: String,
    /// Sorted by these after the groups: `["customer", "amount desc"]`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub order: Vec<String>,
    /// A query of its own in place of `table` - it must sort by the groups itself.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub sql: String,
    /// Design only: the Server Explorer connection the designer previews with.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub connection: String,
}

impl DataSpec {
    pub fn is_empty(&self) -> bool {
        *self == DataSpec::default()
    }
    /// Rows come from somewhere.
    pub fn is_bound(&self) -> bool {
        !self.table.is_empty() || !self.sql.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BandSpec {
    /// `report_header`, `page_header`, `group_header`, `detail`, `group_footer`, `page_footer`,
    /// `report_footer`.
    pub kind: String,
    /// A group band's level, 0 the outermost.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub group: usize,
    pub height: i32,
    #[serde(default, skip_serializing_if = "is_false")]
    pub new_page: bool,
    #[serde(default, rename = "item", skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ItemSpec>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemSpec {
    /// `label`, `field`, `total`, `line`, `box`, `picture`.
    pub kind: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// x, y from the band's corner, width, height - hundredths of an inch.
    pub bounds: [i32; 4],
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub field: String,
    /// A total's: `sum`, `count`, `avg`, `min`, `max`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub func: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub format: String,
    /// `left`, `center`, `right`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub align: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub font: String,
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub size: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub italic: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub color: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub background: String,
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub border: i32,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub class: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub image: String,
}

fn is_false(b: &bool) -> bool {
    !*b
}
fn is_zero(n: &usize) -> bool {
    *n == 0
}
fn is_zero_i(n: &i32) -> bool {
    *n == 0
}
fn margins() -> [i32; 4] {
    [50; 4]
}
fn is_margins(m: &[i32; 4]) -> bool {
    *m == margins()
}

/// What is wrong with a report, and where.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for ReportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line > 0 {
            write!(f, "line {}: {}", self.line, self.message)
        } else {
            f.write_str(&self.message)
        }
    }
}

fn align_of(w: &str) -> Option<Align> {
    match w {
        "" | "left" => Some(Align::Left),
        "center" => Some(Align::Center),
        "right" => Some(Align::Right),
        _ => None,
    }
}

pub fn align_word(a: Align) -> &'static str {
    match a {
        Align::Left => "",
        Align::Center => "center",
        Align::Right => "right",
    }
}

fn is_pascal(s: &str) -> bool {
    s.chars().next().is_some_and(|c| c.is_ascii_uppercase()) && s.chars().all(|c| c.is_ascii_alphanumeric())
}

impl ReportFile {
    /// A new report named `name`: a title in the report header, a page header, the detail band,
    /// a page footer with the page number - as a report wizard's blank one.
    pub fn new(name: &str) -> ReportFile {
        let label = |name: &str, bounds, text: &str| ItemSpec { kind: "label".into(), name: name.into(), bounds, text: text.into(), ..Default::default() };
        let mut page_no = label("page_number", [450, 4, 300, 18], "Page {page} of {pages}");
        page_no.align = "right".into();
        ReportFile {
            format: FORMAT,
            report: ReportHead { name: name.into(), title: name.into(), landscape: false, margins: margins(), groups: Vec::new(), css: String::new() },
            data: DataSpec::default(),
            bands: vec![
                BandSpec { kind: "report_header".into(), height: 50, items: vec![label("title", [0, 6, 600, 36], "{title}")], ..Default::default() },
                BandSpec { kind: "page_header".into(), height: 26, ..Default::default() },
                BandSpec { kind: "detail".into(), height: 22, ..Default::default() },
                BandSpec { kind: "page_footer".into(), height: 26, items: vec![label("printed", [0, 4, 300, 18], "{date} {time}"), page_no], ..Default::default() },
            ],
        }
    }

    pub fn parse(src: &str) -> Result<ReportFile, ReportError> {
        let f: ReportFile = toml::from_str(src).map_err(|e| ReportError { line: e.span().map(|s| src[..s.start.min(src.len())].matches('\n').count() + 1).unwrap_or(0), message: e.message().to_string() })?;
        if let Some(p) = f.check().into_iter().next() {
            let line = p.1.as_deref().and_then(|n| src.lines().position(|l| l.contains(n))).map_or(0, |i| i + 1);
            return Err(ReportError { line, message: p.0 });
        }
        Ok(f)
    }

    pub fn to_toml(&self) -> String {
        format!("# Phai Studio report - docs/phaireport.md\n{}", toml::to_string(self).unwrap_or_default())
    }

    /// The module the generated code is: `sales_report` for SalesReport.
    pub fn module(&self) -> String {
        crate::model::snake(&self.report.name)
    }

    /// Everything wrong with it, each with text to find its line by. Empty when it is good.
    pub fn check(&self) -> Vec<(String, Option<String>)> {
        let mut out = Vec::new();
        if self.format != FORMAT {
            out.push((format!("format {} - this Phai Studio reads format {FORMAT}", self.format), Some("format".into())));
        }
        if !is_pascal(&self.report.name) {
            out.push((format!("report name {:?} must be PascalCase, like SalesReport", self.report.name), Some("name".into())));
        }
        let mut seen = std::collections::HashSet::new();
        for b in &self.bands {
            let Some(kind) = BandKind::from_word(&b.kind) else {
                out.push((format!("band kind {:?}: report_header, page_header, group_header, detail, group_footer, page_footer or report_footer", b.kind), Some(b.kind.clone())));
                continue;
            };
            let grouped = matches!(kind, BandKind::GroupHeader | BandKind::GroupFooter);
            if grouped && b.group >= self.report.groups.len() {
                out.push((format!("{} {}: the report has {} group(s) - add the column to `groups`", b.kind, b.group, self.report.groups.len()), Some(b.kind.clone())));
            }
            if !seen.insert((kind, if grouped { b.group } else { 0 })) {
                out.push((format!("two {} bands{}", b.kind, if grouped { format!(" for group {}", b.group) } else { String::new() }), Some(b.kind.clone())));
            }
            if b.height < 0 {
                out.push((format!("{}: height {} - not below 0", b.kind, b.height), Some(b.kind.clone())));
            }
            for it in &b.items {
                let at = Some(if it.name.is_empty() { it.kind.clone() } else { it.name.clone() });
                match ItemKind::from_word(&it.kind) {
                    None => out.push((format!("item kind {:?}: label, field, total, line, box or picture", it.kind), at.clone())),
                    Some(ItemKind::Field) if it.field.is_empty() => out.push((format!("field {:?}: which column? (`field = \"...\"`)", it.name), at.clone())),
                    Some(ItemKind::Total) if it.field.is_empty() && it.func != "count" => out.push((format!("total {:?}: which column? (`field = \"...\"`)", it.name), at.clone())),
                    _ => {}
                }
                if !it.func.is_empty() && Func::from_word(&it.func).is_none() {
                    out.push((format!("{}: func {:?} - sum, count, avg, min or max", it.name, it.func), at.clone()));
                }
                if align_of(&it.align).is_none() {
                    out.push((format!("{}: align {:?} - left, center or right", it.name, it.align), at.clone()));
                }
                let cols = &self.data.columns;
                if !cols.is_empty() && !it.field.is_empty() && !cols.iter().any(|c| c.eq_ignore_ascii_case(&it.field)) {
                    out.push((format!("{}: the column {:?} is not among the data's columns", it.name, it.field), at.clone()));
                }
            }
        }
        if let Err(e) = phai_ui::theme::Theme::parse(&format!("{}\n{}", rp::REPORT_CSS, self.report.css)) {
            out.push((format!("css: {}", e.message), Some("css".into())));
        }
        out
    }

    /// The report as phai-ui draws it.
    pub fn to_report(&self) -> rp::Report {
        rp::Report {
            title: if self.report.title.is_empty() { self.report.name.clone() } else { self.report.title.clone() },
            landscape: self.report.landscape,
            margins: self.report.margins,
            groups: self.report.groups.clone(),
            css: self.report.css.clone(),
            bands: self
                .bands
                .iter()
                .filter_map(|b| {
                    Some(rp::Band {
                        kind: BandKind::from_word(&b.kind)?,
                        group: b.group,
                        height: b.height,
                        new_page: b.new_page,
                        items: b
                            .items
                            .iter()
                            .filter_map(|i| {
                                Some(rp::Item {
                                    kind: ItemKind::from_word(&i.kind)?,
                                    name: i.name.clone(),
                                    bounds: i.bounds,
                                    text: i.text.clone(),
                                    field: i.field.clone(),
                                    func: Func::from_word(&i.func).unwrap_or_default(),
                                    format: i.format.clone(),
                                    align: align_of(&i.align).unwrap_or_default(),
                                    font: i.font.clone(),
                                    size: i.size,
                                    bold: i.bold,
                                    italic: i.italic,
                                    color: i.color.clone(),
                                    background: i.background.clone(),
                                    border: i.border,
                                    class: i.class.clone(),
                                    image: i.image.clone(),
                                })
                            })
                            .collect(),
                    })
                })
                .collect(),
        }
    }

    /// The columns its fields and totals name, in the order they come - what a report reads
    /// when its data names none.
    pub fn used_columns(&self) -> Vec<String> {
        let mut out: Vec<String> = self.report.groups.clone();
        for b in &self.bands {
            for i in &b.items {
                if !i.field.is_empty() && !out.contains(&i.field) {
                    out.push(i.field.clone());
                }
            }
        }
        out
    }

    /// The sort a table's rows are read in: the groups, then `order`.
    pub fn sort(&self) -> Vec<String> {
        let mut out = self.report.groups.clone();
        out.extend(self.data.order.iter().cloned());
        out
    }
}

fn lit(s: &str) -> String {
    format!("{s:?}")
}

/// `<name>.rs`, written whole on every save: the report built, its rows read from the project's
/// database (`crate::data_sources::connection()`), and the calls that show, print and save it.
pub fn generate(f: &ReportFile, file_name: &str) -> String {
    let r = f.to_report();
    let mut w = String::new();
    let _ = writeln!(w, "//! Generated by Phai Studio from `{file_name}` - do not edit: it is written again on every save.");
    let _ = writeln!(w, "//! Show it with `{}::preview(hwnd)`, print it with `{0}::print(hwnd)`, save it as a PDF with", f.module());
    let _ = writeln!(w, "//! `{}::export_pdf(hwnd, path)` - or lay out rows of your own with `report()` and `phai_ui::report::Data`.", f.module());
    let _ = writeln!(w);
    let _ = writeln!(w, "#![allow(dead_code)]");
    let _ = writeln!(w);
    let _ = writeln!(w, "use phai_ui::print::Align;");
    let _ = writeln!(w, "use phai_ui::report::{{Band, BandKind, Data, Func, Item, ItemKind, Report}};");
    let _ = writeln!(w, "use windows::Win32::Foundation::HWND;");
    let _ = writeln!(w);
    let _ = writeln!(w, "/// The report as designed.");
    let _ = writeln!(w, "pub fn report() -> Report {{");
    let _ = writeln!(w, "    Report {{");
    let _ = writeln!(w, "        title: {}.into(),", lit(&r.title));
    let _ = writeln!(w, "        landscape: {},", r.landscape);
    let _ = writeln!(w, "        margins: {:?},", r.margins);
    let _ = writeln!(w, "        groups: vec![{}],", r.groups.iter().map(|g| format!("{}.into()", lit(g))).collect::<Vec<_>>().join(", "));
    let _ = writeln!(w, "        css: {}.into(),", lit(&r.css));
    let _ = writeln!(w, "        bands: vec![");
    for b in &r.bands {
        let _ = writeln!(w, "            Band {{");
        let _ = writeln!(w, "                kind: BandKind::{:?},", b.kind);
        let _ = writeln!(w, "                group: {},", b.group);
        let _ = writeln!(w, "                height: {},", b.height);
        let _ = writeln!(w, "                new_page: {},", b.new_page);
        let _ = writeln!(w, "                items: vec![");
        for i in &b.items {
            let mut parts = vec![format!("kind: ItemKind::{:?}", i.kind), format!("bounds: {:?}", i.bounds)];
            let mut text = |name: &str, v: &str| {
                if !v.is_empty() {
                    parts.push(format!("{name}: {}.into()", lit(v)));
                }
            };
            text("name", &i.name);
            text("text", &i.text);
            text("field", &i.field);
            text("format", &i.format);
            text("font", &i.font);
            text("color", &i.color);
            text("background", &i.background);
            text("class", &i.class);
            text("image", &i.image);
            if i.kind == ItemKind::Total {
                parts.push(format!("func: Func::{:?}", i.func));
            }
            if i.align != Align::Left {
                parts.push(format!("align: Align::{:?}", i.align));
            }
            if i.size > 0 {
                parts.push(format!("size: {}", i.size));
            }
            if let Some(b) = i.bold {
                parts.push(format!("bold: Some({b})"));
            }
            if i.italic {
                parts.push("italic: true".into());
            }
            if i.border > 0 {
                parts.push(format!("border: {}", i.border));
            }
            let _ = writeln!(w, "                    Item {{ {}, ..Default::default() }},", parts.join(", "));
        }
        let _ = writeln!(w, "                ],");
        let _ = writeln!(w, "            }},");
    }
    let _ = writeln!(w, "        ],");
    let _ = writeln!(w, "    }}");
    let _ = writeln!(w, "}}");
    let _ = writeln!(w);
    let d = &f.data;
    if d.is_bound() {
        let _ = writeln!(w, "/// The rows, read from the project's database (`src/data_sources.rs`).");
        let _ = writeln!(w, "pub fn data() -> Result<Data, String> {{");
        let _ = writeln!(w, "    let c = crate::data_sources::connection();");
        if d.sql.is_empty() {
            let cols = if d.columns.is_empty() { f.used_columns() } else { d.columns.clone() };
            let list = |v: &[String]| v.iter().map(|s| lit(s)).collect::<Vec<_>>().join(", ");
            let _ = writeln!(w, "    let sql = c.select_sql({}, {}, &[{}], {}, &[{}]);", lit(&d.database), lit(&d.table), list(&cols), lit(&d.filter), list(&f.sort()));
        } else {
            let _ = writeln!(w, "    let sql = {}.to_string();", lit(&d.sql));
        }
        let db = if d.database.is_empty() { "None".to_string() } else { format!("Some({})", lit(&d.database)) };
        let _ = writeln!(w, "    let grid = c.query_all({db}, &sql)?;");
        let _ = writeln!(w, "    Ok(Data::new(grid.columns, grid.rows))");
        let _ = writeln!(w, "}}");
    } else {
        let _ = writeln!(w, "/// The rows: none - the report is not bound to a table. Lay out rows of your own with");
        let _ = writeln!(w, "/// `report().preview(hwnd, Data::new(columns, rows))`.");
        let _ = writeln!(w, "pub fn data() -> Result<Data, String> {{");
        let _ = writeln!(w, "    Ok(Data::default())");
        let _ = writeln!(w, "}}");
    }
    let _ = writeln!(w);
    let _ = writeln!(w, "/// The pages in a print preview window over `owner`; waits until it closes.");
    let _ = writeln!(w, "pub fn preview(owner: HWND) -> Result<(), String> {{");
    let _ = writeln!(w, "    report().preview(owner, data()?);");
    let _ = writeln!(w, "    Ok(())");
    let _ = writeln!(w, "}}");
    let _ = writeln!(w);
    let _ = writeln!(w, "/// Windows' Print dialog, then the pages printed: how many (0 when cancelled).");
    let _ = writeln!(w, "pub fn print(owner: HWND) -> Result<usize, String> {{");
    let _ = writeln!(w, "    Ok(report().print(owner, data()?))");
    let _ = writeln!(w, "}}");
    let _ = writeln!(w);
    let _ = writeln!(w, "/// Saved as a PDF at `path` (Microsoft Print to PDF): how many pages.");
    let _ = writeln!(w, "pub fn export_pdf(owner: HWND, path: &std::path::Path) -> Result<usize, String> {{");
    let _ = writeln!(w, "    Ok(report().export_pdf(owner, data()?, path))");
    let _ = writeln!(w, "}}");
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    const SALES: &str = r##"
format = 1

[report]
name = "SalesReport"
title = "Sales by region"
groups = ["region"]

[data]
database = "main"
table = "orders"
columns = ["region", "customer", "amount"]
order = ["customer"]

[[band]]
kind = "group_header"
height = 24
[[band.item]]
kind = "field"
name = "region"
bounds = [0, 2, 300, 20]
field = "region"

[[band]]
kind = "detail"
height = 20
[[band.item]]
kind = "field"
name = "customer"
bounds = [20, 0, 300, 18]
field = "customer"
[[band.item]]
kind = "field"
name = "amount"
bounds = [500, 0, 200, 18]
field = "amount"
format = "#,##0.00"
align = "right"

[[band]]
kind = "group_footer"
height = 24
[[band.item]]
kind = "total"
name = "region_total"
bounds = [500, 2, 200, 18]
field = "amount"
func = "sum"
format = "#,##0.00"
align = "right"
"##;

    #[test]
    fn a_report_reads_writes_and_reads_again() {
        let f = ReportFile::parse(SALES).unwrap();
        assert_eq!(f.bands.len(), 3);
        assert_eq!(f.module(), "sales_report");
        let again = ReportFile::parse(&f.to_toml()).unwrap();
        assert_eq!(again, f);
        let r = f.to_report();
        assert_eq!(r.bands[1].items[1].align, Align::Right);
        assert_eq!(r.bands[2].items[0].func, Func::Sum);
        let data = rp::Data::new(vec!["region".into(), "customer".into(), "amount".into()], vec![vec!["N".into(), "A".into(), "1.5".into()], vec!["N".into(), "B".into(), "2".into()]]);
        let pages = r.layout(&data, phai_ui::print::Bounds { x: 0, y: 0, width: 700, height: 900 }, &rp::Now::default());
        assert_eq!(pages[0].texts(), ["N", "A", "1.50", "B", "2.00", "3.50"]);
    }

    #[test]
    fn mistakes_are_told_with_their_line() {
        let bad = SALES.replace("func = \"sum\"", "func = \"total\"");
        let e = ReportFile::parse(&bad).unwrap_err();
        assert!(e.message.contains("sum, count"), "{e}");
        assert_eq!(bad.lines().nth(e.line - 1), Some("name = \"region_total\""), "found by the item's name");
        let e = ReportFile::parse(&SALES.replace("field = \"customer\"", "field = \"client\"")).unwrap_err();
        assert!(e.message.contains("client"), "{e}");
        let e = ReportFile::parse(&SALES.replace("groups = [\"region\"]", "")).unwrap_err();
        assert!(e.message.contains("group"), "{e}");
        let e = ReportFile::parse(&SALES.replace("kind = \"detail\"", "kind = \"body\"")).unwrap_err();
        assert!(e.message.contains("body"), "{e}");
        assert!(e.line > 0);
        let e = ReportFile::parse(&SALES.replace("[data]", "[report.data]")).unwrap_err();
        assert!(e.line > 0, "{e}");
    }

    #[test]
    fn a_new_report_is_good_and_generates_its_code() {
        let f = ReportFile::new("InvoiceReport");
        assert!(f.check().is_empty(), "{:?}", f.check());
        let code = generate(&f, "invoice_report.phaireport");
        assert!(code.contains("pub fn report() -> Report"), "{code}");
        assert!(code.contains("Ok(Data::default())"), "not bound: no database");
        assert!(!code.contains("data_sources"));
        let bound = ReportFile::parse(SALES).unwrap();
        let code = generate(&bound, "sales_report.phaireport");
        assert!(code.contains(r#"c.select_sql("main", "orders", &["region", "customer", "amount"], "", &["region", "customer"])"#), "{code}");
        assert!(code.contains("c.query_all(Some(\"main\"), &sql)?"), "{code}");
        assert!(code.contains("func: Func::Sum"), "{code}");
        assert!(code.contains("align: Align::Right"), "{code}");
        assert!(code.contains("pub fn export_pdf(owner: HWND, path: &std::path::Path)"));
    }
}
