//! A grid of cells to read and change, as WinForms' DataGridView, and a rich text box, as its
//! RichTextBox. The grid is Windows' list view with a box that opens over a cell on a
//! double-click (or F2, or typing); Enter keeps what is typed, Escape leaves the cell as it was.
//! A double-click under the last row adds a row. Its parent hears [`crate::win::VALUE_CHANGED`].
//! The rich text box is Windows' own RichEdit (Msftedit.dll).

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::System::LibraryLoader::LoadLibraryW;
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::win;

const EDITOR_ID: i32 = 0x7E01;

/// A grid with these columns (heading, width in design pixels); no rows yet.
pub fn grid(parent: HWND, id: i32, f: windows::Win32::Graphics::Gdi::HFONT, columns: &[(&str, i32)]) -> HWND {
    let h = win::list(parent, id, f, columns);
    const LVS_EX_GRIDLINES: isize = 0x1;
    const LVS_EX_FULLROWSELECT: isize = 0x20;
    const LVS_EX_DOUBLEBUFFER: isize = 0x10000;
    unsafe {
        wm::SendMessageW(h, cc::LVM_SETEXTENDEDLISTVIEWSTYLE, Some(WPARAM(0)), Some(LPARAM(LVS_EX_GRIDLINES | LVS_EX_FULLROWSELECT | LVS_EX_DOUBLEBUFFER)));
        let _ = SetWindowSubclass(h, Some(grid_proc), 7, 0);
    }
    h
}

/// The grid's cells, row by row.
pub fn grid_rows(h: impl Into<HWND>) -> Vec<Vec<String>> {
    let h: HWND = h.into();
    unsafe {
        let rows = wm::SendMessageW(h, cc::LVM_GETITEMCOUNT, None, None).0 as usize;
        let header = wm::SendMessageW(h, cc::LVM_GETHEADER, None, None).0;
        let cols = wm::SendMessageW(HWND(header as *mut _), cc::HDM_GETITEMCOUNT, None, None).0.max(0) as usize;
        (0..rows).map(|r| (0..cols).map(|c| cell(h, r, c)).collect()).collect()
    }
}

/// Adds a row at the end.
pub fn grid_add(h: impl Into<HWND>, cells: &[&str]) {
    let h: HWND = h.into();
    win::list_add(h, cells);
}

/// Empties the grid.
pub fn grid_clear(h: impl Into<HWND>) {
    let h: HWND = h.into();
    unsafe {
        wm::SendMessageW(h, cc::LVM_DELETEALLITEMS, None, None);
    }
}

/// A row's tag: a number the grid keeps with the row, whatever moves around it (rows taken out
/// above it, or added). A new row's is 0.
fn tag(h: HWND, row: usize) -> usize {
    let mut item = cc::LVITEMW { mask: cc::LVIF_PARAM, iItem: row as i32, ..Default::default() };
    unsafe {
        wm::SendMessageW(h, cc::LVM_GETITEMW, None, Some(LPARAM(&mut item as *mut _ as isize)));
    }
    item.lParam.0.max(0) as usize
}

fn set_tag(h: HWND, row: usize, value: usize) {
    let item = cc::LVITEMW { mask: cc::LVIF_PARAM, iItem: row as i32, lParam: LPARAM(value as isize), ..Default::default() };
    unsafe {
        wm::SendMessageW(h, cc::LVM_SETITEMW, None, Some(LPARAM(&item as *const _ as isize)));
    }
}

/// The grid emptied and filled with `rows` as a table gave them: each row tagged with its place
/// (1, 2, ...), so that after the person changes, adds and deletes rows, [`grid_tagged_rows`] still
/// says which row each one was - what a data binding needs to save.
pub fn grid_fill(h: impl Into<HWND>, rows: &[Vec<String>]) {
    let h: HWND = h.into();
    grid_clear(h);
    for (i, r) in rows.iter().enumerate() {
        let cells: Vec<&str> = r.iter().map(String::as_str).collect();
        grid_add(h, &cells);
        set_tag(h, i, i + 1);
    }
}

/// The grid's rows with their tags: the place each had when [`grid_fill`] put it there, 0 for a row
/// added since.
pub fn grid_tagged_rows(h: impl Into<HWND>) -> Vec<(usize, Vec<String>)> {
    let h: HWND = h.into();
    grid_rows(h).into_iter().enumerate().map(|(i, r)| (tag(h, i), r)).collect()
}

