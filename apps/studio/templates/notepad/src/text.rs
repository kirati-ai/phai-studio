//! The program's words in Thai. The English is the key. The test fails on a sentence the program
//! shows with no Thai, so none is forgotten.

pub static THAI: &[(&str, &str)] = &[
    ("Text editor", "โปรแกรมแก้ข้อความ"),
    ("&File", "&ไฟล์"),
    ("New", "ใหม่"),
    ("Open...", "เปิด..."),
    ("Save", "บันทึก"),
    ("Save As...", "บันทึกเป็น..."),
    ("Exit", "ออก"),
    ("&Edit", "แ&ก้ไข"),
    ("Undo", "เลิกทำ"),
    ("Cut", "ตัด"),
    ("Copy", "คัดลอก"),
    ("Paste", "วาง"),
    ("Select All", "เลือกทั้งหมด"),
    ("Find...", "ค้นหา..."),
    ("Find Next", "ค้นหาถัดไป"),
    ("F&ormat", "รูปแ&บบ"),
    ("Word Wrap", "ตัดบรรทัดอัตโนมัติ"),
    ("Font...", "แบบอักษร..."),
    ("Text files|*.txt|All files|*.*", "ไฟล์ข้อความ|*.txt|ทุกไฟล์|*.*"),
    ("Untitled", "ไม่มีชื่อ"),
    ("{0} - Text editor", "{0} - โปรแกรมแก้ข้อความ"),
    ("Find what:", "ค้นหาคำว่า:"),
    ("Cannot find \"{0}\"", "ไม่พบ \"{0}\""),
    ("Cannot open {0}: {1}", "เปิด {0} ไม่ได้: {1}"),
    ("Cannot save {0}: {1}", "บันทึก {0} ไม่ได้: {1}"),
    ("Save the changes to {0}?", "บันทึกการเปลี่ยนแปลงใน {0} ไหม?"),
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
