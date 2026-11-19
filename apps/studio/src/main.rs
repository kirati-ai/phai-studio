//! **Phai Studio** - design Windows programs in Rust: drag controls onto a form, and get real
//! Win32 Rust that is small, needs no runtime and shows Thai right.
//!
//! The window: a toolbox on the left, the form on its canvas in the middle, the chosen control's
//! properties on the right. Saving writes the `.phaiform`, the `<name>_ui.rs` generated from it,
//! and - once - the programmer's `<name>.rs`.
//!
//! `phai-studio [form.phaiform] [--dark] [--thai]`; `PHAI_SHOT=out.png` photographs it and ends.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bottom;
mod canvas;
mod explorer;
mod cargo;
mod project;
mod preview;
mod props;
mod text;
mod toolbox;
mod autohide;
mod docking;
mod servers;
mod designer;
mod gitpane;
mod startwin;
mod outline;
mod gotoall;
mod crates;
mod diag;
mod attach;
mod githistory;
mod merge;
mod coverage;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicPtr, Ordering};

use phai_form::codegen;
use phai_form::{snake, FormFile, Kind};
use phai_ui::controls;
use phai_ui::icon::glyph;
use phai_ui::layout::{Anchor, Dock, Item, Layout, Rect};
use phai_ui::text::{self as words, t, tf};
use phai_ui::theme::{self, Theme};
use phai_ui::win::{self, wide};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HFONT};
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HMENU};

use canvas::Canvas;
use props::{Key, Props};

const ID_NEW: i32 = 100;
const ID_OPEN: i32 = 101;
const ID_SAVE: i32 = 102;
const ID_SAVE_AS: i32 = 103;
const ID_EXIT: i32 = 104;
const ID_UNDO: i32 = 110;
const ID_REDO: i32 = 111;
const ID_DELETE: i32 = 112;
const ID_PREVIEW: i32 = 120;
const ID_LIGHT: i32 = 121;
const ID_DARK: i32 = 122;
const ID_THAI: i32 = 123;
const ID_ENGLISH: i32 = 124;
const ID_LANG: i32 = 125;
const ID_THEME_TOGGLE: i32 = 126;
const ID_NEW_PROJECT: i32 = 105;
const ID_BUILD: i32 = 140;
const ID_RUN: i32 = 141;
const ID_STOP: i32 = 142;
const ID_CLOSE: i32 = 106;
const ID_DEBUG: i32 = 143;
const ID_VIEW_TOOLBOX: i32 = 180;
const ID_VIEW_EXPLORER: i32 = 181;
const ID_VIEW_PROPS: i32 = 182;
const ID_VIEW_OUTPUT: i32 = 183;
const ID_SPLIT_SIDE: i32 = 207;
const ID_SPLIT_BOTTOM: i32 = 208;
const ID_SPLIT_TOOLBOX: i32 = 209;
/// The Debug / Release box on the Standard bar.
const ID_CONFIG: i32 = 210;
const ID_SAVE_ALL: i32 = 211;
const ID_NAV_BACK: i32 = 212;
const ID_NAV_FORWARD: i32 = 213;
const ID_LAYOUT_BAR: i32 = 214;
/// The tray the toolbars lie in, each with a grip to drag it.
const ID_REBAR: i32 = 215;
const ID_BUILD_ALL: i32 = 216;
const ID_REBUILD_ALL: i32 = 217;
const ID_CLEAN_ALL: i32 = 218;
const ID_ANALYSE_ALL: i32 = 219;
const ID_REBUILD: i32 = 220;
const ID_CLEAN: i32 = 221;
const ID_ANALYSE: i32 = 222;
/// Start Without Debugging (Ctrl+F5) - the same as Start, Phai having no debugger yet.
const ID_RUN_NODEBUG: i32 = 223;
/// The ▾ at the end of each bar: Add or Remove Buttons.
const ID_CUSTOM_STD: i32 = 224;
const ID_CUSTOM_LAYOUT: i32 = 225;
const ID_STEP_OVER: i32 = 226;
const ID_STEP_INTO: i32 = 227;
const ID_STEP_OUT: i32 = 228;
const ID_TOGGLE_BP: i32 = 229;
const ID_CLEAR_BPS: i32 = 230;
const ID_BREAK_ALL: i32 = 231;
const ID_DEBUG_BAR: i32 = 232;
const BAND_DEBUG: u32 = 3;
const ID_VIEW_SERVERS: i32 = 233;
const ID_ADD_CONNECTION: i32 = 234;
const ID_NEW_QUERY: i32 = 235;
const ID_EXECUTE: i32 = 236;
const ID_ADD_WATCH: i32 = 237;
const ID_BP_CONDITION: i32 = 238;
/// `Evaluate` ids: Watch's own, and data tips from this up.
const EVAL_WATCH: u64 = 1;
/// The Immediate window's question.
const EVAL_IMMEDIATE: u64 = 2;
const EVAL_TIP: u64 = 1000;
/// Posted by the debugger's thread for each thing it says; `LPARAM` is a `Box<phai_debug::Event>`.
const WM_DEBUG: u32 = wm::WM_APP + 0x0E01;
/// Posted by the Events tab: write the method of the event `wparam` names (`props::event_key_of`).
pub const WM_EVENT_METHOD: u32 = wm::WM_APP + 0x0E02;
/// `--design`: opens the Table Designer once the loop runs.
const ID_DESIGN_TEST: i32 = 239;
const ID_PUBLISH: i32 = 240;
const ID_NEXT_ERROR: i32 = 241;
const ID_PREV_ERROR: i32 = 242;
const ID_MOVE_UP: i32 = 243;
const ID_MOVE_DOWN: i32 = 244;
const ID_PEEK: i32 = 245;
const ID_FORMAT_DOC: i32 = 246;
/// Ctrl+K pressed: the next key finishes a chord (Ctrl+K, Ctrl+D). Does nothing alone.
const ID_CHORD: i32 = 247;
/// A chord's second key that finishes none: the status line cleared.
const ID_CHORD_END: i32 = 248;
const ID_FIND_IN_FILES: i32 = 249;
const ID_TASK_LIST: i32 = 250;
const ID_PARAM_INFO: i32 = 251;
const ID_QUICK_ACTIONS: i32 = 252;
const ID_RUN_TESTS: i32 = 253;
const ID_RUN_ALL_TESTS: i32 = 254;
const ID_GIT_CHANGES: i32 = 255;
const ID_START_WINDOW: i32 = 256;
const ID_VIEW_OUTLINE: i32 = 257;
const ID_FOLD: i32 = 258;
const ID_FOLD_ALL: i32 = 259;
const ID_SPLIT: i32 = 260;
const ID_BATCH_BUILD: i32 = 261;
const ID_NEW_WINDOW: i32 = 262;
const ID_CLOSE_PEEK: i32 = 263;
const ID_GO_TO_ALL: i32 = 264;
const ID_BM_TOGGLE: i32 = 265;
const ID_BM_NEXT: i32 = 266;
const ID_BM_PREV: i32 = 267;
const ID_BM_CLEAR: i32 = 268;
const ID_PROJECT_PROPS: i32 = 269;
const ID_MANAGE_CRATES: i32 = 270;
const ID_SNIPPET: i32 = 271;
const ID_CALLS: i32 = 272;
const ID_IMMEDIATE: i32 = 273;
const ID_DIAG_TOOLS: i32 = 274;
const ID_ATTACH: i32 = 275;
const ID_DETACH: i32 = 276;
const ID_BREAKPOINTS_WIN: i32 = 277;
const ID_THREADS_WIN: i32 = 278;
const ID_EXCEPTIONS: i32 = 279;
const ID_GIT_HISTORY: i32 = 280;
const ID_GIT_BLAME: i32 = 281;
const ID_MEMORY_WIN: i32 = 282;
const ID_DATA_BP: i32 = 283;
const ID_DISASSEMBLY: i32 = 284;
const ID_MERGE: i32 = 285;
const ID_COVERAGE: i32 = 286;
const ID_HELP_STUDIO: i32 = 287;
const ID_HELP_RUST: i32 = 288;
const ID_HELP_ERRORS: i32 = 289;
const ID_ABOUT: i32 = 290;
/// Theme > the project's style sheets, one item each; Theme > Edit the one in use.
const ID_THEME_FILE: i32 = 760;
const THEME_FILES_MAX: usize = 20;
const ID_THEME_EDIT: i32 = 790;
/// Project > Add Form: a second window for the project.
const ID_ADD_FORM: i32 = 291;
/// The Memory window's request.
const EVAL_MEMORY: u64 = 3;

/// Exception Settings: the Windows exceptions that can stop the program as soon as they happen.
const EXCEPTIONS: [(u32, &str); 6] = [
    (0xC000_0005, "Access violation"),
    (0xC000_0094, "Integer divide by zero"),
    (0xC000_0095, "Integer overflow"),
    (0xC000_001D, "Illegal instruction"),
    (0xC000_00FD, "Stack overflow"),
    (0xE06D_7363, "C++ exception"),
];
static DESIGN_TEST: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
const BAND_STANDARD: u32 = 1;
const BAND_LAYOUT: u32 = 2;
/// The timer that slides an auto-hidden pane back once the pointer and focus have left it.
const TIMER_SLIDE: usize = 7301;
/// Diagnostic Tools' samples, twice a second while a program runs.
const TIMER_DIAG: usize = 7302;
/// File > Recent: `ID_RECENT + n`, newest first.
const ID_RECENT: i32 = 160;
/// File > New from template: `ID_TEMPLATE + n` in `Template::ALL` order.
const ID_TEMPLATE: i32 = 170;
const RECENT_MAX: usize = 9;
const ID_RELEASE: i32 = 144;
const ID_EDIT_MENUS: i32 = 113;
const ID_CUT: i32 = 115;
const ID_COPY: i32 = 116;
const ID_PASTE: i32 = 117;
const ID_DUPLICATE: i32 = 118;
const ID_SELECT_ALL: i32 = 119;
const ID_FRONT: i32 = 190;
const ID_BACK: i32 = 191;
const ID_LOCK: i32 = 192;
const ID_TAB_ORDER: i32 = 193;
const ID_GO_DEFINITION: i32 = 194;
const ID_RENAME: i32 = 195;
const ID_REFERENCES: i32 = 197;
const ID_COMMENT: i32 = 198;
/// One step of `--lsptest`, posted again and again until it is done.
const ID_LSP_TEST: i32 = 196;
/// Format menu and the Layout bar: `ID_FORMAT + Op as index` in `FORMATS` order.
const ID_FORMAT: i32 = 400;
const FORMATS: [phai_form::arrange::Op; 21] = {
    use phai_form::arrange::Op::*;
    [Lefts, Centers, Rights, Tops, Middles, Bottoms, SameWidth, SameHeight, SameSize, SpaceAcross, SpaceDown, AlignToGrid, SizeToGrid, MoreAcross, LessAcross, NoneAcross, MoreDown, LessDown, NoneDown, CenterAcross, CenterDown]
};

/// `ID_FORMAT` + the place of `op` in `FORMATS`.
fn format_id(op: phai_form::arrange::Op) -> i32 {
    ID_FORMAT + FORMATS.iter().position(|o| *o == op).unwrap_or(0) as i32
}

/// What the Format menu and the Layout bar call each of `FORMATS`.
fn format_names() -> [&'static str; 21] {
    [
        t("Align lefts"), t("Align centers"), t("Align rights"), t("Align tops"), t("Align middles"), t("Align bottoms"),
        t("Same width"), t("Same height"), t("Same size"), t("Space evenly across"), t("Space evenly down"),
        t("Align to grid"), t("Size to grid"), t("More space across"), t("Less space across"), t("No space across"),
        t("More space down"), t("Less space down"), t("No space down"), t("Center across"), t("Center down"),
    ]
}
const ID_EDIT_TOOLBAR: i32 = 114;
/// Toolbox buttons: the pointer, then each kind in `Kind::ALL` order.
const ID_TOOL: i32 = 300;
const ID_THEME_NEW: i32 = 130;
const ID_THEME_IMPORT: i32 = 131;
/// For the properties pane's Theme "…".
pub const ID_THEME_IMPORT_PUB: i32 = ID_THEME_IMPORT;
pub const ID_TOOL_PUB: i32 = ID_TOOL;
/// The toolbox's Menu bar and Toolbar: not controls - the form has one of each, edited in place.
pub const ID_TOOL_MENUBAR: i32 = 380;
pub const ID_TOOL_TOOLBAR: i32 = 381;

/// The kind a toolbox command puts on the form; `None` for the Pointer.
pub fn tool_kind(id: i32) -> Option<Kind> {
    (id > ID_TOOL).then(|| Kind::ALL.get((id - ID_TOOL - 1) as usize).copied()).flatten()
}
const ID_THEME_NONE: i32 = 132;

const ID_TOOLBAR: i32 = 200;
const ID_TOOLBOX: i32 = 201;
const ID_CANVAS: i32 = 202;
const ID_PROPS: i32 = 203;
const ID_STATUS: i32 = 204;
const ID_ERRORS: i32 = 205;
const ID_TABS: i32 = 206;
/// The error list's height when it shows, design pixels.
const ERRORS_HEIGHT: i32 = 140;

/// The docked panes the View menu shows and hides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pane {
    Toolbox,
    Explorer,
    Properties,
    Output,
}

pub struct Doc {
    pub form: FormFile,
    pub path: Option<PathBuf>,
    pub dirty: bool,
    /// A code file's editor, when this tab holds code rather than a form.
    pub code: Option<HWND>,
    /// The file had `\r\n` line ends; they are written back so.
    pub crlf: bool,
}

impl Doc {
    pub fn new(form: FormFile, path: Option<PathBuf>) -> Doc {
        Doc { form, path, dirty: false, code: None, crlf: false }
    }
}

/// An open form that is not the one on the canvas: everything it needs to come back as it was.
/// A table in Results, as Show Table Data shows it: where it is, its columns, its rows as shown.
struct TableView {
    conn: usize,
    db: String,
    table: String,
    columns: Vec<phai_data::Column>,
    rows: Vec<Vec<String>>,
}

/// What a request to rust-analyzer was for.
#[derive(Clone)]
enum Ask {
    Hover,
    Complete,
    Definition,
    /// Peek Definition: the place shown in a window, not opened.
    Peek,
    /// Parameter Info: what the call takes.
    Signature,
    /// Quick Actions: fixes and refactorings to choose from.
    Actions,
    /// Go to All's symbols.
    Symbols,
    /// Document Outline of a code file: its symbols.
    Outline,
    /// Code lenses: the file's symbols, then the uses of each (the round, its line and place).
    LensSymbols,
    LensUses(u64, u32, (u32, u32)),
    /// Call Hierarchy: the function at the caret, then who calls it.
    PrepareCalls,
    Calls(String),
    /// Whether the name at this place in this file may be renamed.
    PrepareRename(PathBuf, (u32, u32)),
    /// The changes that rename `old` to `new`.
    Rename { old: String, new: String },
    References,
}

struct Tab {
    doc: Doc,
    undo: Vec<FormFile>,
    redo: Vec<FormFile>,
    selected: Option<usize>,
}

impl Tab {
    fn empty() -> Tab {
        Tab { doc: Doc::new(FormFile::new("MainWindow"), None), undo: vec![], redo: vec![], selected: None }
    }

    fn label(&self) -> String {
        let name = self.doc.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| t("Untitled").to_string());
        if self.doc.dirty {
            format!("{name} •")
        } else {
            name
        }
    }
}

pub struct Studio {
    pub hwnd: HWND,
    pub font: HFONT,
    pub toolbar: HWND,
    pub toolbox: HWND,
    pub status: HWND,
    pub errors: HWND,
    bottom: Option<Box<bottom::Bottom>>,
    /// Build with `--release`.
    release: bool,
    diagnostics: Vec<cargo::Diagnostic>,
    /// Diagnostic Tools: the running program watched.
    pub sampler: Option<diag::Sampler>,
    /// Code lenses as they come: the round they belong to, and those worked out.
    lens_round: u64,
    lenses: Vec<(u32, (u32, u32), String)>,
    /// The Error List's rows as filtered: each one's diagnostic.
    diag_rows: Vec<usize>,
    /// The last Find All References, as its list shows them.
    references: Vec<phai_editor::lsp::Location>,
    building: bool,
    running: Option<std::process::Child>,
    /// rust-analyzer for the project of the code being edited.
    lsp: Option<phai_editor::lsp::Lsp>,
    /// Requests sent to it, by id: which editor asked, and what for.
    asked: std::collections::HashMap<i64, (HWND, Ask)>,
    pub canvas: Option<Canvas>,
    side: HWND,
    side_split: HWND,
    toolbox_split: HWND,
    /// The edge the toolbox is docked to, and - while it floats - its window.
    pub toolbox_dock: Dock,
    toolbox_float: Option<HWND>,
    /// Design pixels: the toolbox's height when docked at the top or bottom.
    toolbox_h: i32,
    /// The form designer's Layout bar, under the Standard bar.
    layout_bar: HWND,
    /// The tray both bars lie in.
    rebar: HWND,
    /// The Debug bar: shown in the tray while a program is being debugged.
    debug_bar: HWND,
    /// The program under the debugger, whether it is paused, and a build that is to be debugged.
    debug: Option<phai_debug::Session>,
    paused: bool,
    debug_after_build: bool,
    /// Breakpoints by file - those of files not open as well - lines from 1.
    bps: std::collections::HashMap<PathBuf, Vec<u32>>,
    /// Breakpoints turned off in the Breakpoints window: kept, but the debugger is not given them.
    bp_off: Vec<(PathBuf, u32)>,
    /// The Breakpoints window's rows, and whether it is being filled (its ticks then mean nothing).
    bp_rows: Vec<(PathBuf, u32)>,
    bp_filling: bool,
    /// rust-analyzer still loading: its diagnostics held until it is quiet, the last of each file's.
    lsp_loading: bool,
    held_diagnostics: std::collections::HashMap<PathBuf, Vec<phai_editor::lsp::Diagnostic>>,
    /// The last code coverage found: each file's lines and counts.
    coverage: Vec<coverage::FileCoverage>,
    /// What the Memory window shows: an address or a variable.
    memory_at: String,
    /// Git line blame: shown after the caret's line, each line's for the file in front.
    blame_on: bool,
    blame: Vec<String>,
    /// The Threads window's rows: each thread's id.
    thread_ids: Vec<u32>,
    /// The call stack of the last stop, for a double-click in its list.
    frames: Vec<phai_debug::Frame>,
    /// Server Explorer: the data connections.
    pub servers: Option<servers::Servers>,
    /// Watch: the expressions, in order.
    watches: Vec<String>,
    /// Breakpoint conditions by file and line.
    conds: Vec<(PathBuf, u32, String)>,
    /// The data tip asked for, and the editor and words it is for.
    tip: Option<(u64, HWND, String)>,
    /// Split: a second editor of the code file in front, below the first; each shows its own
    /// part, and what is typed in one is in both.
    pub split_view: Option<HWND>,
    /// The split's divider, and the upper pane's share of the room, thousandths.
    split_bar: Option<HWND>,
    split_share: i32,
    /// New Window: the code file in front, also in windows of their own - (window, editor).
    pub windows: Vec<(HWND, HWND)>,
    /// Peek Definition's window: its frame, its editor, the file, whether it was changed.
    pub peek: Option<(HWND, HWND, PathBuf, bool)>,
    /// Document Outline's tree.
    outline: Option<outline::Outline>,
    /// The last Run Tests, as the Tests list shows them.
    test_results: Vec<cargo::TestResult>,
    /// How the References list's count is said: references, or Find in Files' matches.
    found_words: &'static str,
    /// Publish: the project's other files and folders to go with the program.
    publish_include: Vec<String>,
    /// Publish: what to pack once the Release build is done.
    publish_after_build: Option<phai_pack::Manifest>,
    /// The table Results shows, when it shows one (Show Table Data): its rows can be changed.
    table_view: Option<TableView>,
    /// Query files, by path: the connection and database each runs in.
    queries: std::collections::HashMap<PathBuf, (usize, String)>,
    /// The Debug / Release box on the Standard bar.
    config: HWND,
    /// Places in code that F12, a reference or an error went from, for Navigate backward and
    /// forward; `nav_at` is where in them we are.
    nav: Vec<(PathBuf, (usize, usize))>,
    nav_at: usize,
    /// Auto Hide: the toolbox, and the right-hand panes (as one group), each hidden at its edge.
    auto_left: bool,
    auto_right: bool,
    /// Which right-hand panes were showing when the group's pin was pulled.
    right_saved: (bool, bool),
    strip_left: HWND,
    strip_right: HWND,
    /// The auto-hidden pane slid out over the form now.
    sliding: Option<Pane>,
    /// Design pixels: the toolbox's width.
    toolbox_w: i32,
    bottom_split: HWND,
    /// Design pixels: the side panes' width and the bottom pane's height.
    side_w: i32,
    /// The side panes' edge: right, as Visual Studio has them, or wherever dragged.
    pub side_dock: Dock,
    /// The side panes' height when at the top or bottom, design pixels.
    side_h: i32,
    /// The side panes' own window, while they float.
    pub side_float: Option<HWND>,
    bottom_h: i32,
    toolbox_on: bool,
    bottom_on: bool,
    pub props: Option<Props>,
    pub explorer: Option<explorer::Explorer>,
    tabbar: HWND,
    /// Every open form; the active one's slot is stale - its contents live in the fields below.
    tabs: Vec<Tab>,
    active: usize,
    layout: Layout,
    pub doc: Doc,
    undo: Vec<FormFile>,
    redo: Vec<FormFile>,
    pub selected: Option<usize>,
    /// Chosen besides `selected`, in the order chosen.
    pub also: Vec<usize>,
    pub tool: Option<Kind>,
}

static BUILD_TEST: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Find in Files' search: `what` in every text file under `root` - Cargo's `target` and hidden
/// folders left out - as places (line, UTF-16 column, as the language server counts), at most `max`.
fn find_in_tree(root: &Path, what: &str, case: bool, max: usize) -> Vec<phai_editor::lsp::Location> {
    const KINDS: [&str; 9] = ["rs", "toml", "css", "phaiform", "md", "txt", "json", "sql", "html"];
    let mut found = Vec::new();
    let mut dirs = vec![root.to_path_buf()];
    while let Some(d) = dirs.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
            if p.is_dir() {
                if !name.starts_with('.') && name != "target" && name != "node_modules" {
                    dirs.push(p);
                }
                continue;
            }
            if !p.extension().is_some_and(|e| KINDS.contains(&e.to_string_lossy().to_ascii_lowercase().as_str())) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&p) else { continue };
            let text = text.replace("\r\n", "\n");
            let starts: Vec<usize> = std::iter::once(0).chain(text.match_indices('\n').map(|(i, _)| i + 1)).collect();
            for (a, b) in phai_editor::model::find_all(&text, what, case) {
                let line = starts.partition_point(|s| *s <= a) - 1;
                let col = |x: usize| text[starts[line]..x].encode_utf16().count() as u32;
                found.push(phai_editor::lsp::Location { path: p.clone(), start: (line as u32, col(a)), end: (line as u32, col(b.min(text[..].len()))) });
                if found.len() >= max {
                    return found;
                }
            }
        }
    }
    found
}

/// Whether a comment at this line is a task: `// TODO`, `FIXME`, `HACK` or `UNDONE` first in it.
fn is_task(path: &Path, line: u32) -> bool {
    if path.extension().is_none_or(|e| e != "rs") {
        return false;
    }
    thread_local! {
        static CACHE: std::cell::RefCell<(PathBuf, Vec<String>)> = const { std::cell::RefCell::new((PathBuf::new(), vec![])) };
    }
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.0 != path {
            *c = (path.to_path_buf(), std::fs::read_to_string(path).unwrap_or_default().lines().map(str::to_string).collect());
        }
        task_words(c.1.get(line as usize).map(String::as_str).unwrap_or_default())
    })
}

/// A line whose comment starts with a task word.
fn task_words(line: &str) -> bool {
    let Some(i) = line.find("//") else { return false };
    let rest = line[i..].trim_start_matches('/').trim_start_matches('!').trim_start();
    ["TODO", "FIXME", "HACK", "UNDONE"].iter().any(|w| rest.starts_with(w) && !rest[w.len()..].starts_with(|c: char| c.is_alphanumeric()))
}

/// Peek Definition's window: its editor fills it; a change, Escape, or leaving it is the studio's.
extern "system" fn peek_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_SIZE => {
            if let Some(e) = studio().and_then(|s| s.peek.as_ref().filter(|p| p.0 == h).map(|p| p.1)) {
                win::move_to(e, 0, 0, win::loword(l.0 as usize) as i32, win::hiword(l.0 as usize) as i32);
            }
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            let code = win::hiword(w.0);
            if let Some(s) = studio() {
                if code == phai_editor::view::CHANGED {
                    s.peek_changed();
                } else if code == phai_editor::view::ESCAPE {
                    s.close_peek();
                }
            }
            LRESULT(0)
        }
        // Clicked away from: it goes, as Visual Studio's peek closes when left.
        wm::WM_ACTIVATE if win::loword(w.0) == 0 && std::env::var_os("PHAI_SHOT").is_none() => {
            if let Some(s) = studio() {
                let _ = unsafe { wm::PostMessageW(Some(s.hwnd), wm::WM_COMMAND, WPARAM(ID_CLOSE_PEEK as usize), LPARAM(0)) };
            }
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            if let Some(s) = studio() {
                s.close_peek();
            }
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}

/// A code file's window of its own (New Window): its editor fills it, and says what it does to
/// the canvas, as the editor on the canvas does.
extern "system" fn code_window_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_SIZE => {
            if let Some(e) = studio().and_then(|s| s.windows.iter().find(|(f, _)| *f == h).map(|(_, e)| *e)) {
                win::move_to(e, 0, 0, win::loword(l.0 as usize) as i32, win::hiword(l.0 as usize) as i32);
            }
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            if let Some(c) = studio().and_then(|s| s.canvas.as_ref().map(|c| c.hwnd)) {
                unsafe { wm::SendMessageW(c, msg, Some(w), Some(l)) };
            }
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            if let Some(s) = studio() {
                s.windows.retain(|(f, _)| *f != h);
            }
            unsafe {
                let _ = wm::DestroyWindow(h);
            }
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}

/// The pictures a project's forms show, by their paths in it - what Publish takes along unless
/// told otherwise.
fn pictures_of(project: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![project.join("src")];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for p in rd.flatten().map(|e| e.path()) {
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|e| e == "phaiform") {
                if let Ok(f) = std::fs::read_to_string(&p).map_err(|_| ()).and_then(|t| FormFile::parse(&t).map_err(|_| ())) {
                    for c in f.control.iter().filter_map(|c| c.image.clone()) {
                        if !std::path::Path::new(&c).is_absolute() && project.join(&c).exists() && !out.contains(&c) {
                            out.push(c);
                        }
                    }
                }
            }
        }
    }
    out
}

/// What Peek Definition shows: the file and line, then the code from the line before the
/// definition on - to its closing brace at the same indent, at most 14 lines.
fn peek_text(path: &Path, line: usize, text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let first = line.saturating_sub(1);
    let indent = lines.get(line).map(|l| l.len() - l.trim_start().len()).unwrap_or(0);
    let mut last = (line + 12).min(lines.len().saturating_sub(1));
    for (i, l) in lines.iter().enumerate().skip(line + 1).take(12) {
        if l.trim() == "}" && l.len() - l.trim_start().len() == indent {
            last = i;
            break;
        }
    }
    let mut out = format!("{}:{}", path.file_name().unwrap_or_default().to_string_lossy(), line + 1);
    for l in lines.get(first..=last).unwrap_or_default() {
        out.push('\n');
        out.push_str(&l.replace('\t', "    "));
    }
    out
}

/// `--publishtest`: Publish without asking, then end.
static PUBLISH_TEST: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// `--debugtest`: 0 off, then how many stops so far plus one.
static DEBUG_TEST: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
/// How far `--lsptest` has come: 0-1 the definition, 2 the references, 3 the rename, 4 done.
static LSP_TEST: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static STUDIO: AtomicPtr<Studio> = AtomicPtr::new(std::ptr::null_mut());

/// The one Studio of this process - its window's thread is the only one that asks.
pub fn studio() -> Option<&'static mut Studio> {
    unsafe { STUDIO.load(Ordering::Relaxed).as_mut() }
}

/// What the toolbox and the properties call a kind.
pub fn kind_label(k: Kind) -> &'static str {
    match k {
        Kind::Label => t("Label"),
        Kind::Edit => t("Text box"),
        Kind::Button => t("Button"),
        Kind::Checkbox => t("Check box"),
        Kind::Radio => t("Option"),
        Kind::Combo => t("Drop-down"),
        Kind::List => t("List"),
        Kind::Group => t("Group"),
        Kind::Tabs => t("Tabs"),
        Kind::Statusbar => t("Status bar"),
        Kind::Textarea => t("Text area"),
        Kind::Date => t("Date"),
        Kind::Number => t("Number"),
        Kind::Progress => t("Progress bar"),
        Kind::Slider => t("Slider"),
        Kind::Tree => t("Tree view"),
        Kind::Picture => t("Picture"),
        Kind::Timer => t("Timer"),
        Kind::Flow => t("Flow layout"),
        Kind::Table => t("Table layout"),
        Kind::Rich => t("Rich text box"),
        Kind::Grid => t("Data grid"),
        Kind::ContextMenu => t("Context menu"),
    }
}

/// The toolbox: the pointer, then every kind - its command, name and icon.
pub fn toolbox_items() -> Vec<(i32, &'static str, u16)> {
    let mut v = vec![(ID_TOOL, t("Pointer"), glyph::POINTER), (ID_TOOL_MENUBAR, t("Menu bar"), glyph::MENU_BAR), (ID_TOOL_TOOLBAR, t("Toolbar"), glyph::TOOLBAR)];
    for (i, k) in Kind::ALL.iter().enumerate() {
        let g = match k {
            Kind::Label => glyph::FONT,
            Kind::Edit => glyph::RENAME,
            Kind::Button => glyph::BUTTON,
            Kind::Checkbox => glyph::CHECKBOX,
            Kind::Radio => glyph::RADIO,
            Kind::Combo => glyph::DROPDOWN,
            Kind::List => glyph::LIST,
            Kind::Group => glyph::GROUP,
            Kind::Tabs => glyph::TABS,
            Kind::Statusbar => glyph::STATUS,
            Kind::Textarea => glyph::TEXTAREA,
            Kind::Date => glyph::CALENDAR,
            Kind::Number => glyph::NUMBER,
            Kind::Progress => glyph::PROGRESS,
            Kind::Slider => glyph::SLIDER,
            Kind::Tree => glyph::TREE,
            Kind::Picture => glyph::PICTURE,
            Kind::Timer => glyph::TIMER,
            Kind::Flow => glyph::LAY_SPACE_ACROSS,
            Kind::Table => glyph::LAY_GRID,
            Kind::Rich => glyph::DOCUMENT,
            Kind::Grid => glyph::APPS,
            Kind::ContextMenu => glyph::MENU_CONTEXT,
        };
        v.push((ID_TOOL + 1 + i as i32, kind_label(*k), g));
    }
    v
}

/// The Standard bar, as Visual Studio's: back and forward, new and open, save and save all, undo
/// and redo, the Debug / Release box, Start with the project's name, Stop, Build, then Phai's own.
fn toolbar_items() -> Vec<(i32, &'static str, u16)> {
    vec![
        (ID_NAV_BACK, t("Navigate backward"), glyph::BACK),
        (ID_NAV_FORWARD, t("Navigate forward"), glyph::FORWARD),
        (0, "", 0),
        (ID_NEW_PROJECT, t("New project"), glyph::NEW_FOLDER),
        (ID_NEW, t("New form"), glyph::NEW),
        (ID_OPEN, t("Open"), glyph::OPEN),
        (ID_SAVE, t("Save"), glyph::SAVE),
        (ID_SAVE_ALL, t("Save all"), glyph::SAVE_ALL),
        (0, "", 0),
        (ID_UNDO, t("Undo"), glyph::UNDO),
        (ID_REDO, t("Redo"), glyph::REDO),
        (0, "", 0),
        (0, "gap:96", 0),
        (0, "", 0),
        (ID_RUN, t("Start"), glyph::PLAY),
        (ID_STOP, t("Stop"), glyph::STOP),
        (ID_BUILD, t("Build"), glyph::BUILD),
        (0, "", 0),
        (ID_PREVIEW, t("Preview"), glyph::MONITOR),
        (ID_THEME_TOGGLE, t("Theme"), glyph::BRIGHTNESS),
        (ID_LANG, t("Language"), glyph::LANGUAGE),
        (ID_CUSTOM_STD, t("Add or Remove Buttons"), glyph::DROPDOWN),
    ]
}

/// The Debug bar: Continue, Break All, Stop, then the three steps.
fn debug_items() -> Vec<(i32, &'static str, u16)> {
    vec![
        (ID_RUN, t("Continue"), glyph::PLAY),
        (ID_BREAK_ALL, t("Break All"), glyph::PAUSE),
        (ID_STOP, t("Stop Debugging"), glyph::STOP),
        (0, "", 0),
        (ID_STEP_INTO, t("Step Into"), glyph::STEP_INTO),
        (ID_STEP_OVER, t("Step Over"), glyph::STEP_OVER),
        (ID_STEP_OUT, t("Step Out"), glyph::STEP_OUT),
    ]
}

/// The Layout bar of the form designer, as Visual Studio's.
fn layout_items() -> Vec<(i32, &'static str, u16)> {
    use phai_form::arrange::Op::*;
    let n = format_names();
    let f = |op: phai_form::arrange::Op, g: u16| (format_id(op), n[FORMATS.iter().position(|o| *o == op).unwrap_or(0)], g);
    vec![
        f(AlignToGrid, glyph::LAY_GRID),
        (0, "", 0),
        f(Lefts, glyph::LAY_LEFTS),
        f(Centers, glyph::LAY_CENTERS),
        f(Rights, glyph::LAY_RIGHTS),
        (0, "", 0),
        f(Tops, glyph::LAY_TOPS),
        f(Middles, glyph::LAY_MIDDLES),
        f(Bottoms, glyph::LAY_BOTTOMS),
        (0, "", 0),
        f(SameWidth, glyph::LAY_SAME_WIDTH),
        f(SizeToGrid, glyph::LAY_SIZE_GRID),
        f(SameHeight, glyph::LAY_SAME_HEIGHT),
        f(SameSize, glyph::LAY_SAME_SIZE),
        (0, "", 0),
        f(SpaceAcross, glyph::LAY_SPACE_ACROSS),
        f(MoreAcross, glyph::LAY_MORE_ACROSS),
        f(LessAcross, glyph::LAY_LESS_ACROSS),
        f(NoneAcross, glyph::LAY_NONE_ACROSS),
        (0, "", 0),
        f(SpaceDown, glyph::LAY_SPACE_DOWN),
        f(MoreDown, glyph::LAY_MORE_DOWN),
        f(LessDown, glyph::LAY_LESS_DOWN),
        f(NoneDown, glyph::LAY_NONE_DOWN),
        (0, "", 0),
        f(CenterAcross, glyph::LAY_CENTER_ACROSS),
        f(CenterDown, glyph::LAY_CENTER_DOWN),
        (0, "", 0),
        (ID_FRONT, t("Bring to front"), glyph::LAY_FRONT),
        (ID_BACK, t("Send to back"), glyph::LAY_BACK),
        (0, "", 0),
        (ID_TAB_ORDER, t("Tab order"), glyph::LAY_TAB_ORDER),
        (ID_CUSTOM_LAYOUT, t("Add or Remove Buttons"), glyph::DROPDOWN),
    ]
}

fn build_menu() -> HMENU {
    unsafe {
        let bar = wm::CreateMenu().unwrap_or_default();
        let add = |m: HMENU, id: i32, label: &str, key: &str| {
            let text = if key.is_empty() { label.to_string() } else { format!("{label}\t{key}") };
            let w = wide(&text);
            let _ = wm::AppendMenuW(m, wm::MF_STRING, id as usize, PCWSTR(w.as_ptr()));
        };
        let sep = |m: HMENU| {
            let _ = wm::AppendMenuW(m, wm::MF_SEPARATOR, 0, None);
        };
        let sub_of = |parent: HMENU, label: &str, m: HMENU| {
            let w = wide(label);
            let _ = wm::AppendMenuW(parent, wm::MF_POPUP, m.0 as usize, PCWSTR(w.as_ptr()));
        };
        let sub = |label: &str, m: HMENU| {
            let w = wide(label);
            let _ = wm::AppendMenuW(bar, wm::MF_POPUP, m.0 as usize, PCWSTR(w.as_ptr()));
        };
        let file = wm::CreatePopupMenu().unwrap_or_default();
        add(file, ID_NEW, t("New"), "Ctrl+N");
        let tm = wm::CreatePopupMenu().unwrap_or_default();
        for (k, tpl) in phai_design::templates::Template::ALL.iter().enumerate() {
            add(tm, ID_TEMPLATE + k as i32, t(tpl.name()), "");
        }
        sub_of(file, t("New from template"), tm);
        add(file, ID_OPEN, t("Open..."), "Ctrl+O");
        add(file, ID_SAVE, t("Save"), "Ctrl+S");
        add(file, ID_SAVE_AS, t("Save as..."), "");
        add(file, ID_CLOSE, t("Close"), "Ctrl+W");
        sep(file);
        add(file, ID_NEW_PROJECT, t("New project..."), "");
        let recent = recent_files();
        if !recent.is_empty() {
            let rm = wm::CreatePopupMenu().unwrap_or_default();
            for (k, p) in recent.iter().enumerate() {
                add(rm, ID_RECENT + k as i32, &format!("&{} {}", k + 1, p.display()), "");
            }
            sub_of(file, t("Recent"), rm);
        }
        sep(file);
        add(file, ID_EXIT, t("Exit"), "");
        sub(t("&File"), file);
        let edit = wm::CreatePopupMenu().unwrap_or_default();
        add(edit, ID_UNDO, t("Undo"), "Ctrl+Z");
        add(edit, ID_REDO, t("Redo"), "Ctrl+Y");
        sep(edit);
        add(edit, ID_CUT, t("Cut"), "Ctrl+X");
        add(edit, ID_COPY, t("Copy"), "Ctrl+C");
        add(edit, ID_PASTE, t("Paste"), "Ctrl+V");
        add(edit, ID_DUPLICATE, t("Duplicate"), "Ctrl+D");
        add(edit, ID_DELETE, t("Delete"), "Del");
        sep(edit);
        add(edit, ID_SELECT_ALL, t("Select all"), "Ctrl+A");
        sep(edit);
        add(edit, ID_GO_DEFINITION, t("Go To Definition"), "F12");
        add(edit, ID_PEEK, t("Peek Definition"), "Alt+F12");
        add(edit, ID_PARAM_INFO, t("Parameter Info"), "Ctrl+Shift+Space");
        add(edit, ID_QUICK_ACTIONS, t("Quick Actions and Refactorings..."), "Ctrl+.");
        add(edit, ID_REFERENCES, t("Find All References"), "Shift+F12");
        add(edit, ID_CALLS, t("View Call Hierarchy"), "Ctrl+K, Ctrl+T");
        add(edit, ID_RENAME, t("Rename..."), "F2");
        add(edit, ID_COMMENT, t("Toggle Comment"), "Ctrl+/");
        add(edit, ID_FORMAT_DOC, t("Format Document"), "Ctrl+K, Ctrl+D");
        add(edit, ID_FOLD, t("Toggle Outlining Expansion"), "Ctrl+M, Ctrl+M");
        add(edit, ID_FOLD_ALL, t("Toggle All Outlining"), "Ctrl+M, Ctrl+L");
        add(edit, ID_FIND_IN_FILES, t("Find in Files..."), "Ctrl+Shift+F");
        add(edit, ID_GO_TO_ALL, t("Go to All..."), "Ctrl+T");
        add(edit, ID_SNIPPET, t("Insert Snippet..."), "Ctrl+K, Ctrl+X");
        add(edit, ID_BM_TOGGLE, t("Toggle Bookmark"), "Ctrl+K, Ctrl+K");
        add(edit, ID_BM_NEXT, t("Next Bookmark"), "Ctrl+K, Ctrl+N");
        add(edit, ID_BM_PREV, t("Previous Bookmark"), "Ctrl+K, Ctrl+P");
        add(edit, ID_BM_CLEAR, t("Clear Bookmarks"), "Ctrl+K, Ctrl+L");
        add(edit, ID_MOVE_UP, t("Move Line Up"), "Alt+Up");
        add(edit, ID_MOVE_DOWN, t("Move Line Down"), "Alt+Down");
        sep(edit);
        add(edit, ID_NEXT_ERROR, t("Go To Next Error"), "F8");
        add(edit, ID_PREV_ERROR, t("Go To Previous Error"), "Shift+F8");
        sep(edit);
        add(edit, ID_EDIT_MENUS, t("Menus..."), "");
        add(edit, ID_EDIT_TOOLBAR, t("Toolbar..."), "");
        sub(t("&Edit"), edit);
        let view = wm::CreatePopupMenu().unwrap_or_default();
        add(view, ID_PREVIEW, t("Preview"), "F4");
        sep(view);
        add(view, ID_VIEW_TOOLBOX, t("Toolbox"), "");
        add(view, ID_VIEW_EXPLORER, t("Project explorer"), "");
        add(view, ID_VIEW_PROPS, t("Properties"), "");
        add(view, ID_VIEW_OUTPUT, t("Errors and output"), "");
        add(view, ID_VIEW_SERVERS, t("Server Explorer"), "Ctrl+Alt+S");
        add(view, ID_VIEW_OUTLINE, t("Document Outline"), "Ctrl+Alt+T");
        add(view, ID_SPLIT, t("Split"), "");
        add(view, ID_NEW_WINDOW, t("New Window"), "");
        if studio().is_some_and(|s| s.split_view.is_some()) {
            wm::CheckMenuItem(view, ID_SPLIT as u32, wm::MF_CHECKED.0);
        }
        add(view, ID_TASK_LIST, t("Task List"), "");
        add(view, ID_START_WINDOW, t("Start Window"), "");
        if studio().is_some_and(|s| explorer::servers_shown(s.side)) {
            wm::CheckMenuItem(view, ID_VIEW_SERVERS as u32, wm::MF_CHECKED.0);
        }
        if studio().is_some_and(|s| explorer::outline_shown(s.side)) {
            wm::CheckMenuItem(view, ID_VIEW_OUTLINE as u32, wm::MF_CHECKED.0);
        }
        if let Some(s) = studio() {
            let (e, p) = explorer::side_panes(s.side);
            for (id, on) in [(ID_VIEW_TOOLBOX, s.toolbox_on), (ID_VIEW_EXPLORER, e), (ID_VIEW_PROPS, p), (ID_VIEW_OUTPUT, s.bottom_on)] {
                if on {
                    wm::CheckMenuItem(view, id as u32, wm::MF_CHECKED.0);
                }
            }
        }
        sep(view);
        add(view, ID_LIGHT, t("Light"), "");
        add(view, ID_DARK, t("Dark"), "");
        sep(view);
        add(view, ID_THAI, "ไทย", "");
        add(view, ID_ENGLISH, "English", "");
        wm::CheckMenuItem(view, if theme::current().is_dark() { ID_DARK } else { ID_LIGHT } as u32, wm::MF_CHECKED.0);
        wm::CheckMenuItem(view, if words::is_thai() { ID_THAI } else { ID_ENGLISH } as u32, wm::MF_CHECKED.0);
        sub(t("&View"), view);
        let fmt = wm::CreatePopupMenu().unwrap_or_default();
        let names = format_names();
        for (k, name) in names.iter().enumerate() {
            if matches!(k, 6 | 9 | 11 | 13 | 19) {
                sep(fmt);
            }
            add(fmt, ID_FORMAT + k as i32, name, "");
        }
        sep(fmt);
        add(fmt, ID_FRONT, t("Bring to front"), "");
        add(fmt, ID_BACK, t("Send to back"), "");
        sep(fmt);
        add(fmt, ID_LOCK, t("Lock controls"), "");
        add(fmt, ID_TAB_ORDER, t("Tab order"), "");
        // The Layout bar's pictures beside the same commands, as Visual Studio's Format menu has them.
        for (id, _, g) in layout_items() {
            if id != 0 && g != 0 {
                phai_ui::icon::set_menu_glyph(fmt, id as u32, g);
            }
        }
        phai_ui::icon::set_menu_glyph(fmt, ID_LOCK as u32, 0xE72E); // Lock
        if let Some(c) = studio().and_then(|s| s.canvas.as_ref()) {
            for (id, on) in [(ID_LOCK, c.locked), (ID_TAB_ORDER, c.tab_mode())] {
                if on {
                    wm::CheckMenuItem(fmt, id as u32, wm::MF_CHECKED.0);
                }
            }
        }
        sub(t("F&ormat"), fmt);
        // Build, as Visual Studio's: the whole workspace (its Solution), then the one project.
        let project = studio().and_then(|s| s.doc.path.as_deref().and_then(cargo::project_of)).and_then(|p| cargo::package_name(&p).or_else(|| p.file_name().map(|n| n.to_string_lossy().to_string()))).unwrap_or_else(|| t("project").to_string());
        let build = wm::CreatePopupMenu().unwrap_or_default();
        add(build, ID_BUILD_ALL, t("Build Solution"), "Ctrl+Shift+B");
        add(build, ID_REBUILD_ALL, t("Rebuild Solution"), "");
        add(build, ID_CLEAN_ALL, t("Clean Solution"), "");
        add(build, ID_BATCH_BUILD, t("Batch Build..."), "");
        add(build, ID_ANALYSE_ALL, t("Run Code Analysis on Solution"), "Alt+F11");
        sep(build);
        add(build, ID_BUILD, &tf("Build {0}", &[&project]), "Ctrl+B");
        add(build, ID_REBUILD, &tf("Rebuild {0}", &[&project]), "");
        add(build, ID_CLEAN, &tf("Clean {0}", &[&project]), "");
        add(build, ID_ANALYSE, &tf("Run Code Analysis on {0}", &[&project]), "");
        sep(build);
        add(build, ID_PUBLISH, &tf("Publish {0}...", &[&project]), "");
        sep(build);
        add(build, ID_DEBUG, "Debug", "");
        add(build, ID_RELEASE, "Release", "");
        let release = studio().is_some_and(|s| s.release);
        wm::CheckMenuItem(build, if release { ID_RELEASE } else { ID_DEBUG } as u32, wm::MF_CHECKED.0);
        sub(t("&Build"), build);
        let debug = wm::CreatePopupMenu().unwrap_or_default();
        let paused = studio().is_some_and(|s| s.paused);
        add(debug, ID_RUN, if paused { t("Continue") } else { t("Start Debugging") }, "F5");
        add(debug, ID_RUN_NODEBUG, t("Start Without Debugging"), "Ctrl+F5");
        add(debug, ID_BREAK_ALL, t("Break All"), "Ctrl+Alt+Break");
        add(debug, ID_STOP, t("Stop Debugging"), "Shift+F5");
        add(debug, ID_ATTACH, t("Attach to Process..."), "Ctrl+Alt+P");
        add(debug, ID_DETACH, t("Detach All"), "");
        sep(debug);
        add(debug, ID_STEP_INTO, t("Step Into"), "F11");
        add(debug, ID_STEP_OVER, t("Step Over"), "F10");
        add(debug, ID_STEP_OUT, t("Step Out"), "Shift+F11");
        sep(debug);
        add(debug, ID_ADD_WATCH, t("Add Watch..."), "Shift+F9");
        add(debug, ID_IMMEDIATE, t("Immediate Window"), "Ctrl+Alt+I");
        add(debug, ID_DIAG_TOOLS, t("Show Diagnostic Tools"), "Ctrl+Alt+F2");
        add(debug, ID_BREAKPOINTS_WIN, t("Breakpoints Window"), "Ctrl+Alt+B");
        add(debug, ID_THREADS_WIN, t("Threads Window"), "Ctrl+Alt+H");
        add(debug, ID_EXCEPTIONS, t("Exception Settings..."), "Ctrl+Alt+E");
        add(debug, ID_MEMORY_WIN, t("Memory Window"), "Ctrl+Alt+M");
        add(debug, ID_DISASSEMBLY, t("Disassembly"), "Ctrl+Alt+D");
        add(debug, ID_DATA_BP, t("New Data Breakpoint..."), "");
        sep(debug);
        add(debug, ID_TOGGLE_BP, t("Toggle Breakpoint"), "F9");
        add(debug, ID_BP_CONDITION, t("Breakpoint Condition..."), "Alt+F9");
        add(debug, ID_CLEAR_BPS, t("Delete All Breakpoints"), "Ctrl+Shift+F9");
        sub(t("&Debug"), debug);
        let proj = wm::CreatePopupMenu().unwrap_or_default();
        add(proj, ID_ADD_FORM, t("Add Form..."), "");
        sep(proj);
        add(proj, ID_MANAGE_CRATES, t("Manage Crates..."), "");
        sep(proj);
        add(proj, ID_PROJECT_PROPS, &tf("{0} Properties...", &[&project]), "Alt+F7");
        sub(t("&Project"), proj);
        let test = wm::CreatePopupMenu().unwrap_or_default();
        add(test, ID_RUN_TESTS, &tf("Run Tests of {0}", &[&project]), "Ctrl+R, T");
        add(test, ID_RUN_ALL_TESTS, t("Run All Tests"), "Ctrl+R, A");
        sep(test);
        add(test, ID_COVERAGE, t("Analyze Code Coverage for All Tests"), "");
        sub(t("&Test"), test);
        let gitm = wm::CreatePopupMenu().unwrap_or_default();
        add(gitm, ID_GIT_CHANGES, t("Git Changes..."), "Ctrl+Alt+G");
        sep(gitm);
        add(gitm, ID_GIT_HISTORY, t("View History of This File..."), "");
        add(gitm, ID_MERGE, t("Resolve Merge Conflicts..."), "");
        add(gitm, ID_GIT_BLAME, t("Show Line Blame"), "");
        if studio().is_some_and(|s| s.blame_on) {
            wm::CheckMenuItem(gitm, ID_GIT_BLAME as u32, wm::MF_CHECKED.0);
        }
        sub(t("&Git"), gitm);
        let data = wm::CreatePopupMenu().unwrap_or_default();
        add(data, ID_ADD_CONNECTION, t("Add Connection..."), "");
        add(data, ID_VIEW_SERVERS, t("Server Explorer"), "Ctrl+Alt+S");
        sep(data);
        add(data, ID_NEW_QUERY, t("New Query"), "");
        add(data, ID_EXECUTE, t("Execute Query"), "Ctrl+Shift+E");
        sub(t("D&ata"), data);
        let th = wm::CreatePopupMenu().unwrap_or_default();
        // The project's style sheets, the form's own ticked - choose one to use it.
        let (sheets, current) = studio().map(|s| (s.theme_files(), s.doc.form.form.theme.clone())).unwrap_or_default();
        for (i, name) in sheets.iter().enumerate().take(THEME_FILES_MAX) {
            add(th, ID_THEME_FILE + i as i32, &name.replace('&', "&&"), "");
            if current.as_deref() == Some(name.as_str()) {
                wm::CheckMenuItem(th, (ID_THEME_FILE + i as i32) as u32, wm::MF_CHECKED.0);
            }
        }
        if current.is_some() {
            add(th, ID_THEME_EDIT, &tf("Edit {0}", &[&current.clone().unwrap_or_default().rsplit(['/', char::from(92)]).next().unwrap_or_default().to_string()]), "");
        }
        if !sheets.is_empty() || current.is_some() {
            sep(th);
        }
        add(th, ID_THEME_NEW, t("New theme from the standard..."), "");
        add(th, ID_THEME_IMPORT, t("Import CSS..."), "");
        sep(th);
        add(th, ID_THEME_NONE, t("No theme"), "");
        if current.is_none() {
            wm::CheckMenuItem(th, ID_THEME_NONE as u32, wm::MF_CHECKED.0);
        }
        sub(t("&Theme"), th);
        let help = wm::CreatePopupMenu().unwrap_or_default();
        add(help, ID_HELP_STUDIO, t("Phai Studio Guide"), "F1");
        add(help, ID_HELP_RUST, t("Learn Rust"), "");
        add(help, ID_HELP_ERRORS, t("Reading Compiler Errors"), "");
        sep(help);
        add(help, ID_ABOUT, t("About Phai Studio"), "");
        sub(t("&Help"), help);
        bar
    }
}

impl Studio {
    // --- editing the model -------------------------------------------------------------------

    /// Remembers the form as it is, for Undo.
    pub fn checkpoint(&mut self) {
        self.undo.push(self.doc.form.clone());
        if self.undo.len() > 200 {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// One change, one undo step.
    pub fn edit(&mut self, change: impl FnOnce(&mut FormFile)) {
        self.checkpoint();
        change(&mut self.doc.form);
        self.changed();
    }

    fn changed(&mut self) {
        let was = self.doc.dirty;
        self.doc.dirty = true;
        self.say_title();
        if !was {
            self.refresh_tabs();
        }
    }

    /// A property typed into its box: the model takes it now, the canvas shows it now. Several
    /// controls chosen: each of them takes it.
    pub fn apply(&mut self, key: Key, text: &str) {
        let chosen = self.selection();
        let form_key = matches!(key, Key::FormName | Key::FormTitle | Key::FormWidth | Key::FormHeight | Key::FormSize | Key::FormFont | Key::FormTheme | Key::FormStart | Key::FormState | Key::FormMore(_) | Key::FormOn(_));
        if chosen.len() > 1 && !form_key && !matches!(key, Key::Id | Key::Event) {
            let primary = self.selected;
            for j in chosen {
                self.selected = Some(j);
                self.apply_one(key, text);
            }
            self.selected = primary;
            self.say_selection();
            return;
        }
        self.apply_one(key, text);
    }

    fn apply_one(&mut self, key: Key, text: &str) {
        // A composite typed in whole: its parts, one by one.
        match key {
            Key::Location | Key::Size | Key::FormSize => {
                let Some((a, b)) = props::pair(text) else { return };
                let (ka, kb) = match key {
                    Key::Location => (Key::X, Key::Y),
                    Key::Size => (Key::W, Key::H),
                    _ => (Key::FormWidth, Key::FormHeight),
                };
                self.apply_one(ka, &a.to_string());
                return self.apply_one(kb, &b.to_string());
            }
            Key::FontAll => {
                let Some((size, bold)) = props::font_spec(text) else { return };
                // The form's own size is no size of this control's.
                let own = size.filter(|n| Some(*n) != self.doc.form.form.font_size.or(Some(9)));
                self.apply_one(Key::Font, &own.map(|n| n.to_string()).unwrap_or_default());
                return self.apply_one(Key::Bold, if bold { "True" } else { "False" });
            }
            _ => {}
        }
        let f = &mut self.doc.form;
        let num = text.trim().parse::<i32>().ok();
        // A list is one item a line, as the "…" dialog writes it; commas also part, as typed in a box.
        let list = || text.split(['\n', ',']).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect::<Vec<_>>();
        let yes = text.trim().eq_ignore_ascii_case("true");
        // A colour as CSS writes it, or nothing; a word that is not one changes nothing.
        let colour = || -> Result<Option<String>, ()> {
            let t = text.trim();
            if t.is_empty() {
                Ok(None)
            } else if phai_ui::theme::parse_color(t).is_some() {
                Ok(Some(t.to_string()))
            } else {
                Err(())
            }
        };
        match (key, self.selected) {
            (Key::FormName, _) => f.form.name = text.trim().to_string(),
            (Key::FormTitle, _) => f.form.title = text.to_string(),
            (Key::FormWidth, _) => match num {
                Some(n) if n >= 120 => f.form.size[0] = n,
                _ => return,
            },
            (Key::FormHeight, _) => match num {
                Some(n) if n >= 80 => f.form.size[1] = n,
                _ => return,
            },
            (Key::FormFont, _) => {
                f.form.font_size = num.filter(|n| (6..=48).contains(n));
                if !text.trim().is_empty() && f.form.font_size.is_none() {
                    return;
                }
            }
            (Key::FormStart, _) => f.form.start = (text == t("Windows default")).then(|| "windows".to_string()),
            (Key::FormState, _) => f.form.maximized = (text == t("Maximized")).then_some(true),
            (Key::FormOn(name), _) => {
                let h = text.trim();
                if h.is_empty() {
                    f.form.events.remove(name);
                } else {
                    f.form.events.insert(name.to_string(), h.to_string());
                }
            }
            (Key::FormMore(name), _) => {
                let Some(sp) = phai_form::props::find_form(name) else { return };
                let Ok(v) = phai_form::props::parse(sp, text) else { return };
                use phai_form::HasProps;
                f.form.set_prop(name, v);
            }
            (Key::FormTheme, _) => {
                f.form.theme = (!text.trim().is_empty()).then(|| text.trim().to_string());
                self.changed();
                self.load_theme();
                self.rebuild_menu();
                return;
            }
            (_, None) => return,
            (k, Some(i)) => {
                let c = &mut f.control[i];
                let [x, y, w, h] = c.rect();
                match k {
                    Key::Id => c.id = text.trim().to_string(),
                    Key::Text => c.text = Some(text.to_string()),
                    Key::X => c.bounds = Some([num.unwrap_or(x), y, w, h]),
                    Key::Y => c.bounds = Some([x, num.unwrap_or(y), w, h]),
                    Key::W => c.bounds = Some([x, y, num.unwrap_or(w).max(1), h]),
                    Key::H => c.bounds = Some([x, y, w, num.unwrap_or(h).max(1)]),
                    Key::Class => c.class = (!text.trim().is_empty()).then(|| text.trim().to_string()),
                    Key::Font => {
                        c.font_size = num.filter(|n| (6..=48).contains(n));
                        if !text.trim().is_empty() && c.font_size.is_none() {
                            return;
                        }
                    }
                    Key::Event => c.set_handler((!text.trim().is_empty()).then(|| text.trim().to_string())),
                    Key::Bold => c.bold = yes.then_some(true),
                    Key::BackColor | Key::ForeColor | Key::BorderColor => {
                        let Ok(v) = colour() else { return };
                        match k {
                            Key::BackColor => c.back_color = v,
                            Key::ForeColor => c.fore_color = v,
                            _ => c.border_color = v,
                        }
                    }
                    Key::Radius => match num {
                        Some(n) if (0..=50).contains(&n) => c.radius = Some(n),
                        None if text.trim().is_empty() => c.radius = None,
                        _ => return,
                    },
                    Key::Enabled => c.enabled = (!yes).then_some(false),
                    Key::Locked => c.locked = yes.then_some(true),
                    Key::ContextMenu => c.context_menu = (!text.trim().is_empty()).then(|| text.trim().to_string()),
                    Key::Cols => match num {
                        Some(n) if (1..=12).contains(&n) => c.cols = Some(n),
                        _ => return,
                    },
                    Key::Visible => c.visible = (!yes).then_some(false),
                    Key::Checked => c.checked = yes.then_some(true),
                    Key::Password => c.password = yes.then_some(true),
                    Key::Group => c.group = num.filter(|n| *n != 0),
                    Key::Dock => {
                        let d = text.trim();
                        if !phai_form::DOCKS.contains(&d) {
                            return;
                        }
                        c.dock = (d != "none").then(|| d.to_string());
                        if c.dock.is_none() && c.bounds.is_none() {
                            let (dw, dh) = c.kind.default_size();
                            c.bounds = Some([11, 11, dw.max(20), dh]);
                        }
                    }
                    Key::Anchor => match phai_form::parse_anchor(Some(text.trim())) {
                        Ok(a) => c.anchor = phai_form::anchor_text(a),
                        Err(_) => return,
                    },
                    Key::Value if c.kind == Kind::Timer => match num {
                        Some(n) if n >= 1 => c.value = Some(n),
                        _ => return,
                    },
                    Key::Min | Key::Max | Key::Value => {
                        let Some(n) = num else { return };
                        let (mut lo, mut hi, mut v) = (c.min.unwrap_or(0), c.max.unwrap_or(100), c.value.unwrap_or(0));
                        match k {
                            Key::Min => lo = n,
                            Key::Max => hi = n,
                            _ => v = n,
                        }
                        if lo > hi {
                            return;
                        }
                        v = v.clamp(lo, hi);
                        c.min = (lo != 0).then_some(lo);
                        c.max = (hi != 100).then_some(hi);
                        c.value = (v != 0).then_some(v);
                    }
                    Key::Image => c.image = (!text.trim().is_empty()).then(|| text.trim().replace('\\', "/")),
                    Key::On(name) => {
                        let h = text.trim();
                        if h.is_empty() {
                            c.events.remove(name);
                        } else {
                            c.events.insert(name.to_string(), h.to_string());
                        }
                    }
                    Key::More(name) => {
                        let Some(sp) = phai_form::props::find(c.kind, name) else { return };
                        let Ok(v) = phai_form::props::parse(sp, text) else { return };
                        use phai_form::HasProps;
                        c.set_prop(name, v);
                    }
                    Key::Items => match c.kind {
                        Kind::Combo => c.items = list(),
                        Kind::ContextMenu => c.items = text.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string).collect(),
                        // A tree's lines keep their indent: it is the tree.
                        Kind::Tree => c.items = text.lines().map(str::trim_end).filter(|l| !l.trim().is_empty()).map(str::to_string).collect(),
                        Kind::Tabs => c.tabs = list(),
                        Kind::List | Kind::Grid => {
                            c.columns = list().into_iter().map(|s| match s.rsplit_once(':') {
                                Some((n, w)) => (n.trim().to_string(), w.trim().parse().unwrap_or(100)),
                                None => (s, 100),
                            }).collect()
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        self.changed();
        self.rebuild();
        self.say_selection();
    }

    pub fn rebuild(&mut self) {
        // Pictures with a relative path are the project's.
        let base = self.doc.path.as_deref().and_then(cargo::project_of);
        phai_ui::picture::set_base(base.as_deref());
        if let Some(c) = self.canvas.as_mut() {
            c.rebuild();
        }
        self.refresh_outline();
    }

    /// The smart tag clicked: the chosen control's common tasks, as a menu by the tag.
    pub fn smart_tag(&mut self) {
        let Some(i) = self.selected else { return };
        let Some(c) = self.doc.form.control.get(i).cloned() else { return };
        const TEXT: usize = 1;
        const ITEMS: usize = 2;
        const IMAGE: usize = 3;
        const DOCK: usize = 4;
        const EVENT: usize = 5;
        const LOCK: usize = 6;
        let mut tasks: Vec<(usize, String)> = Vec::new();
        if c.kind.has_text() {
            tasks.push((TEXT, t("Edit Text...").to_string()));
        }
        match c.kind {
            Kind::List | Kind::Grid => tasks.push((ITEMS, t("Edit Columns...").to_string())),
            Kind::Combo | Kind::Tree => tasks.push((ITEMS, t("Edit Items...").to_string())),
            Kind::Tabs => tasks.push((ITEMS, t("Edit Tabs...").to_string())),
            Kind::Picture => tasks.push((IMAGE, t("Choose Image...").to_string())),
            _ => {}
        }
        if matches!(c.kind, Kind::List | Kind::Tree | Kind::Textarea | Kind::Picture | Kind::Group | Kind::Tabs) {
            tasks.push((DOCK, if c.dock.as_deref() == Some("fill") { t("Undock in Parent Container") } else { t("Dock in Parent Container") }.to_string()));
        }
        if let Some(e) = c.kind.event() {
            let name = match e {
                "on_click" => t("Click"),
                "on_change" => t("Change"),
                "on_tick" => t("Tick"),
                _ => t("Select"),
            };
            tasks.push((EVENT, tf("Write the {0} Method", &[&name])));
        }
        tasks.push((LOCK, if c.locked == Some(true) { t("Unlock") } else { t("Lock") }.to_string()));
        // `PHAI_TAG=n`: the n-th task, without the menu - for testing; its titles printed.
        let chosen = if let Ok(n) = std::env::var("PHAI_TAG") {
            println!("tasks: {}", tasks.iter().map(|(_, s)| s.as_str()).collect::<Vec<_>>().join(" | "));
            n.parse::<usize>().ok().and_then(|n| tasks.get(n)).map(|(id, _)| *id).unwrap_or(0)
        } else {
            unsafe {
                let m = wm::CreatePopupMenu().unwrap_or_default();
                let w = wide(&tf("{0} Tasks", &[&kind_label(c.kind)]));
                let _ = wm::AppendMenuW(m, wm::MF_STRING | wm::MF_GRAYED, 0, PCWSTR(w.as_ptr()));
                let _ = wm::AppendMenuW(m, wm::MF_SEPARATOR, 0, PCWSTR::null());
                for (id, label) in &tasks {
                    let w = wide(label);
                    let _ = wm::AppendMenuW(m, wm::MF_STRING, *id, PCWSTR(w.as_ptr()));
                }
                let mut pt = windows::Win32::Foundation::POINT::default();
                let _ = wm::GetCursorPos(&mut pt);
                let n = wm::TrackPopupMenu(m, wm::TPM_RETURNCMD | wm::TPM_LEFTALIGN | wm::TPM_TOPALIGN, pt.x, pt.y, None, self.hwnd, None).0 as usize;
                let _ = wm::DestroyMenu(m);
                n
            }
        };
        let set = |s: &mut Studio, key: Key, value: &str| {
            s.checkpoint();
            s.apply(key, value);
            if let Some(p) = s.props.as_mut() {
                p.refresh();
            }
        };
        match chosen {
            TEXT => {
                if let Some(v) = phai_ui::dialogs::ask(Some(self.hwnd), t("Edit Text..."), &tf("The words {0} shows:", &[&c.id]), c.text.as_deref().unwrap_or(""), false) {
                    set(self, Key::Text, &v);
                }
            }
            ITEMS => {
                let now = match c.kind {
                    Kind::Tabs => c.tabs.join("\n"),
                    Kind::List | Kind::Grid => c.columns.iter().map(|(n, w)| format!("{n}:{w}")).collect::<Vec<_>>().join("\n"),
                    _ => c.items.join("\n"),
                };
                if let Some(v) = phai_ui::dialogs::ask_lines(Some(self.hwnd), &tf("{0} Tasks", &[&kind_label(c.kind)]), t("One a line; a list's columns as Name:width."), &now) {
                    set(self, Key::Items, &v);
                }
            }
            IMAGE => {
                if let Some(p) = file_dialog(self.hwnd, false, t("Pictures (PNG, JPEG, BMP, GIF)"), "*.png;*.jpg;*.jpeg;*.bmp;*.gif", "") {
                    // Inside the project: kept relative, so the project can move.
                    let base = self.doc.path.as_deref().and_then(cargo::project_of);
                    let shown = base.as_deref().and_then(|b| p.strip_prefix(b).ok()).map(|r| r.to_string_lossy().replace('\\', "/")).unwrap_or_else(|| p.to_string_lossy().to_string());
                    set(self, Key::Image, &shown);
                }
            }
            DOCK => set(self, Key::Dock, if c.dock.as_deref() == Some("fill") { "none" } else { "fill" }),
            EVENT => self.add_handler_for_selected(),
            LOCK => set(self, Key::Locked, if c.locked == Some(true) { "False" } else { "True" }),
            _ => {}
        }
    }

    /// The code file's lenses drawn: the references counted, and the caret line's blame after.
    fn show_lenses(&self) {
        let Some(h) = self.doc.code else { return };
        let mut all = self.lenses.clone();
        if self.blame_on {
            let line = phai_editor::view::caret_line(h).saturating_sub(1);
            if let Some(words) = self.blame.get(line as usize) {
                match all.iter_mut().find(|x| x.0 == line) {
                    Some(x) => x.2 = format!("{}  ·  {words}", x.2),
                    None => all.push((line, (line, u32::MAX), words.clone())),
                }
            }
        }
        phai_editor::view::set_lenses(h, all);
    }

    /// The Thai guide's page `page` opened in the browser: from beside Phai Studio when it is
    /// installed, else from the source's docs folder.
    fn open_help(&self, page: &str) {
        match help_page(page) {
            Some(p) => {
                if std::env::var_os("PHAI_HELP_TEST").is_some() {
                    println!("help: {}", p.display());
                    return;
                }
                open_with_windows(&p.to_string_lossy());
            }
            None => {
                win::message(Some(self.hwnd), t("The guide is not installed with this copy of Phai Studio."), t("Phai Studio Guide"), wm::MB_ICONINFORMATION);
            }
        }
    }

    /// Test > Analyze Code Coverage: the project's tests run with coverage (a wait shown), each
    /// file's share of lines run listed, and the editors marked line by line.
    fn analyze_coverage(&mut self) {
        let Some(project) = self.doc.path.as_deref().and_then(cargo::project_of).and_then(|p| std::path::absolute(p).ok()) else {
            win::message(Some(self.hwnd), t("Save the form in a project first - or make one with New project."), "Phai Studio", wm::MB_ICONINFORMATION);
            return;
        };
        let Some(cargo) = cargo::find_cargo() else { return };
        if let Err(why) = coverage::tools_ready(&cargo) {
            // Offered, as Visual Studio offers a missing workload; installed by rustup itself.
            let yes = std::env::var_os("PHAI_COVERAGE_TEST").is_none() && win::message(Some(self.hwnd), &format!("{why}\n\n{}", t("Install it now?")), t("Code Coverage"), wm::MB_YESNO | wm::MB_ICONQUESTION) == wm::IDYES;
            if !yes {
                return;
            }
            let rustup = cargo.with_file_name("rustup.exe");
            let ok = phai_ui::dialogs::wait_for(Some(self.hwnd), t("Code Coverage"), t("Installing llvm-tools..."), move |_| {
                use std::os::windows::process::CommandExt;
                std::process::Command::new(rustup).args(["component", "add", "llvm-tools"]).creation_flags(0x0800_0000).status().is_ok_and(|s| s.success())
            });
            if ok != Some(true) {
                return;
            }
        }
        let p = project.clone();
        let result = if std::env::var_os("PHAI_COVERAGE_TEST").is_some() {
            Some(coverage::run(&p, &cargo))
        } else {
            phai_ui::dialogs::wait_for(Some(self.hwnd), t("Code Coverage"), t("Running the tests with code coverage..."), move |_| coverage::run(&p, &cargo))
        };
        let Some(result) = result else { return };
        match result {
            Ok(files) => {
                self.coverage = files;
                self.show_coverage();
            }
            Err(e) => {
                if std::env::var_os("PHAI_COVERAGE_TEST").is_some() {
                    println!("coverage failed: {e}");
                }
                win::message(Some(self.hwnd), &e, t("Code Coverage"), wm::MB_ICONWARNING);
            }
        }
    }

    /// The Code Coverage tab filled - all files, then each - and the open editors marked.
    fn show_coverage(&mut self) {
        let Some(list) = self.bottom.as_ref().map(|b| b.coverage) else { return };
        unsafe {
            wm::SendMessageW(list, windows::Win32::UI::Controls::LVM_DELETEALLITEMS, None, None);
        }
        let pct = |c: usize, n: usize| if n == 0 { "-".to_string() } else { format!("{:.1}%", c as f64 * 100.0 / n as f64) };
        let (c, n) = self.coverage.iter().fold((0, 0), |(c, n), f| (c + f.covered(), n + f.lines.len()));
        win::list_add(list, &[t("All files"), &c.to_string(), &n.to_string(), &pct(c, n)]);
        let root = self.doc.path.as_deref().and_then(cargo::project_of).and_then(|p| std::path::absolute(p).ok());
        for f in &self.coverage {
            let name = root.as_ref().and_then(|r| f.path.strip_prefix(r).ok()).unwrap_or(&f.path).to_string_lossy().to_string();
            win::list_add(list, &[&name, &f.covered().to_string(), &f.lines.len().to_string(), &pct(f.covered(), f.lines.len())]);
        }
        if std::env::var_os("PHAI_COVERAGE_TEST").is_some() {
            println!("coverage: {c} of {n} lines ({})", pct(c, n));
            for f in &self.coverage {
                println!("  {}: not run {:?}", f.path.file_name().unwrap_or_default().to_string_lossy(), f.lines.iter().filter(|(_, k)| *k == 0).map(|(l, _)| *l).collect::<Vec<_>>());
            }
        }
        self.mark_coverage();
        self.show_pane(Pane::Output, true);
        if let Some(b) = self.bottom.as_ref() {
            b.show_tab(14);
        }
        self.say_status(&tf("Code coverage: {0} of {1} lines run by the tests ({2})", &[&c, &n, &pct(c, n)]));
    }

    /// Each open editor's lines marked as the last coverage found them.
    fn mark_coverage(&mut self) {
        let open: Vec<(PathBuf, HWND)> = self.tabs.iter().filter_map(|t| Some((t.doc.path.clone()?, t.doc.code?))).chain(self.doc.path.clone().zip(self.doc.code)).collect();
        for (p, h) in open {
            let p = std::path::absolute(&p).unwrap_or(p);
            let lines: Vec<(u32, bool)> = self.coverage.iter().find(|f| phai_editor::lsp::same_path(&f.path, &p)).map(|f| f.lines.iter().map(|(l, n)| (*l, *n > 0)).collect()).unwrap_or_default();
            phai_editor::view::set_coverage(h, &lines);
        }
    }

    /// Git > Resolve Merge Conflicts: the Merge Editor on the file in front when it has conflicts,
    /// else on the first file of its repository that does; the editor shows the result after.
    fn resolve_conflicts(&mut self) {
        let from = self.doc.path.clone().or_else(|| std::env::current_dir().ok());
        let Some(repo) = from.as_deref().and_then(gitpane::repository_of) else {
            win::message(Some(self.hwnd), t("This file is not in a Git repository."), t("Merge Editor"), wm::MB_ICONINFORMATION);
            return;
        };
        let files = merge::conflicted(&repo);
        let here = self.doc.path.as_deref().and_then(|p| std::path::absolute(p).ok());
        let file = here.clone().filter(|h| files.iter().any(|f| phai_editor::lsp::same_path(f, h))).or_else(|| files.first().cloned());
        let Some(file) = file else {
            win::message(Some(self.hwnd), t("Nothing in this repository has merge conflicts."), t("Merge Editor"), wm::MB_ICONINFORMATION);
            return;
        };
        if merge::open(self.hwnd, &repo, &file) {
            self.open_code(&file);
            if let Some(h) = self.doc.code {
                phai_editor::view::replace_whole(h, &std::fs::read_to_string(&file).unwrap_or_default());
            }
            self.say_status(&tf("{0}: merged and staged", &[&file.file_name().unwrap_or_default().to_string_lossy()]));
        }
    }

    /// Git blame of the code file in front read again (from what is saved), when it is shown.
    fn refresh_blame(&mut self) {
        self.blame = match (self.blame_on, self.doc.code, self.doc.path.clone()) {
            (true, Some(_), Some(p)) => match (gitpane::repository_of(&p), std::path::absolute(&p)) {
                (Some(repo), Ok(file)) => githistory::blame(&repo, &file),
                _ => vec![],
            },
            _ => vec![],
        };
        self.show_lenses();
    }

    /// Code lenses for the code file in front: asked of the language server, then each worked out.
    pub fn refresh_lenses(&mut self) {
        let (Some(h), Some(path)) = (self.doc.code, self.doc.path.clone()) else { return };
        if path.extension().is_none_or(|e| e != "rs") {
            return;
        }
        if let Some(l) = self.lsp_for(&path) {
            let id = l.document_symbols(&path);
            self.asked.insert(id, (h, Ask::LensSymbols));
        }
    }

    /// Document Outline again from the form, when it shows.
    pub fn refresh_outline(&mut self) {
        if !explorer::outline_shown(self.side) {
            return;
        }
        // A code file: its symbols, from the language server.
        if let (Some(h), Some(path)) = (self.doc.code, self.doc.path.clone()) {
            if path.extension().is_some_and(|e| e == "rs") {
                if let Some(l) = self.lsp_for(&path) {
                    let id = l.document_symbols(&path);
                    self.asked.insert(id, (h, Ask::Outline));
                }
            } else if let Some(o) = self.outline.as_mut() {
                o.fill_code(&path.file_name().unwrap_or_default().to_string_lossy(), &[]);
            }
            return;
        }
        if let Some(o) = self.outline.as_mut() {
            o.fill(&self.doc.form, self.selected);
        }
    }

    /// A control chosen in Document Outline: chosen on the form.
    pub fn outline_picked(&mut self) {
        let Some(o) = self.outline.as_ref() else { return };
        if o.quiet.get() {
            return;
        }
        let Some(n) = win::tree_selected(o.tree) else { return };
        // A code file's symbol: there in the editor.
        if let (Some(h), Some(&at)) = (self.doc.code, usize::try_from(n).ok().and_then(|i| o.places.get(i))) {
            phai_editor::view::select_lsp(h, at, at);
            return;
        }
        let pick = (n >= 0).then_some(n as usize);
        if pick != self.selected {
            self.select(pick);
        }
    }

    pub fn refresh_props(&mut self) {
        if let Some(p) = self.props.as_mut() {
            p.refresh();
        }
    }

    /// Everything chosen, the first one first.
    pub fn selection(&self) -> Vec<usize> {
        self.selected.into_iter().chain(self.also.iter().copied()).collect()
    }

    /// Ctrl+click: in or out of the choice.
    pub fn toggle(&mut self, i: usize) {
        if self.selected == Some(i) {
            self.selected = if self.also.is_empty() { None } else { Some(self.also.remove(0)) };
        } else if let Some(k) = self.also.iter().position(|x| *x == i) {
            self.also.remove(k);
        } else if self.selected.is_none() {
            self.selected = Some(i);
        } else {
            self.also.push(i);
        }
        self.refresh_props();
        self.say_selection();
    }

    pub fn make_primary(&mut self, i: usize) {
        if let Some(k) = self.also.iter().position(|x| *x == i) {
            self.also.remove(k);
            if let Some(old) = self.selected.replace(i) {
                self.also.insert(0, old);
            }
            self.refresh_props();
        }
    }

    pub fn select_many(&mut self, all: &[usize]) {
        self.selected = all.first().copied();
        self.also = all.iter().skip(1).copied().collect();
        self.refresh_props();
        if let Some(c) = self.canvas.as_ref() {
            c.place_handles();
        }
        self.say_selection();
    }

    pub fn select(&mut self, i: Option<usize>) {
        self.also.clear();
        if self.selected != i {
            self.selected = i;
            self.refresh_props();
        }
        if let Some(c) = self.canvas.as_ref() {
            c.place_handles();
        }
        self.say_selection();
    }

    pub fn delete_selected(&mut self) {
        let mut chosen = self.selection();
        if chosen.is_empty() {
            return;
        }
        chosen.sort_unstable();
        self.edit(|f| {
            for i in chosen.iter().rev() {
                f.control.remove(*i);
            }
        });
        self.selected = None;
        self.also.clear();
        self.rebuild();
        self.refresh_props();
        self.say_selection();
    }

    fn undo_redo(&mut self, back: bool) {
        let (from, to) = if back { (&mut self.undo, &mut self.redo) } else { (&mut self.redo, &mut self.undo) };
        let Some(f) = from.pop() else { return };
        to.push(std::mem::replace(&mut self.doc.form, f));
        if self.selected.is_some_and(|i| i >= self.doc.form.control.len()) {
            self.selected = None;
        }
        let n = self.doc.form.control.len();
        self.also.retain(|i| *i < n);
        self.changed();
        self.rebuild();
        self.refresh_props();
        self.say_selection();
    }

    /// Double-click: give the control a handler if it has none, and put the empty method in the
    /// programmer's file if the form is saved and the method is not there yet.
    pub fn add_handler_for_selected(&mut self) {
        // The form itself: its code, at the first method - or a new `loaded`, which runs as the
        // window opens, as a WinForms form's double-click writes Form_Load.
        let Some(i) = self.selected else {
            let Some(path) = self.doc.path.clone() else { return };
            let user = path.with_file_name(format!("{}.rs", snake(&self.doc.form.form.name)));
            let Ok(src) = std::fs::read_to_string(&user) else { return };
            if first_method_line(&src).is_none() {
                if let Some(more) = codegen::add_handler(&src, &self.doc.form.form.name, "loaded") {
                    let _ = std::fs::write(&user, more);
                }
            }
            let src = std::fs::read_to_string(&user).unwrap_or_default();
            self.open_code_at(&user, first_method_line(&src));
            return;
        };
        let c = &self.doc.form.control[i];
        let Some(event) = c.kind.event() else {
            // No one event of its own: its Click, as a WinForms label's double-click writes.
            if phai_form::events::find(c.kind, "click").is_some() {
                self.add_event_method(Key::On("click"));
            }
            return;
        };
        let name = c.handler().map(str::to_string).unwrap_or_else(|| {
            let verb = match event {
                "on_click" => "clicked",
                "on_change" => "changed",
                _ => "chosen",
            };
            format!("{}_{verb}", c.id)
        });
        if c.handler() != Some(name.as_str()) {
            let n = name.clone();
            self.edit(|f| f.control[i].set_handler(Some(n)));
            self.refresh_props();
        }
        let Some(path) = self.doc.path.clone() else { return };
        let user = path.with_file_name(format!("{}.rs", snake(&self.doc.form.form.name)));
        if let Ok(src) = std::fs::read_to_string(&user) {
            if let Some(more) = codegen::add_handler(&src, &self.doc.form.form.name, &name) {
                if std::fs::write(&user, more).is_ok() {
                    controls::status_text(self.status, 0, &tf("Added {0} to {1}", &[&format!("fn {name}"), &user.file_name().unwrap_or_default().to_string_lossy()]));
                }
            }
        }
        // Then to it, as Visual Studio goes to the handler it writes.
        let src = std::fs::read_to_string(&user).unwrap_or_default();
        let line = src.lines().position(|l| l.trim_start().starts_with(&format!("fn {name}("))).map(|i| i + 1);
        self.open_code_at(&user, line);
    }

    /// A double-click on an event in the Events tab: the method it names - or a new one, named as
    /// Visual Studio names it (`name_key_down`, `main_window_closing`) - written into the
    /// programmer's file with what the event gives it, and the code opened there.
    pub fn add_event_method(&mut self, key: Key) {
        let form = self.doc.form.form.name.clone();
        let (name, param) = match (key, self.selected) {
            (Key::Event, _) => return self.add_handler_for_selected(),
            (Key::FormOn(e), _) => {
                let Some(spec) = phai_form::events::find_form(e) else { return };
                let name = self.doc.form.form.events.get(e).cloned().unwrap_or_else(|| format!("{}_{e}", snake(&form)));
                if !self.doc.form.form.events.contains_key(e) {
                    let n = name.clone();
                    self.edit(|f| {
                        f.form.events.insert(e.to_string(), n);
                    });
                }
                (name, spec.args.param())
            }
            (Key::On(e), Some(i)) => {
                let c = &self.doc.form.control[i];
                let Some(spec) = phai_form::events::find(c.kind, e) else { return };
                let name = c.events.get(e).cloned().unwrap_or_else(|| phai_form::events::method_name(&c.id, e));
                if !c.events.contains_key(e) {
                    let n = name.clone();
                    self.edit(|f| {
                        f.control[i].events.insert(e.to_string(), n);
                    });
                }
                (name, spec.args.param())
            }
            _ => return,
        };
        self.refresh_props();
        let Some(path) = self.doc.path.clone() else { return };
        // Saved first, so the generated trait has the method the new code implements.
        self.save(false);
        let user = path.with_file_name(format!("{}.rs", snake(&form)));
        if let Ok(src) = std::fs::read_to_string(&user) {
            if let Some(more) = codegen::add_handler_with(&src, &form, &name, param) {
                if std::fs::write(&user, more).is_ok() {
                    controls::status_text(self.status, 0, &tf("Added {0} to {1}", &[&format!("fn {name}"), &user.file_name().unwrap_or_default().to_string_lossy()]));
                }
            }
        }
        let src = std::fs::read_to_string(&user).unwrap_or_default();
        let line = src.lines().position(|l| l.trim_start().starts_with(&format!("fn {name}("))).map(|i| i + 1);
        self.open_code_at(&user, line);
    }

    /// A code file in its tab, the caret on `line` (from 1) - inside the method's body when the
    /// line opens one.
    fn open_code_at(&mut self, path: &Path, line: Option<usize>) {
        self.open_code(path);
        if let (Some(code), Some(l)) = (self.doc.code, line) {
            phai_editor::view::go_to(code, l + 1, 9);
        }
    }

    pub fn toolbox_reset(&mut self) {
        toolbox::choose(self.toolbox, ID_TOOL);
    }

    /// The form's size as the canvas shows it now.
    pub fn drag_size(&self) -> [i32; 2] {
        self.canvas.as_ref().and_then(|c| c.shown_size()).unwrap_or(self.doc.form.form.size)
    }

    // --- clipboard and format ---------------------------------------------------------------------

    fn chosen_controls(&self) -> Vec<phai_form::Control> {
        self.selection().into_iter().filter_map(|i| self.doc.form.control.get(i).cloned()).collect()
    }

    fn copy(&mut self) {
        let chosen = self.chosen_controls();
        if !chosen.is_empty() {
            set_clipboard(self.hwnd, &phai_form::arrange::to_clipboard(&chosen));
        }
    }

    /// Adds controls read from `text` and chooses them.
    fn add_controls(&mut self, text: &str) {
        let Some(new) = phai_form::arrange::from_clipboard(text, &self.doc.form, canvas::GRID * 2) else { return };
        let first = self.doc.form.control.len();
        let count = new.len();
        self.edit(|f| f.control.extend(new));
        self.rebuild();
        self.select_many(&(first..first + count).collect::<Vec<_>>());
    }

    fn paste(&mut self) {
        if let Some(text) = get_clipboard(self.hwnd) {
            self.add_controls(&text);
        }
    }

    fn duplicate(&mut self) {
        // In code: the line again.
        if let Some(h) = self.code_shown() {
            phai_editor::view::duplicate(h);
            return;
        }
        let chosen = self.chosen_controls();
        if !chosen.is_empty() {
            self.add_controls(&phai_form::arrange::to_clipboard(&chosen));
        }
    }

    /// Format > Bring to front / Send to back. Windows stacks the first child made on top and
    /// visits it first with Tab, so the front is the start of the form's list, the back its end.
    fn restack(&mut self, front: bool) {
        let mut chosen = self.selection();
        if chosen.is_empty() {
            return;
        }
        chosen.sort_unstable();
        let mut moved: Vec<usize> = Vec::new();
        self.edit(|f| {
            let taken: Vec<phai_form::Control> = chosen.iter().rev().map(|i| f.control.remove(*i)).collect::<Vec<_>>().into_iter().rev().collect();
            let n = taken.len();
            if front {
                for (k, c) in taken.into_iter().enumerate() {
                    f.control.insert(k, c);
                }
                moved = (0..n).collect();
            } else {
                let start = f.control.len();
                f.control.extend(taken);
                moved = (start..start + n).collect();
            }
        });
        self.rebuild();
        self.select_many(&moved);
    }

    /// The toolbox double-clicked: the kind goes on the form at once, as in Visual Studio, at the
    /// margin or a little below and right of what is already there.
    pub fn add_at_margin(&mut self, kind: Kind) {
        let m = phai_design::metrics::MARGIN;
        let (w, h) = kind.default_size();
        let taken: Vec<(i32, i32)> = self.doc.form.control.iter().filter_map(|c| c.bounds).map(|b| (b[0], b[1])).collect();
        let mut at = (m, m + if self.doc.form.toolbar.is_empty() { 0 } else { phai_ui::win::TOOLBAR_HEIGHT });
        while taken.contains(&at) {
            at = (at.0 + canvas::GRID * 2, at.1 + canvas::GRID * 2);
        }
        self.edit(|f| {
            let id = f.next_id(kind);
            let mut c = phai_form::Control::new(kind, &id, [at.0, at.1, w, h]);
            if kind == Kind::Statusbar {
                c.bounds = Some([0, 0, 0, 26]);
            }
            f.control.push(c);
        });
        self.tool = None;
        self.toolbox_reset();
        self.rebuild();
        let last = self.doc.form.control.len() - 1;
        self.select(Some(last));
    }

    /// Lines the chosen controls up against the first one chosen.
    fn format(&mut self, op: phai_form::arrange::Op) {
        let chosen: Vec<usize> = self.selection().into_iter().filter(|i| self.doc.form.control[*i].dock.as_deref().is_none_or(|d| d == "none")).collect();
        if chosen.is_empty() || chosen.len() < 2 && !op.works_alone() {
            return;
        }
        let mut rects: Vec<[i32; 4]> = chosen.iter().map(|i| self.doc.form.control[*i].rect()).collect();
        phai_form::arrange::arrange_in(&mut rects, 0, op, self.doc.form.form.size, canvas::GRID * 2);
        self.edit(|f| {
            for (i, r) in chosen.iter().zip(&rects) {
                f.control[*i].bounds = Some(*r);
            }
        });
        self.rebuild();
        self.refresh_props();
        self.say_selection();
    }

    // --- menus and toolbar -----------------------------------------------------------------------

    /// The form's menus as text in one box; asked again until it reads or is cancelled.
    pub fn edit_menus(&mut self) {
        let mut text = phai_form::bars::menus_to_text(&self.doc.form.menu);
        if text.is_empty() {
            text = "&File\n  exit | Exit\n".into();
        }
        loop {
            let Some(answer) = phai_ui::dialogs::ask_lines(Some(self.hwnd), t("Menus"), t("A menu at the left; its items indented: id | label | key. A line of - separates."), &text) else { return };
            match phai_form::bars::menus_from_text(&answer) {
                Ok(menus) => {
                    self.edit(|f| f.menu = menus);
                    self.rebuild();
                    return;
                }
                Err(e) => {
                    win::message(Some(self.hwnd), &e, t("Menus"), wm::MB_ICONWARNING);
                    text = answer;
                }
            }
        }
    }

    /// The form's toolbar as text: one button a line, id | label | GLYPH.
    pub fn edit_toolbar(&mut self) {
        let mut text = phai_form::bars::toolbar_to_text(&self.doc.form.toolbar);
        if text.is_empty() {
            text = "save | Save | SAVE\n".into();
        }
        loop {
            let Some(answer) = phai_ui::dialogs::ask_lines(Some(self.hwnd), t("Toolbar"), t("One button a line: id | label | GLYPH. The label is the tooltip."), &text) else { return };
            match phai_form::bars::toolbar_from_text(&answer) {
                Ok(tools) => {
                    self.edit(|f| f.toolbar = tools);
                    self.rebuild();
                    return;
                }
                Err(e) => {
                    win::message(Some(self.hwnd), &e, t("Toolbar"), wm::MB_ICONWARNING);
                    text = answer;
                }
            }
        }
    }

    // --- tabs -----------------------------------------------------------------------------------

    /// The form on the canvas, packed away into its tab.
    fn stash(&mut self) {
        self.unsplit();
        self.close_windows();
        let doc = std::mem::replace(&mut self.doc, Doc::new(FormFile::new("MainWindow"), None));
        self.tabs[self.active] = Tab { doc, undo: std::mem::take(&mut self.undo), redo: std::mem::take(&mut self.redo), selected: self.selected.take() };
    }

    /// Tab `i` onto the canvas.
    fn activate(&mut self, i: usize) {
        if i >= self.tabs.len() {
            return;
        }
        if i != self.active {
            self.stash();
            self.active = i;
        }
        let tab = std::mem::replace(&mut self.tabs[i], Tab::empty());
        self.doc = tab.doc;
        self.undo = tab.undo;
        self.redo = tab.redo;
        self.selected = tab.selected;
        self.after_switch();
    }

    /// Shows the active tab's editor and hides every other one; the canvas shows when it is a form.
    fn show_code(&mut self) {
        let active = self.doc.code;
        for t in &self.tabs {
            if let Some(h) = t.doc.code {
                win::show(h, false);
            }
        }
        if let Some(c) = self.canvas.as_ref() {
            win::show(c.surface, active.is_none());
            if let Some(h) = active {
                let mut r = RECT::default();
                unsafe {
                    let _ = wm::GetClientRect(c.hwnd, &mut r);
                }
                win::move_to(h, 0, 0, r.right, r.bottom);
                unsafe {
                    // Children made later sit lower: lift the editor over the form's windows.
                    let _ = wm::SetWindowPos(h, Some(wm::HWND_TOP), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_SHOWWINDOW);
                }
                unsafe {
                    let _ = km::SetFocus(Some(h));
                }
            }
        }
    }

    /// Opens a code file in an editor tab - or the tab it is already in.
    pub fn open_code(&mut self, path: &Path) {
        let same = |q: &Option<PathBuf>| q.as_deref().is_some_and(|q| phai_editor::lsp::same_path(q, path));
        if same(&self.doc.path) {
            return self.after_switch();
        }
        if let Some(i) = self.tabs.iter().enumerate().position(|(i, t)| i != self.active && same(&t.doc.path)) {
            return self.activate(i);
        }
        let Ok(raw) = std::fs::read_to_string(path) else {
            win::message(Some(self.hwnd), &tf("{0} cannot be opened", &[&path.display()]), "Phai Studio", wm::MB_ICONWARNING);
            return;
        };
        let (text, crlf) = phai_editor::buffer::normalize(&raw);
        let Some(parent) = self.canvas.as_ref().map(|c| c.hwnd) else { return };
        let h = phai_editor::view::create(parent, 900, &text, &path.to_string_lossy());
        if let Some(b) = self.bps.iter().find(|(p, _)| phai_editor::lsp::same_path(p, path)).map(|(_, b)| b.clone()) {
            phai_editor::view::set_breakpoints(h, &b);
        }
        let blank = self.doc.path.is_none() && !self.doc.dirty && self.doc.form.control.is_empty() && self.doc.code.is_none();
        if !blank {
            self.stash();
            self.tabs.push(Tab::empty());
            self.active = self.tabs.len() - 1;
        }
        self.doc = Doc { code: Some(h), crlf, ..Doc::new(FormFile::new("Code"), Some(path.to_path_buf())) };
        if path.extension().is_some_and(|e| e == "rs") {
            if let Some(l) = self.lsp_for(path) {
                l.did_open(path, &text);
            }
        }
        self.undo.clear();
        self.redo.clear();
        self.selected = None;
        self.after_switch();
    }

    /// rust-analyzer for the project holding `path`, started if it is not running for it already.
    fn lsp_for(&mut self, path: &Path) -> Option<&mut phai_editor::lsp::Lsp> {
        // The standard library and fetched crates, reached by F12, are the running server's too.
        if cargo::is_library(path) && self.lsp.is_some() {
            return self.lsp.as_mut();
        }
        let root = cargo::workspace_of(path)?;
        if self.lsp.as_ref().is_none_or(|l| !phai_editor::lsp::same_path(&l.root, &root)) {
            self.lsp = phai_editor::lsp::Lsp::start(&root, self.hwnd);
            if self.lsp.is_none() {
                controls::status_text(self.status, 0, t("rust-analyzer is not installed: rustup component add rust-analyzer"));
            }
        }
        self.lsp.as_mut()
    }

    /// The editor of the open file at `path`, in any tab.
    fn editor_of(&self, path: &Path) -> Option<HWND> {
        let same = |p: &Option<PathBuf>| p.as_deref().is_some_and(|q| phai_editor::lsp::same_path(q, path));
        if same(&self.doc.path) {
            return self.doc.code;
        }
        self.tabs.iter().find(|t| same(&t.doc.path)).and_then(|t| t.doc.code)
    }

    /// A message from rust-analyzer.
    fn lsp_message(&mut self, msg: serde_json::Value) {
        // A request of ours answered.
        if let (Some(id), None) = (msg.get("id").and_then(|i| i.as_i64()), msg.get("method")) {
            if let Some((h, ask)) = self.asked.remove(&id) {
                let result = msg.get("result").cloned().unwrap_or_default();
                let error = msg["error"]["message"].as_str().map(str::to_string);
                self.lsp_answer(h, ask, result, error);
            }
            return;
        }
        // The server asking something of us: it waits for an answer.
        if let (Some(id), Some(_)) = (msg.get("id"), msg.get("method")) {
            if let Some(l) = self.lsp.as_ref() {
                l.respond(id);
            }
            return;
        }
        if std::env::var_os("PHAI_PRINT_DIAGS").is_some() && (msg["method"] == "textDocument/publishDiagnostics" || msg["method"] == "experimental/serverStatus") {
            println!("{}: {}", msg["method"].as_str().unwrap_or(""), msg["params"]);
        }
        // Still loading (the project, the standard library): its findings wait - "unresolved macro
        // `println!`" is only that it has not read std yet. Once quiet, the last of each are shown.
        if msg["method"] == "experimental/serverStatus" {
            let quiet = msg["params"]["quiescent"].as_bool().unwrap_or(true);
            self.lsp_loading = !quiet;
            if quiet {
                for (path, diags) in std::mem::take(&mut self.held_diagnostics) {
                    self.show_lsp_diagnostics(&path, &diags);
                }
            }
            return;
        }
        if let Some((path, diags)) = phai_editor::lsp::diagnostics_of(&msg) {
            if self.lsp_loading {
                self.held_diagnostics.insert(path, diags);
                return;
            }
            self.show_lsp_diagnostics(&path, &diags);
        }
    }

    /// rust-analyzer's findings for a file drawn under its code; the counts said when it is in front.
    fn show_lsp_diagnostics(&mut self, path: &Path, diags: &[phai_editor::lsp::Diagnostic]) {
        if std::env::var_os("PHAI_PRINT_DIAGS").is_some() {
            println!("shown: {} for {}", diags.len(), path.file_name().unwrap_or_default().to_string_lossy());
        }
        {
            if let Some(h) = self.editor_of(path) {
                phai_editor::view::set_diagnostics(h, diags);
                if Some(h) == self.doc.code {
                    let (e, w) = phai_editor::view::diagnostic_counts(h);
                    controls::status_text(self.status, 0, &tf("{0} errors, {1} warnings", &[&e, &w]));
                    self.refresh_outline();
                    self.refresh_lenses();
                }
            }
        }
    }

    fn lsp_answer(&mut self, h: HWND, ask: Ask, result: serde_json::Value, error: Option<String>) {
        match ask {
            Ask::Hover => {
                if let Some(text) = phai_editor::lsp::hover_text(&result) {
                    phai_editor::view::show_tip(h, &text);
                }
            }
            Ask::Complete => phai_editor::view::show_completions(h, phai_editor::lsp::completions_of(&result, 300)),
            Ask::Definition => match phai_editor::lsp::locations_of(&result).into_iter().next() {
                Some(place) => {
                    self.remember_place();
                    self.open_code(&place.path);
                    if let Some(code) = self.doc.code {
                        phai_editor::view::select_lsp(code, place.start, place.end);
                    }
                    let (l, _) = phai_editor::view::caret_position(self.doc.code.unwrap_or(h));
                    controls::status_text(self.status, 0, &format!("{}:{l}", place.path.file_name().unwrap_or_default().to_string_lossy()));
                }
                None => controls::status_text(self.status, 0, error.as_deref().unwrap_or(t("No definition found here"))),
            },
            Ask::Actions => self.choose_action(h, phai_editor::lsp::actions_of(&result)),
            Ask::LensSymbols => {
                // A new round: its uses counted afresh for every function, method, type and trait.
                self.lens_round += 1;
                self.lenses.clear();
                let round = self.lens_round;
                fn counted(list: &[phai_editor::lsp::DocSymbol], out: &mut Vec<(u32, u32)>) {
                    for s in list {
                        if matches!(s.kind, "Function" | "Method" | "Struct" | "Enum" | "Trait" | "Class" | "Constant") {
                            out.push(s.start);
                        }
                        counted(&s.children, out);
                    }
                }
                let mut places = Vec::new();
                counted(&phai_editor::lsp::doc_symbols_of(&result), &mut places);
                if let Some(path) = self.doc.path.clone() {
                    for at in places.into_iter().take(80) {
                        if let Some(c) = self.lsp_for(&path) {
                            let id = c.uses(&path, at.0, at.1);
                            self.asked.insert(id, (h, Ask::LensUses(round, at.0, at)));
                        }
                    }
                }
                self.show_lenses();
            }
            Ask::LensUses(round, line, at) => {
                if round == self.lens_round {
                    let n = result.as_array().map(|a| a.len()).unwrap_or(0);
                    let words = if n == 1 { t("1 reference").to_string() } else { tf("{0} references", &[&n]) };
                    self.lenses.retain(|x| x.0 != line);
                    self.lenses.push((line, at, words));
                    if std::env::var_os("PHAI_LENS_TEST").is_some() {
                        let mut all = self.lenses.clone();
                        all.sort();
                        println!("lenses: {}", all.iter().map(|(l, _, w)| format!("line {} {w}", l + 1)).collect::<Vec<_>>().join(", "));
                    }
                    self.show_lenses();
                }
            }
            Ask::Outline => {
                let symbols = phai_editor::lsp::doc_symbols_of(&result);
                if std::env::var_os("PHAI_OUTLINE_TEST").is_some() && !symbols.is_empty() {
                    fn names(list: &[phai_editor::lsp::DocSymbol], out: &mut Vec<String>, depth: usize) {
                        for s in list {
                            out.push(format!("{}{} ({})", "  ".repeat(depth), s.name, s.kind));
                            names(&s.children, out, depth + 1);
                        }
                    }
                    let mut out = Vec::new();
                    names(&symbols, &mut out, 0);
                    println!("outline:\n{}", out.join("\n"));
                    LSP_TEST.store(9, Ordering::Relaxed);
                    if std::env::var_os("PHAI_SHOT").is_none() {
                        std::process::exit(0);
                    }
                }
                let file = self.doc.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                if let (Some(o), true) = (self.outline.as_mut(), self.doc.code == Some(h)) {
                    o.fill_code(&file, &symbols);
                }
            }
            Ask::PrepareCalls => match result.as_array().and_then(|a| a.first()).cloned() {
                Some(item) => {
                    let name = item["name"].as_str().unwrap_or("").to_string();
                    if let Some(path) = self.doc.path.clone() {
                        if let Some(l) = self.lsp_for(&path) {
                            let id = l.incoming_calls(item);
                            self.asked.insert(id, (h, Ask::Calls(name)));
                        }
                    }
                }
                None => controls::status_text(self.status, 0, t("Put the caret on a function's name first")),
            },
            Ask::Calls(name) => {
                let calls = phai_editor::lsp::calls_of(&result);
                if std::env::var_os("PHAI_CALLS_TEST").is_some() {
                    println!("calls of {name}: {}", calls.iter().map(|(l, f)| format!("{f} line {}", l.start.0 + 1)).collect::<Vec<_>>().join(", "));
                    std::process::exit(0);
                }
                self.found_words = "{0} calls in {1} files";
                self.show_references(calls.into_iter().map(|(l, _)| l).collect());
                controls::status_text(self.status, 0, &tf("Calls to {0}: {1}", &[&name, &self.references.len()]));
            }
            Ask::Symbols => {
                let items = phai_editor::lsp::symbols_of(&result)
                    .into_iter()
                    .filter(|s| !cargo::is_library(&s.path))
                    .map(|s| gotoall::Item { name: s.name, kind: t(s.kind).to_string(), path: s.path, at: Some(s.start) })
                    .collect();
                gotoall::set_symbols(items);
            }
            Ask::Signature => {
                if let Some(text) = phai_editor::lsp::signature_text(&result) {
                    if std::env::var_os("PHAI_SIG_TEST").is_some() {
                        println!("signature {}", text.lines().next().unwrap_or_default());
                        LSP_TEST.store(9, Ordering::Relaxed);
                    }
                    phai_editor::view::show_peek(h, &text);
                }
            }
            Ask::Peek => match phai_editor::lsp::locations_of(&result).into_iter().next() {
                Some(place) => {
                    let text = std::fs::read_to_string(&place.path).unwrap_or_default();
                    let text = peek_text(&place.path, place.start.0 as usize, &text);
                    if std::env::var_os("PHAI_PEEK_TEST").is_some() {
                        println!("peek {}", text.lines().take(3).collect::<Vec<_>>().join(" | "));
                    }
                    self.open_peek(h, &place.path, place.start.0 as usize + 1);
                    // `PHAI_PEEK_TYPE`: words typed in the peek, then the tab's first line printed.
                    if let (Some(words), Some((_, pe, _, _))) = (std::env::var("PHAI_PEEK_TYPE").ok(), self.peek.clone()) {
                        phai_editor::view::go_to(pe, 1, 1);
                        for u in format!("{words}
").encode_utf16() {
                            let u = if u == 10 { 13 } else { u };
                            unsafe { wm::SendMessageW(pe, wm::WM_CHAR, Some(WPARAM(u as usize)), Some(LPARAM(1))) };
                        }
                        println!("tab starts: {}", phai_editor::view::text(h).lines().next().unwrap_or_default());
                        println!("peek starts: {}", phai_editor::view::text(pe).lines().next().unwrap_or_default());
                    }
                    if std::env::var_os("PHAI_PEEK_TEST").is_some() {
                        LSP_TEST.store(9, Ordering::Relaxed);
                        if std::env::var_os("PHAI_SHOT").is_none() {
                            std::process::exit(0);
                        }
                    }
                }
                None => controls::status_text(self.status, 0, error.as_deref().unwrap_or(t("No definition found here"))),
            },
            Ask::PrepareRename(path, pos) => {
                let text = phai_editor::view::text(h);
                let Some(old) = phai_editor::lsp::rename_placeholder(&result, &text) else {
                    controls::status_text(self.status, 0, error.as_deref().unwrap_or(t("This cannot be renamed")));
                    return;
                };
                // `--lsptest` names it without the box.
                let test = std::env::var("PHAI_RENAME_TO").ok();
                let Some(new) = test.or_else(|| phai_ui::dialogs::ask(Some(self.hwnd), t("Rename"), &tf("New name for {0}, in every file that uses it:", &[&old]), &old, false)) else { return };
                let new = new.trim().to_string();
                if new.is_empty() || new == old {
                    return;
                }
                if let Some(l) = self.lsp_for(&path) {
                    let id = l.rename(&path, pos.0, pos.1, &new);
                    self.asked.insert(id, (h, Ask::Rename { old, new }));
                }
            }
            Ask::References => {
                let found = phai_editor::lsp::locations_of(&result);
                self.show_references(found);
            }
            Ask::Rename { old, new } => {
                if let Some(e) = error {
                    win::message(Some(self.hwnd), &e, t("Rename"), wm::MB_ICONWARNING);
                    return;
                }
                let (places, files) = self.apply_workspace_edit(&result);
                let said = tf("Renamed {0} to {1}: {2} places in {3} files", &[&old, &new, &places, &files]);
                if LSP_TEST.load(Ordering::Relaxed) > 0 {
                    println!("{said}");
                }
                controls::status_text(self.status, 0, &said);
            }
        }
    }

    /// Makes a rename's changes: in the editor of a file that is open (one undo step, left unsaved
    /// as Visual Studio leaves it), on disk for one that is not. How many places, in how many files.
    fn apply_workspace_edit(&mut self, result: &serde_json::Value) -> (usize, usize) {
        let mut places = 0;
        let mut files = 0;
        for (path, edits) in phai_editor::lsp::workspace_edit_of(result) {
            if edits.is_empty() {
                continue;
            }
            files += 1;
            if let Some(h) = self.editor_of(&path) {
                places += phai_editor::view::apply_lsp_edits(h, &edits);
                if Some(h) != self.doc.code {
                    // Not the file in front: its tab is changed, and the server told.
                    let text = phai_editor::view::text(h);
                    let version = phai_editor::view::version(h);
                    if let Some(l) = self.lsp.as_ref() {
                        l.did_change(&path, version, &text);
                    }
                    if let Some(tab) = self.tabs.iter_mut().find(|t| t.doc.code == Some(h)) {
                        tab.doc.dirty = true;
                    }
                }
                continue;
            }
            let Ok(raw) = std::fs::read_to_string(&path) else { continue };
            let (text, crlf) = phai_editor::buffer::normalize(&raw);
            places += phai_editor::lsp::edit_spans(&text, &edits).len();
            let out = phai_editor::lsp::apply_edits(&text, &edits);
            let out = if crlf { out.replace('\n', "\r\n") } else { out };
            if std::fs::write(&path, out).is_err() {
                win::message(Some(self.hwnd), &tf("{0} cannot be saved", &[&path.display()]), t("Rename"), wm::MB_ICONWARNING);
            }
        }
        self.refresh_tabs();
        (places, files)
    }

    /// `--lsptest`, one step: the open file's second line calls a function the first defines. F12
    /// on the call must land on line 1; F2 then renames it to `PHAI_RENAME_TO` everywhere. A step
    /// not done yet is asked again each second, while rust-analyzer loads.
    fn lsp_test_step(&mut self) {
        let Some(h) = self.doc.code else { std::process::exit(3) };
        let to = std::env::var("PHAI_RENAME_TO").unwrap_or_else(|_| "renamed".into());
        let on_call = |h: HWND| {
            let text = phai_editor::view::text(h);
            let second = text.lines().nth(1).unwrap_or_default();
            let name = text.lines().next().unwrap_or_default().trim_start_matches("pub ").trim_start_matches("fn ").split('(').next().unwrap_or_default().to_string();
            let col = second.find(&format!("{name}(")).map(|b| second[..b].chars().count() + 2).unwrap_or(1);
            phai_editor::view::go_to(h, 2, col);
        };
        match LSP_TEST.load(Ordering::Relaxed) {
            // `PHAI_ACTION`: Quick Actions at `PHAI_ACTION_AT` (line:column), asked until some come.
            0 | 1 if std::env::var_os("PHAI_ACTION").is_some() => {
                let at = std::env::var("PHAI_ACTION_AT").unwrap_or_else(|_| "6:9".into());
                let (l, c) = at.split_once(':').and_then(|(l, c)| Some((l.parse().ok()?, c.parse().ok()?))).unwrap_or((6, 9));
                phai_editor::view::go_to(h, l, c);
                self.lsp_actions(h);
                LSP_TEST.store(1, Ordering::Relaxed);
            }
            // `PHAI_CALLS_TEST`: Call Hierarchy of the function on line 1, asked until it answers.
            0 | 1 if std::env::var_os("PHAI_CALLS_TEST").is_some() => {
                phai_editor::view::go_to(h, 1, 5);
                self.command(ID_CALLS);
                LSP_TEST.store(1, Ordering::Relaxed);
            }
            // `PHAI_LENS_TEST`: the code lenses asked for again and again; printed as they come.
            0 | 1 if std::env::var_os("PHAI_LENS_TEST").is_some() => {
                self.refresh_lenses();
                LSP_TEST.store(1, Ordering::Relaxed);
            }
            // `PHAI_OUTLINE_TEST`: Document Outline of the code file, asked until it answers.
            0 | 1 if std::env::var_os("PHAI_OUTLINE_TEST").is_some() => {
                if !explorer::outline_shown(self.side) {
                    self.command(ID_VIEW_OUTLINE);
                } else {
                    self.refresh_outline();
                }
                LSP_TEST.store(1, Ordering::Relaxed);
            }
            // `PHAI_SIG_TEST`: Parameter Info inside the call on line 2, asked until it answers.
            0 | 1 if std::env::var_os("PHAI_SIG_TEST").is_some() => {
                on_call(h);
                let (l, c) = phai_editor::view::caret_position(h);
                phai_editor::view::go_to(h, l, c + 6);
                self.lsp_signature(h);
                LSP_TEST.store(1, Ordering::Relaxed);
            }
            9 if std::env::var_os("PHAI_SIG_TEST").is_some() && std::env::var_os("PHAI_SHOT").is_none() => std::process::exit(0),
            // `PHAI_PEEK_TEST`: Peek Definition instead, asked again until it answers.
            0 | 1 if std::env::var_os("PHAI_PEEK_TEST").is_some() => {
                on_call(h);
                self.lsp_peek(h);
                LSP_TEST.store(1, Ordering::Relaxed);
            }
            0 => {
                on_call(h);
                self.lsp_definition(h);
                LSP_TEST.store(1, Ordering::Relaxed);
            }
            1 if phai_editor::view::caret_position(h).0 == 1 => {
                println!("definition: line 1");
                LSP_TEST.store(2, Ordering::Relaxed);
            }
            2 if !self.references.is_empty() => LSP_TEST.store(3, Ordering::Relaxed),
            2 => {
                on_call(h);
                self.lsp_references(h);
            }
            1 => {
                on_call(h);
                self.lsp_definition(h);
            }
            3 if phai_editor::view::text(h).contains(&format!("{to}(")) => {
                self.save(false);
                LSP_TEST.store(4, Ordering::Relaxed);
                // Being photographed, it stays for the picture.
                if std::env::var_os("PHAI_SHOT").is_none() {
                    std::process::exit(0);
                }
            }
            3 => {
                on_call(h);
                self.lsp_rename(h);
            }
            _ => {}
        }
    }

    /// F12 or Ctrl+click in an editor: to where the thing at the caret is defined.
    pub fn lsp_definition(&mut self, h: HWND) {
        let Some(path) = self.doc.path.clone() else { return };
        let pos = phai_editor::view::caret_lsp(h);
        match self.lsp_for(&path) {
            Some(l) => {
                let id = l.definition(&path, pos.0, pos.1);
                self.asked.insert(id, (h, Ask::Definition));
            }
            None => controls::status_text(self.status, 0, t("rust-analyzer is not installed: rustup component add rust-analyzer")),
        }
    }

    /// Ctrl+. in an editor: the fixes and refactorings the language server offers here.
    pub fn lsp_actions(&mut self, h: HWND) {
        let Some(path) = self.doc.path.clone() else { return };
        let (a, b) = phai_editor::view::selection_lsp(h);
        match self.lsp_for(&path) {
            Some(l) => {
                let id = l.code_actions(&path, a, b);
                self.asked.insert(id, (h, Ask::Actions));
            }
            None => controls::status_text(self.status, 0, t("rust-analyzer is not installed: rustup component add rust-analyzer")),
        }
    }

    /// The Quick Actions offered, as a menu under the caret; the one chosen made.
    fn choose_action(&mut self, h: HWND, actions: Vec<phai_editor::lsp::Action>) {
        if actions.is_empty() {
            controls::status_text(self.status, 0, t("No quick actions here"));
            return;
        }
        // `PHAI_ACTION=words`: the first action whose title has them, without the menu.
        let chosen = if let Ok(want) = std::env::var("PHAI_ACTION") {
            println!("actions: {}", actions.iter().map(|a| a.title.as_str()).collect::<Vec<_>>().join(" | "));
            actions.iter().position(|a| a.title.to_lowercase().contains(&want.to_lowercase()))
        } else {
            unsafe {
                let m = wm::CreatePopupMenu().unwrap_or_default();
                for (i, a) in actions.iter().take(40).enumerate() {
                    let w = wide(&a.title);
                    let _ = wm::AppendMenuW(m, wm::MF_STRING, i + 1, PCWSTR(w.as_ptr()));
                }
                let (x, y) = phai_editor::view::caret_screen(h);
                let n = wm::TrackPopupMenu(m, wm::TPM_RETURNCMD | wm::TPM_LEFTALIGN | wm::TPM_TOPALIGN, x, y, None, self.hwnd, None).0 as usize;
                let _ = wm::DestroyMenu(m);
                n.checked_sub(1)
            }
        };
        let Some(a) = chosen.and_then(|i| actions.get(i)).cloned() else { return };
        let (places, files) = self.apply_workspace_edit(&a.edit);
        let said = tf("{0}: {1} changes in {2} files", &[&a.title, &places, &files]);
        if std::env::var_os("PHAI_ACTION").is_some() {
            println!("{said}");
            println!("now: {}", phai_editor::view::text(h).lines().nth(5).unwrap_or_default().trim());
            if std::env::var_os("PHAI_SHOT").is_none() {
                std::process::exit(0);
            }
        }
        controls::status_text(self.status, 0, &said);
    }

    /// Parameter Info in an editor: what the call the caret is in takes, under the caret.
    pub fn lsp_signature(&mut self, h: HWND) {
        let Some(path) = self.doc.path.clone() else { return };
        let pos = phai_editor::view::caret_lsp(h);
        if let Some(l) = self.lsp_for(&path) {
            let id = l.signature_help(&path, pos.0, pos.1);
            self.asked.insert(id, (h, Ask::Signature));
        }
    }

    /// Alt+F12 in an editor: where the thing at the caret is defined, shown in a window under it.
    pub fn lsp_peek(&mut self, h: HWND) {
        let Some(path) = self.doc.path.clone() else { return };
        let pos = phai_editor::view::caret_lsp(h);
        match self.lsp_for(&path) {
            Some(l) => {
                let id = l.definition(&path, pos.0, pos.1);
                self.asked.insert(id, (h, Ask::Peek));
            }
            None => controls::status_text(self.status, 0, t("rust-analyzer is not installed: rustup component add rust-analyzer")),
        }
    }

    /// Shift+F12 in an editor: every place the thing at the caret is used, in the References list.
    pub fn lsp_references(&mut self, h: HWND) {
        let Some(path) = self.doc.path.clone() else { return };
        let pos = phai_editor::view::caret_lsp(h);
        if let Some(l) = self.lsp_for(&path) {
            let id = l.references(&path, pos.0, pos.1);
            self.asked.insert(id, (h, Ask::References));
        }
    }

    /// Fills the References list - file, line, the line's code - and shows it.
    fn show_references(&mut self, mut found: Vec<phai_editor::lsp::Location>) {
        use windows::Win32::UI::Controls as cc;
        found.sort_by(|a, b| a.path.cmp(&b.path).then(a.start.cmp(&b.start)));
        let Some(list) = self.bottom.as_ref().map(|b| b.refs) else { return };
        unsafe {
            wm::SendMessageW(list, cc::LVM_DELETEALLITEMS, None, None);
        }
        let mut texts: std::collections::HashMap<PathBuf, Vec<String>> = std::collections::HashMap::new();
        for r in &found {
            let lines = texts.entry(r.path.clone()).or_insert_with(|| {
                let text = self.editor_of(&r.path).map(phai_editor::view::text).or_else(|| std::fs::read_to_string(&r.path).ok()).unwrap_or_default();
                text.lines().map(str::to_string).collect()
            });
            let code = lines.get(r.start.0 as usize).map(|l| l.trim().to_string()).unwrap_or_default();
            let file = r.path.file_name().unwrap_or_default().to_string_lossy().to_string();
            win::list_add(list, &[&file, &(r.start.0 + 1).to_string(), &code]);
        }
        let files = texts.len();
        let said = tf(self.found_words, &[&found.len(), &files]);
        self.found_words = "{0} references in {1} files";
        if LSP_TEST.load(Ordering::Relaxed) > 0 && !found.is_empty() {
            println!("{said}");
        }
        controls::status_text(self.status, 0, &said);
        self.references = found;
        self.show_pane(Pane::Output, true);
        if let Some(b) = self.bottom.as_ref() {
            b.show_tab(2);
        }
    }

    /// Go to All (Ctrl+T): files and symbols of the solution by name; the one chosen opened there.
    fn go_to_all(&mut self, query: &str) {
        let root = self.doc.path.as_deref().and_then(cargo::workspace_of).or_else(|| std::env::current_dir().ok());
        let Some(root) = root else { return };
        let root = std::path::absolute(if root.as_os_str().is_empty() { Path::new(".") } else { &root }).unwrap_or(root);
        let mut files = Vec::new();
        let mut stack = vec![root.clone()];
        while let Some(d) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&d) else { continue };
            for p in rd.flatten().map(|e| e.path()) {
                let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                if p.is_dir() {
                    if !name.starts_with('.') && name != "target" && name != "node_modules" {
                        stack.push(p);
                    }
                } else if p.extension().is_some_and(|e| ["rs", "phaiform", "toml", "css", "md"].contains(&e.to_string_lossy().as_ref())) {
                    files.push(p);
                }
            }
        }
        let Some(it) = gotoall::run(self.hwnd, files, root, query) else { return };
        self.remember_place();
        if it.path.extension().is_some_and(|e| e == "phaiform") {
            self.open_path(&it.path);
            return;
        }
        self.open_code(&it.path);
        if let (Some(h), Some(at)) = (self.doc.code, it.at) {
            phai_editor::view::select_lsp(h, at, at);
        }
    }

    /// Go to All typed: the language server asked for the symbols that match.
    pub fn lsp_symbols(&mut self, query: &str) {
        let Some(path) = self.doc.path.clone().or_else(|| self.tabs.iter().find_map(|t| t.doc.path.clone())) else { return };
        if let Some(l) = self.lsp_for(&path) {
            let id = l.workspace_symbols(query);
            let h = self.doc.code.unwrap_or(self.hwnd);
            self.asked.insert(id, (h, Ask::Symbols));
        }
    }

    /// Find in Files (Ctrl+Shift+F): every place some words are, in the solution's files - the
    /// References list shows them, a double-click opens each. `query` given: asked no question.
    fn find_in_files(&mut self, query: Option<String>) {
        let root = self.doc.path.as_deref().and_then(cargo::workspace_of).or_else(|| std::env::current_dir().ok());
        let Some(root) = root else { return };
        // A path given relative may leave the folder empty: the one we are in.
        let root = std::path::absolute(if root.as_os_str().is_empty() { Path::new(".") } else { &root }).unwrap_or(root);
        let (what, case) = match query {
            Some(q) => (q, false),
            None => {
                let start = self.doc.code.map(phai_editor::view::selected_text).filter(|t| !t.is_empty() && !t.contains('\n')).unwrap_or_default();
                let fields = [phai_ui::dialogs::Field::Text(t("Find what:"), &start), phai_ui::dialogs::Field::Check(t("Match case"), false)];
                let Some(a) = phai_ui::dialogs::ask_fields(Some(self.hwnd), &tf("Find in Files - {0}", &[&root.file_name().unwrap_or_default().to_string_lossy()]), &fields) else { return };
                (a[0].clone(), a[1] == "1")
            }
        };
        if what.is_empty() {
            return;
        }
        let found = find_in_tree(&root, &what, case, 5000);
        self.found_words = "{0} matches in {1} files";
        self.show_references(found);
    }

    /// The start window: open a recent form or file, open another, make a new project, or go on.
    fn start_window(&mut self) {
        let again = setting("start_window").is_none_or(|v| v != "0");
        let (choice, again) = startwin::run(self.hwnd, recent_files(), again);
        remember_setting("start_window", if again { "1" } else { "0" });
        match choice {
            startwin::Choice::Recent(p) => {
                if p.extension().is_some_and(|e| e == "phaiform") {
                    self.open_path(&p);
                } else {
                    self.open_code(&p);
                }
            }
            startwin::Choice::OpenFile => self.command(ID_OPEN),
            startwin::Choice::NewProject => self.new_project(),
            startwin::Choice::Continue => {}
        }
    }

    /// Peek Definition: the definition's file in a small editor under the caret, at its line -
    /// to read and to change. A file open in a tab changes there as it is typed; one that is not
    /// is saved when the peek closes (Esc, or a click elsewhere).
    fn open_peek(&mut self, from: HWND, path: &Path, line: usize) {
        self.close_peek();
        let text = match self.editor_of(path) {
            Some(e) => phai_editor::view::text(e),
            None => phai_editor::buffer::normalize(&std::fs::read_to_string(path).unwrap_or_default()).0,
        };
        let class = win::register("PhaiPeek", Some(peek_proc), None);
        let (x, y) = phai_editor::view::caret_screen(from);
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetWindowRect(from, &mut r);
        }
        let (w, hgt) = ((r.right - r.left - win::px(self.hwnd, 60)).max(win::px(self.hwnd, 400)), win::px(self.hwnd, 300));
        let title = win::wide(&format!("{}:{line}  -  Esc", path.file_name().unwrap_or_default().to_string_lossy()));
        unsafe {
            let frame = wm::CreateWindowExW(wm::WS_EX_TOOLWINDOW, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_THICKFRAME | wm::WS_CLIPCHILDREN, r.left + win::px(self.hwnd, 40), y.min(r.bottom - hgt), w, hgt, Some(self.hwnd), None, Some(win::instance()), None).unwrap_or_default();
            let _ = x;
            let editor = phai_editor::view::create(frame, 904, &text, &path.to_string_lossy());
            let mut cr = RECT::default();
            let _ = wm::GetClientRect(frame, &mut cr);
            win::move_to(editor, 0, 0, cr.right, cr.bottom);
            phai_editor::view::go_to(editor, line, 1);
            win::theme_frame(frame);
            win::show_top(frame);
            self.peek = Some((frame, editor, path.to_path_buf(), false));
        }
    }

    /// The peek's text changed: into the file's tab when it has one, else kept to save.
    pub fn peek_changed(&mut self) {
        let Some((_, editor, path, changed)) = self.peek.as_mut() else { return };
        *changed = true;
        let (editor, path) = (*editor, path.clone());
        if let Some(tab) = self.editor_of(&path) {
            phai_editor::view::replace_whole(tab, &phai_editor::view::text(editor));
        }
    }

    /// The peek closed: a changed file with no tab saved, as it was written (its line ends kept).
    pub fn close_peek(&mut self) {
        let Some((frame, editor, path, changed)) = self.peek.take() else { return };
        if changed && self.editor_of(&path).is_none() {
            let raw = std::fs::read_to_string(&path).unwrap_or_default();
            let (_, crlf) = phai_editor::buffer::normalize(&raw);
            let text = phai_editor::view::text(editor);
            let text = if crlf { text.replace('\n', "\r\n") } else { text };
            match std::fs::write(&path, text) {
                Ok(()) => controls::status_text(self.status, 0, &tf("Saved {0}", &[&path.file_name().unwrap_or_default().to_string_lossy()])),
                Err(e) => {
                    win::message(Some(self.hwnd), &e.to_string(), "Phai Studio", wm::MB_ICONWARNING);
                }
            };
        }
        unsafe {
            let _ = wm::DestroyWindow(frame);
        }
    }

    /// Project > Properties: the project's name, version, edition, description and authors - what
    /// Cargo.toml says of it - changed in a form and written back, the rest of the file kept.
    fn project_properties(&mut self) {
        let Some(dir) = self.doc.path.as_deref().and_then(cargo::project_of) else {
            win::message(Some(self.hwnd), t("Save the form in a project first - or make one with New project."), "Phai Studio", wm::MB_ICONINFORMATION);
            return;
        };
        let get = |k: &str| cargo::package_field(&dir, k).unwrap_or_default();
        let version = cargo::package_version(&dir).unwrap_or_default();
        let now = [get("name"), version, get("edition"), get("description"), get("authors")];
        let answers = match std::env::var("PHAI_PROPS_TEST") {
            // `PHAI_PROPS_TEST=description`: that description set, the rest as it is.
            Ok(d) => vec![now[0].clone(), now[1].clone(), now[2].clone(), d, now[4].clone()],
            Err(_) => {
                let fields = [
                    phai_ui::dialogs::Field::Text(t("Package name (the program's file name):"), &now[0]),
                    phai_ui::dialogs::Field::Text(t("Version:"), &now[1]),
                    phai_ui::dialogs::Field::Text(t("Rust edition (2021 or 2024):"), &now[2]),
                    phai_ui::dialogs::Field::Text(t("Description:"), &now[3]),
                    phai_ui::dialogs::Field::Text(t("Authors (comma between):"), &now[4]),
                ];
                let Some(a) = phai_ui::dialogs::ask_fields(Some(self.hwnd), &tf("{0} Properties", &[&now[0]]), &fields) else { return };
                a
            }
        };
        let name = answers[0].trim();
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            win::message(Some(self.hwnd), t("A package name is letters, digits, - and _ only."), "Phai Studio", wm::MB_ICONWARNING);
            return;
        }
        let Ok(mut text) = std::fs::read_to_string(dir.join("Cargo.toml")) else { return };
        let keys = [("name", false), ("version", false), ("edition", false), ("description", false), ("authors", true)];
        for (k, (key, list)) in keys.iter().enumerate() {
            if answers[k].trim() != now[k] {
                text = cargo::with_package_field(&text, key, answers[k].trim(), *list);
            }
        }
        match std::fs::write(dir.join("Cargo.toml"), &text) {
            Ok(()) => {
                controls::status_text(self.status, 0, &tf("Saved {0}", &[&"Cargo.toml"]));
                self.rebuild_menu();
                if let Some(e) = self.explorer.as_mut() {
                    e.show_project_of(self.doc.path.as_deref());
                }
            }
            Err(e) => {
                win::message(Some(self.hwnd), &e.to_string(), "Phai Studio", wm::MB_ICONWARNING);
            }
        }
        if std::env::var_os("PHAI_PROPS_TEST").is_some() {
            println!("description now: {}", cargo::package_field(&dir, "description").unwrap_or_default());
        }
    }

    /// Diagnostic Tools start watching the program with process id `pid`.
    fn watch_process(&mut self, pid: u32) {
        self.sampler = diag::Sampler::new(pid);
        unsafe {
            wm::SetTimer(Some(self.hwnd), TIMER_DIAG, 500, None);
        }
    }

    /// Twice a second while a program runs: a sample, the graphs drawn again; stopped when it ends.
    fn diag_tick(&mut self) {
        if self.sampler.is_none() {
            if let Some(pid) = self.debug.as_ref().map(|d| d.pid()).filter(|p| *p != 0) {
                self.sampler = diag::Sampler::new(pid);
            }
        }
        let alive = self.sampler.as_mut().is_some_and(|s| s.sample());
        if let Some(b) = self.bottom.as_ref() {
            unsafe {
                let _ = Gdi::InvalidateRect(Some(b.diagnostics), None, false);
            }
        }
        if std::env::var_os("PHAI_DIAG_TEST").is_some() {
            if let Some(s) = self.sampler.as_ref() {
                if s.mem.len() >= 4 {
                    println!("diag: {} samples, memory {:.1} MB, cpu {:.0}%", s.mem.len(), *s.mem.back().unwrap_or(&0) as f64 / 1_048_576.0, s.cpu.back().unwrap_or(&0.0));
                    if std::env::var_os("PHAI_SHOT").is_none() {
                        std::process::exit(0);
                    }
                }
            }
        }
        if !alive && self.debug.is_none() && self.sampler.as_ref().is_none_or(|s| s.ended) {
            unsafe {
                let _ = wm::KillTimer(Some(self.hwnd), TIMER_DIAG);
            }
        }
    }

    /// The Immediate window: an expression asked of the debugger while the program is paused -
    /// a name, a path (`order.total`), a comparison - its answer written under it.
    pub fn immediate(&mut self, expr: &str) {
        if expr.is_empty() {
            return;
        }
        let Some(b) = self.bottom.as_ref() else { return };
        b.immediate_line(&format!("> {expr}"));
        match (&self.debug, self.paused) {
            (Some(d), true) => d.send(phai_debug::Command::Evaluate { id: EVAL_IMMEDIATE, exprs: vec![expr.to_string()] }),
            _ => b.immediate_line(t("The program is not paused: start it with F5 and stop at a breakpoint.")),
        }
    }

    /// Batch Build, as Visual Studio's: every project of the solution in Debug and in Release,
    /// ticked to build; Build, Rebuild or Clean them one after another.
    fn batch_build(&mut self) {
        if self.building {
            return;
        }
        let Some(ws) = self.doc.path.as_deref().and_then(cargo::workspace_of) else {
            win::message(Some(self.hwnd), t("Save the form in a project first - or make one with New project."), "Phai Studio", wm::MB_ICONINFORMATION);
            return;
        };
        let ws = std::path::absolute(if ws.as_os_str().is_empty() { Path::new(".") } else { &ws }).unwrap_or(ws);
        let members = cargo::workspace_members(&ws);
        let mine = self.doc.path.as_deref().and_then(cargo::project_of).and_then(|p| std::path::absolute(p).ok());
        let mut rows: Vec<(PathBuf, bool, String)> = Vec::new();
        for m in &members {
            let name = cargo::package_name(m).unwrap_or_else(|| m.file_name().unwrap_or_default().to_string_lossy().to_string());
            for release in [false, true] {
                rows.push((m.clone(), release, format!("{name} - {}", if release { "Release" } else { "Debug" })));
            }
        }
        if rows.is_empty() {
            return;
        }
        // `PHAI_BATCH=job`: every project in Debug, without the questions - for testing.
        let (chosen, job) = if let Ok(j) = std::env::var("PHAI_BATCH") {
            let job = if j == "clean" { cargo::Job::Clean } else { cargo::Job::Build };
            (rows.iter().filter(|r| !r.1).map(|r| (r.0.clone(), r.1)).collect::<Vec<_>>(), job)
        } else {
            let mut fields: Vec<phai_ui::dialogs::Field> = rows.iter().map(|(p, release, label)| phai_ui::dialogs::Field::Check(label, !release && Some(p) == mine.as_ref())).collect();
            let how = t("Build, Rebuild or Clean:");
            let build_word = "Build".to_string();
            fields.push(phai_ui::dialogs::Field::Text(how, &build_word));
            let Some(a) = phai_ui::dialogs::ask_fields(Some(self.hwnd), t("Batch Build"), &fields) else { return };
            let job = match a.last().map(|s| s.trim().to_lowercase()).as_deref() {
                Some("rebuild") => cargo::Job::Rebuild,
                Some("clean") => cargo::Job::Clean,
                _ => cargo::Job::Build,
            };
            (rows.iter().zip(&a).filter(|(_, v)| *v == "1").map(|(r, _)| (r.0.clone(), r.1)).collect(), job)
        };
        if chosen.is_empty() {
            return;
        }
        self.building = true;
        controls::status_text(self.status, 0, &tf("Batch Build: {0} builds", &[&chosen.len()]));
        if let Some(b) = self.bottom.as_ref() {
            b.clear_output();
            b.show_tab(1);
        }
        self.show_errors(true);
        cargo::run_batch(self.hwnd, chosen, job);
    }

    /// Task List: every `// TODO`, `FIXME`, `HACK` and `UNDONE` comment in the solution's code, in
    /// the References list - a double-click opens each.
    fn task_list(&mut self) {
        let root = self.doc.path.as_deref().and_then(cargo::workspace_of).or_else(|| std::env::current_dir().ok());
        let Some(root) = root else { return };
        let root = std::path::absolute(if root.as_os_str().is_empty() { Path::new(".") } else { &root }).unwrap_or(root);
        let mut found: Vec<phai_editor::lsp::Location> = find_in_tree(&root, "//", true, 20000).into_iter().filter(|l| is_task(&l.path, l.start.0)).collect();
        // A line with two `//` is one task.
        found.dedup_by(|a, b| a.path == b.path && a.start.0 == b.start.0);
        self.found_words = "{0} tasks in {1} files";
        self.show_references(found);
    }

    /// Run Tests done: each test, how it went and why it failed, in the Tests list; the count said.
    fn show_tests(&mut self, out: &cargo::Outcome) {
        use windows::Win32::UI::Controls as cc;
        let Some(list) = self.bottom.as_ref().map(|b| b.tests) else { return };
        unsafe {
            wm::SendMessageW(list, cc::LVM_DELETEALLITEMS, None, None);
        }
        // Failed first, as Test Explorer groups them.
        let mut tests = out.tests.clone();
        tests.sort_by_key(|t| match t.outcome.as_str() {
            "FAILED" => 0,
            "ok" => 1,
            _ => 2,
        });
        for r in &tests {
            let said = match r.outcome.as_str() {
                "ok" => t("Passed"),
                "FAILED" => t("Failed"),
                _ => t("Skipped"),
            };
            win::list_add(list, &[&r.name, said, &r.message]);
        }
        let n = |o: &str| tests.iter().filter(|t| t.outcome == o).count();
        let said = if tests.is_empty() && !out.ok { t("The tests could not be built - see Errors").to_string() } else { tf("{0} passed, {1} failed, {2} skipped ({3} s)", &[&n("ok"), &n("FAILED"), &n("ignored"), &format!("{:.1}", out.seconds)]) };
        if let Some(b) = self.bottom.as_ref() {
            b.add_line(&said);
            b.show_tab(if tests.is_empty() && !self.diagnostics.is_empty() { 0 } else { 7 });
        }
        self.show_pane(Pane::Output, true);
        controls::status_text(self.status, 0, &said);
        self.test_results = tests;
    }

    /// Double-click in the Tests list: the test's function, in Phai's editor.
    fn open_test(&mut self, row: usize) {
        let Some(r) = self.test_results.get(row).cloned() else { return };
        let Some(root) = self.doc.path.as_deref().and_then(cargo::workspace_of) else { return };
        let root = std::path::absolute(if root.as_os_str().is_empty() { Path::new(".") } else { &root }).unwrap_or(root);
        let short = r.name.rsplit("::").next().unwrap_or(&r.name).to_string();
        let found = find_in_tree(&root, &format!("fn {short}("), true, 50);
        let Some(place) = found.into_iter().find(|l| l.path.extension().is_some_and(|e| e == "rs")) else { return };
        self.remember_place();
        self.open_code(&place.path);
        if let Some(h) = self.doc.code {
            phai_editor::view::select_lsp(h, place.start, place.end);
        }
    }

    /// Double-click in the References list: that place, in Phai's editor.
    fn open_reference(&mut self, row: usize) {
        let Some(r) = self.references.get(row).cloned() else { return };
        self.remember_place();
        self.open_code(&r.path);
        if let Some(h) = self.doc.code {
            phai_editor::view::select_lsp(h, r.start, r.end);
        }
    }

    /// F2 in an editor: renames the thing at the caret wherever it is used.
    pub fn lsp_rename(&mut self, h: HWND) {
        let Some(path) = self.doc.path.clone() else { return };
        let pos = phai_editor::view::caret_lsp(h);
        if let Some(l) = self.lsp_for(&path) {
            let id = l.prepare_rename(&path, pos.0, pos.1);
            self.asked.insert(id, (h, Ask::PrepareRename(path, pos)));
        }
    }

    /// The pointer rested in an editor: ask what is under it.
    pub fn lsp_hover(&mut self, h: HWND) {
        // Paused in the debugger: the value of what the pointer rests on, as a data tip.
        if let (Some(d), true) = (&self.debug, self.paused) {
            if let Some(path) = phai_editor::view::hover_path(h) {
                static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(EVAL_TIP);
                let id = NEXT.fetch_add(1, Ordering::Relaxed);
                self.tip = Some((id, h, path.clone()));
                d.send(phai_debug::Command::Evaluate { id, exprs: vec![path] });
                return;
            }
        }
        let (Some(path), Some(pos)) = (self.doc.path.clone(), phai_editor::view::hover_position(h)) else { return };
        if let Some(l) = self.lsp_for(&path) {
            let id = l.hover(&path, pos.0, pos.1);
            self.asked.insert(id, (h, Ask::Hover));
        }
    }

    /// Completions were asked for at the caret.
    pub fn lsp_complete(&mut self, h: HWND) {
        let Some(path) = self.doc.path.clone() else { return };
        let pos = phai_editor::view::caret_lsp(h);
        if let Some(l) = self.lsp_for(&path) {
            let id = l.completion(&path, pos.0, pos.1);
            self.asked.insert(id, (h, Ask::Complete));
        }
    }

    /// The editor said its text changed.
    /// Split on or off for the code file in front.
    fn toggle_split(&mut self) {
        if self.split_view.is_some() {
            self.unsplit();
            return;
        }
        let (Some(h), Some(path)) = (self.code_shown(), self.doc.path.clone()) else {
            controls::status_text(self.status, 0, t("Open a code file first"));
            return;
        };
        let Some(parent) = self.canvas.as_ref().map(|c| c.hwnd) else { return };
        let second = phai_editor::view::create(parent, 901, &phai_editor::view::text(h), &path.to_string_lossy());
        self.split_bar = Some(controls::splitter(parent, 902, false));
        phai_editor::view::set_breakpoints(second, &phai_editor::view::breakpoints(h));
        self.split_view = Some(second);
        self.relayout_editors();
        // The second shows where the first is.
        let (l, c) = phai_editor::view::caret_position(h);
        phai_editor::view::go_to(second, l, c);
    }

    fn unsplit(&mut self) {
        if let Some(h) = self.split_view.take() {
            unsafe {
                let _ = wm::DestroyWindow(h);
                if let Some(b) = self.split_bar.take() {
                    let _ = wm::DestroyWindow(b);
                }
            }
            self.relayout_editors();
        }
    }

    fn relayout_editors(&mut self) {
        let Some(c) = self.canvas.as_ref().map(|c| c.hwnd) else { return };
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(c, &mut r);
        }
        self.place_editors(r.right, r.bottom);
    }

    /// The code editor fills the canvas - split, the two share it, one above the other.
    pub fn place_editors(&mut self, w: i32, h: i32) {
        let Some(code) = self.doc.code else { return };
        match self.split_view {
            Some(second) => {
                let gap = win::px(self.hwnd, 6);
                let top = ((h - gap) as i64 * self.split_share as i64 / 1000) as i32;
                win::move_to(code, 0, 0, w, top);
                win::move_to(second, 0, top + gap, w, h - top - gap);
                if let Some(b) = self.split_bar {
                    win::move_to(b, 0, top, w, gap);
                    win::show(b, true);
                }
                win::show(second, true);
                // Above the form designer's surface, which stays behind the code.
                unsafe {
                    let _ = wm::SetWindowPos(second, Some(wm::HWND_TOP), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOACTIVATE);
                    let _ = wm::SetWindowPos(code, Some(wm::HWND_TOP), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOACTIVATE);
                }
            }
            None => win::move_to(code, 0, 0, w, h),
        }
        // Both drawn again where they now are, and only the strip between them on the canvas -
        // the canvas does not clip its children, so its background must not go over them.
        if self.canvas.is_some() {
            unsafe {
                if let Some(b) = self.split_bar {
                    let _ = Gdi::InvalidateRect(Some(b), None, true);
                }
                let _ = Gdi::InvalidateRect(Some(code), None, false);
                if let Some(second) = self.split_view {
                    let _ = Gdi::InvalidateRect(Some(second), None, false);
                }
            }
        }
    }

    /// The split's divider dragged `d` device pixels: the upper pane's share again.
    pub fn split_dragged(&mut self, d: i32) {
        let Some(c) = self.canvas.as_ref().map(|c| c.hwnd) else { return };
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(c, &mut r);
        }
        let room = (r.bottom - win::px(self.hwnd, 6)).max(1);
        let top = (room as i64 * self.split_share as i64 / 1000) as i32 + d;
        self.split_share = ((top.clamp(room / 10, room - room / 10) as i64 * 1000 / room as i64) as i32).clamp(100, 900);
        self.place_editors(r.right, r.bottom);
    }

    /// New Window: the code file in front in a window of its own, to put anywhere - another
    /// screen; what is typed in it is in the file's tab too, and the other way round.
    fn new_window(&mut self) {
        let (Some(h), Some(path)) = (self.code_shown(), self.doc.path.clone()) else {
            controls::status_text(self.status, 0, t("Open a code file first"));
            return;
        };
        let class = win::register("PhaiCodeWindow", Some(code_window_proc), None);
        let title = win::wide(&format!("{} - Phai Studio", path.file_name().unwrap_or_default().to_string_lossy()));
        unsafe {
            let mut r = RECT::default();
            let _ = wm::GetWindowRect(self.hwnd, &mut r);
            let frame = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_OVERLAPPEDWINDOW | wm::WS_CLIPCHILDREN, r.left + 60, r.top + 60, (r.right - r.left) * 2 / 3, (r.bottom - r.top) * 2 / 3, Some(self.hwnd), None, Some(win::instance()), None).unwrap_or_default();
            let editor = phai_editor::view::create(frame, 903, &phai_editor::view::text(h), &path.to_string_lossy());
            let (l, c) = phai_editor::view::caret_position(h);
            phai_editor::view::go_to(editor, l, c);
            win::theme_frame(frame);
            win::show_top(frame);
            let mut cr = RECT::default();
            let _ = wm::GetClientRect(frame, &mut cr);
            win::move_to(editor, 0, 0, cr.right, cr.bottom);
            self.windows.push((frame, editor));
        }
    }

    fn close_windows(&mut self) {
        for (frame, _) in std::mem::take(&mut self.windows) {
            unsafe {
                let _ = wm::DestroyWindow(frame);
            }
        }
    }

    /// An editor's text changed: the others showing the file - a split's other half, its own
    /// windows - take it, and the first goes on as any change does.
    pub fn code_changed_in(&mut self, from: HWND) {
        let Some(code) = self.doc.code else { return };
        let others: Vec<HWND> = self.split_view.into_iter().chain(self.windows.iter().map(|(_, e)| *e)).collect();
        if others.contains(&from) {
            phai_editor::view::replace_whole(code, &phai_editor::view::text(from));
            return;
        }
        self.code_changed();
        let text = phai_editor::view::text(code);
        for h in others {
            phai_editor::view::replace_whole(h, &text);
        }
    }

    pub fn code_changed(&mut self) {
        if let (Some(h), Some(path)) = (self.doc.code, self.doc.path.clone()) {
            if path.extension().is_some_and(|e| e == "rs") {
                let text = phai_editor::view::text(h);
                let version = phai_editor::view::version(h);
                if let Some(l) = self.lsp_for(&path) {
                    l.did_change(&path, version, &text);
                }
            }
        }
        if let Some(h) = self.doc.code {
            let dirty = phai_editor::view::is_dirty(h);
            if dirty != self.doc.dirty {
                self.doc.dirty = dirty;
                self.say_title();
                self.refresh_tabs();
            }
            let (l, c) = phai_editor::view::caret_position(h);
            controls::status_text(self.status, 1, &tf("Line {0}, column {1}", &[&l, &c]));
        }
    }

    /// Everything that shows the active form, again.
    fn after_switch(&mut self) {
        self.load_theme();
        if self.doc.code.is_none() {
            self.rebuild();
        } else {
            self.refresh_outline();
            self.refresh_lenses();
            self.mark_breakpoints();
            self.refresh_blame();
            self.mark_coverage();
        }
        self.refresh_props();
        self.say_title();
        self.say_selection();
        self.refresh_tabs();
        let path = self.doc.path.clone();
        if let Some(e) = self.explorer.as_mut() {
            e.show_project_of(path.as_deref());
        }
        // Last, so nothing shown above leaves the form's surface over the editor.
        self.show_code();
    }

    fn refresh_tabs(&mut self) {
        let mut labels: Vec<String> = Vec::new();
        for (i, tab) in self.tabs.iter().enumerate() {
            labels.push(if i == self.active { Tab { doc: Doc { dirty: self.doc.dirty, ..Doc::new(FormFile::new("X"), self.doc.path.clone()) }, undo: vec![], redo: vec![], selected: None }.label() } else { tab.label() });
        }
        let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
        controls::tabs_relabel(self.tabbar, &refs);
        controls::tabs_select(self.tabbar, self.active);
    }

    /// A form in a tab of its own - or the tab it is already in. An untouched empty first tab is
    /// used rather than left behind.
    fn open_in_tab(&mut self, form: FormFile, path: Option<PathBuf>) {
        if let Some(p) = &path {
            let same = |q: &Option<PathBuf>| q.as_ref().is_some_and(|q| q == p);
            if same(&self.doc.path) {
                return self.after_switch();
            }
            if let Some(i) = self.tabs.iter().enumerate().position(|(i, t)| i != self.active && same(&t.doc.path)) {
                return self.activate(i);
            }
        }
        let blank = self.doc.path.is_none() && !self.doc.dirty && self.doc.form.control.is_empty();
        if !blank {
            self.stash();
            self.tabs.push(Tab::empty());
            self.active = self.tabs.len() - 1;
        }
        self.doc = Doc::new(form, path);
        self.undo.clear();
        self.redo.clear();
        self.selected = None;
        self.after_switch();
    }

    /// Closes the active tab, asking to save; the last one closed leaves an empty form.
    fn close_tab(&mut self) -> bool {
        if !self.may_leave() {
            return false;
        }
        if let Some(h) = self.doc.code.take() {
            if let (Some(l), Some(p)) = (self.lsp.as_ref(), self.doc.path.as_ref()) {
                l.did_close(p);
            }
            unsafe {
                let _ = wm::DestroyWindow(h);
            }
        }
        if self.tabs.len() == 1 {
            self.doc = Doc::new(FormFile::new("MainWindow"), None);
            self.undo.clear();
            self.redo.clear();
            self.selected = None;
            self.after_switch();
            return true;
        }
        self.tabs.remove(self.active);
        let next = self.active.min(self.tabs.len() - 1);
        self.active = next;
        let tab = std::mem::replace(&mut self.tabs[next], Tab::empty());
        self.doc = tab.doc;
        self.undo = tab.undo;
        self.redo = tab.redo;
        self.selected = tab.selected;
        self.after_switch();
        true
    }

    /// Asks about every unsaved tab before the window closes; false when one said Cancel.
    fn may_quit(&mut self) -> bool {
        for i in 0..self.tabs.len() {
            let dirty = if i == self.active { self.doc.dirty } else { self.tabs[i].doc.dirty };
            if dirty {
                self.activate(i);
                if !self.may_leave() {
                    return false;
                }
            }
        }
        true
    }

    // --- saying things --------------------------------------------------------------------------

    fn say_title(&self) {
        let name = self.doc.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| t("Untitled").to_string());
        win::set_text(self.hwnd, &format!("{}{name} - Phai Studio", if self.doc.dirty { "• " } else { "" }));
    }

    pub fn say_selection(&self) {
        self.say_bars();
        // Chosen on the form: chosen in the outline, without that choosing again.
        if let Some(o) = self.outline.as_ref().filter(|o| !o.quiet.get() && explorer::outline_shown(self.side)) {
            o.select(self.selected);
        }
        if let Some(h) = self.doc.code {
            let (l, c) = phai_editor::view::caret_position(h);
            controls::status_text(self.status, 1, &tf("Line {0}, column {1}", &[&l, &c]));
            if self.blame_on {
                self.show_lenses();
            }
            return;
        }
        let f = &self.doc.form;
        let text = match self.selected.and_then(|i| f.control.get(i)) {
            Some(c) => {
                let [x, y, w, h] = c.rect();
                match c.dock.as_deref() {
                    _ if c.kind.is_component() => format!("{} · {}", c.id, kind_label(c.kind)),
                    Some(d) => format!("{} · {d}", c.id),
                    None => format!("{} · {x}, {y} · {w} × {h}", c.id),
                }
            }
            None => format!("{} · {} × {}", f.form.name, f.form.size[0], f.form.size[1]),
        };
        controls::status_text(self.status, 1, &text);
    }

    // --- files --------------------------------------------------------------------------------

    /// Asks to save changes; false when the person cancelled what they were doing.
    fn may_leave(&mut self) -> bool {
        if !self.doc.dirty {
            return true;
        }
        let name = self.doc.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| t("Untitled").to_string());
        match win::message(Some(self.hwnd), &tf("Save changes to {0}?", &[&name]), "Phai Studio", wm::MB_YESNOCANCEL | wm::MB_ICONQUESTION) {
            wm::IDYES => self.save(false),
            wm::IDNO => true,
            _ => false,
        }
    }

    fn set_doc(&mut self, form: FormFile, path: Option<PathBuf>) {
        self.open_in_tab(form, path);
    }

    pub fn open_path(&mut self, path: &Path) {
        let name = path.display().to_string();
        match std::fs::read_to_string(path).map_err(|e| e.to_string()).and_then(|src| FormFile::parse(&src).map_err(|e| e.to_string())) {
            Ok(f) => {
                self.set_doc(f, Some(path.to_path_buf()));
                remember_recent(path);
                self.rebuild_menu();
            }
            Err(e) => {
                win::message(Some(self.hwnd), &format!("{}\n\n{e}", tf("{0} cannot be opened", &[&name])), "Phai Studio", wm::MB_ICONWARNING);
            }
        }
    }

    /// Writes the form, its generated code and - the first time - the programmer's file.
    fn save(&mut self, choose: bool) -> bool {
        // A code file: its text, with the line ends it came with.
        if let (Some(h), Some(path)) = (self.doc.code, self.doc.path.clone()) {
            let text = phai_editor::view::text(h);
            let text = if self.doc.crlf { text.replace('\n', "\r\n") } else { text };
            return match std::fs::write(&path, text) {
                Ok(()) => {
                    if let Some(l) = self.lsp.as_ref() {
                        l.did_save(&path);
                    }
                    phai_editor::view::set_clean(h);
                    self.doc.dirty = false;
                    self.say_title();
                    self.refresh_tabs();
                    controls::status_text(self.status, 0, &tf("Saved {0}", &[&path.file_name().unwrap_or_default().to_string_lossy()]));
                    true
                }
                Err(e) => {
                    win::message(Some(self.hwnd), &format!("{}\n\n{e}", tf("{0} cannot be saved", &[&path.display()])), "Phai Studio", wm::MB_ICONWARNING);
                    false
                }
            };
        }
        if let Some((message, _)) = self.doc.form.check().into_iter().next() {
            win::message(Some(self.hwnd), &message, t("The form has a mistake"), wm::MB_ICONWARNING);
            return false;
        }
        let path = match (&self.doc.path, choose) {
            (Some(p), false) => p.clone(),
            _ => match file_dialog(self.hwnd, true, "Phai form (*.phaiform)", "*.phaiform", &format!("{}.phaiform", snake(&self.doc.form.form.name))) {
                Some(p) => p,
                None => return false,
            },
        };
        let f = &self.doc.form;
        let ui_path = path.with_file_name(format!("{}_ui.rs", snake(&f.form.name)));
        let user_path = path.with_file_name(format!("{}.rs", snake(&f.form.name)));
        let opts = theme_options(&path, f.form.theme.as_deref());
        let wrote = std::fs::write(&path, f.to_toml()).and_then(|_| std::fs::write(&ui_path, codegen::generate(f, &opts))).and_then(|_| if user_path.exists() { Ok(()) } else { std::fs::write(&user_path, codegen::user_stub(f)) });
        match wrote {
            Ok(()) => {
                self.doc.path = Some(path.clone());
                self.doc.dirty = false;
                self.say_title();
                self.refresh_tabs();
                if let Some(e) = self.explorer.as_mut() {
                    e.show_project_of(Some(&path));
                }
                let file = |p: &Path| p.file_name().unwrap_or_default().to_string_lossy().to_string();
                controls::status_text(self.status, 0, &tf("Saved {0} and wrote {1}", &[&file(&path), &file(&ui_path)]));
                true
            }
            Err(e) => {
                win::message(Some(self.hwnd), &format!("{}\n\n{e}", tf("{0} cannot be saved", &[&path.display()])), "Phai Studio", wm::MB_ICONWARNING);
                false
            }
        }
    }

    // --- themes -------------------------------------------------------------------------------

    /// The form's theme, read beside the form and watched; the built-in one when it has none.
    fn load_theme(&mut self) {
        let file = match (&self.doc.path, &self.doc.form.form.theme) {
            (Some(p), Some(th)) => Some(p.with_file_name("").join(th)),
            _ => None,
        };
        match file {
            Some(f) if f.exists() => {
                win::watch_theme(self.hwnd, &f);
                controls::status_text(self.status, 0, &tf("Theme: {0}", &[&self.doc.form.form.theme.clone().unwrap_or_default()]));
            }
            _ => {
                win::unwatch_theme(self.hwnd);
                let dark = theme::current().is_dark();
                theme::set(if dark { Theme::dark() } else { Theme::light() });
                self.after_theme();
            }
        }
    }

    fn after_theme(&mut self) {
        unsafe {
            let old = wm::GetMenu(self.hwnd);
            let _ = wm::SetMenu(self.hwnd, Some(build_menu()));
            let _ = wm::DestroyMenu(old);
        }
        win::retheme(self.hwnd);
        win::rebar_colours(self.rebar);
        self.rebuild();
    }

    /// Project > Add Form, as Visual Studio's "Add Form (Windows Forms)": a new window in the
    /// project's `src` - its form, its generated code and the programmer's file - declared in
    /// main.rs, and opened to design. Another window shows it with `<Name>Ui::show(ui.hwnd, ...)`.
    fn add_form(&mut self) {
        let Some(dir) = self.doc.path.as_deref().and_then(cargo::project_of) else {
            win::message(Some(self.hwnd), t("Open a form of the project first, so the new form can join it."), t("Add Form"), wm::MB_ICONINFORMATION);
            return;
        };
        let src = dir.join("src");
        let mut n = 2;
        while src.join(format!("form{n}.phaiform")).exists() {
            n += 1;
        }
        let default = format!("Form{n}");
        let Some(answer) = phai_ui::dialogs::ask_fields(Some(self.hwnd), t("Add Form"), &[phai_ui::dialogs::Field::Text(t("Name"), &default)]) else { return };
        let name = form_type_name(&answer[0]);
        if name.is_empty() {
            return;
        }
        let file = snake(&name);
        let path = src.join(format!("{file}.phaiform"));
        if path.exists() || src.join(format!("{file}.rs")).exists() || src.join(format!("{file}_ui.rs")).exists() {
            win::message(Some(self.hwnd), &tf("The project already has {0}", &[&file]), t("Add Form"), wm::MB_ICONINFORMATION);
            return;
        }
        let (path, declared) = match write_new_form(&dir, &name) {
            Ok(r) => r,
            Err(e) => {
                win::message(Some(self.hwnd), &format!("{}

{e}", tf("{0} cannot be saved", &[&path.display()])), t("Add Form"), wm::MB_ICONWARNING);
                return;
            }
        };
        self.open_path(&path);
        if let Some(e) = self.explorer.as_mut() {
            e.show_project_of(Some(&path));
        }
        let how = format!("{name}Ui::show(ui.hwnd, {file}::{name}::default())");
        if declared {
            self.say_status(&tf("Added {0} - show it from another window with {1}", &[&name, &how]));
        } else {
            self.say_status(&tf("Added {0} - add `mod {1}; mod {1}_ui;` to main.rs", &[&name, &file]));
        }
    }

    /// Server Explorer's "Add to Form", as dragging a table from Data Sources in Visual Studio: a
    /// grid bound to the table under what the form has, with Reload and Save buttons below it;
    /// the project told where the data is (`src/data_sources.rs`, phai-data in Cargo.toml).
    pub fn bind_table(&mut self, conn: &phai_data::Connection, database: &str, table: &str, cols: &[phai_data::Column]) {
        let Some(dir) = self.doc.path.as_deref().and_then(cargo::project_of) else {
            win::message(Some(self.hwnd), t("Open a form of the project first, so the table can go on it."), t("Add to Form"), wm::MB_ICONINFORMATION);
            return;
        };
        if cols.is_empty() {
            return;
        }
        let (controls, need) = bound_controls(&self.doc.form, database, table, cols);
        self.edit(|f| {
            f.control.extend(controls);
            if f.form.size[1] < need {
                f.form.size[1] = need;
            }
        });
        if let Err(e) = bind_project_files(&dir, conn) {
            win::message(Some(self.hwnd), &e.to_string(), t("Add to Form"), wm::MB_ICONWARNING);
        }
        let keyless = cols.iter().all(|c| c.key != "PRI");
        self.say_status(&if keyless { tf("{0} is on the form - it has no primary key, so rows can only be added", &[&table]) } else { tf("{0} is on the form: its rows load when the window opens, Save writes changes back", &[&table]) });
    }

    /// The project's style sheets (`.css` in the project folder and under `src`), each as the form
    /// names its theme: relative to the form's folder, `/` between parts (`../theme.css`).
    pub fn theme_files(&self) -> Vec<String> {
        let Some(form_dir) = self.doc.path.as_deref().and_then(Path::parent).map(Path::to_path_buf) else { return vec![] };
        let Some(root) = self.doc.path.as_deref().and_then(cargo::project_of) else { return vec![] };
        let mut out = Vec::new();
        for dir in [root.clone(), root.join("src")] {
            let Ok(rd) = std::fs::read_dir(&dir) else { continue };
            let mut names: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("css"))).collect();
            names.sort();
            for p in names {
                if let Some(rel) = relative_to(&form_dir, &p) {
                    if !out.contains(&rel) {
                        out.push(rel);
                    }
                }
            }
        }
        out
    }

    fn theme_from_standard(&mut self) {
        let Some(path) = self.doc.path.clone() else {
            win::message(Some(self.hwnd), t("Save the form first, so its theme can sit beside it."), "Phai Studio", wm::MB_ICONINFORMATION);
            return;
        };
        let css = path.with_file_name("theme.css");
        if !css.exists() && std::fs::write(&css, theme::STANDARD_CSS).is_err() {
            return;
        }
        self.edit(|f| f.form.theme = Some("theme.css".into()));
        self.load_theme();
        self.refresh_props();
        self.rebuild_menu();
    }

    fn theme_import(&mut self) {
        let Some(path) = self.doc.path.clone() else {
            win::message(Some(self.hwnd), t("Save the form first, so its theme can sit beside it."), "Phai Studio", wm::MB_ICONINFORMATION);
            return;
        };
        let Some(chosen) = file_dialog(self.hwnd, false, "CSS (*.css)", "*.css", "") else { return };
        let dir = path.with_file_name("");
        // A sheet from elsewhere is copied in, so the project carries everything it is built from.
        let name = chosen.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "theme.css".into());
        if chosen.parent() != Some(dir.as_path()) && std::fs::copy(&chosen, dir.join(&name)).is_err() {
            return;
        }
        self.edit(|f| f.form.theme = Some(name));
        self.load_theme();
        self.refresh_props();
        self.rebuild_menu();
    }

    // --- projects, building and running --------------------------------------------------------

    fn new_project(&mut self) {
        let Some(dir) = project::pick_folder(self.hwnd, t("Choose an empty folder for the new project")) else { return };
        match project::create(&dir) {
            Ok(form) => {
                self.open_path(&form);
                controls::status_text(self.status, 0, &tf("New project in {0}", &[&dir.display()]));
            }
            Err(e) => {
                win::message(Some(self.hwnd), &format!("{}\n\n{e}", tf("{0} cannot be made", &[&dir.display()])), "Phai Studio", wm::MB_ICONWARNING);
            }
        }
    }

    /// Saves, then builds the project the form is in; `then_run` starts the program after.
    fn start_build(&mut self, then_run: bool) {
        self.start_job_run(cargo::Job::Build, false, then_run);
    }

    pub fn say_status(&self, text: &str) {
        controls::status_text(self.status, 0, text);
    }

    pub fn rebuild_menu_pub(&self) {
        self.rebuild_menu();
    }

    /// New Query: a `.sql` file of its own, tied to a connection and database, opened in the
    /// editor; Ctrl+Shift+E runs it.
    pub fn new_query(&mut self, conn: usize, db: &str, text: &str) {
        let Some(dir) = settings_dir().map(|d| d.join("Queries")) else { return };
        let _ = std::fs::create_dir_all(&dir);
        let mut n = 1;
        let path = loop {
            let p = dir.join(format!("Query{n}.sql"));
            if !p.exists() && !self.queries.contains_key(&p) {
                break p;
            }
            n += 1;
        };
        let label = self.servers.as_ref().and_then(|s| s.conns.get(conn)).map(|c| c.label()).unwrap_or_default();
        let head = format!("-- {label}{}\n-- {}\n", if db.is_empty() { String::new() } else { format!(" · {db}") }, t("Ctrl+Shift+E runs the query; its rows show under Results."));
        if std::fs::write(&path, format!("{head}{text}")).is_err() {
            return;
        }
        self.queries.insert(path.clone(), (conn, db.to_string()));
        self.open_code(&path);
    }

    /// Runs the query file in front, waiting with a Cancel; its rows into Results.
    fn execute_query(&mut self) {
        let Some(path) = self.doc.path.clone() else { return };
        let Some((conn, db)) = self.queries.iter().find(|(p, _)| phai_editor::lsp::same_path(p, &path)).map(|(_, v)| v.clone()) else {
            self.say_status(t("Open a query first: Data > New Query, or right-click a table in Server Explorer"));
            return;
        };
        let Some(c) = self.servers.as_ref().and_then(|s| s.conns.get(conn)).cloned() else { return };
        let sql = self.doc.code.map(phai_editor::view::text).unwrap_or_default();
        // The query without the comment lines this file starts with.
        let sql: String = sql.lines().filter(|l| !l.trim_start().starts_with("--")).collect::<Vec<_>>().join("\n");
        if sql.trim().is_empty() {
            return;
        }
        let db2 = db.clone();
        let answer = phai_ui::dialogs::wait_for(Some(self.hwnd), t("Execute Query"), t("Running the query..."), move |_| c.query((!db2.is_empty()).then_some(db2.as_str()), &sql));
        match answer {
            Some(Ok(grid)) => {
                self.table_view = None;
                self.show_grid(&grid);
            }
            Some(Err(e)) => {
                if let Some(b) = self.bottom.as_ref() {
                    b.add_line(&e);
                    b.show_tab(1);
                }
                self.show_pane(Pane::Output, true);
                self.say_status(&e);
            }
            None => {}
        }
    }

    /// Show Table Data: the first rows of a table, into Results.
    pub fn show_table_rows(&mut self, conn: usize, db: &str, table: &str) {
        let Some(c) = self.servers.as_ref().and_then(|s| s.conns.get(conn)).cloned() else { return };
        let (db, table) = (db.to_string(), table.to_string());
        let (d2, t2) = (db.clone(), table.clone());
        match phai_ui::dialogs::wait_for(Some(self.hwnd), t("Show Table Data"), t("Running the query..."), move |_| c.table_rows(&d2, &t2).and_then(|g| Ok((g, c.columns(&d2, &t2)?)))) {
            Some(Ok((grid, columns))) => {
                self.show_grid(&grid);
                self.table_view = Some(TableView { conn, db, table, columns, rows: grid.rows.clone() });
            }
            Some(Err(e)) => {
                win::message(Some(self.hwnd), &e, t("Show Table Data"), wm::MB_ICONWARNING);
            }
            None => {}
        }
    }

    /// The primary key of a row shown: its columns and values; `None` when the table has none.
    fn row_keys(&self, row: usize) -> Option<Vec<(String, String)>> {
        let v = self.table_view.as_ref()?;
        let r = v.rows.get(row)?;
        let keys: Vec<(String, String)> = v.columns.iter().enumerate().filter(|(_, c)| c.key == "PRI").filter_map(|(i, c)| Some((c.name.clone(), r.get(i)?.clone()))).collect();
        (!keys.is_empty()).then_some(keys)
    }

    /// A cell of a table in Results changed, as Visual Studio's Show Table Data lets one be.
    fn edit_cell(&mut self, row: usize, col: usize) {
        let Some(v) = self.table_view.as_ref() else {
            self.say_status(t("A query's result is not a table: open one with Show Table Data to change its rows"));
            return;
        };
        let Some(column) = v.columns.get(col).cloned() else { return };
        let Some(keys) = self.row_keys(row) else {
            win::message(Some(self.hwnd), t("This table has no primary key, so its rows cannot be told apart to change one."), t("Show Table Data"), wm::MB_ICONINFORMATION);
            return;
        };
        let now = v.rows.get(row).and_then(|r| r.get(col)).cloned().unwrap_or_default();
        let Some(new) = phai_ui::dialogs::ask(Some(self.hwnd), &column.name, &tf("{0} ({1}) - NULL for none:", &[&column.name, &column.ty]), &now, false) else { return };
        if new == now {
            return;
        }
        let value = if new == "NULL" && column.nullable { None } else { Some(new.clone()) };
        let (Some(c), db, table) = (self.servers.as_ref().and_then(|s| s.conns.get(v.conn)).cloned(), v.db.clone(), v.table.clone()) else { return };
        let name = column.name.clone();
        let result = phai_ui::dialogs::wait_for(Some(self.hwnd), t("Show Table Data"), t("Saving..."), move |_| c.update_cell(&db, &table, &name, value.as_deref(), &keys));
        match result {
            Some(Ok(n)) => {
                if let Some(v) = self.table_view.as_mut() {
                    if let Some(cell) = v.rows.get_mut(row).and_then(|r| r.get_mut(col)) {
                        *cell = new.clone();
                    }
                }
                if let Some(list) = self.bottom.as_ref().map(|b| b.results) {
                    set_list_cell(list, row, col, &new);
                }
                self.say_status(&tf("{0} rows affected", &[&n]));
            }
            Some(Err(e)) => {
                win::message(Some(self.hwnd), &e, t("Show Table Data"), wm::MB_ICONWARNING);
            }
            None => {}
        }
    }

    /// Results' right-click: change the cell, add a row, delete the row.
    fn results_menu(&mut self, row: Option<usize>, col: usize) {
        if self.table_view.is_none() {
            return;
        }
        const EDIT: u32 = 1;
        const ADD: u32 = 2;
        const DELETE: u32 = 3;
        const REFRESH: u32 = 4;
        let mut items: Vec<(u32, &str)> = Vec::new();
        if row.is_some() {
            items.push((EDIT, t("Edit Cell...")));
        }
        items.push((ADD, t("Add Row...")));
        if row.is_some() {
            items.push((DELETE, t("Delete Row")));
        }
        items.extend([(0, ""), (REFRESH, t("Refresh"))]);
        match win::popup(self.hwnd, &items) {
            EDIT => self.edit_cell(row.unwrap_or(0), col),
            ADD => self.add_row(),
            DELETE => self.delete_row(row.unwrap_or(0)),
            REFRESH => {
                if let Some(v) = self.table_view.as_ref() {
                    let (c, d, t2) = (v.conn, v.db.clone(), v.table.clone());
                    self.show_table_rows(c, &d, &t2);
                }
            }
            _ => {}
        }
    }

    fn delete_row(&mut self, row: usize) {
        let Some(keys) = self.row_keys(row) else {
            win::message(Some(self.hwnd), t("This table has no primary key, so its rows cannot be told apart to change one."), t("Show Table Data"), wm::MB_ICONINFORMATION);
            return;
        };
        let Some(v) = self.table_view.as_ref() else { return };
        let what = keys.iter().map(|(k, val)| format!("{k} = {val}")).collect::<Vec<_>>().join(", ");
        if win::message(Some(self.hwnd), &tf("Delete the row {0} from {1}? This cannot be undone.", &[&what, &v.table]), t("Delete Row"), wm::MB_YESNO | wm::MB_ICONWARNING) != wm::IDYES {
            return;
        }
        let (Some(c), db, table) = (self.servers.as_ref().and_then(|s| s.conns.get(v.conn)).cloned(), v.db.clone(), v.table.clone()) else { return };
        let conn = v.conn;
        match phai_ui::dialogs::wait_for(Some(self.hwnd), t("Delete Row"), t("Saving..."), move |_| c.delete_row(&db, &table, &keys)) {
            Some(Ok(_)) => {
                let (d, t2) = (self.table_view.as_ref().map(|v| v.db.clone()).unwrap_or_default(), self.table_view.as_ref().map(|v| v.table.clone()).unwrap_or_default());
                self.show_table_rows(conn, &d, &t2);
            }
            Some(Err(e)) => {
                win::message(Some(self.hwnd), &e, t("Delete Row"), wm::MB_ICONWARNING);
            }
            None => {}
        }
    }

    /// Add Row: a box for each column (at most twelve); those left empty take their defaults.
    fn add_row(&mut self) {
        let Some(v) = self.table_view.as_ref() else { return };
        let cols: Vec<phai_data::Column> = v.columns.iter().take(12).cloned().collect();
        let labels: Vec<String> = cols.iter().map(|c| format!("{} ({})", c.name, c.ty)).collect();
        let fields: Vec<phai_ui::dialogs::Field> = labels.iter().map(|l| phai_ui::dialogs::Field::Text(l, "")).collect();
        let Some(answers) = phai_ui::dialogs::ask_fields(Some(self.hwnd), &tf("Add Row - {0}", &[&v.table]), &fields) else { return };
        let values: Vec<(String, Option<String>)> = cols.iter().zip(answers).filter(|(_, a)| !a.is_empty()).map(|(c, a)| (c.name.clone(), if a == "NULL" { None } else { Some(a) })).collect();
        if values.is_empty() {
            return;
        }
        let (Some(c), db, table) = (self.servers.as_ref().and_then(|s| s.conns.get(v.conn)).cloned(), v.db.clone(), v.table.clone()) else { return };
        let conn = v.conn;
        let (d2, t2) = (db.clone(), table.clone());
        match phai_ui::dialogs::wait_for(Some(self.hwnd), t("Add Row..."), t("Saving..."), move |_| c.insert_row(&db, &table, &values)) {
            Some(Ok(_)) => self.show_table_rows(conn, &d2, &t2),
            Some(Err(e)) => {
                win::message(Some(self.hwnd), &e, t("Add Row..."), wm::MB_ICONWARNING);
            }
            None => {}
        }
    }

    fn show_grid(&mut self, grid: &phai_data::Grid) {
        let Some(list) = self.bottom.as_ref().map(|b| b.results) else { return };
        if grid.columns.is_empty() {
            win::list_columns(list, &[(t("Result"), 400)]);
            let said = tf("{0} rows affected", &[&grid.affected]);
            win::list_add(list, &[&said]);
            self.say_status(&said);
        } else {
            let widths: Vec<(&str, i32)> = grid.columns.iter().map(|c| (c.as_str(), (c.chars().count() as i32 * 9 + 40).clamp(80, 260))).collect();
            win::list_columns(list, &widths);
            for r in &grid.rows {
                let cells: Vec<&str> = r.iter().map(String::as_str).collect();
                win::list_add(list, &cells);
            }
            let said = if grid.cut { tf("First {0} rows", &[&grid.rows.len()]) } else { tf("{0} rows", &[&grid.rows.len()]) };
            self.say_status(&said);
        }
        self.show_pane(Pane::Output, true);
        if let Some(b) = self.bottom.as_ref() {
            b.show_tab(5);
        }
    }

    /// Watch again: while paused the debugger is asked for every value; else the names alone.
    fn refresh_watch(&mut self) {
        match (&self.debug, self.paused) {
            (Some(d), true) if !self.watches.is_empty() => d.send(phai_debug::Command::Evaluate { id: EVAL_WATCH, exprs: self.watches.clone() }),
            _ => {
                if let Some(b) = self.bottom.as_ref() {
                    use windows::Win32::UI::Controls as cc;
                    unsafe {
                        wm::SendMessageW(b.watch, cc::LVM_DELETEALLITEMS, None, None);
                    }
                    for w in &self.watches {
                        win::list_add(b.watch, &[w, "", ""]);
                    }
                }
            }
        }
    }

    /// A local double-clicked while paused: a new value typed over it, as in Visual Studio.
    fn edit_local(&mut self, row: i32) {
        let (Some(d), true) = (&self.debug, self.paused) else { return };
        let Some(list) = self.bottom.as_ref().map(|b| b.locals) else { return };
        if row < 0 {
            return;
        }
        let cell = |col: i32| win::list_text(list, row as usize, col);
        let (name, value, ty) = (cell(0), cell(1), cell(2));
        let Some(new) = phai_ui::dialogs::ask(Some(self.hwnd), &tf("Change {0}", &[&name]), &tf("A new value for {0} ({1}):", &[&name, &ty]), &value, false) else { return };
        if new.trim() != value {
            d.send(phai_debug::Command::SetValue { expr: name, value: new });
        }
    }

    /// Breakpoint Condition: the caret line's breakpoint (made if there is none) stops only when
    /// what is typed holds; nothing typed takes the condition off.
    fn edit_condition(&mut self) {
        let (Some(h), Some(path)) = (self.doc.code, self.doc.path.clone()) else { return };
        let line = phai_editor::view::caret_line(h);
        if !phai_editor::view::breakpoints(h).contains(&line) {
            phai_editor::view::toggle_breakpoint(h);
        }
        let now = self.conds.iter().find(|(p, l, _)| phai_editor::lsp::same_path(p, &path) && *l == line).map(|(_, _, c)| c.clone()).unwrap_or_default();
        let (expr, hits) = phai_debug::split_condition(&now);
        let hits_text = hits.map(|h| h.text()).unwrap_or_default();
        let fields = [
            phai_ui::dialogs::Field::Text(&tf("Stop at line {0} only when this holds - x > 5, name == \"ไผ่\":", &[&line]), &expr),
            phai_ui::dialogs::Field::Text(t("Hit count - = 3, >= 3 or % 3 (every third time); empty for every time:"), &hits_text),
        ];
        let Some(a) = phai_ui::dialogs::ask_fields(Some(self.hwnd), t("Breakpoint Condition"), &fields) else { return };
        let hits = match phai_debug::Hits::parse(&a[1]) {
            Ok(h) => h,
            Err(e) => {
                win::message(Some(self.hwnd), &e, t("Breakpoint Condition"), wm::MB_ICONWARNING);
                return;
            }
        };
        self.conds.retain(|(p, l, _)| !(phai_editor::lsp::same_path(p, &path) && *l == line));
        if !a[0].trim().is_empty() || hits.is_some() {
            self.conds.push((path.clone(), line, phai_debug::join_condition(&a[0], hits)));
        }
        self.show_conditions();
        self.send_breakpoints();
    }

    /// Each editor's conditional breakpoints drawn as such; the debugger told.
    fn show_conditions(&mut self) {
        let open: Vec<(PathBuf, HWND)> = self.tabs.iter().filter_map(|t| Some((t.doc.path.clone()?, t.doc.code?))).chain(self.doc.path.clone().zip(self.doc.code)).collect();
        for (p, h) in open {
            let lines: Vec<u32> = self.conds.iter().filter(|(q, _, _)| phai_editor::lsp::same_path(q, &p)).map(|(_, l, _)| *l).collect();
            phai_editor::view::set_conditional(h, &lines);
        }
        if let Some(d) = &self.debug {
            d.send(phai_debug::Command::Conditions(self.conds.clone()));
        }
        self.fill_breakpoints();
    }

    /// Publish: asks what the product is called and by whom, builds it as Release, then makes
    /// a setup program and a portable folder of it in the project's `publish` folder.
    fn publish(&mut self) {
        let Some(path) = self.doc.path.clone() else {
            win::message(Some(self.hwnd), t("Save the form in a project first - or make one with New project."), "Phai Studio", wm::MB_ICONINFORMATION);
            return;
        };
        let Some(dir) = cargo::project_of(&path) else { return };
        let package = cargo::package_name(&dir).unwrap_or_else(|| "program".into());
        let version = cargo::package_version(&dir).unwrap_or_else(|| "0.1.0".into());
        // What was said last time, kept beside the project.
        let saved = std::fs::read_to_string(dir.join("phai-publish.txt")).unwrap_or_default();
        let get = |k: &str| saved.lines().find_map(|l| l.strip_prefix(&format!("{k}=")).map(str::to_string));
        let name = get("name").unwrap_or_else(|| package.clone());
        let publisher = get("publisher").unwrap_or_default();
        let desktop = get("desktop").map(|d| d == "1").unwrap_or(true);
        let all_users = get("all_users").is_some_and(|d| d == "1");
        // What else goes with the program: said before, or the pictures its forms show.
        let include = get("include").unwrap_or_else(|| pictures_of(&dir).join(", "));
        let answers = if std::env::var_os("PHAI_PUBLISH_TEST").is_some() {
            vec![name, version, publisher, (desktop as u8).to_string(), include, (all_users as u8).to_string()]
        } else {
            let fields = [
                phai_ui::dialogs::Field::Text(t("Product name"), &name),
                phai_ui::dialogs::Field::Text(t("Version"), &version),
                phai_ui::dialogs::Field::Text(t("Publisher"), &publisher),
                phai_ui::dialogs::Field::Check(t("Offer a desktop shortcut"), desktop),
                phai_ui::dialogs::Field::Text(t("Also include - files or folders of the project, comma between:"), &include),
                phai_ui::dialogs::Field::Check(t("Install for everyone on the computer (asks for an administrator)"), all_users),
            ];
            let Some(a) = phai_ui::dialogs::ask_fields(Some(self.hwnd), &tf("Publish {0}", &[&package]), &fields) else { return };
            a
        };
        let _ = std::fs::write(dir.join("phai-publish.txt"), format!("name={}\npublisher={}\ndesktop={}\ninclude={}\nall_users={}\n", answers[0].trim(), answers[2].trim(), answers[3], answers[4].trim(), answers[5]));
        self.publish_include = answers[4].split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        self.publish_after_build = Some(phai_pack::Manifest { name: answers[0].trim().to_string(), version: answers[1].trim().to_string(), publisher: answers[2].trim().to_string(), desktop: answers[3] == "1", all_users: answers[5] == "1", ..Default::default() });
        let was = self.release;
        self.release = true;
        self.start_job_run(cargo::Job::Build, false, false);
        self.release = was;
        if !self.building {
            self.publish_after_build = None;
        }
    }

    /// The Release build done: the setup program and the portable folder made of it.
    fn finish_publish(&mut self, mut m: phai_pack::Manifest, exe: &Path) {
        let fail = |s: &mut Studio, e: String| {
            if PUBLISH_TEST.load(Ordering::Relaxed) {
                println!("publish failed: {e}");
                std::process::exit(2);
            }
            win::message(Some(s.hwnd), &e, t("Publish"), wm::MB_ICONWARNING);
        };
        let Some(dir) = self.doc.path.as_deref().and_then(cargo::project_of) else { return };
        let stub_path = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("phai-setup.exe")));
        let Some(stub) = stub_path.as_ref().and_then(|p| std::fs::read(p).ok()) else {
            return fail(self, t("phai-setup.exe is not beside Phai Studio, so no setup program can be made.").to_string());
        };
        let Ok(bytes) = std::fs::read(exe) else { return fail(self, format!("{} cannot be read", exe.display())) };
        let exe_name = exe.file_name().unwrap_or_default().to_string_lossy().to_string();
        m.exe = exe_name.clone();
        let out = dir.join("publish");
        let base = format!("{}-{}", m.safe_name().replace(' ', "-"), m.version);
        let setup_path = out.join(format!("{base}-setup.exe"));
        let portable = out.join(format!("{base}-portable"));
        // The program, then what else was asked for - by their paths in the project.
        let mut files = vec![(exe_name.clone(), bytes.clone())];
        for inc in &self.publish_include {
            let from = dir.join(inc);
            let mut stack = vec![from.clone()];
            while let Some(p) = stack.pop() {
                if p.is_dir() {
                    if let Ok(rd) = std::fs::read_dir(&p) {
                        stack.extend(rd.flatten().map(|e| e.path()));
                    }
                } else if let (Ok(data), Ok(rel)) = (std::fs::read(&p), p.strip_prefix(&dir)) {
                    let rel = rel.to_string_lossy().replace('\\', "/");
                    if !files.iter().any(|(n, _)| *n == rel) {
                        files.push((rel, data));
                    }
                }
            }
            if !from.exists() {
                return fail(self, tf("{0} is not in the project", &[inc]));
            }
        }
        let _ = std::fs::remove_dir_all(&portable);
        let mut written = std::fs::create_dir_all(&portable).and_then(|_| std::fs::write(&setup_path, phai_pack::pack(&stub, &m, &files)));
        for (rel, data) in &files {
            if written.is_ok() {
                let at = portable.join(rel);
                written = at.parent().map(std::fs::create_dir_all).unwrap_or(Ok(())).and_then(|_| std::fs::write(&at, data));
            }
        }
        if let Err(e) = written {
            return fail(self, e.to_string());
        }
        let size = std::fs::metadata(&setup_path).map(|m| m.len()).unwrap_or(0);
        let said = tf("Published: {0} ({1} KB) and a portable folder", &[&setup_path.file_name().unwrap_or_default().to_string_lossy(), &(size / 1024)]);
        if let Some(b) = self.bottom.as_ref() {
            b.add_line(&said);
            b.add_line(&setup_path.display().to_string());
        }
        self.say_status(&said);
        if PUBLISH_TEST.load(Ordering::Relaxed) {
            println!("files: {}", files.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(", "));
            println!("published {}", setup_path.display());
            std::process::exit(0);
        }
        open_with_windows(&out.to_string_lossy());
    }

    /// F5: build the project as Debug, then run it under the debugger.
    fn start_debugging(&mut self) {
        let was = self.release;
        self.release = false;
        self.debug_after_build = true;
        self.start_job_run(cargo::Job::Build, false, false);
        self.release = was;
        if !self.building {
            self.debug_after_build = false;
        }
    }

    /// Every breakpoint, of open files and of those set before and closed.
    /// The breakpoints the debugger is given: every one but those turned off.
    fn all_breakpoints(&mut self) -> Vec<(PathBuf, u32)> {
        let all = self.every_breakpoint();
        all.into_iter().filter(|(p, l)| !self.bp_off.iter().any(|(q, m)| m == l && phai_editor::lsp::same_path(p, q))).collect()
    }

    /// Every breakpoint, in every file, as the editors have them now.
    fn every_breakpoint(&mut self) -> Vec<(PathBuf, u32)> {
        let open: Vec<(PathBuf, HWND)> = self.tabs.iter().filter_map(|t| Some((t.doc.path.clone()?, t.doc.code?))).chain(self.doc.path.clone().zip(self.doc.code)).collect();
        for (p, h) in open {
            self.bps.retain(|q, _| !phai_editor::lsp::same_path(q, &p));
            let lines = phai_editor::view::breakpoints(h);
            if !lines.is_empty() {
                self.bps.insert(p, lines);
            }
        }
        let mut all: Vec<(PathBuf, u32)> = self.bps.iter().flat_map(|(p, ls)| ls.iter().map(move |l| (p.clone(), *l))).collect();
        all.sort();
        // Turned off and then taken away: forgotten.
        let keep = all.clone();
        self.bp_off.retain(|(p, l)| keep.iter().any(|(q, m)| m == l && phai_editor::lsp::same_path(p, q)));
        all
    }

    /// The Breakpoints window filled: each breakpoint, its condition and hit count, ticked when on.
    fn fill_breakpoints(&mut self) {
        use windows::Win32::UI::Controls as cc;
        let Some(list) = self.bottom.as_ref().map(|b| b.breakpoints) else { return };
        let rows = self.every_breakpoint();
        self.bp_filling = true;
        unsafe {
            wm::SendMessageW(list, cc::LVM_DELETEALLITEMS, None, None);
        }
        for (i, (p, l)) in rows.iter().enumerate() {
            let cond = self.conds.iter().find(|(q, m, _)| m == l && phai_editor::lsp::same_path(p, q)).map(|c| phai_debug::split_condition(&c.2));
            let (expr, hits) = cond.unwrap_or_default();
            let name = tf("{0}, line {1}", &[&p.file_name().unwrap_or_default().to_string_lossy(), l]);
            let when = if expr.is_empty() { t("(no condition)").to_string() } else { tf("when {0} is true", &[&expr]) };
            let count = hits.map(|h| h.text()).unwrap_or_else(|| t("break always").to_string());
            win::list_add(list, &[&name, &when, &count, &p.to_string_lossy()]);
            let on = !self.bp_off.iter().any(|(q, m)| m == l && phai_editor::lsp::same_path(p, q));
            let item = cc::LVITEMW { stateMask: cc::LIST_VIEW_ITEM_STATE_FLAGS(0xF000), state: cc::LIST_VIEW_ITEM_STATE_FLAGS(if on { 2 } else { 1 } << 12), ..Default::default() };
            unsafe {
                wm::SendMessageW(list, cc::LVM_SETITEMSTATE, Some(WPARAM(i)), Some(LPARAM(&item as *const _ as isize)));
            }
        }
        self.bp_rows = rows;
        self.bp_filling = false;
    }

    /// A tick in the Breakpoints window changed: that breakpoint turned on or off.
    fn breakpoint_ticked(&mut self, row: usize, on: bool) {
        if self.bp_filling {
            return;
        }
        let Some((p, l)) = self.bp_rows.get(row).cloned() else { return };
        self.bp_off.retain(|(q, m)| !(*m == l && phai_editor::lsp::same_path(&p, q)));
        if !on {
            self.bp_off.push((p, l));
        }
        self.mark_breakpoints();
        self.send_breakpoints();
    }

    /// The breakpoint on a row of the Breakpoints window taken away (its Delete key).
    fn delete_breakpoint_row(&mut self, row: usize) {
        let Some((p, l)) = self.bp_rows.get(row).cloned() else { return };
        let open = self.tabs.iter().filter_map(|t| Some((t.doc.path.clone()?, t.doc.code?))).chain(self.doc.path.clone().zip(self.doc.code)).find(|(q, _)| phai_editor::lsp::same_path(q, &p));
        match open {
            Some((_, h)) => {
                let lines: Vec<u32> = phai_editor::view::breakpoints(h).into_iter().filter(|m| *m != l).collect();
                phai_editor::view::set_breakpoints(h, &lines);
            }
            None => {
                for (q, ls) in self.bps.iter_mut() {
                    if phai_editor::lsp::same_path(q, &p) {
                        ls.retain(|m| *m != l);
                    }
                }
            }
        }
        self.conds.retain(|(q, m, _)| !(*m == l && phai_editor::lsp::same_path(q, &p)));
        self.send_breakpoints();
        self.show_conditions();
        self.fill_breakpoints();
    }

    /// Each open editor's breakpoints drawn as they are: with a condition, or turned off.
    fn mark_breakpoints(&mut self) {
        let open: Vec<(PathBuf, HWND)> = self.tabs.iter().filter_map(|t| Some((t.doc.path.clone()?, t.doc.code?))).chain(self.doc.path.clone().zip(self.doc.code)).collect();
        for (p, h) in open {
            let off: Vec<u32> = self.bp_off.iter().filter(|(q, _)| phai_editor::lsp::same_path(q, &p)).map(|(_, l)| *l).collect();
            phai_editor::view::set_disabled(h, &off);
        }
    }

    fn send_breakpoints(&mut self) {
        let all = self.all_breakpoints();
        if let Some(d) = &self.debug {
            d.send(phai_debug::Command::Breakpoints(all));
        }
    }

    /// An editor's breakpoints changed: kept, and the running program told.
    pub fn breakpoints_changed(&mut self, _h: HWND) {
        self.send_breakpoints();
        self.fill_breakpoints();
        self.mark_breakpoints();
    }

    fn launch_debugger(&mut self, exe: &Path) {
        self.debug_session(exe, None);
    }

    /// The Memory window shows what is at `at` - an address or a variable - now and at every stop.
    pub fn show_memory(&mut self, at: &str) {
        if !at.is_empty() {
            self.memory_at = at.to_string();
        }
        let Some(b) = self.bottom.as_ref() else { return };
        if win::text_of(b.memory_input) != self.memory_at {
            win::set_text(b.memory_input, &self.memory_at);
        }
        match (&self.debug, self.paused, self.memory_at.is_empty()) {
            (_, _, true) => {}
            (Some(d), true, false) => d.send(phai_debug::Command::Memory { id: EVAL_MEMORY, at: self.memory_at.clone(), len: 256 }),
            _ => win::set_text(b.memory, t("The program is not paused: start it with F5 and stop at a breakpoint.")),
        }
    }

    /// The Disassembly window, when it shows and the program is paused: the code asked for.
    pub fn ask_disassembly(&self) {
        let Some(b) = self.bottom.as_ref() else { return };
        if !unsafe { wm::IsWindowVisible(b.disassembly) }.as_bool() {
            return;
        }
        match (&self.debug, self.paused) {
            (Some(d), true) => d.send(phai_debug::Command::Disassemble),
            _ => win::set_text(b.disassembly, t("The program is not paused: start it with F5 and stop at a breakpoint.")),
        }
    }

    /// The Disassembly window filled: each source line, then its instructions; scrolled to where
    /// the program stopped.
    fn show_disassembly(&mut self, function: &str, lines: &[phai_debug::Asm]) {
        let Some(b) = self.bottom.as_ref() else { return };
        let mut files: std::collections::HashMap<PathBuf, Vec<String>> = std::collections::HashMap::new();
        let mut out = format!("{function}\r\n");
        let mut at = 0usize;
        for a in lines {
            if let Some((f, n)) = &a.source {
                let text = files.entry(f.clone()).or_insert_with(|| std::fs::read_to_string(f).unwrap_or_default().lines().map(str::to_string).collect());
                let code = text.get(*n as usize - 1).map(|l| l.trim_end().to_string()).unwrap_or_default();
                out.push_str(&format!("\r\n{:>5}: {code}\r\n", n));
            }
            if a.current {
                at = out.matches('\n').count();
            }
            out.push_str(&format!("{} {:016X}  {:<24} {}\r\n", if a.current { "→" } else { " " }, a.address, a.bytes, a.text));
        }
        if DEBUG_TEST.load(Ordering::Relaxed) > 0 {
            println!("disassembly: {function}, {} instructions; at: {}", lines.len(), out.lines().nth(at).unwrap_or("").trim());
        }
        win::set_text(b.disassembly, &out);
        unsafe {
            wm::SendMessageW(b.disassembly, 0x00B6 /* EM_LINESCROLL */, Some(WPARAM(0)), Some(LPARAM(at.saturating_sub(4) as isize)));
        }
    }

    /// Debug > New Data Breakpoint: stop when a variable changes - the local chosen in Locals by
    /// default.
    fn new_data_breakpoint(&mut self) {
        use windows::Win32::UI::Controls as cc;
        let (Some(d), true) = (&self.debug, self.paused) else {
            win::message(Some(self.hwnd), t("A data breakpoint is set while the program is paused: it watches a variable that exists then."), t("New Data Breakpoint..."), wm::MB_ICONINFORMATION);
            return;
        };
        let chosen = self.bottom.as_ref().map(|b| unsafe { wm::SendMessageW(b.locals, cc::LVM_GETNEXTITEM, Some(WPARAM(usize::MAX)), Some(LPARAM(2))).0 }).filter(|i| *i >= 0);
        let name = chosen.and_then(|i| self.bottom.as_ref().map(|b| win::list_text(b.locals, i as usize, 0))).unwrap_or_default();
        let expr = match std::env::var("PHAI_DATA_BP") {
            Ok(e) => e,
            Err(_) => match phai_ui::dialogs::ask(Some(self.hwnd), t("New Data Breakpoint"), t("Break when this variable changes (a name, or a field: order.total):"), &name, false) {
                Some(e) if !e.trim().is_empty() => e.trim().to_string(),
                _ => return,
            },
        };
        d.send(phai_debug::Command::DataBreakpoint(expr));
    }

    /// Exception Settings as remembered: whether a panic stops the program, and the exceptions
    /// that stop it at once.
    fn exceptions() -> phai_debug::Command {
        let words = setting("exceptions").unwrap_or_else(|| "panic".into());
        let on: Vec<&str> = words.split(',').map(str::trim).collect();
        let first_chance = EXCEPTIONS.iter().filter(|(c, _)| on.contains(&format!("0x{c:08X}").as_str())).map(|(c, _)| *c).collect();
        phai_debug::Command::Exceptions { panics: on.contains(&"panic"), first_chance }
    }

    /// Debug > Exception Settings: what stops the program - a panic, and which exceptions as soon
    /// as they happen (before the program's handlers); remembered, and the debugger told.
    fn exception_settings(&mut self) {
        let phai_debug::Command::Exceptions { panics, first_chance } = Self::exceptions() else { return };
        let label = |c: u32, name: &'static str| format!("{} (0x{c:08X})", t(name));
        let labels: Vec<String> = EXCEPTIONS.iter().map(|(c, n)| label(*c, n)).collect();
        let mut fields = vec![phai_ui::dialogs::Field::Check(t("Break when a Rust program panics"), panics)];
        for (i, (c, _)) in EXCEPTIONS.iter().enumerate() {
            fields.push(phai_ui::dialogs::Field::Check(&labels[i], first_chance.contains(c)));
        }
        let answer = match std::env::var("PHAI_EXCEPTIONS_TEST") {
            // `PHAI_EXCEPTIONS_TEST=1,0,1,...`: the ticks, without the question - for testing.
            Ok(t) => t.split(',').map(str::to_string).collect(),
            Err(_) => match phai_ui::dialogs::ask_fields(Some(self.hwnd), t("Exception Settings"), &fields) {
                Some(a) => a,
                None => return,
            },
        };
        let mut on = Vec::new();
        if answer.first().is_some_and(|v| v == "1") {
            on.push("panic".to_string());
        }
        for (i, (c, _)) in EXCEPTIONS.iter().enumerate() {
            if answer.get(i + 1).is_some_and(|v| v == "1") {
                on.push(format!("0x{c:08X}"));
            }
        }
        remember_setting("exceptions", &on.join(","));
        if let Some(d) = &self.debug {
            d.send(Self::exceptions());
        }
        self.say_status(&tf("Exception Settings: {0}", &[&if on.is_empty() { t("nothing stops the program but breakpoints").to_string() } else { on.join(", ") }]));
    }

    /// Debug > Attach to Process: a running program chosen, the debugger joined to it.
    fn attach_to_process(&mut self) {
        if self.debug.is_some() {
            return;
        }
        let project = self.doc.path.as_deref().and_then(cargo::project_of).and_then(|p| std::path::absolute(p).ok());
        // `PHAI_ATTACH_PID`: that process, without the question - for testing.
        let chosen = match std::env::var("PHAI_ATTACH_PID").ok().and_then(|p| p.parse().ok()) {
            Some(pid) => phai_debug::process_image(pid).map(|exe| (pid, exe)),
            None => attach::choose(self.hwnd, project.as_deref()),
        };
        if let Some((pid, exe)) = chosen {
            self.debug_session(&exe, Some(pid));
        }
    }

    /// The debugger started on `exe`, or joined to it already running as process `pid`.
    fn debug_session(&mut self, exe: &Path, pid: Option<u32>) {
        let bps = self.all_breakpoints();
        let target = self.hwnd.0 as isize;
        // It runs in its project, as `cargo run` runs it; the PDB names files from there. A program
        // attached to is from the project in front, or the one its file is built in.
        let built_in = || exe.ancestors().skip(1).find(|d| d.join("Cargo.toml").is_file()).map(Path::to_path_buf);
        let dir = self.doc.path.as_deref().and_then(cargo::project_of).and_then(|p| std::path::absolute(p).ok()).filter(|p| pid.is_none() || exe.starts_with(p)).or_else(built_in).or_else(|| exe.parent().map(Path::to_path_buf)).unwrap_or_default();
        let say = move |e| {
            let boxed = Box::into_raw(Box::new(e));
            unsafe {
                if wm::PostMessageW(Some(HWND(target as *mut _)), WM_DEBUG, WPARAM(0), LPARAM(boxed as isize)).is_err() {
                    drop(Box::from_raw(boxed));
                }
            }
        };
        self.debug = Some(match pid {
            Some(pid) => phai_debug::Session::attach(pid, &dir, bps, say),
            None => phai_debug::Session::launch(exe, &dir, bps, say),
        });
        self.paused = false;
        // Its process id comes once it has started: Diagnostic Tools ask for it on their timer.
        self.sampler = None;
        unsafe {
            wm::SetTimer(Some(self.hwnd), TIMER_DIAG, 500, None);
        }
        if !self.conds.is_empty() {
            if let Some(d) = &self.debug {
                d.send(phai_debug::Command::Conditions(self.conds.clone()));
            }
        }
        if let Some(d) = &self.debug {
            d.send(Self::exceptions());
        }
        self.show_debug_bar(true);
        let name = exe.file_name().unwrap_or_default().to_string_lossy().to_string();
        let words = match pid {
            Some(pid) => tf("Attached to {0} ({1})", &[&name, &pid]),
            None => tf("Debugging {0}", &[&name]),
        };
        controls::status_text(self.status, 0, &words);
        if let Some(b) = self.bottom.as_ref() {
            if pid.is_some() {
                b.add_line(&words);
            }
        }
        self.rebuild_menu();
    }

    fn show_debug_bar(&mut self, on: bool) {
        unsafe {
            wm::SendMessageW(self.rebar, 0x0423 /* RB_SHOWBAND */, Some(WPARAM(2)), Some(LPARAM(on as isize)));
        }
        win::rebar_fit(self.rebar);
        self.tray_height_changed();
    }

    fn clear_debug_line(&mut self) {
        for h in self.tabs.iter().filter_map(|t| t.doc.code).chain(self.doc.code) {
            phai_editor::view::set_current(h, None);
        }
    }

    /// What the debugger says, on the window's thread.
    fn debug_event(&mut self, e: phai_debug::Event) {
        // `--debugtest` acts after the stop is shown, as a person would.
        let test = (DEBUG_TEST.load(Ordering::Relaxed) > 0).then(|| e.clone());
        self.show_debug_event(e);
        if let Some(e) = test {
            self.debug_test_step(&e);
        }
    }

    fn show_debug_event(&mut self, e: phai_debug::Event) {
        use phai_debug::Event;
        match e {
            Event::Stopped { reason, file, line, frames, locals, threads } => {
                if let Some(b) = self.bottom.as_ref() {
                    use windows::Win32::UI::Controls as cc;
                    unsafe {
                        wm::SendMessageW(b.threads, cc::LVM_DELETEALLITEMS, None, None);
                    }
                    for th in &threads {
                        let name = if th.name.is_empty() { t("(no name)") } else { th.name.as_str() };
                        win::list_add(b.threads, &[if th.current { "→" } else { "" }, &th.id.to_string(), name, &th.location]);
                    }
                }
                self.thread_ids = threads.iter().map(|th| th.id).collect();
                if !self.memory_at.is_empty() {
                    if let Some(d) = &self.debug {
                        d.send(phai_debug::Command::Memory { id: EVAL_MEMORY, at: self.memory_at.clone(), len: 256 });
                    }
                }
                self.paused = true;
                self.ask_disassembly();
                self.paused = true;
                self.clear_debug_line();
                if let Some(f) = file.as_ref().filter(|f| f.is_file()) {
                    self.open_code(f);
                    if let Some(h) = self.doc.code {
                        phai_editor::view::set_current(h, Some(line));
                    }
                }
                if let Some(b) = self.bottom.as_ref() {
                    use windows::Win32::UI::Controls as cc;
                    unsafe {
                        wm::SendMessageW(b.locals, cc::LVM_DELETEALLITEMS, None, None);
                        wm::SendMessageW(b.stack, cc::LVM_DELETEALLITEMS, None, None);
                    }
                    for v in &locals {
                        win::list_add(b.locals, &[&v.name, &v.value, &v.ty]);
                    }
                    for f in &frames {
                        let file = f.file.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                        let line = if f.line > 0 { f.line.to_string() } else { String::new() };
                        win::list_add(b.stack, &[&f.function, &file, &line]);
                    }
                }
                self.frames = frames;
                self.refresh_watch();
                self.show_pane(Pane::Output, true);
                if let Some(b) = self.bottom.as_ref() {
                    b.show_tab(3);
                }
                let why = match reason.as_str() {
                    "breakpoint" => t("breakpoint"),
                    "step" => t("step"),
                    "panic" => t("panic"),
                    "pause" => t("Break All"),
                    r => r,
                }
                .to_string();
                let place = file.as_ref().and_then(|f| f.file_name()).map(|n| format!("{}:{line}", n.to_string_lossy())).unwrap_or_default();
                controls::status_text(self.status, 0, &tf("Paused - {0} - {1}", &[&why, &place]));
                if std::env::var_os("PHAI_SHOT").is_none() {
                    unsafe {
                        let _ = wm::SetForegroundWindow(self.hwnd);
                    }
                }
                self.rebuild_menu();
            }
            Event::Running => {
                self.paused = false;
                self.clear_debug_line();
                controls::status_text(self.status, 0, t("Running"));
                self.rebuild_menu();
            }
            Event::Output(text) => {
                if let Some(b) = self.bottom.as_ref() {
                    b.add_line(text.trim_end());
                }
            }
            Event::BreakpointAt { file, asked, line } => {
                let h = self.editor_of(&file);
                let mut lines: Vec<u32> = h.map(phai_editor::view::breakpoints).unwrap_or_default();
                lines.retain(|l| *l != asked);
                if line > 0 {
                    lines.push(line);
                } else {
                    controls::status_text(self.status, 0, &tf("No code at line {0}: that breakpoint will not be hit", &[&asked]));
                }
                if let Some(h) = h {
                    phai_editor::view::set_breakpoints(h, &lines);
                }
            }
            Event::Exited(code) => {
                self.debug = None;
                self.paused = false;
                self.clear_debug_line();
                self.show_debug_bar(false);
                let said = tf("The program ended with code {0}", &[&code]);
                if let Some(b) = self.bottom.as_ref() {
                    b.add_line(&said);
                }
                controls::status_text(self.status, 0, &said);
                self.rebuild_menu();
            }
            Event::ValueSet { expr, result } => match result {
                Ok(locals) => {
                    if let Some(b) = self.bottom.as_ref() {
                        use windows::Win32::UI::Controls as cc;
                        unsafe {
                            wm::SendMessageW(b.locals, cc::LVM_DELETEALLITEMS, None, None);
                        }
                        for v in &locals {
                            win::list_add(b.locals, &[&v.name, &v.value, &v.ty]);
                        }
                    }
                    self.refresh_watch();
                    self.say_status(&tf("{0} changed", &[&expr]));
                }
                Err(e) => {
                    win::message(Some(self.hwnd), &e, &tf("Change {0}", &[&expr]), wm::MB_ICONINFORMATION);
                }
            },
            Event::Values { id, vars } => {
                if id == EVAL_IMMEDIATE {
                    for v in &vars {
                        let line = if v.ty.is_empty() { v.value.clone() } else { format!("{}    {}", v.value, v.ty) };
                        if DEBUG_TEST.load(Ordering::Relaxed) > 0 {
                            println!("immediate: {line}");
                        }
                        if let Some(b) = self.bottom.as_ref() {
                            b.immediate_line(&line);
                        }
                    }
                } else if id == EVAL_WATCH {
                    if let Some(b) = self.bottom.as_ref() {
                        use windows::Win32::UI::Controls as cc;
                        unsafe {
                            wm::SendMessageW(b.watch, cc::LVM_DELETEALLITEMS, None, None);
                        }
                        for v in &vars {
                            win::list_add(b.watch, &[&v.name, &v.value, &v.ty]);
                        }
                    }
                } else if let Some((want, h, _)) = self.tip.take() {
                    // A data tip: shown only if it still means something.
                    match vars.first() {
                        Some(v) if want == id && !v.ty.is_empty() => phai_editor::view::show_tip(h, &format!("{} = {}    {}", v.name, v.value, v.ty)),
                        _ => {}
                    }
                }
            }
            Event::Memory { result, .. } => {
                let text = match result {
                    Ok((addr, bytes)) => hex_dump(addr, &bytes),
                    Err(e) => e,
                };
                if DEBUG_TEST.load(Ordering::Relaxed) > 0 {
                    println!("memory: {}", text.lines().next().unwrap_or(""));
                }
                if let Some(b) = self.bottom.as_ref() {
                    win::set_text(b.memory, &text);
                }
            }
            Event::Disassembly { function, lines } => self.show_disassembly(&function, &lines),
            Event::DataBreakpoint(result) => match result {
                Ok(what) => {
                    let said = tf("Data breakpoint: stops when {0} changes", &[&what]);
                    if DEBUG_TEST.load(Ordering::Relaxed) > 0 {
                        println!("{said}");
                    }
                    if let Some(b) = self.bottom.as_ref() {
                        b.add_line(&said);
                    }
                    self.say_status(&said);
                }
                Err(e) => {
                    win::message(Some(self.hwnd), &e, t("New Data Breakpoint..."), wm::MB_ICONINFORMATION);
                }
            },
            Event::Detached => {
                self.debug = None;
                self.paused = false;
                self.clear_debug_line();
                self.show_debug_bar(false);
                let said = t("Detached: the program runs on without the debugger");
                if let Some(b) = self.bottom.as_ref() {
                    b.add_line(said);
                }
                controls::status_text(self.status, 0, said);
                self.rebuild_menu();
            }
            Event::Failed(msg) => {
                self.debug = None;
                self.paused = false;
                self.show_debug_bar(false);
                win::message(Some(self.hwnd), &msg, t("The debugger could not start the program"), wm::MB_ICONWARNING);
                self.rebuild_menu();
            }
        }
    }

    /// `--debugtest`: F5 with a breakpoint on line `PHAI_DEBUG_LINE` (7), then a Step Over, its
    /// stops printed; being photographed, it stays at the second stop, else it stops the program.
    fn debug_test_step(&mut self, e: &phai_debug::Event) {
        match e {
            phai_debug::Event::Stopped { reason, line, locals, threads, .. } => {
                let vars: Vec<String> = locals.iter().map(|v| format!("{}={}", v.name, v.value)).collect();
                println!("stopped {reason} at line {line}: {}", vars.join(", "));
                println!("threads: {}", threads.iter().map(|t| format!("{} {}", if t.name.is_empty() { "-" } else { &t.name }, t.location)).collect::<Vec<_>>().join("; "));
                let n = DEBUG_TEST.fetch_add(1, Ordering::Relaxed);
                // `PHAI_MEMORY=at`: the Memory window shown at the first stop; `PHAI_DATA_BP`: a
                // data breakpoint set there, the line breakpoints taken away so it is what stops.
                if n == 1 {
                    // `PHAI_DISASM`: the Disassembly window shown at the first stop.
                    if std::env::var_os("PHAI_DISASM").is_some() {
                        self.paused = true;
                        self.command(ID_DISASSEMBLY);
                    }
                    if let Ok(at) = std::env::var("PHAI_MEMORY") {
                        self.paused = true;
                        self.show_memory(&at);
                    }
                    if std::env::var_os("PHAI_DATA_BP").is_some() {
                        self.paused = true;
                        self.new_data_breakpoint();
                        if let Some(h) = self.doc.code {
                            phai_editor::view::set_breakpoints(h, &[]);
                        }
                        if let Some(d) = &self.debug {
                            d.send(phai_debug::Command::Breakpoints(vec![]));
                            d.send(phai_debug::Command::Continue);
                        }
                        return;
                    }
                }
                // `PHAI_IMMEDIATE=expr`: asked in the Immediate window at the first stop.
                if n == 1 {
                    if let Ok(q) = std::env::var("PHAI_IMMEDIATE") {
                        self.paused = true;
                        self.immediate(&q);
                    }
                }
                if n == 2 && std::env::var_os("PHAI_SHOT").is_some() {
                    // The picture shows Watch at the second stop.
                    self.watches = vec!["x".into(), "y".into(), "y > 41".into()];
                    self.refresh_watch();
                    if let Some(b) = self.bottom.as_ref() {
                        // `PHAI_DEBUG_TAB`: another tab in the picture (11 Threads).
                        b.show_tab(std::env::var("PHAI_DEBUG_TAB").ok().and_then(|v| v.parse().ok()).unwrap_or(6));
                    }
                }
                if let Some(d) = &self.debug {
                    if n == 1 {
                        d.send(phai_debug::Command::StepOver);
                    } else if std::env::var_os("PHAI_SHOT").is_none() {
                        d.send(phai_debug::Command::Stop);
                    }
                }
            }
            phai_debug::Event::Exited(code) => {
                println!("ended {code}");
                std::process::exit(0);
            }
            phai_debug::Event::Detached => {
                println!("detached");
                std::process::exit(0);
            }
            phai_debug::Event::Failed(m) => {
                println!("failed {m}");
                std::process::exit(2);
            }
            _ => {}
        }
    }

    /// The Build menu: `job` for the form's project, or for its whole workspace (`whole`).
    fn start_job(&mut self, job: cargo::Job, whole: bool) {
        self.start_job_run(job, whole, false);
    }

    fn start_job_run(&mut self, job: cargo::Job, whole: bool, then_run: bool) {
        if self.building {
            return;
        }
        let Some(path) = self.doc.path.clone() else {
            win::message(Some(self.hwnd), t("Save the form in a project first - or make one with New project."), "Phai Studio", wm::MB_ICONINFORMATION);
            return;
        };
        if self.doc.dirty && !self.save(false) {
            return;
        }
        let found = if whole { cargo::workspace_of(&path) } else { cargo::project_of(&path) };
        let Some(dir) = found else {
            win::message(Some(self.hwnd), t("This form is not in a Cargo project - make one with New project."), "Phai Studio", wm::MB_ICONINFORMATION);
            return;
        };
        if then_run {
            self.stop();
        }
        self.building = true;
        controls::status_text(self.status, 0, match job {
            cargo::Job::Clean => t("Cleaning..."),
            cargo::Job::Analyse => t("Running code analysis..."),
            cargo::Job::Test => t("Running tests..."),
            _ => t("Building..."),
        });
        if let Some(b) = self.bottom.as_ref() {
            b.clear_output();
            b.show_tab(1);
        }
        self.show_errors(true);
        cargo::run_job(self.hwnd, dir, job, whole, then_run, self.release);
    }

    fn stop(&mut self) {
        if let Some(mut child) = self.running.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn show_errors(&mut self, on: bool) {
        self.show_pane(Pane::Output, on);
    }

    /// The layout item of window `h`.
    fn item(&mut self, h: HWND) -> Option<&mut Item> {
        self.layout.items.iter_mut().find(|(w, _)| *w == h).map(|(_, i)| i)
    }

    /// The bars as things are: Start named for the project, Debug or Release in its box, the box
    /// over its room, back and forward and the Layout bar usable only when they would do something.
    pub fn say_bars(&self) {
        let project = self.doc.path.as_deref().and_then(cargo::project_of).and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));
        let start = if self.paused { t("Continue").to_string() } else { project.unwrap_or_else(|| t("Start").to_string()) };
        win::toolbar_text(self.toolbar, ID_RUN, &start);
        let gap = toolbar_items().iter().position(|(_, l, _)| l.starts_with("gap:")).unwrap_or(0);
        let r = win::toolbar_item_rect(self.toolbar, gap);
        win::move_to(self.config, r.left, r.top + (r.bottom - r.top - win::px(self.toolbar, 24)) / 2, r.right - r.left, win::px(self.toolbar, 200));
        unsafe {
            wm::SendMessageW(self.config, 0x014E /* CB_SETCURSEL */, Some(WPARAM(self.release as usize)), None);
        }
        win::toolbar_enable(self.toolbar, ID_NAV_BACK, self.nav_at > 0);
        win::toolbar_enable(self.toolbar, ID_NAV_FORWARD, self.nav_at + 1 < self.nav.len());
        let design = self.doc.code.is_none();
        let n = self.selection().len();
        for (k, op) in FORMATS.iter().enumerate() {
            win::toolbar_enable(self.layout_bar, ID_FORMAT + k as i32, design && (n >= 2 || n == 1 && op.works_alone()));
        }
        win::toolbar_enable(self.layout_bar, ID_FRONT, design && n >= 1);
        win::toolbar_enable(self.layout_bar, ID_BACK, design && n >= 1);
        win::toolbar_enable(self.layout_bar, ID_TAB_ORDER, design);
        win::toolbar_check(self.layout_bar, ID_TAB_ORDER, self.canvas.as_ref().is_some_and(|c| c.tab_mode()));
        // Start's words change the bar's length: the tray must know, or its » never shows.
        win::rebar_fit(self.rebar);
    }

    /// Remembers where the caret is before a jump - F12, a reference, an error - for Navigate
    /// backward; what was ahead is forgotten, as in a browser.
    fn remember_place(&mut self) {
        let (Some(h), Some(path)) = (self.doc.code, self.doc.path.clone()) else { return };
        let at = phai_editor::view::caret_position(h);
        self.nav.truncate(self.nav_at);
        if self.nav.last() != Some(&(path.clone(), at)) {
            self.nav.push((path, at));
        }
        self.nav_at = self.nav.len();
        self.say_bars();
    }

    /// Navigate backward (`back`) or forward through the places jumped from.
    fn navigate(&mut self, back: bool) {
        if back {
            if self.nav_at == 0 {
                return;
            }
            // Leaving the newest place: keep it, so forward comes back to it.
            if self.nav_at == self.nav.len() {
                if let (Some(h), Some(path)) = (self.doc.code, self.doc.path.clone()) {
                    let here = (path, phai_editor::view::caret_position(h));
                    if self.nav.last() != Some(&here) {
                        self.nav.push(here);
                    }
                }
            }
            self.nav_at -= 1;
        } else {
            if self.nav_at + 1 >= self.nav.len() {
                return;
            }
            self.nav_at += 1;
        }
        let (path, (line, col)) = self.nav[self.nav_at].clone();
        self.open_code(&path);
        if let Some(h) = self.doc.code {
            phai_editor::view::go_to(h, line, col);
        }
        self.say_bars();
    }

    /// File > Save all: every tab with changes, the one in front last shown again.
    fn save_all(&mut self) {
        let front = self.active;
        for i in 0..self.tabs.len() {
            let dirty = if i == self.active { self.doc.dirty } else { self.tabs[i].doc.dirty };
            if dirty {
                self.activate(i);
                if !self.save(false) {
                    return;
                }
            }
        }
        self.activate(front);
    }

    /// The toolbox's room in the layout as it is docked: its width at the left or right, its height
    /// at the top or bottom; none when hidden.
    fn size_toolbox(&mut self, on: bool) {
        let across = matches!(self.toolbox_dock, Dock::Left | Dock::Right);
        let (tb, split, w, h) = (self.toolbox, self.toolbox_split, self.toolbox_w, self.toolbox_h);
        if let Some(i) = self.item(tb) {
            i.bounds.w = if on && across { w } else { 0 };
            i.bounds.h = if on && !across { h } else { 0 };
        }
        if let Some(i) = self.item(split) {
            i.bounds.w = if on && across { 6 } else { 0 };
            i.bounds.h = if on && !across { 6 } else { 0 };
        }
    }

    /// Dragging the toolbox by its caption: a shaded box follows the pointer to show where it
    /// would go - an edge, or floating - until the button comes up; Escape leaves it where it was.
    fn drag_toolbox(&mut self) {
        let (across, down) = (self.toolbox_w, self.toolbox_h);
        if let Some(t) = self.drag_target(across, down, &[Dock::Left, Dock::Right, Dock::Top, Dock::Bottom], true) {
            self.dock_toolbox(t);
        }
    }

    /// Dragging Project, Server Explorer or Properties by its caption: the side panes go together
    /// to the left or the right edge.
    fn drag_side(&mut self) {
        if let Some(t) = self.drag_target(self.side_w, self.side_h, &[Dock::Left, Dock::Right, Dock::Top, Dock::Bottom], true) {
            self.dock_side(t);
        }
    }

    /// The side panes against an edge - their splitter on the inner side, turned the right way -
    /// or in a window of their own. Where they are is remembered for next time.
    pub fn dock_side(&mut self, target: docking::Target) {
        let side = self.side;
        // Out of a floating window first: back among the main window's children, before the splitter.
        if let Some(f) = self.side_float.take() {
            unsafe {
                let _ = wm::SetParent(side, Some(self.hwnd));
                let _ = wm::DestroyWindow(f);
            }
            let at = self.layout.items.iter().position(|(h, _)| *h == self.side_split).unwrap_or(0);
            self.layout.items.insert(at, (side, Item { bounds: Rect::new(0, 0, 0, 0), dock: self.side_dock, anchor: Anchor::default() }));
        }
        match target {
            docking::Target::Edge(d) => {
                let d = if matches!(d, Dock::Left | Dock::Right | Dock::Top | Dock::Bottom) { d } else { Dock::Right };
                self.side_dock = d;
                let across = matches!(d, Dock::Left | Dock::Right);
                let old = self.side_split;
                let split = controls::splitter(self.hwnd, ID_SPLIT_SIDE, across);
                if let Some(e) = self.layout.items.iter_mut().find(|(h, _)| *h == old) {
                    e.0 = split;
                    e.1.dock = d;
                }
                unsafe {
                    let _ = wm::DestroyWindow(old);
                }
                self.side_split = split;
                if let Some(i) = self.item(side) {
                    i.dock = d;
                }
                let any = { let (e, p) = explorer::side_panes(side); e || p };
                self.size_side(any);
                win::show(side, any);
                win::show(split, any);
                remember_setting("side_dock", match d {
                    Dock::Left => "left",
                    Dock::Top => "top",
                    Dock::Bottom => "bottom",
                    _ => "right",
                });
            }
            docking::Target::Float(r) => {
                self.layout.items.retain(|(h, _)| *h != side);
                if let Some(i) = self.item(self.side_split) {
                    i.bounds.w = 0;
                    i.bounds.h = 0;
                }
                win::show(self.side_split, false);
                let f = docking::float_window(self.hwnd, t("Properties"), r);
                unsafe {
                    let _ = wm::SetParent(side, Some(f));
                }
                let mut c = RECT::default();
                unsafe {
                    let _ = wm::GetClientRect(f, &mut c);
                }
                win::move_to(side, 0, 0, c.right, c.bottom);
                win::show(side, true);
                win::show(f, true);
                self.side_float = Some(f);
                remember_setting("side_dock", "float");
            }
        }
        self.layout.apply(self.hwnd);
        self.refresh_strips();
        self.rebuild_menu();
    }

    /// The side panes' room in the layout as they are docked: their width at the left or right,
    /// their height at the top or bottom; none when hidden.
    fn size_side(&mut self, on: bool) {
        let across = matches!(self.side_dock, Dock::Left | Dock::Right);
        let (side, split, w, h) = (self.side, self.side_split, self.side_w, self.side_h);
        if let Some(i) = self.item(side) {
            i.bounds.w = if on && across { w } else { 0 };
            i.bounds.h = if on && !across { h } else { 0 };
        }
        if let Some(i) = self.item(split) {
            i.bounds.w = if on && across { 6 } else { 0 };
            i.bounds.h = if on && !across { 6 } else { 0 };
        }
    }

    /// The drag a pane's caption starts: a shaded box where the pane would go, the dock guides
    /// for the edges in `docks`, until the button comes up; Escape, or a place it may not go,
    /// gives `None`. `across` and `down` are the pane's design width and height.
    fn drag_target(&mut self, across: i32, down: i32, docks: &[Dock], can_float: bool) -> Option<docking::Target> {
        unsafe {
            let mut start = windows::Win32::Foundation::POINT::default();
            let _ = wm::GetCursorPos(&mut start);
            // The room the panes share: under the bars, above the status bar.
            let mut top_r = RECT::default();
            let mut status_r = RECT::default();
            let mut client = RECT::default();
            let _ = wm::GetWindowRect(self.rebar, &mut top_r);
            let _ = wm::GetWindowRect(self.status, &mut status_r);
            let _ = wm::GetClientRect(self.hwnd, &mut client);
            let mut pts = [windows::Win32::Foundation::POINT { x: client.left, y: client.top }, windows::Win32::Foundation::POINT { x: client.right, y: client.bottom }];
            Gdi::MapWindowPoints(Some(self.hwnd), None, &mut pts);
            let area = RECT { left: pts[0].x, top: top_r.bottom, right: pts[1].x, bottom: status_r.top };
            let (across, down) = (win::px(self.hwnd, across), win::px(self.hwnd, down));
            let zone = win::px(self.hwnd, 56);
            let preview = docking::preview_window(self.hwnd);
            let mut guide_rects = docking::guides(area, win::px(self.hwnd, 36));
            guide_rects.retain(|(d, _)| docks.contains(d));
            let mut guide_windows: Vec<HWND> = vec![];
            let mut moved_off = false;
            km::SetCapture(self.hwnd);
            let mut target: Option<docking::Target> = None;
            let mut msg = wm::MSG::default();
            loop {
                if !wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
                    break;
                }
                match msg.message {
                    wm::WM_MOUSEMOVE => {
                        let mut pt = windows::Win32::Foundation::POINT::default();
                        let _ = wm::GetCursorPos(&mut pt);
                        if target.is_none() && (pt.x - start.x).abs() < 6 && (pt.y - start.y).abs() < 6 {
                            continue;
                        }
                        // The guides show once the drag begins; the one under the pointer lights.
                        if guide_windows.is_empty() {
                            guide_windows = guide_rects.iter().map(|(d, r)| docking::guide_window(self.hwnd, *d, *r)).collect();
                        }
                        for (g, (_, r)) in guide_windows.iter().zip(&guide_rects) {
                            docking::light_guide(*g, pt.x >= r.left && pt.x < r.right && pt.y >= r.top && pt.y < r.bottom);
                        }
                        let t = match docking::guide_at(pt, &guide_rects) {
                            Some(d) => docking::Target::Edge(d),
                            None => docking::target_at(pt, area, zone, (across, win::px(self.hwnd, 460))),
                        };
                        // Only where this pane may go: elsewhere no box, and letting go leaves it.
                        let allowed = match t {
                            docking::Target::Edge(d) => docks.contains(&d),
                            docking::Target::Float(_) => can_float,
                        };
                        if allowed {
                            let r = docking::preview_rect(t, area, across, down);
                            let _ = wm::SetWindowPos(preview, Some(wm::HWND_TOPMOST), r.left, r.top, r.right - r.left, r.bottom - r.top, wm::SWP_NOACTIVATE | wm::SWP_SHOWWINDOW);
                        } else {
                            let _ = wm::ShowWindow(preview, wm::SW_HIDE);
                        }
                        target = Some(t);
                        moved_off = !allowed;
                    }
                    wm::WM_LBUTTONUP => break,
                    wm::WM_KEYDOWN if msg.wParam.0 == 0x1B => {
                        target = None;
                        break;
                    }
                    wm::WM_CAPTURECHANGED => break,
                    _ => {
                        let _ = wm::TranslateMessage(&msg);
                        wm::DispatchMessageW(&msg);
                    }
                }
            }
            let _ = km::ReleaseCapture();
            let _ = wm::DestroyWindow(preview);
            for g in guide_windows {
                let _ = wm::DestroyWindow(g);
            }
            if moved_off {
                None
            } else {
                target
            }
        }
    }

    /// Puts the toolbox against an edge of the window, or in a window of its own.
    pub fn dock_toolbox(&mut self, target: docking::Target) {
        let tb = self.toolbox;
        // Out of a floating window first: back among the main window's children, in its old
        // place in the layout - before the splitter, so it stays outside it.
        if let Some(f) = self.toolbox_float.take() {
            unsafe {
                let _ = wm::SetParent(tb, Some(self.hwnd));
                let _ = wm::DestroyWindow(f);
            }
            let at = self.layout.items.iter().position(|(h, _)| *h == self.toolbox_split).unwrap_or(0);
            self.layout.items.insert(at, (tb, Item { bounds: Rect::new(0, 0, 0, 0), dock: self.toolbox_dock, anchor: Anchor::default() }));
        }
        match target {
            docking::Target::Edge(d) => {
                let d = if matches!(d, Dock::Left | Dock::Right | Dock::Top | Dock::Bottom) { d } else { Dock::Left };
                self.toolbox_dock = d;
                remember_setting("toolbox_dock", match d {
                    Dock::Right => "right",
                    Dock::Top => "top",
                    Dock::Bottom => "bottom",
                    _ => "left",
                });
                // A splitter that drags the right way: across for a side, down for top or bottom.
                let across = matches!(d, Dock::Left | Dock::Right);
                let old = self.toolbox_split;
                let split = controls::splitter(self.hwnd, ID_SPLIT_TOOLBOX, across);
                if let Some(e) = self.layout.items.iter_mut().find(|(h, _)| *h == old) {
                    e.0 = split;
                    e.1.dock = d;
                }
                unsafe {
                    let _ = wm::DestroyWindow(old);
                }
                self.toolbox_split = split;
                if let Some(i) = self.item(tb) {
                    i.dock = d;
                }
                self.size_toolbox(self.toolbox_on);
                win::show(tb, self.toolbox_on);
                win::show(split, self.toolbox_on);
            }
            docking::Target::Float(r) => {
                // Out of the layout, into a window of its own.
                self.layout.items.retain(|(h, _)| *h != tb);
                if let Some(i) = self.item(self.toolbox_split) {
                    i.bounds.w = 0;
                    i.bounds.h = 0;
                }
                win::show(self.toolbox_split, false);
                let f = docking::float_window(self.hwnd, docking::title(), r);
                unsafe {
                    let _ = wm::SetParent(tb, Some(f));
                }
                let mut c = RECT::default();
                unsafe {
                    let _ = wm::GetClientRect(f, &mut c);
                }
                win::move_to(tb, 0, 0, c.right, c.bottom);
                win::show(tb, true);
                win::show(f, true);
                self.toolbox_float = Some(f);
                self.toolbox_on = true;
            }
        }
        self.layout.apply(self.hwnd);
        self.rebuild_menu();
    }

    /// A bar's ▾: Add or Remove Buttons - every button with a tick when it shows; one clicked
    /// shows or hides, and the choice is kept for the next time. Reset puts them all back.
    fn customise_bar(&mut self, bar: HWND, own: i32, items: Vec<(i32, &'static str, u16)>) {
        let mut r = win::toolbar_button_rect(bar, own);
        unsafe {
            let mut pt = windows::Win32::Foundation::POINT { x: r.left, y: r.bottom };
            let _ = Gdi::ClientToScreen(bar, &mut pt);
            r.left = pt.x;
            r.bottom = pt.y;
            let m = wm::CreatePopupMenu().unwrap_or_default();
            let sub = wm::CreatePopupMenu().unwrap_or_default();
            let buttons: Vec<(i32, &str)> = items.iter().filter(|(c, _, _)| *c != 0 && *c != own).map(|(c, l, _)| (*c, *l)).collect();
            for (k, (cmd, label)) in buttons.iter().enumerate() {
                let w = wide(label);
                let on = win::toolbar_button_shown(bar, *cmd);
                let _ = wm::AppendMenuW(sub, wm::MF_STRING | if on { wm::MF_CHECKED } else { wm::MF_UNCHECKED }, k + 1, PCWSTR(w.as_ptr()));
            }
            let w = wide(t("Add or Remove Buttons"));
            let _ = wm::AppendMenuW(m, wm::MF_POPUP, sub.0 as usize, PCWSTR(w.as_ptr()));
            let _ = wm::AppendMenuW(m, wm::MF_SEPARATOR, 0, PCWSTR::null());
            let w = wide(t("Reset Toolbar"));
            const RESET: usize = 9999;
            let _ = wm::AppendMenuW(m, wm::MF_STRING, RESET, PCWSTR(w.as_ptr()));
            let chosen = wm::TrackPopupMenu(m, wm::TPM_RETURNCMD | wm::TPM_LEFTALIGN | wm::TPM_TOPALIGN, r.left, r.bottom, None, self.hwnd, None).0 as usize;
            let _ = wm::DestroyMenu(m);
            if chosen == RESET {
                for (cmd, _) in &buttons {
                    win::toolbar_show_button(bar, *cmd, true);
                }
            } else if let Some((cmd, _)) = chosen.checked_sub(1).and_then(|k| buttons.get(k)) {
                win::toolbar_show_button(bar, *cmd, !win::toolbar_button_shown(bar, *cmd));
            } else {
                return;
            }
        }
        self.remember_hidden();
        self.say_bars();
    }

    /// The buttons hidden on both bars, kept in the settings for the next start.
    fn remember_hidden(&self) {
        let mut hidden: Vec<String> = Vec::new();
        for (bar, items) in [(self.toolbar, toolbar_items()), (self.layout_bar, layout_items())] {
            for (cmd, _, _) in items {
                if cmd != 0 && !win::toolbar_button_shown(bar, cmd) {
                    hidden.push(cmd.to_string());
                }
            }
        }
        remember_setting("hidden_buttons", &hidden.join(","));
    }

    /// The buttons hidden last time, hidden again.
    fn restore_hidden(&self) {
        let hidden: Vec<i32> = setting("hidden_buttons").unwrap_or_default().split(',').filter_map(|n| n.trim().parse().ok()).collect();
        for (bar, items) in [(self.toolbar, toolbar_items()), (self.layout_bar, layout_items())] {
            for (cmd, _, _) in items {
                if hidden.contains(&cmd) {
                    win::toolbar_show_button(bar, cmd, false);
                }
            }
        }
    }

    /// The tray's rows changed - a bar dragged onto another row, or up beside the other: the panes
    /// below take the room that is left.
    fn tray_height_changed(&mut self) {
        let h = phai_ui::layout::unscale(win::rebar_height(self.rebar), win::dpi(self.hwnd)).max(8);
        let rb = self.rebar;
        if let Some(i) = self.item(rb) {
            if i.bounds.h == h {
                return;
            }
            i.bounds.h = h;
        }
        self.layout.apply(self.hwnd);
    }

    /// A bar's » pressed: the buttons it has no room for, as a menu under it.
    fn tray_chevron(&mut self, band_child: HWND, at: RECT) {
        let mut tr = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(band_child, &mut tr);
        }
        let hidden = win::toolbar_hidden(band_child, tr.right);
        if hidden.is_empty() {
            return;
        }
        unsafe {
            let m = wm::CreatePopupMenu().unwrap_or_default();
            for (cmd, label) in &hidden {
                let w = wide(label);
                let _ = wm::AppendMenuW(m, wm::MF_STRING, *cmd as usize, PCWSTR(w.as_ptr()));
            }
            let mut pt = windows::Win32::Foundation::POINT { x: at.left, y: at.bottom };
            let _ = Gdi::ClientToScreen(self.rebar, &mut pt);
            let chosen = wm::TrackPopupMenu(m, wm::TPM_RETURNCMD | wm::TPM_LEFTALIGN | wm::TPM_TOPALIGN, pt.x, pt.y, None, self.hwnd, None).0;
            let _ = wm::DestroyMenu(m);
            if chosen != 0 {
                self.command(chosen);
            }
        }
    }

    /// Whether a pane hides itself at its edge.
    fn is_auto(&self, pane: Pane) -> bool {
        match pane {
            Pane::Toolbox => self.auto_left,
            Pane::Explorer | Pane::Properties => self.auto_right,
            Pane::Output => false,
        }
    }

    /// A pane caption's button: ▾ the Window Position menu, the pin Auto Hide, × Close.
    pub fn pane_button(&mut self, pane: Pane, code: u16, caption: HWND) {
        match code {
            controls::CAPTION_CLOSE => self.show_pane(pane, false),
            controls::CAPTION_PIN => self.set_auto_hide(pane, !self.is_auto(pane)),
            controls::CAPTION_DRAG if pane == Pane::Toolbox && !self.auto_left => self.drag_toolbox(),
            controls::CAPTION_DRAG if matches!(pane, Pane::Explorer | Pane::Properties) && !self.auto_right => self.drag_side(),
            controls::CAPTION_MENU => {
                const FLOAT: usize = 1;
                const DOCK: usize = 2;
                const AUTO: usize = 3;
                const HIDE: usize = 4;
                let auto = self.is_auto(pane);
                unsafe {
                    let m = wm::CreatePopupMenu().unwrap_or_default();
                    let item = |id: usize, label: &str, flags: wm::MENU_ITEM_FLAGS| {
                        let w = wide(label);
                        let _ = wm::AppendMenuW(m, flags, id, PCWSTR(w.as_ptr()));
                    };
                    let floats = pane == Pane::Toolbox && self.toolbox_float.is_some();
                    item(FLOAT, t("Float"), wm::MF_STRING | if pane != Pane::Toolbox { wm::MF_GRAYED } else if floats { wm::MF_CHECKED } else { wm::MF_UNCHECKED });
                    item(DOCK, t("Dock"), wm::MF_STRING | if auto || floats { wm::MF_UNCHECKED } else { wm::MF_CHECKED });
                    item(AUTO, t("Auto Hide"), wm::MF_STRING | if auto { wm::MF_CHECKED } else { wm::MF_UNCHECKED });
                    item(HIDE, t("Hide"), wm::MF_STRING);
                    let (x, y) = controls::caption_menu_point(caption);
                    let chosen = wm::TrackPopupMenu(m, wm::TPM_RETURNCMD | wm::TPM_LEFTALIGN | wm::TPM_TOPALIGN, x, y, None, self.hwnd, None).0 as usize;
                    let _ = wm::DestroyMenu(m);
                    match chosen {
                        FLOAT => {
                            let mut r = RECT::default();
                            let _ = wm::GetWindowRect(self.toolbox, &mut r);
                            let r = RECT { left: r.left + 40, top: r.top + 40, right: r.left + 40 + win::px(self.hwnd, self.toolbox_w), bottom: r.top + 40 + win::px(self.hwnd, 460) };
                            self.dock_toolbox(docking::Target::Float(r));
                        }
                        DOCK if floats => self.dock_toolbox(docking::Target::Edge(self.toolbox_dock)),
                        DOCK => self.set_auto_hide(pane, false),
                        AUTO => self.set_auto_hide(pane, true),
                        HIDE => self.show_pane(pane, false),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    /// Auto Hide on or off: the pane gives its room back to the form and becomes a tab at its edge,
    /// or comes back to stay.
    pub fn set_auto_hide(&mut self, pane: Pane, on: bool) {
        self.slide_in();
        match pane {
            Pane::Toolbox => {
                self.auto_left = on;
                if on && self.toolbox_float.is_some() {
                    self.dock_toolbox(docking::Target::Edge(self.toolbox_dock));
                }
                self.auto_left = on;
                let (tb, split, shown) = (self.toolbox, self.toolbox_split, self.toolbox_on);
                self.size_toolbox(!on && shown);
                win::show(tb, !on && shown);
                win::show(split, !on && shown);
                controls::caption_set_auto_hidden(toolbox::caption_of(tb), on);
            }
            Pane::Explorer | Pane::Properties => {
                if on == self.auto_right {
                    return;
                }
                if on && (self.side_dock != Dock::Right || self.side_float.is_some()) {
                    self.dock_side(docking::Target::Edge(Dock::Right));
                }
                self.auto_right = on;
                let (side, split) = (self.side, self.side_split);
                if on {
                    self.right_saved = explorer::side_panes(side);
                } else {
                    explorer::show_side_panes(side, self.right_saved.0, self.right_saved.1);
                }
                self.size_side(!on);
                win::show(side, !on);
                win::show(split, !on);
                if let Some((a, b)) = explorer::side_captions(side) {
                    controls::caption_set_auto_hidden(a, on);
                    controls::caption_set_auto_hidden(b, on);
                }
            }
            Pane::Output => {}
        }
        self.refresh_strips();
        self.layout.apply(self.hwnd);
        self.rebuild_menu();
    }

    /// The strips' tabs, and room for a strip only while it has some.
    fn refresh_strips(&mut self) {
        let left: Vec<Pane> = if self.auto_left && self.toolbox_on { vec![Pane::Toolbox] } else { vec![] };
        let mut right: Vec<Pane> = Vec::new();
        if self.auto_right {
            if self.right_saved.0 {
                right.push(Pane::Explorer);
            }
            if self.right_saved.1 {
                right.push(Pane::Properties);
            }
        }
        for (strip, tabs) in [(self.strip_left, left), (self.strip_right, right)] {
            let any = !tabs.is_empty();
            autohide::set_tabs(strip, tabs);
            if let Some(i) = self.item(strip) {
                i.bounds.w = if any { autohide::WIDTH } else { 0 };
            }
            win::show(strip, any);
        }
    }

    /// An auto-hidden pane, slid out over the form beside its strip, until the pointer and the
    /// focus leave it.
    pub fn slide_out(&mut self, pane: Pane) {
        if self.sliding == Some(pane) {
            return;
        }
        self.slide_in();
        let strip = if pane == Pane::Toolbox { self.strip_left } else { self.strip_right };
        let mut r = RECT::default();
        unsafe {
            let _ = wm::GetWindowRect(strip, &mut r);
            let mut pts = [windows::Win32::Foundation::POINT { x: r.left, y: r.top }, windows::Win32::Foundation::POINT { x: r.right, y: r.bottom }];
            Gdi::MapWindowPoints(None, Some(self.hwnd), &mut pts);
            r = RECT { left: pts[0].x, top: pts[0].y, right: pts[1].x, bottom: pts[1].y };
            // Siblings must not paint over the pane that lies across them.
            for (h, _) in &self.layout.items {
                let st = wm::GetWindowLongW(*h, wm::GWL_STYLE) as u32;
                wm::SetWindowLongW(*h, wm::GWL_STYLE, (st | wm::WS_CLIPSIBLINGS.0) as i32);
            }
        }
        let (h, x, w) = if pane == Pane::Toolbox {
            let w = win::px(self.hwnd, self.toolbox_w);
            (self.toolbox, r.right, w)
        } else {
            explorer::show_side_panes(self.side, pane == Pane::Explorer, pane == Pane::Properties);
            let w = win::px(self.hwnd, self.side_w);
            (self.side, r.left - w, w)
        };
        unsafe {
            let _ = wm::SetWindowPos(h, Some(wm::HWND_TOP), x, r.top, w, r.bottom - r.top, wm::SWP_SHOWWINDOW);
            wm::SetTimer(Some(self.hwnd), TIMER_SLIDE, 250, None);
        }
        self.sliding = Some(pane);
    }

    /// The pane slid out goes back to its strip.
    fn slide_in(&mut self) {
        let Some(pane) = self.sliding.take() else { return };
        unsafe {
            let _ = wm::KillTimer(Some(self.hwnd), TIMER_SLIDE);
        }
        if pane == Pane::Toolbox {
            win::show(self.toolbox, false);
        } else {
            win::show(self.side, false);
            explorer::show_side_panes(self.side, self.right_saved.0, self.right_saved.1);
        }
    }

    /// The slide timer: the pane stays while the pointer is over it or its strip, or the focus is
    /// in it. A program being photographed ignores the real pointer.
    fn slide_check(&mut self) {
        let Some(pane) = self.sliding else { return };
        if std::env::var_os("PHAI_SHOT").is_some() {
            return;
        }
        let (h, strip) = if pane == Pane::Toolbox { (self.toolbox, self.strip_left) } else { (self.side, self.strip_right) };
        unsafe {
            let mut pt = windows::Win32::Foundation::POINT::default();
            let _ = wm::GetCursorPos(&mut pt);
            let inside = |w: HWND| {
                let mut r = RECT::default();
                let _ = wm::GetWindowRect(w, &mut r);
                pt.x >= r.left && pt.x < r.right && pt.y >= r.top && pt.y < r.bottom
            };
            let focus = km::GetFocus();
            let focused = !focus.is_invalid() && (focus == h || wm::IsChild(h, focus).as_bool());
            let held = km::GetKeyState(0x01) < 0 || !km::GetCapture().is_invalid();
            if !(inside(h) || inside(strip) || focused || held) {
                self.slide_in();
            }
        }
    }

    /// Shows or hides a docked pane, as the View menu and the panes' close buttons do.
    pub fn show_pane(&mut self, pane: Pane, on: bool) {
        // An auto-hidden pane asked for slides out; one closed stops hiding itself first.
        if self.is_auto(pane) {
            if on {
                if pane == Pane::Toolbox {
                    self.toolbox_on = true;
                } else if pane == Pane::Explorer {
                    self.right_saved.0 = true;
                } else {
                    self.right_saved.1 = true;
                }
                self.refresh_strips();
                self.layout.apply(self.hwnd);
                return self.slide_out(pane);
            }
            self.set_auto_hide(pane, false);
        }
        match pane {
            Pane::Toolbox => {
                self.toolbox_on = on;
                if let Some(f) = self.toolbox_float {
                    win::show(f, on);
                    win::show(self.toolbox, on);
                } else {
                    self.size_toolbox(on);
                    win::show(self.toolbox, on);
                    win::show(self.toolbox_split, on);
                }
            }
            Pane::Explorer | Pane::Properties => {
                let (mut e, mut p) = explorer::side_panes(self.side);
                if pane == Pane::Explorer {
                    e = on;
                } else {
                    p = on;
                }
                let any = explorer::show_side_panes(self.side, e, p);
                let (side, split) = (self.side, self.side_split);
                if let Some(f) = self.side_float {
                    win::show(f, any);
                } else {
                    self.size_side(any);
                    win::show(split, any);
                }
                win::show(side, any);
            }
            Pane::Output => {
                self.bottom_on = on;
                let (bh, split, h) = (self.bottom.as_ref().map(|b| b.hwnd).unwrap_or_default(), self.bottom_split, self.bottom_h);
                if let Some(i) = self.item(bh) {
                    i.bounds.h = if on { h } else { 0 };
                }
                if let Some(i) = self.item(split) {
                    i.bounds.h = if on { 6 } else { 0 };
                }
                win::show(bh, on);
                win::show(split, on);
            }
        }
        self.layout.apply(self.hwnd);
        self.rebuild_menu();
    }

    /// A splitter dragged `d` device pixels: the pane beside it takes or gives the room.
    fn split(&mut self, id: i32, d: i32) {
        let d = phai_ui::layout::unscale(d, win::dpi(self.hwnd));
        if id == ID_SPLIT_TOOLBOX {
            match self.toolbox_dock {
                Dock::Left => self.toolbox_w = (self.toolbox_w + d).clamp(140, 600),
                Dock::Right => self.toolbox_w = (self.toolbox_w - d).clamp(140, 600),
                Dock::Top => self.toolbox_h = (self.toolbox_h + d).clamp(80, 600),
                _ => self.toolbox_h = (self.toolbox_h - d).clamp(80, 600),
            }
            self.size_toolbox(true);
        } else if id == ID_SPLIT_SIDE {
            match self.side_dock {
                Dock::Left => self.side_w = (self.side_w + d).clamp(180, 900),
                Dock::Right => self.side_w = (self.side_w - d).clamp(180, 900),
                Dock::Top => self.side_h = (self.side_h + d).clamp(120, 900),
                _ => self.side_h = (self.side_h - d).clamp(120, 900),
            }
            self.size_side(true);
        } else {
            self.bottom_h = (self.bottom_h - d).clamp(60, 900);
            let (bh, h) = (self.bottom.as_ref().map(|b| b.hwnd).unwrap_or_default(), self.bottom_h);
            if let Some(i) = self.item(bh) {
                i.bounds.h = h;
            }
        }
        self.layout.apply(self.hwnd);
    }

    fn build_done(&mut self, out: cargo::Outcome) {
        self.building = false;
        if out.no_cargo {
            controls::status_text(self.status, 0, t("Rust is not installed"));
            let answer = win::message(Some(self.hwnd), t("Building needs Rust, and it is not on this computer. Open rustup.rs to install it?"), "Phai Studio", wm::MB_YESNO | wm::MB_ICONINFORMATION);
            if answer == wm::IDYES {
                open_with_windows("https://rustup.rs");
            }
            return;
        }
        self.diagnostics = out.diagnostics.clone();
        if let Some(e) = &out.cargo_error {
            self.diagnostics.insert(0, cargo::Diagnostic { error: true, message: e.clone(), file: None, line: 0, column: 0 });
        }
        self.fill_errors();
        let errors = self.diagnostics.iter().filter(|d| d.error).count();
        let warnings = self.diagnostics.len() - errors;
        // Publish: built as Release, now packed.
        if let Some(m) = self.publish_after_build.take() {
            if out.ok {
                if let Some(b) = self.bottom.as_ref() {
                    b.add_line(&format!("{} ({:.1} s)", t("Build succeeded"), out.seconds));
                }
                match out.executable.clone() {
                    Some(exe) => self.finish_publish(m, &exe),
                    None => controls::status_text(self.status, 0, t("The project has no program to run")),
                }
                return;
            }
            if PUBLISH_TEST.load(Ordering::Relaxed) {
                println!("publish: the build failed");
                std::process::exit(3);
            }
        }
        // F5: built, now run under the debugger.
        if std::mem::take(&mut self.debug_after_build) && out.ok {
            if let Some(b) = self.bottom.as_ref() {
                b.add_line(&format!("{} ({:.1} s)", t("Build succeeded"), out.seconds));
            }
            match out.executable.clone() {
                Some(exe) => self.launch_debugger(&exe),
                None => controls::status_text(self.status, 0, t("The project has no program to run")),
            }
            return;
        }
        // Run Tests: each test in the Tests list.
        if out.job == cargo::Job::Test {
            self.show_tests(&out);
            return;
        }
        // Problems show their list; a clean build leaves its output on screen.
        if let Some(b) = self.bottom.as_ref() {
            let said = match (out.job, out.ok) {
                (cargo::Job::Clean, true) => t("Clean succeeded"),
                (cargo::Job::Clean, false) => t("Clean failed"),
                (cargo::Job::Analyse, true) => t("Code analysis succeeded"),
                (cargo::Job::Analyse, false) => t("Code analysis failed"),
                (_, true) => t("Build succeeded"),
                (_, false) => t("Build failed"),
            };
            b.add_line(&format!("{said} ({:.1} s)", out.seconds));
            if !self.diagnostics.is_empty() {
                b.show_tab(0);
            }
        }
        if out.ok && out.job == cargo::Job::Clean {
            controls::status_text(self.status, 0, &tf("Cleaned in {0} s", &[&format!("{:.1}", out.seconds)]));
        } else if out.ok && out.job == cargo::Job::Analyse {
            controls::status_text(self.status, 0, &tf("Code analysis: {0} errors, {1} warnings", &[&errors, &warnings]));
        } else if out.ok {
            controls::status_text(self.status, 0, &tf("Built in {0} s - {1} warnings", &[&format!("{:.1}", out.seconds), &warnings]));
            if out.then_run {
                match out.executable.as_deref().map(|e| cargo::run(e, self.hwnd)) {
                    Some(Ok(child)) => {
                        self.watch_process(child.id());
                        self.running = Some(child);
                        controls::status_text(self.status, 0, &tf("Running {0}", &[&out.executable.as_ref().and_then(|e| e.file_name()).unwrap_or_default().to_string_lossy()]));
                    }
                    Some(Err(e)) => {
                        controls::status_text(self.status, 0, &e.to_string());
                    }
                    None => {
                        controls::status_text(self.status, 0, t("The project has no program to run"));
                    }
                }
            }
        } else {
            controls::status_text(self.status, 0, &tf("Build failed - {0} errors, {1} warnings", &[&errors.max(1), &warnings]));
        }
    }

    /// Double-click in the error list: the file, opened as Windows opens it.
    /// Format Document: the open Rust file as rustfmt writes it, one undo step.
    fn format_document(&mut self) {
        let (Some(h), Some(path)) = (self.code_shown(), self.doc.path.clone()) else {
            controls::status_text(self.status, 0, t("Open a code file first"));
            return;
        };
        if path.extension().is_none_or(|e| e != "rs") {
            controls::status_text(self.status, 0, t("Format Document is for Rust files"));
            return;
        }
        match cargo::rustfmt(&phai_editor::view::text(h), &path) {
            Ok(new) => {
                let said = if phai_editor::view::replace_whole(h, &new) { t("Formatted with rustfmt") } else { t("Already formatted") };
                controls::status_text(self.status, 0, said);
            }
            Err(e) => controls::status_text(self.status, 0, &e),
        }
    }

    /// The code editor, when it is what the window shows.
    fn code_shown(&self) -> Option<HWND> {
        self.doc.code.filter(|h| unsafe { wm::IsWindowVisible(*h) }.as_bool())
    }

    /// F8 and Shift+F8: the next (or last) error or warning in the Error List, opened at its line.
    /// The Error List again from the diagnostics: the kinds its filters show, their counts on them.
    pub fn fill_errors(&mut self) {
        use windows::Win32::UI::Controls as cc;
        let (errs, warns) = self.bottom.as_ref().map(|b| (win::is_checked(b.show_errors), win::is_checked(b.show_warnings))).unwrap_or((true, true));
        unsafe {
            wm::SendMessageW(self.errors, cc::LVM_DELETEALLITEMS, None, None);
        }
        self.diag_rows.clear();
        for (i, d) in self.diagnostics.iter().enumerate() {
            if (d.error && !errs) || (!d.error && !warns) {
                continue;
            }
            let file = d.file.as_ref().and_then(|f| f.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            let line = if d.line > 0 { d.line.to_string() } else { String::new() };
            let first = d.message.lines().next().unwrap_or_default();
            win::list_add(self.errors, &[if d.error { t("Error") } else { t("Warning") }, first, &file, &line]);
            self.diag_rows.push(i);
        }
        let e = self.diagnostics.iter().filter(|d| d.error).count();
        if let Some(b) = self.bottom.as_ref() {
            win::set_text(b.show_errors, &tf("Errors ({0})", &[&e]));
            win::set_text(b.show_warnings, &tf("Warnings ({0})", &[&(self.diagnostics.len() - e)]));
        }
    }

    fn step_diagnostic(&mut self, forward: bool) {
        use windows::Win32::UI::Controls as cc;
        let n = self.diag_rows.len();
        if n == 0 {
            controls::status_text(self.status, 0, t("No errors or warnings"));
            return;
        }
        let at = unsafe { wm::SendMessageW(self.errors, cc::LVM_GETNEXTITEM, Some(WPARAM(usize::MAX)), Some(LPARAM(cc::LVNI_SELECTED as isize))) }.0;
        // Only rows that lead somewhere are stepped to.
        let mut row = if at < 0 { if forward { n - 1 } else { 0 } } else { at as usize };
        for _ in 0..n {
            row = if forward { (row + 1) % n } else { (row + n - 1) % n };
            if self.diag_rows.get(row).and_then(|i| self.diagnostics.get(*i)).is_some_and(|d| d.file.is_some()) {
                break;
            }
        }
        if let Some(b) = self.bottom.as_ref() {
            b.show_tab(0);
        }
        win::list_select(self.errors, row);
        self.open_diagnostic(row);
    }

    fn open_diagnostic(&mut self, row: usize) {
        // A row of the list as filtered: its diagnostic.
        let Some(d) = self.diag_rows.get(row).and_then(|i| self.diagnostics.get(*i)) else { return };
        let Some(file) = d.file.clone() else { return };
        let (line, column) = (d.line as usize, d.column as usize);
        controls::status_text(self.status, 0, &format!("{}:{line}:{column}", file.display()));
        // Into Phai's own editor, at the line.
        self.remember_place();
        self.open_code(&file);
        if let Some(h) = self.doc.code {
            phai_editor::view::go_to(h, line, column);
        }
    }

    // --- commands -----------------------------------------------------------------------------

    fn command(&mut self, id: i32) {
        match id {
            ID_NEW => self.open_in_tab(FormFile::new("MainWindow"), None),
            ID_OPEN => {
                if let Some(p) = file_dialog(self.hwnd, false, "Phai form (*.phaiform)", "*.phaiform", "") {
                    self.open_path(&p);
                }
            }
            ID_VIEW_TOOLBOX => self.show_pane(Pane::Toolbox, !self.toolbox_on),
            ID_VIEW_EXPLORER => {
                let on = !explorer::side_panes(self.side).0;
                self.show_pane(Pane::Explorer, on);
            }
            ID_VIEW_PROPS => {
                let on = !explorer::side_panes(self.side).1;
                self.show_pane(Pane::Properties, on);
            }
            ID_VIEW_OUTPUT => self.show_pane(Pane::Output, !self.bottom_on),
            ID_EDIT_MENUS => self.edit_menus(),
            ID_CUT => {
                self.copy();
                self.delete_selected();
            }
            ID_COPY => self.copy(),
            ID_PASTE => self.paste(),
            ID_DUPLICATE => self.duplicate(),
            ID_SELECT_ALL => {
                let all: Vec<usize> = (0..self.doc.form.control.len()).collect();
                self.select_many(&all);
            }
            f if (ID_FORMAT..ID_FORMAT + FORMATS.len() as i32).contains(&f) => self.format(FORMATS[(f - ID_FORMAT) as usize]),
            ID_EDIT_TOOLBAR => self.edit_toolbar(),
            ID_LSP_TEST => self.lsp_test_step(),
            ID_CHORD => controls::status_text(self.status, 0, t("A chord's first key was pressed. Waiting for the second key...")),
            ID_FORMAT_DOC => self.format_document(),
            ID_FOLD | ID_FOLD_ALL => match self.code_shown() {
                Some(h) if id == ID_FOLD => phai_editor::view::toggle_fold_at_caret(h),
                Some(h) => phai_editor::view::toggle_all_folds(h),
                None => controls::status_text(self.status, 0, t("Open a code file first")),
            },
            ID_FIND_IN_FILES => self.find_in_files(None),
            ID_GO_TO_ALL => self.go_to_all(""),
            ID_ATTACH => self.attach_to_process(),
            ID_EXCEPTIONS => self.exception_settings(),
            ID_MEMORY_WIN => {
                self.show_pane(Pane::Output, true);
                if let Some(b) = self.bottom.as_ref() {
                    b.show_tab(12);
                    let _ = unsafe { km::SetFocus(Some(b.memory_input)) };
                }
            }
            ID_DATA_BP => self.new_data_breakpoint(),
            ID_DISASSEMBLY => {
                self.show_pane(Pane::Output, true);
                if let Some(b) = self.bottom.as_ref() {
                    b.show_tab(13);
                }
            }
            ID_THREADS_WIN => {
                self.show_pane(Pane::Output, true);
                if let Some(b) = self.bottom.as_ref() {
                    b.show_tab(11);
                }
            }
            ID_BREAKPOINTS_WIN => {
                self.show_pane(Pane::Output, true);
                self.fill_breakpoints();
                if let Some(b) = self.bottom.as_ref() {
                    b.show_tab(10);
                }
            }
            ID_DETACH => {
                if let Some(d) = &self.debug {
                    d.send(phai_debug::Command::Detach);
                }
            }
            ID_DIAG_TOOLS => {
                self.show_pane(Pane::Output, true);
                if let Some(b) = self.bottom.as_ref() {
                    b.show_tab(9);
                }
            }
            ID_IMMEDIATE => {
                self.show_pane(Pane::Output, true);
                if let Some(b) = self.bottom.as_ref() {
                    b.show_tab(8);
                    let _ = unsafe { km::SetFocus(Some(b.immediate_input)) };
                }
            }
            ID_CALLS => match self.code_shown() {
                Some(h) => {
                    let Some(path) = self.doc.path.clone() else { return };
                    let pos = phai_editor::view::caret_lsp(h);
                    if let Some(l) = self.lsp_for(&path) {
                        let id = l.prepare_calls(&path, pos.0, pos.1);
                        self.asked.insert(id, (h, Ask::PrepareCalls));
                    }
                }
                None => controls::status_text(self.status, 0, t("Open a code file first")),
            },
            ID_SNIPPET => match self.code_shown() {
                Some(h) => unsafe {
                    // Every snippet, its word and its first line, as a menu under the caret.
                    let m = wm::CreatePopupMenu().unwrap_or_default();
                    for (i, (word, body)) in phai_editor::model::SNIPPETS.iter().enumerate() {
                        let label = wide(&format!("{word}	{}", body.lines().next().unwrap_or_default().replace("$0", "").replace('&', "&&")));
                        let _ = wm::AppendMenuW(m, wm::MF_STRING, i + 1, PCWSTR(label.as_ptr()));
                    }
                    let (x, y) = phai_editor::view::caret_screen(h);
                    let n = wm::TrackPopupMenu(m, wm::TPM_RETURNCMD | wm::TPM_LEFTALIGN | wm::TPM_TOPALIGN, x, y, None, self.hwnd, None).0 as usize;
                    let _ = wm::DestroyMenu(m);
                    if let Some((word, _)) = n.checked_sub(1).and_then(|i| phai_editor::model::SNIPPETS.get(i)) {
                        phai_editor::view::insert_snippet(h, word);
                    }
                },
                None => controls::status_text(self.status, 0, t("Open a code file first")),
            },
            ID_BM_TOGGLE | ID_BM_NEXT | ID_BM_PREV | ID_BM_CLEAR => match self.code_shown() {
                Some(h) => {
                    use phai_editor::view::Bookmark;
                    let what = match id {
                        ID_BM_TOGGLE => Bookmark::Toggle,
                        ID_BM_NEXT => Bookmark::Next,
                        ID_BM_PREV => Bookmark::Previous,
                        _ => Bookmark::Clear,
                    };
                    if !phai_editor::view::bookmark(h, what) {
                        controls::status_text(self.status, 0, t("No bookmarks in this file"));
                    }
                }
                None => controls::status_text(self.status, 0, t("Open a code file first")),
            },
            ID_TASK_LIST => self.task_list(),
            ID_BATCH_BUILD => self.batch_build(),
            ID_PROJECT_PROPS => self.project_properties(),
            ID_ADD_FORM => self.add_form(),
            ID_MANAGE_CRATES => match self.doc.path.as_deref().and_then(cargo::project_of) {
                Some(dir) => {
                    let dir = std::path::absolute(&dir).unwrap_or(dir);
                    if crates::run(self.hwnd, &dir, cargo::find_cargo()) {
                        if let Some(e) = self.explorer.as_mut() {
                            e.show_project_of(self.doc.path.as_deref());
                        }
                    }
                }
                None => {
                    win::message(Some(self.hwnd), t("Save the form in a project first - or make one with New project."), "Phai Studio", wm::MB_ICONINFORMATION);
                }
            },
            ID_NEW_WINDOW => self.new_window(),
            ID_CLOSE_PEEK => {
                // Left for another of this program's windows than the peek's own editor.
                let focus = unsafe { km::GetFocus() };
                if self.peek.as_ref().is_some_and(|p| focus != p.1) {
                    self.close_peek();
                }
            }
            ID_SPLIT => {
                self.toggle_split();
                self.rebuild_menu();
            }
            ID_VIEW_OUTLINE => {
                let on = !explorer::outline_shown(self.side);
                if on && !explorer::side_panes(self.side).0 && !explorer::side_panes(self.side).1 {
                    self.show_pane(Pane::Explorer, true);
                }
                explorer::show_outline(self.side, on);
                self.refresh_outline();
                self.rebuild_menu();
            }
            ID_START_WINDOW => self.start_window(),
            ID_RUN_TESTS => self.start_job(cargo::Job::Test, false),
            ID_GIT_HISTORY => match self.doc.path.clone().and_then(|p| Some((gitpane::repository_of(&p)?, std::path::absolute(&p).ok()?))) {
                Some((repo, file)) => githistory::open(self.hwnd, &repo, &file),
                None => {
                    win::message(Some(self.hwnd), t("This file is not in a Git repository."), t("View History of This File..."), wm::MB_ICONINFORMATION);
                }
            },
            ID_MERGE => self.resolve_conflicts(),
            ID_GIT_BLAME => {
                self.blame_on = !self.blame_on;
                remember_setting("blame", if self.blame_on { "1" } else { "0" });
                self.refresh_blame();
                self.rebuild_menu();
            }
            ID_GIT_CHANGES => {
                let from = self.doc.path.clone().or_else(|| std::env::current_dir().ok());
                match from.as_deref().and_then(gitpane::repository_of) {
                    Some(repo) => gitpane::open(self.hwnd, repo),
                    None => {
                        win::message(Some(self.hwnd), t("This project is not in a Git repository. Run git init in its folder to start one."), t("Git Changes..."), wm::MB_ICONINFORMATION);
                    }
                }
            }
            ID_RUN_ALL_TESTS => self.start_job(cargo::Job::Test, true),
            ID_COVERAGE => self.analyze_coverage(),
            ID_HELP_STUDIO => self.open_help("studio.html"),
            ID_HELP_RUST => self.open_help("rust-01.html"),
            ID_HELP_ERRORS => self.open_help("rust-10.html"),
            ID_ABOUT => {
                let words = tf("Phai Studio {0}\n\nA free, open-source studio for Windows programs in Rust.\nMIT License - Monrak Net\n\n{1}", &[&env!("CARGO_PKG_VERSION"), &env!("CARGO_PKG_REPOSITORY")]);
                win::message(Some(self.hwnd), &words, t("About Phai Studio"), wm::MB_ICONINFORMATION);
            }
            ID_QUICK_ACTIONS => match self.code_shown() {
                Some(h) => self.lsp_actions(h),
                None => controls::status_text(self.status, 0, t("Open a code file first")),
            },
            ID_PARAM_INFO => match self.code_shown() {
                Some(h) => self.lsp_signature(h),
                None => controls::status_text(self.status, 0, t("Open a code file first")),
            },
            ID_CHORD_END => controls::status_text(self.status, 0, t("That key combination is not a command")),
            ID_PEEK => match self.code_shown() {
                Some(h) => self.lsp_peek(h),
                None => controls::status_text(self.status, 0, t("Open a code file first")),
            },
            ID_GO_DEFINITION | ID_RENAME | ID_REFERENCES | ID_COMMENT => match self.doc.code {
                Some(h) if id == ID_GO_DEFINITION => self.lsp_definition(h),
                Some(h) if id == ID_REFERENCES => self.lsp_references(h),
                Some(h) if id == ID_COMMENT => phai_editor::view::toggle_comment(h),
                Some(h) => self.lsp_rename(h),
                None => controls::status_text(self.status, 0, t("Open a code file first")),
            },
            ID_FRONT | ID_BACK => self.restack(id == ID_FRONT),
            ID_LOCK => {
                if let Some(c) = self.canvas.as_mut() {
                    c.locked = !c.locked;
                    let on = c.locked;
                    controls::status_text(self.status, 0, if on { t("Controls are locked") } else { t("Controls are unlocked") });
                }
                self.rebuild_menu();
            }
            ID_TAB_ORDER => {
                if let Some(c) = self.canvas.as_mut() {
                    let on = !c.tab_mode();
                    c.set_tab_mode(on);
                    if on {
                        controls::status_text(self.status, 0, t("Click the controls in the order Tab should visit them; Esc ends"));
                    }
                }
                self.rebuild_menu();
            }
            ID_CLOSE => {
                self.close_tab();
            }
            ID_TABS => {}
            // Menu bar, Toolbar: the form's own, opened to edit (made when it has none), the
            // pointer chosen again.
            ID_TOOL_MENUBAR | ID_TOOL_TOOLBAR => {
                self.tool = None;
                toolbox::choose(self.toolbox, ID_TOOL);
                if self.doc.code.is_none() {
                    if id == ID_TOOL_MENUBAR {
                        self.edit_menus();
                    } else {
                        self.edit_toolbar();
                    }
                }
            }
            t if (ID_TOOL..=ID_TOOL + Kind::ALL.len() as i32).contains(&t) => {
                self.tool = if t == ID_TOOL { None } else { Kind::ALL.get((t - ID_TOOL - 1) as usize).copied() };
            }
            ID_SAVE => {
                self.save(false);
            }
            ID_SAVE_ALL => self.save_all(),
            ID_NAV_BACK | ID_NAV_FORWARD => self.navigate(id == ID_NAV_BACK),
            ID_CONFIG => {
                // The box speaks of many things; only a change of choice matters.
                let release = win::combo_selected(self.config) == 1;
                if release != self.release {
                    self.command(if release { ID_RELEASE } else { ID_DEBUG });
                }
            }
            ID_SAVE_AS => {
                self.save(true);
            }
            ID_EXIT => {
                if self.may_quit() {
                    unsafe {
                        let _ = wm::DestroyWindow(self.hwnd);
                    }
                }
            }
            ID_UNDO | ID_REDO if self.doc.code.is_some() => {
                if let Some(h) = self.doc.code {
                    phai_editor::view::undo_redo(h, id == ID_UNDO);
                }
            }
            ID_UNDO => self.undo_redo(true),
            ID_REDO => self.undo_redo(false),
            ID_DELETE => self.delete_selected(),
            ID_PREVIEW => preview::open(self.hwnd, &self.doc.form),
            ID_LIGHT | ID_DARK | ID_THEME_TOGGLE => {
                win::unwatch_theme(self.hwnd);
                let dark = if id == ID_THEME_TOGGLE { !theme::current().is_dark() } else { id == ID_DARK };
                theme::set(if dark { Theme::dark() } else { Theme::light() });
                self.after_theme();
            }
            ID_THAI | ID_ENGLISH | ID_LANG => {
                words::set_thai(if id == ID_LANG { !words::is_thai() } else { id == ID_THAI });
                self.relabel();
            }
            id if (ID_THEME_FILE..ID_THEME_FILE + THEME_FILES_MAX as i32).contains(&id) => {
                if let Some(name) = self.theme_files().get((id - ID_THEME_FILE) as usize).cloned() {
                    self.edit(|f| f.form.theme = Some(name));
                    self.load_theme();
                    self.refresh_props();
                    self.rebuild_menu();
                }
            }
            ID_THEME_EDIT => {
                if let (Some(p), Some(th)) = (self.doc.path.clone(), self.doc.form.form.theme.clone()) {
                    let file = p.with_file_name("").join(&th);
                    if file.is_file() {
                        self.open_path(&file);
                    }
                }
            }
            ID_THEME_NEW => self.theme_from_standard(),
            ID_THEME_IMPORT => self.theme_import(),
            ID_NEW_PROJECT => self.new_project(),
            ID_BUILD => self.start_build(false),
            ID_RUN_NODEBUG => self.start_build(true),
            ID_RUN => match (&self.debug, self.paused) {
                (Some(d), true) => d.send(phai_debug::Command::Continue),
                (Some(_), false) => {}
                (None, _) => self.start_debugging(),
            },
            ID_STEP_OVER | ID_STEP_INTO | ID_STEP_OUT => match (&self.debug, self.paused) {
                (Some(d), true) => d.send(match id {
                    ID_STEP_OVER => phai_debug::Command::StepOver,
                    ID_STEP_INTO => phai_debug::Command::StepInto,
                    _ => phai_debug::Command::StepOut,
                }),
                (None, _) if id != ID_STEP_OUT => self.start_debugging(),
                _ => {}
            },
            ID_BREAK_ALL => {
                if let (Some(d), false) = (&self.debug, self.paused) {
                    d.send(phai_debug::Command::Pause);
                }
            }
            ID_VIEW_SERVERS => {
                let on = !explorer::servers_shown(self.side);
                if on && !explorer::side_panes(self.side).0 && !explorer::side_panes(self.side).1 {
                    self.show_pane(Pane::Explorer, true);
                }
                explorer::show_servers(self.side, on);
                self.rebuild_menu();
            }
            ID_ADD_CONNECTION => {
                explorer::show_servers(self.side, true);
                let owner = self.hwnd;
                if let Some(sv) = self.servers.as_mut() {
                    sv.add_connection(owner, None);
                }
                self.rebuild_menu();
            }
            ID_NEW_QUERY => {
                let has = self.servers.as_ref().is_some_and(|s| !s.conns.is_empty());
                if has {
                    self.new_query(0, "", "");
                } else {
                    self.command(ID_ADD_CONNECTION);
                }
            }
            ID_EXECUTE => self.execute_query(),
            ID_PUBLISH => self.publish(),
            ID_NEXT_ERROR | ID_PREV_ERROR => self.step_diagnostic(id == ID_NEXT_ERROR),
            ID_MOVE_UP | ID_MOVE_DOWN => match self.code_shown() {
                Some(h) => phai_editor::view::move_lines(h, id == ID_MOVE_UP),
                None => controls::status_text(self.status, 0, t("Open a code file first")),
            },
            ID_DESIGN_TEST => {
                let t = DESIGN_TEST.lock().unwrap_or_else(|e| e.into_inner()).take();
                let owner = self.hwnd;
                if let (Some(t), Some(sv)) = (t, self.servers.as_mut()) {
                    if t == "-" {
                        if let Some(c) = sv.conns.first().cloned() {
                            designer::open(owner, c, "main", None);
                        }
                    } else {
                        sv.design_first(owner, &t);
                    }
                }
            }
            ID_ADD_WATCH => {
                let start = self.doc.code.and_then(phai_editor::view::caret_path).unwrap_or_default();
                if let Some(e) = phai_ui::dialogs::ask(Some(self.hwnd), t("Add Watch"), t("An expression: a name, a path like order.total, or a comparison like x > 5"), &start, false) {
                    if !e.trim().is_empty() {
                        self.watches.push(e.trim().to_string());
                        self.refresh_watch();
                        self.show_pane(Pane::Output, true);
                        if let Some(b) = self.bottom.as_ref() {
                            b.show_tab(6);
                        }
                    }
                }
            }
            ID_BP_CONDITION => self.edit_condition(),
            ID_TOGGLE_BP => {
                if let Some(h) = self.doc.code {
                    phai_editor::view::toggle_breakpoint(h);
                }
            }
            ID_CLEAR_BPS => {
                self.bps.clear();
                for h in self.tabs.iter().filter_map(|t| t.doc.code).chain(self.doc.code) {
                    phai_editor::view::set_breakpoints(h, &[]);
                }
                self.bp_off.clear();
                if let Some(d) = &self.debug {
                    d.send(phai_debug::Command::ClearDataBreakpoints);
                }
                self.send_breakpoints();
                self.mark_breakpoints();
                self.fill_breakpoints();
            }
            ID_BUILD_ALL => self.start_job(cargo::Job::Build, true),
            ID_REBUILD_ALL => self.start_job(cargo::Job::Rebuild, true),
            ID_CLEAN_ALL => self.start_job(cargo::Job::Clean, true),
            ID_ANALYSE_ALL => self.start_job(cargo::Job::Analyse, true),
            ID_REBUILD => self.start_job(cargo::Job::Rebuild, false),
            ID_CLEAN => self.start_job(cargo::Job::Clean, false),
            ID_ANALYSE => self.start_job(cargo::Job::Analyse, false),
            ID_CUSTOM_STD => self.customise_bar(self.toolbar, ID_CUSTOM_STD, toolbar_items()),
            ID_CUSTOM_LAYOUT => self.customise_bar(self.layout_bar, ID_CUSTOM_LAYOUT, layout_items()),
            ID_STOP => match &self.debug {
                Some(d) => d.send(phai_debug::Command::Stop),
                None => self.stop(),
            },
            n if (ID_TEMPLATE..ID_TEMPLATE + phai_design::templates::Template::ALL.len() as i32).contains(&n) => {
                let tpl = phai_design::templates::Template::ALL[(n - ID_TEMPLATE) as usize];
                let name = match tpl {
                    phai_design::templates::Template::Dialog => "AskDialog",
                    phai_design::templates::Template::MainWindow => "MainWindow",
                    phai_design::templates::Template::DataEntry => "DetailsWindow",
                };
                self.open_in_tab(tpl.make(name), None);
            }
            r if (ID_RECENT..ID_RECENT + RECENT_MAX as i32).contains(&r) => {
                if let Some(p) = recent_files().get((r - ID_RECENT) as usize).cloned() {
                    self.open_path(&p);
                }
            }
            ID_DEBUG | ID_RELEASE => {
                self.release = id == ID_RELEASE;
                controls::status_text(self.status, 2, if self.release { "Release" } else { "Debug" });
                unsafe {
                    let old = wm::GetMenu(self.hwnd);
                    let _ = wm::SetMenu(self.hwnd, Some(build_menu()));
                    let _ = wm::DestroyMenu(old);
                }
            }
            ID_THEME_NONE => {
                self.edit(|f| f.form.theme = None);
                self.load_theme();
                self.refresh_props();
                self.rebuild_menu();
            }
            _ => {}
        }
    }

    fn rebuild_menu(&self) {
        unsafe {
            let old = wm::GetMenu(self.hwnd);
            let _ = wm::SetMenu(self.hwnd, Some(build_menu()));
            let _ = wm::DestroyMenu(old);
        }
        self.say_bars();
    }

    fn relabel(&mut self) {
        win::set_toolbar(self.toolbar, &toolbar_items());
        win::set_toolbar(self.layout_bar, &layout_items());
        win::set_toolbar(self.debug_bar, &debug_items());
        win::rebar_fit(self.rebar);
        self.say_bars();
        toolbox::relabel(self.toolbox);
        toolbox::choose(self.toolbox, self.tool.and_then(|k| Kind::ALL.iter().position(|x| *x == k)).map(|i| ID_TOOL + 1 + i as i32).unwrap_or(ID_TOOL));
        unsafe {
            let old = wm::GetMenu(self.hwnd);
            let _ = wm::SetMenu(self.hwnd, Some(build_menu()));
            let _ = wm::DestroyMenu(old);
        }
        if let Some(p) = self.props.as_mut() {
            p.relabel();
        }
        if let Some(e) = self.explorer.as_mut() {
            e.relabel();
        }
        if let Some(b) = self.bottom.as_ref() {
            b.relabel();
        }
        explorer::relabel_side(self.side);
        self.refresh_tabs();
        controls::status_text(self.status, 0, t("Ready"));
        self.say_title();
        self.rebuild();
    }
}

/// Where the theme is for the generated code: beside the form for `include_str!`, and from the
/// crate's `Cargo.toml` for a debug build to watch.
pub fn theme_options(form: &Path, theme_file: Option<&str>) -> codegen::Options {
    let Some(th) = theme_file else { return codegen::Options::default() };
    let dir = form.with_file_name("");
    let watch = dir.ancestors().find(|d| d.join("Cargo.toml").exists()).and_then(|root| dir.join(th).strip_prefix(root).ok().map(|p| p.to_string_lossy().replace('\\', "/")));
    codegen::Options { theme_include: Some(th.replace('\\', "/")), theme_watch: watch }
}

/// Where Phai Studio keeps what it remembers for this person: `%APPDATA%\Phai Studio`.
/// One cell of a list, its words changed.
fn set_list_cell(list: HWND, row: usize, col: usize, text: &str) {
    use windows::Win32::UI::Controls as cc;
    let mut w = wide(text);
    let item = cc::LVITEMW { iSubItem: col as i32, pszText: windows::core::PWSTR(w.as_mut_ptr()), ..Default::default() };
    unsafe {
        wm::SendMessageW(list, 0x1074 /* LVM_SETITEMTEXTW */, Some(WPARAM(row)), Some(LPARAM(&item as *const _ as isize)));
    }
}

pub fn file_dialog_pub(owner: HWND, save: bool, description: &str, pattern: &str, suggested: &str) -> Option<PathBuf> {
    file_dialog(owner, save, description, pattern, suggested)
}

pub fn settings_dir_pub() -> Option<PathBuf> {
    settings_dir()
}

pub fn copy_text(owner: HWND, text: &str) {
    set_clipboard(owner, text);
}

fn settings_dir() -> Option<PathBuf> {
    // `PHAI_SETTINGS_DIR`: settings of a test's own, the person's left alone.
    if let Some(d) = std::env::var_os("PHAI_SETTINGS_DIR") {
        return Some(PathBuf::from(d));
    }
    Some(PathBuf::from(std::env::var_os("APPDATA")?).join("Phai Studio"))
}

/// Bytes as the Memory window shows them: an address, sixteen bytes in hex, and as text.
fn hex_dump(addr: u64, bytes: &[Option<u8>]) -> String {
    let mut out = String::new();
    for (row, chunk) in bytes.chunks(16).enumerate() {
        let hex: Vec<String> = chunk.iter().map(|b| b.map(|b| format!("{b:02X}")).unwrap_or_else(|| "??".into())).collect();
        let text: String = chunk.iter().map(|b| match b {
            Some(c) if (0x20..0x7f).contains(c) => *c as char,
            Some(_) => '.',
            None => '?',
        }).collect();
        out.push_str(&format!("0x{:016X}  {:<47}  {text}\r\n", addr + row as u64 * 16, hex.join(" ")));
    }
    out
}

/// A remembered setting, `name=value` lines in `settings.txt`.
/// A remembered setting, for the other modules.
pub fn setting_pub(name: &str) -> Option<String> {
    setting(name)
}

fn setting(name: &str) -> Option<String> {
    let text = std::fs::read_to_string(settings_dir()?.join("settings.txt")).ok()?;
    text.lines().find_map(|l| l.strip_prefix(&format!("{name}=")).map(str::to_string))
}

fn remember_setting(name: &str, value: &str) {
    let Some(dir) = settings_dir() else { return };
    let path = dir.join("settings.txt");
    let mut lines: Vec<String> = std::fs::read_to_string(&path).unwrap_or_default().lines().filter(|l| !l.starts_with(&format!("{name}="))).map(str::to_string).collect();
    lines.push(format!("{name}={value}"));
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(path, lines.join("\n"));
}

/// The forms opened last, newest first, those still there.
fn recent_files() -> Vec<PathBuf> {
    let Some(dir) = settings_dir() else { return vec![] };
    std::fs::read_to_string(dir.join("recent.txt")).unwrap_or_default().lines().map(PathBuf::from).filter(|p| p.is_file()).take(RECENT_MAX).collect()
}

fn remember_recent(path: &Path) {
    let Some(dir) = settings_dir() else { return };
    let path = path.canonicalize().map(|p| PathBuf::from(p.to_string_lossy().trim_start_matches(r"\\?\"))).unwrap_or_else(|_| path.to_path_buf());
    let mut list = recent_files();
    list.retain(|p| p != &path);
    list.insert(0, path);
    list.truncate(RECENT_MAX);
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join("recent.txt"), list.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("\n"));
}

/// Puts `text` on the clipboard.
fn set_clipboard(owner: HWND, text: &str) {
    use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData};
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    let w: Vec<u16> = text.replace('\n', "\r\n").encode_utf16().chain(Some(0)).collect();
    unsafe {
        if OpenClipboard(Some(owner)).is_err() {
            return;
        }
        let _ = EmptyClipboard();
        if let Ok(mem) = GlobalAlloc(GMEM_MOVEABLE, w.len() * 2) {
            let p = GlobalLock(mem) as *mut u16;
            if !p.is_null() {
                std::ptr::copy_nonoverlapping(w.as_ptr(), p, w.len());
                let _ = GlobalUnlock(mem);
                let _ = SetClipboardData(13 /* CF_UNICODETEXT */, Some(windows::Win32::Foundation::HANDLE(mem.0)));
            }
        }
        let _ = CloseClipboard();
    }
}

/// The clipboard's text, if it holds some.
fn get_clipboard(owner: HWND) -> Option<String> {
    use windows::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
    use windows::Win32::Foundation::HGLOBAL;
    use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
    unsafe {
        OpenClipboard(Some(owner)).ok()?;
        let out = (|| {
            let h = GetClipboardData(13).ok()?;
            let mem = HGLOBAL(h.0);
            let p = GlobalLock(mem) as *const u16;
            if p.is_null() {
                return None;
            }
            let mut n = 0;
            while *p.add(n) != 0 {
                n += 1;
            }
            let s = String::from_utf16_lossy(std::slice::from_raw_parts(p, n));
            let _ = GlobalUnlock(mem);
            Some(s.replace("\r\n", "\n"))
        })();
        let _ = CloseClipboard();
        out
    }
}

/// A page of the Thai guide: `help\` beside the program (installed), or `docs\help\` of the
/// source tree it was built in.
fn help_page(page: &str) -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    std::iter::once(dir.join("help"))
        .chain(exe.ancestors().map(|a| a.join("docs").join("help")))
        .map(|d| d.join(page))
        .find(|p| p.is_file())
}

/// A file or web address, opened by whatever Windows opens it with.
pub fn open_with_windows(what: &str) {
    let w = wide(what);
    unsafe {
        windows::Win32::UI::Shell::ShellExecuteW(None, windows::core::w!("open"), PCWSTR(w.as_ptr()), PCWSTR::null(), PCWSTR::null(), wm::SW_SHOWNORMAL);
    }
}

/// Asks for a file to open or a place to save.
fn file_dialog(owner: HWND, save: bool, description: &str, pattern: &str, suggested: &str) -> Option<PathBuf> {
    use windows::Win32::UI::Controls::Dialogs as dlg;
    let filter: Vec<u16> = [description, pattern, ""].iter().flat_map(|s| s.encode_utf16().chain(Some(0))).collect();
    let mut file = [0u16; 1024];
    for (i, u) in suggested.encode_utf16().take(1000).enumerate() {
        file[i] = u;
    }
    let ext = wide(pattern.trim_start_matches("*."));
    let mut ofn = dlg::OPENFILENAMEW {
        lStructSize: std::mem::size_of::<dlg::OPENFILENAMEW>() as u32,
        hwndOwner: owner,
        lpstrFilter: PCWSTR(filter.as_ptr()),
        lpstrFile: windows::core::PWSTR(file.as_mut_ptr()),
        nMaxFile: file.len() as u32,
        lpstrDefExt: PCWSTR(ext.as_ptr()),
        Flags: if save { dlg::OFN_OVERWRITEPROMPT | dlg::OFN_PATHMUSTEXIST } else { dlg::OFN_FILEMUSTEXIST | dlg::OFN_PATHMUSTEXIST },
        ..Default::default()
    };
    let ok = unsafe { if save { dlg::GetSaveFileNameW(&mut ofn) } else { dlg::GetOpenFileNameW(&mut ofn) } }.as_bool();
    ok.then(|| {
        let n = file.iter().position(|c| *c == 0).unwrap_or(0);
        PathBuf::from(String::from_utf16_lossy(&file[..n]))
    })
}

extern "system" fn proc_(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(s) = studio().filter(|s| s.hwnd == hwnd) else { return win::def(hwnd, msg, w, l) };
    match msg {
        wm::WM_TIMER if w.0 == TIMER_DIAG => {
            s.diag_tick();
            LRESULT(0)
        }
        wm::WM_TIMER if w.0 == TIMER_SLIDE => {
            s.slide_check();
            LRESULT(0)
        }
        wm::WM_SIZE => {
            s.layout.apply(hwnd);
            LRESULT(0)
        }
        phai_ui::controls::WM_SPLIT => {
            s.split(w.0 as i32, l.0 as i32);
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            let id = win::loword(w.0) as i32;
            if id == ID_TABS {
                let i = win::hiword(w.0) as usize;
                if i & phai_ui::controls::TAB_CLOSE != 0 {
                    // A tab's close button: that tab, asking to save first if it has changes.
                    let i = i & !phai_ui::controls::TAB_CLOSE;
                    if i != s.active {
                        s.activate(i);
                    }
                    s.close_tab();
                } else {
                    s.activate(i);
                }
            } else {
                s.command(id);
            }
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            use windows::Win32::UI::Controls as cc;
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.hwndFrom == s.rebar {
                const RBN_HEIGHTCHANGE: u32 = (-831i32) as u32;
                const RBN_CHEVRONPUSHED: u32 = (-841i32) as u32;
                if hdr.code == RBN_HEIGHTCHANGE {
                    s.tray_height_changed();
                    return LRESULT(0);
                }
                if hdr.code == RBN_CHEVRONPUSHED {
                    let c = unsafe { &*(l.0 as *const cc::NMREBARCHEVRON) };
                    let mut info = cc::REBARBANDINFOW { cbSize: std::mem::size_of::<cc::REBARBANDINFOW>() as u32, fMask: cc::RBBIM_CHILD, ..Default::default() };
                    unsafe {
                        wm::SendMessageW(s.rebar, 0x041C /* RB_GETBANDINFOW */, Some(WPARAM(c.uBand as usize)), Some(LPARAM(&mut info as *mut _ as isize)));
                    }
                    s.tray_chevron(info.hwndChild, c.rc);
                    return LRESULT(0);
                }
            }
            if s.bottom.as_ref().is_some_and(|b| b.results == hdr.hwndFrom) && (hdr.code == cc::NM_DBLCLK || hdr.code == cc::NM_RCLICK) {
                let it = unsafe { &*(l.0 as *const cc::NMITEMACTIVATE) };
                let row = (it.iItem >= 0).then_some(it.iItem as usize);
                let col = it.iSubItem.max(0) as usize;
                if hdr.code == cc::NM_DBLCLK {
                    if let Some(r) = row {
                        s.edit_cell(r, col);
                    }
                } else {
                    s.results_menu(row, col);
                }
                return LRESULT(0);
            }
            if s.bottom.as_ref().is_some_and(|b| b.coverage == hdr.hwndFrom) && hdr.code == cc::NM_DBLCLK {
                // A file of the coverage: opened, its lines marked.
                let it = unsafe { &*(l.0 as *const cc::NMITEMACTIVATE) };
                if it.iItem >= 1 {
                    if let Some(f) = s.coverage.get(it.iItem as usize - 1).map(|f| f.path.clone()) {
                        s.open_code(&f);
                        s.mark_coverage();
                    }
                }
                return LRESULT(0);
            }
            if s.bottom.as_ref().is_some_and(|b| b.threads == hdr.hwndFrom) && hdr.code == cc::NM_DBLCLK {
                // Switch to Thread: its call stack and locals shown, the steps taken on it.
                let it = unsafe { &*(l.0 as *const cc::NMITEMACTIVATE) };
                if let (Some(id), Some(d), true) = (s.thread_ids.get(it.iItem.max(0) as usize), &s.debug, s.paused) {
                    d.send(phai_debug::Command::Thread(*id));
                }
                return LRESULT(0);
            }
            if s.bottom.as_ref().is_some_and(|b| b.breakpoints == hdr.hwndFrom) {
                match hdr.code {
                    cc::LVN_ITEMCHANGED => {
                        let it = unsafe { &*(l.0 as *const cc::NMLISTVIEW) };
                        if it.iItem >= 0 && (it.uNewState ^ it.uOldState) & 0xF000 != 0 && it.uOldState & 0xF000 != 0 {
                            s.breakpoint_ticked(it.iItem as usize, (it.uNewState >> 12) == 2);
                        }
                    }
                    cc::LVN_KEYDOWN => {
                        let k = unsafe { &*(l.0 as *const cc::NMLVKEYDOWN) };
                        if k.wVKey == 0x2E {
                            let list = hdr.hwndFrom;
                            let i = unsafe { wm::SendMessageW(list, cc::LVM_GETNEXTITEM, Some(WPARAM(usize::MAX)), Some(LPARAM(2))).0 };
                            if i >= 0 {
                                s.delete_breakpoint_row(i as usize);
                            }
                        }
                    }
                    cc::NM_DBLCLK => {
                        let it = unsafe { &*(l.0 as *const cc::NMITEMACTIVATE) };
                        if let Some((p, line)) = s.bp_rows.get(it.iItem.max(0) as usize).cloned() {
                            if p.is_file() {
                                s.open_code(&p);
                                if let Some(h) = s.doc.code {
                                    phai_editor::view::go_to(h, line as usize, 1);
                                }
                            }
                        }
                    }
                    _ => {}
                }
                return LRESULT(0);
            }
            if s.bottom.as_ref().is_some_and(|b| b.tests == hdr.hwndFrom) && hdr.code == cc::NM_DBLCLK {
                let it = unsafe { &*(l.0 as *const cc::NMITEMACTIVATE) };
                if it.iItem >= 0 {
                    s.open_test(it.iItem as usize);
                }
                return LRESULT(0);
            }
            if s.bottom.as_ref().is_some_and(|b| b.locals == hdr.hwndFrom) && hdr.code == cc::NM_DBLCLK {
                let it = unsafe { &*(l.0 as *const cc::NMITEMACTIVATE) };
                s.edit_local(it.iItem);
                return LRESULT(0);
            }
            if s.bottom.as_ref().is_some_and(|b| b.watch == hdr.hwndFrom) && hdr.code == cc::NM_DBLCLK {
                let it = unsafe { &*(l.0 as *const cc::NMITEMACTIVATE) };
                if let Some(old) = s.watches.get(it.iItem.max(0) as usize).cloned() {
                    if let Some(e) = phai_ui::dialogs::ask(Some(s.hwnd), t("Edit Watch"), t("The expression; nothing takes it away."), &old, false) {
                        let i = it.iItem as usize;
                        if e.trim().is_empty() {
                            s.watches.remove(i);
                        } else {
                            s.watches[i] = e.trim().to_string();
                        }
                        s.refresh_watch();
                    }
                }
                return LRESULT(0);
            }
            if s.bottom.as_ref().is_some_and(|b| b.stack == hdr.hwndFrom) && hdr.code == cc::NM_DBLCLK {
                let it = unsafe { &*(l.0 as *const cc::NMITEMACTIVATE) };
                if let Some(f) = s.frames.get(it.iItem.max(0) as usize).cloned() {
                    if let Some(file) = f.file.filter(|p| p.is_file()) {
                        s.open_code(&file);
                        if let Some(h) = s.doc.code {
                            phai_editor::view::go_to(h, f.line as usize, 1);
                        }
                    }
                }
                return LRESULT(0);
            }
            if s.bottom.as_ref().is_some_and(|b| b.refs == hdr.hwndFrom) && hdr.code == cc::NM_DBLCLK {
                let it = unsafe { &*(l.0 as *const cc::NMITEMACTIVATE) };
                if it.iItem >= 0 {
                    s.open_reference(it.iItem as usize);
                }
                return LRESULT(0);
            }
            if hdr.hwndFrom == s.errors && hdr.code == cc::NM_DBLCLK {
                let it = unsafe { &*(l.0 as *const cc::NMITEMACTIVATE) };
                if it.iItem >= 0 {
                    s.open_diagnostic(it.iItem as usize);
                }
            }
            win::def(hwnd, msg, w, l)
        }
        phai_editor::lsp::WM_LSP => {
            let msg = unsafe { Box::from_raw(l.0 as *mut serde_json::Value) };
            s.lsp_message(*msg);
            LRESULT(0)
        }
        cargo::WM_BUILD_LINE => {
            let line = unsafe { Box::from_raw(l.0 as *mut String) };
            if let Some(b) = s.bottom.as_ref() {
                b.add_line(&line);
            }
            LRESULT(0)
        }
        WM_EVENT_METHOD => {
            s.add_event_method(props::event_key_of(w.0));
            LRESULT(0)
        }
        WM_DEBUG => {
            let e = unsafe { Box::from_raw(l.0 as *mut phai_debug::Event) };
            s.debug_event(*e);
            LRESULT(0)
        }
        cargo::WM_BUILD_DONE => {
            let out = unsafe { Box::from_raw(l.0 as *mut cargo::Outcome) };
            if BUILD_TEST.load(Ordering::Relaxed) && std::env::var_os("PHAI_SHOT").is_some() && out.job == cargo::Job::Test {
                // Photographed: the Tests list shown.
                for r in &out.tests {
                    println!("test {} {} {}", r.name, r.outcome, r.message);
                }
                s.build_done(*out);
                return LRESULT(0);
            }
            if BUILD_TEST.load(Ordering::Relaxed) && std::env::var_os("PHAI_SHOT").is_none() {
                println!("job={:?} ok={} seconds={:.1} exe={:?} no_cargo={} cargo_error={:?}", out.job, out.ok, out.seconds, out.executable, out.no_cargo, out.cargo_error);
                for r in &out.tests {
                    println!("test {} {} {}", r.name, r.outcome, r.message);
                }

                for d in &out.diagnostics {
                    println!("{} {}:{}:{} {}", if d.error { "error" } else { "warning" }, d.file.as_ref().map(|f| f.display().to_string()).unwrap_or_default(), d.line, d.column, d.message);
                }
                // `PHAI_F8=1`: the errors shown, then F8 - where it took the caret.
                if std::env::var_os("PHAI_F8").is_none() {
                    std::process::exit(if out.ok { 0 } else { 2 });
                }
                s.build_done(*out);
                s.command(ID_NEXT_ERROR);
                let file = s.doc.path.as_ref().map(|p| p.display().to_string()).unwrap_or_default();
                println!("f8 {} line {}", file, s.doc.code.map(phai_editor::view::caret_line).unwrap_or(0));
                std::process::exit(0);
            }
            s.build_done(*out);
            LRESULT(0)
        }
        win::WM_THEME_CHANGED => {
            if w.0 == 1 {
                s.after_theme();
            } else if let Some(e) = win::theme_error() {
                controls::status_text(s.status, 0, &e);
                win::message(Some(hwnd), &e, t("Theme file has a mistake"), wm::MB_ICONWARNING);
            }
            LRESULT(0)
        }
        0x02E0 => {
            // WM_DPICHANGED
            let r = unsafe { &*(l.0 as *const RECT) };
            unsafe {
                let _ = wm::SetWindowPos(hwnd, None, r.left, r.top, r.right - r.left, r.bottom - r.top, wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
                let _ = Gdi::DeleteObject(s.font.into());
            }
            s.font = win::font(hwnd, 0, false);
            win::set_font_all(hwnd, s.font);
            win::retheme(hwnd);
            s.layout.apply(hwnd);
            s.rebuild();
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            if s.may_quit() {
                unsafe {
                    let _ = wm::DestroyWindow(hwnd);
                }
            }
            LRESULT(0)
        }
        wm::WM_DESTROY => {
            s.stop();
            remember_setting("editor_font_pt", &phai_editor::view::font_size().to_string());
            unsafe { wm::PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => win::def(hwnd, msg, w, l),
    }
}

/// Shortcuts, before a box can take the key: Ctrl+N/O/S/Z/Y and F5.
fn shortcut(msg: &wm::MSG) -> Option<i32> {
    // Alt+F11: Run Code Analysis on Solution. F10 comes as a system key: Step Over.
    // Alt+F7: the project's properties.
    if msg.message == wm::WM_SYSKEYDOWN && msg.wParam.0 == 0x76 {
        return Some(ID_PROJECT_PROPS);
    }
    if msg.message == wm::WM_SYSKEYDOWN && msg.wParam.0 == 0x7A {
        return Some(ID_ANALYSE_ALL);
    }
    if msg.message == wm::WM_SYSKEYDOWN && msg.wParam.0 == 0x79 {
        return Some(ID_STEP_OVER);
    }
    if msg.message == wm::WM_SYSKEYDOWN && msg.wParam.0 == 0x78 {
        return Some(ID_BP_CONDITION);
    }
    if msg.message == wm::WM_KEYDOWN {
        let shift = unsafe { km::GetKeyState(0x10) } < 0;
        let ctrl = unsafe { km::GetKeyState(0x11) } < 0;
        // Chords: Ctrl+K, then a second key.
        // The chord's first key (K or R), or 0.
        static CHORD: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let k = msg.wParam.0;
        if !(0x10..=0x12).contains(&k) {
            let first = CHORD.swap(0, Ordering::Relaxed);
            if first != 0 {
                return match (first, k, ctrl) {
                    (0x4B, 0x44, true) => Some(ID_FORMAT_DOC), // Ctrl+K, Ctrl+D
                    (0x4B, 0x43, true) => Some(ID_COMMENT),    // Ctrl+K, Ctrl+C
                    (0x4B, 0x55, true) => Some(ID_COMMENT),    // Ctrl+K, Ctrl+U
                    (0x4B, 0x4B, true) => Some(ID_BM_TOGGLE),  // Ctrl+K, Ctrl+K
                    (0x4B, 0x58, true) => Some(ID_SNIPPET),    // Ctrl+K, Ctrl+X
                    (0x4B, 0x54, true) => Some(ID_CALLS),      // Ctrl+K, Ctrl+T
                    (0x4B, 0x4E, true) => Some(ID_BM_NEXT),    // Ctrl+K, Ctrl+N
                    (0x4B, 0x50, true) => Some(ID_BM_PREV),    // Ctrl+K, Ctrl+P
                    (0x4B, 0x4C, true) => Some(ID_BM_CLEAR),   // Ctrl+K, Ctrl+L
                    (0x4D, 0x4D, true) => Some(ID_FOLD),       // Ctrl+M, Ctrl+M
                    (0x4D, 0x4C, true) => Some(ID_FOLD_ALL),   // Ctrl+M, Ctrl+L
                    (0x52, 0x41, _) => Some(ID_RUN_ALL_TESTS), // Ctrl+R, A
                    (0x52, 0x54, _) => Some(ID_RUN_TESTS),     // Ctrl+R, T
                    _ => Some(ID_CHORD_END),
                };
            }
            if (k == 0x4B || k == 0x52 || (k == 0x4D && studio().is_some_and(|s| s.code_shown().is_some()))) && ctrl && !shift {
                CHORD.store(k, Ordering::Relaxed);
                return Some(ID_CHORD);
            }
        }
        match msg.wParam.0 {
            0x70 if !ctrl && !shift => return Some(ID_HELP_STUDIO),
            0x45 if ctrl && shift => return Some(ID_EXECUTE),
            0x46 if ctrl && shift => return Some(ID_FIND_IN_FILES),
            0x53 if ctrl && unsafe { km::GetKeyState(0x12) } < 0 => return Some(ID_VIEW_SERVERS),
            0x47 if ctrl && unsafe { km::GetKeyState(0x12) } < 0 => return Some(ID_GIT_CHANGES),
            0x54 if ctrl && unsafe { km::GetKeyState(0x12) } < 0 => return Some(ID_VIEW_OUTLINE),
            0x49 if ctrl && unsafe { km::GetKeyState(0x12) } < 0 => return Some(ID_IMMEDIATE),
            0x71 if ctrl && unsafe { km::GetKeyState(0x12) } < 0 => return Some(ID_DIAG_TOOLS),
            0x50 if ctrl && unsafe { km::GetKeyState(0x12) } < 0 => return Some(ID_ATTACH),
            0x42 if ctrl && unsafe { km::GetKeyState(0x12) } < 0 => return Some(ID_BREAKPOINTS_WIN),
            0x48 if ctrl && unsafe { km::GetKeyState(0x12) } < 0 => return Some(ID_THREADS_WIN),
            0x45 if ctrl && unsafe { km::GetKeyState(0x12) } < 0 => return Some(ID_EXCEPTIONS),
            0x4D if ctrl && unsafe { km::GetKeyState(0x12) } < 0 => return Some(ID_MEMORY_WIN),
            0x44 if ctrl && unsafe { km::GetKeyState(0x12) } < 0 => return Some(ID_DISASSEMBLY),
            0x77 if shift && !ctrl => return Some(ID_PREV_ERROR),
            0x77 if !ctrl => return Some(ID_NEXT_ERROR),
            0x78 if ctrl && shift => return Some(ID_CLEAR_BPS),
            0x78 if shift => return Some(ID_ADD_WATCH),
            0x78 => return Some(ID_TOGGLE_BP),
            0x7A if shift => return Some(ID_STEP_OUT),
            0x7A => return Some(ID_STEP_INTO),
            0x13 if ctrl => return Some(ID_BREAK_ALL), // Ctrl+Alt+Break comes as Ctrl+Pause
            0x03 => return Some(ID_BREAK_ALL),         // Ctrl+Break
            _ => {}
        }
    }
    if msg.message != wm::WM_KEYDOWN {
        return None;
    }
    let shift = unsafe { km::GetKeyState(0x10) } < 0;
    if unsafe { km::GetKeyState(0x11) } < 0 {
        match msg.wParam.0 {
            0x42 if shift => return Some(ID_BUILD_ALL),
            0x74 => return Some(ID_RUN_NODEBUG),
            _ => {}
        }
    }
    let ctrl = unsafe { km::GetKeyState(0x11) } < 0;
    let in_box = !unsafe { km::GetFocus() }.0.is_null() && win::kind_of(unsafe { km::GetFocus() }) == Some(win::Kind::Edit);
    match (msg.wParam.0 as u8, ctrl) {
        (b'T', true) if !shift => Some(ID_GO_TO_ALL),
        (b'N', true) => Some(ID_NEW),
        (b'O', true) => Some(ID_OPEN),
        (b'W', true) => Some(ID_CLOSE),
        (b'S', true) => Some(ID_SAVE),
        // In a box Ctrl+Z is the box's own undo.
        (b'Z', true) if !in_box => Some(ID_UNDO),
        (b'Y', true) if !in_box => Some(ID_REDO),
        (b'C', true) if !in_box => Some(ID_COPY),
        (b'X', true) if !in_box => Some(ID_CUT),
        (b'V', true) if !in_box => Some(ID_PASTE),
        (b'D', true) if !in_box => Some(ID_DUPLICATE),
        (b'A', true) if !in_box => Some(ID_SELECT_ALL),
        (b'B', true) => Some(ID_BUILD),
        (0x73, _) => Some(ID_PREVIEW),
        (0x74, _) if unsafe { km::GetKeyState(0x10) } < 0 => Some(ID_STOP),
        (0x74, _) => Some(ID_RUN),
        _ => None,
    }
}

/// Debug builds: a crash says what and where, with a backtrace, instead of vanishing.
#[cfg(debug_assertions)]
fn report_crashes() {
    use windows::Win32::System::Diagnostics::Debug::{AddVectoredExceptionHandler, EXCEPTION_POINTERS};
    unsafe extern "system" fn handler(info: *mut EXCEPTION_POINTERS) -> i32 {
        let rec = unsafe { &*(*info).ExceptionRecord };
        let code = rec.ExceptionCode.0 as u32;
        // Access violations, stack overflows and the like; not the exceptions Windows uses itself.
        if matches!(code, 0xC0000005 | 0xC00000FD | 0xC0000409 | 0xC000001D) {
            eprintln!("crash {code:#x} at {:?}
{}", rec.ExceptionAddress, std::backtrace::Backtrace::force_capture());
        }
        0 // EXCEPTION_CONTINUE_SEARCH
    }
    unsafe {
        AddVectoredExceptionHandler(1, Some(handler));
    }
}

/// `--selftest-multi out.phaiform`: chooses several controls with a band, lines them up, makes
/// a copy and saves - checked by what lands in the file.
fn selftest_multi(out: &Path) {
    let Some(s) = studio() else { return };
    let Some(surface) = s.canvas.as_ref().map(|c| c.surface) else { return };
    let at = |x: i32, y: i32| LPARAM(((win::px(surface, y) as isize) << 16) | (win::px(surface, x) as isize & 0xFFFF));
    let send = |msg: u32, l: LPARAM| unsafe {
        wm::SendMessageW(surface, msg, Some(WPARAM(1)), Some(l));
    };
    // A band over the two buttons at the bottom right of the sample form.
    send(wm::WM_LBUTTONDOWN, at(300, 298));
    send(wm::WM_MOUSEMOVE, at(470, 330));
    send(wm::WM_LBUTTONUP, at(470, 330));
    println!("chosen: {:?}", s.selection().iter().map(|i| s.doc.form.control[*i].id.clone()).collect::<Vec<_>>());
    s.format(phai_form::arrange::Op::SameWidth);
    s.duplicate();
    println!("after duplicate: {:?}", s.selection().iter().map(|i| s.doc.form.control[*i].id.clone()).collect::<Vec<_>>());
    s.doc.path = Some(out.to_path_buf());
    s.save(false);
}

/// See `--selftest` in `main`.
fn selftest(out: &Path) {
    let Some(s) = studio() else { return };
    let Some(surface) = s.canvas.as_ref().map(|c| c.surface) else { return };
    let at = |x: i32, y: i32| LPARAM(((win::px(surface, y) as isize) << 16) | (win::px(surface, x) as isize & 0xFFFF));
    let send = |msg: u32, l: LPARAM| unsafe {
        wm::SendMessageW(surface, msg, Some(WPARAM(1)), Some(l));
    };
    // Draw a button from (40, 300) to (140, 324).
    s.tool = Some(Kind::Button);
    send(wm::WM_LBUTTONDOWN, at(40, 300));
    send(wm::WM_MOUSEMOVE, at(100, 312));
    send(wm::WM_MOUSEMOVE, at(140, 324));
    send(wm::WM_LBUTTONUP, at(140, 324));
    // Move it 20 right, 8 down, by grabbing its middle.
    send(wm::WM_LBUTTONDOWN, at(90, 312));
    send(wm::WM_MOUSEMOVE, at(110, 320));
    send(wm::WM_LBUTTONUP, at(110, 320));
    // Widen it from its right-middle handle by 24.
    send(wm::WM_LBUTTONDOWN, at(160, 320));
    send(wm::WM_MOUSEMOVE, at(184, 320));
    send(wm::WM_LBUTTONUP, at(184, 320));
    // Give it words and a handler, as the properties pane would.
    s.apply(Key::Text, "Print");
    s.add_handler_for_selected();
    let path = out.to_path_buf();
    s.doc.path = Some(path);
    s.save(false);
}

fn main() {
    #[cfg(debug_assertions)]
    report_crashes();
    let args: Vec<String> = std::env::args().collect();
    words::add(text::THAI);
    phai_form::live::set_translate(false);
    if let Some(pt) = setting("editor_font_pt").and_then(|v| v.parse().ok()) {
        phai_editor::view::set_font_size(pt);
    }
    words::set_thai(args.iter().any(|a| a == "--thai"));
    if args.iter().any(|a| a == "--dark") {
        theme::set(Theme::dark());
    }
    win::init();
    // Phai Studio's own icon (phai-studio.ico, embedded by build.rs): </> grown from bamboo.
    let icon = phai_ui::icon::app_icon(32).or_else(|| phai_ui::icon::glyph_icon(glyph::APPS, 32, theme::Color::rgb(0x12, 0x85, 0x5f)));
    let class = win::register("PhaiStudio", Some(proc_), icon);
    let title = wide("Phai Studio");
    unsafe {
        let hwnd = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_OVERLAPPEDWINDOW | wm::WS_CLIPCHILDREN, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, 100, 100, None, Some(build_menu()), Some(win::instance()), None).unwrap_or_default();
        phai_ui::icon::set_app_icon(hwnd);
        let font = win::font(hwnd, 0, false);
        // The bars lie in a tray, as Visual Studio's: each has a grip to drag it along its row,
        // to another row, or up beside the other.
        let rebar = win::rebar(hwnd, ID_REBAR);
        let toolbar = win::toolbar(rebar, ID_TOOLBAR, &toolbar_items());
        let layout_bar = win::toolbar(rebar, ID_LAYOUT_BAR, &layout_items());
        win::rebar_add(rebar, toolbar, BAND_STANDARD, false);
        win::rebar_add(rebar, layout_bar, BAND_LAYOUT, true);
        // The Debug bar, as Visual Studio's: it shows while a program is being debugged.
        let debug_bar = win::toolbar(rebar, ID_DEBUG_BAR, &debug_items());
        win::rebar_add(rebar, debug_bar, BAND_DEBUG, false);
        wm::SendMessageW(rebar, 0x0423 /* RB_SHOWBAND */, Some(WPARAM(2)), Some(LPARAM(0)));
        // The Debug / Release box lies over the room the Standard bar keeps for it.
        let config = win::combo(toolbar, ID_CONFIG, font, &["Debug", "Release"]);
        let toolbox = toolbox::new(hwnd, ID_TOOLBOX, font);
        let tabbar = controls::tabs(hwnd, ID_TABS, font, &[t("Untitled")]);
        controls::tabs_closable(tabbar);
        let status = controls::status_bar(hwnd, ID_STATUS, font);
        let bottom = bottom::Bottom::new(hwnd, font, ID_ERRORS);
        let errors = bottom.errors;
        let bottom_hwnd = bottom.hwnd;
        let bottom_split = controls::splitter(hwnd, ID_SPLIT_BOTTOM, false);
        win::show(bottom_split, false);
        controls::status_parts(status, &[260, 80]);
        controls::status_text(status, 2, "Debug");
        controls::status_text(status, 0, t("Ready"));
        let dpi = win::dpi(hwnd);
        let status_h = phai_ui::layout::unscale(controls::status_height(status), dpi);
        let mut layout = Layout::new((1100, 700));
        layout
            .add(rebar, Item { bounds: Rect::new(0, 0, 0, phai_ui::layout::unscale(win::rebar_height(rebar), win::dpi(hwnd)).max(win::TOOLBAR_HEIGHT)), dock: Dock::Top, anchor: Anchor::default() })
            .add(status, Item { bounds: Rect::new(0, 0, 0, status_h), dock: Dock::Bottom, anchor: Anchor::default() })
            .add(bottom_hwnd, Item { bounds: Rect::new(0, 0, 0, 0), dock: Dock::Bottom, anchor: Anchor::default() })
            .add(bottom_split, Item { bounds: Rect::new(0, 0, 0, 0), dock: Dock::Bottom, anchor: Anchor::default() })
            .add(toolbox, Item { bounds: Rect::new(0, 0, toolbox::WIDTH, 0), dock: Dock::Left, anchor: Anchor::default() });
        // Every docked pane has its edge to drag, as in Visual Studio.
        let toolbox_split = controls::splitter(hwnd, ID_SPLIT_TOOLBOX, true);
        layout.add(toolbox_split, Item { bounds: Rect::new(0, 0, 6, 0), dock: Dock::Left, anchor: Anchor::default() });
        let studio_box = Box::new(Studio {
            hwnd,
            font,
            toolbar,
            toolbox,
            status,
            errors,
            bottom: Some(bottom),
            release: false,
            diagnostics: vec![],
            diag_rows: vec![],
            lens_round: 0,
            sampler: None,
            lenses: vec![],
            references: vec![],
            building: false,
            running: None,
            lsp: None,
            asked: std::collections::HashMap::new(),
            canvas: None,
            side: HWND::default(),
            side_split: HWND::default(),
            toolbox_split,
            toolbox_dock: Dock::Left,
            toolbox_float: None,
            toolbox_h: 200,
            layout_bar,
            rebar,
            debug_bar,
            debug: None,
            paused: false,
            debug_after_build: false,
            bps: std::collections::HashMap::new(),
            bp_off: vec![],
            bp_rows: vec![],
            bp_filling: false,
            thread_ids: vec![],
            blame_on: setting("blame").as_deref() == Some("1"),
            blame: vec![],
            memory_at: String::new(),
            coverage: vec![],
            lsp_loading: false,
            held_diagnostics: Default::default(),
            frames: vec![],
            servers: None,
            watches: vec![],
            conds: vec![],
            tip: None,
            queries: std::collections::HashMap::new(),
            table_view: None,
            publish_after_build: None,
            publish_include: vec![],
            found_words: "{0} references in {1} files",
            test_results: vec![],
            outline: None,
            split_view: None,
            split_bar: None,
            split_share: 500,
            windows: vec![],
            peek: None,
            config,
            nav: vec![],
            nav_at: 0,
            auto_left: false,
            auto_right: false,
            right_saved: (true, true),
            strip_left: HWND::default(),
            strip_right: HWND::default(),
            sliding: None,
            toolbox_w: toolbox::WIDTH,
            bottom_split: HWND::default(),
            side_w: 280,
            side_dock: Dock::Right,
            side_h: 260,
            side_float: None,
            bottom_h: ERRORS_HEIGHT,
            toolbox_on: true,
            bottom_on: false,
            props: None,
            explorer: None,
            tabbar,
            tabs: vec![Tab::empty()],
            active: 0,
            layout,
            doc: Doc::new(FormFile::new("MainWindow"), None),
            undo: vec![],
            redo: vec![],
            selected: None,
            also: vec![],
            tool: None,
        });
        STUDIO.store(Box::into_raw(studio_box), Ordering::Relaxed);
        let s = studio().unwrap();
        // The right side: the project's files above, the chosen thing's properties below.
        let side = explorer::side_panel(hwnd);
        let explorer = explorer::Explorer::new(side, font);
        let props = Props::new(side, ID_PROPS, font);
        let servers = servers::Servers::new(side, font);
        let outline = outline::Outline::new(side, font);
        explorer::set_side_parts(side, &explorer, props.hwnd, servers.tree, outline.tree, font);
        s.outline = Some(outline);
        s.servers = Some(servers);
        let canvas = Canvas::new(hwnd, ID_CANVAS);
        let side_split = controls::splitter(hwnd, ID_SPLIT_SIDE, true);
        s.side = side;
        s.side_split = side_split;
        s.bottom_split = bottom_split;
        s.layout.add(side, Item { bounds: Rect::new(0, 0, 280, 0), dock: Dock::Right, anchor: Anchor::default() });
        s.layout.add(side_split, Item { bounds: Rect::new(0, 0, 6, 0), dock: Dock::Right, anchor: Anchor::default() });
        // Auto Hide's strips, empty until a pane's pin is pulled.
        let strip_left = autohide::new(hwnd, true, font);
        let strip_right = autohide::new(hwnd, false, font);
        s.strip_left = strip_left;
        s.strip_right = strip_right;
        s.layout.add(strip_left, Item { bounds: Rect::new(0, 0, 0, 0), dock: Dock::Left, anchor: Anchor::default() });
        s.layout.add(strip_right, Item { bounds: Rect::new(0, 0, 0, 0), dock: Dock::Right, anchor: Anchor::default() });
        s.layout.add(tabbar, Item { bounds: Rect::new(0, 0, 0, 30), dock: Dock::Top, anchor: Anchor::default() });
        s.layout.add(canvas.hwnd, Item { bounds: Rect::default(), dock: Dock::Fill, anchor: Anchor::default() });
        s.props = Some(props);
        s.explorer = Some(explorer);
        s.canvas = Some(canvas);
        s.restore_hidden();
        win::theme_frame(hwnd);
        let mut r = RECT { left: 0, top: 0, right: win::px(hwnd, 1100), bottom: win::px(hwnd, 700) };
        let st = wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32;
        let _ = windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(st), true, Default::default(), dpi as u32);
        let _ = wm::SetWindowPos(hwnd, None, 0, 0, r.right - r.left, r.bottom - r.top, wm::SWP_NOMOVE | wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
        // Every form named on the command line, each in its tab; the first one shown. A value
        // after a `--flag` that takes one is not a form.
        let forms: Vec<&String> = args.iter().enumerate().skip(1).filter(|(i, a)| !a.starts_with("--") && !["--selftest", "--selftest-multi", "--tab", "--command", "--select", "--find", "--slide", "--dock-toolbox", "--jobtest", "--show-rows", "--design", "--guide-shot", "--dock-side", "--goto", "--find-in-files", "--box", "--go-to-all", "--new-project"].contains(&args[i - 1].as_str())).map(|(_, a)| a).collect();
        if forms.is_empty() {
            s.set_doc(FormFile::new("MainWindow"), None);
            // Opened with nothing: the start window, once the window is up - unless switched off,
            // or a test runs (`--start` asks for it anyway).
            let testing = args.iter().skip(1).any(|a| a.starts_with("--") && a != "--start") || std::env::var_os("PHAI_SHOT").is_some();
            if (args.iter().any(|a| a == "--start") || !testing) && setting("start_window").is_none_or(|v| v != "0") {
                let _ = wm::PostMessageW(Some(hwnd), wm::WM_COMMAND, WPARAM(ID_START_WINDOW as usize), LPARAM(0));
            }
        }
        for p in &forms {
            if p.ends_with(".phaiform") {
                s.open_path(Path::new(p));
            } else {
                s.open_code(Path::new(p));
            }
        }
        if forms.len() > 1 {
            s.activate(0);
        }
        // `--command N` (more than once) does what menu command N does - for checking by picture.
        for (i, a) in args.iter().enumerate() {
            if a == "--command" {
                if let Some(id) = args.get(i + 1).and_then(|n| n.parse::<usize>().ok()) {
                    let _ = wm::PostMessageW(Some(hwnd), wm::WM_COMMAND, WPARAM(id), LPARAM(0));
                }
            }
        }
        // `--select N` chooses control N; `--events` shows the Events view - for checking by picture.
        // `--select 2` or `--select 2,3`: those controls chosen.
        if let Some(list) = args.iter().position(|a| a == "--select").and_then(|i| args.get(i + 1)) {
            let many: Vec<usize> = list.split(',').filter_map(|n| n.trim().parse().ok()).collect();
            match many.as_slice() {
                [n] => s.select(Some(*n)),
                [] => {}
                _ => s.select_many(&many),
            }
        }
        if args.iter().any(|a| a == "--events") {
            if let Some(p) = s.props.as_mut() {
                p.set_mode(1);
            }
        }
        // `--find WORD` opens the code editor's find bar on it (with replace when `--replace`).
        if let (Some(q), Some(h)) = (args.iter().position(|a| a == "--find").and_then(|i| args.get(i + 1)), s.doc.code) {
            phai_editor::view::find_text(h, q, args.iter().any(|a| a == "--replace"));
        }
        // `--menus` opens the menu editor at once - for checking it by its picture.
        if args.iter().any(|a| a == "--menus") {
            let _ = wm::PostMessageW(Some(hwnd), wm::WM_COMMAND, WPARAM(ID_EDIT_MENUS as usize), LPARAM(0));
        }
        // `--tab N` shows tab N, the way a click on it does - for checking the switch.
        if let Some(n) = args.iter().position(|a| a == "--tab").and_then(|i| args.get(i + 1)).and_then(|n| n.parse::<usize>().ok()) {
            wm::SendMessageW(hwnd, wm::WM_COMMAND, Some(WPARAM((n << 16) | ID_TABS as usize)), Some(LPARAM(0)));
        }
        win::show_top(hwnd);
        s.layout.apply(hwnd);
        s.rebuild();
        s.refresh_props();
        // `--selftest out.phaiform`: draws, moves and sizes a control with the mouse messages a
        // `--one-row`: the Layout bar dragged up beside the Standard bar - checked by picture.
        if args.iter().any(|a| a == "--one-row") {
            use windows::Win32::UI::Controls as cc;
            let mut info = cc::REBARBANDINFOW { cbSize: std::mem::size_of::<cc::REBARBANDINFOW>() as u32, fMask: cc::RBBIM_STYLE, ..Default::default() };
            wm::SendMessageW(s.rebar, 0x041C /* RB_GETBANDINFOW */, Some(WPARAM(1)), Some(LPARAM(&mut info as *mut _ as isize)));
            info.fStyle &= !0x0001; // RBBS_BREAK
            wm::SendMessageW(s.rebar, 0x040B /* RB_SETBANDINFOW */, Some(WPARAM(1)), Some(LPARAM(&info as *const _ as isize)));
            s.tray_height_changed();
        }
        // `--dock-toolbox top|right|bottom|float`: the toolbox moved as a drag would - checked by picture.
        // `--go-to-all WORDS`: Go to All with that typed - what it found printed, the first opened.
        if let Some(q) = args.iter().position(|a| a == "--go-to-all").and_then(|i| args.get(i + 1)).cloned() {
            std::env::set_var("PHAI_GOTO_TEST", "1");
            s.go_to_all(&q);
            if let Some(h) = s.doc.code {
                println!("opened {} at line {}", s.doc.path.as_ref().and_then(|p| p.file_name()).unwrap_or_default().to_string_lossy(), phai_editor::view::caret_line(h));
            }
        }
        // `--find-in-files WORDS`: Find in Files without the question; prints how many it found.
        if let Some(q) = args.iter().position(|a| a == "--find-in-files").and_then(|i| args.get(i + 1)).cloned() {
            s.find_in_files(Some(q));
            println!("{} found", s.references.len());
        }
        // `--task-list`: View > Task List; prints how many it found.
        if args.iter().any(|a| a == "--task-list") {
            s.task_list();
            println!("{} tasks", s.references.len());
        }
        // `--smart-tag`: the chosen control's smart tag, as a click on it does (`PHAI_TAG=n` picks a task).
        if args.iter().any(|a| a == "--smart-tag") {
            s.smart_tag();
            if let Some(i) = s.selected {
                let c = &s.doc.form.control[i];
                println!("now: dock={:?} locked={:?}", c.dock, c.locked);
            }
        }
        // `--new-window`: the code file in front also in a window of its own; words typed there
        // are printed back from the tab's editor.
        if args.iter().any(|a| a == "--new-window") {
            s.command(ID_NEW_WINDOW);
            if let (Some(&(_, e)), Some(code)) = (s.windows.first(), s.doc.code) {
                phai_editor::view::go_to(e, 1, 1);
                for u in "// from the window\n".encode_utf16() {
                    let u = if u == 10 { 13 } else { u };
                    wm::SendMessageW(e, wm::WM_CHAR, Some(WPARAM(u as usize)), Some(LPARAM(1)));
                }
                println!("tab starts: {}", phai_editor::view::text(code).lines().next().unwrap_or_default());
            }
        }
        // `--split`: the code file in front split in two - checked by picture; with `--split-type`
        // words typed in the lower half, to see them in the upper.
        if args.iter().any(|a| a == "--split") {
            s.command(ID_SPLIT);

            if let (Some(second), Some(code), false) = (s.split_view, s.doc.code, std::env::var_os("PHAI_SPLIT_QUIET").is_some()) {
                phai_editor::view::go_to(second, 1, 1);
                for u in "// typed below\n".encode_utf16() {
                    let u = if u == 10 { 13 } else { u };
                    wm::SendMessageW(second, wm::WM_CHAR, Some(WPARAM(u as usize)), Some(LPARAM(1)));
                }
                println!("top starts: {}", phai_editor::view::text(code).lines().next().unwrap_or_default());
                if let Ok(p) = std::env::var("PHAI_SPLIT_SHOT") {
                    let _ = Gdi::RedrawWindow(Some(second), None, None, Gdi::RDW_INVALIDATE | Gdi::RDW_UPDATENOW);
                    let _ = phai_ui::shot::save(second, Path::new(&p));
                }
            }
        }
        // `--outline`: Document Outline shown - checked by picture.
        if args.iter().any(|a| a == "--outline") {
            s.command(ID_VIEW_OUTLINE);
        }
        // `--box L:C-L:C`: a box selection in the open code, then words typed into it - checked by picture.
        if let Some(b) = args.iter().position(|a| a == "--box").and_then(|i| args.get(i + 1)) {
            let parse = |p: &str| p.split_once(':').and_then(|(l, c)| Some((l.parse().ok()?, c.parse().ok()?)));
            if let (Some(h), Some((a, z))) = (s.doc.code, b.split_once('-')) {
                if let (Some(a), Some(z)) = (parse(a), parse(z)) {
                    phai_editor::view::box_select(h, a, z);
                    for u in "new_".encode_utf16() {
                        wm::SendMessageW(h, wm::WM_CHAR, Some(WPARAM(u as usize)), Some(LPARAM(1)));
                    }
                    println!("{} carets", phai_editor::view::caret_count(h));
                }
            }
        }
        // `--bookmarks`: lines 2 and 5 marked, then Next from line 1 twice - where it went printed.
        if args.iter().any(|a| a == "--bookmarks") {
            if let Some(h) = s.doc.code {
                for l in [2, 5] {
                    phai_editor::view::go_to(h, l, 1);
                    s.command(ID_BM_TOGGLE);
                }
                phai_editor::view::go_to(h, 1, 1);
                s.command(ID_BM_NEXT);
                let first = phai_editor::view::caret_line(h);
                s.command(ID_BM_NEXT);
                println!("bookmarks {:?}, next {first} then {}", phai_editor::view::bookmarks(h), phai_editor::view::caret_line(h));
            }
        }
        // `--fold-all`: every block in the open code folded - checked by picture.
        if args.iter().any(|a| a == "--fold-all") {
            if let Some(h) = s.doc.code {
                phai_editor::view::toggle_all_folds(h);
                println!("{} lines folded", phai_editor::view::hidden_lines(h));
            }
        }
        // `--coverage-test`: Test > Analyze Code Coverage, its findings printed.
        if args.iter().any(|a| a == "--coverage-test") {
            std::env::set_var("PHAI_COVERAGE_TEST", "1");
            s.command(ID_COVERAGE);
            if std::env::var_os("PHAI_SHOT").is_none() {
                std::process::exit(0);
            }
        }
        // `--help-test`: where Help > Phai Studio Guide would open, printed.
        if args.iter().any(|a| a == "--help-test") {
            std::env::set_var("PHAI_HELP_TEST", "1");
            s.command(ID_HELP_STUDIO);
            std::process::exit(0);
        }
        // `--merge-test`: Git > Resolve Merge Conflicts, settled as `PHAI_MERGE_TAKE` says.
        if args.iter().any(|a| a == "--merge-test") {
            s.command(ID_MERGE);
            std::process::exit(0);
        }
        // `--history-test`: Git > View History of the file in front, printed; `--blame-test`: its
        // blame, the caret's line shown.
        if args.iter().any(|a| a == "--history-test") {
            std::env::set_var("PHAI_HISTORY_TEST", "1");
            s.command(ID_GIT_HISTORY);
            std::process::exit(0);
        }
        if args.iter().any(|a| a == "--blame-test") {
            if !s.blame_on {
                s.command(ID_GIT_BLAME);
            }
            if let (Some(h), Some(l)) = (s.doc.code, std::env::var("PHAI_BLAME_LINE").ok().and_then(|l| l.parse().ok())) {
                phai_editor::view::go_to(h, l, 1);
                s.show_lenses();
            }
            println!("blamed lines: {}", s.blame.len());
            for (i, b) in s.blame.iter().enumerate().take(3) {
                println!("line {}: {}", i + 1, b.split(',').nth(1).unwrap_or(b).trim());
            }
            if std::env::var_os("PHAI_SHOT").is_none() {
                std::process::exit(0);
            }
        }
        // `--exceptions-test`: Exception Settings set from `PHAI_EXCEPTIONS_TEST`, then read back.
        // `--menu-test`: what the menus hold as Windows has them - the Format menu's icons, the
        // Theme menu's style sheets - and a double-click on the form going to its code; printed.
        if args.iter().any(|a| a == "--menu-test") {
            let bar = wm::GetMenu(hwnd);
            let (mut icons, mut sheets, mut lock_menu) = (0, Vec::new(), false);
            for i in 0..wm::GetMenuItemCount(Some(bar)).max(0) {
                let m = wm::GetSubMenu(bar, i);
                let ids: Vec<u32> = (0..wm::GetMenuItemCount(Some(m)).max(0)).map(|k| wm::GetMenuItemID(m, k)).collect();
                if ids.contains(&(ID_LOCK as u32)) {
                    lock_menu = true;
                    for k in 0..ids.len() {
                        let mut info = wm::MENUITEMINFOW { cbSize: std::mem::size_of::<wm::MENUITEMINFOW>() as u32, fMask: wm::MIIM_BITMAP, ..Default::default() };
                        if wm::GetMenuItemInfoW(m, k as u32, true, &mut info).is_ok() && !info.hbmpItem.is_invalid() {
                            icons += 1;
                        }
                    }
                }
                if ids.contains(&(ID_THEME_NONE as u32)) {
                    for (k, id) in ids.iter().enumerate() {
                        if (ID_THEME_FILE as u32..ID_THEME_FILE as u32 + THEME_FILES_MAX as u32).contains(id) {
                            let mut buf = [0u16; 260];
                            let n = wm::GetMenuStringW(m, k as u32, Some(&mut buf), wm::MF_BYPOSITION);
                            let checked = wm::GetMenuState(m, *id, wm::MF_BYCOMMAND) & wm::MF_CHECKED.0 != 0;
                            sheets.push(format!("{}{}", String::from_utf16_lossy(&buf[..n.max(0) as usize]), if checked { " (ticked)" } else { "" }));
                        }
                    }
                }
            }
            println!("format menu found: {lock_menu}, items with icons: {icons}");
            println!("theme menu sheets: {sheets:?}");
            s.select(None);
            s.add_handler_for_selected();
            let at = s.doc.code.map(phai_editor::view::caret_line);
            println!("double-click on the form: code open {}, at line {:?}", s.doc.code.is_some(), at);
            std::process::exit(0);
        }
        if args.iter().any(|a| a == "--exceptions-test") {
            s.command(ID_EXCEPTIONS);
            println!("remembered: {:?}", setting("exceptions"));
            println!("given to the debugger: {:?}", Studio::exceptions());
            std::process::exit(0);
        }
        // `--bpwin-test`: breakpoints on lines 4 and 5 (5 with a condition), the Breakpoints window
        // shown, the first unticked there as a click would; what the debugger is given printed.
        if args.iter().any(|a| a == "--bpwin-test") {
            if let (Some(h), Some(path)) = (s.doc.code, s.doc.path.clone()) {
                phai_editor::view::set_breakpoints(h, &[4, 5]);
                s.conds.push((path, 5, phai_debug::join_condition("count > 3", phai_debug::Hits::parse(">= 2").ok().flatten())));
                s.show_conditions();
                s.command(ID_BREAKPOINTS_WIN);
                if let Some(list) = s.bottom.as_ref().map(|b| b.breakpoints) {
                    use windows::Win32::UI::Controls as cc;
                    let item = cc::LVITEMW { stateMask: cc::LIST_VIEW_ITEM_STATE_FLAGS(0xF000), state: cc::LIST_VIEW_ITEM_STATE_FLAGS(1 << 12), ..Default::default() };
                    wm::SendMessageW(list, cc::LVM_SETITEMSTATE, Some(WPARAM(0)), Some(LPARAM(&item as *const _ as isize)));
                }
                println!("rows: {:?}", s.bp_rows.iter().map(|(_, l)| *l).collect::<Vec<_>>());
                println!("given to the debugger: {:?}", s.all_breakpoints().iter().map(|(_, l)| *l).collect::<Vec<_>>());
                if std::env::var_os("PHAI_SHOT").is_none() {
                    std::process::exit(0);
                }
            }
        }
        // `--diag-test`: Diagnostic Tools watching this very program - printed after a few samples.
        if args.iter().any(|a| a == "--diag-test") {
            std::env::set_var("PHAI_DIAG_TEST", "1");
            s.watch_process(std::process::id());
            // A few samples now, some work between them, so a photograph shows lines.
            for i in 0..12u64 {
                let mut x = 0u64;
                for j in 0..(i % 4) * 3_000_000 {
                    x = x.wrapping_add(j * j);
                }
                std::hint::black_box(x);
                std::thread::sleep(std::time::Duration::from_millis(60));
                if let Some(d) = s.sampler.as_mut() {
                    d.sample();
                }
            }
            s.command(ID_DIAG_TOOLS);
        }
        // `--new-project DIR`: a new project made there, as File > New project does; its path printed.
        if let Some(d) = args.iter().position(|a| a == "--new-project").and_then(|i| args.get(i + 1)) {
            match project::create(Path::new(d)) {
                Ok(form) => println!("project {} with phai-ui at {}", form.display(), project::phai_ui_path().display()),
                Err(e) => println!("new project failed: {e}"),
            }
            std::process::exit(0);
        }
        // `--goto LINE:COLUMN`: the caret there in the open code - for checking by picture.
        if let Some((l, c)) = args.iter().position(|a| a == "--goto").and_then(|i| args.get(i + 1)).and_then(|p| p.split_once(':')) {
            if let (Some(h), Ok(l), Ok(c)) = (s.doc.code, l.parse(), c.parse()) {
                phai_editor::view::go_to(h, l, c);
            }
        }
        // Where the side panes and the toolbox were last time.
        if !args.iter().any(|a| a == "--dock-side" || a == "--dock-toolbox") && std::env::var_os("PHAI_SHOT").is_none() {
            match setting("side_dock").as_deref() {
                Some("left") => s.dock_side(docking::Target::Edge(Dock::Left)),
                Some("top") => s.dock_side(docking::Target::Edge(Dock::Top)),
                Some("bottom") => s.dock_side(docking::Target::Edge(Dock::Bottom)),
                _ => {}
            }
            match setting("toolbox_dock").as_deref() {
                Some("right") => s.dock_toolbox(docking::Target::Edge(Dock::Right)),
                Some("top") => s.dock_toolbox(docking::Target::Edge(Dock::Top)),
                Some("bottom") => s.dock_toolbox(docking::Target::Edge(Dock::Bottom)),
                _ => {}
            }
        }
        // `--dock-side left|right`: the side panes against that edge, as a drag of their caption puts them.
        if let Some(d) = args.iter().position(|a| a == "--dock-side").and_then(|i| args.get(i + 1)) {
            let t = match d.as_str() {
                "left" => docking::Target::Edge(Dock::Left),
                "top" => docking::Target::Edge(Dock::Top),
                "bottom" => docking::Target::Edge(Dock::Bottom),
                "float" => docking::Target::Float(RECT { left: win::px(hwnd, 700), top: win::px(hwnd, 200), right: win::px(hwnd, 1000), bottom: win::px(hwnd, 700) }),
                _ => docking::Target::Edge(Dock::Right),
            };
            s.dock_side(t);
        }
        if let Some(d) = args.iter().position(|a| a == "--dock-toolbox").and_then(|i| args.get(i + 1)) {
            let t = match d.as_str() {
                "top" => docking::Target::Edge(Dock::Top),
                "right" => docking::Target::Edge(Dock::Right),
                "bottom" => docking::Target::Edge(Dock::Bottom),
                "float" => docking::Target::Float(RECT { left: win::px(hwnd, 300), top: win::px(hwnd, 200), right: win::px(hwnd, 520), bottom: win::px(hwnd, 640) }),
                _ => docking::Target::Edge(Dock::Left),
            };
            s.dock_toolbox(t);
        }
        // `--autohide`: the toolbox and the right-hand panes pinned away to their edges; `--slide N`
        // then slides one out (0 the toolbox, 1 the project, 2 the properties) - checked by picture.
        if args.iter().any(|a| a == "--autohide") {
            s.set_auto_hide(Pane::Toolbox, true);
            s.set_auto_hide(Pane::Properties, true);
            if let Some(n) = args.iter().position(|a| a == "--slide").and_then(|i| args.get(i + 1)).and_then(|n| n.parse::<usize>().ok()) {
                s.slide_out([Pane::Toolbox, Pane::Explorer, Pane::Properties][n.min(2)]);
            }
        }
        // `--snaptest`: drags the sample's Save button a little off its row and leaves it held,
        // so a picture shows the snapline it is drawn to.
        if args.iter().any(|a| a == "--snaptest") {
            if let Some(surface) = s.canvas.as_ref().map(|c| c.surface) {
                let at = |x: i32, y: i32| LPARAM(((win::px(surface, y) as isize) << 16) | (win::px(surface, x) as isize & 0xFFFF));
                wm::SendMessageW(surface, wm::WM_LBUTTONDOWN, Some(WPARAM(1)), Some(at(340, 312)));
                wm::SendMessageW(surface, wm::WM_MOUSEMOVE, Some(WPARAM(1)), Some(at(300, 290)));
                wm::SendMessageW(surface, wm::WM_MOUSEMOVE, Some(WPARAM(1)), Some(at(260, 315)));
            }
        }
        // person's hand would send, then saves - the designer checked end to end.
        if let Some(i) = args.iter().position(|a| a == "--selftest") {
            selftest(Path::new(args.get(i + 1).map(String::as_str).unwrap_or("selftest.phaiform")));
        }
        if let Some(i) = args.iter().position(|a| a == "--selftest-multi") {
            selftest_multi(Path::new(args.get(i + 1).map(String::as_str).unwrap_or("multi.phaiform")));
        }
        // `--codetest`: types into the open code file as a keyboard would - Thai, a new line, a
        // backspace - then saves it.
        if args.iter().any(|a| a == "--codetest") {
            if let Some(h) = s.doc.code {
                for u in "// ทดสอบ พิมพ์ไทยx".encode_utf16() {
                    wm::SendMessageW(h, wm::WM_CHAR, Some(WPARAM(u as usize)), Some(LPARAM(1)));
                }
                wm::SendMessageW(h, wm::WM_CHAR, Some(WPARAM(0x08)), Some(LPARAM(1)));
                wm::SendMessageW(h, wm::WM_CHAR, Some(WPARAM(0x0D)), Some(LPARAM(1)));
                s.save(false);
            }
        }
        // `--lsptest`: F12 and F2 against rust-analyzer, end to end - see `lsp_test_step`.
        if args.iter().any(|a| a == "--lsptest") {
            let target = hwnd.0 as isize;
            std::thread::spawn(move || {
                for _ in 0..90 {
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    let _ = wm::PostMessageW(Some(HWND(target as *mut _)), wm::WM_COMMAND, WPARAM(ID_LSP_TEST as usize), LPARAM(0));
                }
                println!("lsptest: timed out");
                std::process::exit(4);
            });
        }
        // `--buildtest`: builds the open form's project the way Build does, prints what came of
        // it and ends - the Build command checked end to end.
        if args.iter().any(|a| a == "--buildtest") {
            BUILD_TEST.store(true, Ordering::Relaxed);
            s.start_build(false);
        }
        // `--servers`: Server Explorer shown - checked by picture.
        if args.iter().any(|a| a == "--servers") {
            explorer::show_servers(s.side, true);
            // `--expand-servers`: the first connection opened, as a click on its [+] does.
            if args.iter().any(|a| a == "--expand-servers") {
                if let Some(sv) = s.servers.as_mut() {
                    sv.expand_first();
                }
            }
            // `--design TABLE`: the Table Designer on the first connection's table (or a new one: `-`).
            if let Some(t) = args.iter().position(|a| a == "--design").and_then(|i| args.get(i + 1)).cloned() {
                // Once the window's loop runs, so a picture can be taken of it.
                *DESIGN_TEST.lock().unwrap_or_else(|e| e.into_inner()) = Some(t);
                let _ = wm::PostMessageW(Some(hwnd), wm::WM_COMMAND, WPARAM(ID_DESIGN_TEST as usize), LPARAM(0));
            }
            // `--show-rows TABLE`: the first connection's table in Results, as Show Table Data.
            if let Some(t) = args.iter().position(|a| a == "--show-rows").and_then(|i| args.get(i + 1)).cloned() {
                s.show_table_rows(0, "main", &t);
            }
        }
        // `--guide-shot DIR`: the dock guides drawn, one plain and one lit, as pictures in DIR.
        if let Some(dir) = args.iter().position(|a| a == "--guide-shot").and_then(|i| args.get(i + 1)) {
            let size = win::px(hwnd, 36);
            for (n, (d, lit)) in [(Dock::Left, false), (Dock::Bottom, true)].into_iter().enumerate() {
                let g = docking::guide_window(hwnd, d, RECT { left: 40 + n as i32 * 80, top: 40, right: 40 + n as i32 * 80 + size, bottom: 40 + size });
                docking::light_guide(g, lit);
                // See-through windows photograph black: solid for the picture.
                let ex = wm::GetWindowLongPtrW(g, wm::GWL_EXSTYLE);
                wm::SetWindowLongPtrW(g, wm::GWL_EXSTYLE, ex & !(wm::WS_EX_LAYERED.0 as isize));
                let _ = Gdi::RedrawWindow(Some(g), None, None, Gdi::RDW_INVALIDATE | Gdi::RDW_UPDATENOW);
                // A moment for the screen to put it together, messages handled meanwhile.
                let mut m = wm::MSG::default();
                for _ in 0..40 {
                    while wm::PeekMessageW(&mut m, None, 0, 0, wm::PM_REMOVE).as_bool() {
                        let _ = wm::TranslateMessage(&m);
                        wm::DispatchMessageW(&m);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                let _ = phai_ui::shot::save(g, &Path::new(dir).join(format!("guide{n}.png")));
                let _ = wm::DestroyWindow(g);
            }
            std::process::exit(0);
        }
        // `--publishtest`: Build > Publish without the questions; prints where the setup went.
        if args.iter().any(|a| a == "--publishtest") {
            PUBLISH_TEST.store(true, Ordering::Relaxed);
            std::env::set_var("PHAI_PUBLISH_TEST", "1");
            s.command(ID_PUBLISH);
        }
        // `--debugtest`: see `debug_test_step`. With `PHAI_ATTACH_PID`, attached to that program
        // rather than started - Stop then lets it go on.
        if args.iter().any(|a| a == "--debugtest") {
            DEBUG_TEST.store(1, Ordering::Relaxed);
            let line = std::env::var("PHAI_DEBUG_LINE").ok().and_then(|l| l.parse().ok()).unwrap_or(7);
            if let Some(h) = s.doc.code {
                phai_editor::view::set_breakpoints(h, &[line]);
            }
            s.command(if std::env::var_os("PHAI_ATTACH_PID").is_some() { ID_ATTACH } else { ID_RUN });
        }
        // `--jobtest clean|rebuild|analyse|solution`: a Build menu job, printed as --buildtest prints.
        if let Some(j) = args.iter().position(|a| a == "--jobtest").and_then(|i| args.get(i + 1)) {
            BUILD_TEST.store(true, Ordering::Relaxed);
            match j.as_str() {
                "clean" => s.start_job(cargo::Job::Clean, false),
                "rebuild" => s.start_job(cargo::Job::Rebuild, false),
                "analyse" => s.start_job(cargo::Job::Analyse, false),
                "test" => s.start_job(cargo::Job::Test, false),
                "batch" => {
                    std::env::set_var("PHAI_BATCH", "build");
                    s.command(ID_BATCH_BUILD);
                }
                _ => s.start_job(cargo::Job::Build, true),
            }
        }
        // The loop: shortcuts first, then Tab between boxes, then everything else.
        if std::env::var("PHAI_SHOT").is_ok() {
            win::run_loop(Some(hwnd));
            return;
        }
        let quiet = std::env::var_os("PHAI_SHOT").is_some();
        let mut msg = wm::MSG::default();
        while wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
            // Being photographed, a program takes no keys and no mouse: a person typing or moving the
            // mouse elsewhere must not reach it. Tests send theirs straight to the window, past this queue.
            if quiet && ((wm::WM_KEYFIRST..=wm::WM_KEYLAST).contains(&msg.message) || (wm::WM_MOUSEFIRST..=wm::WM_MOUSELAST).contains(&msg.message)) {
                continue;
            }
            if let Some(cmd) = shortcut(&msg) {
                if let Some(s) = studio() {
                    s.command(cmd);
                }
                continue;
            }
            if wm::IsDialogMessageW(hwnd, &msg).as_bool() {
                continue;
            }
            let _ = wm::TranslateMessage(&msg);
            wm::DispatchMessageW(&msg);
        }
    }
}

/// A new form in the project at `dir`: `src/<name>.phaiform`, its generated `_ui.rs` and the
/// programmer's file, and main.rs told of it. The form's path, and whether main.rs was changed.
fn write_new_form(dir: &Path, name: &str) -> std::io::Result<(PathBuf, bool)> {
    let src = dir.join("src");
    let file = snake(name);
    let path = src.join(format!("{file}.phaiform"));
    let mut f = FormFile::new(name);
    f.form.title = name.to_string();
    f.form.size = [400, 300];
    if dir.join("theme.css").is_file() {
        f.form.theme = Some("../theme.css".into());
    }
    let opts = theme_options(&path, f.form.theme.as_deref());
    std::fs::create_dir_all(&src)?;
    std::fs::write(&path, f.to_toml())?;
    std::fs::write(src.join(format!("{file}_ui.rs")), codegen::generate(&f, &opts))?;
    std::fs::write(src.join(format!("{file}.rs")), codegen::user_stub(&f))?;
    project::add_thai(&src, &[(name, &format!("ฟอร์ม {name}"))])?;
    let main_rs = src.join("main.rs");
    let declared = std::fs::read_to_string(&main_rs).map(|text| declare_form(&text, name)).and_then(|text| std::fs::write(&main_rs, text)).is_ok();
    Ok((path, declared))
}

/// A table's controls for a form: a grid bound to it under what the form has (anchored to grow
/// with the window), Reload and Save below it at the right. And how tall the form must be.
fn bound_controls(form: &FormFile, database: &str, table: &str, cols: &[phai_data::Column]) -> (Vec<phai_form::Control>, i32) {
    let taken: Vec<String> = form.control.iter().map(|c| c.id.clone()).collect();
    let free = |base: String| -> String {
        let mut id = base.clone();
        let mut n = 2;
        while taken.contains(&id) {
            id = format!("{base}{n}");
            n += 1;
        }
        id
    };
    let grid_id = free(bound_id(table));
    let reload_id = free(format!("{grid_id}_reload"));
    let save_id = free(format!("{grid_id}_save"));
    let fw = form.form.size[0];
    let top = form.control.iter().filter(|c| c.dock.is_none() && !c.kind.is_component()).filter_map(|c| c.bounds).map(|[_, y, _, h]| y + h).max().map_or(11, |b| b + 8);
    let width = (fw - 22).max(200);
    let mut grid = phai_form::Control::new(Kind::Grid, &grid_id, [11, top, width, 200]);
    grid.columns = cols.iter().map(|c| (c.name.clone(), 120)).collect();
    grid.anchor = Some("all".into());
    grid.bind = Some(phai_form::Bind {
        database: database.to_string(),
        table: table.to_string(),
        columns: cols.iter().map(|c| c.name.clone()).collect(),
        keys: cols.iter().filter(|c| c.key == "PRI").map(|c| c.name.clone()).collect(),
    });
    let mut reload = phai_form::Control::new(Kind::Button, &reload_id, [11 + width - 162, top + 208, 75, 23]);
    reload.text = Some("Reload".into());
    reload.anchor = Some("right bottom".into());
    reload.bind_action = Some(format!("{grid_id}:reload"));
    let mut save = phai_form::Control::new(Kind::Button, &save_id, [11 + width - 75, top + 208, 75, 23]);
    save.text = Some("Save".into());
    save.class = Some("primary".into());
    save.anchor = Some("right bottom".into());
    save.bind_action = Some(format!("{grid_id}:save"));
    (vec![grid, reload, save], top + 208 + 23 + 11)
}

/// The project made ready for a bound table: `src/data_sources.rs` saying where the data is
/// (written once), `mod data_sources;` in main.rs, phai-data in Cargo.toml, and Thai for the
/// buttons' words so the project's test of its Thai table still passes.
fn bind_project_files(dir: &Path, conn: &phai_data::Connection) -> std::io::Result<()> {
    let src = dir.join("src");
    let sources = src.join("data_sources.rs");
    if !sources.exists() {
        std::fs::write(&sources, project::data_sources_rs(conn))?;
    }
    let main_rs = src.join("main.rs");
    if let Ok(text) = std::fs::read_to_string(&main_rs) {
        if !text.lines().any(|l| l.trim() == "mod data_sources;") {
            let lines: Vec<&str> = text.lines().collect();
            let at = lines.iter().rposition(|l| l.trim_start().starts_with("mod ") && l.trim_end().ends_with(';')).map_or(0, |i| i + 1);
            let mut out: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
            out.insert(at, "mod data_sources;".into());
            std::fs::write(&main_rs, out.join("\n") + "\n")?;
        }
    }
    let cargo_toml = dir.join("Cargo.toml");
    if let Ok(text) = std::fs::read_to_string(&cargo_toml) {
        let with = project::with_phai_data(&text);
        if with != text {
            std::fs::write(&cargo_toml, with)?;
        }
    }
    project::add_thai(&src, &[("Reload", "โหลดใหม่"), ("Save", "บันทึก")])
}

/// `file` as seen from `dir`: `theme.css`, or `../theme.css` one folder up. Both must be in the
/// same tree (a project).
fn relative_to(dir: &Path, file: &Path) -> Option<String> {
    let d: Vec<_> = dir.components().collect();
    let f: Vec<_> = file.components().collect();
    let same = d.iter().zip(&f).take_while(|(a, b)| a == b).count();
    if same == 0 {
        return None;
    }
    let mut parts: Vec<String> = vec!["..".to_string(); d.len() - same];
    parts.extend(f[same..].iter().map(|c| c.as_os_str().to_string_lossy().to_string()));
    Some(parts.join("/"))
}

/// The line (from 1) of the first method in a form's code file: its first `fn` inside an `impl`.
fn first_method_line(src: &str) -> Option<usize> {
    let mut in_impl = false;
    for (i, l) in src.lines().enumerate() {
        let t = l.trim_start();
        if t.starts_with("impl ") {
            in_impl = true;
        } else if in_impl && (t.starts_with("fn ") || t.starts_with("pub fn ")) {
            return Some(i + 1);
        }
    }
    None
}

/// A table's name as a control's id: `Order Items` is `order_items`.
fn bound_id(table: &str) -> String {
    let mut id: String = table.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' }).collect();
    id = id.trim_matches('_').to_string();
    if id.is_empty() || id.starts_with(|c: char| c.is_ascii_digit()) || ["type", "match", "fn", "struct", "use", "mod", "ref", "loop", "move", "self", "impl", "let"].contains(&id.as_str()) {
        id.insert_str(0, "t_");
    }
    id
}

/// What a person typed as a form's name, as a Rust type name: letters and digits only, the first
/// a capital - "settings window" is `SettingsWindow`.
fn form_type_name(typed: &str) -> String {
    let mut out = String::new();
    let mut up = true;
    for c in typed.trim().chars() {
        if c.is_ascii_alphanumeric() {
            if up {
                out.extend(c.to_uppercase());
            } else {
                out.push(c);
            }
            up = false;
        } else {
            up = true;
        }
    }
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert_str(0, "Form");
    }
    out
}

/// main.rs with a new form's two modules declared after the last `mod` line (or at the top), with
/// a line saying how to show it; unchanged if they are there already.
fn declare_form(main_rs: &str, name: &str) -> String {
    let file = snake(name);
    if main_rs.lines().any(|l| l.trim() == format!("mod {file};")) {
        return main_rs.to_string();
    }
    let block = format!("// {name}: show it from another window with {name}Ui::show(ui.hwnd, {file}::{name}::default())
// (or {name}Ui::show_dialog(...) to wait until it is closed).
#[allow(dead_code)]
mod {file};
#[allow(dead_code)]
mod {file}_ui;
");
    let lines: Vec<&str> = main_rs.lines().collect();
    let at = lines.iter().rposition(|l| l.trim_start().starts_with("mod ") && l.trim_end().ends_with(';')).map(|i| i + 1).unwrap_or(0);
    let mut out = String::new();
    for (i, l) in lines.iter().enumerate() {
        if i == at {
            out.push_str(&block);
        }
        out.push_str(l);
        out.push(char::from(10));
    }
    if at >= lines.len() {
        out.push_str(&block);
    }
    out
}

#[cfg(test)]
mod add_form_tests {
    use super::*;

    #[test]
    fn the_first_method_is_found() {
        let src = "use x;

pub struct A {}

impl AEvents for A {
    fn ok_clicked(&mut self, ui: &AUi) {
    }
}
";
        assert_eq!(first_method_line(src), Some(6));
        assert_eq!(first_method_line("impl AEvents for A {
}
"), None);
    }

    #[test]
    fn a_new_form_is_named_and_declared() {
        assert_eq!(form_type_name("settings window"), "SettingsWindow");
        assert_eq!(form_type_name("Form2"), "Form2");
        assert_eq!(form_type_name("2nd"), "Form2nd");
        let main = "//! app

mod main_window;
mod main_window_ui;
mod text;

fn main() {}
";
        let out = declare_form(main, "Form2");
        assert!(out.contains("mod text;
// Form2: show it"), "{out}");
        assert!(out.contains("#[allow(dead_code)]
mod form2;
#[allow(dead_code)]
mod form2_ui;

fn main()"), "{out}");
        assert_eq!(declare_form(&out, "Form2"), out, "declared once");
    }

    /// A real project with a second form shown from the first, both ways, built by Cargo.
    #[test]
    #[ignore = "builds a project with cargo"]
    fn a_project_with_a_second_form_builds() {
        let dir = std::env::temp_dir().join(format!("phai-addform-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        crate::project::create(&dir).unwrap();
        let (path, declared) = write_new_form(&dir, &form_type_name("Settings Window")).unwrap();
        assert!(declared && path.ends_with("settings_window.phaiform"));
        let user = dir.join("src").join("main_window.rs");
        let text = std::fs::read_to_string(&user).unwrap();
        let text = text.replacen("fn ok_clicked(&mut self, ui: &MainWindowUi) {", "fn ok_clicked(&mut self, ui: &MainWindowUi) {
        crate::settings_window_ui::SettingsWindowUi::show(ui.hwnd, crate::settings_window::SettingsWindow::default());
        crate::settings_window_ui::SettingsWindowUi::show_dialog(ui.hwnd, crate::settings_window::SettingsWindow::default());", 1);
        std::fs::write(&user, text).unwrap();
        let cargo = crate::cargo::find_cargo().expect("cargo");
        let out = std::process::Command::new(cargo).args(["build", "--quiet"]).current_dir(&dir).env("CARGO_TARGET_DIR", std::env::temp_dir().join("phai-addform-target")).output().unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        let warnings = String::from_utf8_lossy(&out.stderr).matches("warning").count();
        assert_eq!(warnings, 0, "{}", String::from_utf8_lossy(&out.stderr));
        // Running: the second window opened and closed, and the program goes on - its main window
        // still running when the timer fires (42); quitting with the second window would end it at 0.
        let text = std::fs::read_to_string(&user).unwrap();
        let text = text.replacen("impl MainWindowEvents for MainWindow {", "impl MainWindowEvents for MainWindow {
    fn loaded(&mut self, ui: &MainWindowUi) {
        let second = crate::settings_window_ui::SettingsWindowUi::show(ui.hwnd, crate::settings_window::SettingsWindow::default());
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(second);
            unsafe extern \"system\" fn done(_: windows::Win32::Foundation::HWND, _: u32, _: usize, _: u32) { std::process::exit(42) }
            windows::Win32::UI::WindowsAndMessaging::SetTimer(None, 0, 700, Some(done));
        }
    }
", 1);
        std::fs::write(&user, text).unwrap();
        let target = std::env::temp_dir().join("phai-addform-target");
        let out = std::process::Command::new(crate::cargo::find_cargo().unwrap()).args(["build", "--quiet"]).current_dir(&dir).env("CARGO_TARGET_DIR", &target).output().unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        let tests = std::process::Command::new(crate::cargo::find_cargo().unwrap()).args(["test", "--quiet"]).current_dir(&dir).env("CARGO_TARGET_DIR", &target).output().unwrap();
        assert!(tests.status.success(), "the project's own tests: {}{}", String::from_utf8_lossy(&tests.stdout), String::from_utf8_lossy(&tests.stderr));
        let exe = target.join("debug").join(format!("{}.exe", crate::project::package_name(&dir.file_name().unwrap().to_string_lossy())));
        let code = std::process::Command::new(exe).env("PHAI_SHOT", std::env::temp_dir().join("phai-addform.png")).env("PHAI_SHOT_DELAY", "20000").status().unwrap().code();
        assert_eq!(code, Some(42), "the program ended when its second window closed");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A real project whose form and controls use WinForms' properties from the table - built,
    /// warning-free, and run: each property found in the window Windows made.
    #[test]
    #[ignore = "builds and runs a project with cargo"]
    fn winforms_properties_reach_the_running_program() {
        let dir = std::env::temp_dir().join(format!("phai-props-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        crate::project::create(&dir).unwrap();
        let form_path = dir.join("src").join("main_window.phaiform");
        let mut f = FormFile::parse(&std::fs::read_to_string(&form_path).unwrap()).unwrap();
        let v = |s: &str| phai_form::toml::Value::String(s.into());
        let b = phai_form::toml::Value::Boolean;
        let n = |x: i64| phai_form::toml::Value::Integer(x);
        use phai_form::{Control, HasProps};
        let mut name = Control::new(Kind::Edit, "name", [11, 40, 200, 23]);
        name.set_prop("read_only", Some(b(true)));
        name.set_prop("max_length", Some(n(12)));
        name.set_prop("placeholder", Some(v("Your name")));
        name.set_prop("text_align", Some(v("right")));
        name.set_prop("tooltip", Some(v("Your name")));
        name.set_prop("tab_index", Some(n(1)));
        let mut pick = Control::new(Kind::Combo, "pick", [11, 70, 200, 23]);
        pick.items = vec!["Pear".into(), "Apple".into(), "Fig".into()];
        pick.set_prop("sorted", Some(b(true)));
        pick.set_prop("selected_index", Some(n(2)));
        pick.set_prop("editable", Some(b(true)));
        pick.set_prop("tab_index", Some(n(0)));
        let mut list = Control::new(Kind::List, "rows", [11, 100, 200, 80]);
        list.set_prop("grid_lines", Some(b(true)));
        list.set_prop("multi_select", Some(b(false)));
        let mut tree = Control::new(Kind::Tree, "places", [220, 40, 150, 120]);
        tree.set_prop("check_boxes", Some(b(true)));
        tree.set_prop("show_lines", Some(b(true)));
        tree.set_prop("expand_all", Some(b(true)));
        let mut when = Control::new(Kind::Date, "when", [220, 170, 150, 23]);
        when.set_prop("format", Some(v("custom")));
        when.set_prop("custom_format", Some(v("yyyy/MM/dd")));
        when.set_prop("date", Some(v("2026-11-19")));
        let mut go = Control::new(Kind::Button, "go", [220, 200, 75, 23]);
        go.text = Some("Go".into());
        go.set_prop("dialog_result", Some(v("ok")));
        go.set_prop("text_align", Some(v("left")));
        f.control.extend([name, pick, list, tree, when, go]);
        f.form.set_prop("accept_button", Some(v("go")));
        f.form.set_prop("border", Some(v("fixed_single")));
        f.form.set_prop("maximize_box", Some(b(false)));
        f.form.set_prop("opacity", Some(n(95)));
        f.form.set_size("max_size", Some([900, 700]));
        assert!(f.check().is_empty(), "{:?}", f.check());
        let again = FormFile::parse(&f.to_toml()).unwrap();
        assert_eq!(again, f, "the properties are kept in the file");
        std::fs::write(&form_path, f.to_toml()).unwrap();
        std::fs::write(dir.join("src").join("main_window_ui.rs"), codegen::generate(&f, &theme_options(&form_path, f.form.theme.as_deref()))).unwrap();
        crate::project::add_thai(&dir.join("src"), &[("Your name", "ชื่อของคุณ"), ("Go", "ไป"), ("Pear", "สาลี่"), ("Apple", "แอปเปิล"), ("Fig", "มะเดื่อ"), ("Name", "ชื่อ"), ("Value", "ค่า"), ("Root", "ราก"), ("  First", "  แรก"), ("  Second", "  สอง")]).unwrap();
        let user = dir.join("src").join("main_window.rs");
        let check = "impl MainWindowEvents for MainWindow {\n    fn loaded(&mut self, ui: &MainWindowUi) {\n        use windows::Win32::UI::WindowsAndMessaging as wm;\n        use windows::Win32::Foundation::{LPARAM, WPARAM};\n        let style = |h| unsafe { wm::GetWindowLongW(h, wm::GWL_STYLE) } as u32;\n        let send = |h, m, w: usize, l: isize| unsafe { wm::SendMessageW(h, m, Some(WPARAM(w)), Some(LPARAM(l))).0 };\n        let mut bad = 0;\n        if style(ui.name.hwnd()) & 0x0800 == 0 { bad |= 1 } // ES_READONLY\n        if send(ui.name.hwnd(), 0x00D5, 0, 0) != 12 { bad |= 2 } // EM_GETLIMITTEXT\n        if style(ui.name.hwnd()) & 0x3 != 2 { bad |= 4 } // ES_RIGHT\n        if send(ui.pick.hwnd(), 0x0147, 0, 0) != 2 { bad |= 8 } // CB_GETCURSEL\n        if style(ui.pick.hwnd()) & 0x3 != 2 { bad |= 16 } // CBS_DROPDOWN\n        if style(ui.rows.hwnd()) & 0x4 == 0 { bad |= 32 } // LVS_SINGLESEL\n        if send(ui.rows.hwnd(), 0x1037, 0, 0) & 1 == 0 { bad |= 64 } // LVS_EX_GRIDLINES\n        if style(ui.places.hwnd()) & 0x102 != 0x102 { bad |= 128 } // TVS_CHECKBOXES | TVS_HASLINES\n        if style(ui.hwnd) & wm::WS_MAXIMIZEBOX.0 != 0 || style(ui.hwnd) & wm::WS_THICKFRAME.0 != 0 { bad |= 256 }\n        if phai_ui::win::text_of(ui.pick) != \"Pear\" && phai_ui::win::text_of(ui.pick) != \"\u{e2a}\u{e32}\u{e25}\u{e35}\u{e48}\" { bad |= 512 } // sorted: Apple, Fig, Pear\n        if phai_ui::win::date_value(ui.when) != (2026, 11, 19) { bad |= 1024 }\n        std::process::exit(if bad == 0 { 42 } else { 100 + bad });\n    }\n";
        let text = std::fs::read_to_string(&user).unwrap().replacen("impl MainWindowEvents for MainWindow {", check, 1);
        std::fs::write(&user, text).unwrap();
        let target = std::env::temp_dir().join("phai-props-target");
        let cargo = crate::cargo::find_cargo().unwrap();
        let run = |args: &[&str]| std::process::Command::new(&cargo).args(args).current_dir(&dir).env("CARGO_TARGET_DIR", &target).output().unwrap();
        let out = run(&["build", "--quiet"]);
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        assert_eq!(String::from_utf8_lossy(&out.stderr).matches("warning").count(), 0, "{}", String::from_utf8_lossy(&out.stderr));
        let tests = run(&["test", "--quiet"]);
        assert!(tests.status.success(), "the project's own tests: {}{}", String::from_utf8_lossy(&tests.stdout), String::from_utf8_lossy(&tests.stderr));
        let exe = target.join("debug").join(format!("{}.exe", crate::project::package_name(&dir.file_name().unwrap().to_string_lossy())));
        let code = std::process::Command::new(exe).env("PHAI_SHOT", std::env::temp_dir().join("phai-props.png")).status().unwrap().code();
        assert_eq!(code, Some(42), "100 + the bits of what the window lacks");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A real project whose controls and form have events from the table - built and run, the
    /// events made with Win32 messages, each method marking that it ran.
    #[test]
    #[ignore = "builds and runs a project with cargo"]
    fn events_reach_the_programmers_methods() {
        use phai_form::{Control, HasProps};
        let dir = std::env::temp_dir().join(format!("phai-events-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        crate::project::create(&dir).unwrap();
        let form_path = dir.join("src").join("main_window.phaiform");
        let mut f = FormFile::parse(&std::fs::read_to_string(&form_path).unwrap()).unwrap();
        let mut name = Control::new(Kind::Edit, "name", [11, 40, 200, 23]);
        for e in ["enter", "key_down", "key_press"] {
            name.events.insert(e.into(), phai_form::events::method_name("name", e));
        }
        let mut hint = Control::new(Kind::Label, "hint", [11, 70, 200, 23]);
        hint.text = Some("Hint".into());
        hint.events.insert("click".into(), "hint_click".into());
        hint.events.insert("mouse_enter".into(), "hint_mouse_enter".into());
        let mut agree = Control::new(Kind::Checkbox, "agree", [11, 100, 200, 23]);
        agree.text = Some("Agree".into());
        agree.events.insert("checked_changed".into(), "agree_changed".into());
        f.control.extend([name, hint, agree]);
        f.form.set_prop("key_preview", Some(phai_form::toml::Value::Boolean(true)));
        for e in ["key_down", "closing", "resize", "shown"] {
            f.form.events.insert(e.into(), format!("window_{e}"));
        }
        assert!(f.check().is_empty(), "{:?}", f.check());
        assert_eq!(FormFile::parse(&f.to_toml()).unwrap(), f, "the events are kept in the file");
        std::fs::write(&form_path, f.to_toml()).unwrap();
        std::fs::write(dir.join("src").join("main_window_ui.rs"), codegen::generate(&f, &theme_options(&form_path, f.form.theme.as_deref()))).unwrap();
        crate::project::add_thai(&dir.join("src"), &[("Hint", "คำแนะนำ"), ("Agree", "ยอมรับ")]).unwrap();
        // The methods, each a bit; a timer makes the events once the window shows, another reads the bits.
        let user = dir.join("src").join("main_window.rs");
        let methods = r#"use std::sync::atomic::{AtomicIsize, AtomicU32, Ordering::SeqCst};
static HIT: AtomicU32 = AtomicU32::new(0);
static WIN: AtomicIsize = AtomicIsize::new(0);
static NAME: AtomicIsize = AtomicIsize::new(0);
static HINT: AtomicIsize = AtomicIsize::new(0);
static AGREE: AtomicIsize = AtomicIsize::new(0);
fn hit(bit: u32) { HIT.fetch_or(1 << bit, SeqCst); }
fn h(a: &AtomicIsize) -> windows::Win32::Foundation::HWND { windows::Win32::Foundation::HWND(a.load(SeqCst) as *mut _) }
unsafe extern "system" fn poke(_: windows::Win32::Foundation::HWND, _: u32, id: usize, _: u32) {
    use windows::Win32::UI::WindowsAndMessaging as wm;
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    unsafe {
        let _ = wm::KillTimer(None, id);
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(h(&NAME)));
        wm::SendMessageW(h(&NAME), wm::WM_KEYDOWN, Some(WPARAM(0x41)), Some(LPARAM(0)));
        wm::SendMessageW(h(&NAME), wm::WM_KEYDOWN, Some(WPARAM(0x58)), Some(LPARAM(0)));
        wm::SendMessageW(h(&NAME), wm::WM_CHAR, Some(WPARAM('z' as usize)), Some(LPARAM(0)));
        wm::SendMessageW(h(&NAME), wm::WM_CHAR, Some(WPARAM('b' as usize)), Some(LPARAM(0)));
        wm::SendMessageW(h(&HINT), wm::WM_MOUSEMOVE, Some(WPARAM(0)), Some(LPARAM((5 << 16) | 5)));
        wm::SendMessageW(h(&HINT), wm::WM_LBUTTONDOWN, Some(WPARAM(1)), Some(LPARAM((5 << 16) | 5)));
        wm::SendMessageW(h(&HINT), wm::WM_LBUTTONUP, Some(WPARAM(0)), Some(LPARAM((5 << 16) | 5)));
        wm::SendMessageW(h(&AGREE), 0x00F5, None, None);
        let _ = wm::SetWindowPos(h(&WIN), None, 0, 0, 700, 500, wm::SWP_NOMOVE | wm::SWP_NOZORDER);
        wm::SendMessageW(h(&WIN), wm::WM_CLOSE, None, None);
        if !wm::IsWindow(Some(h(&WIN))).as_bool() { std::process::exit(5) }
        if phai_ui::win::text_of(h(&NAME)) != "b" { std::process::exit(6) }
        let bits = HIT.load(SeqCst);
        std::process::exit(if bits == 0x3FF { 42 } else { 1000 + bits as i32 });
    }
}

impl MainWindowEvents for MainWindow {
    fn loaded(&mut self, ui: &MainWindowUi) {
        WIN.store(ui.hwnd.0 as isize, SeqCst);
        NAME.store(ui.name.hwnd().0 as isize, SeqCst);
        HINT.store(ui.hint.hwnd().0 as isize, SeqCst);
        AGREE.store(ui.agree.hwnd().0 as isize, SeqCst);
    }
    fn window_shown(&mut self, _ui: &MainWindowUi) {
        hit(0);
        unsafe { windows::Win32::UI::WindowsAndMessaging::SetTimer(None, 0, 200, Some(poke)); }
    }
    fn name_enter(&mut self, _ui: &MainWindowUi) { hit(1) }
    fn name_key_down(&mut self, _ui: &MainWindowUi, e: &phai_ui::events::KeyEventArgs) {
        if e.key == 0x41 { hit(2) }
    }
    fn window_key_down(&mut self, _ui: &MainWindowUi, e: &phai_ui::events::KeyEventArgs) {
        // The form first: X never reaches the box.
        if e.key == 0x58 { hit(3); e.set_handled(); }
    }
    fn name_key_press(&mut self, _ui: &MainWindowUi, e: &phai_ui::events::KeyPressEventArgs) {
        if e.ch == 'z' { hit(4); e.set_handled(); }
    }
    fn hint_mouse_enter(&mut self, _ui: &MainWindowUi) { hit(5) }
    fn hint_click(&mut self, _ui: &MainWindowUi) { hit(6) }
    fn agree_changed(&mut self, _ui: &MainWindowUi) { hit(7) }
    fn window_resize(&mut self, _ui: &MainWindowUi) { hit(8) }
    fn window_closing(&mut self, _ui: &MainWindowUi, e: &phai_ui::events::ClosingEventArgs) { hit(9); e.cancel(); }
"#;
        let src = std::fs::read_to_string(&user).unwrap();
        let src = src.replacen("impl MainWindowEvents for MainWindow {", methods, 1);
        std::fs::write(&user, src).unwrap();
        let target = std::env::temp_dir().join("phai-props-target");
        let cargo = crate::cargo::find_cargo().unwrap();
        let run = |args: &[&str]| std::process::Command::new(&cargo).args(args).current_dir(&dir).env("CARGO_TARGET_DIR", &target).output().unwrap();
        let out = run(&["build", "--quiet"]);
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        assert_eq!(String::from_utf8_lossy(&out.stderr).matches("warning").count(), 0, "{}", String::from_utf8_lossy(&out.stderr));
        let tests = run(&["test", "--quiet"]);
        assert!(tests.status.success(), "the project's own tests: {}{}", String::from_utf8_lossy(&tests.stdout), String::from_utf8_lossy(&tests.stderr));
        let exe = target.join("debug").join(format!("{}.exe", crate::project::package_name(&dir.file_name().unwrap().to_string_lossy())));
        let code = std::process::Command::new(exe).env("PHAI_SHOT", std::env::temp_dir().join("phai-events.png")).env("PHAI_SHOT_DELAY", "20000").status().unwrap().code();
        assert_eq!(code, Some(42), "1000 + the bits of the methods that ran");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A real project using its controls as objects - text boxes, drop-downs, lists, trees,
    /// progress bars, buttons, timers - and handing work to its window from another thread.
    #[test]
    #[ignore = "builds and runs a project with cargo"]
    fn control_methods_and_invoke_work() {
        use phai_form::Control;
        let dir = std::env::temp_dir().join(format!("phai-methods-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        crate::project::create(&dir).unwrap();
        let form_path = dir.join("src").join("main_window.phaiform");
        let mut f = FormFile::parse(&std::fs::read_to_string(&form_path).unwrap()).unwrap();
        let mut go = Control::new(Kind::Button, "go", [220, 40, 75, 23]);
        go.text = Some("Go".into());
        go.on_click = Some("go_clicked".into());
        let mut clock = Control::new(Kind::Timer, "clock", [0, 0, 0, 0]);
        clock.bounds = None;
        clock.value = Some(50);
        clock.enabled = Some(false);
        clock.on_tick = Some("clock_ticked".into());
        let mut pick = Control::new(Kind::Combo, "pick", [11, 70, 200, 23]);
        pick.items = vec![];
        let mut places = Control::new(Kind::Tree, "places", [220, 70, 150, 100]);
        places.items = vec![];
        f.control.extend([Control::new(Kind::Edit, "name", [11, 40, 200, 23]), pick, Control::new(Kind::List, "rows", [11, 100, 200, 80]), places, Control::new(Kind::Progress, "bar", [11, 190, 200, 20]), go, clock]);
        assert!(f.check().is_empty(), "{:?}", f.check());
        std::fs::write(&form_path, f.to_toml()).unwrap();
        std::fs::write(dir.join("src").join("main_window_ui.rs"), codegen::generate(&f, &theme_options(&form_path, f.form.theme.as_deref()))).unwrap();
        crate::project::add_thai(&dir.join("src"), &[("Go", "ไป"), ("Name", "ชื่อ"), ("Value", "ค่า")]).unwrap();
        let user = dir.join("src").join("main_window.rs");
        let code = r#"use crate::main_window_ui::{MainWindowEvents, MainWindowUi};
use std::sync::atomic::{AtomicU32, Ordering::SeqCst};
use std::sync::OnceLock;
use phai_ui::ctl::*;
static HIT: AtomicU32 = AtomicU32::new(0);
fn hit(bit: u32) { HIT.fetch_or(1 << bit, SeqCst); }
struct Parts { name: TextBox, pick: ComboBox, rows: ListView, places: TreeView, bar: ProgressBar, go: Button, clock: Timer, inv: phai_ui::invoke::Invoker }
static UI: OnceLock<Parts> = OnceLock::new();
fn ok(b: bool, bit: u32) { if b { hit(bit) } }
unsafe extern "system" fn poke(_: windows::Win32::Foundation::HWND, _: u32, id: usize, _: u32) {
    unsafe { let _ = windows::Win32::UI::WindowsAndMessaging::KillTimer(None, id); }
    let u = UI.get().unwrap();
    u.name.set_text("abc");
    u.name.append_text("de");
    u.name.select(1, 2);
    let picked = u.name.selected_text();
    u.name.hide();
    let hidden = !u.name.visible();
    u.name.show();
    ok(u.name.text() == "abcde" && picked == "bc" && hidden && u.name.visible(), 0);
    u.pick.items_add("Apple");
    u.pick.items_add("Banana");
    u.pick.set_selected_index(Some(1));
    ok(u.pick.selected_item().as_deref() == Some("Banana") && u.pick.find_string("app") == Some(0) && u.pick.items_count() == 2, 1);
    let r = u.rows.items_add(&["x", "y"]);
    u.rows.set_cell(r, 1, "z");
    u.rows.select(r);
    ok(u.rows.cell(0, 1) == "z" && u.rows.selected_index() == Some(0) && u.rows.items_count() == 1, 2);
    let root = u.places.nodes_add("Root");
    let kid = u.places.nodes_add_under(root, "Kid");
    u.places.expand_all();
    u.places.select(kid);
    ok(u.places.selected_text().as_deref() == Some("Kid"), 3);
    u.bar.set_value(10);
    u.bar.perform_step();
    u.bar.increment(5);
    ok(u.bar.value() == 25, 4);
    u.go.perform_click();
    u.clock.start();
    ok(u.clock.enabled() && u.clock.interval() == 50, 5);
    let inv = u.inv;
    std::thread::spawn(move || {
        MainWindowUi::begin_invoke(inv, |me: &mut MainWindow, ui: &MainWindowUi| {
            me.from_thread += 1;
            ui.name.set_text("from a thread");
            hit(7);
        });
    });
    unsafe { windows::Win32::UI::WindowsAndMessaging::SetTimer(None, 0, 1500, Some(judge)); }
}
unsafe extern "system" fn judge(_: windows::Win32::Foundation::HWND, _: u32, _: usize, _: u32) {
    let u = UI.get().unwrap();
    let bits = HIT.load(SeqCst);
    std::process::exit(if bits == 0x3FF && u.name.text() == "from a thread" { 42 } else { 1000 + bits as i32 });
}

#[derive(Default)]
pub struct MainWindow {
    from_thread: u32,
}

impl MainWindowEvents for MainWindow {
    fn loaded(&mut self, ui: &MainWindowUi) {
        let _ = UI.set(Parts { name: ui.name, pick: ui.pick, rows: ui.rows, places: ui.places, bar: ui.bar, go: ui.go, clock: ui.clock, inv: ui.invoker() });
        unsafe { windows::Win32::UI::WindowsAndMessaging::SetTimer(None, 0, 300, Some(poke)); }
    }
    fn go_clicked(&mut self, _ui: &MainWindowUi) { hit(6) }
    fn clock_ticked(&mut self, ui: &MainWindowUi) {
        ui.clock.stop();
        ok(!ui.clock.enabled() && self.from_thread <= 1, 8);
        ui.status.set_part_text(0, "tick");
        hit(9);
    }
}
"#;
        std::fs::write(&user, code).unwrap();
        let target = std::env::temp_dir().join("phai-props-target");
        let cargo = crate::cargo::find_cargo().unwrap();
        let run = |args: &[&str]| std::process::Command::new(&cargo).args(args).current_dir(&dir).env("CARGO_TARGET_DIR", &target).output().unwrap();
        let out = run(&["build", "--quiet"]);
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        assert_eq!(String::from_utf8_lossy(&out.stderr).matches("warning").count(), 0, "{}", String::from_utf8_lossy(&out.stderr));
        let exe = target.join("debug").join(format!("{}.exe", crate::project::package_name(&dir.file_name().unwrap().to_string_lossy())));
        let code = std::process::Command::new(exe).env("PHAI_SHOT", std::env::temp_dir().join("phai-methods.png")).env("PHAI_SHOT_DELAY", "20000").status().unwrap().code();
        assert_eq!(code, Some(42), "1000 + the bits of what worked");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A real project with a SQLite table on its form: built, its Thai test passing, and run -
    /// the grid filled when the window opens, a cell changed and saved, the file holding it.
    #[test]
    #[ignore = "builds and runs a project with cargo"]
    fn a_table_on_a_form_loads_and_saves() {
        let dir = std::env::temp_dir().join(format!("phai-bind-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        crate::project::create(&dir).unwrap();
        let db = dir.join("shop.db");
        let conn = phai_data::Connection { kind: phai_data::Kind::Sqlite, host: db.to_string_lossy().to_string(), ..Default::default() };
        conn.query(None, "CREATE TABLE customers (id INTEGER PRIMARY KEY, name TEXT NOT NULL, phone TEXT); INSERT INTO customers (name, phone) VALUES ('Somchai', '081-234-5678'), ('Monrak Net', '02-111-2222');").unwrap();
        let cols = conn.columns("main", "customers").unwrap();
        let form_path = dir.join("src").join("main_window.phaiform");
        let mut f = FormFile::parse(&std::fs::read_to_string(&form_path).unwrap()).unwrap();
        let (controls, need) = bound_controls(&f, "main", "customers", &cols);
        f.control.extend(controls);
        f.form.size[1] = f.form.size[1].max(need);
        assert!(f.check().is_empty(), "{:?}", f.check());
        std::fs::write(&form_path, f.to_toml()).unwrap();
        std::fs::write(dir.join("src").join("main_window_ui.rs"), codegen::generate(&f, &theme_options(&form_path, f.form.theme.as_deref()))).unwrap();
        bind_project_files(&dir, &conn).unwrap();
        // The program: when its window opens, the grid must hold both rows; one cell changed and
        // saved must be in the file, and the grid read again.
        let user = dir.join("src").join("main_window.rs");
        let text = std::fs::read_to_string(&user).unwrap().replacen(
            "impl MainWindowEvents for MainWindow {",
            "impl MainWindowEvents for MainWindow {\n    fn loaded(&mut self, ui: &MainWindowUi) {\n        let rows = phai_ui::grid::grid_rows(ui.customers);\n        if rows.len() != 2 || rows[1][1] != \"Monrak Net\" { std::process::exit(3) }\n        phai_ui::grid::set_cell(ui.customers, 0, 1, \"สมชาย ใจดี\");\n        let n = ui.save_customers().unwrap_or(99);\n        let now = crate::data_sources::connection().query(None, \"SELECT name FROM customers WHERE id = 1\").unwrap();\n        let again = phai_ui::grid::grid_rows(ui.customers);\n        std::process::exit(if n == 1 && now.rows[0][0] == \"สมชาย ใจดี\" && again[0][1] == \"สมชาย ใจดี\" { 42 } else { 4 });\n    }\n",
            1,
        );
        std::fs::write(&user, text).unwrap();
        let target = std::env::temp_dir().join("phai-bind-target");
        let cargo = crate::cargo::find_cargo().unwrap();
        let run = |args: &[&str]| std::process::Command::new(&cargo).args(args).current_dir(&dir).env("CARGO_TARGET_DIR", &target).output().unwrap();
        let out = run(&["build", "--quiet"]);
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        assert_eq!(String::from_utf8_lossy(&out.stderr).matches("warning").count(), 0, "{}", String::from_utf8_lossy(&out.stderr));
        let out = run(&["test", "--quiet"]);
        assert!(out.status.success(), "the project's own tests: {}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        let exe = target.join("debug").join(format!("{}.exe", crate::project::package_name(&dir.file_name().unwrap().to_string_lossy())));
        let code = std::process::Command::new(exe).env("PHAI_SHOT", std::env::temp_dir().join("phai-bind.png")).env("PHAI_SHOT_DELAY", "20000").status().unwrap().code();
        assert_eq!(code, Some(42), "3: the grid was not filled, 4: the change was not saved");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
