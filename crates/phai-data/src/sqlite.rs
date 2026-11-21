//! SQLite: a database in a file, through the SQLite library itself (rusqlite, the library built
//! in). Statements run one after another; each result kept as text, as the grid shows it.

use rusqlite::types::ValueRef;

use crate::wire::Result1;

fn err(e: rusqlite::Error) -> String {
    e.to_string()
}

pub(crate) fn open(path: &str) -> Result<rusqlite::Connection, String> {
    rusqlite::Connection::open(path).map_err(err)
}

/// The SQLite library's version, the file opened (and made, when it is new) to be sure.
pub fn version(path: &str) -> Result<String, String> {
    let c = open(path)?;
    c.query_row("SELECT sqlite_version()", [], |r| r.get::<_, String>(0)).map_err(err)
}

fn text(v: ValueRef) -> Option<String> {
    match v {
        ValueRef::Null => None,
        ValueRef::Integer(i) => Some(i.to_string()),
        ValueRef::Real(f) => Some(f.to_string()),
        ValueRef::Text(t) => Some(String::from_utf8_lossy(t).to_string()),
        ValueRef::Blob(b) => Some(format!("0x{}", b.iter().map(|x| format!("{x:02X}")).collect::<String>())),
    }
}

fn run(stmt: &mut rusqlite::Statement, params: &[Option<&str>], max: usize) -> Result<(Result1, bool), String> {
    let values = rusqlite::params_from_iter(params.iter().map(|p| p.map(str::to_string)));
    let n = stmt.column_count();
    if n == 0 {
        let affected = stmt.execute(values).map_err(err)? as u64;
        return Ok((Result1 { affected, ..Default::default() }, false));
    }
    let columns: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();
    let mut rows = Vec::new();
    let mut cut = false;
    let mut q = stmt.query(values).map_err(err)?;
    while let Some(row) = q.next().map_err(err)? {
        if rows.len() >= max {
            cut = true;
            break;
        }
        rows.push((0..n).map(|i| row.get_ref(i).ok().and_then(text)).collect());
    }
    Ok((Result1 { columns, rows, affected: 0 }, cut))
}

/// Every statement of `sql`, in order, and each one's result.
pub fn query(path: &str, sql: &str, max: usize) -> Result<Vec<(Result1, bool)>, String> {
    let c = open(path)?;
    let mut out = Vec::new();
    let mut batch = rusqlite::Batch::new(&c, sql);
    while let Some(mut stmt) = batch.next().map_err(err)? {
        out.push(run(&mut stmt, &[], max)?);
    }
    Ok(out)
}

/// One statement with `?` marks and `params` in their places.
pub fn execute(path: &str, sql: &str, params: &[Option<&str>], max: usize) -> Result<(Result1, bool), String> {
    execute_on(&open(path)?, sql, params, max)
}

/// [`execute`] on a database already open - a transaction's.
pub(crate) fn execute_on(c: &rusqlite::Connection, sql: &str, params: &[Option<&str>], max: usize) -> Result<(Result1, bool), String> {
    let mut stmt = c.prepare(sql).map_err(err)?;
    run(&mut stmt, params, max)
}
