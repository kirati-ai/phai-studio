//! A tree on the page - Windows' TreeView: items under items, each that holds others opened and
//! closed with its arrow (or a double-click), one chosen at a time. The same `TreeView` methods
//! as `phai_ui::ctl`, the same `Node`s; the form's Items as Windows reads them (two spaces before
//! an item for each level down).

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

use crate::ctl::TreeView;
use crate::win::{el, keep, make, HWND};

/// An item of a tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Node(pub isize);

fn root_list(tree: HWND) -> Option<HtmlElement> {
    let e = el(tree)?;
    if let Ok(Some(ul)) = e.query_selector(":scope > ul") {
        return ul.dyn_into().ok();
    }
    let ul = make("ul");
    ul.set_attribute("role", "tree").ok();
    let _ = e.append_child(&ul);
    Some(ul)
}

fn node_el(n: Node) -> Option<HtmlElement> {
    el(HWND(n.0 as u32))
}

/// The tree a click happened in: its items followed - the arrow opens, a click chooses.
pub(crate) fn make_tree(h: HWND) {
    let Some(e) = el(h) else { return };
    let _ = root_list(h);
    e.set_attribute("tabindex", "0").ok();
    let click = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |ev: web_sys::MouseEvent| {
        let Some(t) = ev.target().and_then(|t| t.dyn_into::<Element>().ok()) else { return };
        let Some(li) = t.closest("li").ok().flatten() else { return };
        let li: HtmlElement = li.unchecked_into();
        if t.class_list().contains("phai-twist") {
            toggle(&li);
            return;
        }
        choose(h, &li);
    });
    let _ = e.add_event_listener_with_callback("click", click.as_ref().unchecked_ref());
    click.forget();
    let dbl = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |ev: web_sys::MouseEvent| {
        let Some(t) = ev.target().and_then(|t| t.dyn_into::<Element>().ok()) else { return };
        if let Some(li) = t.closest("li").ok().flatten() {
            toggle(&li.unchecked_into());
        }
    });
    let _ = e.add_event_listener_with_callback("dblclick", dbl.as_ref().unchecked_ref());
    dbl.forget();
    // The keys Windows' tree has: up and down, right opens, left closes (or goes to the parent).
    let keys = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |ev: web_sys::KeyboardEvent| {
        let t = TreeView(h);
        let shown = shown_items(h);
        let at = t.selected_node().and_then(node_el).and_then(|li| shown.iter().position(|x| *x == li));
        let go = |li: Option<&HtmlElement>| {
            if let Some(li) = li {
                choose(h, li);
            }
        };
        match ev.key().as_str() {
            "ArrowDown" => go(shown.get(at.map(|i| i + 1).unwrap_or(0))),
            "ArrowUp" => go(at.and_then(|i| i.checked_sub(1)).and_then(|i| shown.get(i))),
            "ArrowRight" => {
                if let Some(li) = at.and_then(|i| shown.get(i)) {
                    if li.class_list().contains("closed") {
                        toggle(li);
                    }
                }
            }
            "ArrowLeft" => {
                if let Some(li) = at.and_then(|i| shown.get(i)) {
                    if li.class_list().contains("open") {
                        toggle(li);
                    } else if let Some(p) = li.parent_element().and_then(|ul| ul.closest("li").ok().flatten()) {
                        go(Some(&p.unchecked_into()));
                    }
                }
            }
            _ => return,
        }
        ev.prevent_default();
    });
    let _ = e.add_event_listener_with_callback("keydown", keys.as_ref().unchecked_ref());
    keys.forget();
}

/// The items one can see now, top to bottom.
fn shown_items(h: HWND) -> Vec<HtmlElement> {
    let mut out = Vec::new();
    let Some(e) = el(h) else { return out };
    if let Ok(list) = e.query_selector_all("li") {
        for i in 0..list.length() {
            let Some(li) = list.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) else { continue };
            // Hidden when an item above it is closed.
            let mut up = li.parent_element();
            let mut seen = true;
            while let Some(p) = up {
                if p.tag_name().eq_ignore_ascii_case("li") && p.class_list().contains("closed") {
                    seen = false;
                    break;
                }
                if el(h).is_some_and(|t| *t == p) {
                    break;
                }
                up = p.parent_element();
            }
            if seen {
                out.push(li);
            }
        }
    }
    out
}

fn toggle(li: &HtmlElement) {
    let c = li.class_list();
    if c.contains("open") {
        let _ = c.replace("open", "closed");
    } else if c.contains("closed") {
        let _ = c.replace("closed", "open");
    }
}

