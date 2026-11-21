//! A grid column's filter, as Excel's and DevExpress's: the drop-down on a heading opens a box of
//! the column's values to tick, a search over them, Sort A to Z / Z to A and Clear filter. OK
//! leaves only the rows whose value is ticked ([`crate::grid::set_column_filter`]).

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::Controls as cc;
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::text::t;
use crate::win;

const ID_SEARCH: i32 = 101;
const ID_LIST: i32 = 102;
const ID_UP: i32 = 103;
const ID_DOWN: i32 = 104;
const ID_CLEAR: i32 = 105;
const ID_OK: i32 = 106;
const ID_CANCEL: i32 = 107;

/// What the box was closed with.
#[derive(Clone, Debug, PartialEq)]
pub enum Choice {
    /// Only these values (raw, as the cells hold them); `None`: every value - no filter.
    Filter(Option<Vec<String>>),
    Sort(bool),
}

struct Pop {
    search: HWND,
    list: HWND,
    up: HWND,
    down: HWND,
    clear: HWND,
    ok: HWND,
    cancel: HWND,
    /// Each value as the cells hold it and as it is shown.
    values: Vec<(String, String)>,
    checked: Vec<bool>,
    /// The values the search leaves, by place in `values`.
    shown: Vec<usize>,
    filling: bool,
    done: bool,
    choice: Option<Choice>,
    font: Gdi::HFONT,
}

/// The values a column's filter offers: each different one once, numbers in order of size, the
/// others without regard to case, the empty one first.
pub fn distinct(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut v: Vec<String> = values.into_iter().collect();
    v.sort_by(|a, b| {
        let num = |s: &str| s.trim().replace(',', "").parse::<f64>().ok();
        match (a.trim().is_empty(), b.trim().is_empty()) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }
        match (num(a), num(b)) {
            (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
            _ => a.to_lowercase().cmp(&b.to_lowercase()).then_with(|| a.cmp(b)),
        }
    });
    v.dedup();
    v
}

/// What a search over the values leaves: those whose shown words have it, case aside.
pub fn matching(values: &[(String, String)], search: &str) -> Vec<usize> {
    let s = search.trim().to_lowercase();
    (0..values.len()).filter(|i| s.is_empty() || values[*i].1.to_lowercase().contains(&s)).collect()
}

/// The filter that the ticks make: none when every value is ticked.
pub fn filter_of(values: &[(String, String)], checked: &[bool]) -> Option<Vec<String>> {
    if checked.iter().all(|c| *c) {
        return None;
    }
    Some(values.iter().zip(checked).filter(|(_, c)| **c).map(|(v, _)| v.0.clone()).collect())
}

fn set_check(list: HWND, item: usize, on: bool) {
    let it = cc::LVITEMW { stateMask: cc::LVIS_STATEIMAGEMASK, state: cc::LIST_VIEW_ITEM_STATE_FLAGS(if on { 2 << 12 } else { 1 << 12 }), ..Default::default() };
    unsafe {
        wm::SendMessageW(list, cc::LVM_SETITEMSTATE, Some(WPARAM(item)), Some(LPARAM(&it as *const _ as isize)));
    }
}

fn fill(p: &mut Pop) {
    p.filling = true;
    let search = win::text_of(p.search);
    p.shown = matching(&p.values, &search);
    unsafe {
        wm::SendMessageW(p.list, wm::WM_SETREDRAW, Some(WPARAM(0)), None);
        wm::SendMessageW(p.list, cc::LVM_DELETEALLITEMS, None, None);
    }
    let all = t("(Select all)");
    let mut rows: Vec<(&str, isize)> = vec![(all, 0)];
    for i in &p.shown {
        rows.push((p.values[*i].1.as_str(), *i as isize + 1));
    }
    for (n, (text, tag)) in rows.iter().enumerate() {
        let mut w = win::wide(text);
        let item = cc::LVITEMW { mask: cc::LVIF_TEXT | cc::LVIF_PARAM, iItem: n as i32, pszText: windows::core::PWSTR(w.as_mut_ptr()), lParam: LPARAM(*tag), ..Default::default() };
        unsafe {
            wm::SendMessageW(p.list, cc::LVM_INSERTITEMW, None, Some(LPARAM(&item as *const _ as isize)));
        }
    }
    for (n, i) in p.shown.iter().enumerate() {
        set_check(p.list, n + 1, p.checked[*i]);
    }
    set_check(p.list, 0, p.shown.iter().all(|i| p.checked[*i]));
    unsafe {
        wm::SendMessageW(p.list, cc::LVM_SETCOLUMNWIDTH, Some(WPARAM(0)), Some(LPARAM(-2 /* LVSCW_AUTOSIZE_USEHEADER */)));
        wm::SendMessageW(p.list, wm::WM_SETREDRAW, Some(WPARAM(1)), None);
        let _ = Gdi::InvalidateRect(Some(p.list), None, true);
    }
    win::enable(p.ok, p.shown.iter().any(|i| p.checked[*i]) || p.shown.is_empty() && p.checked.iter().any(|c| *c));
    p.filling = false;
}

