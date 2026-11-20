//! Compare Files and Class View, as Visual Studio has them. Compare Files: two files side by
//! side, each in an editor, the lines only the left has in red, the right's new ones in green and
//! changed ones in blue - in the margin and on the scroll bar map. Class View: the solution's
//! types, traits and modules in a tree, each type's methods under it from its `impl` blocks, the
//! free functions apart; a double-click opens the file there.

use std::path::{Path, PathBuf};

use phai_ui::text::{t, tf};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging as wm;

// --- Compare Files -------------------------------------------------------------------------------

struct Compare {
    left: HWND,
    right: HWND,
    left_name: HWND,
    right_name: HWND,
}

/// Two files side by side, how they differ marked.
pub fn compare(owner: HWND, a: &Path, b: &Path) {
    let (Ok(ta), Ok(tb)) = (std::fs::read_to_string(a), std::fs::read_to_string(b)) else {
        win::message(Some(owner), t("One of the files cannot be read."), t("Compare Files"), wm::MB_ICONWARNING);
        return;
    };
    let (ta, tb) = (ta.replace("\r\n", "\n"), tb.replace("\r\n", "\n"));
    win::init();
    let class = win::register("PhaiCompare", Some(compare_proc), None);
    let title = win::wide(&tf("Compare: {0} - {1}", &[&a.file_name().unwrap_or_default().to_string_lossy(), &b.file_name().unwrap_or_default().to_string_lossy()]));
    unsafe {
        let hwnd = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_OVERLAPPEDWINDOW | wm::WS_CLIPCHILDREN, wm::CW_USEDEFAULT, wm::CW_USEDEFAULT, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let left_name = win::label(hwnd, &a.display().to_string(), 0, font);
        let right_name = win::label(hwnd, &b.display().to_string(), 0, font);
        let left = phai_editor::view::create(hwnd, 10, &ta, &a.to_string_lossy());
        let right = phai_editor::view::create(hwnd, 11, &tb, &b.to_string_lossy());
        let (l, r) = phai_editor::tools::diff_lines(&ta, &tb);
        let differ = l.len() + r.len();
        phai_editor::view::set_changes(left, &l);
        phai_editor::view::set_changes(right, &r);
        win::set_state(hwnd, Box::new(Compare { left, right, left_name, right_name }));
        let mut o = RECT::default();
        let _ = wm::GetWindowRect(owner, &mut o);
        let (w, h) = (((o.right - o.left) * 9 / 10).max(win::px(hwnd, 800)), ((o.bottom - o.top) * 9 / 10).max(win::px(hwnd, 500)));
        let _ = wm::SetWindowPos(hwnd, None, o.left + (o.right - o.left - w) / 2, o.top + (o.bottom - o.top - h) / 2, w, h, wm::SWP_NOZORDER);
        win::theme_frame(hwnd);
        win::show_top(hwnd);
        if differ == 0 {
            win::message(Some(hwnd), t("The two files are the same."), t("Compare Files"), wm::MB_ICONINFORMATION);
        }
    }
}

extern "system" fn compare_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(c) = (unsafe { win::state::<Compare>(h) }) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            let (cw, ch) = (win::loword(l.0 as usize) as i32, win::hiword(l.0 as usize) as i32);
            let head = win::px(h, 26);
            let half = cw / 2;
            win::move_to(c.left_name, win::px(h, 8), 0, half - win::px(h, 12), head);
            win::move_to(c.right_name, half + win::px(h, 4), 0, half - win::px(h, 12), head);
            win::move_to(c.left, 0, head, half - 2, ch - head);
            win::move_to(c.right, half + 2, head, cw - half - 2, ch - head);
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            drop(unsafe { win::take_state::<Compare>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::ctl_color(msg, w, l).unwrap_or_else(|| win::def(h, msg, w, l)),
    }
}

// --- Class View ----------------------------------------------------------------------------------

/// A place Class View leads to.
#[derive(Clone, Debug, PartialEq)]
pub struct Place {
    pub path: PathBuf,
    pub line: usize,
}

/// The tree's nodes for `files`: each type, trait or module with its file and line, and its
/// methods under it; the free functions under one node of their own. Sorted by name.
/// A node of Class View: its name, where it is declared (when it is), and its methods.
pub type Node = (String, Option<Place>, Vec<(String, Place)>);

