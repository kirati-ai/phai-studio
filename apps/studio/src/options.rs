//! Tools > Options, as Visual Studio has it: Phai Studio's own settings in one window, by kind in
//! a tree on the left - Environment, Text Editor, Form Designer, Projects and Solutions,
//! Debugging, Source Control - each page's settings on the right.

use phai_ui::text::t;
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::WindowsAndMessaging as wm;

const ID_TREE: i32 = 10;
const ID_OK: i32 = 1;
const ID_CANCEL: i32 = 2;

/// Phai Studio's settings as Options shows them.
#[derive(Clone, Debug, PartialEq)]
pub struct Values {
    // Environment
    /// 0 as Windows is set, 1 light, 2 dark.
    pub theme: usize,
    pub thai: bool,
    /// How many files and projects File > Recent lists, 1 to 9.
    pub recent: usize,
    /// On startup: 0 the start window, 1 nothing open, 2 the project opened last.
    pub startup: usize,
    /// Project and Properties as tabs along the bottom.
    pub tabbed: bool,
    /// A form changed outside Phai Studio (the Live Inspector, Git) is noticed...
    pub detect_changes: bool,
    /// ...and loaded again without asking while it has no unsaved changes here.
    pub auto_reload: bool,
    // Text Editor
    pub line_numbers: bool,
    /// Reference counts after a function's line.
    pub lenses: bool,
    pub braces: bool,
    /// rust-analyzer: completion, errors as you type, Go To Definition.
    pub rust_analyzer: bool,
    pub tab_size: usize,
    /// Tab and a new indent type spaces; off, a tab character.
    pub spaces: bool,
    pub font: String,
    /// The code editor's text, points.
    pub font_pt: i32,
    /// rustfmt runs on a Rust file as it is saved.
    pub format_on_save: bool,
    // Form Designer
    pub grid: i32,
    pub snap_to_grid: bool,
    pub snaplines: bool,
    // Projects and Solutions
    pub save_before_build: bool,
    /// The Output pane comes forward as a build starts.
    pub output_on_build: bool,
    /// Debug or Release remembered for next time.
    pub remember_config: bool,
    /// cargo.exe to use; empty, found as usual (PATH, CARGO_HOME, ~/.cargo/bin).
    pub cargo_path: String,
    /// Projects take the Phai libraries from GitHub, at this Phai Studio's release.
    pub git: bool,
    // Debugging
    pub break_on_panic: bool,
    // Source Control
    pub blame: bool,
    // Environment > Error List
    /// The Error List brought forward when a build ends with errors or warnings.
    pub errors_forward: bool,
    /// Errors and warnings shown in it to begin with (its two ticks).
    pub show_errors: bool,
    pub show_warnings: bool,
    // Environment > Other Windows
    /// The tabs of the panel below open when Studio starts, by the order of `bottom::labels`.
    pub start_tabs: Vec<usize>,
    /// The tabs left open kept for next time, in place of `start_tabs`.
    pub remember_tabs: bool,
}

/// The panel's tabs Options offers to open at start: all but the debugger's own (Immediate to
/// Disassembly), which open as debugging needs them.
pub const START_TABS: [usize; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 14, 15];

/// The panel's tabs open at start as a setting says them: `0,1` - Errors and Output when none.
pub fn tabs_of(v: Option<String>) -> Vec<usize> {
    let tabs: Vec<usize> = v.unwrap_or_else(|| "0,1".into()).split(',').filter_map(|n| n.trim().parse().ok()).filter(|k| *k < 16).collect();
    if tabs.is_empty() { vec![0, 1] } else { tabs }
}

pub const THEMES: [&str; 3] = ["windows", "light", "dark"];
pub const STARTUPS: [&str; 3] = ["start", "empty", "last"];
/// Fixed-width fonts Windows has, offered first; any other can be typed.
pub const FONTS: [&str; 5] = ["Cascadia Mono", "Cascadia Code", "Consolas", "Courier New", "Lucida Console"];

