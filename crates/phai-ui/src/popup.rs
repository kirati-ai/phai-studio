//! A context menu built each time it opens - WinForms' ContextMenuStrip: its items as the form
//! has them, then as the program changed them since (`Visible`, `Checked`, `Enabled`), menus
//! within it opening beside their item.
//!
//! The generated code keeps the items in the form's order and calls [`show`] on a right-click
//! (or the icon in the corner's), after the menu's `opening` event - so code there can hide, tick
//! or grey items with [`set_visible`], [`set_checked`], [`set_enabled`] just before it opens.
//! A choice comes to the window as `WM_COMMAND`, its item's id.

use std::cell::RefCell;
use std::collections::HashMap;

use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::win::wide;

/// One item of a context menu.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Item {
    /// Its name in the form - what [`set_visible`] and the rest are given; empty for a separator.
    pub name: &'static str,
    /// Its `WM_COMMAND` id; 0 for a separator or a menu within the menu.
    pub id: i32,
    /// Its words, `&` before the letter Alt takes.
    pub label: &'static str,
    /// Its shortcut's words, shown at the right (`Ctrl+C`).
    pub key: &'static str,
    /// A glyph beside it (`phai_ui::icon::glyph`), 0 for none.
    pub glyph: u16,
    /// As the form has them, until the program says otherwise.
    pub checked: bool,
    pub disabled: bool,
    pub hidden: bool,
    pub separator: bool,
    /// A menu of its own, opening beside it - DropDownItems.
    pub items: Vec<Item>,
}

impl Item {
    pub fn separator() -> Item {
        Item { separator: true, ..Default::default() }
    }
}

/// What the program changed of an item, over what the form says.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct State {
    visible: Option<bool>,
    checked: Option<bool>,
    enabled: Option<bool>,
}

thread_local! {
    /// By window and item name.
    static STATES: RefCell<HashMap<(isize, String), State>> = RefCell::new(HashMap::new());
}

fn change(window: HWND, name: &str, f: impl FnOnce(&mut State)) {
    STATES.with(|s| f(s.borrow_mut().entry((window.0 as isize, name.to_string())).or_default()));
}

fn state(window: HWND, name: &str) -> State {
    STATES.with(|s| s.borrow().get(&(window.0 as isize, name.to_string())).copied().unwrap_or_default())
}

/// Item `name` of `window`'s context menus shown or left out the next time one opens - Visible.
/// A separator beside an item left out is left out too when it would stand at an end of the menu
/// or next to another.
pub fn set_visible(window: HWND, name: &str, on: bool) {
    change(window, name, |s| s.visible = Some(on));
}

/// Item `name` ticked or not - Checked.
pub fn set_checked(window: HWND, name: &str, on: bool) {
    change(window, name, |s| s.checked = Some(on));
}

/// Item `name` usable or greyed - Enabled.
pub fn set_enabled(window: HWND, name: &str, on: bool) {
    change(window, name, |s| s.enabled = Some(on));
}

/// Whether item `name` is ticked: as the program set it, or `from_form` when it has not.
pub fn checked(window: HWND, name: &str, from_form: bool) -> bool {
    state(window, name).checked.unwrap_or(from_form)
}

/// Whether item `name` is shown: as the program set it, or `from_form` when it has not.
pub fn visible(window: HWND, name: &str, from_form: bool) -> bool {
    state(window, name).visible.unwrap_or(from_form)
}

/// Whether item `name` can be chosen: as the program set it, or `from_form` when it has not.
pub fn enabled(window: HWND, name: &str, from_form: bool) -> bool {
    state(window, name).enabled.unwrap_or(from_form)
}

/// CheckOnClick: item `name` chosen, its tick turned over; the tick now.
pub fn toggle(window: HWND, name: &str, from_form: bool) -> bool {
    let on = !checked(window, name, from_form);
    set_checked(window, name, on);
    on
}

/// What `window` changed of its items, forgotten - the generated code does it as it closes.
pub fn forget(window: HWND) {
    STATES.with(|s| s.borrow_mut().retain(|(w, _), _| *w != window.0 as isize));
}

/// The items to show: those not hidden, with no separator at either end or two together, and no
/// menu within the menu left with nothing in it. `shown` says whether an item is visible.
fn to_show<'a>(items: &'a [Item], shown: &dyn Fn(&Item) -> bool) -> Vec<&'a Item> {
    let mut out: Vec<&Item> = Vec::new();
    for it in items {
        if it.separator {
            if out.last().is_some_and(|l| !l.separator) {
                out.push(it);
            }
            continue;
        }
        if !shown(it) || (!it.items.is_empty() && to_show(&it.items, shown).is_empty()) {
            continue;
        }
        out.push(it);
    }
    while out.last().is_some_and(|l| l.separator) {
        out.pop();
    }
    out
}

