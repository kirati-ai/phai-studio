//! A first step toward extensions, with no code to write: language servers one adds for other
//! kinds of file (Tools > Options > Text Editor > Language Servers) and Visual Studio's External
//! Tools (Tools > External Tools...) - other programs run on the file or project open here, each
//! one an item on the Tools menu, what it prints in the Output pane.

use std::path::{Path, PathBuf};

use phai_ui::text::{t, tf};
use phai_ui::win;
use serde::{Deserialize, Serialize};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::WindowsAndMessaging as wm;

/// A language server one added: the files it is for and the command line that starts it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Server {
    /// `.toml` or `.sql, .psql` - the dots and spaces as one likes.
    pub exts: String,
    /// `taplo lsp stdio`: the program, then its arguments.
    pub command: String,
}

impl Server {
    /// The extensions it is for, small letters, no dots: `["sql", "psql"]`.
    pub fn extensions(&self) -> Vec<String> {
        self.exts.split([',', ';', ' ']).map(|e| e.trim().trim_start_matches("*.").trim_start_matches('.').to_ascii_lowercase()).filter(|e| !e.is_empty()).collect()
    }

    /// Whether the file at `path` is one of its kind.
    pub fn takes(&self, path: &Path) -> bool {
        let Some(ext) = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()) else { return false };
        self.extensions().contains(&ext)
    }
}

/// The servers as a setting holds them (JSON); nothing, or anything else, is none.
pub fn parse_servers(text: Option<&str>) -> Vec<Server> {
    text.and_then(|t| serde_json::from_str(t).ok()).unwrap_or_default()
}

/// The servers as they are saved: one line.
pub fn servers_text(list: &[Server]) -> String {
    let kept: Vec<&Server> = list.iter().filter(|s| !s.command.trim().is_empty() && !s.extensions().is_empty()).collect();
    serde_json::to_string(&kept).unwrap_or_else(|_| "[]".into())
}

/// The server for the file at `path`: the first in the list for its extension. Rust files are
/// rust-analyzer's, always - its own setting turns it off.
pub fn server_for<'a>(list: &'a [Server], path: &Path) -> Option<&'a Server> {
    if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("rs")) {
        return None;
    }
    list.iter().find(|s| !s.command.trim().is_empty() && s.takes(path))
}

/// The servers added in Options, as saved.
pub fn servers() -> Vec<Server> {
    parse_servers(crate::setting_pub("language_servers").as_deref())
}

/// An item of Tools > External Tools, as Visual Studio's: its name on the menu, the program,
/// its arguments and folder (placeholders in both), and whether what it prints goes to Output.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tool {
    pub title: String,
    pub command: String,
    #[serde(default)]
    pub args: String,
    #[serde(default)]
    pub dir: String,
    #[serde(default = "yes")]
    pub output: bool,
}

fn yes() -> bool {
    true
}

impl Default for Tool {
    fn default() -> Tool {
        Tool { title: String::new(), command: String::new(), args: String::new(), dir: String::new(), output: true }
    }
}

/// The most the Tools menu shows.
pub const MAX_TOOLS: usize = 24;

pub fn parse_tools(text: Option<&str>) -> Vec<Tool> {
    text.and_then(|t| serde_json::from_str(t).ok()).unwrap_or_default()
}

pub fn tools_text(list: &[Tool]) -> String {
    let kept: Vec<&Tool> = list.iter().filter(|s| !s.command.trim().is_empty()).collect();
    serde_json::to_string(&kept).unwrap_or_else(|_| "[]".into())
}

/// The tools as saved.
pub fn tools() -> Vec<Tool> {
    parse_tools(crate::setting_pub("external_tools").as_deref())
}

/// What the placeholders stand for when a tool runs.
#[derive(Clone, Debug, Default)]
pub struct Places {
    /// The file open in front.
    pub file: Option<PathBuf>,
    /// Its project's folder (where Cargo.toml is), else the file's own.
    pub project: Option<PathBuf>,
    /// What is chosen in the editor.
    pub selected: String,
}

