//! The program's words in Thai. The English is the key. The test fails on a sentence the program
//! shows with no Thai, so none is forgotten.

pub static THAI: &[(&str, &str)] = &[
    ("Explorer", "สำรวจไฟล์"),
    ("&File", "&ไฟล์"),
    ("Open", "เปิด"),
    ("Refresh", "โหลดใหม่"),
    ("Exit", "ออก"),
    ("&View", "&มุมมอง"),
    ("Up one level", "ขึ้นไปหนึ่งระดับ"),
    ("Name", "ชื่อ"),
    ("Size", "ขนาด"),
    ("Date modified", "วันที่แก้ไข"),
    ("Ready", "พร้อม"),
    ("Documents", "เอกสาร"),
    ("Program folder", "โฟลเดอร์ของโปรแกรม"),
    ("{0} items", "{0} รายการ"),
    ("{0} - Explorer", "{0} - สำรวจไฟล์"),
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