/// "1"/"0" as a setting holds a tick; what was never set is `or`.
fn tick(v: Option<String>, or: bool) -> bool {
    v.map(|v| v == "1").unwrap_or(or)
}

fn one(b: bool) -> String {
    if b { "1" } else { "0" }.to_string()
}

impl Values {
    /// The settings as saved; `get` reads one by name. What was never set is Phai Studio's default.
    pub fn load(get: impl Fn(&str) -> Option<String>) -> Values {
        let startup = match get("startup").as_deref() {
            Some(v) => STARTUPS.iter().position(|s| *s == v).unwrap_or(0),
            // Before Options: the start window's own "show at startup" tick.
            None if get("start_window").as_deref() == Some("0") => 1,
            None => 0,
        };
        let number = |k: &str, or: i64, lo: i64, hi: i64| get(k).and_then(|v| v.parse::<i64>().ok()).unwrap_or(or).clamp(lo, hi);
        Values {
            theme: get("theme").and_then(|v| THEMES.iter().position(|s| *s == v)).unwrap_or(1),
            thai: get("lang").as_deref() == Some("th"),
            recent: number("recent_max", 9, 1, 9) as usize,
            startup,
            tabbed: tick(get("side_tabbed"), true),
            detect_changes: tick(get("detect_changes"), true),
            auto_reload: tick(get("auto_reload"), true),
            line_numbers: tick(get("line_numbers"), true),
            lenses: tick(get("code_lens"), true),
            braces: tick(get("brace_highlight"), true),
            rust_analyzer: tick(get("rust_analyzer"), true),
            tab_size: number("tab_size", 4, 1, 16) as usize,
            spaces: tick(get("tab_spaces"), true),
            font: get("editor_font").filter(|f| !f.trim().is_empty()).unwrap_or_else(|| FONTS[0].to_string()),
            font_pt: number("editor_font_pt", 10, 6, 40) as i32,
            format_on_save: tick(get("format_on_save"), false),
            grid: number("grid_size", 4, 1, 64) as i32,
            snap_to_grid: tick(get("snap_to_grid"), true),
            snaplines: tick(get("snaplines"), true),
            save_before_build: tick(get("save_before_build"), true),
            output_on_build: tick(get("output_on_build"), true),
            remember_config: tick(get("remember_config"), false),
            cargo_path: get("cargo_path").unwrap_or_default(),
            git: tick(get("library_from_git"), false),
            // Debug > Exception Settings keeps it among the exceptions: "panic" there.
            break_on_panic: get("exceptions").map(|e| e.split(',').any(|w| w.trim() == "panic")).unwrap_or(true),
            blame: tick(get("blame"), false),
            errors_forward: tick(get("errors_forward"), true),
            show_errors: tick(get("show_errors"), true),
            show_warnings: tick(get("show_warnings"), true),
            start_tabs: tabs_of(get("bottom_tabs")),
            remember_tabs: tick(get("remember_tabs"), false),
        }
    }

    /// Each setting by name, as it is saved. `exceptions` is the list as it was, with "panic" put
    /// in it or taken out.
    pub fn pairs(&self, exceptions: Option<&str>) -> Vec<(&'static str, String)> {
        let mut ex: Vec<String> = exceptions.unwrap_or("panic").split(',').map(|w| w.trim().to_string()).filter(|w| !w.is_empty() && w != "panic").collect();
        if self.break_on_panic {
            ex.insert(0, "panic".to_string());
        }
        vec![
            ("theme", THEMES[self.theme].to_string()),
            ("lang", if self.thai { "th" } else { "en" }.to_string()),
            ("recent_max", self.recent.to_string()),
            ("startup", STARTUPS[self.startup].to_string()),
            // The start window's own tick says the same.
            ("start_window", one(self.startup == 0)),
            ("side_tabbed", one(self.tabbed)),
            ("detect_changes", one(self.detect_changes)),
            ("auto_reload", one(self.auto_reload)),
            ("line_numbers", one(self.line_numbers)),
            ("code_lens", one(self.lenses)),
            ("brace_highlight", one(self.braces)),
            ("rust_analyzer", one(self.rust_analyzer)),
            ("tab_size", self.tab_size.to_string()),
            ("tab_spaces", one(self.spaces)),
            ("editor_font", self.font.clone()),
            ("editor_font_pt", self.font_pt.to_string()),
            ("format_on_save", one(self.format_on_save)),
            ("grid_size", self.grid.to_string()),
            ("snap_to_grid", one(self.snap_to_grid)),
            ("snaplines", one(self.snaplines)),
            ("save_before_build", one(self.save_before_build)),
            ("output_on_build", one(self.output_on_build)),
            ("remember_config", one(self.remember_config)),
            ("cargo_path", self.cargo_path.trim().to_string()),
            ("library_from_git", one(self.git)),
            ("exceptions", ex.join(",")),
            ("blame", one(self.blame)),
            ("errors_forward", one(self.errors_forward)),
            ("show_errors", one(self.show_errors)),
            ("show_warnings", one(self.show_warnings)),
            ("bottom_tabs", self.start_tabs.iter().map(|k| k.to_string()).collect::<Vec<_>>().join(",")),
            ("remember_tabs", one(self.remember_tabs)),
        ]
    }

