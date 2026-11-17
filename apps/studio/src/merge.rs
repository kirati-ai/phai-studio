//! The Merge Editor, as Visual Studio's: a file Git could not merge by itself, its conflicts one
//! after another - what this branch has (Current) beside what came in (Incoming) - each taken as
//! one, the other or both; Accept Merge writes the file and stages it.

use std::path::{Path, PathBuf};

use phai_ui::text::{t, tf};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::gitpane::git;

const ID_CURRENT: i32 = 11;
const ID_INCOMING: i32 = 12;
const ID_RESULT: i32 = 13;
const ID_PREV: i32 = 20;
const ID_NEXT: i32 = 21;
const ID_TAKE_CURRENT: i32 = 22;
const ID_TAKE_INCOMING: i32 = 23;
const ID_TAKE_BOTH: i32 = 24;
const ID_ACCEPT: i32 = 1;
const ID_CANCEL: i32 = 2;

/// A part of a file with conflicts: lines both sides agree on, or a conflict.
#[derive(Clone, Debug, PartialEq)]
pub enum Part {
    Same(Vec<String>),
    Conflict { current: Vec<String>, incoming: Vec<String>, current_name: String, incoming_name: String },
}

/// How a conflict was settled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Take {
    Current,
    Incoming,
    Both,
}

/// A file's text read into what agrees and what conflicts, by Git's `<<<<<<<`, `=======` and
/// `>>>>>>>` lines (a `|||||||` base section, from diff3, is left out).
pub fn parts_of(text: &str) -> Vec<Part> {
    let mut parts = Vec::new();
    let mut same = Vec::new();
    let mut lines = text.lines();
    while let Some(l) = lines.next() {
        let Some(name) = l.strip_prefix("<<<<<<<") else {
            same.push(l.to_string());
            continue;
        };
        if !same.is_empty() {
            parts.push(Part::Same(std::mem::take(&mut same)));
        }
        let (mut current, mut incoming) = (Vec::new(), Vec::new());
        let mut side = 0; // 0 current, 1 base (skipped), 2 incoming
        let mut incoming_name = String::new();
        for l in lines.by_ref() {
            if l.starts_with("|||||||") {
                side = 1;
            } else if l == "=======" {
                side = 2;
            } else if let Some(n) = l.strip_prefix(">>>>>>>") {
                incoming_name = n.trim().to_string();
                break;
            } else if side == 0 {
                current.push(l.to_string());
            } else if side == 2 {
                incoming.push(l.to_string());
            }
        }
        parts.push(Part::Conflict { current, incoming, current_name: name.trim().to_string(), incoming_name });
    }
    if !same.is_empty() {
        parts.push(Part::Same(same));
    }
    parts
}

/// How many conflicts there are.
pub fn conflicts(parts: &[Part]) -> usize {
    parts.iter().filter(|p| matches!(p, Part::Conflict { .. })).count()
}

/// The file as settled: each conflict as taken, one not yet taken still marked as Git marks it.
pub fn merged(parts: &[Part], takes: &[Option<Take>]) -> String {
    let mut out = Vec::new();
    let mut i = 0;
    for p in parts {
        match p {
            Part::Same(l) => out.extend(l.iter().cloned()),
            Part::Conflict { current, incoming, current_name, incoming_name } => {
                match takes.get(i).copied().flatten() {
                    Some(Take::Current) => out.extend(current.iter().cloned()),
                    Some(Take::Incoming) => out.extend(incoming.iter().cloned()),
                    Some(Take::Both) => out.extend(current.iter().chain(incoming).cloned()),
                    None => {
                        out.push(format!("<<<<<<< {current_name}"));
                        out.extend(current.iter().cloned());
                        out.push("=======".into());
                        out.extend(incoming.iter().cloned());
                        out.push(format!(">>>>>>> {incoming_name}"));
                    }
                }
                i += 1;
            }
        }
    }
    let mut s = out.join("\n");
    s.push('\n');
    s
}

/// The files of a repository Git left with conflicts.
pub fn conflicted(repo: &Path) -> Vec<PathBuf> {
    let (_, out) = git(repo, &["diff", "--name-only", "--diff-filter=U"]);
    out.lines().filter(|l| !l.trim().is_empty()).map(|l| repo.join(l.trim())).collect()
}

struct Dialog {
    file: PathBuf,
    repo: PathBuf,
    parts: Vec<Part>,
    takes: Vec<Option<Take>>,
    at: usize,
    current: HWND,
    incoming: HWND,
    result: HWND,
    labels: Vec<HWND>,
    buttons: Vec<(i32, HWND)>,
    accepted: bool,
    done: bool,
}

fn state<'a>(h: HWND) -> Option<&'a mut Dialog> {
    unsafe { win::state::<Dialog>(h) }
}