/// The placeholders, as the `>` buttons offer them, with what each is.
pub fn placeholders() -> [(&'static str, &'static str); 5] {
    [
        ("$(FilePath)", t("The open file, its whole path")),
        ("$(FileDir)", t("The open file's folder")),
        ("$(FileName)", t("The open file's name")),
        ("$(ProjectDir)", t("The project's folder")),
        ("$(SelectedText)", t("What is chosen in the editor")),
    ]
}

/// `text` with each placeholder put in: `"$(FilePath)"` becomes `"D:\app\src\main.rs"`.
/// Capitals do not matter, as in Visual Studio; one that is not known is left as it is, and one
/// with nothing to stand for (no file open) is nothing. Folders end without a `\`, so
/// `"$(FileDir)"` in quotes stays one argument.
pub fn expand(text: &str, p: &Places) -> String {
    let dir_of = |f: &Path| f.parent().map(|d| d.to_string_lossy().to_string()).unwrap_or_default();
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find("$(") {
        out.push_str(&rest[..at]);
        let after = &rest[at + 2..];
        let Some(end) = after.find(')') else {
            out.push_str(&rest[at..]);
            return out;
        };
        let name = after[..end].to_ascii_lowercase();
        let value = match name.as_str() {
            "filepath" | "itempath" => p.file.as_ref().map(|f| f.to_string_lossy().to_string()),
            "filedir" | "itemdir" => p.file.as_deref().map(dir_of),
            "filename" | "itemfilename" => p.file.as_ref().and_then(|f| f.file_name()).map(|n| n.to_string_lossy().to_string()),
            "projectdir" | "solutiondir" => p.project.as_ref().map(|d| d.to_string_lossy().trim_end_matches('\\').to_string()).or_else(|| p.file.as_deref().map(dir_of)),
            "selectedtext" | "curtext" => Some(p.selected.clone()),
            _ => None,
        };
        match value {
            Some(v) => out.push_str(&v),
            None if ["filepath", "itempath", "filedir", "itemdir", "filename", "itemfilename", "projectdir", "solutiondir"].contains(&name.as_str()) => {}
            None => out.push_str(&rest[at..at + 2 + end + 1]),
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

/// Whether a tool works on the open file - which is then saved before it runs.
pub fn uses_file(tool: &Tool) -> bool {
    let all = format!("{} {} {}", tool.command, tool.args, tool.dir).to_ascii_lowercase();
    ["$(filepath)", "$(filedir)", "$(filename)", "$(itempath)", "$(itemdir)", "$(itemfilename)"].iter().any(|p| all.contains(p))
}

/// A tool's line in the Output pane, posted for each line it prints; `LPARAM` a `Box<String>`.
pub const WM_TOOL_LINE: u32 = wm::WM_APP + 0x0E30;
/// A tool ended; `LPARAM` a `Box<(String, Option<i32>)>` - its title and exit code.
pub const WM_TOOL_DONE: u32 = wm::WM_APP + 0x0E31;

/// What a tool runs, its placeholders put in: the program found, its arguments as one line (as
/// Windows passes them, quotes and all), the folder it starts in.
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub program: PathBuf,
    pub args: String,
    pub dir: PathBuf,
}

pub fn prepare(tool: &Tool, p: &Places) -> Result<Run, String> {
    let command = expand(&tool.command, p);
    let program = phai_editor::lsp::find_program(&command).ok_or_else(|| tf("{0} was not found: give its whole path, or put its folder on PATH", &[&command.trim()]))?;
    let dir = expand(&tool.dir, p);
    let dir = if dir.trim().is_empty() { p.project.clone().or_else(|| p.file.as_deref().and_then(Path::parent).map(Path::to_path_buf)).unwrap_or_else(std::env::temp_dir) } else { PathBuf::from(dir.trim().trim_matches('"')) };
    Ok(Run { program, args: expand(&tool.args, p), dir })
}

/// Starts a tool. To Output: what it prints is posted to `window` line by line
/// ([`WM_TOOL_LINE`]), then its end ([`WM_TOOL_DONE`]). Otherwise it runs on its own, a console
/// window of its own for a console program, as Visual Studio does.
pub fn start(run: &Run, title: &str, output: bool, window: HWND) -> Result<(), String> {
    use std::io::BufRead;
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};
    let mut cmd = Command::new(&run.program);
    if !run.args.trim().is_empty() {
        cmd.raw_arg(&run.args);
    }
    cmd.current_dir(&run.dir);
    if !output {
        // CREATE_NEW_CONSOLE
        return cmd.creation_flags(0x10).spawn().map(|_| ()).map_err(|e| e.to_string());
    }
    // CREATE_NO_WINDOW: what it prints comes here.
    let mut child = cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).creation_flags(0x0800_0000).spawn().map_err(|e| e.to_string())?;
    let target = window.0 as isize;
    let post = move |msg: u32, l: isize| unsafe {
        let _ = wm::PostMessageW(Some(HWND(target as *mut _)), msg, WPARAM(0), LPARAM(l));
    };
    let mut readers = Vec::new();
    let out = child.stdout.take().map(|o| Box::new(o) as Box<dyn std::io::Read + Send>);
    let err = child.stderr.take().map(|e| Box::new(e) as Box<dyn std::io::Read + Send>);
    for pipe in [out, err].into_iter().flatten() {
        readers.push(std::thread::spawn(move || {
            let mut r = std::io::BufReader::new(pipe);
            let mut buf = Vec::new();
            while r.read_until(b'\n', &mut buf).unwrap_or(0) > 0 {
                let line = String::from_utf8_lossy(&buf).trim_end_matches(['\r', '\n']).to_string();
                post(WM_TOOL_LINE, Box::into_raw(Box::new(line)) as isize);
                buf.clear();
            }
        }));
    }
    let title = title.to_string();
    std::thread::spawn(move || {
        for r in readers {
            let _ = r.join();
        }
        let code = child.wait().ok().and_then(|s| s.code());
        post(WM_TOOL_DONE, Box::into_raw(Box::new((title, code))) as isize);
    });
    Ok(())
}