    /// What the code editor is told.
    pub fn editor(&self) -> phai_editor::prefs::Prefs {
        phai_editor::prefs::Prefs { tab_size: self.tab_size, spaces: self.spaces, line_numbers: self.line_numbers, lenses: self.lenses, braces: self.braces, font: self.font.clone() }
    }
}

/// Whether Windows is set to dark for apps (Settings > Personalisation > Colours).
pub fn windows_dark() -> bool {
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
    let mut v: u32 = 1;
    let mut size = 4u32;
    let key = win::wide(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize");
    let name = win::wide("AppsUseLightTheme");
    let ok = unsafe { RegGetValueW(HKEY_CURRENT_USER, PCWSTR(key.as_ptr()), PCWSTR(name.as_ptr()), RRF_RT_REG_DWORD, None, Some(&mut v as *mut u32 as *mut _), Some(&mut size)) };
    ok.is_ok() && v == 0
}

/// The pages, in the tree's order: (the group it is under, its name).
fn pages() -> Vec<(&'static str, &'static str)> {
    vec![
        (t("Environment"), t("General")),
        (t("Environment"), t("Startup")),
        (t("Environment"), t("Tabs and Windows")),
        (t("Environment"), t("Documents")),
        (t("Environment"), t("Error List")),
        (t("Environment"), t("Other Windows")),
        (t("Text Editor"), t("General")),
        (t("Text Editor"), t("Tabs")),
        (t("Text Editor"), t("Font")),
        (t("Text Editor"), t("Formatting")),
        (t("Form Designer"), t("General")),
        (t("Projects and Solutions"), t("Build and Run")),
        (t("Projects and Solutions"), t("Phai libraries")),
        (t("Debugging"), t("General")),
        (t("Source Control"), t("Git")),
    ]
}

/// What a row of a page holds.
#[derive(Clone, Copy, PartialEq)]
enum Row {
    /// A label and its control beside it.
    Pair(HWND, HWND),
    /// A check box, the whole width.
    Check(HWND),
    /// A quiet note, the whole width.
    Note(HWND),
}

struct Controls {
    theme: HWND,
    language: HWND,
    recent: HWND,
    startup: HWND,
    tabbed: HWND,
    detect: HWND,
    reload: HWND,
    numbers: HWND,
    lenses: HWND,
    braces: HWND,
    analyzer: HWND,
    tab_size: HWND,
    indent: HWND,
    font: HWND,
    font_pt: HWND,
    format: HWND,
    grid: HWND,
    snap: HWND,
    snaplines: HWND,
    save_first: HWND,
    output: HWND,
    remember: HWND,
    cargo: HWND,
    git: HWND,
    panic: HWND,
    blame: HWND,
    errors_forward: HWND,
    show_errors: HWND,
    show_warnings: HWND,
    /// One tick per tab of `START_TABS`.
    start_tabs: Vec<HWND>,
    remember_tabs: HWND,
}

struct Dialog {
    tree: HWND,
    heading: HWND,
    rows: Vec<Vec<Row>>,
    page: usize,
    c: Controls,
    ok: HWND,
    cancel: HWND,
    chosen: Option<Values>,
    done: bool,
}

fn state<'a>(h: HWND) -> Option<&'a mut Dialog> {
    unsafe { win::state::<Dialog>(h) }
}

/// Shows Options over `owner`, starting from `now`, open on `page`; the settings chosen, or
/// `None` on Cancel.
pub fn choose(owner: HWND, now: &Values, page: usize) -> Option<Values> {
    let hwnd = make(owner, now, page);
    unsafe {
        win::enable(owner, false);
        win::show_top(hwnd);
        let mut msg = wm::MSG::default();
        while state(hwnd).is_some_and(|d| !d.done) {
            if !wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
                wm::PostQuitMessage(msg.wParam.0 as i32);
                break;
            }
            if !wm::IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
        }
        let chosen = state(hwnd).and_then(|d| d.chosen.take());
        win::enable(owner, true);
        let _ = wm::SetForegroundWindow(owner);
        let _ = wm::DestroyWindow(hwnd);
        chosen
    }
}

