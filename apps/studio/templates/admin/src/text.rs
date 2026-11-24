//! The program's words in Thai. The English is the key. The test fails on a sentence the program
//! shows with no Thai, so none is forgotten.

pub static THAI: &[(&str, &str)] = &[
    // The menu and the header.
    ("Back office", "ระบบหลังร้าน"),
    ("MENU", "เมนู"),
    ("Dashboard", "ภาพรวม"),
    ("Users", "ผู้ใช้"),
    ("Orders", "คำสั่งซื้อ"),
    ("Settings", "ตั้งค่า"),
    ("Sign out", "ออกจากระบบ"),
    ("Search users and orders...", "ค้นหาผู้ใช้และคำสั่งซื้อ..."),
    ("Administrator", "ผู้ดูแลระบบ"),
    ("Account", "บัญชี"),
    ("Signed in as {0} ({1}).", "เข้าระบบในชื่อ {0} ({1})"),
    ("Sign out and close the back office?", "ออกจากระบบและปิดระบบหลังร้านไหม?"),
    // Dashboard.
    ("Welcome back, Somchai", "ยินดีต้อนรับกลับมา สมชาย"),
    ("Here is how the shop is doing today.", "ภาพรวมของร้านวันนี้"),
    ("Sales today", "ยอดขายวันนี้"),
    ("Orders today", "คำสั่งซื้อวันนี้"),
    ("Customers", "ลูกค้า"),
    ("Low in stock", "สินค้าใกล้หมด"),
    ("▲ {0}% on yesterday", "▲ {0}% จากเมื่อวาน"),
    ("{0} waiting for payment", "รอชำระเงิน {0} รายการ"),
    ("▲ {0} new this week", "▲ ใหม่ {0} คนในสัปดาห์นี้"),
    ("Order more soon", "ควรสั่งของเพิ่มเร็ว ๆ นี้"),
    ("Recent orders", "คำสั่งซื้อล่าสุด"),
    ("Goals this month", "เป้าหมายเดือนนี้"),
    ("Sales target", "เป้ายอดขาย"),
    ("New customers", "ลูกค้าใหม่"),
    ("Orders shipped on time", "ส่งของตรงเวลา"),
    ("Needs attention", "เรื่องที่ต้องดู"),
    ("• {0} products low in stock", "• สินค้าใกล้หมด {0} รายการ"),
    ("• {0} orders waiting over 2 days", "• คำสั่งซื้อรอเกิน 2 วัน {0} รายการ"),
    ("• {0} refund to approve", "• คืนเงินรออนุมัติ {0} รายการ"),
    // Orders.
    ("Order", "เลขที่"),
    ("Date", "วันที่"),
    ("Time", "เวลา"),
    ("Customer", "ลูกค้า"),
    ("Items", "ชิ้น"),
    ("Total", "ยอดรวม"),
    ("Status", "สถานะ"),
    ("Paid", "ชำระแล้ว"),
    ("Pending", "รอชำระ"),
    ("Shipped", "ส่งแล้ว"),
    ("Cancelled", "ยกเลิก"),
    ("All orders", "ทุกคำสั่งซื้อ"),
    ("Show", "แสดง"),
    ("Every order of the last few days. Click a heading to sort.", "คำสั่งซื้อในช่วงไม่กี่วันนี้ คลิกหัวคอลัมน์เพื่อเรียง"),
    ("{0} orders shown", "แสดง {0} คำสั่งซื้อ"),
    // Users.
    ("Everyone who can sign in to the back office.", "ทุกคนที่เข้าระบบหลังร้านได้"),
    ("Add user", "เพิ่มผู้ใช้"),
    ("Edit user", "แก้ไขผู้ใช้"),
    ("Edit", "แก้ไข"),
    ("Delete", "ลบ"),
    ("Delete user", "ลบผู้ใช้"),
    ("Delete {0}? They will no longer be able to sign in.", "ลบ {0} ไหม? คนนี้จะเข้าระบบไม่ได้อีก"),
    ("Choose a user in the list first.", "เลือกผู้ใช้ในรายการก่อน"),
    ("ID", "รหัส"),
    ("Name", "ชื่อ"),
    ("Email", "อีเมล"),
    ("Role", "บทบาท"),
    ("Last sign-in", "เข้าระบบล่าสุด"),
    ("Manager", "ผู้จัดการ"),
    ("Staff", "พนักงาน"),
    ("Active", "ใช้งาน"),
    ("Suspended", "ระงับ"),
    ("Never", "ยังไม่เคย"),
    ("{0} users shown", "แสดงผู้ใช้ {0} คน"),
    ("User", "ผู้ใช้"),
    ("May sign in", "เข้าระบบได้"),
    ("Save", "บันทึก"),
    ("Cancel", "ยกเลิก"),
    // Settings.
    ("How the back office works for your shop.", "การทำงานของระบบหลังร้านสำหรับร้านของคุณ"),
    ("Shop name", "ชื่อร้าน"),
    ("Send order emails to", "ส่งอีเมลคำสั่งซื้อไปที่"),
    ("Currency", "สกุลเงิน"),
    ("Thai baht (฿)", "บาท (฿)"),
    ("US dollar ($)", "ดอลลาร์สหรัฐ ($)"),
    ("Euro (€)", "ยูโร (€)"),
    ("Low stock below", "แจ้งสินค้าใกล้หมดเมื่อต่ำกว่า"),
    ("Email me a summary every morning", "ส่งอีเมลสรุปให้ทุกเช้า"),
    ("Play a sound for each new order", "เล่นเสียงเมื่อมีคำสั่งซื้อใหม่"),
    ("Save changes", "บันทึกการเปลี่ยนแปลง"),
    ("Saved at {0}.", "บันทึกแล้วเมื่อ {0}"),
    ("The email address does not look right.", "ที่อยู่อีเมลดูไม่ถูกต้อง"),
];

/// Words the same in every language: the program's name, the signed-in person's name and initials.
pub static KEEP: &[&str] = &["Phai Admin", "Somchai K.", "SK"];

/// Thai when started with `--thai`, else English.
pub fn load() {
    phai_ui::text::add(THAI);
    phai_ui::text::keep(KEEP);
    phai_ui::text::set_thai(std::env::args().any(|a| a == "--thai"));
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_sentence_has_thai() {
        phai_ui::text::add(super::THAI);
        phai_ui::text::keep(super::KEEP);
        // Every form's words, and every t("...") and tf("...") in the code.
        let forms = [
            crate::main_window_ui::TEXTS,
            crate::dashboard_page_ui::TEXTS,
            crate::users_page_ui::TEXTS,
            crate::orders_page_ui::TEXTS,
            crate::settings_page_ui::TEXTS,
            crate::user_form_ui::TEXTS,
        ];
        let mut used: Vec<String> = forms.iter().flat_map(|f| f.iter()).map(|s| s.to_string()).collect();
        let generated = ["main_window_ui.rs", "dashboard_page_ui.rs", "users_page_ui.rs", "orders_page_ui.rs", "settings_page_ui.rs", "user_form_ui.rs"];
        used.extend(phai_ui::text::scan(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src")), &generated));
        assert_eq!(phai_ui::text::problems(&used), Vec::<String>::new());
    }
}
