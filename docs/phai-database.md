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

## AI agents and private data

The rule is in docs/standards.md ("Data and AI"). This is how phai-database carries it out.
Encrypting the file is not enough on its own: an AI asks the engine, and the engine answers in
plain text.

| Part | What | Stage |
|---|---|---|
| Read-only connections | `Database::open_read_only`; every write is refused at the engine, not only in the UI | 2 |
| Limits per connection | Most rows per answer; a deadline per query (checked in the executor's loop, so a runaway join stops); most memory for sorts | 2 |
| Safe mode | `PRAGMA phai_safe = on` for a connection: `UPDATE` / `DELETE` without `WHERE` refused; `DROP`, `ALTER` and `VACUUM` need a confirmation token that only the person's UI gives | 3 |
| Audit log | The table `_phai_audit` (time, user, statement, rows touched), appended in the same transaction as the change, so a rolled-back change leaves no false line | 3 |
| A backup first | `db.backup_to(path)` (an online copy, page by page) before an AI session that may write | 3 |
| Column labels | The table `_phai_columns` (table, column, label: `personal` / `secret`, mask), plus masks for a national ID, phone, e-mail and address, and a custom pattern | 3 |
| Masking | Only the rule is stored, in `_phai_columns`; a masked copy is never stored. The real value stays in the file, under the file's encryption, and the mask is applied as each answer is made, for a connection opened with the role `ai` or `guest`. Such a connection may not use a labelled column in `WHERE`, `ORDER BY`, `GROUP BY` or a join either. Otherwise `WHERE phone = '0812345678'` answering "one row" or "none" would let it test guesses until it learns the real value. The owner can allow exact-match search per column, with a limit on lookups per minute | 3 |
| Column encryption | `secret` columns stored as AES-256-GCM BLOBs under a column key kept apart from the file's password (DPAPI or a second password). With the key, values read and write as normal; without it, they read as `[encrypted]`. Equality search on such a column needs a deterministic mode (AES-SIV), which reveals which rows are equal, so it is off unless asked for | 4.5 |
| Users and roles | `_users.phdb`; the role `ai` by default has `SELECT` on the tables chosen, no `secret` columns, 1,000 rows and 5 seconds, and no DDL | 5 |

Safe when:
- **A test agent tries every write as the `ai` user**, through the API and through the server, and
  every one is refused.
- **Masked and encrypted values never reach the AI**, in any column, `JOIN`, `UNION`, subquery or
  error message. The test greps every answer for the real values.
- **`_phai_audit` matches what happened**, through crashes as in stage 3's crash test.

Limits to state plainly in the guide:
- **`_phai_audit` and `_phai_columns` are ordinary tables in the file.** Another SQLite program
  can read or change them, so the labels and masks hold only through phai-database. Against
  other programs, the file needs whole-file encryption.
- **A column-encrypted value is a BLOB to other tools**, and stays secret from them too.
- **On one computer, without the server, the audit log is a record, not a proof.** Someone who
  may write to the file may also change the log. Only on the server (stage 5), where users can't
  touch `_phai_*`, is it tamper-resistant.
- **Prompt injection inside the data can't be stopped by the engine.** The read-only role and safe
  mode only limit what a fooled AI can do.

## Large tables: 10 million rows and more

The file format is not the limit:
- Page numbers are 32-bit, so a file can hold up to 2^32 - 2 pages. That is 16 TB with 4 KB pages,
  or 281 TB with 64 KB pages.
- Rowids are 64-bit.

Ten million rows of about 100 bytes are a file of about 1 GB, plus its indexes. A 4 KB leaf page
holds about 35 such rows, and an interior page points to about 400 children. A lookup by key is
therefore 4 pages deep, and the top 3 levels stay in the cache, so it costs about one read from
the disk. The engine has to be built for this from stage 1. These rules make it so:

1. **Memory has a budget, not the table's size.** The page cache is a fixed number of pages:
   `cache_size`, 2,000 pages (8 MB) by default, settable. It is evicted by a clock or LRU policy.
   Nothing ever loads a whole table. Peak memory is tested: it stays under the budget plus a small
   fixed overhead, whatever the row count.
2. **Results stream.** Every executor node is a pull iterator, and a row is produced only when it
   is asked for. `LIMIT 50` on 10 million rows reads about 50 rows. The server (stage 5) sends rows
   as they come, never a whole result set at once.
3. **Sorting spills to disk.** `ORDER BY`, `GROUP BY`, `DISTINCT` and `CREATE INDEX` sort in
   memory up to `temp_memory` (64 MB by default). Above that, they write sorted runs to a temporary
   file and merge them. The temporary file is encrypted when the database is, and deleted at once.
4. **The planner uses indexes and knows sizes.**
   - It seeks instead of scanning when a `WHERE` or `ORDER BY` matches an index.
   - It uses covering indexes, so a query that needs only indexed columns never reads the table.
   - `ANALYZE` writes `sqlite_stat1`, as SQLite does, so the join order and index choice follow the
     real row counts.
   - `EXPLAIN QUERY PLAN` shows what was chosen.
5. **Joins without an index don't go quadratic.** Nested loops are used when an index exists. A
   hash join (spilling to disk like the sort) is used when one doesn't, so two 10-million-row
   tables don't make 10^14 comparisons.
6. **Loading fast.**
   - Inside one transaction, rows with rising rowids are appended to the right-most leaf without
     a full search and balance.
   - `CREATE INDEX` on a full table sorts the keys and builds the B-tree bottom up, which is much
     faster than inserting one by one.
   - The guide says: wrap a bulk load in one `BEGIN … COMMIT`, and create the indexes afterwards.
7. **Paging by key, not by `OFFSET`.** `LIMIT 50 OFFSET 9000000` must step past 9 million rows.
   `WHERE id > last_seen ORDER BY id LIMIT 50` seeks straight there. Studio's grid and the server's
   browsing use keyset paging. `OFFSET` still works, but it is documented as slow on big tables.
8. **`COUNT(*)` reads the smallest index.** It has no row counter to cheat with: the format has
   none, and keeping one would break compatibility. On 10 million rows that is roughly a second.
   Studio shows "about N rows" from `sqlite_stat1` at once, and the exact count when asked.
9. **WAL and journal stay bounded.** The WAL auto-checkpoints at 1,000 pages. `journal_size_limit`
   trims the files after big transactions. A transaction that changes millions of rows is allowed,
   but the guide recommends batches of 10,000 to 100,000 rows.
10. **Space comes back without doubling the disk.** `VACUUM` needs room for a full copy, which is
    hard on a 50 GB file. Incremental vacuum (`auto_vacuum = INCREMENTAL`) gives pages back a few
    at a time. Studio offers it when the freelist is large.
11. **Big values.** BLOBs and long texts go into overflow chains. The default maximum value is
    1 GB, as in SQLite, and it can be set. Reading a big BLOB can stream (`blob_open`), so a 500 MB
    video needn't be in memory at once.
12. **All sizes are 64-bit in our code.** Counts and file offsets are `u64`, and rowids `i64`. Page
    numbers are `u32`, as the format says. Nothing breaks at 2^31 rows or 2 GB of file.

On the web, browsers cap storage, and a phone may allow only a few hundred MB to a page. A table
of 10 million rows belongs on a computer or behind the server (stage 5). The web build checks
`navigator.storage.estimate()` and says plainly when the data won't fit.

### Inside Studio, for big tables

Today the data grid fills a ListView with every row, and phai-data stops at `MAX_ROWS = 1000`
(`phai-data/src/lib.rs:100`). For big tables:
- **The grid gets a virtual mode** (`LVS_OWNERDATA`). It asks for only the rows on the screen,
  and a cursor fetches them by keyset paging, a page ahead and a page behind.
- **Sorting a column** becomes `ORDER BY` in the engine, using an index when there is one, not a
  sort of the rows in memory.
- **The filter row** becomes `WHERE`.
- **The status bar** shows "about 10,000,000 rows" from `sqlite_stat1`.
- **The 1,000-row cap stays** for engines that can't page cheaply. phai-database's tables page
  instead.
- On the web, phai-web's grid does the same: it draws only the visible rows.

### Benchmarks for big tables (`bench_db`, from stage 2)

A generated table of 10 million rows (and one run at 100 million), with
`id INTEGER PRIMARY KEY`, a Thai name, a price, a date and an index on the name. Every result is
measured against SQLite on the same computer and written down here:

| What | Target |
|---|---|
| Load 10M rows in one transaction | within 3× of SQLite |
| Create the index on the name after loading | within 3× of SQLite |
| Look up by `id` | under 1 ms once the cache is warm |
| Look up a name through the index | under 1 ms once the cache is warm |
| The first screen of the grid, sorted by an indexed column | under 50 ms |
| A jump to the middle of the grid | under 50 ms |
| `ORDER BY price` (no index) with `temp_memory` 64 MB | finishes, with peak memory under 64 MB + cache + 32 MB |
| `GROUP BY` the month, `SUM(price)` | within 3× of SQLite |
| `COUNT(*)` | within 2× of SQLite |
| Peak memory over the whole run | under the cache budget + 32 MB, at 10M and at 100M rows alike |

## Stages

Each stage works on its own and is used before the next starts. The rules for large tables
apply from the stage that brings each part: streaming and the cache budget from stage 1; spilling
sorts, the planner's indexes and keyset paging from stage 2; fast loading and bottom-up
`CREATE INDEX` from stage 3; WAL limits from stage 4.

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
unbounded memory. Reading all of a 10-million-row file stays under the cache budget.

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
- Studio's Server Explorer and data grid can browse a `.phdb` with phai-database alone. The grid's
  virtual mode scrolls a 10-million-row table smoothly.
- `bench_db`'s read targets (lookups, the first screen, a jump, the sort with spilling, `GROUP BY`,
  `COUNT(*)`) are met at 10M rows.

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
- `bench_db`'s load and `CREATE INDEX` targets are met at 10M rows, and a 100M-row run finishes
  within the memory budget.

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