/// The window, filled from `now`, not yet shown - for [`choose`], and for a picture in a test.
pub fn make(owner: HWND, now: &Values, page: usize) -> HWND {
    win::init();
    let class = win::register("PhaiOptions", Some(proc_), None);
    let title = win::wide(t("Options"));
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_DLGMODALFRAME, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_THICKFRAME | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let bold = win::font(hwnd, 0, true);
        let tree = win::tree(hwnd, ID_TREE, font);
        let heading = win::label(hwnd, "", 0, bold);
        let next = std::cell::Cell::new(100);
        let id = || {
            next.set(next.get() + 1);
            next.get()
        };
        let label = |text: &str| win::label(hwnd, text, 0, font);
        let note = |text: &str| {
            let h = win::label(hwnd, text, 0, font);
            win::set_css(h, "muted", "");
            Row::Note(h)
        };
        let combo = |items: &[&str], at: usize| {
            let h = win::combo(hwnd, id(), font, items);
            wm::SendMessageW(h, 0x014E /* CB_SETCURSEL */, Some(WPARAM(at)), None);
            h
        };
        let check = |text: &str, on: bool| win::checkbox(hwnd, text, id(), font, on);
        let c = Controls {
            theme: combo(&[t("Use Windows setting"), t("Light"), t("Dark")], now.theme),
            language: combo(&["English", "ไทย"], now.thai as usize),
            recent: win::number(hwnd, id(), font, 1, 9, now.recent as i32),
            startup: combo(&[t("Start Window"), t("Empty environment"), t("Most recent project")], now.startup),
            tabbed: check(t("Project and Properties as tabs along the bottom"), now.tabbed),
            detect: check(t("Detect when a form is changed outside Phai Studio"), now.detect_changes),
            reload: check(t("Reload it without asking when it has no unsaved changes"), now.auto_reload),
            numbers: check(t("Line numbers"), now.line_numbers),
            lenses: check(t("Reference counts after a function (CodeLens)"), now.lenses),
            braces: check(t("Highlight the matching bracket"), now.braces),
            analyzer: check(t("rust-analyzer: completion, errors as you type, Go To Definition"), now.rust_analyzer),
            tab_size: win::number(hwnd, id(), font, 1, 16, now.tab_size as i32),
            indent: combo(&[t("Insert spaces"), t("Keep tabs")], (!now.spaces) as usize),
            font: {
                let h = win::combo_with(hwnd, id(), font, &FONTS, true);
                win::set_text(h, &now.font);
                h
            },
            font_pt: win::number(hwnd, id(), font, 6, 40, now.font_pt),
            format: check(t("Format a Rust file with rustfmt when it is saved"), now.format_on_save),
            grid: win::number(hwnd, id(), font, 1, 64, now.grid),
            snap: check(t("Snap to grid"), now.snap_to_grid),
            snaplines: check(t("Snaplines: line controls up with each other"), now.snaplines),
            save_first: check(t("Save changes before building"), now.save_before_build),
            output: check(t("Show Output when a build starts"), now.output_on_build),
            remember: check(t("Remember Debug or Release for next time"), now.remember_config),
            cargo: win::edit(hwnd, &now.cargo_path, id(), font, false),
            git: check(t("Take the Phai libraries from GitHub, at this Phai Studio's release"), now.git),
            panic: check(t("Break when a Rust program panics"), now.break_on_panic),
            blame: check(t("Show who last changed each line (blame)"), now.blame),
            errors_forward: check(t("Bring the Error List forward when a build has errors or warnings"), now.errors_forward),
            show_errors: check(t("Show errors in it"), now.show_errors),
            show_warnings: check(t("Show warnings in it"), now.show_warnings),
            start_tabs: START_TABS.iter().map(|k| check(crate::bottom::labels()[*k], now.start_tabs.contains(k))).collect(),
            remember_tabs: check(t("Open the tabs I left open, instead"), now.remember_tabs),
        };
        let rows = vec![
            vec![Row::Pair(label(t("Color theme:")), c.theme), Row::Pair(label(t("Language:")), c.language), Row::Pair(label(t("Items in recently used lists:")), c.recent)],
            vec![Row::Pair(label(t("On startup, open:")), c.startup)],
            vec![Row::Check(c.tabbed)],
            vec![Row::Check(c.detect), Row::Check(c.reload)],
            vec![Row::Check(c.errors_forward), Row::Check(c.show_errors), Row::Check(c.show_warnings), note(t("The two ticks are the Error List's own Errors and Warnings - changed there, they are kept too."))],
            {
                let mut v = vec![note(t("Open in the panel below when Phai Studio starts:"))];
                v.extend(c.start_tabs.iter().map(|h| Row::Check(*h)));
                v.push(Row::Check(c.remember_tabs));
                v.push(note(t("The debugger's windows open when debugging. A closed tab: View > Other Windows.")));
                v
            },
            vec![Row::Check(c.numbers), Row::Check(c.lenses), Row::Check(c.braces), Row::Check(c.analyzer), note(t("Off: less memory and work in the background; the editor still colours and indents."))],
            vec![Row::Pair(label(t("Tab size:")), c.tab_size), Row::Pair(label(t("Indent with:")), c.indent)],
            vec![Row::Pair(label(t("Font:")), c.font), Row::Pair(label(t("Size (points):")), c.font_pt), note(t("A fixed-width font keeps code in columns."))],
            vec![Row::Check(c.format), note(t("Format Document (Ctrl+K, Ctrl+D) does it by hand."))],
            vec![Row::Pair(label(t("Grid size (pixels):")), c.grid), Row::Check(c.snap), Row::Check(c.snaplines)],
            vec![Row::Check(c.save_first), Row::Check(c.output), Row::Check(c.remember), Row::Pair(label(t("cargo.exe:")), c.cargo), note(t("Empty: found as usual - PATH, CARGO_HOME, then .cargo\\bin in your folder."))],
            vec![Row::Check(c.git), note(t("Off: from this machine, beside Phai Studio.")), note(t("On: a project builds on any machine, the same next year."))],
            vec![Row::Check(c.panic), note(t("Which Windows exceptions stop it: Debug > Exception Settings."))],
            vec![Row::Check(c.blame)],
        ];
        let ok = win::button(hwnd, t("OK"), ID_OK, font, true);
        let cancel = win::button(hwnd, t("Cancel"), ID_CANCEL, font, false);
        // The tree: each group, its pages under it; a group's own item opens its first page.
        let all = pages();
        let mut items = vec![0isize; all.len()];
        let mut group_item = 0isize;
        for (k, (group, name)) in all.iter().enumerate() {
            if k == 0 || all[k - 1].0 != *group {
                if group_item != 0 {
                    win::tree_expand(tree, group_item);
                }
                group_item = win::tree_add(tree, 0, group, -1, k as isize);
            }
            items[k] = win::tree_add(tree, group_item, name, -1, k as isize);
        }
        win::tree_expand(tree, group_item);
        let page = page.min(all.len() - 1);
        win::set_state(hwnd, Box::new(Dialog { tree, heading, rows, page, c, ok, cancel, chosen: None, done: false }));
        let (w, h) = (win::px(hwnd, 800), win::px(hwnd, 540));
        let mut r = RECT::default();
        let _ = wm::GetWindowRect(owner, &mut r);
        let _ = wm::SetWindowPos(hwnd, None, r.left + (r.right - r.left - w) / 2, r.top + (r.bottom - r.top - h) / 2, w, h, wm::SWP_NOZORDER);
        win::theme_frame(hwnd);
        wm::SendMessageW(tree, 0x110B /* TVM_SELECTITEM */, Some(WPARAM(9 /* TVGN_CARET */)), Some(LPARAM(items[page])));
        show_page(hwnd, page);
        hwnd
    }
}