pub fn class_tree(files: &[(PathBuf, String)]) -> Vec<Node> {
    let mut types: Vec<Node> = Vec::new();
    let mut free: Vec<(String, Place)> = Vec::new();
    for (path, text) in files {
        // A macro's `$name` is not a type of the program.
        for it in phai_editor::crumbs::items(text).into_iter().filter(|i| !i.name.starts_with('$') && i.owner.as_deref().is_none_or(|o| !o.starts_with('$'))) {
            let place = Place { path: path.clone(), line: it.line };
            match (it.kind.as_str(), it.owner) {
                ("fn", Some(owner)) => match types.iter_mut().find(|(n, _, _)| *n == owner) {
                    Some((_, _, methods)) => methods.push((it.name, place)),
                    None => types.push((owner, None, vec![(it.name, place)])),
                },
                ("fn", None) => free.push((format!("{}  ({})", it.name, path.file_name().unwrap_or_default().to_string_lossy()), place)),
                (kind, _) => {
                    let label = format!("{kind} {}", it.name);
                    match types.iter_mut().find(|(n, _, _)| *n == it.name) {
                        Some(t) => {
                            t.0 = it.name.clone();
                            t.1.get_or_insert(place);
                        }
                        None => types.push((it.name.clone(), Some(place), vec![])),
                    }
                    let _ = label;
                }
            }
        }
    }
    types.sort_by_key(|a| a.0.to_lowercase());
    for t in &mut types {
        t.2.sort_by_key(|a| a.0.to_lowercase());
    }
    free.sort_by_key(|a| a.0.to_lowercase());
    if !free.is_empty() {
        types.push((t("(functions)").to_string(), None, free));
    }
    types
}

struct ClassView {
    tree: HWND,
    places: Vec<Place>,
}

/// Class View over `owner` for the `.rs` files under `root`.
pub fn class_view(owner: HWND, root: &Path) {
    let mut files = Vec::new();
    let mut dirs = vec![root.to_path_buf()];
    while let Some(d) = dirs.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for p in rd.flatten().map(|e| e.path()) {
            let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
            if p.is_dir() {
                if !name.starts_with('.') && name != "target" {
                    dirs.push(p);
                }
            } else if p.extension().is_some_and(|e| e == "rs") {
                if let Ok(text) = std::fs::read_to_string(&p) {
                    files.push((p, text));
                }
            }
        }
    }
    let nodes = class_tree(&files);
    win::init();
    let class = win::register("PhaiClassView", Some(class_proc), None);
    let title = win::wide(&tf("Class View - {0}", &[&root.file_name().unwrap_or_default().to_string_lossy()]));
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_TOOLWINDOW, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_THICKFRAME | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let tree = win::tree(hwnd, 10, font);
        let mut places = Vec::new();
        for (name, place, methods) in &nodes {
            let data = place.as_ref().map(|p| {
                places.push(p.clone());
                places.len() as isize - 1
            });
            let node = win::tree_add(tree, 0, name, -1, data.unwrap_or(-1));
            for (m, p) in methods {
                places.push(p.clone());
                win::tree_add(tree, node, m, -1, places.len() as isize - 1);
            }
        }
        win::set_state(hwnd, Box::new(ClassView { tree, places }));
        let mut o = RECT::default();
        let _ = wm::GetWindowRect(owner, &mut o);
        let (w, h) = (win::px(hwnd, 380), (o.bottom - o.top) * 3 / 4);
        let _ = wm::SetWindowPos(hwnd, None, o.right - w - win::px(hwnd, 40), o.top + win::px(hwnd, 80), w, h, wm::SWP_NOZORDER);
        win::theme_frame(hwnd);
        win::show_top(hwnd);
    }
}

extern "system" fn class_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(c) = (unsafe { win::state::<ClassView>(h) }) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            win::move_to(c.tree, 0, 0, win::loword(l.0 as usize) as i32, win::hiword(l.0 as usize) as i32);
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            use windows::Win32::UI::Controls as cc;
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.code == cc::NM_CUSTOMDRAW {
                return win::tree_custom_draw(l);
            }
            if hdr.code == cc::NM_DBLCLK {
                if let Some(p) = win::tree_selected(c.tree).and_then(|i| usize::try_from(i).ok()).and_then(|i| c.places.get(i)).cloned() {
                    if let Some(s) = crate::studio() {
                        s.open_code(&p.path);
                        if let Some(e) = s.doc.code {
                            phai_editor::view::go_to(e, p.line + 1, 1);
                        }
                    }
                }
                return LRESULT(1);
            }
            win::def(h, msg, w, l)
        }
        wm::WM_NCDESTROY => {
            drop(unsafe { win::take_state::<ClassView>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types_hold_their_methods_free_functions_apart() {
        let files = vec![
            (PathBuf::from("a.rs"), "struct Cart;\nimpl Cart {\n    fn total(&self) {}\n}\nfn main() {}\n".to_string()),
            (PathBuf::from("b.rs"), "impl Display for Cart {\n    fn fmt(&self) {}\n}\nenum Size { S, M }\n".to_string()),
        ];
        let tree = class_tree(&files);
        let cart = tree.iter().find(|n| n.0 == "Cart").unwrap();
        assert_eq!(cart.1.as_ref().map(|p| p.line), Some(0));
        assert_eq!(cart.2.iter().map(|m| m.0.as_str()).collect::<Vec<_>>(), vec!["fmt", "total"]);
        assert!(tree.iter().any(|n| n.0 == "Size"));
        assert!(tree.last().unwrap().2.iter().any(|m| m.0.starts_with("main")));
    }
}
