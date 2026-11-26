# Blueprint: phai-database, Phai's own database engine

Written 2026-11-26 at the owner's word. It replaces docs/phdb-plan.md, which put redb and
GlueSQL under Phai. This one is Phai's own engine, written in Rust from published specs. Nothing
is built yet.

## What it is

- **An engine in the program, not a server to install.** A database is one file, `shop.phdb`,
  that a person can copy, back up or send.
- **The file is byte for byte a SQLite 3 file.** That is the blueprint, and it is complete, public
  domain and tested by billions of devices. DB Browser for SQLite, Adminer, Python and any SQLite
  tool open a `.phdb`.
- **It is marked as Phai's.** The header's application ID (offset 68) is set to `0x50484149`
  ("PHAI"), so Studio knows the file is its own. Check that the number isn't taken in SQLite's
  `magic.txt`.
- **It understands the SQL people know.** SQLite's dialect, plus the MySQL forms Thai PHP
  programmers write: `` `name` ``, `AUTO_INCREMENT`, `LIMIT 10, 20`, `VARCHAR(50)`, `DATETIME` and
  `SHOW TABLES`.
- **Later, a server.** It speaks MySQL's wire protocol, so HeidiSQL, DBeaver, PHP `mysqli` and in
  the end phpMyAdmin connect to it.
- **Optionally encrypted**, in SQLCipher 4's page format, so DB Browser opens it with the password.
- **The same engine in a web page.** It compiles to WebAssembly with phai-web, and the file lives
  in the browser's Origin Private File System.

### Why not a format of our own

A new format would have no blueprint for its bytes and nothing to check our answers against. Data
put in it would also be locked in. With SQLite's format, real SQLite (already in the project
through rusqlite) is the referee at every stage. Phai's own part is the engine: the code, the
SQL, the encryption, the server and Studio around it.

## The blueprints

| Layer | Spec | Where |
|---|---|---|
| File: header, pages, B-trees, records, overflow, freelist, pointer maps | Database File Format | sqlite.org/fileformat2.html |
| Commit and crash recovery | Atomic Commit In SQLite; the journal part of the file format | sqlite.org/atomiccommit.html |
| WAL and its index | Write-Ahead Logging; WAL-mode File Format | sqlite.org/wal.html, sqlite.org/walformat.html |
| Locks between programs | File Locking And Concurrency In SQLite Version 3 | sqlite.org/lockingv3.html |
| SQL grammar | SQL As Understood By SQLite (railroad diagrams) | sqlite.org/lang.html |
| Types, affinity, comparison, collation | Datatypes In SQLite | sqlite.org/datatype3.html |
| Built-in functions | Core / Aggregate / Date and Time Functions | sqlite.org/lang_corefunc.html, lang_aggfunc.html, lang_datefunc.html |
| Query planning | The SQLite Query Optimizer Overview; Query Planning | sqlite.org/optoverview.html, sqlite.org/queryplanner.html |
| How SQLite tests itself | How SQLite Is Tested | sqlite.org/testing.html |
| Encryption | SQLCipher Design (version 4 defaults) | zetetic.net/sqlcipher/design |
| Server wire protocol | MySQL Client/Server Protocol; MariaDB protocol pages | dev.mysql.com/doc/dev/mysql-server/latest/PAGE_PROTOCOL.html, mariadb.com |
| What phpMyAdmin asks | `information_schema`, `SHOW`, system variables | MySQL Reference Manual, chapters on INFORMATION_SCHEMA and SHOW |
| Background | CMU 15-445 lectures; *Architecture of a Database System* (Hellerstein, Stonebraker, Hamilton); *Database Internals* (Petrov) | Free online / books |

SQLite's code is public domain too, so reading it to understand a spec's corner is allowed.
MySQL's and MariaDB's code is GPL. **Never read it to write ours**: write the server from the
protocol documents only.

## How it is built

A new crate, `crates/phai-database`, MIT like the other runtime libraries. It has no C, and its
only dependencies are RustCrypto for the encryption.