/// The menu as `window` shows it now; the caller destroys it (its menus within go with it).
pub fn build(window: HWND, items: &[Item]) -> wm::HMENU {
    let shown = |it: &Item| it.name.is_empty() || visible(window, it.name, !it.hidden);
    let shown = &shown;
    let m = unsafe { wm::CreatePopupMenu() }.unwrap_or_default();
    for it in to_show(items, shown) {
        unsafe {
            if it.separator {
                let _ = wm::AppendMenuW(m, wm::MF_SEPARATOR, 0, None);
                continue;
            }
            let words = if it.key.is_empty() { it.label.to_string() } else { format!("{}\t{}", it.label, it.key) };
            let s = wide(&words);
            let mut flags = wm::MF_STRING;
            let (ticked, usable) = if it.name.is_empty() { (it.checked, !it.disabled) } else { (checked(window, it.name, it.checked), enabled(window, it.name, !it.disabled)) };
            if ticked {
                flags |= wm::MF_CHECKED;
            }
            if !usable {
                flags |= wm::MF_GRAYED;
            }
            if it.items.is_empty() {
                let _ = wm::AppendMenuW(m, flags, it.id as usize, PCWSTR(s.as_ptr()));
                if it.glyph != 0 {
                    crate::icon::set_menu_glyph(m, it.id as u32, it.glyph);
                }
            } else {
                let sub = build(window, &it.items);
                let _ = wm::AppendMenuW(m, flags | wm::MF_POPUP, sub.0 as usize, PCWSTR(s.as_ptr()));
            }
        }
    }
    m
}

/// The menu at (`x`, `y`) on the screen until a choice or a click elsewhere; the choice comes to
/// `window` as `WM_COMMAND`.
pub fn show(window: HWND, x: i32, y: i32, items: &[Item]) {
    let m = build(window, items);
    unsafe {
        let _ = wm::TrackPopupMenu(m, wm::TPM_LEFTALIGN | wm::TPM_TOPALIGN, x, y, None, window, None);
        let _ = wm::DestroyMenu(m);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &'static str) -> Item {
        Item { name, id: 1, label: name, ..Default::default() }
    }

    fn names(v: &[&Item]) -> Vec<&'static str> {
        v.iter().map(|i| if i.separator { "-" } else { i.name }).collect()
    }

    #[test]
    fn separators_beside_hidden_items_do_not_double_up() {
        let items = vec![item("open"), Item::separator(), item("id"), Item::separator(), item("end"), Item::separator(), item("about")];
        let hide = |gone: &'static [&'static str]| move |it: &Item| !gone.contains(&it.name);
        assert_eq!(names(&to_show(&items, &hide(&["id"]))), ["open", "-", "end", "-", "about"]);
        assert_eq!(names(&to_show(&items, &hide(&["id", "end"]))), ["open", "-", "about"]);
        assert_eq!(names(&to_show(&items, &hide(&["open"]))), ["id", "-", "end", "-", "about"]);
        assert_eq!(names(&to_show(&items, &hide(&["about"]))), ["open", "-", "id", "-", "end"]);
        assert!(to_show(&items, &hide(&["open", "id", "end", "about"])).is_empty());
    }

    #[test]
    fn a_menu_within_with_nothing_shown_is_left_out() {
        let lang = Item { name: "language", label: "Language", items: vec![item("en"), item("th")], ..Default::default() };
        let items = vec![item("open"), Item::separator(), lang, Item::separator(), item("exit")];
        assert_eq!(names(&to_show(&items, &|it: &Item| !["en", "th"].contains(&it.name))), ["open", "-", "exit"]);
        assert_eq!(names(&to_show(&items, &|it: &Item| it.name != "en")), ["open", "-", "language", "-", "exit"]);
    }

    #[test]
    fn what_the_program_set_wins_over_the_form() {
        let w = HWND(42 as *mut _);
        assert!(checked(w, "en", true));
        set_checked(w, "en", false);
        assert!(!checked(w, "en", true));
        assert!(toggle(w, "en", true));
        set_visible(w, "end", false);
        assert!(!visible(w, "end", true) && visible(w, "other", true));
        set_enabled(w, "end", false);
        assert!(!enabled(w, "end", true));
        forget(w);
        assert!(visible(w, "end", true) && checked(w, "en", true));
    }
}
