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
    ("Phai Tray", "ไผ่ในถาด"),
    ("Phai Tray lives in the corner of the taskbar. Right-click its icon for its menu.", "ไผ่ในถาดอยู่ที่มุมของแถบงาน คลิกขวาที่ไอคอนเพื่อเปิดเมนู"),
    ("Hide", "ซ่อน"),
    ("Open Phai Tray", "เปิดไผ่ในถาด"),
    ("Pretend a remote session", "ทำเป็นว่ามีการเชื่อมต่อระยะไกล"),
    ("End remote session", "จบการเชื่อมต่อระยะไกล"),
    ("Language", "ภาษา"),
    ("Thai", "ไทย"),
    ("Show a notice", "แสดงข้อความแจ้ง"),
    ("Click this notice to open the window.", "คลิกข้อความนี้เพื่อเปิดหน้าต่าง"),
];

/// Words the same in Thai: a language's own name.
pub static SAME: &[&str] = &["English"];

/// Thai when asked (`--thai`), else English.
pub fn load() {
    phai_ui::text::add(THAI);
    phai_ui::text::keep(SAME);
    phai_ui::text::set_thai(std::env::args().any(|a| a == "--thai"));
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_sentence_has_thai() {
        phai_ui::text::add(super::THAI);
        phai_ui::text::keep(super::SAME);
        let mut used: Vec<String> = crate::main_window_ui::TEXTS.iter().chain(crate::tray_window_ui::TEXTS).map(|s| s.to_string()).collect();
        used.extend(phai_ui::text::scan(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src")), &["main_window_ui.rs", "tray_window_ui.rs"]));
        assert_eq!(phai_ui::text::problems(&used), Vec::<String>::new());
    }
}
