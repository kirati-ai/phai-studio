//! The program's words in Thai. The English is the key. The test fails on a sentence the program
//! shows with no Thai, so none is forgotten.

pub static THAI: &[(&str, &str)] = &[
    ("Setup wizard", "ตัวช่วยติดตั้ง"),
    ("Welcome", "ยินดีต้อนรับ"),
    ("This wizard takes you through a few short steps.", "ตัวช่วยนี้จะพาคุณทำไปทีละขั้น ไม่กี่ขั้นก็เสร็จ"),
    ("Options", "ตัวเลือก"),
    ("Details", "รายละเอียด"),
    ("Summary", "สรุป"),
    ("Click Next to begin.", "กด ถัดไป เพื่อเริ่ม"),
    ("You can go Back at any step to change an answer.", "กด ย้อนกลับ เพื่อแก้คำตอบได้ทุกขั้น"),
    ("Nothing is kept until you click Finish.", "ยังไม่มีอะไรถูกเก็บจนกว่าจะกด เสร็จสิ้น"),
    ("Question", "คำถาม"),
    ("Answer", "คำตอบ"),
    ("Who will use it?", "ใครจะใช้โปรแกรมนี้?"),
    ("Just me", "ฉันคนเดียว"),
    ("My team at work", "ทีมของฉันที่ทำงาน"),
    ("A class at school", "ห้องเรียนที่โรงเรียน"),
    ("Put a shortcut on the desktop", "สร้างทางลัดบนเดสก์ท็อป"),
    ("Look for updates by itself", "ตรวจหารุ่นใหม่เอง"),
    ("Your name", "ชื่อของคุณ"),
    ("E-mail", "อีเมล"),
    ("Both are needed to go on.", "ต้องกรอกทั้งสองช่องจึงจะไปต่อได้"),
    ("< Back", "< ย้อนกลับ"),
    ("Next >", "ถัดไป >"),
    ("Finish", "เสร็จสิ้น"),
    ("Cancel", "ยกเลิก"),
    ("Choose options", "เลือกตัวเลือก"),
    ("Choose how the program will be used.", "เลือกว่าจะใช้โปรแกรมอย่างไร"),
    ("Enter your details", "กรอกรายละเอียด"),
    ("Who the program is for.", "โปรแกรมนี้สำหรับใคร"),
    ("Ready to finish", "พร้อมเสร็จสิ้น"),
    ("Check your answers, then click Finish.", "ตรวจคำตอบ แล้วกด เสร็จสิ้น"),
    ("Step {0} of {1}", "ขั้นที่ {0} จาก {1}"),
    ("All done. Your answers:", "เสร็จแล้ว คำตอบของคุณ:"),
    ("Leave the wizard? Your answers will not be kept.", "ออกจากตัวช่วยไหม? คำตอบจะไม่ถูกเก็บไว้"),
    ("Yes", "ใช่"),
    ("No", "ไม่"),
    ("Used by", "ผู้ใช้"),
    ("Shortcut on the desktop", "ทางลัดบนเดสก์ท็อป"),
    ("Look for updates", "ตรวจหารุ่นใหม่"),
    ("Name", "ชื่อ"),
];

/// Words the same in Thai: the example in the e-mail box.
pub static SAME: &[&str] = &["name@example.com"];

/// Thai when started with `--thai`, else English.
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
        let mut used: Vec<String> = crate::main_window_ui::TEXTS.iter().map(|s| s.to_string()).collect();
        used.extend(phai_ui::text::scan(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src")), &["main_window_ui.rs"]));
        assert_eq!(phai_ui::text::problems(&used), Vec::<String>::new());
    }
}