/// One cell's text.
pub fn cell(h: impl Into<HWND>, row: usize, col: usize) -> String {
    let h: HWND = h.into();
    let mut buf = vec![0u16; 1024];
    let mut item = cc::LVITEMW { iSubItem: col as i32, pszText: windows::core::PWSTR(buf.as_mut_ptr()), cchTextMax: 1024, ..Default::default() };
    let n = unsafe { wm::SendMessageW(h, cc::LVM_GETITEMTEXTW, Some(WPARAM(row)), Some(LPARAM(&mut item as *mut _ as isize))).0 } as usize;
    String::from_utf16_lossy(&buf[..n.min(1024)])
}

/// Sets one cell's text.
pub fn set_cell(h: impl Into<HWND>, row: usize, col: usize, text: &str) {
    let h: HWND = h.into();
    let mut t = win::wide(text);
    let item = cc::LVITEMW { iSubItem: col as i32, pszText: windows::core::PWSTR(t.as_mut_ptr()), ..Default::default() };
    unsafe {
        wm::SendMessageW(h, cc::LVM_SETITEMTEXTW, Some(WPARAM(row)), Some(LPARAM(&item as *const _ as isize)));
    }
}

fn columns(h: HWND) -> usize {
    unsafe {
        let header = wm::SendMessageW(h, cc::LVM_GETHEADER, None, None).0;
        wm::SendMessageW(HWND(header as *mut _), cc::HDM_GETITEMCOUNT, None, None).0.max(0) as usize
    }
}

/// The cell under a point in the grid, if there is one.
fn hit(h: HWND, x: i32, y: i32) -> Option<(usize, usize)> {
    let mut info = cc::LVHITTESTINFO { pt: POINT { x, y }, ..Default::default() };
    let row = unsafe { wm::SendMessageW(h, cc::LVM_SUBITEMHITTEST, None, Some(LPARAM(&mut info as *mut _ as isize))).0 };
    (row >= 0 && info.iItem >= 0).then_some((info.iItem as usize, info.iSubItem.max(0) as usize))
}

/// The box over a cell: made, filled with the cell, all chosen.
fn open_editor(h: HWND, row: usize, col: usize) {
    close_editor(h, true);
    let mut r = RECT { top: col as i32, left: 0 /* LVIR_BOUNDS */, ..Default::default() };
    unsafe {
        wm::SendMessageW(h, cc::LVM_GETSUBITEMRECT, Some(WPARAM(row)), Some(LPARAM(&mut r as *mut _ as isize)));
    }
    if col == 0 {
        // The first column's rectangle is the whole row: as wide as its column only.
        let w0 = unsafe { wm::SendMessageW(h, cc::LVM_GETCOLUMNWIDTH, Some(WPARAM(0)), None).0 } as i32;
        r.right = r.left + w0;
    }
    let font = unsafe { wm::SendMessageW(h, wm::WM_GETFONT, None, None).0 };
    let e = win::edit(h, &cell(h, row, col), EDITOR_ID, windows::Win32::Graphics::Gdi::HFONT(font as *mut _), false);
    win::move_to(e, r.left, r.top, r.right - r.left, r.bottom - r.top);
    unsafe {
        wm::SetPropW(e, w!("PhaiGridCell"), Some(windows::Win32::Foundation::HANDLE(((row << 16) | col) as *mut _))).ok();
        wm::SendMessageW(e, 0x00B1 /* EM_SETSEL */, Some(WPARAM(0)), Some(LPARAM(-1)));
        let _ = SetWindowSubclass(e, Some(editor_proc), 8, h.0 as usize);
        let _ = km::SetFocus(Some(e));
    }
}

fn editor_of(h: HWND) -> Option<HWND> {
    unsafe { wm::GetDlgItem(Some(h), EDITOR_ID) }.ok().filter(|e| !e.is_invalid())
}

