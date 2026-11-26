# The standards Phai implements itself

Checked 2026-11-26 by reading the code against each spec. Specs that still change were looked up
that day. Review once a year, and whenever a part listed here is changed.

This covers only the code Phai writes itself. Where a library does the work (png, miniz_oxide,
serde_json, rusqlite / SQLite, native-tls / SChannel, sha2, pbkdf2), its authors keep it to the
spec. The rule for which is which: we write what is Phai's own surface and has a clear spec. We
leave crypto, TLS and crash-safe storage to libraries.

Each part's summary is followed by a table of what still falls short, most important first.

## The list to fix, most important first

| # | Where | What | Kind |
|---|---|---|---|
| 1 | `phai-data/src/pg.rs:122` | SCRAM without SASLprep: a password with `ำ` (U+0E33) never logs in to PostgreSQL | Thai users can't connect |
| 2 | `phai-ui/src/xlsx.rs:83` | `"น่าน"` is safe, but `"Nan"`, `"inf"`, `"2E10"`, `"+66..."` and 16+ digit IDs become numbers; NaN makes Excel ask to repair the file | data changed / file broken |
| 3 | `phai-ui/src/grid.rs:1841` | A cell is read through a 1024-character buffer: CSV / Excel export cuts longer text silently | data lost |
| 4 | `phai-data/src/server.rs:365,389,355` | Before the key is checked: header lines of any length, a buffer as large as Content-Length says (to 64 MB), a thread per connection without a limit | memory exhausted by anyone who can reach it |
| 5 | `phai-data/src/lib.rs:119,751` | SQL NULL shown as the text `NULL`; saving turns a real text `"NULL"` into SQL NULL | data changed |
| 6 | `phai-data/src/wire.rs:472,603` | Every MySQL column read as UTF-8 text, the charset ignored: BLOB / BINARY corrupted | data changed |
| 7 | `studio/src/update.rs:67` | No `digest` from GitHub → the setup runs unchecked. It should refuse instead. Nothing checks the signature before running | security |
| 8 | `studio/src/ai.rs:544` | The agent's `read`, `search` and `fetch` tool calls are allowed without asking: a prompt injection can read outside the project or send data out | security |
| 9 | `phai-form/src/web.rs:1008` | The service worker caches every GET: error pages and `/phai-data/` answers. Offline, people see stale data, and private rows stay on the phone | correctness / privacy |
| 10 | `phai-data/src/wire.rs:317` | Unknown password plugins fall back to `mysql_native_password`: MariaDB `client_ed25519` / PARSEC fail as a misleading "Access denied" | interoperability |

Smaller points are in each part's table.

## Data and AI: Phai's own rule, for every database Studio opens

This is not a published spec. It is the rule Phai holds itself to, written down on 2026-11-26.
It covers phai-database, MySQL, PostgreSQL and SQLite alike, wherever Studio's Copilot, an ACP
agent or a program connects with AI in the loop.

**What encryption does and does not do.** Encrypting the file (phai-database stage 4.5) protects
the file while it is closed: copied, backed up to a cloud, or left on a lost USB stick. Once a
program opens it with the key, the engine answers SQL in plain text. An AI never reads the file's
bytes; it asks the engine, and the engine decrypts. So the file being encrypted does not stop an
AI that is allowed to query.

Three layers, used together:

| Against | Layer |
|---|---|
| The file leaving the computer | Whole-file encryption |
| An AI changing or deleting data, or being tricked into it | Its own user with fewer rights, safe mode, limits, an audit log, a backup first |
| An AI seeing private data | Columns labelled personal and masked for it; columns encrypted with a second key it doesn't hold |

The rules:
1. **An AI connects as its own user**, never as the owner. By default it may read only (`SELECT`),
   only the tables chosen for it, at most 1,000 rows per answer, for at most 5 seconds per query.
2. **Safe mode for that user.** `UPDATE` / `DELETE` without `WHERE` are refused. `DROP`,
   `ALTER` and `TRUNCATE` wait for a person to confirm.
3. **Every statement an AI sends is logged**: when, which user, what, and how many rows. Studio
   shows the log as "what the AI did".
4. **A backup before an AI may write.** The file, or a savepoint on a server, is kept so that its
   work can be undone in one step.
5. **Personal columns are labelled** (national ID, phone, address, salary, grades) and masked for
   the AI's user: `1-2345-xxxxx-xx-x`, `08x-xxx-5678`. This helps with PDPA; it does not by itself
   make a program comply.
