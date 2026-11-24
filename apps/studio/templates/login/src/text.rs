//! The program's words in Thai. The English is the key. The test fails on a sentence the program
//! shows with no Thai, so none is forgotten.

pub static THAI: &[(&str, &str)] = &[
    // The sign-in form (sign_in.phf, sign_in.rs).
    ("Sign in", "เข้าสู่ระบบ"),
    ("Please sign in to continue.", "กรุณาเข้าสู่ระบบก่อนใช้งาน"),
    ("User name:", "ชื่อผู้ใช้:"),
    ("Password:", "รหัสผ่าน:"),
    ("Try admin and 1234.", "ลองใช้ admin กับ 1234"),
    ("Cancel", "ยกเลิก"),
    ("The user name or password is wrong.", "ชื่อผู้ใช้หรือรหัสผ่านไม่ถูกต้อง"),
    // The main window (main_window.phf, main_window.rs).
    ("My program", "โปรแกรมของฉัน"),
    ("&File", "&ไฟล์"),
    ("Sign out", "ออกจากระบบ"),
    ("Exit", "ปิดโปรแกรม"),
    ("Welcome", "ยินดีต้อนรับ"),
    ("Put the program's work here. File > Sign out goes back to the sign-in form.", "ใส่งานของโปรแกรมไว้ที่นี่ ไฟล์ > ออกจากระบบ จะกลับไปหน้าเข้าสู่ระบบ"),
    ("Ready", "พร้อม"),
    ("Welcome, {0}", "ยินดีต้อนรับ {0}"),
    ("Signed in at {0}", "เข้าสู่ระบบเมื่อ {0}"),
    ("Signed in as {0}", "ผู้ใช้: {0}"),
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
        used.extend(crate::sign_in_ui::TEXTS.iter().map(|s| s.to_string()));
        used.extend(phai_ui::text::scan(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src")), &["main_window_ui.rs", "sign_in_ui.rs"]));
        assert_eq!(phai_ui::text::problems(&used), Vec::<String>::new());
    }
}
