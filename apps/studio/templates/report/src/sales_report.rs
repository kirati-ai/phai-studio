//! The sales report, in bands one under another as a report designer lays them out: the report
//! header once at the top, the column headings at the top of every page, each branch's rows with
//! a heading and a total, the grand total at the end and the page number at the bottom of every
//! page. Places and sizes are hundredths of an inch; the look (fonts, colours, the striped rows)
//! is phai-ui's report CSS, which `css` may change.

use phai_ui::print::Align;
use phai_ui::report::{Band, BandKind, Data, Func, Item, ItemKind, Report};
use phai_ui::text::t;

/// The rows' columns by name, in the grid's order: a field prints one (`field`), a label may
/// name one in braces (`{branch}`).
pub const COLUMNS: [&str; 6] = ["date", "branch", "product", "qty", "price", "amount"];

/// The printable width of a portrait page: 8.5 inches less half an inch each side.
const WIDTH: i32 = 750;

/// The report, its words in the language chosen.
pub fn report() -> Report {
    Report {
        title: t("Sales report").into(),
        // Rows of the same branch together, a heading before them and a total after.
        groups: vec!["branch".into()],
        bands: vec![
            band(BandKind::ReportHeader, 80, vec![
                Item { size: 18, ..label(t("Sales report"), [0, 0, WIDTH, 36]) },
                // Smaller and grey: the report header's CSS makes its words large and blue.
                Item { size: 10, bold: Some(false), color: "#6b7280".into(), ..label(t("Printed {date} at {time}"), [0, 42, WIDTH, 20]) },
            ]),
            band(BandKind::PageHeader, 28, vec![
                label(t("Date"), [0, 4, 90, 20]),
                label(t("Product"), [100, 4, 260, 20]),
                Item { align: Align::Right, ..label(t("Qty"), [360, 4, 60, 20]) },
                Item { align: Align::Right, ..label(t("Price"), [430, 4, 120, 20]) },
                Item { align: Align::Right, ..label(t("Amount"), [560, 4, 190, 20]) },
                line([0, 26, WIDTH, 0]),
            ]),
            band(BandKind::GroupHeader, 28, vec![label("{branch}", [0, 5, WIDTH, 20])]),
            band(BandKind::Detail, 20, vec![
                field("date", [0, 1, 90, 18], Align::Left, ""),
                field("product", [100, 1, 260, 18], Align::Left, ""),
                field("qty", [360, 1, 60, 18], Align::Right, "#,##0"),
                field("price", [430, 1, 120, 18], Align::Right, "#,##0.00"),
                field("amount", [560, 1, 190, 18], Align::Right, "#,##0.00"),
            ]),
            band(BandKind::GroupFooter, 30, vec![
                line([360, 2, WIDTH - 360, 0]),
                Item { align: Align::Right, ..label(t("Total of {branch}"), [100, 6, 450, 20]) },
                total("amount", [560, 6, 190, 20]),
            ]),
            band(BandKind::ReportFooter, 40, vec![
                line([0, 6, WIDTH, 0]),
                Item { size: 12, bold: Some(true), align: Align::Right, ..label(t("Grand total"), [100, 12, 450, 24]) },
                Item { size: 12, ..total("amount", [560, 12, 190, 24]) },
            ]),
            band(BandKind::PageFooter, 26, vec![
                label("{date}", [0, 6, 300, 18]),
                Item { align: Align::Right, ..label(t("Page {page} of {pages}"), [WIDTH - 300, 6, 300, 18]) },
            ]),
        ],
        ..Default::default()
    }
}

/// `rows` - each in the order of [`COLUMNS`] - as the report reads them.
pub fn data(rows: Vec<Vec<String>>) -> Data {
    Data::new(COLUMNS.iter().map(|c| c.to_string()).collect(), rows)
}

fn band(kind: BandKind, height: i32, items: Vec<Item>) -> Band {
    Band { kind, height, items, ..Default::default() }
}

/// Words; `{column}`, `{date}`, `{time}`, `{page}` and `{pages}` filled in when it prints.
fn label(text: &str, bounds: [i32; 4]) -> Item {
    Item { kind: ItemKind::Label, text: text.into(), bounds, ..Default::default() }
}

/// A row's value of `column`; a number written as `format` says (`#,##0.00`), when there is one.
fn field(column: &str, bounds: [i32; 4], align: Align, format: &str) -> Item {
    Item { kind: ItemKind::Field, field: column.into(), bounds, align, format: format.into(), ..Default::default() }
}

/// The sum of `column`: of the branch's rows in a group's footer, of every row in the report's.
fn total(column: &str, bounds: [i32; 4]) -> Item {
    Item { kind: ItemKind::Total, field: column.into(), func: Func::Sum, bounds, align: Align::Right, format: "#,##0.00".into(), ..Default::default() }
}

/// A line from the corner of `bounds` across it; no height makes it level.
fn line(bounds: [i32; 4]) -> Item {
    Item { kind: ItemKind::Line, bounds, ..Default::default() }
}

#[cfg(test)]
mod tests {
    use phai_ui::print::Bounds;
    use phai_ui::report::Now;

    /// The report laid out without printing: each branch's heading and total, and the grand total.
    #[test]
    fn branches_are_totalled() {
        let rows = vec![
            vec!["01/11/2026", "B", "Tea", "2", "50", "100"],
            vec!["02/11/2026", "A", "Rice", "1", "200", "200"],
            vec!["03/11/2026", "B", "Coffee", "3", "100", "300"],
        ];
        let rows = rows.into_iter().map(|r| r.into_iter().map(String::from).collect()).collect();
        let page = Bounds { x: 50, y: 50, width: 750, height: 1000 };
        let pages = super::report().layout(&super::data(rows), page, &Now::default());
        assert_eq!(pages.len(), 1);
        let texts = pages[0].texts();
        assert!(texts.contains(&"Total of A") && texts.contains(&"Total of B"), "{texts:?}");
        assert!(texts.contains(&"400.00") && texts.contains(&"600.00"), "{texts:?}");
    }
}
