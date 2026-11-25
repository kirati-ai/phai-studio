# Phai's own database file: `.phdb` (plan for a later project)

Written 2026-11-25 at the owner's word: "write it down, for the next project". Nothing here is
built yet.

## What the owner wants

A database of Phai's own, written in Rust, kept in one file a person can copy - `shop.phdb`.
Today phai-data only connects: MySQL / MariaDB (its own wire protocol), PostgreSQL, a SQLite
file, and the data server. The engine under all of them is someone else's (SQLite is C).

## The file's name: `.phdb`, not `.pdb`

The owner first asked for `xxx.pdb`. Don't use it:

- `.pdb` is Microsoft's Program Database (debug symbols). Rust on Windows writes one beside every
  build (`target\release\score_book.pdb`), so a `shop.pdb` of data would sit among them.
- On a computer with Visual Studio, a double-click on `.pdb` opens Visual Studio.
- It is also the Protein Data Bank's format, and the old Palm database's.

`.phdb` (Phai DataBase) is short, says whose it is, and as far as we know nobody uses it.
Check that again before it ships.

## How: Rust throughout, without writing the risky part ourselves

| Layer | What | Why |
|---|---|---|
| Storage in the `.phdb` file | [redb](https://github.com/cberner/redb) | Pure Rust; transactions; a crash or power cut leaves the file whole; file format stable since 1.0 |
| SQL (`SELECT`, `JOIN`, `GROUP BY` ...) | [GlueSQL](https://github.com/gluesql/gluesql) | Pure Rust; its storage is pluggable (store traits), so redb can sit under it |
| In a web page | GlueSQL over IndexedDB | The same program code runs in a page Studio publishes |
| Phai's own | `phai_data::open("shop.phdb")`; a table bound to the data grid; a table dragged onto a form in Studio; `.phdb` opened, browsed and edited in Studio's data pane | What people using Studio actually get |

Writing B-trees, a write-ahead log, crash recovery and concurrency ourselves would take years
before it is as safe as SQLite, and a bug there loses people's data (a shop's, a school's). The
plan above keeps the file, the name and the experience Phai's, and leaves the dangerous layer
to a library that already got it right.

Later, a pure-Rust SQLite rewrite (Turso) may mature enough to sit under the same API instead.
Programs written against `phai_data::open` would not change.

## Check first, when the project starts

1. **The licences of redb and GlueSQL.** Our runtime libraries are MIT, so theirs must allow
   that. We remember redb as MIT/Apache-2.0 and GlueSQL as Apache-2.0, but we haven't checked.
2. **GlueSQL's store traits in its current release:** how much has to be written to put redb under
   it, and whether a redb storage already exists in its repository.
3. **Speed.** Measure against SQLite with the data grid benchmark (`bench_grid`): filling 100k rows,
   sorting, finding.

## A first prototype is done when

- `phai_data::open("x.phdb")`, then `CREATE TABLE`, `INSERT`, `SELECT ... WHERE ... ORDER BY`,
  `UPDATE` and `DELETE` work and survive closing and reopening.
- A transaction cut off half way (the process killed) leaves the file as it was before it.
- The same test program runs as a web page with the data kept in IndexedDB.
- Studio's Server Explorer opens a `.phdb` like a SQLite file: tables, columns, rows.
