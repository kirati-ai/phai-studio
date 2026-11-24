//! The program's words in Thai. The English is the key. The test fails on a sentence the program
//! shows with no Thai, so none is forgotten.

pub static THAI: &[(&str, &str)] = &[
    ("My tray program", "โปรแกรมในถาดของฉัน"),
    ("This program lives in its icon by the clock.", "โปรแกรมนี้อยู่ในไอคอนข้างนาฬิกา"),
    ("Right-click the icon there for its menu.", "คลิกขวาที่ไอคอนนั้นเพื่อเปิดเมนู"),
    ("Does not start with Windows.", "ไม่เริ่มพร้อม Windows"),
    ("Starts with Windows when you sign in.", "เริ่มพร้อม Windows เมื่อเข้าสู่ระบบ"),
    ("Hide", "ซ่อน"),
    ("Open", "เปิด"),
    ("Start with Windows", "เริ่มพร้อม Windows"),
    ("Exit", "ออก"),
    ("Windows did not take the change.", "Windows ไม่ยอมรับการเปลี่ยนแปลง"),
    ("Still running here. Right-click the icon for its menu.", "ยังทำงานอยู่ตรงนี้ คลิกขวาที่ไอคอนเพื่อเปิดเมนู"),
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
