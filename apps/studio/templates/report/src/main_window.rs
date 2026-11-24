//! MainWindow: sales in a grid, and the buttons that print them as a report (sales_report.rs) -
//! seen first in a print preview, sent to a printer, or saved as a PDF.
//! The report prints what the grid holds, so a row changed here prints changed. A program of
//! your own would read its rows from its database instead (phai-data's `query_all`).

use crate::main_window_ui::{MainWindowEvents, MainWindowUi};
use crate::sales_report;
use phai_ui::prelude::*;

/// Sample sales: date, branch, product, quantity, price. The amount is worked out.
const SALES: &[(&str, &str, &str, u32, f64)] = &[
    ("03/11/2026", "สำนักงานใหญ่", "ข้าวหอมมะลิ 5 กก.", 12, 245.0),
    ("03/11/2026", "สาขาเชียงใหม่", "กาแฟดอยช้าง 250 ก.", 8, 180.0),
    ("05/11/2026", "สำนักงานใหญ่", "น้ำปลาแท้ 700 มล.", 24, 42.0),
    ("06/11/2026", "สาขาขอนแก่น", "ข้าวเหนียว 2 กก.", 15, 96.0),
    ("08/11/2026", "สาขาเชียงใหม่", "ชาไทย 400 ก.", 10, 120.0),
    ("10/11/2026", "สาขาขอนแก่น", "ปลาร้าบอง 200 ก.", 30, 55.0),
    ("12/11/2026", "สำนักงานใหญ่", "กาแฟดอยช้าง 250 ก.", 6, 180.0),
    ("14/11/2026", "สาขาเชียงใหม่", "ข้าวหอมมะลิ 5 กก.", 9, 245.0),
    ("17/11/2026", "สาขาขอนแก่น", "น้ำปลาแท้ 700 มล.", 18, 42.0),
    ("19/11/2026", "สำนักงานใหญ่", "ชาไทย 400 ก.", 14, 120.0),
    ("21/11/2026", "สาขาเชียงใหม่", "น้ำผึ้งป่า 500 ก.", 5, 260.0),
    ("24/11/2026", "สาขาขอนแก่น", "ข้าวหอมมะลิ 5 กก.", 20, 245.0),
];

#[derive(Default)]
pub struct MainWindow;

impl MainWindowEvents for MainWindow {
    fn loaded(&mut self, ui: &MainWindowUi) {
        for (date, branch, product, qty, price) in SALES {
            let amount = *qty as f64 * price;
            ui.sales.rows_add(&[date, branch, product, &qty.to_string(), &price.to_string(), &amount.to_string()]);
        }
    }

    fn preview_clicked(&mut self, ui: &MainWindowUi) {
        // A window with the pages, its own Print button among them; waits until it closes.
        sales_report::report().preview(ui.hwnd, rows(ui));
    }

    fn print_clicked(&mut self, ui: &MainWindowUi) {
        // Windows' Print dialog first: which printer, which pages, how many copies.
        let pages = sales_report::report().print(ui.hwnd, rows(ui));
        if pages > 0 {
            ui.status.set_part_text(0, &tf("Printed {0} pages", &[&pages]));
        }
    }

    fn export_pdf_clicked(&mut self, ui: &MainWindowUi) {
        let Some(path) = ui.save_file(t("PDF files|*.pdf"), "sales.pdf") else { return };
        // Through Windows' own "Microsoft Print to PDF" printer.
        let pages = sales_report::report().export_pdf(ui.hwnd, rows(ui), &path);
        if pages == 0 {
            message_box(ui.hwnd, t("The PDF could not be made: this Windows has no \"Microsoft Print to PDF\" printer."), t("Sales report"), MessageBoxButtons::Ok, MessageBoxIcon::Warning);
            return;
        }
        ui.status.set_part_text(0, &tf("Saved {0} pages to {1}", &[&pages, &path.display()]));
        open_with_windows(&path.to_string_lossy());
    }
}

/// The grid's rows as the report reads them - the amount worked out again from what is typed.
fn rows(ui: &MainWindowUi) -> phai_ui::report::Data {
    let rows = ui
        .sales
        .rows()
        .into_iter()
        .map(|mut row| {
            row.resize(sales_report::COLUMNS.len(), String::new());
            row[5] = (to_float(&row[3]) * to_float(&row[4])).to_string();
            row
        })
        .collect();
    sales_report::data(rows)
}