```
phai-database/src/
  lib.rs          Database::open / create, execute, query, transaction, prepare
  vfs.rs          the file underneath: Windows (LockFileEx), memory, the browser's OPFS; fault injection for tests
  file/
    header.rs     the 100-byte header
    varint.rs     1-9 byte big-endian varints
    record.rs     serial types 0-13+, the record header
    pager.rs      page cache, dirty pages, reading and writing through vfs
    btree.rs      cursors; table B-trees (rowid) and index B-trees; insert, delete, balance
    overflow.rs   payload spill (the U, X, M, K formulas)
    freelist.rs   trunk and leaf pages
    ptrmap.rs     auto-vacuum pointer maps
    journal.rs    rollback journal: write, hot-journal recovery
    wal.rs        WAL frames, checksums, salts, checkpoint, the -shm index
    lock.rs       SHARED / RESERVED / PENDING / EXCLUSIVE on the lock bytes
    schema.rs     sqlite_schema on page 1, the schema cookie
  sql/
    lexer.rs      tokens; both dialects
    parser.rs     recursive descent following lang.html; Pratt parsing for expressions
    ast.rs
    mysql.rs      MySQL forms rewritten to the AST (backticks, AUTO_INCREMENT, LIMIT a,b, types, SHOW)
    resolve.rs    names, columns, `*`, aliases
    plan.rs       index choice, join order (nested loops first)
    exec.rs       iterators (scan, seek, filter, project, sort, group, join, limit)
    expr.rs       evaluation, three-valued NULL logic, affinity
    func.rs       core, aggregate and date/time functions
    collate.rs    BINARY, NOCASE, RTRIM
  crypt.rs        the SQLCipher 4 page codec
  server/         (stage 5) the MySQL protocol server; apps/phai-dbserver runs it
```

The executor is a tree of iterators, not SQLite's bytecode machine. In Rust it is simpler to write
and to test, and the file stays the same either way. A bytecode stage can come later if the
iterators are too slow.

## Rules that keep data safe

1. **Never write what we don't understand.** If a file uses something we don't support yet (a
   virtual table such as FTS5 or R-tree, a schema format above 4, an unknown text encoding, a
   collation we don't have), it opens **read-only** and says why. It never half-writes.
2. **Every commit goes through the journal or the WAL**, with the order of writes and flushes the
   spec gives: journal, flush, database, flush, delete the journal.
3. **After every test that writes, SQLite's `PRAGMA integrity_check` must answer `ok`.**
4. **Stage gates.** A stage is not given to people until its "safe when" list passes. Until stage 3
   passes, Studio keeps using rusqlite for writing.
5. **A copy before the first write.** The first time phai-database writes to a file made by
   another program, it keeps `shop.phdb.bak` beside it.

## Stages

Each stage works on its own and is used before the next starts.

### Stage 1: read any SQLite file

Header, pager, B-tree cursors, records, overflow, the schema. Tables and indexes are read through
a Rust API (`db.table("customers").rows()`), with no SQL yet.

Done when:
- A corpus of real files reads exactly as rusqlite reads them, row for row and byte for byte. The
  corpus includes files from DB Browser, Python, an Android app, Studio's own samples, Thai text
  and BLOBs, and pages of 512 to 65536 bytes.
- Files with auto-vacuum, WAL mode or a hot journal are read correctly, or refused with a clear
  reason.

Safe when: `cargo-fuzz` feeds broken pages for hours and gets only errors, never a panic and never
unbounded memory.

### Stage 2: SQL that reads

Lexer, parser (both dialects), resolver, planner, executor:
- `SELECT` with `WHERE`, `ORDER BY`, `GROUP BY`, `HAVING`, `LIMIT`
- joins (inner and left)
- subqueries, `IN` and `EXISTS`
- `CASE` and `CAST`
- the core, aggregate and date functions
- views
- index use when a `WHERE` matches an index

Done when:
- sqllogictest's `select1` to `select5` and the `random/` and `index/` SELECT files pass (target:
  all, with each skip written down and justified).
- A differential test runs random queries on random data in both phai-database and SQLite, and the
  results are identical.
- Studio's Server Explorer and data grid can browse a `.phdb` with phai-database alone.

Safe when: the parser is fuzzed as the pages were, and a query plan never reads past a page.

### Stage 3: writing, with the rollback journal

