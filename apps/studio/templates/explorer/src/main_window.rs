//! MainWindow: an Explorer-style window. Folders on the left, in a tree that reads each folder
//! only when it is opened; the chosen folder's folders and files on the right, with their size
//! and the day they were last changed. The window itself is in main_window_ui.rs, which Phai
//! Studio writes from main_window.phf.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::main_window_ui::{MainWindowEvents, MainWindowUi};
use phai_ui::ctl::Node;
use phai_ui::prelude::*;

/// A folder shown in the tree.
struct Folder {
    path: PathBuf,
    /// The folder above it in the tree; `None` for a top one.
    parent: Option<Node>,
    /// Whether the folders inside it have been read and put under it.
    loaded: bool,
}

/// A row of the list on the right.
struct Entry {
    path: PathBuf,
    is_folder: bool,
}

#[derive(Default)]
pub struct MainWindow {
    /// Every folder in the tree, by its item there.
    folders: HashMap<Node, Folder>,
    /// The folder chosen in the tree.
    current: Option<Node>,
    /// The list's rows, in the order shown.
    entries: Vec<Entry>,
}

impl MainWindowEvents for MainWindow {
    fn loaded(&mut self, ui: &MainWindowUi) {
        // A folder picture before every item of the tree.
        phai_ui::win::tree_icons(ui.folders.hwnd(), &[phai_ui::icon::glyph::FOLDER]);

        // The top folders: the user's Documents, and the folder the program was started in.
        let documents = std::env::var_os("USERPROFILE").map(|home| PathBuf::from(home).join("Documents"));
        let program = std::env::current_dir().ok();
        let mut first = None;
        for (name, path) in [(t("Documents"), documents), (t("Program folder"), program)] {
            let Some(path) = path.filter(|p| p.is_dir()) else { continue };
            let node = self.add_folder(ui, None, name, path);
            // Its folders read now, so it has its open arrow.
            self.load_children(ui, node);
            first.get_or_insert(node);
        }
        // The first one chosen, and what it holds listed.
        if let Some(node) = first {
            ui.folders.select(node);
            self.folder_chosen(ui);
        }
    }

    /// Another folder chosen in the tree: read the folders under it, and list what it holds.
    fn folder_chosen(&mut self, ui: &MainWindowUi) {
        let Some(node) = ui.folders.selected_node() else { return };
        self.current = Some(node);
        self.load_children(ui, node);
        // One level further, so the folders just shown have their open arrow when they hold more.
        let children: Vec<Node> = self.folders.iter().filter(|(_, f)| f.parent == Some(node)).map(|(n, _)| *n).collect();
        for child in children {
            self.load_children(ui, child);
        }
        ui.folders.expand(node);
        self.fill_list(ui);
    }

    /// A row double-clicked (or File > Open): a folder opens in the tree, a file
    /// opens in the program Windows opens it with.
    fn open_item(&mut self, ui: &MainWindowUi) {
        let Some(row) = ui.files.selected_index() else { return };
        let Some(entry) = self.entries.get(row) else { return };
        if !entry.is_folder {
            open_with_windows(&entry.path.to_string_lossy());
            return;
        }
        let target = entry.path.clone();
        let found = self.folders.iter().find(|(_, f)| f.parent == self.current && f.path == target).map(|(n, _)| *n);
        if let Some(node) = found {
            ui.folders.select(node);
        }
    }

    /// Up one level: the folder above the chosen one.
    fn go_up(&mut self, ui: &MainWindowUi) {
        let parent = self.current.and_then(|n| self.folders.get(&n)).and_then(|f| f.parent);
        if let Some(node) = parent {
            ui.folders.select(node);
        }
    }

    /// F5: the chosen folder read again.
    fn refresh(&mut self, ui: &MainWindowUi) {
        self.fill_list(ui);
    }

    fn exit(&mut self, ui: &MainWindowUi) {
        ui.close();
    }
}

impl MainWindow {
    /// Puts a folder in the tree under `parent` (or at the top) and remembers its path.
    fn add_folder(&mut self, ui: &MainWindowUi, parent: Option<Node>, name: &str, path: PathBuf) -> Node {
        // Picture 0 (the folder) of the tree's icons.
        let node = Node(phai_ui::win::tree_add(ui.folders.hwnd(), parent.map_or(0, |p| p.0), name, 0, 0));
        self.folders.insert(node, Folder { path, parent, loaded: false });
        node
    }

    /// Reads the folders inside `node`'s folder and puts them under it - once.
    fn load_children(&mut self, ui: &MainWindowUi, node: Node) {
        let Some(folder) = self.folders.get_mut(&node) else { return };
        if folder.loaded {
            return;
        }
        folder.loaded = true;
        let path = folder.path.clone();
        let (subfolders, _) = read_folder(&path);
        for sub in subfolders {
            let name = file_name(&sub);
            self.add_folder(ui, Some(node), &name, sub);
        }
    }