/// Settles `takes` without asking, writes the file and stages it - what Accept Merge does.
pub fn accept(repo: &Path, file: &Path, parts: &[Part], takes: &[Option<Take>]) -> Result<(), String> {
    std::fs::write(file, merged(parts, takes)).map_err(|e| e.to_string())?;
    let (ok, out) = git(repo, &["add", "--", &file.to_string_lossy()]);
    if ok {
        Ok(())
    } else {
        Err(out)
    }
}

/// Opens the Merge Editor on `file`; whether the merge was accepted.
pub fn open(owner: HWND, repo: &Path, file: &Path) -> bool {
    let text = std::fs::read_to_string(file).unwrap_or_default();
    let parts = parts_of(&text);
    let n = conflicts(&parts);
    if n == 0 {
        win::message(Some(owner), t("This file has no merge conflicts."), t("Merge Editor"), wm::MB_ICONINFORMATION);
        return false;
    }
    // `PHAI_MERGE_TAKE=current,incoming,both`: settled so, without the window - for testing.
    if let Ok(list) = std::env::var("PHAI_MERGE_TAKE") {
        let takes: Vec<Option<Take>> = list
            .split(',')
            .map(|w| match w.trim() {
                "current" => Some(Take::Current),
                "incoming" => Some(Take::Incoming),
                "both" => Some(Take::Both),
                _ => None,
            })
            .collect();
        return accept(repo, file, &parts, &takes).is_ok();
    }
    win::init();
    let class = win::register("PhaiMerge", Some(proc_), None);
    let title = win::wide(&tf("Merge Editor - {0}", &[&file.file_name().unwrap_or_default().to_string_lossy()]));
    unsafe {
        let hwnd = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_THICKFRAME | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let bold = win::font(hwnd, 0, true);
        let current = win::edit_lines(hwnd, "", ID_CURRENT, font);
        let incoming = win::edit_lines(hwnd, "", ID_INCOMING, font);
        let result = win::edit_lines(hwnd, "", ID_RESULT, font);
        for e in [current, incoming, result] {
            wm::SendMessageW(e, 0x00CF /* EM_SETREADONLY */, Some(WPARAM(1)), None);
        }
        let labels = vec![win::label(hwnd, "", 0, bold), win::label(hwnd, "", 0, bold), win::label(hwnd, t("Result"), 0, bold), win::label(hwnd, "", 0, font)];
        let mut buttons = Vec::new();
        for (id, label) in [
            (ID_PREV, t("Previous")),
            (ID_NEXT, t("Next")),
            (ID_TAKE_CURRENT, t("Take Current")),
            (ID_TAKE_INCOMING, t("Take Incoming")),
            (ID_TAKE_BOTH, t("Take Both")),
            (ID_ACCEPT, t("Accept Merge")),
            (ID_CANCEL, t("Cancel")),
        ] {
            buttons.push((id, win::button(hwnd, label, id, font, id == ID_ACCEPT)));
        }
        win::set_state(hwnd, Box::new(Dialog { file: file.to_path_buf(), repo: repo.to_path_buf(), takes: vec![None; n], parts, at: 0, current, incoming, result, labels, buttons, accepted: false, done: false }));
        show(hwnd);
        let (w, h) = (win::px(hwnd, 1000), win::px(hwnd, 700));
        let mut r = RECT::default();
        let _ = wm::GetWindowRect(owner, &mut r);
        let _ = wm::SetWindowPos(hwnd, None, r.left + (r.right - r.left - w) / 2, r.top + (r.bottom - r.top - h) / 2, w, h, wm::SWP_NOZORDER);
        layout(hwnd);
        win::theme_frame(hwnd);
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
        let accepted = state(hwnd).is_some_and(|d| d.accepted);
        win::enable(owner, true);
        let _ = wm::SetForegroundWindow(owner);
        let _ = wm::DestroyWindow(hwnd);
        accepted
    }
}

/// The conflict looked at, its two sides, the file as it would be, and how many are left.
fn show(h: HWND) {
    let Some(d) = state(h) else { return };
    let Some(Part::Conflict { current, incoming, current_name, incoming_name }) = d.parts.iter().filter(|p| matches!(p, Part::Conflict { .. })).nth(d.at) else { return };
    win::set_text(d.current, &current.join("\r\n"));
    win::set_text(d.incoming, &incoming.join("\r\n"));
    win::set_text(d.labels[0], &tf("Current ({0})", &[current_name]));
    win::set_text(d.labels[1], &tf("Incoming ({0})", &[incoming_name]));
    win::set_text(d.result, &merged(&d.parts, &d.takes).replace('\n', "\r\n"));
    let left = d.takes.iter().filter(|t| t.is_none()).count();
    let taken = match d.takes[d.at] {
        Some(Take::Current) => t("Current taken"),
        Some(Take::Incoming) => t("Incoming taken"),
        Some(Take::Both) => t("Both taken"),
        None => t("Not yet settled"),
    };
    win::set_text(d.labels[3], &tf("Conflict {0} of {1} - {2}; {3} left", &[&(d.at + 1), &d.takes.len(), &taken, &left]));
    if let Some((_, b)) = d.buttons.iter().find(|(i, _)| *i == ID_ACCEPT) {
        win::enable(*b, left == 0);
    }
}

