//! The program's words in Thai. The English is the key. The test fails on a sentence the program
//! shows with no Thai, so none is forgotten.

pub static THAI: &[(&str, &str)] = &[
    ("Catch the Ball", "จับลูกบอล"),
    ("Score", "คะแนน"),
    ("Time left", "เวลาที่เหลือ"),
    ("Misses", "พลาด"),
    ("Start", "เริ่ม"),
    ("Stop", "หยุด"),
    ("Press Start, then click the ball.", "กดเริ่ม แล้วคลิกที่ลูกบอล"),
    ("Click the ball!", "คลิกที่ลูกบอล!"),
    ("Caught it! {0} so far.", "จับได้! ได้ {0} ครั้งแล้ว"),
    ("Missed - try again!", "พลาด - ลองอีกที!"),
    ("Game over: {0} caught, {1} missed. Press Start to play again.", "จบเกม: จับได้ {0} พลาด {1} กดเริ่มเพื่อเล่นอีกครั้ง"),
    ("Game over! You caught the ball {0} times.\n\nPress Start to play again.", "จบเกม! จับลูกบอลได้ {0} ครั้ง\n\nกดเริ่มเพื่อเล่นอีกครั้ง"),
    ("Click the ball as many times as you can in 30 seconds.\n\nPress Start to begin.", "คลิกลูกบอลให้ได้มากที่สุดใน 30 วินาที\n\nกดเริ่มเพื่อเริ่มเล่น"),
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
