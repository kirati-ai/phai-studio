//! The program's words in Thai. The English is the key. The test fails on a sentence the program
//! shows with no Thai, so none is forgotten.

pub static THAI: &[(&str, &str)] = &[
    ("My documents", "เอกสารของฉัน"),
    ("&File", "แ&ฟ้ม"),
    ("New", "สร้างใหม่"),
    ("Close", "ปิด"),
    ("Exit", "ออก"),
    ("&Window", "&หน้าต่าง"),
    ("Cascade", "เรียงซ้อนกัน"),
    ("Tile Horizontally", "เรียงบนลงล่าง"),
    ("Tile Vertically", "เรียงซ้ายไปขวา"),
    ("Arrange Icons", "จัดไอคอน"),
    ("&Help", "&วิธีใช้"),
    ("About", "เกี่ยวกับ"),
    ("No documents open", "ไม่มีเอกสารเปิดอยู่"),
    ("1 document open", "เปิดเอกสารอยู่ 1 ฉบับ"),
    ("{0} documents open", "เปิดเอกสารอยู่ {0} ฉบับ"),
    ("Document", "เอกสาร"),
    ("Document {0}", "เอกสาร {0}"),
    ("A program with documents in one window, made with Phai Studio.", "โปรแกรมที่เปิดเอกสารหลายฉบับในหน้าต่างเดียว สร้างด้วย Phai Studio"),
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
        let mut used: Vec<String> = Vec::new();
        for texts in [crate::main_window_ui::TEXTS, crate::document_window_ui::TEXTS] {
            used.extend(texts.iter().map(|s| s.to_string()));
        }
        let generated = ["main_window_ui.rs", "document_window_ui.rs"];
        used.extend(phai_ui::text::scan(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src")), &generated));
        assert_eq!(phai_ui::text::problems(&used), Vec::<String>::new());
    }
}