// --- Tools > External Tools... -----------------------------------------------------------------

const ID_LIST: i32 = 10;
const ID_ADD: i32 = 11;
const ID_DELETE: i32 = 12;
const ID_UP: i32 = 13;
const ID_DOWN: i32 = 14;
const ID_TITLE: i32 = 20;
const ID_COMMAND: i32 = 21;
const ID_ARGS: i32 = 22;
const ID_DIR: i32 = 23;
const ID_OUTPUT: i32 = 24;
const ID_BROWSE: i32 = 25;
const ID_ARGS_MENU: i32 = 26;
const ID_DIR_MENU: i32 = 27;
const ID_OK: i32 = 1;
const ID_CANCEL: i32 = 2;

struct Dialog {
    tools: Vec<Tool>,
    list: HWND,
    /// Each row: its label and its box (and the button beside it, if any).
    rows: Vec<(HWND, HWND, Option<HWND>)>,
    title: HWND,
    command: HWND,
    args: HWND,
    dir: HWND,
    output: HWND,
    notes: [HWND; 2],
    buttons: Vec<(i32, HWND)>,
    loading: bool,
    ok: bool,
    done: bool,
}

fn state<'a>(h: HWND) -> Option<&'a mut Dialog> {
    unsafe { win::state::<Dialog>(h) }
}