    /// Fills the list with what the chosen folder holds: its folders first, then its files.
    fn fill_list(&mut self, ui: &MainWindowUi) {
        let Some(path) = self.current.and_then(|n| self.folders.get(&n)).map(|f| f.path.clone()) else { return };
        ui.files.begin_update();
        ui.files.items_clear();
        self.entries.clear();
        let (subfolders, files) = read_folder(&path);
        for sub in &subfolders {
            let modified = std::fs::metadata(sub).ok().and_then(|m| m.modified().ok());
            ui.files.items_add_with_icon(&[&file_name(sub), "", &date_text(modified)], 0);
            self.entries.push(Entry { path: sub.clone(), is_folder: true });
        }
        for file in &files {
            let meta = std::fs::metadata(file).ok();
            let size = meta.as_ref().map_or(String::new(), |m| size_text(m.len()));
            let modified = meta.and_then(|m| m.modified().ok());
            ui.files.items_add_with_icon(&[&file_name(file), &size, &date_text(modified)], 1);
            self.entries.push(Entry { path: file.clone(), is_folder: false });
        }
        ui.files.end_update();
        // The folder's name in the title bar, how many things it holds in the status bar.
        if let Some(node) = self.current {
            ui.set_title(&tf("{0} - Explorer", &[&ui.folders.node_text(node)]));
        }
        ui.status.set_part_text(0, &tf("{0} items", &[&self.entries.len()]));
    }
}

/// The folders and the files in `dir`, each sorted by name; hidden and system ones left out,
/// as Explorer leaves them out. A folder that cannot be read gives nothing.
fn read_folder(dir: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    use std::os::windows::fs::MetadataExt;
    const HIDDEN: u32 = 0x2;
    const SYSTEM: u32 = 0x4;
    let (mut folders, mut files) = (Vec::new(), Vec::new());
    let Ok(read) = std::fs::read_dir(dir) else { return (folders, files) };
    for entry in read.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        if meta.file_attributes() & (HIDDEN | SYSTEM) != 0 {
            continue;
        }
        if meta.is_dir() {
            folders.push(entry.path());
        } else {
            files.push(entry.path());
        }
    }
    let by_name = |a: &PathBuf, b: &PathBuf| file_name(a).to_lowercase().cmp(&file_name(b).to_lowercase());
    folders.sort_by(by_name);
    files.sort_by(by_name);
    (folders, files)
}

/// The last part of a path: the file's or the folder's own name.
fn file_name(p: &Path) -> String {
    p.file_name().map_or_else(|| p.display().to_string(), |n| n.to_string_lossy().into_owned())
}

/// A file's size as Explorer shows it: in KB, rounded up - "1 KB", "2,048 KB".
fn size_text(bytes: u64) -> String {
    let kb = bytes.div_ceil(1024).to_string();
    // A comma between every three figures.
    let mut grouped = String::new();
    for (i, c) in kb.chars().enumerate() {
        if i > 0 && (kb.len() - i).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(c);
    }
    // KB reads the same in Thai.
    format!("{grouped} KB")
}

/// When a file was last changed, in this computer's time: "23/11/2026 21:30".
fn date_text(time: Option<SystemTime>) -> String {
    let Some(secs) = time.and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok()).map(|d| d.as_secs() as i64) else {
        return String::new();
    };
    let local = secs + local_offset_secs();
    let (days, rest) = (local.div_euclid(86_400), local.rem_euclid(86_400));
    let (year, month, day) = civil_from_days(days);
    let when = DateTime { year, month, day, hour: (rest / 3600) as u32, minute: (rest / 60 % 60) as u32, ..Default::default() };
    when.format("dd/MM/yyyy HH:mm")
}

/// How far this computer's clock is ahead of UTC, in seconds - its time now less the time in
/// UTC, to the quarter hour.
fn local_offset_secs() -> i64 {
    let n = now();
    let local = days_from_civil(n.year, n.month, n.day) * 86_400 + (n.hour * 3600 + n.minute * 60 + n.second) as i64;
    let utc = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
    ((local - utc) as f64 / 900.0).round() as i64 * 900
}

/// Days since 1 January 1970 for a date (Howard Hinnant's method).
fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y } as i64;
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m as i64 + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The date of a day counted from 1 January 1970.
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    (y as i32, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_and_sizes_read_as_explorer_shows_them() {
        assert_eq!(civil_from_days(days_from_civil(2026, 11, 25)), (2026, 11, 25));
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(size_text(1), "1 KB");
        assert_eq!(size_text(2048 * 1024), "2,048 KB");
    }
}
