//! The sample's words in Thai. The test fails on a sentence the form shows with no Thai.

pub static THAI: &[(&str, &str)] = &[
    ("Customers", "ลูกค้า"),
    ("&File", "&ไฟล์"),
    ("New customer", "ลูกค้าใหม่"),
    ("Exit", "ออก"),
    ("Save", "บันทึก"),
    ("Customer", "ลูกค้า"),
    ("Name", "ชื่อ"),
    ("Active", "ใช้งานอยู่"),
    ("Shop", "ร้าน"),
    ("Office", "สำนักงาน"),
    ("Phone", "โทรศัพท์"),
    ("Close", "ปิด"),
    ("Ready", "พร้อม"),
    ("{0} customers", "ลูกค้า {0} ราย"),
    ("Saved {0}", "บันทึก {0} แล้ว"),
];

/// Thai when asked (`--thai`), else English.
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
