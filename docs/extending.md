# Language servers and External Tools

Two ways to make Phai Studio do more without writing code. The Thai guide covers the same
ground: `docs/help/studio.html`, sections *Language Server สำหรับไฟล์ชนิดอื่น* and *เครื่องมือภายนอก*.

## Language servers for other kinds of file

Rust files always use rust-analyzer. Any other kind of file can get errors as you type,
completion, hover and Go To Definition (F12) from its own language server, if one is installed.

1. Install the server, e.g. `cargo install taplo-cli` for TOML.
2. **Tools > Options > Text Editor > Language Servers**, **Add**.
3. **File extensions**: one or more, e.g. `.sql, .psql`.
4. **Command line**: what starts the server on stdio, e.g. `taplo lsp stdio`. A program that is
   not on PATH needs its whole path, in `"..."` if it has spaces.
5. **OK**, then open a file of that kind. The status bar says *Language server ready*.

| Language | Extensions | Command line | Install |
|---|---|---|---|
| TOML | `.toml` | `taplo lsp stdio` | `cargo install taplo-cli` |
| SQL | `.sql` | `sqls` | `go install github.com/sqls-server/sqls@latest` |
| Python | `.py` | `pyright-langserver --stdio` | `npm install -g pyright` |

Each server runs once per project folder (where Cargo.toml is, else the file's folder). One
that will not start is reported on the status bar and in Output (source *Tools*), and is not
tried again until Options changes it. The list is saved as `language_servers` in Studio's
`settings.txt`, so Export/Import in Options carries it to another computer.

Tip: taplo knows Cargo.toml's keys once it has the schema. Put this on the file's first line:
`#:schema https://json.schemastore.org/cargo.json`.

## External Tools

**Tools > External Tools...** puts programs you use often on the Tools menu, as Visual Studio
does: one choice runs it, and what it prints and its exit code go to the Output pane.

Each tool has a **Title** (its menu item), a **Command** (the program; **...** finds it),
**Arguments**, an **Initial directory** (empty: the project's folder) and **Use Output window**
(off: it runs in a window of its own). The **›** buttons put in these placeholders:

| Placeholder | Becomes |
|---|---|
| `$(FilePath)` | the open file, its whole path |
| `$(FileDir)` | the open file's folder |
| `$(FileName)` | the open file's name |
| `$(ProjectDir)` | the project's folder (where Cargo.toml is) |
| `$(SelectedText)` | what is chosen in the editor |

Capitals do not matter. Put quotes around a path that may hold spaces, e.g. a "rustfmt this
file" tool: command `rustfmt`, arguments `--edition 2021 "$(FilePath)"`. A tool that uses the
open file saves it first; when the tool ends, open files it changed are loaded again (Ctrl+Z
undoes that). The list is saved as `external_tools` in `settings.txt`; the menu holds 24 at most.

## Checking them

- `cargo test -p phai-studio --bin phai-studio extend` - placeholders, picking a server by
  extension, the saved lists.
- `cargo test -p phai-editor lsp` - command lines split into words, programs found on PATH.
- `PHAI_LSP_TRY=LINE:COL` asks the open file's server for hover, then completion, and prints
  the answers; `PHAI_PRINT_DIAGS=1` prints its diagnostics; `PHAI_SHOT_SCREEN=out.png` takes a
  picture with the tip or list showing.
- `--options 10`, `--external-tools`, `--tools-menu` and `--run-tool N`, with `PHAI_SHOT`,
  photograph the two dialogs and the menu, and run a tool.
