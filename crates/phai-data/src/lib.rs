//! Data connections, as Visual Studio's Server Explorer has them: a MySQL (or MariaDB) server by
//! host, port, user and password; its databases, their tables and views, their columns; the rows
//! of a table; any query, its result as columns and rows. Each call opens a connection of its own
//! and closes it - nothing held open behind the person's back.
//!
//! A remembered password is kept encrypted by Windows itself (DPAPI: only this Windows user on
//! this computer can read it back) - see [`protect`] and [`unprotect`].

pub mod wire;

/// A connection to a MySQL server.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Connection {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    /// The database to start in; empty for none.
    pub database: String,
    /// Talk to the server over TLS.
    pub ssl: bool,
    /// Accept the server's certificate though no authority vouches for it - a server's own,
    /// self-signed one, as SQL Server Management Studio's "Trust server certificate".
    pub trust_cert: bool,
}

/// A column of a table, as the server describes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    pub ty: String,
    pub nullable: bool,
    /// `PRI`, `UNI`, `MUL` or empty.
    pub key: String,
}

/// A query's result: its columns and rows as text, or for a statement without rows how many it
/// changed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Grid {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub affected: u64,
    /// More rows came than are kept.
    pub cut: bool,
}

/// At most this many rows are kept of a result.
pub const MAX_ROWS: usize = 1000;

/// A name quoted for MySQL: `back``tick` style.
pub fn quote(name: &str) -> String {
    format!("`{}`", name.replace('`', "``"))
}

impl Connection {
    /// How the connection is named in the tree: `user@host:port/database`.
    pub fn label(&self) -> String {
        let db = if self.database.is_empty() { String::new() } else { format!("/{}", self.database) };
        format!("MySQL · {}@{}:{}{}", self.user, self.host, self.port, db)
    }

    /// A URL for a program's code - the password left out, to come from somewhere safer.
    pub fn url_without_password(&self) -> String {
        let db = if self.database.is_empty() { String::new() } else { format!("/{}", self.database) };
        format!("mysql://{}@{}:{}{}", self.user, self.host, self.port, db)
    }

    fn open(&self, database: Option<&str>) -> Result<wire::Conn, String> {
        let db = database.map(str::to_string).or_else(|| (!self.database.is_empty()).then(|| self.database.clone()));
        wire::Conn::connect(&wire::Login { host: &self.host, port: self.port, user: &self.user, password: &self.password, database: db.as_deref(), ssl: self.ssl, trust_cert: self.trust_cert })
    }

    /// Connects and says which server it is: `8.0.36` and the like.
    pub fn test(&self) -> Result<String, String> {
        Ok(self.open(None)?.server_version.clone())
    }

    fn strings(&self, database: Option<&str>, sql: &str) -> Result<Vec<Vec<String>>, String> {
        let mut c = self.open(database)?;
        let res = c.query(sql, usize::MAX)?;
        Ok(res.into_iter().next().map(|(r, _)| r.rows.into_iter().map(|row| row.into_iter().map(|v| v.unwrap_or_default()).collect()).collect()).unwrap_or_default())
    }

    /// The databases the user may see, the server's own left out.
    pub fn databases(&self) -> Result<Vec<String>, String> {
        let all = self.strings(None, "SHOW DATABASES")?;
        Ok(all.into_iter().filter_map(|r| r.into_iter().next()).filter(|d| !["information_schema", "performance_schema", "mysql", "sys"].contains(&d.as_str())).collect())
    }

    /// The tables and views of a database: name, and whether it is a view.
    pub fn tables(&self, database: &str) -> Result<Vec<(String, bool)>, String> {
        let rows = self.strings(Some(database), &format!("SHOW FULL TABLES FROM {}", quote(database)))?;
        Ok(rows.into_iter().map(|r| (r.first().cloned().unwrap_or_default(), r.get(1).is_some_and(|k| k.eq_ignore_ascii_case("VIEW")))).collect())
    }

