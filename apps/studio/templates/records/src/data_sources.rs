//! Where this program's data is: a SQLite file beside the program, `customers.db`. The form's
//! binding source (`bind = { table = "customers", ... }` in main_window.phf) reads and saves its
//! rows through `connection()`. The file is made, with a few customers in it, the first time the
//! program runs - delete it to start again.

use phai_ui::prelude::app_folder;

/// The file's name, in the program's folder.
const FILE: &str = "customers.db";

/// The connection every bound control of the program uses.
pub fn connection() -> phai_data::Connection {
    // A program installed under Program Files may not write beside itself: keep the file in the
    // person's own folder instead, as
    //     std::env::var("APPDATA").map(std::path::PathBuf::from).unwrap_or_else(|_| app_folder()).join(FILE)
    phai_data::Connection { kind: phai_data::Kind::Sqlite, host: app_folder().join(FILE).to_string_lossy().into_owned(), ..Default::default() }
}

/// The table made when it is not there yet, and a few customers put in when it is empty.
pub fn prepare() -> Result<(), String> {
    let c = connection();
    // Empty text rather than NULL in the columns one may leave blank: the boxes show it as it is.
    c.query(
        None,
        "CREATE TABLE IF NOT EXISTS customers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            phone TEXT NOT NULL DEFAULT '',
            email TEXT NOT NULL DEFAULT '',
            note TEXT NOT NULL DEFAULT ''
        )",
    )?;
    let count = c.query(None, "SELECT COUNT(*) FROM customers")?;
    if count.rows.first().and_then(|r| r.first()).map(String::as_str) != Some("0") {
        return Ok(());
    }
    let samples = [
        ["สมชาย ใจดี", "081-234-5678", "somchai@example.com", "ลูกค้าประจำ"],
        ["มาลี ศรีสุข", "089-876-5432", "malee@example.com", ""],
        ["บริษัท ไผ่ทอง จำกัด", "02-123-4567", "contact@phaithong.example", "ส่งของทุกวันจันทร์"],
        ["นรินทร์ แก้วมณี", "086-555-0101", "narin@example.com", ""],
        ["Peter Brown", "090-111-2233", "peter@example.com", "Speaks English"],
        ["ร้านกาแฟบ้านสวน", "053-222-333", "baansuan@example.com", "เชียงใหม่"],
    ];
    for s in samples {
        // Each value given apart from the SQL (`?`), so no value can change what the statement does.
        c.execute(None, "INSERT INTO customers (name, phone, email, note) VALUES (?, ?, ?, ?)", &s.map(|v| Some(v.to_string())))?;
    }
    Ok(())
}