extern "system" fn proc_(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(p) = (unsafe { win::state::<Pop>(hwnd) }) else { return win::def(hwnd, msg, w, l) };
    match msg {
        wm::WM_SIZE => {
            let (cw, ch) = (win::loword(l.0 as usize) as i32, win::hiword(l.0 as usize) as i32);
            let m = win::px(hwnd, 8);
            let bh = win::px(hwnd, 26);
            let gap = win::px(hwnd, 4);
            let mut y = m;
            win::move_to(p.up, m, y, cw - 2 * m, bh);
            y += bh + gap;
            win::move_to(p.down, m, y, cw - 2 * m, bh);
            y += bh + gap;
            win::move_to(p.clear, m, y, cw - 2 * m, bh);
            y += bh + m;
            let eh = win::px(hwnd, 24);
            win::move_to(p.search, m, y, cw - 2 * m, eh);
            y += eh + gap;
            let bw = win::px(hwnd, 75);
            let by = ch - m - win::px(hwnd, 23);
            win::move_to(p.cancel, cw - m - bw, by, bw, win::px(hwnd, 23));
            win::move_to(p.ok, cw - m - 2 * bw - gap, by, bw, win::px(hwnd, 23));
            win::move_to(p.list, m, y, cw - 2 * m, by - m - y);
            LRESULT(0)
        }
        wm::WM_COMMAND => {
            let id = win::loword(w.0) as i32;
            let code = win::hiword(w.0) as u32;
            match id {
                ID_SEARCH if code == 0x300 /* EN_CHANGE */ => fill(p),
                ID_UP | ID_DOWN => {
                    p.choice = Some(Choice::Sort(id == ID_UP));
                    p.done = true;
                }
                ID_CLEAR => {
                    p.choice = Some(Choice::Filter(None));
                    p.done = true;
                }
                ID_OK => {
                    // A search that leaves some: only those of its ticked, as Excel's.
                    let search = !win::text_of(p.search).trim().is_empty();
                    let checked: Vec<bool> = (0..p.values.len()).map(|i| p.checked[i] && (!search || p.shown.contains(&i))).collect();
                    p.choice = Some(Choice::Filter(filter_of(&p.values, &checked)));
                    p.done = true;
                }
                ID_CANCEL => p.done = true,
                _ => {}
            }
            LRESULT(0)
        }
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            if hdr.idFrom == ID_LIST as usize && hdr.code == cc::LVN_ITEMCHANGED && !p.filling {
                let nm = unsafe { &*(l.0 as *const cc::NMLISTVIEW) };
                if nm.uChanged.0 & cc::LVIF_STATE.0 != 0 && (nm.uNewState ^ nm.uOldState) & 0xF000 != 0 && nm.uOldState & 0xF000 != 0 {
                    let on = (nm.uNewState & 0xF000) >> 12 == 2;
                    let tag = nm.lParam.0;
                    if tag == 0 {
                        for i in p.shown.clone() {
                            p.checked[i] = on;
                        }
                    } else if let Some(c) = p.checked.get_mut(tag as usize - 1) {
                        *c = on;
                    }
                    // The ticks again, after Windows is done with this one.
                    unsafe {
                        let _ = wm::PostMessageW(Some(hwnd), wm::WM_APP + 1, WPARAM(0), LPARAM(0));
                    }
                }
            }
            win::def(hwnd, msg, w, l)
        }
        m if m == wm::WM_APP + 1 => {
            p.filling = true;
            for (n, i) in p.shown.iter().enumerate() {
                set_check(p.list, n + 1, p.checked[*i]);
            }
            set_check(p.list, 0, p.shown.iter().all(|i| p.checked[*i]));
            win::enable(p.ok, p.checked.iter().any(|c| *c));
            p.filling = false;
            LRESULT(0)
        }
        wm::WM_ACTIVATE if win::loword(w.0) == 0 /* WA_INACTIVE */ => {
            p.done = true;
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            p.done = true;
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            if let Some(p) = unsafe { win::take_state::<Pop>(hwnd) } {
                unsafe {
                    let _ = Gdi::DeleteObject(p.font.into());
                }
            }
            LRESULT(0)
        }
        _ => win::def(hwnd, msg, w, l),
    }
}