/// `li` chosen: marked, scrolled to, and the tree's `change` raised - its AfterSelect.
fn choose(h: HWND, li: &HtmlElement) {
    let Some(e) = el(h) else { return };
    if li.class_list().contains("selected") {
        return;
    }
    if let Ok(list) = e.query_selector_all("li.selected") {
        for i in 0..list.length() {
            if let Some(x) = list.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                let _ = x.class_list().remove_1("selected");
            }
        }
    }
    let _ = li.class_list().add_1("selected");
    if let Ok(Some(row)) = li.query_selector(":scope > .phai-node") {
        if let Some(r) = row.dyn_ref::<HtmlElement>() {
            r.scroll_into_view_with_bool(false);
        }
    }
    if let Ok(ev) = web_sys::Event::new("change") {
        let _ = e.dispatch_event(&ev);
    }
}

/// An item with `text` at the end of `parent`'s (the tree's own list when `None`).
fn add(tree: HWND, parent: Option<Node>, text: &str) -> Node {
    let list = match parent.and_then(node_el) {
        Some(p) => {
            if let Ok(Some(ul)) = p.query_selector(":scope > ul") {
                ul.dyn_into::<HtmlElement>().ok()
            } else {
                let ul = make("ul");
                ul.set_attribute("role", "group").ok();
                let _ = p.append_child(&ul);
                // It holds others now: an arrow, open as Windows' tree_items leaves it.
                let _ = p.class_list().add_1("open");
                Some(ul)
            }
        }
        None => root_list(tree),
    };
    let li = make("li");
    li.set_attribute("role", "treeitem").ok();
    let row = make("div");
    row.set_class_name("phai-node");
    let twist = make("span");
    twist.set_class_name("phai-twist");
    let words = make("span");
    words.set_class_name("phai-node-text");
    words.set_text_content(Some(text));
    let _ = row.append_child(&twist);
    let _ = row.append_child(&words);
    let _ = li.append_child(&row);
    if let Some(l) = list {
        let _ = l.append_child(&li);
    }
    Node(keep(li).0 as isize)
}

/// The tree emptied and filled from the form's lines: two spaces before an item per level down.
pub fn set_lines(tree: HWND, lines: &[&str]) {
    TreeView(tree).nodes_clear();
    let mut parents: Vec<Node> = Vec::new();
    for line in lines {
        let raw = line.replace('\t', "  ");
        let depth = (raw.len() - raw.trim_start().len()) / 2;
        parents.truncate(depth);
        let n = add(tree, parents.last().copied(), raw.trim());
        parents.push(n);
    }
}

impl TreeView {
    /// A top item; to add under it, [`TreeView::nodes_add_under`].
    pub fn nodes_add(self, text: &str) -> Node {
        add(self.0, None, text)
    }
    pub fn nodes_add_under(self, parent: Node, text: &str) -> Node {
        add(self.0, Some(parent), text)
    }
    pub fn nodes_clear(self) {
        if let Some(ul) = root_list(self.0) {
            ul.set_inner_html("");
        }
    }
    /// The lines as the form's Items write them: two spaces before an item per level down.
    pub fn set_nodes(self, lines: &[&str]) {
        set_lines(self.0, lines)
    }
    pub fn expand_all(self) {
        self.all_to("open")
    }
    pub fn collapse_all(self) {
        self.all_to("closed")
    }
    fn all_to(self, state: &str) {
        let Some(e) = el(self.0) else { return };
        if let Ok(list) = e.query_selector_all("li.open, li.closed") {
            for i in 0..list.length() {
                if let Some(li) = list.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                    let c = li.class_list();
                    let _ = c.remove_2("open", "closed");
                    let _ = c.add_1(state);
                }
            }
        }
    }
    /// The item chosen.
    pub fn selected_node(self) -> Option<Node> {
        let li: HtmlElement = el(self.0)?.query_selector("li.selected").ok().flatten()?.dyn_into().ok()?;
        node_of(&li)
    }
    pub fn select(self, n: Node) {
        if let Some(li) = node_el(n) {
            choose(self.0, &li);
        }
    }
    /// An item's words.
    pub fn node_text(self, n: Node) -> String {
        node_el(n).and_then(|li| li.query_selector(":scope > .phai-node > .phai-node-text").ok().flatten()).and_then(|t| t.text_content()).unwrap_or_default()
    }
    /// The chosen item's words.
    pub fn selected_text(self) -> Option<String> {
        self.selected_node().map(|n| self.node_text(n))
    }
    pub fn expand(self, n: Node) {
        if let Some(li) = node_el(n) {
            let _ = li.class_list().replace("closed", "open");
        }
    }
    pub fn collapse(self, n: Node) {
        if let Some(li) = node_el(n) {
            let _ = li.class_list().replace("open", "closed");
        }
    }
}

/// The node handle an item element was given.
fn node_of(li: &HtmlElement) -> Option<Node> {
    crate::win::handle_of(li).map(|h| Node(h.0 as isize))
}

