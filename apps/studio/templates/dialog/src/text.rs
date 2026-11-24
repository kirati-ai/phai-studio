//! The program's words in Thai. The English is the key. The test fails on a sentence the program
//! shows with no Thai, so none is forgotten.

pub static THAI: &[(&str, &str)] = &[
    ("Unit Converter", "แปลงหน่วย"),
    ("Length", "ความยาว"),
    ("From", "จาก"),
    ("To", "เป็น"),
    ("Millimetres", "มิลลิเมตร"),
    ("Centimetres", "เซนติเมตร"),
    ("Metres", "เมตร"),
    ("Kilometres", "กิโลเมตร"),
    ("Inches", "นิ้ว"),
    ("Feet", "ฟุต"),
    ("Yards", "หลา"),
    ("Miles", "ไมล์"),
    ("Convert", "แปลง"),
    ("Type a length and press Convert.", "พิมพ์ความยาวแล้วกด แปลง"),
    ("Please type a number.", "กรุณาพิมพ์ตัวเลข"),
    ("Please choose both units.", "กรุณาเลือกหน่วยทั้งสองช่อง"),
    ("{0} {1} = {2} {3}", "{0} {1} = {2} {3}"),
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