/// The box closed - its words kept in the cell when `keep` - and the parent told of a change.
fn close_editor(h: HWND, keep: bool) {
    let Some(e) = editor_of(h) else { return };
    let at = unsafe { wm::GetPropW(e, w!("PhaiGridCell")) }.0 as usize;
    let (row, col) = (at >> 16, at & 0xFFFF);
    let text = win::text_of(e);
    unsafe {
        let _ = wm::RemovePropW(e, w!("PhaiGridCell"));
        let _ = wm::DestroyWindow(e);
    }
    if keep && text != cell(h, row, col) {
        set_cell(h, row, col, &text);
        unsafe {
            let id = wm::GetDlgCtrlID(h) as usize;
            if let Ok(p) = wm::GetParent(h) {
                wm::SendMessageW(p, wm::WM_COMMAND, Some(WPARAM(((win::VALUE_CHANGED as usize) << 16) | (id & 0xFFFF))), Some(LPARAM(h.0 as isize)));
            }
        }
    }
    unsafe {
        let _ = km::SetFocus(Some(h));
    }
}

fn chosen_row(h: HWND) -> Option<usize> {
    let i = unsafe { wm::SendMessageW(h, cc::LVM_GETNEXTITEM, Some(WPARAM(usize::MAX)), Some(LPARAM(2))).0 };
    (i >= 0).then_some(i as usize)
}

static READ_ONLY: std::sync::Mutex<Vec<isize>> = std::sync::Mutex::new(Vec::new());

/// A grid whose cells are read, not changed: no editor opens, no row is added or taken out.
pub fn set_read_only(h: HWND, on: bool) {
    let mut v = READ_ONLY.lock().unwrap_or_else(|e| e.into_inner());
    v.retain(|x| *x != h.0 as isize);
    if on {
        v.push(h.0 as isize);
    }
}

fn is_read_only(h: HWND) -> bool {
    READ_ONLY.lock().unwrap_or_else(|e| e.into_inner()).contains(&(h.0 as isize))
}

unsafe extern "system" fn grid_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, _: usize) -> LRESULT {
    if is_read_only(h) && (msg == wm::WM_LBUTTONDBLCLK || (msg == wm::WM_KEYDOWN && (w.0 == 0x71 || w.0 == 0x2E))) {
        // Read only: a double-click, F2 or Delete do what a list's would.
        return unsafe { DefSubclassProc(h, msg, w, l) };
    }
    match msg {
        wm::WM_LBUTTONDBLCLK => {
            let (x, y) = win::point_of(l);
            match hit(h, x, y) {
                Some((row, col)) => open_editor(h, row, col),
                None => {
                    // Under the last row: a new one, its first cell open.
                    let blank: Vec<&str> = vec![""; columns(h).max(1)];
                    grid_add(h, &blank);
                    let last = unsafe { wm::SendMessageW(h, cc::LVM_GETITEMCOUNT, None, None).0 } as usize - 1;
                    win::list_select(h, last);
                    open_editor(h, last, 0);
                }
            }
            LRESULT(0)
        }
        wm::WM_KEYDOWN if w.0 == 0x71 /* F2 */ => {
            if let Some(r) = chosen_row(h) {
                open_editor(h, r, 0);
            }
            LRESULT(0)
        }
        wm::WM_KEYDOWN if w.0 == 0x2E /* Delete: the chosen row goes */ => {
            if let Some(r) = chosen_row(h) {
                unsafe {
                    wm::SendMessageW(h, cc::LVM_DELETEITEM, Some(WPARAM(r)), None);
                    let id = wm::GetDlgCtrlID(h) as usize;
                    if let Ok(p) = wm::GetParent(h) {
                        wm::SendMessageW(p, wm::WM_COMMAND, Some(WPARAM(((win::VALUE_CHANGED as usize) << 16) | (id & 0xFFFF))), Some(LPARAM(h.0 as isize)));
                    }
                }
            }
            LRESULT(0)
        }
        m if m == wm::WM_APP + 0x7E => {
            grid_idle(h);
            LRESULT(0)
        }
        wm::WM_VSCROLL | wm::WM_HSCROLL | wm::WM_MOUSEWHEEL => {
            close_editor(h, true);
            unsafe { DefSubclassProc(h, msg, w, l) }
        }
        _ => unsafe { DefSubclassProc(h, msg, w, l) },
    }
}

