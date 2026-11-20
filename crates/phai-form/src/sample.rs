//! Design-time sample data, as Android Studio's `tools:` attributes and Xcode's previews give: the
//! designer shows a few made-up rows in a list or a grid, guessed from each column's name, so a
//! form looks as it will with data in it. Only the designer shows them; the program starts empty.

/// How many rows the designer shows.
pub const ROWS: usize = 3;

/// A made-up value for column `column`, row `row` (from 0): a name for Name, a number for Qty,
/// money for Price, a date for Date...; else the column's name and the row's number.
pub fn cell(column: &str, row: usize) -> String {
    let c = column.to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| c.contains(w));
    let pick = |v: &[&str]| v[row % v.len()].to_string();
    if has(&["email", "mail"]) {
        pick(&["somchai@example.com", "malee@example.com", "anan@example.com"])
    } else if has(&["phone", "tel", "mobile", "โทร"]) {
        pick(&["081-234-5678", "089-876-5432", "02-123-4567"])
    } else if has(&["date", "day", "when", "วันที่"]) {
        pick(&["2026-11-20", "2026-11-21", "2026-12-01"])
    } else if has(&["price", "amount", "total", "cost", "baht", "ราคา", "ยอด"]) {
        pick(&["120.00", "1,250.50", "89.00"])
    } else if has(&["qty", "quantity", "count", "number", "no.", "จำนวน"]) {
        pick(&["1", "3", "12"])
    } else if has(&["size"]) {
        pick(&["Small", "Medium", "Large"])
    } else if has(&["status", "state", "สถานะ"]) {
        pick(&["Open", "Paid", "Shipped"])
    } else if c == "id" || c.ends_with(" id") || c.ends_with("_id") || c == "#" {
        (row + 1).to_string()
    } else if has(&["city", "province", "จังหวัด"]) {
        pick(&["Bangkok", "Chiang Mai", "Khon Kaen"])
    } else if has(&["address", "ที่อยู่"]) {
        pick(&["12 Sukhumvit Rd", "88 Nimman Rd", "5 Mittraphap Rd"])
    } else if has(&["product", "item", "สินค้า"]) {
        pick(&["Coffee", "Green tea", "Croissant"])
    } else if has(&["name", "customer", "user", "ชื่อ", "ลูกค้า"]) {
        pick(&["Somchai", "Malee", "Anan"])
    } else {
        format!("{column} {}", row + 1)
    }
}

/// The sample rows for `columns`.
pub fn rows(columns: &[String]) -> Vec<Vec<String>> {
    (0..ROWS).map(|r| columns.iter().map(|c| cell(c, r)).collect()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_guessed_from_the_columns_names() {
        let r = rows(&["Name".into(), "Qty".into(), "Price".into(), "Note".into()]);
        assert_eq!(r.len(), ROWS);
        assert_eq!(r[0], vec!["Somchai", "1", "120.00", "Note 1"]);
        assert_eq!(cell("Customer Email", 1), "malee@example.com");
        assert_eq!(cell("ราคา", 2), "89.00");
        assert_eq!(cell("ID", 2), "3");
    }
}