/// Tools > External Tools: edits `tools`; the list as it is at OK.
pub fn edit_tools(owner: HWND, tools: Vec<Tool>) -> Option<Vec<Tool>> {
    let hwnd = make(owner, tools);
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
        let out = state(hwnd).filter(|d| d.ok).map(|d| d.tools.clone());
        win::enable(owner, true);
        let _ = wm::SetForegroundWindow(owner);
        let _ = wm::DestroyWindow(hwnd);
        out
    }
}

/// The window, filled, not yet shown - for [`edit_tools`] and for a picture.
pub fn make(owner: HWND, tools: Vec<Tool>) -> HWND {
    win::init();
    let class = win::register("PhaiExternalTools", Some(proc_), None);
    let caption = win::wide(t("External Tools"));
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_DLGMODALFRAME, PCWSTR(class.as_ptr()), PCWSTR(caption.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_THICKFRAME | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let list = win::list(hwnd, ID_LIST, font, &[(t("Menu contents"), 420)]);
        let mut rows = Vec::new();
        let mut row = |label: &str, b: HWND, side: Option<HWND>| {
            rows.push((win::label(hwnd, label, 0, font), b, side));
            b
        };
        let title = row(t("Title:"), win::edit(hwnd, "", ID_TITLE, font, false), None);
        let browse = win::button(hwnd, "...", ID_BROWSE, font, false);
        let command = row(t("Command:"), win::edit(hwnd, "", ID_COMMAND, font, false), Some(browse));
        let args_menu = win::button(hwnd, "›", ID_ARGS_MENU, font, false);
        let args = row(t("Arguments:"), win::edit(hwnd, "", ID_ARGS, font, false), Some(args_menu));
        let dir_menu = win::button(hwnd, "›", ID_DIR_MENU, font, false);
        let dir = row(t("Initial directory:"), win::edit(hwnd, "", ID_DIR, font, false), Some(dir_menu));
        let output = win::checkbox(hwnd, t("Use Output window"), ID_OUTPUT, font, true);
        let notes = [t("Each tool is an item on the Tools menu; what it prints goes to Output."), t("The open file is saved before a tool that uses it runs, and loaded again if the tool changes it.")].map(|words| {
            let h = win::label(hwnd, words, 0, font);
            win::set_css(h, "muted", "");
            h
        });
        let mut buttons = Vec::new();
        for (id, text, glyph) in [(ID_ADD, t("Add"), 0xE710u16), (ID_DELETE, t("Delete"), 0xE74D), (ID_UP, t("Move Up"), 0xE74A), (ID_DOWN, t("Move Down"), 0xE74B), (ID_OK, t("OK"), 0), (ID_CANCEL, t("Cancel"), 0)] {
            let b = win::button(hwnd, text, id, font, id == ID_OK);
            if glyph != 0 {
                win::set_button_glyph(b, glyph);
            }
            buttons.push((id, b));
        }
        win::set_state(hwnd, Box::new(Dialog { tools, list, rows, title, command, args, dir, output, notes, buttons, loading: false, ok: false, done: false }));
        fill_list(hwnd, 0);
        let (w, h) = (win::px(hwnd, 640), win::px(hwnd, 520));
        let _ = wm::SetWindowPos(hwnd, None, 0, 0, w, h, wm::SWP_NOZORDER | wm::SWP_NOMOVE);
        win::center_on(hwnd, owner);
        layout(hwnd);
        win::theme_frame(hwnd);
        hwnd
    }
}

fn chosen(d: &Dialog) -> Option<usize> {
    let i = unsafe { wm::SendMessageW(d.list, cc::LVM_GETNEXTITEM, Some(WPARAM(usize::MAX)), Some(LPARAM(2))).0 };
    usize::try_from(i).ok().filter(|i| *i < d.tools.len())
}

fn shown_title(tool: &Tool) -> String {
    if tool.title.trim().is_empty() { t("[New Tool]").to_string() } else { tool.title.replace('&', "") }
}