- `CREATE TABLE`, `CREATE INDEX`, `DROP`, `ALTER TABLE ADD COLUMN` / `RENAME`
- `INSERT` (with `AUTO_INCREMENT` / `AUTOINCREMENT` and `sqlite_sequence`), `UPDATE`, `DELETE`
- `UNIQUE`, `NOT NULL`, `CHECK`, `DEFAULT` and foreign keys (when `PRAGMA foreign_keys` is on)
- `BEGIN` / `COMMIT` / `ROLLBACK` / `SAVEPOINT`
- B-tree balance, the freelist and `VACUUM`

Done when:
- sqllogictest passes in full, including its INSERT, UPDATE and DELETE files.
- Files written by phai-database are read by SQLite, and files written by SQLite are written to
  by phai-database, with `integrity_check` `ok` both ways.
- Studio's data grid edits a `.phdb` through phai-database, behind a "preview" switch.

Safe when:
- **The crash test passes.** The fault-injection VFS stops the process at every write and flush in
  10,000 random transactions, and also tears writes (half a page written) and fails flushes. After
  each stop, the file opened again, by us and by SQLite, holds exactly the last committed
  transaction, and `integrity_check` answers `ok`.
- Thai text, empty strings, NULL and BLOBs come back exactly as written.

### Stage 4: WAL and more than one user

- WAL frames and checksums, the `-shm` index, checkpoints (passive, full, restart, truncate)
- readers that don't wait for the writer
- locks that SQLite programs on the same file respect, and that we respect in turn
- `busy_timeout`

Done when:
- phai-database and SQLite open the same file at the same time, one writing and one reading, in
  both directions, for an hour of random work, with no wrong answer and `integrity_check` `ok`.

Safe when: the crash test of stage 3 passes in WAL mode, with stops during checkpoints too.

### Stage 4.5: encryption

The SQLCipher 4 page codec, in `crypt.rs`:
- AES-256-CBC for each page, with a random IV
- HMAC-SHA512 over the page, so tampering is found
- the key from the password by PBKDF2-HMAC-SHA512 (256,000 iterations)
- the salt in the file's first 16 bytes
- 80 reserved bytes per page

The journal, the WAL and temporary files are encrypted the same way. All primitives come from
RustCrypto (`aes`, `cbc`, `hmac`, `sha2`, `pbkdf2`; MIT / Apache-2.0). None are written by us.

People get:
- **A password**, or a key kept by Windows' DPAPI for the Windows user (phai-data already uses
  DPAPI for saved passwords), so no one has to type it every time.