6. **Studio never sends rows to an AI service without asking** the person first, and never sends
   personal columns unmasked.
7. **Data is data.** Text read from a table can contain instructions written to fool an AI
   (prompt injection). No engine can stop that, because to the engine it is only text. The AI's
   client must treat it as data, and rules 1-2 keep the harm small if it is fooled.

What none of this can do:
- **Tell an AI from a person.** Protection comes from rights, not from guessing who is
  connecting.
- **Stop a person who may see the data from pasting it into an AI themselves.**

Where it stands today:
- **Rule 6 is broken in Studio.** `ai.rs:544` allows the agent's `read`, `search` and `fetch`
  without asking; see item 8 of the list above.
- **Rules 1-5 are designed for phai-database** (docs/phai-database.md, "AI agents and private
  data"). For MySQL and PostgreSQL, Studio should create or use a separate read-only user for AI
  and say so in the connection dialog.

## HTTP/1.1: the data server (`phai-data/src/server.rs`) and Studio's phone/web server (`studio/src/web.rs`)

Spec: RFC 9110 (semantics) and RFC 9112 (HTTP/1.1), both June 2022, current.
Scope on purpose: one request per connection (`Connection: close`), Content-Length bodies only,
JSON answers. It listens on `127.0.0.1:8791` by default. Behind nginx or Caddy when put on a
network.

Done well:
- The key compare is constant-time.
- A wrong key waits one second, taken one request at a time.
- 413 above 64 MB.
- `no-store` on answers.
- No keep-alive, so no request smuggling.
- Requests name a binding from a fixed list, never SQL.
- The web server refuses `..`.
- A Host check stops DNS rebinding.

| Gap | Line | Fix |
|---|---|---|
| Request line and headers unbounded; 30 s timeout per read, so slowloris works | `server.rs:362-371` | Cap a line (8 KB) and the header count (100), with one deadline for the whole head |
| Body allocated before the key is checked | `server.rs:389` | Check `Authorization` first; read the body only for a good key |
| A thread per connection, no limit | `server.rs:355` | A fixed pool, or a counter that answers 503 |
| Access key: 12 digits from `RandomState` | `server.rs:271-276` | `OsRng` (rand is already a dependency), 128 bits |
| No `Origin` lets a request through | `web.rs:655` | Also check `Sec-Fetch-Site` |
| Bad Content-Length becomes 0; no write timeout | `server.rs:376` | Answer 400; set a write timeout |
| No Content-Security-Policy | `phai-form/src/web.rs:917` | Add a CSP header, since the page loads only its own files |

Test with: curl, a browser, and a slowloris script against a local server.

## MySQL / MariaDB client protocol (`phai-data/src/wire.rs`)

Spec: MySQL Client/Server Protocol (dev.mysql.com) and the MariaDB protocol pages (mariadb.com).
Changing: MySQL 9.0 removed `mysql_native_password`, and `caching_sha2_password` is the default.
MariaDB 11.6 added PARSEC, which is not the default yet but is meant to be.

Done well:
- Handshake v10.
- TLS by SChannel.
- Password plugins: `caching_sha2` (fast and full), `sha256_password`, and clear text only over TLS.
- Auth switch.
- Several result sets.
- Prepared statements, with the binary types decoded correctly.
- Packets over 16 MB split and joined both ways.
- `LOCAL INFILE` refused.
- utf8mb4.

| Gap | Line | Fix |
|---|---|---|
| BLOB / BINARY read as lossy UTF-8 | `:472`, `:603` | Charset 63 means bytes; show them as `0x…` as SQLite does |
| Unknown plugin → native password | `:317` | A clear error naming the plugin; later `client_ed25519` and PARSEC |
| The RSA key is taken from the server without TLS | `:348` | Ask for it only over TLS, or with a key the user gave |
| `SERVER_MORE_RESULTS` ignored in `execute`: a `CALL` leaves the connection out of step | `execute` | Read on to the last result |
| Statement not closed after an error mid-rows | `:672` | Close it on every path |
| Length read off the wire can overflow / allocate unbounded | `:119`, `:378` | `checked_add`; a cap |
| `5.5.5-` prefix of MariaDB's version kept | `:246` | Strip it |

Test with: MySQL 8.0, 8.4, 9.x and MariaDB 10.11, 11.4 and 11.8 (Docker), each with its default
user and with TLS on and off.

## PostgreSQL frontend/backend protocol v3 (`phai-data/src/pg.rs`)

Spec: PostgreSQL docs, "Frontend/Backend Protocol". SCRAM is RFC 5802 / RFC 7677. SASLprep is
RFC 4013, with Unicode NFKC. In March 2026 PostgreSQL also made its SASLprep stricter for ASCII
control characters.

Done well:
- SSLRequest, and an error instead of plain text when TLS is refused.
- MD5.
- SCRAM-SHA-256, with the server's signature checked and an RFC 7677 test vector.
- Simple and extended query.
- Reads on to ReadyForQuery after an error.
- NULL (length -1).

| Gap | Line | Fix |
|---|---|---|
| **No SASLprep**: `น้ำ` is U+0E19 U+0E49 U+0E33 but the server hashes U+0E19 U+0E49 U+0E4D U+0E32 (checked with Python's NFKC) | `:122` | NFKC (the `unicode-normalization` crate, MIT/Apache) plus RFC 4013's tables, and the raw password when prohibited characters are found, as PostgreSQL does |
| AuthenticationOk accepted before SCRAM finishes | `:195` | Refuse it once SCRAM has started |
| Message length unbounded / negative | `:154` | Reject < 4 and > a cap |
| Indexing that panics on a short message | `:203`, `:264`, `:284` | `get(..)` with an error |
| COPY FROM STDIN waits 60 s | `:314` | Send CopyFail at once |
| No SSPI / GSS; no SCRAM-SHA-256-PLUS | `:241` | Later; say so in the error |

Test with: PostgreSQL 14 to 18 (Docker), users whose passwords are Thai (`น้ำ`, `กำ`), ASCII and empty.

## SQLite (`phai-data/src/sqlite.rs`)

The file format is SQLite's own, through rusqlite `bundled`, so there is no spec of ours to keep.

| Gap | Line | Fix |
|---|---|---|
| A BLOB is shown as `0x…` but bound back as TEXT | `:28`, `:33` | Bind `0x…` back as a BLOB |

## XLSX: SpreadsheetML (`phai-ui/src/xlsx.rs`)

Spec: ECMA-376 / ISO/IEC 29500, a strict subset. One sheet, inline strings, styles, a frozen
pane and an autoFilter. The zip inside follows PKWARE APPNOTE: stored entries, CRC-32, and the
UTF-8 name flag (bit 11). There is no ZIP64, so the limit is 4 GB, which is fine for a sheet.

Done well:
- Escaping of `& < > "` and control characters.
- ฿ formats quoted.
- Sheet names cleaned and cut to 31 characters.
- `_FilterDatabase` defined.
- Columns past Z.
- Leading-zero phone numbers kept as text.

| Gap | Line | Fix |
|---|---|---|
| Rust's `parse::<f64>` takes `nan`, `inf` and `infinity` in any case, and exponents: `Nan` (the province) gives `<v>NaN</v>`, so Excel repairs the file; `2E10`, `+66…` and long IDs change | `:83` | Accept only `-?digits(.digits)?` with grouping commas, reject non-finite, and keep 16+ digits as text |
| 2026-02-30 accepted (becomes 2 March) | `:54` | Check the month's length |
| A `+07:00` offset dropped silently | `:65` | Keep the local time; say so in the docs |

Test with: Excel, LibreOffice Calc and Google Sheets opening a sheet with the cells above.

## CSV (`phai-ui/src/grid.rs:1671`)

Spec: RFC 4180 (2005, informational). What Excel actually reads matters more.

Done well:
- A UTF-8 BOM, so Excel reads Thai.
- CRLF line ends.
- Quoting of `,` `"` `\n` `\r` with doubled quotes.

| Gap | Line | Fix |
|---|---|---|
| Cells cut at 1023 characters (also xlsx) | `grid.rs:1841` | Ask the list for the length (grow until it fits), or read the grid's own data |
| `= + - @` at the start can run as a formula in Excel | `:1674` | Prefix `'` for values that start with these and aren't numbers (OWASP CSV injection) |

## JSON (`phai-data/src/server.rs`, `phai-web/src/data.rs`, `phai-form/src/web.rs`)

Spec: RFC 8259 (2017), current. A hand-written parser and writers. Elsewhere serde_json is used.

Done well:
- Writers escape `"`, `\` and below 0x20.
- The parser has a depth limit (32).
- Trailing data is rejected.
- Surrogate pairs are decoded.

| Gap | Line | Fix |
|---|---|---|
| A high surrogate followed by any `\u` is merged unchecked | `server.rs:225-228` | Merge only when the second one is DC00-DFFF; otherwise U+FFFD |

Test with: JSONTestSuite (the `y_` and `n_` files).

## QR code (`studio/src/qr.rs`)

Spec: ISO/IEC 18004. Byte mode, level M, versions 1-6.

Done well, with nothing to fix:
- Reed-Solomon and the format bits tested against the standard's tables.
- Interleaving.
- All 8 masks scored.
- No ECI for non-ASCII text, but the comment limits it to addresses.

## Base64 (`phai-form/src/winforms.rs:55`)

Spec: RFC 4648, decode only (.resx pictures). It is lenient, and gives None on a bad character.
Nothing to fix.

## PHAIPACK: Phai's own setup format (`phai-pack/src/lib.rs`)

Our own format, so this file is its spec. Inflate is capped at the declared size, and paths with
`..`, `/` or `:` are refused.

| Gap | Line | Fix |
|---|---|---|
| Manifest `key=value` without escaping: a newline or `\|` in a name shifts fields | `:46`, `:51`, `:69` | Escape `\n`, `\|` and `\` |
| No checksum on the deflate data | `:105` | CRC-32 or SHA-256 per file, checked before writing |
| `*at + n > end` can wrap and panic | `:185` | `checked_add` |

## LSP client (`phai-editor/src/lsp.rs`)

Spec: LSP 3.18 is current (3.19 in progress). We use the 3.17 subset; later versions keep older
clients working. JSON-RPC 2.0.

Done well:
- Content-Length counted in bytes, so Thai text is framed right.
- UTF-16 positions offered and converted.
- Server-to-client requests answered.
- Messages held until `initialize` is answered (for other servers).

| Gap | Line | Fix |
|---|---|---|
| `initialized` sent before the `initialize` answer | `:251` | Wait for it, as for other servers. rust-analyzer tolerates it |
| Unknown server requests get `null`, not -32601 | `studio/src/main.rs:3662` | Answer `MethodNotFound` |
| `exit` without waiting for `shutdown`'s answer | `:541` | Wait briefly, then kill |

## ACP: Agent Client Protocol (`studio/src/acp.rs`, `ai.rs`)

Spec: agentclientprotocol.com, still young. A v2 is proposed that widens
`session/request_permission` beyond tool calls (a required `title`, an optional `subject`). Check
again before each release.

Done well:
- Paths confined to the project.
- `-32601` for unknown methods.
- Edits reviewed before they are made.
- The permission option picked by its `kind`.

| Gap | Line | Fix |
|---|---|---|
| `read`, `search`, `think` and `fetch` allowed without asking | `ai.rs:544` | Allow `read` / `search` only inside the project; ask for `fetch` (or allow it in Act only) |

## Updates (`studio/src/update.rs`)

GitHub's release `digest` (SHA-256) has existed for every uploaded asset since June 2025.

Done well:
- HTTPS.
- The URL must be this project's releases.
- SHA-256 checked when given.

| Gap | Line | Fix |
|---|---|---|
| No digest → runs unchecked | `:67` | Refuse, as with a wrong digest |
| Only a hash from the same place as the file | `main.rs:7033` | Check the setup's Authenticode signature (WinVerifyTrust) and its signer once releases are signed |
| `curl -L` without `--proto =https` | `:65` | Add `--proto =https --proto-redir =https` |

## PWA: Web App Manifest and Service Worker (`phai-form/src/web.rs`)

Specs: W3C Web App Manifest and Service Workers, both living. Safari's rules change with iOS
releases.

Done well:
- Title HTML-escaped and manifest strings JSON-escaped.
- `</style` broken in user CSS.
- Network first.
- A new cache name per build, with `skipWaiting` / `clients.claim` and old caches deleted.
- Icons 192 / 512 / maskable, plus `apple-touch-icon`.

| Gap | Line | Fix |
|---|---|---|
| Every GET cached: errors and `/phai-data/` answers | `:1008` | Cache only `r.ok` answers, and never `/phai-data/` |
| The maskable icon is the plain icon filling the square | `:986`, `studio/src/web.rs:398` | Pad it to the safe zone (80%) |
| `<html lang="th">` fixed | `:916` | From the project's language |