fn fill_list(h: HWND, at: usize) {
    let Some(d) = state(h) else { return };
    unsafe {
        wm::SendMessageW(d.list, cc::LVM_DELETEALLITEMS, None, None);
    }
    for tool in &d.tools {
        win::list_add(d.list, &[&shown_title(tool)]);
    }
    if !d.tools.is_empty() {
        win::list_select(d.list, at.min(d.tools.len() - 1));
    }
    load(h);
}

/// The chosen tool into the boxes.
fn load(h: HWND) {
    let Some(d) = state(h) else { return };
    d.loading = true;
    let at = chosen(d);
    let tool = at.and_then(|i| d.tools.get(i)).cloned().unwrap_or_default();
    for (_, b, side) in &d.rows {
        win::enable(*b, at.is_some());
        if let Some(s) = side {
            win::enable(*s, at.is_some());
        }
    }
    win::enable(d.output, at.is_some());
    win::set_text(d.title, &tool.title);
    win::set_text(d.command, &tool.command);
    win::set_text(d.args, &tool.args);
    win::set_text(d.dir, &tool.dir);
    win::set_checked(d.output, tool.output);
    d.loading = false;
}

/// The boxes into the chosen tool.
fn store(h: HWND) {
    let Some(d) = state(h) else { return };
    if d.loading {
        return;
    }
    let Some(i) = chosen(d) else { return };
    let tool = &mut d.tools[i];
    tool.title = win::text_of(d.title);
    tool.command = win::text_of(d.command).trim().to_string();
    tool.args = win::text_of(d.args);
    tool.dir = win::text_of(d.dir).trim().to_string();
    tool.output = win::is_checked(d.output);
    let shown = shown_title(tool);
    phai_ui::grid::set_cell(d.list, i, 0, &shown);
}

/// A placeholder chosen from the menu beside a box, put in at its caret.
fn offer_placeholders(h: HWND, into: HWND, dirs_only: bool) {
    let all = placeholders();
    let items: Vec<(u32, String)> = all.iter().enumerate().filter(|(_, (p, _))| !dirs_only || p.ends_with("Dir)")).map(|(k, (p, what))| (k as u32 + 1, format!("{p}\t{what}"))).collect();
    let refs: Vec<(u32, &str)> = items.iter().map(|(k, s)| (*k, s.as_str())).collect();
    let k = win::popup(h, &refs);
    if k == 0 {
        return;
    }
    let w = win::wide(all[k as usize - 1].0);
    unsafe {
        wm::SendMessageW(into, 0x00C2 /* EM_REPLACESEL */, Some(WPARAM(1)), Some(LPARAM(w.as_ptr() as isize)));
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(into));
    }
}

fn command(h: HWND, id: i32, code: u32) {
    let Some(d) = state(h) else { return };
    match id {
        ID_ADD => {
            if d.tools.len() >= MAX_TOOLS {
                win::message(Some(h), &tf("The Tools menu holds {0} tools at most.", &[&MAX_TOOLS]), t("External Tools"), wm::MB_ICONINFORMATION);
                return;
            }
            d.tools.push(Tool::default());
            let last = d.tools.len() - 1;
            fill_list(h, last);
            let _ = unsafe { windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(d.title)) };
        }
        ID_DELETE => {
            if let Some(i) = chosen(d) {
                d.tools.remove(i);
                fill_list(h, i.saturating_sub(usize::from(i == d.tools.len())));
            }
        }
        ID_UP | ID_DOWN => {
            if let Some(i) = chosen(d) {
                let j = if id == ID_UP { i.checked_sub(1) } else { (i + 1 < d.tools.len()).then_some(i + 1) };
                if let Some(j) = j {
                    d.tools.swap(i, j);
                    fill_list(h, j);
                }
            }
        }
        ID_BROWSE => {
            if let Some(p) = crate::file_dialog_pub(h, false, t("Programs"), "*.exe;*.cmd;*.bat;*.com", "") {
                win::set_text(d.command, &p.to_string_lossy());
                store(h);
            }
        }
        ID_ARGS_MENU => offer_placeholders(h, d.args, false),
        ID_DIR_MENU => offer_placeholders(h, d.dir, true),
        ID_OK => {
            store(h);
            d.ok = true;
            d.done = true;
        }
        ID_CANCEL => d.done = true,
        ID_TITLE | ID_COMMAND | ID_ARGS | ID_DIR if code == 0x0300 /* EN_CHANGE */ => store(h),
        ID_OUTPUT => store(h),
        _ => {}
    }
}

