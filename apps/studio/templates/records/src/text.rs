//! The program's words in Thai. The English is the key. The test fails on a sentence the program
//! shows with no Thai, so none is forgotten.

pub static THAI: &[(&str, &str)] = &[
    // The window (main_window.phf).
    ("Customers", "ลูกค้า"),
    ("Search:", "ค้นหา:"),
    ("Name, phone or email", "ชื่อ เบอร์โทร หรืออีเมล"),
    ("ID", "รหัส"),
    ("Name", "ชื่อ"),
    ("Phone", "โทรศัพท์"),
    ("Email", "อีเมล"),
    ("Customer", "ข้อมูลลูกค้า"),
    ("Name:", "ชื่อ:"),
    ("Phone:", "โทรศัพท์:"),
    ("Email:", "อีเมล:"),
    ("Note:", "หมายเหตุ:"),
    ("New", "เพิ่ม"),
    ("Save", "บันทึก"),
    ("Delete", "ลบ"),
    ("Save writes every change to the file at once.", "ปุ่มบันทึกเขียนทุกอย่างที่แก้ลงไฟล์ในครั้งเดียว"),
    ("Ready", "พร้อม"),
    // The code (main_window.rs).
    ("{0} customers", "ลูกค้า {0} ราย"),
    ("Not saved yet", "ยังไม่ได้บันทึก"),
    ("Saved", "บันทึกแล้ว"),
    ("Type the customer's name.", "กรุณาพิมพ์ชื่อลูกค้า"),
    ("Delete {0}?", "ลบ {0} ไหม?"),
    ("Save the changes before closing?", "บันทึกสิ่งที่แก้ก่อนปิดไหม?"),
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