fn take(h: HWND, how: Take) {
    let Some(d) = state(h) else { return };
    d.takes[d.at] = Some(how);
    // On to the next one not yet settled.
    if let Some(next) = (0..d.takes.len()).map(|k| (d.at + 1 + k) % d.takes.len()).find(|k| d.takes[*k].is_none()) {
        d.at = next;
    }
    show(h);
}

fn layout(h: HWND) {
    let Some(d) = state(h) else { return };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut r);
    }
    let px = |v| win::px(h, v);
    let (m, row, bw, gap) = (px(11), px(26), px(110), px(6));
    let button = |id: i32| d.buttons.iter().find(|(i, _)| *i == id).map(|(_, b)| *b).unwrap_or_default();
    let half = (r.right - 3 * m) / 2;
    let side_h = (r.bottom - 4 * m - 4 * row - 2 * gap) * 2 / 5;
    win::move_to(d.labels[0], m, m, half, row);
    win::move_to(d.labels[1], 2 * m + half, m, half, row);
    win::move_to(d.current, m, m + row, half, side_h);
    win::move_to(d.incoming, 2 * m + half, m + row, half, side_h);
    let by = m + row + side_h + gap;
    win::move_to(button(ID_TAKE_CURRENT), m, by, bw, row);
    win::move_to(button(ID_TAKE_BOTH), m + bw + gap, by, bw, row);
    win::move_to(button(ID_TAKE_INCOMING), 2 * m + half, by, bw, row);
    win::move_to(button(ID_PREV), r.right - m - 2 * bw - gap, by, bw, row);
    win::move_to(button(ID_NEXT), r.right - m - bw, by, bw, row);
    let ry = by + row + m;
    win::move_to(d.labels[2], m, ry, r.right - 2 * m, row);
    win::move_to(d.result, m, ry + row, r.right - 2 * m, r.bottom - ry - row - 2 * m - row);
    let fy = r.bottom - m - row;
    win::move_to(d.labels[3], m, fy, r.right - 2 * bw - 4 * m, row);
    win::move_to(button(ID_ACCEPT), r.right - m - 2 * bw - gap, fy, bw, row);
    win::move_to(button(ID_CANCEL), r.right - m - bw, fy, bw, row);
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
            mm.ptMinTrackSize = POINT { x: win::px(h, 700), y: win::px(h, 480) };
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            match win::loword(w.0) as i32 {
                ID_TAKE_CURRENT => take(h, Take::Current),
                ID_TAKE_INCOMING => take(h, Take::Incoming),
                ID_TAKE_BOTH => take(h, Take::Both),
                ID_PREV => {
                    d.at = (d.at + d.takes.len() - 1) % d.takes.len();
                    show(h);
                }
                ID_NEXT => {
                    d.at = (d.at + 1) % d.takes.len();
                    show(h);
                }
                ID_ACCEPT => match accept(&d.repo, &d.file, &d.parts, &d.takes) {
                    Ok(()) => {
                        d.accepted = true;
                        d.done = true;
                    }
                    Err(e) => {
                        win::message(Some(h), &e, t("Merge Editor"), wm::MB_ICONWARNING);
                    }
                },
                ID_CANCEL => d.done = true,
                _ => return win::def(h, msg, w, l),
            }
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            d.done = true;
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conflicts_are_read_and_settled() {
        let text = "fn main() {\n<<<<<<< HEAD\n    let x = 1;\n=======\n    let x = 2;\n>>>>>>> feature\n    println!(\"{x}\");\n<<<<<<< HEAD\na\n||||||| base\nb\n=======\nc\n>>>>>>> feature\n}\n";
        let parts = parts_of(text);
        assert_eq!(conflicts(&parts), 2);
        assert_eq!(merged(&parts, &[Some(Take::Incoming), Some(Take::Both)]), "fn main() {\n    let x = 2;\n    println!(\"{x}\");\na\nc\n}\n");
        // One left unsettled stays marked, without the base.
        assert!(merged(&parts, &[Some(Take::Current), None]).contains("<<<<<<< HEAD\na\n=======\nc\n>>>>>>> feature"));
    }
}
