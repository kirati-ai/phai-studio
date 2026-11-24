//! The program's words in Thai. The English is the key. The test fails on a sentence the program
//! shows with no Thai, so none is forgotten.

pub static THAI: &[(&str, &str)] = &[
    // The window (main_window.phf, main_window.rs).
    ("Sales report", "รายงานยอดขาย"),
    ("Print Preview", "ดูก่อนพิมพ์"),
    ("Print...", "พิมพ์..."),
    ("Export PDF...", "บันทึกเป็น PDF..."),
    ("The report prints the rows below, by branch. Change a row and preview again.", "รายงานพิมพ์แถวด้านล่างแยกตามสาขา ลองแก้แถวแล้วกดดูก่อนพิมพ์อีกครั้ง"),
    ("Date", "วันที่"),
    ("Branch", "สาขา"),
    ("Product", "สินค้า"),
    ("Qty", "จำนวน"),
    ("Price", "ราคา"),
    ("Amount", "จำนวนเงิน"),
    ("Ready", "พร้อม"),
    ("Printed {0} pages", "พิมพ์แล้ว {0} หน้า"),
    ("PDF files|*.pdf", "ไฟล์ PDF|*.pdf"),
    ("The PDF could not be made: this Windows has no \"Microsoft Print to PDF\" printer.", "ทำ PDF ไม่ได้ Windows เครื่องนี้ไม่มีเครื่องพิมพ์ \"Microsoft Print to PDF\""),
    ("Saved {0} pages to {1}", "บันทึก {0} หน้าที่ {1}"),
    // The report (sales_report.rs).
    ("Printed {date} at {time}", "พิมพ์วันที่ {date} เวลา {time}"),
    ("Total of {branch}", "รวม {branch}"),
    ("Grand total", "รวมทั้งหมด"),
    ("Page {page} of {pages}", "หน้า {page} จาก {pages}"),
];

/// Thai when started with `--thai`, else English.
pub fn load() {
    phai_ui::text::add(THAI);
    phai_ui::text::set_thai(std::env::args().any(|a| a == "--thai"));
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_sentence_has_thai() {
        phai_ui::text::add(super::THAI);
        let mut used: Vec<String> = crate::main_window_ui::TEXTS.iter().map(|s| s.to_string()).collect();
        used.extend(phai_ui::text::scan(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src")), &["main_window_ui.rs"]));
        assert_eq!(phai_ui::text::problems(&used), Vec::<String>::new());
    }
}