    /// A table's columns.
    pub fn columns(&self, database: &str, table: &str) -> Result<Vec<Column>, String> {
        let rows = self.strings(Some(database), &format!("SHOW COLUMNS FROM {} FROM {}", quote(table), quote(database)))?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let get = |i: usize| r.get(i).cloned().unwrap_or_default();
                Column { name: get(0), ty: get(1), nullable: get(2) == "YES", key: get(3) }
            })
            .collect())
    }

    /// The first rows of a table, as Show Table Data does.
    pub fn table_rows(&self, database: &str, table: &str) -> Result<Grid, String> {
        self.query(Some(database), &format!("SELECT * FROM {}.{} LIMIT {}", quote(database), quote(table), MAX_ROWS))
    }

    /// Runs one statement with `?` marks, `params` in their places (`None` for NULL) - kept
    /// apart from the SQL, so a value can never change what the statement does.
    pub fn execute(&self, database: Option<&str>, sql: &str, params: &[Option<String>]) -> Result<Grid, String> {
        let mut c = self.open(database)?;
        let refs: Vec<Option<&str>> = params.iter().map(|p| p.as_deref()).collect();
        let (r, cut) = c.execute(sql, &refs, MAX_ROWS)?;
        Ok(Grid { columns: r.columns, rows: r.rows.into_iter().map(|row| row.into_iter().map(|v| v.unwrap_or_else(|| "NULL".into())).collect()).collect(), affected: r.affected, cut })
    }

    /// Changes one value of a row, the row found by its primary key - `keys` its columns and
    /// their values now. How many rows changed: 1, or 0 when the row is no longer there.
    pub fn update_cell(&self, database: &str, table: &str, column: &str, value: Option<&str>, keys: &[(String, String)]) -> Result<u64, String> {
        let wheres: Vec<String> = keys.iter().map(|(k, _)| format!("{} = ?", quote(k))).collect();
        let sql = format!("UPDATE {}.{} SET {} = ? WHERE {} LIMIT 1", quote(database), quote(table), quote(column), wheres.join(" AND "));
        let mut params: Vec<Option<String>> = vec![value.map(str::to_string)];
        params.extend(keys.iter().map(|(_, v)| Some(v.clone())));
        Ok(self.execute(Some(database), &sql, &params)?.affected)
    }

    /// Deletes the row with these primary key values.
    pub fn delete_row(&self, database: &str, table: &str, keys: &[(String, String)]) -> Result<u64, String> {
        let wheres: Vec<String> = keys.iter().map(|(k, _)| format!("{} = ?", quote(k))).collect();
        let sql = format!("DELETE FROM {}.{} WHERE {} LIMIT 1", quote(database), quote(table), wheres.join(" AND "));
        let params: Vec<Option<String>> = keys.iter().map(|(_, v)| Some(v.clone())).collect();
        Ok(self.execute(Some(database), &sql, &params)?.affected)
    }

    /// Adds a row: these columns, these values (`None` for NULL); the others take their defaults.
    pub fn insert_row(&self, database: &str, table: &str, values: &[(String, Option<String>)]) -> Result<u64, String> {
        let cols: Vec<String> = values.iter().map(|(c, _)| quote(c)).collect();
        let marks: Vec<&str> = values.iter().map(|_| "?").collect();
        let sql = format!("INSERT INTO {}.{} ({}) VALUES ({})", quote(database), quote(table), cols.join(", "), marks.join(", "));
        let params: Vec<Option<String>> = values.iter().map(|(_, v)| v.clone()).collect();
        Ok(self.execute(Some(database), &sql, &params)?.affected)
    }

    /// Runs `sql` - one statement or several - in `database`; the last result with rows is
    /// shown, or how many rows the statements changed.
    pub fn query(&self, database: Option<&str>, sql: &str) -> Result<Grid, String> {
        let mut c = self.open(database)?;
        let mut grid = Grid::default();
        for (r, cut) in c.query(sql, MAX_ROWS)? {
            if r.columns.is_empty() {
                grid.affected += r.affected;
            } else {
                grid.columns = r.columns;
                grid.rows = r.rows.into_iter().map(|row| row.into_iter().map(|v| v.unwrap_or_else(|| "NULL".into())).collect()).collect();
                grid.cut = cut;
            }
        }
        Ok(grid)
    }
}