fn layout(h: HWND) {
    let Some(d) = state(h) else { return };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let px = |v| win::px(h, v);
    let (m, row, gap) = (px(12), px(24), px(6));
    let button = |id: i32| d.buttons.iter().find(|(i, _)| *i == id).map(|(_, b)| *b).unwrap_or_default();
    let bw = px(120);
    let by = r.bottom - m - row;
    // Below the list: the boxes, the tick, the note, then OK and Cancel.
    let under = d.rows.len() as i32 * (row + gap) + 3 * (row + gap);
    let list_bottom = (by - gap * 2 - under).max(m + px(80));
    let list_w = r.right - 2 * m - bw - gap;
    win::move_to(d.list, m, m, list_w, list_bottom - m);
    let mut y = m;
    for id in [ID_ADD, ID_DELETE, ID_UP, ID_DOWN] {
        win::move_to(button(id), r.right - m - bw, y, bw, row);
        y += row + gap;
    }
    let label_w = px(130);
    let side_w = px(30);
    let mut y = list_bottom + gap * 2;
    for (label, b, side) in &d.rows {
        win::move_to(*label, m, y, label_w, row);
        let box_w = r.right - m - (m + label_w) - side_w - gap;
        win::move_to(*b, m + label_w, y, box_w, row);
        if let Some(s) = side {
            win::move_to(*s, m + label_w + box_w + gap, y, side_w, row);
        }
        y += row + gap;
    }
    win::move_to(d.output, m + label_w, y, r.right - 2 * m - label_w, row);
    y += row + gap;
    for n in d.notes {
        win::move_to(n, m, y, r.right - 2 * m, row);
        y += row;
    }
    win::move_to(button(ID_CANCEL), r.right - m - px(90), by, px(90), row);
    win::move_to(button(ID_OK), r.right - m - px(90) * 2 - gap, by, px(90), row);
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
            mm.ptMinTrackSize = windows::Win32::Foundation::POINT { x: win::px(h, 520), y: win::px(h, 440) };
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.hwndFrom == d.list && hdr.code == cc::LVN_ITEMCHANGED {
                let nm = unsafe { &*(l.0 as *const cc::NMLISTVIEW) };
                if nm.uNewState & cc::LVIS_SELECTED.0 != 0 {
                    load(h);
                }
                return LRESULT(0);
            }
            win::def(h, msg, w, l)
        }
        wm::WM_COMMAND => {
            command(h, win::loword(w.0) as i32, win::hiword(w.0) as u32);
            LRESULT(0)
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

    fn places() -> Places {
        Places { file: Some(PathBuf::from(r"D:\work\app\src\main.rs")), project: Some(PathBuf::from(r"D:\work\app")), selected: "fn main".into() }
    }

    #[test]
    fn placeholders_are_put_in() {
        let p = places();
        assert_eq!(expand(r#""$(FilePath)""#, &p), r#""D:\work\app\src\main.rs""#);
        assert_eq!(expand("$(FileDir)|$(FileName)|$(ProjectDir)|$(SelectedText)", &p), r"D:\work\app\src|main.rs|D:\work\app|fn main");
        assert_eq!(expand("--edition 2021 $(filepath)", &p), r"--edition 2021 D:\work\app\src\main.rs", "capitals do not matter");
        assert_eq!(expand("$(ItemPath) $(SolutionDir)", &p), r"D:\work\app\src\main.rs D:\work\app", "Visual Studio's own names too");
        assert_eq!(expand("$(Nothing) $(FileName", &p), "$(Nothing) $(FileName", "unknown or unclosed: left as they are");
        assert_eq!(expand("cost $5 (each)", &p), "cost $5 (each)");
    }

    #[test]
    fn placeholders_with_nothing_open() {
        let none = Places::default();
        assert_eq!(expand(r#"x "$(FilePath)" $(ProjectDir)."#, &none), r#"x "" ."#);
        // No project: the file's own folder.
        let p = Places { project: None, ..places() };
        assert_eq!(expand("$(ProjectDir)", &p), r"D:\work\app\src");
    }

    #[test]
    fn a_tool_that_works_on_the_file() {
        let fmt = Tool { title: "rustfmt this file".into(), command: "rustfmt".into(), args: r#""$(FilePath)""#.into(), ..Tool::default() };
        assert!(uses_file(&fmt));
        assert!(!uses_file(&Tool { command: "cargo".into(), args: "clippy".into(), dir: "$(ProjectDir)".into(), ..Tool::default() }));
    }

    #[test]
    fn the_server_is_picked_by_the_file_s_extension() {
        let list = vec![
            Server { exts: ".toml".into(), command: "taplo lsp stdio".into() },
            Server { exts: ".sql, .PSQL".into(), command: "sqls".into() },
            Server { exts: "py;pyi".into(), command: "".into() },
            Server { exts: "*.py".into(), command: "pyright-langserver --stdio".into() },
            Server { exts: ".rs".into(), command: "my-rust-server".into() },
        ];
        let pick = |p: &str| server_for(&list, Path::new(p)).map(|s| s.command.as_str());
        assert_eq!(pick(r"D:\app\Cargo.toml"), Some("taplo lsp stdio"));
        assert_eq!(pick(r"D:\app\db\q.SQL"), Some("sqls"), "capitals do not matter");
        assert_eq!(pick(r"D:\app\db\q.psql"), Some("sqls"));
        assert_eq!(pick(r"D:\app\x.py"), Some("pyright-langserver --stdio"), "one with no command passed over");
        assert_eq!(pick(r"D:\app\src\main.rs"), None, "Rust is rust-analyzer's");
        assert_eq!(pick(r"D:\app\README"), None);
        assert_eq!(pick(r"D:\app\notes.md"), None);
    }

    #[test]
    fn the_lists_are_saved_and_read_back() {
        let servers = vec![Server { exts: ".toml".into(), command: "taplo lsp stdio".into() }, Server { exts: "".into(), command: "x".into() }];
        let text = servers_text(&servers);
        assert!(!text.contains('\n'), "one line in settings.txt");
        assert_eq!(parse_servers(Some(&text)), servers[..1], "one with no extension left out");
        assert!(parse_servers(Some("not json")).is_empty() && parse_servers(None).is_empty());
        let tools = vec![Tool { title: "&Format".into(), command: "rustfmt".into(), args: r#""$(FilePath)""#.into(), dir: "$(FileDir)".into(), output: false }];
        assert_eq!(parse_tools(Some(&tools_text(&tools))), tools);
        // Saved before `output` was: to Output.
        assert!(parse_tools(Some(r#"[{"title":"a","command":"b"}]"#))[0].output);
    }

    #[test]
    fn a_tool_s_run_is_worked_out() {
        let tool = Tool { title: "Echo".into(), command: "cmd".into(), args: r#"/c echo "$(FileName)""#.into(), ..Tool::default() };
        let run = prepare(&tool, &places()).unwrap();
        assert!(run.program.is_file());
        assert_eq!(run.args, r#"/c echo "main.rs""#);
        assert_eq!(run.dir, PathBuf::from(r"D:\work\app"), "no folder given: the project's");
        assert!(prepare(&Tool { command: "no-such-program-here".into(), ..tool.clone() }, &places()).is_err());
    }
}
