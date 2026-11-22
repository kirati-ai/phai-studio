//! Handles to the page's elements. A control is `Copy` on Windows (a window handle); here it is
//! the number of its element in this list, so a program's code copies controls the same way.

use std::cell::RefCell;

use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlElement, Window};

/// A control or a window on the page - Windows' HWND, as the generated code and the program's
/// code pass it around.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct HWND(pub u32);

impl HWND {
    /// No element.
    pub fn is_invalid(self) -> bool {
        self.0 == 0
    }
}

thread_local! {
    /// Every element a handle names; 0 is none.
    static ELEMENTS: RefCell<Vec<Option<HtmlElement>>> = const { RefCell::new(Vec::new()) };
}

pub fn window() -> Window {
    web_sys::window().expect("a browser window")
}

pub fn document() -> Document {
    window().document().expect("a document")
}

/// A handle for `e`.
pub fn keep(e: HtmlElement) -> HWND {
    ELEMENTS.with(|l| {
        let mut l = l.borrow_mut();
        if l.is_empty() {
            l.push(None);
        }
        l.push(Some(e));
        HWND(l.len() as u32 - 1)
    })
}

/// The element `h` names.
pub fn el(h: HWND) -> Option<HtmlElement> {
    ELEMENTS.with(|l| l.borrow().get(h.0 as usize).cloned().flatten())
}

/// The handle `e` was given, if it was.
pub fn handle_of(e: &HtmlElement) -> Option<HWND> {
    ELEMENTS.with(|l| l.borrow().iter().position(|x| x.as_ref() == Some(e)).map(|i| HWND(i as u32)))
}

/// `e` in the place of `h`'s element, on the page and in the list: `h` names it now.
pub fn replace(h: HWND, e: HtmlElement) {
    if let Some(old) = el(h) {
        let _ = old.replace_with_with_node_1(&e);
    }
    ELEMENTS.with(|l| {
        if let Some(slot) = l.borrow_mut().get_mut(h.0 as usize) {
            *slot = Some(e);
        }
    });
}

/// `h` and what is in it gone from the page and the list.
pub fn forget(h: HWND) {
    if let Some(e) = el(h) {
        e.remove();
    }
    ELEMENTS.with(|l| {
        if let Some(slot) = l.borrow_mut().get_mut(h.0 as usize) {
            *slot = None;
        }
    });
}

/// A new element of `tag`.
pub fn make(tag: &str) -> HtmlElement {
    document().create_element(tag).expect("an element").unchecked_into()
}

/// Shows a message and waits for OK - WinForms' MessageBox.Show.
pub fn message(text: &str) {
    let _ = window().alert_with_message(text);
}

/// Asks yes or no - MessageBox.Show with YesNo; true for yes.
pub fn confirm(text: &str) -> bool {
    window().confirm_with_message(text).unwrap_or(false)
}

/// Whether the browser asks for Thai first.
pub fn browser_thai() -> bool {
    let nav = window().navigator();
    let first = nav.language().unwrap_or_default();
    first.to_ascii_lowercase().starts_with("th")
}