/// Code for a program to read `table` (or to connect, with none) - with the `mysql` crate, the
/// password from an environment variable rather than written into the code.
pub fn code_for(c: &Connection, table: Option<(&str, &[Column])>) -> String {
    let mut s = String::new();
    s.push_str("// Cargo.toml: phai-data = { path = \"...\" } - the same client Phai Studio's Server Explorer uses.\n\n");
    s.push_str("fn connection() -> phai_data::Connection {\n");
    s.push_str("    phai_data::Connection {\n");
    s.push_str(&format!("        host: {:?}.into(),\n        port: {},\n        user: {:?}.into(),\n", c.host, c.port, c.user));
    s.push_str("        // The password from the environment (PHAI_DB_PASSWORD), not from the code.\n");
    s.push_str("        password: std::env::var(\"PHAI_DB_PASSWORD\").unwrap_or_default(),\n");
    s.push_str(&format!("        database: {:?}.into(),\n        ssl: {},\n        trust_cert: {},\n    }}\n}}\n", c.database, c.ssl, c.trust_cert));
    if let Some((t, cols)) = table {
        let names: Vec<String> = cols.iter().map(|c| quote(&c.name)).collect();
        let fields: Vec<String> = cols.iter().map(|c| format!("    pub {}: {},", rust_field(&c.name), rust_type(&c.ty, c.nullable))).collect();
        let row_name = rust_struct(t);
        s.push_str(&format!("\n/// A row of `{t}`.\n#[derive(Debug, Clone, Default)]\npub struct {row_name} {{\n{}\n}}\n", fields.join("\n")));
        let reads: Vec<String> = cols
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let f = rust_field(&c.name);
                let ty = rust_type(&c.ty, c.nullable);
                if ty == "String" {
                    format!("            {f}: r[{i}].clone(),")
                } else if ty == "Option<String>" {
                    format!("            {f}: (r[{i}] != \"NULL\").then(|| r[{i}].clone()),")
                } else if ty.starts_with("Option<") {
                    format!("            {f}: r[{i}].parse().ok(),")
                } else if ty == "bool" {
                    format!("            {f}: r[{i}] == \"1\",")
                } else {
                    format!("            {f}: r[{i}].parse().unwrap_or_default(),")
                }
            })
            .collect();
        s.push_str(&format!(
            "\n/// Every row of `{t}`.\npub fn all_{}() -> Result<Vec<{row_name}>, String> {{\n    let grid = connection().query(None, \"SELECT {} FROM {}\")?;\n    Ok(grid\n        .rows\n        .iter()\n        .map(|r| {row_name} {{\n{}\n        }})\n        .collect())\n}}\n",
            rust_field(t),
            names.join(", "),
            quote(t),
            reads.join("\n")
        ));
    }
    s
}

fn rust_field(name: &str) -> String {
    let mut out: String = name.chars().map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' }).collect();
    if out.starts_with(|c: char| c.is_ascii_digit()) || ["type", "match", "fn", "struct", "use", "mod", "ref", "loop", "move", "self"].contains(&out.as_str()) {
        out.insert_str(0, "r_");
    }
    out
}

fn rust_struct(name: &str) -> String {
    let mut out = String::new();
    let mut up = true;
    for c in name.chars() {
        if c.is_alphanumeric() {
            out.push(if up { c.to_ascii_uppercase() } else { c });
            up = false;
        } else {
            up = true;
        }
    }
    if out.is_empty() { "Row".into() } else { out }
}

/// The Rust type a column's values read into.
pub fn rust_type(ty: &str, nullable: bool) -> String {
    let t = ty.to_ascii_lowercase();
    let unsigned = t.contains("unsigned");
    let base = if t.starts_with("tinyint(1)") || t.starts_with("bool") {
        "bool"
    } else if t.starts_with("tinyint") {
        if unsigned { "u8" } else { "i8" }
    } else if t.starts_with("smallint") {
        if unsigned { "u16" } else { "i16" }
    } else if t.starts_with("mediumint") || t.starts_with("int") {
        if unsigned { "u32" } else { "i32" }
    } else if t.starts_with("bigint") {
        if unsigned { "u64" } else { "i64" }
    } else if t.starts_with("float") {
        "f32"
    } else if t.starts_with("double") || t.starts_with("real") {
        "f64"
    } else if t.starts_with("blob") || t.starts_with("binary") || t.starts_with("varbinary") || t.contains("blob") {
        "Vec<u8>"
    } else {
        "String"
    };
    if nullable { format!("Option<{base}>") } else { base.to_string() }
}

// --- remembered passwords, encrypted by Windows -----------------------------------------------

/// Encrypts `data` for this Windows user on this computer (DPAPI).
pub fn protect(data: &[u8]) -> Option<Vec<u8>> {
    use windows::Win32::Security::Cryptography::{CryptProtectData, CRYPT_INTEGER_BLOB};
    let input = CRYPT_INTEGER_BLOB { cbData: data.len() as u32, pbData: data.as_ptr() as *mut u8 };
    let mut out = CRYPT_INTEGER_BLOB::default();
    unsafe { CryptProtectData(&input, windows::core::PCWSTR::null(), None, None, None, 0, &mut out) }.ok()?;
    let v = unsafe { std::slice::from_raw_parts(out.pbData, out.cbData as usize) }.to_vec();
    unsafe {
        let _ = windows::Win32::Foundation::LocalFree(Some(windows::Win32::Foundation::HLOCAL(out.pbData as _)));
    }
    Some(v)
}

