//! Data connections, as Visual Studio's Server Explorer has them: a MySQL / MariaDB or PostgreSQL
//! server by host, port, user and password, or a SQLite file; its databases, their tables and
//! views, their columns; the rows of a table, changed one at a time; any query, its result as
//! columns and rows. Each call opens a connection of its own and closes it - nothing held open
//! behind the person's back.
//!
//! A remembered password is kept encrypted by Windows itself (DPAPI: only this Windows user on
//! this computer can read it back) - see [`protect`] and [`unprotect`].

pub mod pg;
pub mod schema;
pub mod sqlite;
pub mod wire;

use wire::Result1;

/// What kind of database a connection is to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    #[default]
    MySql,
    Postgres,
    /// A file; `host` is its path.
    Sqlite,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::MySql => "MySQL",
            Kind::Postgres => "PostgreSQL",
            Kind::Sqlite => "SQLite",
        }
    }
    fn word(self) -> &'static str {
        match self {
            Kind::MySql => "mysql",
            Kind::Postgres => "postgres",
            Kind::Sqlite => "sqlite",
        }
    }
    fn from_word(w: &str) -> Kind {
        match w {
            "postgres" => Kind::Postgres,
            "sqlite" => Kind::Sqlite,
            _ => Kind::MySql,
        }
    }
    /// The port its servers listen on.
    pub fn default_port(self) -> u16 {
        match self {
            Kind::MySql => 3306,
            Kind::Postgres => 5432,
            Kind::Sqlite => 0,
        }
    }
}

/// A connection to a database.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Connection {
    /// The server - or, for SQLite, the file.
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    /// The database to start in; empty for none (PostgreSQL then uses `postgres`).
    pub database: String,
    /// Talk to the server over TLS.
    pub ssl: bool,
    /// Accept the server's certificate though no authority vouches for it - a server's own,
    /// self-signed one, as SQL Server Management Studio's "Trust server certificate".
    pub trust_cert: bool,
    pub kind: Kind,
}

/// A column of a table, as the server describes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    pub ty: String,
    pub nullable: bool,
    /// `PRI` for the primary key's columns, `UNI`, `MUL` or empty.
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

