//! Words in English and Thai. From PhaiDesk's console (`console/text.rs`), made into a library:
//! **the English text is the key**, so code reads as what it says and an untranslated sentence
//! shows in English rather than as a key. `{0}`, `{1}`... are placeholders.
//!
//! A program registers its own tables with [`add`]; the library's few words (OK, Cancel) are
//! there already. Where the choice is remembered is the program's business - this only switches.
//!
//! [`scan`] reads every `t("...")` and `tf("...")` in a folder of source, so a program's test can
//! fail on a sentence with no Thai, as the console's does. See this module's own test.

use std::fmt::Display;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;

static THAI: AtomicBool = AtomicBool::new(false);
static TABLES: RwLock<Vec<&'static [(&'static str, &'static str)]>> = RwLock::new(Vec::new());

/// The library's own words.
pub const LIBRARY: &[(&str, &str)] = &[
    ("OK", "ตกลง"),
    ("Cancel", "ยกเลิก"),
    ("Yes", "ใช่"),
    ("No", "ไม่"),
    // The Live Inspector (`inspect`).
    ("Live Inspector - {0}", "ตัวตรวจสด - {0}"),
    // Printing (`print`).
    ("Print preview - {0}", "ตัวอย่างก่อนพิมพ์ - {0}"),
    ("Print...", "พิมพ์..."),
    ("Close", "ปิด"),
    ("Page {0} of {1}", "หน้า {0} จาก {1}"),
    ("Pick", "ชี้เลือก"),
    ("Picking...", "กำลังชี้..."),
    ("Point at a control in the window and click it. Esc stops.", "ชี้ control ในหน้าต่างแล้วคลิก - Esc เลิก"),
    ("State", "สถานะ"),
    ("normal", "ปกติ"),
    ("own", "ของตัวเอง"),
    ("Text", "ข้อความ"),
    ("Enabled", "ใช้งานได้"),
    ("Visible", "มองเห็น"),
    ("Property", "คุณสมบัติ"),
    ("Set", "ตั้ง"),
    ("Undo all", "ย้อนทั้งหมด"),
    ("Save to form", "เก็บลงฟอร์ม"),
    ("Changes show in the program at once. Save writes them into the form.", "สิ่งที่แก้เห็นในโปรแกรมทันที กดเก็บเพื่อเขียนลงฟอร์ม"),
    ("{0} changed, not saved yet", "แก้แล้ว {0} อย่าง ยังไม่ได้เก็บ"),
    ("Nothing to save in {0}.", "ไม่มีอะไรต้องเก็บใน {0}"),
    ("Saved {0} lines into {1}.", "เก็บ {0} บรรทัดลงใน {1} แล้ว"),
    ("The form has no control {0}.", "ฟอร์มไม่มี control ชื่อ {0}"),
    ("{0} is not a property the inspector knows.", "{0} ไม่ใช่คุณสมบัติที่ตัวตรวจรู้จัก"),
    ("{0}: not a value yet", "{0}: ยังไม่ใช่ค่าที่ใช้ได้"),
    ("a whole number", "จำนวนเต็ม"),
];

pub fn is_thai() -> bool {
    THAI.load(Ordering::SeqCst)
}

/// Switches. Windows relabel themselves when they are told to (a program's `relabel`).
pub fn set_thai(thai: bool) {
    THAI.store(thai, Ordering::SeqCst);
}

/// Adds a program's table of `(English, Thai)`. Later tables are looked in first.
pub fn add(table: &'static [(&'static str, &'static str)]) {
    let mut t = TABLES.write().unwrap_or_else(|e| e.into_inner());
    if !t.iter().any(|x| std::ptr::eq(*x, table)) {
        t.push(table);
    }
}

/// `en` in the chosen language.
pub fn t(en: &'static str) -> &'static str {
    if !is_thai() {
        return en;
    }
    lookup(en).unwrap_or(en)
}

/// `en` in the chosen language with `{0}`, `{1}`... filled in.
pub fn tf(en: &'static str, args: &[&dyn Display]) -> String {
    fill(t(en), args)
}

pub fn fill(template: &str, args: &[&dyn Display]) -> String {
    let mut s = template.to_string();
    for (i, a) in args.iter().enumerate() {
        s = s.replace(&format!("{{{i}}}"), &a.to_string());
    }
    s
}

/// The Thai for `en`, from the program's tables and then the library's.
pub fn lookup(en: &str) -> Option<&'static str> {
    let tables = TABLES.read().unwrap_or_else(|e| e.into_inner());
    tables.iter().rev().chain(std::iter::once(&LIBRARY)).find_map(|t| t.iter().find(|(k, _)| *k == en).map(|(_, v)| *v))
}

/// Every `t("...")` and `tf("...")` literal in the `.rs` files of `dir` (not its subfolders),
/// skipping files named in `skip`. A literal must sit on one line, as the first argument.
pub fn scan(dir: &std::path::Path, skip: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return out };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "rs") || path.file_name().is_some_and(|n| skip.iter().any(|s| n == *s)) {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(&path) else { continue };
        for opener in ["t(\"", "tf(\""] {
            let mut rest = src.as_str();
            while let Some(i) = rest.find(opener) {
                let before = rest[..i].chars().last();
                rest = &rest[i + opener.len()..];
                if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                    continue; // another function whose name ends in t
                }
                let mut s = String::new();
                let mut chars = rest.chars();
                while let Some(c) = chars.next() {
                    match c {
                        '\\' => match chars.next() {
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some(o) => s.push(o),
                            None => {}
                        },
                        '"' => break,
                        c => s.push(c),
                    }
                }
                out.push(s);
            }
        }
    }
    out
}

/// What is wrong with the Thai for `used`: a sentence with none, an entry left in English, or a
/// placeholder lost. Empty when all is well - assert on it in a program's test.
pub fn problems(used: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for en in used {
        let Some(th) = lookup(en) else {
            out.push(format!("no Thai for {en:?}"));
            continue;
        };
        if th == en {
            out.push(format!("{en:?} is not translated"));
        }
        for i in 0..10 {
            let p = format!("{{{i}}}");
            if en.contains(&p) != th.contains(&p) {
                out.push(format!("{p} in {en:?} vs {th:?}"));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sentence_in_the_library_has_thai() {
        // phai-ui's sources - also when phai-web, which shares this file, runs the test.
        let here = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
        let ui = here.join("..").join("..").join("phai-ui").join("src");
        let dir = if here.join("dialogs.rs").is_file() { here.to_path_buf() } else { ui };
        let used = scan(&dir, &["text.rs"]);
        assert!(used.iter().any(|s| s == "OK"), "the scanner should see dialogs.rs: {used:?}");
        assert_eq!(problems(&used), Vec::<String>::new());
    }

    #[test]
    fn placeholders_fill_and_unknown_sentences_stay_english() {
        static T: &[(&str, &str)] = &[("{0} files, {1} folders", "{0} ไฟล์ {1} โฟลเดอร์")];
        add(T);
        assert_eq!(fill(lookup("{0} files, {1} folders").unwrap(), &[&3, &2]), "3 ไฟล์ 2 โฟลเดอร์");
        assert_eq!(lookup("Nobody translated this"), None);
        assert_eq!(problems(&["Nobody translated this".to_string()]).len(), 1);
    }
}
