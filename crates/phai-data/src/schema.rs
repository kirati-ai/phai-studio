//! A table's design, as Visual Studio's Table Designer holds it: its columns (name, type, nulls,
//! primary key, identity, default), its indexes and its foreign keys - read from a database, and
//! the statements that make a design so: `CREATE TABLE` for a new table, `ALTER TABLE` for a
//! changed one, and for SQLite, which can alter little, the table built anew with its rows kept.

use crate::{quote_std, Connection, Kind};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ColDef {
    pub name: String,
    pub ty: String,
    pub nullable: bool,
    /// The default's SQL (`0`, `'x'`, `CURRENT_TIMESTAMP`); empty for none.
    pub default: String,
    pub pk: bool,
    /// Numbered by the database: AUTO_INCREMENT, IDENTITY, AUTOINCREMENT.
    pub auto: bool,
    /// Its name in the database before this design - `None` for a column added here.
    pub was: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IndexDef {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FkDef {
    pub name: String,
    pub columns: Vec<String>,
    pub ref_table: String,
    pub ref_columns: Vec<String>,
    /// `NO ACTION`, `RESTRICT`, `CASCADE`, `SET NULL`, `SET DEFAULT`.
    pub on_delete: String,
    pub on_update: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TableDef {
    pub name: String,
    pub columns: Vec<ColDef>,
    pub indexes: Vec<IndexDef>,
    pub fkeys: Vec<FkDef>,
    /// PostgreSQL's name for the primary key's constraint, to drop it by.
    pub pk_name: String,
}

/// The rules a foreign key may take.
pub const RULES: [&str; 5] = ["NO ACTION", "RESTRICT", "CASCADE", "SET NULL", "SET DEFAULT"];

/// Data types to choose from, in each kind's own words.
pub fn types_for(kind: Kind) -> &'static [&'static str] {
    match kind {
        Kind::MySql => &["INT", "BIGINT", "SMALLINT", "TINYINT", "TINYINT(1)", "DECIMAL(10,2)", "DOUBLE", "FLOAT", "VARCHAR(50)", "VARCHAR(100)", "VARCHAR(255)", "CHAR(10)", "TEXT", "MEDIUMTEXT", "DATE", "DATETIME", "TIMESTAMP", "TIME", "JSON", "BLOB"],
        Kind::Postgres => &["integer", "bigint", "smallint", "boolean", "numeric(10,2)", "double precision", "real", "varchar(50)", "varchar(100)", "varchar(255)", "text", "date", "timestamp", "timestamptz", "time", "uuid", "jsonb", "bytea"],
        Kind::Sqlite => &["INTEGER", "REAL", "TEXT", "NUMERIC", "BLOB"],
    }
}

fn split_list(s: &str) -> Vec<String> {
    s.split(',').map(|x| x.trim().trim_matches('"').trim_matches('`').to_string()).filter(|x| !x.is_empty()).collect()
}

fn pg_rule(c: &str) -> String {
    match c {
        "r" => "RESTRICT",
        "c" => "CASCADE",
        "n" => "SET NULL",
        "d" => "SET DEFAULT",
        _ => "NO ACTION",
    }
    .into()
}

impl Connection {
    /// A table's design as the database has it now.
    pub fn table_def(&self, database: &str, table: &str) -> Result<TableDef, String> {
        let mut d = TableDef { name: table.to_string(), ..Default::default() };
        match self.kind {
            Kind::MySql => {
                let rows = self.strings(Some(database), "SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE, COALESCE(COLUMN_DEFAULT, ''), COLUMN_KEY, EXTRA FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? ORDER BY ORDINAL_POSITION", &[Some(database), Some(table)])?;
                for r in rows {
                    d.columns.push(ColDef { name: r[0].clone(), ty: r[1].to_uppercase(), nullable: r[2] == "YES", default: mysql_default(&r[3], &r[1]), pk: r[4] == "PRI", auto: r[5].contains("auto_increment"), was: Some(r[0].clone()) });
                }
                let rows = self.strings(Some(database), "SELECT INDEX_NAME, COLUMN_NAME, NON_UNIQUE FROM information_schema.STATISTICS WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? AND INDEX_NAME <> 'PRIMARY' ORDER BY INDEX_NAME, SEQ_IN_INDEX", &[Some(database), Some(table)])?;
                for r in rows {
                    match d.indexes.iter_mut().find(|i| i.name == r[0]) {
                        Some(i) => i.columns.push(r[1].clone()),
                        None => d.indexes.push(IndexDef { name: r[0].clone(), columns: vec![r[1].clone()], unique: r[2] == "0" }),
                    }
                }
                let rows = self.strings(
                    Some(database),
                    "SELECT k.CONSTRAINT_NAME, k.COLUMN_NAME, k.REFERENCED_TABLE_NAME, k.REFERENCED_COLUMN_NAME, r.DELETE_RULE, r.UPDATE_RULE FROM information_schema.KEY_COLUMN_USAGE k JOIN information_schema.REFERENTIAL_CONSTRAINTS r ON r.CONSTRAINT_SCHEMA = k.CONSTRAINT_SCHEMA AND r.CONSTRAINT_NAME = k.CONSTRAINT_NAME WHERE k.TABLE_SCHEMA = ? AND k.TABLE_NAME = ? ORDER BY k.CONSTRAINT_NAME, k.ORDINAL_POSITION",
                    &[Some(database), Some(table)],
                )?;
                for r in rows {
                    match d.fkeys.iter_mut().find(|f| f.name == r[0]) {
                        Some(f) => {
                            f.columns.push(r[1].clone());
                            f.ref_columns.push(r[3].clone());
                        }
                        None => d.fkeys.push(FkDef { name: r[0].clone(), columns: vec![r[1].clone()], ref_table: r[2].clone(), ref_columns: vec![r[3].clone()], on_delete: r[4].clone(), on_update: r[5].clone() }),
                    }
                    // MySQL makes an index of the same name for each foreign key: the key's own.
                    let name = r[0].clone();
                    d.indexes.retain(|i| i.name != name);
                }
            }
            Kind::Postgres => {
                let rel = self.table_ref(database, table);
                let r = Some(rel.as_str());
                let pk = self.strings(Some(database), "SELECT a.attname, c.conname FROM pg_constraint c JOIN pg_attribute a ON a.attrelid = c.conrelid AND a.attnum = ANY(c.conkey) WHERE c.conrelid = $1::regclass AND c.contype = 'p'", &[r])?;
                d.pk_name = pk.first().map(|p| p[1].clone()).unwrap_or_default();
                let rows = self.strings(
                    Some(database),
                    "SELECT a.attname, format_type(a.atttypid, a.atttypmod), CASE WHEN a.attnotnull THEN 'NO' ELSE 'YES' END, COALESCE(pg_get_expr(d.adbin, d.adrelid), ''), CASE WHEN a.attidentity <> '' OR COALESCE(pg_get_expr(d.adbin, d.adrelid), '') LIKE 'nextval(%' THEN 'YES' ELSE 'NO' END \
                     FROM pg_attribute a LEFT JOIN pg_attrdef d ON d.adrelid = a.attrelid AND d.adnum = a.attnum WHERE a.attrelid = $1::regclass AND a.attnum > 0 AND NOT a.attisdropped ORDER BY a.attnum",
                    &[r],
                )?;
                for c in rows {
                    let auto = c[4] == "YES";
                    let default = if c[3].starts_with("nextval(") { String::new() } else { c[3].clone() };
                    d.columns.push(ColDef { name: c[0].clone(), ty: c[1].clone(), nullable: c[2] == "YES", default, pk: pk.iter().any(|p| p[0] == c[0]), auto, was: Some(c[0].clone()) });
                }
                let rows = self.strings(
                    Some(database),
                    "SELECT ic.relname, CASE WHEN i.indisunique THEN 'YES' ELSE 'NO' END, array_to_string(ARRAY(SELECT pg_get_indexdef(i.indexrelid, k + 1, true) FROM generate_subscripts(i.indkey, 1) AS k ORDER BY k), ',') \
                     FROM pg_index i JOIN pg_class ic ON ic.oid = i.indexrelid WHERE i.indrelid = $1::regclass AND NOT i.indisprimary AND NOT EXISTS (SELECT 1 FROM pg_constraint c WHERE c.conindid = i.indexrelid AND c.contype = 'u' AND false) ORDER BY 1",
                    &[r],
                )?;
                for i in rows {
                    d.indexes.push(IndexDef { name: i[0].clone(), unique: i[1] == "YES", columns: split_list(&i[2]) });
                }
                let rows = self.strings(
                    Some(database),
                    "SELECT conname, array_to_string(ARRAY(SELECT a.attname FROM unnest(conkey) WITH ORDINALITY k(n, o) JOIN pg_attribute a ON a.attrelid = conrelid AND a.attnum = k.n ORDER BY k.o), ','), confrelid::regclass::text, \
                     array_to_string(ARRAY(SELECT a.attname FROM unnest(confkey) WITH ORDINALITY k(n, o) JOIN pg_attribute a ON a.attrelid = confrelid AND a.attnum = k.n ORDER BY k.o), ','), confdeltype::text, confupdtype::text FROM pg_constraint WHERE conrelid = $1::regclass AND contype = 'f' ORDER BY 1",
                    &[r],
                )?;
                for f in rows {
                    d.fkeys.push(FkDef { name: f[0].clone(), columns: split_list(&f[1]), ref_table: f[2].trim_matches('"').to_string(), ref_columns: split_list(&f[3]), on_delete: pg_rule(&f[4]), on_update: pg_rule(&f[5]) });
                }
            }
            Kind::Sqlite => {
                let sql = self.strings(None, "SELECT sql FROM sqlite_master WHERE name = ?", &[Some(table)])?.into_iter().next().and_then(|r| r.into_iter().next()).unwrap_or_default();
                let auto_table = sql.to_uppercase().contains("AUTOINCREMENT");
                for c in self.strings(None, &format!("PRAGMA table_info({})", quote_std(table)), &[])? {
                    let pk = c[5] != "0";
                    let default = if c[4] == "NULL" { String::new() } else { c[4].clone() };
                    d.columns.push(ColDef { name: c[1].clone(), ty: c[2].clone(), nullable: c[3] == "0" && !pk, default, pk, auto: pk && auto_table && c[2].eq_ignore_ascii_case("INTEGER"), was: Some(c[1].clone()) });
                }
                for i in self.strings(None, &format!("PRAGMA index_list({})", quote_std(table)), &[])? {
                    // Indexes the table's own UNIQUE and PRIMARY KEY made are not designed apart.
                    if i.get(3).is_some_and(|o| o != "c") || i[1].starts_with("sqlite_autoindex") {
                        continue;
                    }
                    let cols: Vec<String> = self.strings(None, &format!("PRAGMA index_info({})", quote_std(&i[1])), &[])?.into_iter().map(|r| r[2].clone()).collect();
                    d.indexes.push(IndexDef { name: i[1].clone(), columns: cols, unique: i[2] == "1" });
                }
                for f in self.strings(None, &format!("PRAGMA foreign_key_list({})", quote_std(table)), &[])? {
                    let id = format!("fk_{}_{}", table, f[0]);
                    match d.fkeys.iter_mut().find(|k| k.name == id) {
                        Some(k) => {
                            k.columns.push(f[3].clone());
                            k.ref_columns.push(f[4].clone());
                        }
                        None => d.fkeys.push(FkDef { name: id, columns: vec![f[3].clone()], ref_table: f[2].clone(), ref_columns: vec![f[4].clone()], on_update: f[5].clone(), on_delete: f[6].clone() }),
                    }
                }
            }
        }
        Ok(d)
    }

    fn col_sql(&self, c: &ColDef, inline_pk: bool) -> String {
        let mut s = format!("{} {}", self.q(&c.name), c.ty);
        match self.kind {
            Kind::Sqlite if inline_pk => s = format!("{} INTEGER PRIMARY KEY AUTOINCREMENT", self.q(&c.name)),
            Kind::Postgres if c.auto => s.push_str(" GENERATED BY DEFAULT AS IDENTITY"),
            _ => {}
        }
        if !c.nullable && !inline_pk {
            s.push_str(" NOT NULL");
        }
        if !c.default.trim().is_empty() && !(c.auto && self.kind != Kind::MySql) {
            s.push_str(&format!(" DEFAULT {}", c.default.trim()));
        }
        if c.auto && self.kind == Kind::MySql {
            s.push_str(" AUTO_INCREMENT");
        }
        s
    }

    fn fk_sql(&self, f: &FkDef) -> String {
        let cols: Vec<String> = f.columns.iter().map(|c| self.q(c)).collect();
        let refs: Vec<String> = f.ref_columns.iter().map(|c| self.q(c)).collect();
        let target = if self.kind == Kind::Postgres { self.table_ref("", &f.ref_table) } else { self.q(&f.ref_table) };
        let mut s = format!("CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({})", self.q(&f.name), cols.join(", "), target, refs.join(", "));
        for (word, rule) in [("DELETE", &f.on_delete), ("UPDATE", &f.on_update)] {
            if !rule.is_empty() && rule != "NO ACTION" {
                s.push_str(&format!(" ON {word} {rule}"));
            }
        }
        s
    }

    fn index_sql(&self, database: &str, table: &str, i: &IndexDef) -> String {
        let cols: Vec<String> = i.columns.iter().map(|c| self.q(c)).collect();
        format!("CREATE {}INDEX {} ON {} ({})", if i.unique { "UNIQUE " } else { "" }, self.q(&i.name), self.table_ref(database, table), cols.join(", "))
    }

    fn drop_index_sql(&self, database: &str, table: &str, name: &str) -> String {
        match self.kind {
            Kind::MySql => format!("DROP INDEX {} ON {}", self.q(name), self.table_ref(database, table)),
            Kind::Postgres => match table.split_once('.') {
                Some((schema, _)) => format!("DROP INDEX {}.{}", quote_std(schema), quote_std(name)),
                None => format!("DROP INDEX {}", quote_std(name)),
            },
            Kind::Sqlite => format!("DROP INDEX {}", quote_std(name)),
        }
    }

    /// The whole `CREATE TABLE` of a design, under `name`.
    fn create_sql_of(&self, database: &str, name: &str, d: &TableDef) -> String {
        let pks: Vec<&ColDef> = d.columns.iter().filter(|c| c.pk).collect();
        // SQLite numbers a row only as `INTEGER PRIMARY KEY AUTOINCREMENT`, written in the column.
        let inline = self.kind == Kind::Sqlite && pks.len() == 1 && pks[0].auto;
        let mut lines: Vec<String> = d.columns.iter().map(|c| format!("    {}", self.col_sql(c, inline && c.pk))).collect();
        if !pks.is_empty() && !inline {
            lines.push(format!("    PRIMARY KEY ({})", pks.iter().map(|c| self.q(&c.name)).collect::<Vec<_>>().join(", ")));
        }
        for f in &d.fkeys {
            lines.push(format!("    {}", self.fk_sql(f)));
        }
        format!("CREATE TABLE {} (\n{}\n)", self.table_ref(database, name), lines.join(",\n"))
    }

    /// The statements that make the table `new` - from nothing, or from `old` as it is now.
    pub fn design_script(&self, database: &str, old: Option<&TableDef>, new: &TableDef) -> Vec<String> {
        let mut out = Vec::new();
        let Some(old) = old else {
            out.push(self.create_sql_of(database, &new.name, new));
            for i in &new.indexes {
                out.push(self.index_sql(database, &new.name, i));
            }
            return out;
        };
        let t = self.table_ref(database, &old.name);
        let renamed = |c: &ColDef| c.was.as_ref().is_some_and(|w| *w != c.name);
        let kept = |o: &ColDef| new.columns.iter().find(|c| c.was.as_deref() == Some(o.name.as_str()));
        if self.kind == Kind::Sqlite {
            // SQLite adds and renames columns; anything more is the table made anew.
            let changed = old.columns.iter().any(|o| match kept(o) {
                None => true,
                Some(c) => c.ty != o.ty || c.nullable != o.nullable || c.default != o.default || c.pk != o.pk || c.auto != o.auto,
            }) || new.fkeys != old.fkeys
                || new.columns.iter().any(|c| c.was.is_none() && (c.pk || c.auto || (!c.nullable && c.default.trim().is_empty())));
            if changed {
                let tmp = format!("_phai_new_{}", old.name);
                out.push("PRAGMA foreign_keys = OFF".into());
                out.push("BEGIN".into());
                out.push(self.create_sql_of(database, &tmp, new));
                let pairs: Vec<(&str, &str)> = new.columns.iter().filter_map(|c| Some((c.name.as_str(), c.was.as_deref()?))).filter(|(_, w)| old.columns.iter().any(|o| o.name == *w)).collect();
                if !pairs.is_empty() {
                    out.push(format!("INSERT INTO {} ({}) SELECT {} FROM {}", quote_std(&tmp), pairs.iter().map(|p| quote_std(p.0)).collect::<Vec<_>>().join(", "), pairs.iter().map(|p| quote_std(p.1)).collect::<Vec<_>>().join(", "), t));
                }
                out.push(format!("DROP TABLE {t}"));
                out.push(format!("ALTER TABLE {} RENAME TO {}", quote_std(&tmp), quote_std(&new.name)));
                for i in &new.indexes {
                    out.push(self.index_sql(database, &new.name, i));
                }
                out.push("COMMIT".into());
                out.push("PRAGMA foreign_keys = ON".into());
                return out;
            }
            for c in new.columns.iter().filter(|c| renamed(c)) {
                out.push(format!("ALTER TABLE {t} RENAME COLUMN {} TO {}", quote_std(c.was.as_deref().unwrap_or_default()), quote_std(&c.name)));
            }
            for c in new.columns.iter().filter(|c| c.was.is_none()) {
                out.push(format!("ALTER TABLE {t} ADD COLUMN {}", self.col_sql(c, false)));
            }
        } else {
            // Foreign keys and indexes that go or change come off first: columns can then move.
            for f in old.fkeys.iter().filter(|f| !new.fkeys.contains(f)) {
                out.push(match self.kind {
                    Kind::MySql => format!("ALTER TABLE {t} DROP FOREIGN KEY {}", self.q(&f.name)),
                    _ => format!("ALTER TABLE {t} DROP CONSTRAINT {}", self.q(&f.name)),
                });
            }
            let old_pk: Vec<&str> = old.columns.iter().filter(|c| c.pk).map(|c| c.name.as_str()).collect();
            let new_pk_was: Vec<Option<&str>> = new.columns.iter().filter(|c| c.pk).map(|c| c.was.as_deref()).collect();
            let pk_changed = old_pk.len() != new_pk_was.len() || old_pk.iter().zip(&new_pk_was).any(|(o, n)| Some(*o) != *n);
            if pk_changed && !old_pk.is_empty() {
                out.push(match self.kind {
                    Kind::MySql => format!("ALTER TABLE {t} DROP PRIMARY KEY"),
                    _ => format!("ALTER TABLE {t} DROP CONSTRAINT {}", self.q(&old.pk_name)),
                });
            }
            for o in &old.columns {
                if kept(o).is_none() {
                    out.push(format!("ALTER TABLE {t} DROP COLUMN {}", self.q(&o.name)));
                }
            }
            for c in &new.columns {
                let Some(was) = c.was.as_deref() else {
                    out.push(format!("ALTER TABLE {t} ADD COLUMN {}", self.col_sql(c, false)));
                    continue;
                };
                let Some(o) = old.columns.iter().find(|o| o.name == was) else { continue };
                if was != c.name {
                    out.push(format!("ALTER TABLE {t} RENAME COLUMN {} TO {}", self.q(was), self.q(&c.name)));
                }
                let differs = c.ty != o.ty || c.nullable != o.nullable || c.default != o.default || c.auto != o.auto;
                if !differs {
                    continue;
                }
                match self.kind {
                    Kind::MySql => out.push(format!("ALTER TABLE {t} MODIFY COLUMN {}", self.col_sql(c, false))),
                    _ => {
                        let n = self.q(&c.name);
                        if c.ty != o.ty {
                            out.push(format!("ALTER TABLE {t} ALTER COLUMN {n} TYPE {} USING {n}::{}", c.ty, c.ty));
                        }
                        if c.nullable != o.nullable {
                            out.push(format!("ALTER TABLE {t} ALTER COLUMN {n} {} NOT NULL", if c.nullable { "DROP" } else { "SET" }));
                        }
                        if c.default != o.default {
                            out.push(if c.default.trim().is_empty() { format!("ALTER TABLE {t} ALTER COLUMN {n} DROP DEFAULT") } else { format!("ALTER TABLE {t} ALTER COLUMN {n} SET DEFAULT {}", c.default.trim()) });
                        }
                        if c.auto != o.auto {
                            out.push(if c.auto { format!("ALTER TABLE {t} ALTER COLUMN {n} ADD GENERATED BY DEFAULT AS IDENTITY") } else { format!("ALTER TABLE {t} ALTER COLUMN {n} DROP IDENTITY IF EXISTS") });
                        }
                    }
                }
            }
            let new_pk: Vec<String> = new.columns.iter().filter(|c| c.pk).map(|c| self.q(&c.name)).collect();
            if pk_changed && !new_pk.is_empty() {
                out.push(format!("ALTER TABLE {t} ADD PRIMARY KEY ({})", new_pk.join(", ")));
            }
            for f in new.fkeys.iter().filter(|f| !old.fkeys.contains(f)) {
                out.push(format!("ALTER TABLE {t} ADD {}", self.fk_sql(f)));
            }
        }
        // Indexes: those gone or changed dropped, the new ones made.
        for i in old.indexes.iter().filter(|i| !new.indexes.contains(i)) {
            out.push(self.drop_index_sql(database, &old.name, &i.name));
        }
        for i in new.indexes.iter().filter(|i| !old.indexes.contains(i)) {
            out.push(self.index_sql(database, &new.name, i));
        }
        if new.name != old.name && self.kind != Kind::Sqlite {
            out.push(match self.kind {
                Kind::MySql => format!("RENAME TABLE {t} TO {}", self.table_ref(database, &new.name)),
                _ => format!("ALTER TABLE {t} RENAME TO {}", quote_std(new.name.rsplit('.').next().unwrap_or(&new.name))),
            });
        }
        out
    }
}

/// MySQL gives a string default bare (`abc`); SQL wants it quoted (`'abc'`) - numbers and
/// expressions as they are.
fn mysql_default(d: &str, ty: &str) -> String {
    if d.is_empty() || d.eq_ignore_ascii_case("NULL") {
        return String::new();
    }
    let t = ty.to_lowercase();
    let textual = t.contains("char") || t.contains("text") || t.contains("enum") || t.contains("set");
    if textual && !d.starts_with('\'') { format!("'{}'", d.replace('\'', "''")) } else { d.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn col(name: &str, ty: &str, pk: bool, auto: bool) -> ColDef {
        ColDef { name: name.into(), ty: ty.into(), nullable: !pk, pk, auto, was: None, ..Default::default() }
    }

    #[test]
    fn scripts_in_each_kind_s_words() {
        let mut d = TableDef { name: "orders".into(), ..Default::default() };
        d.columns = vec![col("id", "INT", true, true), col("customer_id", "INT", false, false)];
        d.indexes = vec![IndexDef { name: "ix_customer".into(), columns: vec!["customer_id".into()], unique: false }];
        d.fkeys = vec![FkDef { name: "fk_customer".into(), columns: vec!["customer_id".into()], ref_table: "customers".into(), ref_columns: vec!["id".into()], on_delete: "CASCADE".into(), on_update: "NO ACTION".into() }];
        let my = Connection { kind: Kind::MySql, ..Default::default() }.design_script("shop", None, &d);
        assert_eq!(my[0], "CREATE TABLE `shop`.`orders` (\n    `id` INT NOT NULL AUTO_INCREMENT,\n    `customer_id` INT,\n    PRIMARY KEY (`id`),\n    CONSTRAINT `fk_customer` FOREIGN KEY (`customer_id`) REFERENCES `customers` (`id`) ON DELETE CASCADE\n)");
        assert_eq!(my[1], "CREATE INDEX `ix_customer` ON `shop`.`orders` (`customer_id`)");
        let pg = Connection { kind: Kind::Postgres, ..Default::default() }.design_script("shop", None, &d);
        assert!(pg[0].contains("\"id\" INT GENERATED BY DEFAULT AS IDENTITY NOT NULL"), "{}", pg[0]);
        // A change: a column renamed, one added, the key's rule changed.
        let mut old = d.clone();
        for c in old.columns.iter_mut() {
            c.was = Some(c.name.clone());
        }
        let mut new = old.clone();
        new.columns[1].name = "buyer_id".into();
        new.columns.push(col("note", "VARCHAR(100)", false, false));
        new.fkeys[0].on_delete = "SET NULL".into();
        new.fkeys[0].columns = vec!["buyer_id".into()];
        let alter = Connection { kind: Kind::MySql, ..Default::default() }.design_script("shop", Some(&old), &new);
        assert_eq!(alter[0], "ALTER TABLE `shop`.`orders` DROP FOREIGN KEY `fk_customer`");
        assert!(alter.contains(&"ALTER TABLE `shop`.`orders` RENAME COLUMN `customer_id` TO `buyer_id`".to_string()), "{alter:?}");
        assert!(alter.contains(&"ALTER TABLE `shop`.`orders` ADD COLUMN `note` VARCHAR(100)".to_string()), "{alter:?}");
        assert!(alter.iter().any(|s| s.contains("ADD CONSTRAINT `fk_customer` FOREIGN KEY (`buyer_id`)") && s.ends_with("ON DELETE SET NULL")), "{alter:?}");
        assert_eq!(mysql_default("abc", "varchar(10)"), "'abc'");
        assert_eq!(mysql_default("0", "int"), "0");
    }

    /// SQLite end to end: two tables made from designs, a foreign key and an index between them,
    /// then a change SQLite cannot ALTER - the table made anew with its rows kept.
    #[test]
    fn sqlite_designs_made_and_changed() {
        let path = std::env::temp_dir().join(format!("phai-design-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let c = Connection { host: path.to_string_lossy().to_string(), kind: Kind::Sqlite, ..Default::default() };
        let mut cust = TableDef { name: "customers".into(), ..Default::default() };
        cust.columns = vec![col("id", "INTEGER", true, true), col("name", "TEXT", false, false)];
        for s in c.design_script("main", None, &cust) {
            c.query(None, &s).unwrap();
        }
        let mut ord = TableDef { name: "orders".into(), ..Default::default() };
        ord.columns = vec![col("id", "INTEGER", true, true), col("customer_id", "INTEGER", false, false), col("total", "REAL", false, false)];
        ord.indexes = vec![IndexDef { name: "ix_orders_customer".into(), columns: vec!["customer_id".into()], unique: false }];
        ord.fkeys = vec![FkDef { name: "fk".into(), columns: vec!["customer_id".into()], ref_table: "customers".into(), ref_columns: vec!["id".into()], on_delete: "CASCADE".into(), on_update: "NO ACTION".into() }];
        for s in c.design_script("main", None, &ord) {
            c.query(None, &s).unwrap();
        }
        c.query(None, "INSERT INTO customers (name) VALUES ('ไผ่'); INSERT INTO orders (customer_id, total) VALUES (1, 99.5);").unwrap();
        let now = c.table_def("main", "orders").unwrap();
        assert_eq!(now.columns.len(), 3);
        assert!(now.columns[0].pk && now.columns[0].auto);
        assert_eq!(now.indexes, ord.indexes);
        assert_eq!(now.fkeys[0].ref_table, "customers");
        assert_eq!(now.fkeys[0].on_delete, "CASCADE");
        // total's type changed and a column dropped: SQLite cannot ALTER that - made anew.
        let mut new = now.clone();
        new.columns[2].ty = "NUMERIC".into();
        new.columns.push(ColDef { name: "note".into(), ty: "TEXT".into(), nullable: true, ..Default::default() });
        let script = c.design_script("main", Some(&now), &new);
        assert!(script.iter().any(|s| s.starts_with("CREATE TABLE \"_phai_new_orders\"")), "{script:?}");
        c.query(None, &script.join(";\n")).unwrap();
        let after = c.table_def("main", "orders").unwrap();
        assert_eq!(after.columns.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), ["id", "customer_id", "total", "note"]);
        assert_eq!(after.columns[2].ty, "NUMERIC");
        assert_eq!(after.indexes.len(), 1, "the index made again");
        let g = c.query(None, "SELECT customer_id, total FROM orders").unwrap();
        assert_eq!(g.rows, vec![vec!["1".to_string(), "99.5".to_string()]], "the rows kept");
        let _ = std::fs::remove_file(&path);
    }
}