/// Decrypts what [`protect`] made.
pub fn unprotect(data: &[u8]) -> Option<Vec<u8>> {
    use windows::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB};
    let input = CRYPT_INTEGER_BLOB { cbData: data.len() as u32, pbData: data.as_ptr() as *mut u8 };
    let mut out = CRYPT_INTEGER_BLOB::default();
    unsafe { CryptUnprotectData(&input, None, None, None, None, 0, &mut out) }.ok()?;
    let v = unsafe { std::slice::from_raw_parts(out.pbData, out.cbData as usize) }.to_vec();
    unsafe {
        let _ = windows::Win32::Foundation::LocalFree(Some(windows::Win32::Foundation::HLOCAL(out.pbData as _)));
    }
    Some(v)
}

/// The connections as lines of the settings file - the password apart, encrypted, as hex.
pub fn to_lines(list: &[Connection]) -> String {
    list.iter()
        .map(|c| {
            let pw = if c.password.is_empty() { String::new() } else { protect(c.password.as_bytes()).map(|b| b.iter().map(|x| format!("{x:02x}")).collect()).unwrap_or_default() };
            format!("{}\t{}\t{}\t{}\t{}\t{}\t{}", c.host, c.port, c.user, c.database, c.ssl as u8, pw, c.trust_cert as u8)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The connections back from [`to_lines`].
pub fn from_lines(text: &str) -> Vec<Connection> {
    text.lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            if f.len() < 5 {
                return None;
            }
            let pw = f.get(5).filter(|h| !h.is_empty()).and_then(|h| {
                let bytes: Option<Vec<u8>> = (0..h.len() / 2).map(|i| u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).ok()).collect();
                unprotect(&bytes?).and_then(|b| String::from_utf8(b).ok())
            });
            Some(Connection { host: f[0].into(), port: f[1].parse().ok()?, user: f[2].into(), database: f[3].into(), ssl: f[4] == "1", password: pw.unwrap_or_default(), trust_cert: f.get(6) == Some(&"1") })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_values_and_code() {
        assert_eq!(quote("we`ird"), "`we``ird`");
        assert_eq!(rust_type("int(10) unsigned", false), "u32");
        assert_eq!(rust_type("varchar(64)", true), "Option<String>");
        assert_eq!(rust_type("tinyint(1)", false), "bool");
        let c = Connection { host: "localhost".into(), port: 3306, user: "app".into(), password: "secret".into(), database: "shop".into(), ssl: false, trust_cert: false };
        assert_eq!(c.url_without_password(), "mysql://app@localhost:3306/shop");
        let cols = vec![Column { name: "id".into(), ty: "int(11)".into(), nullable: false, key: "PRI".into() }, Column { name: "type".into(), ty: "varchar(10)".into(), nullable: true, key: String::new() }];
        let code = code_for(&c, Some(("order_items", &cols)));
        assert!(code.contains("pub struct OrderItems"), "{code}");
        assert!(code.contains("pub r_type: Option<String>,"), "{code}");
        assert!(!code.contains("secret"), "the password never goes into code");
    }

    #[test]
    fn remembered_passwords_are_encrypted_and_come_back() {
        let list = vec![Connection { host: "db.local".into(), port: 3307, user: "u".into(), password: "ไผ่-pw".into(), database: String::new(), ssl: true, trust_cert: true }];
        let lines = to_lines(&list);
        assert!(!lines.contains("pw"), "{lines}");
        assert_eq!(from_lines(&lines), list);
    }

    /// The protocol against the MySQL running here, if one is: a user that does not exist is
    /// turned away by the server - so the handshake, password exchange and error all worked.
    #[test]
    fn a_wrong_user_is_turned_away_by_the_server() {
        let ssl = std::env::var_os("PHAI_TEST_SSL").is_some();
        let c = Connection { host: "127.0.0.1".into(), port: 3306, user: "phai_studio_no_such_user".into(), password: "x".into(), database: String::new(), ssl, trust_cert: ssl };
        match c.test() {
            Ok(v) => panic!("let in as nobody, server {v}"),
            Err(e) if e.contains("Access denied") || e.contains("1045") => eprintln!("server said: {e}"),
            Err(e) if e.contains("onnect") || e.contains("refused") || e.contains("timed out") => eprintln!("no MySQL here: {e}"),
            Err(e) => panic!("unexpected: {e}"),
        }
    }
}