/// Opens the filter box under `at` (screen pixels, a heading's drop-down) for these values (raw,
/// shown), those in `chosen` ticked (`None`: all); what it was closed with, `None` for Cancel.
pub fn ask(grid: HWND, at: RECT, values: Vec<(String, String)>, chosen: Option<&[String]>) -> Option<Choice> {
    win::init();
    let class = win::register("PhaiGridFilter", Some(proc_), None);
    let owner = unsafe { wm::GetAncestor(grid, wm::GA_ROOT) };
    let checked: Vec<bool> = values.iter().map(|(v, _)| chosen.is_none_or(|c| c.iter().any(|x| x == v))).collect();
    unsafe {
        let hwnd = wm::CreateWindowExW(
            wm::WS_EX_TOOLWINDOW,
            windows::core::PCWSTR(class.as_ptr()),
            windows::core::PCWSTR::null(),
            wm::WS_POPUP | wm::WS_BORDER | wm::WS_CLIPCHILDREN,
            at.left,
            at.bottom,
            10,
            10,
            Some(owner),
            None,
            Some(win::instance()),
            None,
        )
        .unwrap_or_default();
        if hwnd.is_invalid() {
            return None;
        }
        let font = win::font(hwnd, 0, false);
        let up = win::button(hwnd, t("Sort A to Z"), ID_UP, font, false);
        let down = win::button(hwnd, t("Sort Z to A"), ID_DOWN, font, false);
        let clear = win::button(hwnd, t("Clear filter"), ID_CLEAR, font, false);
        let search = win::edit(hwnd, "", ID_SEARCH, font, false);
        let cue = win::wide(t("Search"));
        wm::SendMessageW(search, 0x1501 /* EM_SETCUEBANNER */, Some(WPARAM(1)), Some(LPARAM(cue.as_ptr() as isize)));
        let list = win::list(hwnd, ID_LIST, font, &[("", 200)]);
        let style = wm::GetWindowLongW(list, wm::GWL_STYLE);
        wm::SetWindowLongW(list, wm::GWL_STYLE, style | 0x4000 /* LVS_NOCOLUMNHEADER */ | 0x0004 /* LVS_SINGLESEL */);
        wm::SendMessageW(list, cc::LVM_SETEXTENDEDLISTVIEWSTYLE, Some(WPARAM(0x4)), Some(LPARAM(0x4 /* LVS_EX_CHECKBOXES */)));
        let ok = win::button(hwnd, t("OK"), ID_OK, font, true);
        let cancel = win::button(hwnd, t("Cancel"), ID_CANCEL, font, false);
        let n = values.len();
        win::set_state(hwnd, Box::new(Pop { search, list, up, down, clear, ok, cancel, values, checked, shown: (0..n).collect(), filling: false, done: false, choice: None, font }));
        if let Some(p) = win::state::<Pop>(hwnd) {
            fill(p);
        }
        // Under the drop-down, its right edges lined up, kept on the screen.
        let (w, h) = (win::px(hwnd, 240), win::px(hwnd, 360));
        let mut work = RECT::default();
        let _ = wm::SystemParametersInfoW(wm::SPI_GETWORKAREA, 0, Some(&mut work as *mut _ as *mut _), Default::default());
        let x = (at.right - w).max(work.left).min(work.right - w);
        let y = if at.bottom + h > work.bottom { (at.top - h).max(work.top) } else { at.bottom };
        // A photograph of it takes neither the keyboard nor the window in front.
        let quiet = std::env::var_os("PHAI_SHOT").is_some();
        let _ = wm::SetWindowPos(hwnd, Some(wm::HWND_TOP), x, y, w, h, wm::SWP_SHOWWINDOW | if quiet { wm::SWP_NOACTIVATE } else { Default::default() });
        win::theme_frame(hwnd);
        win::theme_native(list);
        if !quiet {
            let _ = km::SetFocus(Some(search));
        }
        let mut msg = wm::MSG::default();
        loop {
            if win::state::<Pop>(hwnd).is_none_or(|p| p.done) {
                break;
            }
            if !wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
                wm::PostQuitMessage(msg.wParam.0 as i32);
                break;
            }
            // Enter is OK and Escape Cancel, wherever the focus is in the box.
            if msg.message == wm::WM_KEYDOWN && (msg.wParam.0 == 0x0D || msg.wParam.0 == 0x1B) && (msg.hwnd == hwnd || wm::IsChild(hwnd, msg.hwnd).as_bool()) {
                let id = if msg.wParam.0 == 0x0D { ID_OK } else { ID_CANCEL };
                let enabled = id == ID_CANCEL || win::state::<Pop>(hwnd).is_some_and(|p| km::IsWindowEnabled(p.ok).as_bool());
                if enabled {
                    wm::SendMessageW(hwnd, wm::WM_COMMAND, Some(WPARAM(id as usize)), None);
                }
                continue;
            }
            if !wm::IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
        }
        let choice = win::state::<Pop>(hwnd).and_then(|p| p.choice.take());
        let _ = wm::DestroyWindow(hwnd);
        if !quiet {
            let _ = km::SetFocus(Some(grid));
        }
        choice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_are_offered_once_in_order() {
        let v = distinct(["Paid", "10", "", "2", "paid", "Due", "2", "1,000"].map(String::from));
        assert_eq!(v, vec!["", "2", "10", "1,000", "Due", "Paid", "paid"]);
    }

    #[test]
    fn the_ticks_make_the_filter() {
        let values: Vec<(String, String)> = ["a", "b", "c"].iter().map(|s| (s.to_string(), s.to_uppercase())).collect();
        assert_eq!(filter_of(&values, &[true, true, true]), None);
        assert_eq!(filter_of(&values, &[true, false, true]), Some(vec!["a".to_string(), "c".to_string()]));
        assert_eq!(matching(&values, " b "), vec![1]);
        assert_eq!(matching(&values, ""), vec![0, 1, 2]);
    }
}