fn row_windows(r: &Row) -> Vec<HWND> {
    match r {
        Row::Pair(a, b) => vec![*a, *b],
        Row::Check(h) | Row::Note(h) => vec![*h],
    }
}

fn show_page(h: HWND, page: usize) {
    let Some(d) = state(h) else { return };
    d.page = page;
    let all = pages();
    win::set_text(d.heading, &format!("{}  ›  {}", all[page].0, all[page].1));
    for (k, rows) in d.rows.iter().enumerate() {
        for r in rows {
            for w in row_windows(r) {
                win::show(w, k == page);
            }
        }
    }
    layout(h);
}

fn layout(h: HWND) {
    let Some(d) = state(h) else { return };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let px = |v| win::px(h, v);
    let (m, row, gap, bw) = (px(12), px(26), px(8), px(96));
    let tree_w = px(230);
    let bottom = r.bottom - m - row - gap;
    win::move_to(d.tree, m, m, tree_w, bottom - m);
    let x = m + tree_w + px(16);
    let right = r.right - m;
    win::move_to(d.heading, x, m, right - x, row);
    let mut y = m + row + gap;
    let label_w = px(200);
    for line in &d.rows[d.page] {
        match line {
            Row::Check(b) | Row::Note(b) => win::move_to(*b, x, y, right - x, row),
            Row::Pair(a, b) => {
                win::move_to(*a, x, y, label_w, row);
                let (w, tall) = match win::kind_of(*b) {
                    Some(win::Kind::Combo) => (px(260), px(220)),
                    Some(win::Kind::Edit) if *b == d.c.cargo => (right - x - label_w, row),
                    _ => (px(90), row),
                };
                win::move_to(*b, x + label_w, y, w, tall);
            }
        }
        y += row + gap;
    }
    let by = r.bottom - m - row;
    win::move_to(d.ok, r.right - 2 * bw - m - px(8), by, bw, row);
    win::move_to(d.cancel, r.right - bw - m, by, bw, row);
}

