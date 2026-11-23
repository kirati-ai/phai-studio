//! What the program changed of its context menus' items - hidden, ticked, greyed - over what the
//! form says, as `phai_ui::popup` keeps it on Windows: the same calls, so one program's code
//! runs on both. A context menu ([`crate::menu::context_menu`]) reads it each time it opens.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::win::HWND;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct State {
    pub visible: Option<bool>,
    pub checked: Option<bool>,
    pub enabled: Option<bool>,
}

thread_local! {
    static STATES: RefCell<HashMap<(u32, String), State>> = RefCell::new(HashMap::new());
}

fn change(window: HWND, name: &str, f: impl FnOnce(&mut State)) {
    STATES.with(|s| f(s.borrow_mut().entry((window.0, name.to_string())).or_default()));
}

pub(crate) fn state(window: HWND, name: &str) -> State {
    STATES.with(|s| s.borrow().get(&(window.0, name.to_string())).copied().unwrap_or_default())
}

/// Item `name` of `window`'s context menus shown or left out the next time one opens - Visible.
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

/// Whether item `name` is shown: as the program set it, or `from_form`.
pub fn visible(window: HWND, name: &str, from_form: bool) -> bool {
    state(window, name).visible.unwrap_or(from_form)
}

/// Whether item `name` can be chosen: as the program set it, or `from_form`.
pub fn enabled(window: HWND, name: &str, from_form: bool) -> bool {
    state(window, name).enabled.unwrap_or(from_form)
}

/// CheckOnClick: item `name` chosen, its tick turned over; the tick now.
pub fn toggle(window: HWND, name: &str, from_form: bool) -> bool {
    let on = !checked(window, name, from_form);
    set_checked(window, name, on);
    on
}

/// What `window` changed, forgotten.
pub fn forget(window: HWND) {
    STATES.with(|s| s.borrow_mut().retain(|(w, _), _| *w != window.0));
}