unsafe extern "system" fn editor_proc(e: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, grid: usize) -> LRESULT {
    let g = HWND(grid as *mut _);
    match msg {
        wm::WM_GETDLGCODE => LRESULT(4 /* DLGC_WANTALLKEYS */),
        wm::WM_KEYDOWN if w.0 == 0x0D => {
            close_editor(g, true);
            LRESULT(0)
        }
        wm::WM_KEYDOWN if w.0 == 0x1B => {
            close_editor(g, false);
            LRESULT(0)
        }
        wm::WM_CHAR if w.0 == 0x0D || w.0 == 0x1B => LRESULT(0),
        wm::WM_KILLFOCUS => {
            let r = unsafe { DefSubclassProc(e, msg, w, l) };
            // Closed after this message is done with: never while Windows still uses the box.
            unsafe {
                let _ = wm::PostMessageW(Some(g), wm::WM_APP + 0x7E, WPARAM(0), LPARAM(0));
            }
            r
        }
        _ => unsafe { DefSubclassProc(e, msg, w, l) },
    }
}

/// Called by the grid's subclass on WM_APP + 0x7E: a box that lost the focus closes, kept.
pub fn grid_idle(h: HWND) {
    if let Some(e) = editor_of(h) {
        let focus = unsafe { km::GetFocus() };
        if focus != e {
            close_editor(h, true);
        }
    }
}

/// A rich text box: Windows' RichEdit, its words `text`, many lines, scrolled as they grow.
pub fn richtext(parent: HWND, id: i32, f: windows::Win32::Graphics::Gdi::HFONT, text: &str) -> HWND {
    unsafe {
        let _ = LoadLibraryW(w!("Msftedit.dll"));
    }
    const ES_MULTILINE: u32 = 0x0004;
    const ES_AUTOVSCROLL: u32 = 0x0040;
    const ES_WANTRETURN: u32 = 0x1000;
    let style = wm::WINDOW_STYLE(ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN) | wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_TABSTOP | wm::WS_VSCROLL | wm::WS_BORDER;
    let t = win::wide(text);
    let h = unsafe { wm::CreateWindowExW(Default::default(), w!("RICHEDIT50W"), PCWSTR(t.as_ptr()), style, 0, 0, 10, 10, Some(parent), Some(wm::HMENU(id as isize as *mut _)), Some(win::instance()), None).unwrap_or_default() };
    unsafe {
        wm::SendMessageW(h, wm::WM_SETFONT, Some(WPARAM(f.0 as usize)), Some(LPARAM(1)));
        // EN_CHANGE to the parent, as an edit box sends it.
        const EM_SETEVENTMASK: u32 = 0x0445;
        const ENM_CHANGE: isize = 0x0001;
        wm::SendMessageW(h, EM_SETEVENTMASK, Some(WPARAM(0)), Some(LPARAM(ENM_CHANGE)));
        // The theme's window colour behind the words.
        const EM_SETBKGNDCOLOR: u32 = 0x0443;
        wm::SendMessageW(h, EM_SETBKGNDCOLOR, Some(WPARAM(0)), Some(LPARAM(win::window_background_for(h).colorref() as isize)));
    }
    win::register_kind(h, win::Kind::Edit);
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_double_click_under_the_rows_adds_one_and_enter_keeps_what_is_typed() {
        win::init();
        extern "system" fn host_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
            win::def(h, msg, w, l)
        }
        let class = win::register("PhaiGridTestHost", Some(host_proc), None);
        let host = unsafe { wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_OVERLAPPEDWINDOW, 0, 0, 400, 300, None, None, Some(win::instance()), None).unwrap_or_default() };
        let font = win::font(host, 0, false);
        let g = grid(host, 5, font, &[("Name", 120), ("Age", 60)]);
        win::move_to(g, 0, 0, 380, 260);
        grid_add(g, &["Somchai", "40"]);
        unsafe {
            // Far below the one row: a new row, its first cell open for typing.
            wm::SendMessageW(g, wm::WM_LBUTTONDBLCLK, Some(WPARAM(0)), Some(LPARAM((200 << 16) | 20)));
        }
        let e = editor_of(g).expect("a box over the new cell");
        win::set_text(e, "Malee");
        unsafe {
            wm::SendMessageW(e, wm::WM_KEYDOWN, Some(WPARAM(0x0D)), Some(LPARAM(0)));
        }
        assert!(editor_of(g).is_none(), "Enter closes the box");
        assert_eq!(grid_rows(g), vec![vec!["Somchai".to_string(), "40".into()], vec!["Malee".into(), String::new()]]);
        unsafe {
            let _ = wm::DestroyWindow(host);
        }
    }
}