/// What the window says now.
fn read(c: &Controls) -> Values {
    let number = |h: HWND, lo: i32, hi: i32, or: i32| win::text_of(h).trim().parse::<i32>().unwrap_or(or).clamp(lo, hi);
    let font = win::text_of(c.font).trim().to_string();
    Values {
        theme: win::combo_selected(c.theme).max(0) as usize,
        thai: win::combo_selected(c.language) == 1,
        recent: number(c.recent, 1, 9, 9) as usize,
        startup: win::combo_selected(c.startup).max(0) as usize,
        tabbed: win::is_checked(c.tabbed),
        detect_changes: win::is_checked(c.detect),
        auto_reload: win::is_checked(c.reload),
        line_numbers: win::is_checked(c.numbers),
        lenses: win::is_checked(c.lenses),
        braces: win::is_checked(c.braces),
        rust_analyzer: win::is_checked(c.analyzer),
        tab_size: number(c.tab_size, 1, 16, 4) as usize,
        spaces: win::combo_selected(c.indent) != 1,
        font: if font.is_empty() { FONTS[0].to_string() } else { font },
        font_pt: number(c.font_pt, 6, 40, 10),
        format_on_save: win::is_checked(c.format),
        grid: number(c.grid, 1, 64, 4),
        snap_to_grid: win::is_checked(c.snap),
        snaplines: win::is_checked(c.snaplines),
        save_before_build: win::is_checked(c.save_first),
        output_on_build: win::is_checked(c.output),
        remember_config: win::is_checked(c.remember),
        cargo_path: win::text_of(c.cargo).trim().trim_matches('"').to_string(),
        git: win::is_checked(c.git),
        break_on_panic: win::is_checked(c.panic),
        blame: win::is_checked(c.blame),
        errors_forward: win::is_checked(c.errors_forward),
        show_errors: win::is_checked(c.show_errors),
        show_warnings: win::is_checked(c.show_warnings),
        start_tabs: {
            let t: Vec<usize> = START_TABS.iter().zip(&c.start_tabs).filter(|(_, h)| win::is_checked(**h)).map(|(k, _)| *k).collect();
            if t.is_empty() { vec![0] } else { t }
        },
        remember_tabs: win::is_checked(c.remember_tabs),
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(d) = state(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            layout(h);
            LRESULT(0)
        }
        wm::WM_GETMINMAXINFO => {
            let mm = unsafe { &mut *(l.0 as *mut wm::MINMAXINFO) };
            mm.ptMinTrackSize = POINT { x: win::px(h, 680), y: win::px(h, 440) };
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            match win::loword(w.0) as i32 {
                ID_OK => {
                    d.chosen = Some(read(&d.c));
                    d.done = true;
                }
                ID_CANCEL => d.done = true,
                _ => return win::def(h, msg, w, l),
            }
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            const TVN_SELCHANGEDW: u32 = (-451i32) as u32;
            if hdr.hwndFrom == d.tree {
                if hdr.code == cc::NM_CUSTOMDRAW {
                    return win::tree_custom_draw(l);
                }
                if hdr.code == TVN_SELCHANGEDW {
                    if let Some(k) = win::tree_selected(d.tree) {
                        show_page(h, (k as usize).min(pages().len() - 1));
                    }
                }
            }
            win::def(h, msg, w, l)
        }
        wm::WM_CLOSE => {
            d.done = true;
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            drop(unsafe { win::take_state::<Dialog>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_go_out_and_come_back_the_same() {
        let d = Values::load(|_| None);
        assert!(d.theme == 1 && d.recent == 9 && d.startup == 0 && d.tabbed && d.tab_size == 4 && d.spaces && d.font == "Cascadia Mono", "the defaults");
        assert!(d.break_on_panic && d.save_before_build && d.output_on_build && d.snap_to_grid && d.grid == 4 && !d.format_on_save && !d.blame);
        let v = Values {
            theme: 2,
            thai: true,
            recent: 5,
            startup: 2,
            tabbed: false,
            detect_changes: false,
            auto_reload: false,
            line_numbers: false,
            lenses: false,
            braces: false,
            rust_analyzer: false,
            tab_size: 2,
            spaces: false,
            font: "Consolas".into(),
            font_pt: 13,
            format_on_save: true,
            grid: 8,
            snap_to_grid: false,
            snaplines: false,
            save_before_build: false,
            output_on_build: false,
            remember_config: true,
            cargo_path: r"C:\rust\cargo.exe".into(),
            git: true,
            break_on_panic: false,
            blame: true,
            errors_forward: false,
            show_errors: true,
            show_warnings: false,
            start_tabs: vec![0, 1, 6],
            remember_tabs: true,
        };
        let saved: std::collections::HashMap<&str, String> = v.pairs(Some("panic,0xC0000005")).into_iter().collect();
        assert_eq!(saved["exceptions"], "0xC0000005", "panic out, the exceptions kept");
        assert_eq!(Values::load(|k| saved.get(k).cloned()), v);
        // Before Options, the start window's tick said whether it opens.
        assert_eq!(Values::load(|k| (k == "start_window").then(|| "0".to_string())).startup, 1);
    }
}
