//! Document Outline, as Visual Studio's: the form's controls as a tree - those inside a group or
//! tabs under it - each with its toolbox icon, name and kind. Choosing one chooses it on the form,
//! and choosing on the form chooses it here.

use phai_form::{FormFile, Kind};
use phai_ui::icon::glyph;
use phai_ui::win;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::HFONT;
use windows::Win32::UI::WindowsAndMessaging as wm;

pub const ID_OUTLINE_TREE: i32 = 420;

pub struct Outline {
    pub tree: HWND,
    /// The tree item of each control, by its index in the form.
    items: Vec<isize>,
    root: isize,
    /// While the tree is filled or chosen in from the form: its notices are not choices.
    pub quiet: std::cell::Cell<bool>,
}

/// The container a control lies in - the form's own rule.
pub fn parent_of(form: &FormFile, i: usize) -> Option<usize> {
    form.container_of(i)
}

impl Outline {
    pub fn new(side: HWND, font: HFONT) -> Outline {
        let tree = win::tree(side, ID_OUTLINE_TREE, font);
        let mut icons = vec![glyph::FORM];
        icons.extend(crate::toolbox_items().iter().map(|(_, _, g)| *g));
        win::tree_icons(tree, &icons);
        Outline { tree, items: vec![], root: 0, quiet: std::cell::Cell::new(false) }
    }

    fn icon_of(k: Kind) -> i32 {
        crate::toolbox_items().iter().position(|(id, _, _)| crate::tool_kind(*id) == Some(k)).map(|p| p as i32 + 1).unwrap_or(0)
    }

    /// The tree again from the form: the form at the top, its controls under it.
    pub fn fill(&mut self, form: &FormFile, selected: Option<usize>) {
        self.quiet.set(true);
        win::tree_clear(self.tree);
        self.root = win::tree_add(self.tree, 0, &format!("{}  ({})", form.form.name, phai_ui::text::t("Form")), 0, -1);
        self.items = vec![0; form.control.len()];
        // Parents before children: each control added once its container is.
        let parents: Vec<Option<usize>> = (0..form.control.len()).map(|i| parent_of(form, i)).collect();
        let mut done = vec![false; form.control.len()];
        for _ in 0..form.control.len() {
            for i in 0..form.control.len() {
                if done[i] {
                    continue;
                }
                let under = match parents[i] {
                    None => self.root,
                    Some(p) if done[p] => self.items[p],
                    Some(_) => continue,
                };
                let c = &form.control[i];
                self.items[i] = win::tree_add(self.tree, under, &format!("{}  ({})", c.id, crate::kind_label(c.kind)), Self::icon_of(c.kind), i as isize);
                done[i] = true;
            }
        }
        win::tree_expand(self.tree, self.root);
        for (i, p) in parents.iter().enumerate() {
            if p.is_none() && matches!(form.control[i].kind, Kind::Group | Kind::Tabs | Kind::Flow | Kind::Table) {
                win::tree_expand(self.tree, self.items[i]);
            }
        }
        self.select(selected);
        self.quiet.set(false);
    }

    /// The control chosen on the form chosen here too (`None`: the form).
    pub fn select(&self, i: Option<usize>) {
        let item = match i {
            Some(i) => self.items.get(i).copied().unwrap_or(self.root),
            None => self.root,
        };
        let was = self.quiet.replace(true);
        unsafe {
            wm::SendMessageW(self.tree, 0x110B /* TVM_SELECTITEM */, Some(WPARAM(9 /* TVGN_CARET */)), Some(LPARAM(item)));
        }
        self.quiet.set(was);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phai_form::Control;

    #[test]
    fn controls_inside_a_group_are_its_children() {
        let mut f = FormFile::new("Main");
        f.control.push(Control::new(Kind::Group, "outer", [10, 10, 300, 200]));
        f.control.push(Control::new(Kind::Group, "inner", [20, 40, 150, 100]));
        f.control.push(Control::new(Kind::Button, "ok", [30, 60, 75, 23]));
        f.control.push(Control::new(Kind::Button, "out", [400, 10, 75, 23]));
        assert_eq!(parent_of(&f, 2), Some(1), "the smallest group that holds it");
        assert_eq!(parent_of(&f, 1), Some(0));
        assert_eq!(parent_of(&f, 0), None);
        assert_eq!(parent_of(&f, 3), None);
    }
}