- **A recovery key** shown once, as text and as a QR code (Studio's own QR encoder), to print and
  keep. A forgotten password with no recovery key means the data is gone, and the UI says so
  plainly before encrypting.
- **Change password / encrypt / decrypt**, made safe against a power cut: written to a new file,
  then swapped.

Done when: files encrypted by phai-database open in SQLCipher (rusqlite's `bundled-sqlcipher` as a
dev-dependency) and in DB Browser for SQLite with the password. SQLCipher's files open in
phai-database.

Safe when:
- The stage 3 crash test passes encrypted, including a stop halfway through a password change.
- A flipped bit anywhere in the file is reported as tampering, never read as data.

### Stage 5: the server, speaking MySQL's protocol

`apps/phai-dbserver`, which also runs inside Studio. A folder of `.phdb` files is served, and each
file is one database (`USE shop` opens `shop.phdb`). Users and their rights live in
`_users.phdb`, with passwords stored as `caching_sha2_password` verifiers.

What it does:
- the handshake v10
- auth with `caching_sha2_password` (fast and full; full only over TLS)
- TLS by SChannel
- `COM_QUERY`, `COM_STMT_PREPARE` / `EXECUTE` / `CLOSE`, `COM_INIT_DB`, `COM_PING` and `COM_QUIT`
- text and binary result sets

Safe by default (the lessons of docs/standards.md):
- It listens on `127.0.0.1`. Other computers are let in only on purpose, with TLS on.
- Head and packet sizes are capped, and a password is checked before any large allocation.
- Connections are limited, with deadlines per connection.
- A wrong password costs time.
- The guide says never to open it to the internet; use a VPN.

Done when:
- HeidiSQL, DBeaver, PHP `mysqli` / PDO and phai-data's own `wire.rs` connect, list, query and edit.
- A MySQL client test (connection, prepared statements, every type) passes against it.

### Stage 6: phpMyAdmin

What MySQL tools expect to find:
- `information_schema`: `SCHEMATA`, `TABLES`, `COLUMNS`, `STATISTICS`, `KEY_COLUMN_USAGE`,
  `TRIGGERS`, `ROUTINES` (empty) and `VIEWS`, as read-only virtual tables over the schema
- the `SHOW` family: `DATABASES`, `TABLES`, `FULL COLUMNS`, `INDEX`, `TABLE STATUS`, `VARIABLES`,
  `GRANTS`, `CREATE TABLE`, `COLLATION`, `CHARSET` and `ENGINES`
- `@@version` and the variables phpMyAdmin reads
- `SET NAMES`

The version string names the engine (`8.0.0-phai-database-0.1`), and the guide never calls it
MySQL.

Done when: phpMyAdmin 5.2 in Docker, logged in to phai-dbserver, can:
- list databases and tables
- browse, search, insert, edit and delete rows
- create a table with its designer
- run SQL
- export to SQL and CSV
- import a file it exported

All of this works with no error on any page.

### After that, when people ask

- Window functions
- Triggers
- `WITH RECURSIVE`
- A Thai collation (`THAI`, dictionary order with leading vowels). Files whose indexes use it
  can't be written by other SQLite tools, so it is only ever used when asked for, and the reason
  is shown when it is chosen.
- Full-text search that splits Thai words
- The bytecode executor, if the iterators are too slow

## Inside Studio and phai-data

- `phai_data::open("shop.phdb")` (and `.db` / `.sqlite`) uses phai-database. Until stage 3 passes,
  writing still goes through rusqlite, after that only when the preview switch is off.
- File > New > Database creates a `.phdb`. The Server Explorer opens it, and its tables drag onto
  forms and data grids as SQLite tables do today.
- Database > Encrypt / Change Password / Recovery Key (stage 4.5).
- Tools > Database Server: start or stop phai-dbserver for a folder, with its address and users
  (stage 5).
- A published web page runs the same engine over OPFS. Its data stays on the phone until it is
  synced with a server.

## Check first, when the work starts

1. **Licences.** sqllogictest's licence (remembered as public domain). The RustCrypto crates'
   (MIT / Apache-2.0). SQLCipher's (BSD-3-Clause; we only test against it, never ship it). Check
   that all of them fit our MIT runtime.
2. **The application ID** `0x50484149` is not in SQLite's `magic.txt`.
3. **SQLCipher 4's exact defaults** from its design page: KDF iterations, HMAC algorithm, the
   page-1 layout and the reserve size. Check them against a file it makes before writing
   `crypt.rs`.
4. **OPFS in the browsers we support.** Synchronous access handles in a worker: Chrome, Edge,
   Safari (iOS) and Firefox.
5. **Speed targets.** With `bench_grid` and 100k rows, measure fill, sort, find and an indexed
   lookup against SQLite, and write the numbers down. The first goal is to be within 3×; then
   close the gap.
6. **The name.** `.phdb` is still free, and the crate name `phai-database` is free on crates.io
   (if it is ever published).

## Tests: the referee at every stage

| Test | What it proves | From |
|---|---|---|
| Differential test against rusqlite | Same answers, same files | Stage 1 |
| sqllogictest | SQL answers match a large known-good corpus | Stage 2 |
| `PRAGMA integrity_check` by SQLite | Our files are well-formed | Stage 3 |
| Crash test (fault-injection VFS) | A stop at any moment loses only the uncommitted transaction | Stage 3 |
| Two programs on one file | Locks and WAL respected both ways | Stage 4 |
| SQLCipher round trip; flipped bits | Encryption compatible; tampering caught | Stage 4.5 |
| MySQL clients and phpMyAdmin | The server is what tools expect | Stages 5-6 |
| `cargo-fuzz` (pages, SQL, packets) | Hostile input gives errors, not crashes | Every stage |