/// A name quoted the standard way, as PostgreSQL and SQLite take it: `"double""quoted"`.
pub fn quote_std(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

fn to_grid(results: Vec<(Result1, bool)>) -> Grid {
    let mut grid = Grid::default();
    for (r, cut) in results {
        if r.columns.is_empty() {
            grid.affected += r.affected;
        } else {
            grid.columns = r.columns;
            grid.rows = r.rows.into_iter().map(|row| row.into_iter().map(|v| v.unwrap_or_else(|| "NULL".into())).collect()).collect();
            grid.cut = cut;
        }
    }
    grid
}

impl Connection {
    /// How the connection is named in the tree.
    pub fn label(&self) -> String {
        let db = if self.database.is_empty() { String::new() } else { format!("/{}", self.database) };
        match self.kind {
            Kind::Sqlite => format!("SQLite · {}", std::path::Path::new(&self.host).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| self.host.clone())),
            k => format!("{} · {}@{}:{}{}", k.name(), self.user, self.host, self.port, db),
        }
    }

    /// A URL for a program's code - the password left out, to come from somewhere safer.
    pub fn url_without_password(&self) -> String {
        let db = if self.database.is_empty() { String::new() } else { format!("/{}", self.database) };
        match self.kind {
            Kind::Sqlite => format!("sqlite://{}", self.host),
            k => format!("{}://{}@{}:{}{}", k.word(), self.user, self.host, self.port, db),
        }
    }

    /// A name quoted as this kind of database takes it.
    pub fn q(&self, name: &str) -> String {
        if self.kind == Kind::MySql { quote(name) } else { quote_std(name) }
    }

    /// The `n`th value's mark in a statement, from 1: `?`, or PostgreSQL's `$n`.
    fn mark(&self, n: usize) -> String {
        if self.kind == Kind::Postgres { format!("${n}") } else { "?".into() }
    }

    /// A table as a statement names it: `db`.`t`, "t", or PostgreSQL's "schema"."t".
    pub fn table_ref(&self, database: &str, table: &str) -> String {
        match self.kind {
            Kind::MySql => format!("{}.{}", quote(database), quote(table)),
            Kind::Sqlite => quote_std(table),
            Kind::Postgres => match table.split_once('.') {
                Some((schema, name)) => format!("{}.{}", quote_std(schema), quote_std(name)),
                None => quote_std(table),
            },
        }
    }

    fn mysql(&self, database: Option<&str>) -> Result<wire::Conn, String> {
        let db = database.filter(|d| !d.is_empty()).map(str::to_string).or_else(|| (!self.database.is_empty()).then(|| self.database.clone()));
        wire::Conn::connect(&wire::Login { host: &self.host, port: self.port, user: &self.user, password: &self.password, database: db.as_deref(), ssl: self.ssl, trust_cert: self.trust_cert })
    }

    fn postgres(&self, database: Option<&str>) -> Result<pg::Conn, String> {
        let db = database.filter(|d| !d.is_empty()).unwrap_or(&self.database).to_string();
        pg::Conn::connect(&pg::Login { host: &self.host, port: self.port, user: &self.user, password: &self.password, database: &db, ssl: self.ssl, trust_cert: self.trust_cert })
    }

    /// Every result of `sql` in `database`.
    fn run(&self, database: Option<&str>, sql: &str, max: usize) -> Result<Vec<(Result1, bool)>, String> {
        match self.kind {
            Kind::MySql => self.mysql(database)?.query(sql, max),
            Kind::Postgres => self.postgres(database)?.query(sql, max),
            Kind::Sqlite => sqlite::query(&self.host, sql, max),
        }
    }

    fn run_params(&self, database: Option<&str>, sql: &str, params: &[Option<&str>], max: usize) -> Result<(Result1, bool), String> {
        match self.kind {
            Kind::MySql => self.mysql(database)?.execute(sql, params, max),
            Kind::Postgres => self.postgres(database)?.execute(sql, params, max),
            Kind::Sqlite => sqlite::execute(&self.host, sql, params, max),
        }
    }

    pub(crate) fn strings(&self, database: Option<&str>, sql: &str, params: &[Option<&str>]) -> Result<Vec<Vec<String>>, String> {
        let r = if params.is_empty() { self.run(database, sql, usize::MAX)?.into_iter().next().map(|(r, _)| r).unwrap_or_default() } else { self.run_params(database, sql, params, usize::MAX)?.0 };
        Ok(r.rows.into_iter().map(|row| row.into_iter().map(|v| v.unwrap_or_default()).collect()).collect())
    }

    /// Connects and says which server it is: `8.0.36`, `16.2`, the SQLite library's version.
    pub fn test(&self) -> Result<String, String> {
        match self.kind {
            Kind::MySql => Ok(self.mysql(None)?.server_version.clone()),
            Kind::Postgres => Ok(self.postgres(None)?.server_version.clone()),
            Kind::Sqlite => sqlite::version(&self.host),
        }
    }

    /// The databases the user may see, the server's own left out. A SQLite file is one: `main`.
    pub fn databases(&self) -> Result<Vec<String>, String> {
        match self.kind {
            Kind::MySql => {
                let all = self.strings(None, "SHOW DATABASES", &[])?;
                Ok(all.into_iter().filter_map(|r| r.into_iter().next()).filter(|d| !["information_schema", "performance_schema", "mysql", "sys"].contains(&d.as_str())).collect())
            }
            Kind::Postgres => Ok(self.strings(None, "SELECT datname FROM pg_database WHERE NOT datistemplate AND datallowconn ORDER BY 1", &[])?.into_iter().filter_map(|r| r.into_iter().next()).collect()),
            Kind::Sqlite => Ok(vec!["main".into()]),
        }
    }

    /// The tables and views of a database: name, and whether it is a view. PostgreSQL's outside
    /// the `public` schema are named `schema.table`.
    pub fn tables(&self, database: &str) -> Result<Vec<(String, bool)>, String> {
        match self.kind {
            Kind::MySql => {
                let rows = self.strings(Some(database), &format!("SHOW FULL TABLES FROM {}", quote(database)), &[])?;
                Ok(rows.into_iter().map(|r| (r.first().cloned().unwrap_or_default(), r.get(1).is_some_and(|k| k.eq_ignore_ascii_case("VIEW")))).collect())
            }
            Kind::Postgres => {
                let rows = self.strings(Some(database), "SELECT table_schema, table_name, table_type FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog', 'information_schema') ORDER BY 1, 2", &[])?;
                Ok(rows
                    .into_iter()
                    .map(|r| {
                        let name = if r[0] == "public" { r[1].clone() } else { format!("{}.{}", r[0], r[1]) };
                        (name, r.get(2).is_some_and(|k| k == "VIEW"))
                    })
                    .collect())
            }
            Kind::Sqlite => {
                let rows = self.strings(None, "SELECT name, type FROM sqlite_master WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%' ORDER BY name", &[])?;
                Ok(rows.into_iter().map(|r| (r[0].clone(), r.get(1).is_some_and(|k| k == "view"))).collect())
            }
        }
    }

    /// A table's columns.
    pub fn columns(&self, database: &str, table: &str) -> Result<Vec<Column>, String> {
        match self.kind {
            Kind::MySql => {
                let rows = self.strings(Some(database), &format!("SHOW COLUMNS FROM {} FROM {}", quote(table), quote(database)), &[])?;
                Ok(rows
                    .into_iter()
                    .map(|r| {
                        let get = |i: usize| r.get(i).cloned().unwrap_or_default();
                        Column { name: get(0), ty: get(1), nullable: get(2) == "YES", key: get(3) }
                    })
                    .collect())
            }
            Kind::Postgres => {
                let (schema, name) = table.split_once('.').unwrap_or(("public", table));
                let sql = "SELECT c.column_name, CASE WHEN c.character_maximum_length IS NOT NULL THEN c.data_type || '(' || c.character_maximum_length || ')' ELSE c.data_type END, c.is_nullable, \
                           CASE WHEN EXISTS (SELECT 1 FROM information_schema.table_constraints tc JOIN information_schema.key_column_usage k ON tc.constraint_name = k.constraint_name AND tc.table_schema = k.table_schema \
                           WHERE tc.constraint_type = 'PRIMARY KEY' AND tc.table_schema = c.table_schema AND tc.table_name = c.table_name AND k.column_name = c.column_name) THEN 'PRI' ELSE '' END \
                           FROM information_schema.columns c WHERE c.table_schema = $1 AND c.table_name = $2 ORDER BY c.ordinal_position";
                let rows = self.strings(Some(database), sql, &[Some(schema), Some(name)])?;
                Ok(rows.into_iter().map(|r| Column { name: r[0].clone(), ty: r[1].clone(), nullable: r[2] == "YES", key: r[3].clone() }).collect())
            }
            Kind::Sqlite => {
                let rows = self.strings(None, &format!("PRAGMA table_info({})", quote_std(table)), &[])?;
                Ok(rows.into_iter().map(|r| Column { name: r[1].clone(), ty: r[2].clone(), nullable: r[3] == "0", key: if r[5] != "0" { "PRI".into() } else { String::new() } }).collect())
            }
        }
    }

    /// The first rows of a table, as Show Table Data does.
    pub fn table_rows(&self, database: &str, table: &str) -> Result<Grid, String> {
        self.query(Some(database), &format!("SELECT * FROM {} LIMIT {}", self.table_ref(database, table), MAX_ROWS))
    }

    /// The statement that makes a table as it is now - for Design Table.
    pub fn create_sql(&self, database: &str, table: &str) -> Result<String, String> {
        match self.kind {
            Kind::MySql => Ok(self.strings(Some(database), &format!("SHOW CREATE TABLE {}", self.table_ref(database, table)), &[])?.into_iter().next().and_then(|r| r.get(1).cloned()).unwrap_or_default() + ";"),
            Kind::Sqlite => Ok(self.strings(None, "SELECT sql FROM sqlite_master WHERE name = ?", &[Some(table)])?.into_iter().next().and_then(|r| r.into_iter().next()).unwrap_or_default() + ";"),
            Kind::Postgres => {
                let cols = self.columns(database, table)?;
                let mut lines: Vec<String> = cols.iter().map(|c| format!("    {} {}{}", quote_std(&c.name), c.ty, if c.nullable { "" } else { " NOT NULL" })).collect();
                let pk: Vec<String> = cols.iter().filter(|c| c.key == "PRI").map(|c| quote_std(&c.name)).collect();
                if !pk.is_empty() {
                    lines.push(format!("    PRIMARY KEY ({})", pk.join(", ")));
                }
                Ok(format!("CREATE TABLE {} (\n{}\n);", self.table_ref(database, table), lines.join(",\n")))
            }
        }
    }

    /// A new table's statement to start from, in this kind's own words.
    pub fn new_table_sql(&self) -> String {
        match self.kind {
            Kind::MySql => "CREATE TABLE new_table (\n    id INT AUTO_INCREMENT PRIMARY KEY,\n    name VARCHAR(100) NOT NULL,\n    created_at DATETIME DEFAULT CURRENT_TIMESTAMP\n);\n",
            Kind::Postgres => "CREATE TABLE new_table (\n    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,\n    name TEXT NOT NULL,\n    created_at TIMESTAMP DEFAULT now()\n);\n",
            Kind::Sqlite => "CREATE TABLE new_table (\n    id INTEGER PRIMARY KEY AUTOINCREMENT,\n    name TEXT NOT NULL,\n    created_at TEXT DEFAULT CURRENT_TIMESTAMP\n);\n",
        }
        .into()
    }

    /// How to change a table, in this kind's own words - for Design Table.
    pub fn alter_examples(&self, database: &str, table: &str) -> String {
        let t = self.table_ref(database, table);
        match self.kind {
            Kind::MySql => format!("-- ALTER TABLE {t} ADD COLUMN note VARCHAR(200) NULL;\n-- ALTER TABLE {t} MODIFY COLUMN note TEXT;\n-- ALTER TABLE {t} RENAME COLUMN note TO remark;\n-- ALTER TABLE {t} DROP COLUMN remark;\n"),
            Kind::Postgres => format!("-- ALTER TABLE {t} ADD COLUMN note TEXT;\n-- ALTER TABLE {t} ALTER COLUMN note TYPE VARCHAR(200);\n-- ALTER TABLE {t} RENAME COLUMN note TO remark;\n-- ALTER TABLE {t} DROP COLUMN remark;\n"),
            Kind::Sqlite => format!("-- ALTER TABLE {t} ADD COLUMN note TEXT;\n-- ALTER TABLE {t} RENAME COLUMN note TO remark;\n-- ALTER TABLE {t} DROP COLUMN remark;\n"),
        }
    }

    /// Runs one statement with value marks (`?`, or `$1`... for PostgreSQL), `params` in their
    /// places (`None` for NULL) - kept apart from the SQL, so a value can never change what the
    /// statement does.
    pub fn execute(&self, database: Option<&str>, sql: &str, params: &[Option<String>]) -> Result<Grid, String> {
        let refs: Vec<Option<&str>> = params.iter().map(|p| p.as_deref()).collect();
        let (r, cut) = self.run_params(database, sql, &refs, MAX_ROWS)?;
        Ok(to_grid(vec![(r, cut)]))
    }

    /// `WHERE "k1" = ? AND "k2" = ?`, the marks numbered from `first`.
    fn key_where(&self, keys: &[(String, String)], first: usize) -> String {
        keys.iter().enumerate().map(|(i, (k, _))| format!("{} = {}", self.q(k), self.mark(first + i))).collect::<Vec<_>>().join(" AND ")
    }

    /// Changes one value of a row, the row found by its primary key - `keys` its columns and
    /// their values now. How many rows changed: 1, or 0 when the row is no longer there.
    pub fn update_cell(&self, database: &str, table: &str, column: &str, value: Option<&str>, keys: &[(String, String)]) -> Result<u64, String> {
        let limit = if self.kind == Kind::MySql { " LIMIT 1" } else { "" };
        let sql = format!("UPDATE {} SET {} = {} WHERE {}{limit}", self.table_ref(database, table), self.q(column), self.mark(1), self.key_where(keys, 2));
        let mut params: Vec<Option<String>> = vec![value.map(str::to_string)];
        params.extend(keys.iter().map(|(_, v)| Some(v.clone())));
        Ok(self.execute(Some(database), &sql, &params)?.affected)
    }

    /// Deletes the row with these primary key values.
    pub fn delete_row(&self, database: &str, table: &str, keys: &[(String, String)]) -> Result<u64, String> {
        let limit = if self.kind == Kind::MySql { " LIMIT 1" } else { "" };
        let sql = format!("DELETE FROM {} WHERE {}{limit}", self.table_ref(database, table), self.key_where(keys, 1));
        let params: Vec<Option<String>> = keys.iter().map(|(_, v)| Some(v.clone())).collect();
        Ok(self.execute(Some(database), &sql, &params)?.affected)
    }

    /// Adds a row: these columns, these values (`None` for NULL); the others take their defaults.
    pub fn insert_row(&self, database: &str, table: &str, values: &[(String, Option<String>)]) -> Result<u64, String> {
        let cols: Vec<String> = values.iter().map(|(c, _)| self.q(c)).collect();
        let marks: Vec<String> = (1..=values.len()).map(|i| self.mark(i)).collect();
        let sql = format!("INSERT INTO {} ({}) VALUES ({})", self.table_ref(database, table), cols.join(", "), marks.join(", "));
        let params: Vec<Option<String>> = values.iter().map(|(_, v)| v.clone()).collect();
        Ok(self.execute(Some(database), &sql, &params)?.affected)
    }

    /// Runs `sql` - one statement or several - in `database`; the last result with rows is
    /// shown, or how many rows the statements changed.
    pub fn query(&self, database: Option<&str>, sql: &str) -> Result<Grid, String> {
        Ok(to_grid(self.run(database, sql, MAX_ROWS)?))
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
    s.push_str(&format!("        database: {:?}.into(),\n        ssl: {},\n        trust_cert: {},\n        kind: phai_data::Kind::{:?},\n    }}\n}}\n", c.database, c.ssl, c.trust_cert, c.kind));
    if let Some((t, cols)) = table {
        let names: Vec<String> = cols.iter().map(|col| c.q(&col.name)).collect();
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
            names.join(", ").replace('"', "\\\""),
            c.table_ref(&c.database, t).replace('"', "\\\""),
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
            format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", c.host, c.port, c.user, c.database, c.ssl as u8, pw, c.trust_cert as u8, c.kind.word())
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
            Some(Connection { host: f[0].into(), port: f[1].parse().ok()?, user: f[2].into(), database: f[3].into(), ssl: f[4] == "1", password: pw.unwrap_or_default(), trust_cert: f.get(6) == Some(&"1"), kind: Kind::from_word(f.get(7).copied().unwrap_or("mysql")) })
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
        let c = Connection { host: "localhost".into(), port: 3306, user: "app".into(), password: "secret".into(), database: "shop".into(), ssl: false, trust_cert: false, kind: Kind::MySql };
        assert_eq!(c.url_without_password(), "mysql://app@localhost:3306/shop");
        let cols = vec![Column { name: "id".into(), ty: "int(11)".into(), nullable: false, key: "PRI".into() }, Column { name: "type".into(), ty: "varchar(10)".into(), nullable: true, key: String::new() }];
        let code = code_for(&c, Some(("order_items", &cols)));
        assert!(code.contains("pub struct OrderItems"), "{code}");
        assert!(code.contains("pub r_type: Option<String>,"), "{code}");
        assert!(!code.contains("secret"), "the password never goes into code");
    }

    #[test]
    fn remembered_passwords_are_encrypted_and_come_back() {
        let list = vec![Connection { host: "db.local".into(), port: 3307, user: "u".into(), password: "ไผ่-pw".into(), database: String::new(), ssl: true, trust_cert: true, kind: Kind::Postgres }];
        let lines = to_lines(&list);
        assert!(!lines.contains("pw"), "{lines}");
        assert_eq!(from_lines(&lines), list);
    }

    /// The protocol against the MySQL running here, if one is: a user that does not exist is
    /// turned away by the server - so the handshake, password exchange and error all worked.
    #[test]
    fn a_wrong_user_is_turned_away_by_the_server() {
        let ssl = std::env::var_os("PHAI_TEST_SSL").is_some();
        let c = Connection { host: "127.0.0.1".into(), port: 3306, user: "phai_studio_no_such_user".into(), password: "x".into(), database: String::new(), ssl, trust_cert: ssl, kind: Kind::MySql };
        match c.test() {
            Ok(v) => panic!("let in as nobody, server {v}"),
            Err(e) if e.contains("Access denied") || e.contains("1045") => eprintln!("server said: {e}"),
            Err(e) if e.contains("onnect") || e.contains("refused") || e.contains("timed out") => eprintln!("no MySQL here: {e}"),
            Err(e) => panic!("unexpected: {e}"),
        }
    }

    /// PostgreSQL here, if one is: a role that does not exist is turned away - after the server's
    /// SCRAM challenge, so startup, SCRAM and the error all worked.
    #[test]
    fn postgres_turns_away_a_wrong_role() {
        let c = Connection { host: "127.0.0.1".into(), port: 5432, user: "phai_studio_no_such_role".into(), password: "x".into(), kind: Kind::Postgres, ..Default::default() };
        match c.test() {
            Ok(v) => panic!("let in as nobody, server {v}"),
            Err(e) if e.starts_with("28P01") || e.starts_with("28000") => eprintln!("server said: {e}"),
            Err(e) if e.contains("onnect") || e.contains("refused") => eprintln!("no PostgreSQL here: {e}"),
            Err(e) => panic!("unexpected: {e}"),
        }
    }

    /// SQLite end to end in a file of its own: a table made, rows added, changed and deleted by
    /// their primary key, as Server Explorer does it.
    #[test]
    fn sqlite_tables_rows_and_edits() {
        let path = std::env::temp_dir().join(format!("phai-data-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let c = Connection { host: path.to_string_lossy().to_string(), kind: Kind::Sqlite, ..Default::default() };
        assert!(c.test().unwrap().starts_with('3'));
        c.query(None, &c.new_table_sql()).unwrap();
        assert_eq!(c.insert_row("main", "new_table", &[("name".into(), Some("ไผ่".into()))]).unwrap(), 1);
        assert_eq!(c.insert_row("main", "new_table", &[("name".into(), Some("bamboo".into()))]).unwrap(), 1);
        assert_eq!(c.tables("main").unwrap(), vec![("new_table".to_string(), false)]);
        let cols = c.columns("main", "new_table").unwrap();
        assert_eq!(cols[0].name, "id");
        assert_eq!(cols[0].key, "PRI");
        let g = c.table_rows("main", "new_table").unwrap();
        assert_eq!(g.rows.len(), 2);
        assert_eq!(g.rows[0][1], "ไผ่");
        assert_eq!(c.update_cell("main", "new_table", "name", Some("ไผ่สีสุก"), &[("id".into(), "1".into())]).unwrap(), 1);
        assert_eq!(c.delete_row("main", "new_table", &[("id".into(), "2".into())]).unwrap(), 1);
        let g = c.query(None, "SELECT name FROM new_table; ").unwrap();
        assert_eq!(g.rows, vec![vec!["ไผ่สีสุก".to_string()]]);
        assert!(c.create_sql("main", "new_table").unwrap().starts_with("CREATE TABLE new_table"));
        let _ = std::fs::remove_file(&path);
    }
}
