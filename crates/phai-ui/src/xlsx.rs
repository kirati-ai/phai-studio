//! An Excel workbook (.xlsx) of one sheet, written with nothing but the standard library: the
//! Office Open XML parts in a zip kept as it is (no packing - every Excel and LibreOffice opens
//! it). Numbers stay numbers, with their format (`#,##0.00`, `฿#,##0.00`, `0%`, dates), so the
//! sheet sums and sorts them; the headings are bold over a fill, frozen above the rows, with a
//! filter on them, as DevExpress's grid exports.

use std::fmt::Write as _;

/// One cell.
#[derive(Clone, Debug, PartialEq)]
pub enum Cell {
    Text(String),
    /// A number with its Excel format (`""`: General).
    Number(f64, String),
    /// A date (and time): its days from 1899-12-30, Excel's, with its format.
    Date(f64, String),
    Empty,
}

/// What a sheet holds.
#[derive(Clone, Debug, Default)]
pub struct Sheet {
    pub name: String,
    pub headings: Vec<String>,
    /// Each column's width, in characters.
    pub widths: Vec<f64>,
    pub rows: Vec<Vec<Cell>>,
    /// A row under the rows, bold: the totals.
    pub totals: Option<Vec<Cell>>,
}

/// The Excel format of a grid column's format (see [`crate::grid::Column::format`]): `n2` -
/// `#,##0.00`, `c2` - `฿#,##0.00`, `p0` - `0%`, `d` - `dd/mm/yyyy`, `dt` - with the time.
pub fn excel_format(format: &str) -> String {
    let f = format.trim();
    let places = |s: &str| s.parse::<usize>().unwrap_or(0).min(6);
    let decimals = |n: usize| if n == 0 { String::new() } else { format!(".{}", "0".repeat(n)) };
    match f.chars().next() {
        Some('n') => format!("#,##0{}", decimals(places(&f[1..]))),
        Some('c') => format!("\"฿\"#,##0{}", decimals(places(&f[1..]))),
        Some('p') => format!("0{}%", decimals(places(&f[1..]))),
        _ if f == "d" => "dd/mm/yyyy".into(),
        _ if f == "dt" => "dd/mm/yyyy hh:mm".into(),
        _ => String::new(),
    }
}

/// Excel's day number of `2026-11-21` (or `2026-11-21 14:30[:05]`); `None` when it is not one.
pub fn excel_date(value: &str) -> Option<f64> {
    let v = value.trim();
    let (date, time) = v.split_once([' ', 'T']).unwrap_or((v, ""));
    let p: Vec<i64> = date.split('-').map(|s| s.parse().ok()).collect::<Option<_>>()?;
    let [y, m, d] = p[..] else { return None };
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) || y < 1900 {
        return None;
    }
    // Days from 1970-01-01 (Howard Hinnant's civil-from-days, the other way).
    let (y2, m2) = if m <= 2 { (y - 1, m + 9) } else { (y, m - 3) };
    let era = y2.div_euclid(400);
    let yoe = y2 - era * 400;
    let doy = (153 * m2 + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    let mut serial = days as f64 + 25569.0;
    let t: Vec<f64> = time.split(':').filter(|s| !s.is_empty()).filter_map(|s| s.trim_end_matches('Z').parse().ok()).collect();
    if let Some(h) = t.first() {
        serial += (h * 3600.0 + t.get(1).unwrap_or(&0.0) * 60.0 + t.get(2).unwrap_or(&0.0)) / 86400.0;
    }
    Some(serial)
}

/// A grid cell as the sheet keeps it: a number or a date when its column's format says so and
/// it is one, else its words.
pub fn cell_of(format: &str, value: &str) -> Cell {
    let v = value.trim();
    if v.is_empty() {
        return Cell::Empty;
    }
    let f = format.trim();
    if f == "d" || f == "dt" {
        return excel_date(v).map(|n| Cell::Date(n, excel_format(f))).unwrap_or_else(|| Cell::Text(value.into()));
    }
    match v.replace(',', "").parse::<f64>() {
        Ok(n) if !f.is_empty() || !v.starts_with('0') || v == "0" || v.starts_with("0.") => Cell::Number(n, excel_format(f)),
        // 0812345678: a phone number, words.
        _ => Cell::Text(value.into()),
    }
}

fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            // Control characters are not XML.
            c if (c as u32) < 0x20 && !matches!(c, '\t' | '\n' | '\r') => {}
            c => o.push(c),
        }
    }
    o
}

/// `A`, `B`, ... `Z`, `AA`.
fn column_name(mut i: usize) -> String {
    let mut s = Vec::new();
    loop {
        s.push(b'A' + (i % 26) as u8);
        if i < 26 {
            break;
        }
        i = i / 26 - 1;
    }
    s.reverse();
    String::from_utf8(s).unwrap_or_default()
}

/// The sheet's parts, by name.
pub fn parts(sheet: &Sheet) -> Vec<(String, Vec<u8>)> {
    // The formats in use, each a style: 0 General, 1 the headings, 2 the totals, then one per format
    // (and its bold twin for the totals).
    let mut formats: Vec<String> = Vec::new();
    let mut each = |c: &Cell| {
        if let Cell::Number(_, f) | Cell::Date(_, f) = c {
            if !f.is_empty() && !formats.contains(f) {
                formats.push(f.clone());
            }
        }
    };
    sheet.rows.iter().flatten().for_each(&mut each);
    sheet.totals.iter().flatten().for_each(&mut each);
    let style = |c: &Cell, bold: bool| -> usize {
        let f = match c {
            Cell::Number(_, f) | Cell::Date(_, f) => f.as_str(),
            _ => "",
        };
        match formats.iter().position(|x| x == f) {
            Some(i) => 3 + i * 2 + bold as usize,
            None => if bold { 2 } else { 0 },
        }
    };
    let mut x = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">");
    x += "<sheetViews><sheetView workbookViewId=\"0\"><pane ySplit=\"1\" topLeftCell=\"A2\" activePane=\"bottomLeft\" state=\"frozen\"/></sheetView></sheetViews>";
    if !sheet.widths.is_empty() {
        x += "<cols>";
        for (i, w) in sheet.widths.iter().enumerate() {
            let _ = write!(x, "<col min=\"{0}\" max=\"{0}\" width=\"{1:.1}\" customWidth=\"1\"/>", i + 1, w);
        }
        x += "</cols>";
    }
    x += "<sheetData>";
    let row = |x: &mut String, n: usize, cells: &[Cell], bold: bool| {
        let _ = write!(x, "<row r=\"{n}\">");
        for (i, c) in cells.iter().enumerate() {
            let at = format!("{}{n}", column_name(i));
            let s = style(c, bold);
            match c {
                Cell::Text(t) => {
                    let _ = write!(x, "<c r=\"{at}\" s=\"{s}\" t=\"inlineStr\"><is><t xml:space=\"preserve\">{}</t></is></c>", esc(t));
                }
                Cell::Number(v, _) | Cell::Date(v, _) => {
                    let _ = write!(x, "<c r=\"{at}\" s=\"{s}\"><v>{v}</v></c>");
                }
                Cell::Empty if bold => {
                    let _ = write!(x, "<c r=\"{at}\" s=\"{s}\"/>");
                }
                Cell::Empty => {}
            }
        }
        x.push_str("</row>");
    };
    let heads: Vec<Cell> = sheet.headings.iter().map(|h| Cell::Text(h.clone())).collect();
    // The headings' style is 1, whatever `style` says of words.
    {
        let _ = write!(x, "<row r=\"1\">");
        for (i, h) in heads.iter().enumerate() {
            if let Cell::Text(t) = h {
                let _ = write!(x, "<c r=\"{}1\" s=\"1\" t=\"inlineStr\"><is><t xml:space=\"preserve\">{}</t></is></c>", column_name(i), esc(t));
            }
        }
        x.push_str("</row>");
    }
    for (i, r) in sheet.rows.iter().enumerate() {
        row(&mut x, i + 2, r, false);
    }
    if let Some(t) = &sheet.totals {
        row(&mut x, sheet.rows.len() + 2, t, true);
    }
    x += "</sheetData>";
    if !sheet.headings.is_empty() {
        let _ = write!(x, "<autoFilter ref=\"A1:{}{}\"/>", column_name(sheet.headings.len() - 1), sheet.rows.len() + 1);
    }
    x += "<pageMargins left=\"0.5\" right=\"0.5\" top=\"0.75\" bottom=\"0.75\" header=\"0.3\" footer=\"0.3\"/></worksheet>";

    let mut st = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<styleSheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">");
    if !formats.is_empty() {
        let _ = write!(st, "<numFmts count=\"{}\">", formats.len());
        for (i, f) in formats.iter().enumerate() {
            let _ = write!(st, "<numFmt numFmtId=\"{}\" formatCode=\"{}\"/>", 164 + i, esc(f));
        }
        st += "</numFmts>";
    }
    st += "<fonts count=\"2\"><font><sz val=\"11\"/><name val=\"Calibri\"/><family val=\"2\"/></font><font><b/><sz val=\"11\"/><name val=\"Calibri\"/><family val=\"2\"/></font></fonts>";
    st += "<fills count=\"3\"><fill><patternFill patternType=\"none\"/></fill><fill><patternFill patternType=\"gray125\"/></fill><fill><patternFill patternType=\"solid\"><fgColor rgb=\"FFE8EEF5\"/><bgColor indexed=\"64\"/></patternFill></fill></fills>";
    st += "<borders count=\"2\"><border><left/><right/><top/><bottom/><diagonal/></border><border><left/><right/><top style=\"thin\"><color rgb=\"FF9CA3AF\"/></top><bottom style=\"thin\"><color rgb=\"FF9CA3AF\"/></bottom><diagonal/></border></borders>";
    st += "<cellStyleXfs count=\"1\"><xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\"/></cellStyleXfs>";
    let _ = write!(st, "<cellXfs count=\"{}\">", 3 + formats.len() * 2);
    st += "<xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\" xfId=\"0\"/>";
    st += "<xf numFmtId=\"0\" fontId=\"1\" fillId=\"2\" borderId=\"1\" xfId=\"0\" applyFont=\"1\" applyFill=\"1\" applyBorder=\"1\"/>";
    st += "<xf numFmtId=\"0\" fontId=\"1\" fillId=\"0\" borderId=\"1\" xfId=\"0\" applyFont=\"1\" applyBorder=\"1\"/>";
    for i in 0..formats.len() {
        let _ = write!(st, "<xf numFmtId=\"{0}\" fontId=\"0\" fillId=\"0\" borderId=\"0\" xfId=\"0\" applyNumberFormat=\"1\"/><xf numFmtId=\"{0}\" fontId=\"1\" fillId=\"0\" borderId=\"1\" xfId=\"0\" applyNumberFormat=\"1\" applyFont=\"1\" applyBorder=\"1\"/>", 164 + i);
    }
    st += "</cellXfs><cellStyles count=\"1\"><cellStyle name=\"Normal\" xfId=\"0\" builtinId=\"0\"/></cellStyles></styleSheet>";

    let name = esc(&sheet.name.chars().filter(|c| !"[]:*?/\\".contains(*c)).take(31).collect::<String>());
    let name = if name.is_empty() { "Sheet1".to_string() } else { name };
    let has_filter = !sheet.headings.is_empty();
    let mut wb = format!("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><sheets><sheet name=\"{name}\" sheetId=\"1\" r:id=\"rId1\"/></sheets>");
    if has_filter {
        let _ = write!(wb, "<definedNames><definedName name=\"_xlnm._FilterDatabase\" localSheetId=\"0\" hidden=\"1\">'{}'!$A$1:${}${}</definedName></definedNames>", name.replace('\'', "''"), column_name(sheet.headings.len() - 1), sheet.rows.len() + 1);
    }
    wb += "</workbook>";

    vec![
        ("[Content_Types].xml".into(), b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/><Override PartName=\"/xl/worksheets/sheet1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/><Override PartName=\"/xl/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml\"/></Types>".to_vec()),
        ("_rels/.rels".into(), b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/></Relationships>".to_vec()),
        ("xl/workbook.xml".into(), wb.into_bytes()),
        ("xl/_rels/workbook.xml.rels".into(), b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet1.xml\"/><Relationship Id=\"rId2\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/></Relationships>".to_vec()),
        ("xl/worksheets/sheet1.xml".into(), x.into_bytes()),
        ("xl/styles.xml".into(), st.into_bytes()),
    ]
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for b in data {
        crc ^= *b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

/// Files in a zip, kept as they are (method 0).
pub fn zip(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    // 2026-01-01 00:00, in DOS's packing - a date is needed, which one is not.
    let (time, date) = (0u16, ((2026 - 1980) << 9 | 1 << 5 | 1) as u16);
    for (name, data) in files {
        let offset = out.len() as u32;
        let crc = crc32(data);
        let n = name.as_bytes();
        let head = |sig: u32, central: bool| {
            let mut h = Vec::new();
            h.extend(sig.to_le_bytes());
            if central {
                h.extend(20u16.to_le_bytes());
            }
            h.extend(20u16.to_le_bytes());
            h.extend(0x0800u16.to_le_bytes()); // UTF-8 names
            h.extend(0u16.to_le_bytes());
            h.extend(time.to_le_bytes());
            h.extend(date.to_le_bytes());
            h.extend(crc.to_le_bytes());
            h.extend((data.len() as u32).to_le_bytes());
            h.extend((data.len() as u32).to_le_bytes());
            h.extend((n.len() as u16).to_le_bytes());
            h.extend(0u16.to_le_bytes());
            if central {
                h.extend([0u8; 6]); // comment length, disk, internal attributes
                h.extend(0u32.to_le_bytes());
                h.extend(offset.to_le_bytes());
            }
            h.extend(n);
            h
        };
        out.extend(head(0x0403_4b50, false));
        out.extend(data);
        central.extend(head(0x0201_4b50, true));
    }
    let start = out.len() as u32;
    let size = central.len() as u32;
    out.extend(central);
    out.extend(0x0605_4b50u32.to_le_bytes());
    out.extend([0u8; 4]);
    out.extend((files.len() as u16).to_le_bytes());
    out.extend((files.len() as u16).to_le_bytes());
    out.extend(size.to_le_bytes());
    out.extend(start.to_le_bytes());
    out.extend(0u16.to_le_bytes());
    out
}

/// The sheet written to `path` as an Excel workbook.
pub fn write(path: &std::path::Path, sheet: &Sheet) -> std::io::Result<()> {
    std::fs::write(path, zip(&parts(sheet)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_and_dates_as_excel_has_them() {
        assert_eq!(excel_format("n2"), "#,##0.00");
        assert_eq!(excel_format("n0"), "#,##0");
        assert_eq!(excel_format("c2"), "\"฿\"#,##0.00");
        assert_eq!(excel_format("p1"), "0.0%");
        assert_eq!(excel_format("d"), "dd/mm/yyyy");
        assert_eq!(excel_date("1970-01-01"), Some(25569.0));
        assert_eq!(excel_date("2026-11-21"), Some(46347.0));
        assert_eq!(excel_date("2026-11-21 12:00"), Some(46347.5));
        assert_eq!(excel_date("21/11/2026"), None);
        assert_eq!(cell_of("", "0812345678"), Cell::Text("0812345678".into()));
        assert_eq!(cell_of("n2", "1,234.5"), Cell::Number(1234.5, "#,##0.00".into()));
        assert_eq!(cell_of("", "42"), Cell::Number(42.0, String::new()));
        assert_eq!(column_name(0), "A");
        assert_eq!(column_name(25), "Z");
        assert_eq!(column_name(26), "AA");
        assert_eq!(column_name(27), "AB");
    }

    #[test]
    fn a_workbook_is_a_zip_of_its_parts() {
        let sheet = Sheet {
            name: "Invoices".into(),
            headings: vec!["Customer".into(), "Amount".into()],
            widths: vec![20.0, 12.0],
            rows: vec![vec![Cell::Text("ไผ่ทอง & Co".into()), cell_of("n2", "1500")]],
            totals: Some(vec![Cell::Text("Sum".into()), Cell::Number(1500.0, "#,##0.00".into())]),
        };
        let p = parts(&sheet);
        let xml = String::from_utf8(p[4].1.clone()).unwrap();
        assert!(xml.contains("ไผ่ทอง &amp; Co"));
        assert!(xml.contains("<c r=\"B2\" s=\"3\"><v>1500</v></c>"));
        assert!(xml.contains("<c r=\"B3\" s=\"4\"><v>1500</v></c>"));
        assert!(xml.contains("<autoFilter ref=\"A1:B2\"/>"));
        let z = zip(&p);
        assert_eq!(&z[..4], b"PK\x03\x04");
        assert_eq!(&z[z.len() - 22..z.len() - 18], b"PK\x05\x06");
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }
}
