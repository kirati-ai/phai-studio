//! Phai Studio's words in Thai.

pub static THAI: &[(&str, &str)] = &[
    ("&File", "&ไฟล์"),
    ("&Edit", "แ&ก้ไข"),
    ("&View", "&มุมมอง"),
    ("&Theme", "&ธีม"),
    ("New", "ใหม่"),
    ("Open...", "เปิด..."),
    ("Save", "บันทึก"),
    ("Save as...", "บันทึกเป็น..."),
    ("Exit", "ออก"),
    ("Undo", "เลิกทำ"),
    ("Redo", "ทำซ้ำ"),
    ("Delete", "ลบ"),
    ("Preview", "ดูตัวอย่าง"),
    ("Light", "สว่าง"),
    ("Dark", "มืด"),
    ("New theme from the standard...", "สร้างธีมจากมาตรฐาน..."),
    ("Import CSS...", "นำเข้า CSS..."),
    ("No theme", "ไม่ใช้ธีม"),
    ("Open", "เปิด"),
    ("Language", "ภาษา"),
    ("Pointer", "ตัวชี้"),
    ("Label", "ป้าย"),
    ("Text box", "ช่องข้อความ"),
    ("Button", "ปุ่ม"),
    ("Check box", "ช่องติ๊ก"),
    ("Option", "ตัวเลือก"),
    ("Drop-down", "รายการเลือก"),
    ("List", "รายการ"),
    ("Group", "กลุ่ม"),
    ("Tabs", "แท็บ"),
    ("Status bar", "แถบสถานะ"),
    ("Form", "ฟอร์ม"),
    ("Name", "ชื่อ"),
    ("Title", "หัวเรื่อง"),
    ("Width", "กว้าง"),
    ("Height", "สูง"),
    ("Theme", "ธีม"),
    ("Id", "รหัส"),
    ("Text", "ข้อความ"),
    ("Anchor", "ยึดขอบ"),
    ("Dock", "ติดขอบ"),
    ("Class", "คลาส"),
    ("Event", "เหตุการณ์"),
    ("Items", "รายการ"),
    ("Left", "ซ้าย"),
    ("Top", "บน"),
    ("Right", "ขวา"),
    ("Bottom", "ล่าง"),
    ("Ready", "พร้อม"),
    ("Untitled", "ไม่มีชื่อ"),
    ("Saved {0}", "บันทึก {0} แล้ว"),
    ("Saved {0} and wrote {1}", "บันทึก {0} และเขียน {1} แล้ว"),
    ("Added {0} to {1}", "เพิ่ม {0} ใน {1} แล้ว"),
    ("Save the form first, so its theme can sit beside it.", "บันทึกฟอร์มก่อน เพื่อวางไฟล์ธีมไว้ข้างกัน"),
    ("Save changes to {0}?", "บันทึกการแก้ไขใน {0} ไหม"),
    ("{0} cannot be opened", "เปิด {0} ไม่ได้"),
    ("{0} cannot be saved", "บันทึก {0} ไม่ได้"),
    ("Theme: {0}", "ธีม: {0}"),
    ("Theme file has a mistake", "ไฟล์ธีมมีจุดผิด"),
    ("The form has a mistake", "ฟอร์มมีจุดผิด"),
];

#[cfg(test)]
mod tests {
    #[test]
    fn every_sentence_has_thai() {
        phai_ui::text::add(super::THAI);
        let used = phai_ui::text::scan(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src")), &["text.rs"]);
        assert!(used.len() > 40, "the scanner found only {}", used.len());
        assert_eq!(phai_ui::text::problems(&used), Vec::<String>::new());
    }
}
