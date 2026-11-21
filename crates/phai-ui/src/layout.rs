//! Where each control goes when its window changes size: **dock** and **anchor**, as WinForms has
//! them. A designer can only put a control where it can be described, and these two describe
//! nearly every window people build - a toolbar docked to the top, a list filling the rest, OK
//! and Cancel anchored to the bottom right.
//!
//! Positions are design pixels at 96 DPI, given for the container at its design size. [`arrange`]
//! is pure arithmetic, so it is tested rather than looked at; [`Layout`] applies it to windows.
//!
//! **Dock** takes controls in the order given: each one docked to an edge takes a strip of what
//! is left, at its own height (top/bottom) or width (left/right); `Fill` takes all that remains.
//! **Anchor** keeps a control's distance to the edges it is anchored to. Anchored to both left
//! and right, it stretches; to neither, it stays centred where it was. The same for top/bottom.

use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging as wm;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Dock {
    #[default]
    None,
    Top,
    Bottom,
    Left,
    Right,
    Fill,
}

/// Which edges a control keeps its distance to. The default is top-left, as in WinForms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Anchor {
    pub left: bool,
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
}

impl Default for Anchor {
    fn default() -> Anchor {
        Anchor::TOP_LEFT
    }
}

impl Anchor {
    pub const TOP_LEFT: Anchor = Anchor { left: true, top: true, right: false, bottom: false };
    pub const ALL: Anchor = Anchor { left: true, top: true, right: true, bottom: true };
    pub const NONE: Anchor = Anchor { left: false, top: false, right: false, bottom: false };
    pub const BOTTOM_RIGHT: Anchor = Anchor { left: false, top: false, right: true, bottom: true };
    pub const TOP_LEFT_RIGHT: Anchor = Anchor { left: true, top: true, right: true, bottom: false };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Item {
    /// Where it is when the container is its design size.
    pub bounds: Rect,
    pub dock: Dock,
    pub anchor: Anchor,
}

/// Each item's place in a container of `size`, all at 96 DPI. `design` is the container's size
/// the bounds were given for. `padding` is kept clear inside the container's edges for docking.
pub fn arrange(design: (i32, i32), size: (i32, i32), padding: i32, items: &[Item]) -> Vec<Rect> {
    let (dw, dh) = (size.0 - design.0, size.1 - design.1);
    // What docking has left, in the current size.
    let mut free = Rect::new(padding, padding, (size.0 - 2 * padding).max(0), (size.1 - 2 * padding).max(0));
    items
        .iter()
        .map(|it| {
            let b = it.bounds;
            match it.dock {
                Dock::Top => {
                    let h = b.h.min(free.h);
                    let r = Rect::new(free.x, free.y, free.w, h);
                    free.y += h;
                    free.h -= h;
                    r
                }
                Dock::Bottom => {
                    let h = b.h.min(free.h);
                    free.h -= h;
                    Rect::new(free.x, free.y + free.h, free.w, h)
                }
                Dock::Left => {
                    let w = b.w.min(free.w);
                    let r = Rect::new(free.x, free.y, w, free.h);
                    free.x += w;
                    free.w -= w;
                    r
                }
                Dock::Right => {
                    let w = b.w.min(free.w);
                    free.w -= w;
                    Rect::new(free.x + free.w, free.y, w, free.h)
                }
                Dock::Fill => {
                    let r = free;
                    free = Rect::new(free.x, free.y, 0, 0);
                    r
                }
                Dock::None => {
                    let a = it.anchor;
                    let (x, w) = axis(b.x, b.w, dw, a.left, a.right);
                    let (y, h) = axis(b.y, b.h, dh, a.top, a.bottom);
                    Rect::new(x, y, w, h)
                }
            }
        })
        .collect()
}

/// One direction of anchoring: the container grew by `grow`.
fn axis(pos: i32, len: i32, grow: i32, near: bool, far: bool) -> (i32, i32) {
    match (near, far) {
        (true, true) => (pos, (len + grow).max(0)),
        (false, true) => (pos + grow, len),
        (true, false) => (pos, len),
        // Centred where it was: half the growth on each side.
        (false, false) => (pos + grow / 2, len),
    }
}

/// `v` design pixels at `dpi`.
pub fn scale(v: i32, dpi: i32) -> i32 {
    (v as i64 * dpi as i64 / 96) as i32
}

/// The other way: `v` device pixels at `dpi` as design pixels.
pub fn unscale(v: i32, dpi: i32) -> i32 {
    (v as i64 * 96 / dpi.max(1) as i64) as i32
}

/// Windows and how each is placed, applied to their parent's client area at its scale.
#[derive(Default)]
pub struct Layout {
    pub design: (i32, i32),
    pub padding: i32,
    pub items: Vec<(HWND, Item)>,
    /// Split containers: the split, its bar's design distance, the controls in each pane.
    pub splits: Vec<(HWND, i32, Vec<HWND>, Vec<HWND>)>,
}

impl Layout {
    pub fn new(design: (i32, i32)) -> Layout {
        Layout { design, padding: 0, items: Vec::new(), splits: Vec::new() }
    }

    /// `split` (a [`crate::split::split`] already added) holds `first` in its first pane and
    /// `second` in its other: they keep their anchors to their pane as the bar moves.
    pub fn add_split(&mut self, split: HWND, first: &[HWND], second: &[HWND]) -> &mut Self {
        let d = crate::split::distance(split);
        self.splits.push((split, d, first.to_vec(), second.to_vec()));
        self
    }

    /// The rectangles `arrange` gave, with the controls in split panes placed in their panes.
    fn in_panes(&self, mut rects: Vec<Rect>) -> Vec<Rect> {
        let index = |h: HWND| self.items.iter().position(|(x, _)| *x == h);
        for (split, d0, first, second) in &self.splits {
            let Some(si) = index(*split) else { continue };
            let (b, r) = (self.items[si].1.bounds, rects[si]);
            let vertical = crate::split::is_vertical(*split);
            let (d1, d2) = crate::split::panes(vertical, *d0, b.w, b.h);
            let (a1, a2) = crate::split::panes(vertical, crate::split::distance(*split), r.w, r.h);
            for (members, dp, ap) in [(first, d1, a1), (second, d2, a2)] {
                for h in members {
                    let Some(i) = index(*h) else { continue };
                    let it = self.items[i].1;
                    let rel = Item { bounds: Rect::new(it.bounds.x - b.x - dp.0, it.bounds.y - b.y - dp.1, it.bounds.w, it.bounds.h), dock: Dock::None, anchor: it.anchor };
                    let p = arrange((dp.2, dp.3), (ap.2, ap.3), 0, &[rel])[0];
                    rects[i] = Rect::new(p.x + r.x + ap.0, p.y + r.y + ap.1, p.w, p.h);
                }
            }
        }
        rects
    }

    pub fn add(&mut self, h: HWND, item: Item) -> &mut Self {
        self.items.push((h, item));
        self
    }

    /// Places every window for `parent`'s client size now. Call from `WM_SIZE` and after a DPI
    /// change. Design pixels are scaled to the parent's DPI after arranging, so an anchored
    /// control keeps its distance in design pixels at any scale.
    pub fn apply(&self, parent: HWND) {
        let dpi = crate::win::dpi(parent);
        let mut c = RECT::default();
        unsafe {
            let _ = wm::GetClientRect(parent, &mut c);
        }
        let mut size = (unscale(c.right - c.left, dpi), unscale(c.bottom - c.top, dpi));
        // AutoScroll: never smaller than designed; scrolled to where its bars are.
        let scrolls = crate::form::scrolls(parent);
        if scrolls {
            size = (size.0.max(self.design.0), size.1.max(self.design.1));
        }
        let (ox, oy) = if scrolls { crate::form::update_scrollbars(parent, (scale(size.0, dpi), scale(size.1, dpi))) } else { (0, 0) };
        let items: Vec<Item> = self.items.iter().map(|(_, i)| *i).collect();
        let rects = self.in_panes(arrange(self.design, size, self.padding, &items));
        let mut combos = Vec::new();
        unsafe {
            let Ok(mut batch) = wm::BeginDeferWindowPos(rects.len() as i32) else { return };
            for ((h, _), r) in self.items.iter().zip(&rects) {
                let (x, y) = (scale(r.x, dpi), scale(r.y, dpi));
                let (w, hgt) = (scale(r.x + r.w, dpi) - x, scale(r.y + r.h, dpi) - y);
                let w = crate::win::auto_width(*h).unwrap_or(w);
                let (x, y) = (x - ox, y - oy);
                // A group frame goes to the back, whenever it was made: every sibling must clip it,
                // or it paints over what it gathers.
                let (after, flags) = if crate::win::kind_of(*h) == Some(crate::win::Kind::Group) {
                    (Some(wm::HWND_BOTTOM), wm::SWP_NOACTIVATE)
                } else {
                    (None, wm::SWP_NOZORDER | wm::SWP_NOACTIVATE)
                };
                // A drop-down: its window's height is its list's room; its box takes the height
                // designed (below).
                let hgt = if crate::win::kind_of(*h) == Some(crate::win::Kind::Combo) {
                    combos.push((*h, hgt));
                    hgt + scale(240, dpi)
                } else {
                    hgt
                };
                match wm::DeferWindowPos(batch, *h, after, x, y, w.max(0), hgt.max(0), flags) {
                    Ok(b) => batch = b,
                    Err(_) => return,
                }
            }
            let _ = wm::EndDeferWindowPos(batch);
        }
        for (h, height) in combos {
            crate::win::combo_box_height(h, height);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(x: i32, y: i32, w: i32, h: i32, dock: Dock, anchor: Anchor) -> Item {
        Item { bounds: Rect::new(x, y, w, h), dock, anchor }
    }

    #[test]
    fn at_its_design_size_everything_stays_where_it_was_put() {
        let items = [item(10, 10, 80, 24, Dock::None, Anchor::TOP_LEFT), item(300, 260, 75, 23, Dock::None, Anchor::BOTTOM_RIGHT), item(10, 40, 380, 200, Dock::None, Anchor::ALL)];
        let r = arrange((400, 300), (400, 300), 0, &items);
        assert_eq!(r, items.iter().map(|i| i.bounds).collect::<Vec<_>>());
    }

    #[test]
    fn anchors_keep_their_distance_to_the_edges_they_hold() {
        let items = [
            item(10, 10, 80, 24, Dock::None, Anchor::TOP_LEFT),
            item(300, 260, 75, 23, Dock::None, Anchor::BOTTOM_RIGHT),
            item(10, 40, 380, 200, Dock::None, Anchor::ALL),
            item(10, 250, 200, 20, Dock::None, Anchor::TOP_LEFT_RIGHT),
            item(150, 100, 100, 50, Dock::None, Anchor::NONE),
        ];
        let r = arrange((400, 300), (600, 400), 0, &items);
        assert_eq!(r[0], Rect::new(10, 10, 80, 24), "top-left does not move");
        assert_eq!(r[1], Rect::new(500, 360, 75, 23), "bottom-right keeps 25 px right and 17 px below");
        assert_eq!(r[2], Rect::new(10, 40, 580, 300), "all four edges: it stretches both ways");
        assert_eq!(r[3], Rect::new(10, 250, 400, 20), "left and right: wider, same height and top");
        assert_eq!(r[4], Rect::new(250, 150, 100, 50), "no anchor: still centred");
    }

    #[test]
    fn a_window_made_smaller_than_designed_never_gives_a_negative_size() {
        let r = arrange((400, 300), (100, 50), 0, &[item(10, 40, 380, 200, Dock::None, Anchor::ALL)]);
        assert_eq!(r[0], Rect::new(10, 40, 80, 0));
    }

    #[test]
    fn docking_takes_strips_in_order_and_fill_takes_the_rest() {
        let items = [
            item(0, 0, 0, 30, Dock::Top, Anchor::default()),    // toolbar
            item(0, 0, 0, 22, Dock::Bottom, Anchor::default()), // status bar
            item(0, 0, 150, 0, Dock::Left, Anchor::default()),  // toolbox
            item(0, 0, 200, 0, Dock::Right, Anchor::default()), // properties
            item(0, 0, 0, 0, Dock::Fill, Anchor::default()),    // the form
        ];
        let r = arrange((800, 600), (1000, 700), 0, &items);
        assert_eq!(r[0], Rect::new(0, 0, 1000, 30));
        assert_eq!(r[1], Rect::new(0, 678, 1000, 22));
        assert_eq!(r[2], Rect::new(0, 30, 150, 648));
        assert_eq!(r[3], Rect::new(800, 30, 200, 648));
        assert_eq!(r[4], Rect::new(150, 30, 650, 648));
    }

    #[test]
    fn padding_is_kept_clear_and_a_strip_never_takes_more_than_is_left() {
        let items = [item(0, 0, 0, 500, Dock::Top, Anchor::default()), item(0, 0, 0, 0, Dock::Fill, Anchor::default())];
        let r = arrange((100, 100), (200, 100), 8, &items);
        assert_eq!(r[0], Rect::new(8, 8, 184, 84));
        assert_eq!(r[1], Rect::new(8, 92, 184, 0));
    }

    #[test]
    fn scaling_rounds_the_same_way_both_ways() {
        assert_eq!(scale(75, 192), 150);
        assert_eq!(scale(75, 144), 112);
        assert_eq!(unscale(150, 192), 75);
        assert_eq!(unscale(1, 0), 96, "a DPI of 0 does not divide by zero");
    }
}

/// A flow panel's places, as WinForms' FlowLayoutPanel: each item at its size, left to right,
/// on to the next row when the row is full - `gap` between items and round the edge.
pub fn flow(area: Rect, sizes: &[(i32, i32)], gap: i32) -> Vec<(i32, i32)> {
    let mut out = Vec::with_capacity(sizes.len());
    let (mut x, mut y, mut row_h) = (area.x + gap, area.y + gap, 0);
    for &(w, h) in sizes {
        if x > area.x + gap && x + w > area.x + area.w - gap {
            x = area.x + gap;
            y += row_h + gap;
            row_h = 0;
        }
        out.push((x, y));
        x += w + gap;
        row_h = row_h.max(h);
    }
    out
}

/// A flow panel's places with WinForms' FlowDirection and WrapContents: `down`, top to bottom and
/// on to the next column; `wrap` off, one row (or column) however long.
pub fn flow_with(area: Rect, sizes: &[(i32, i32)], gap: i32, down: bool, wrap: bool) -> Vec<(i32, i32)> {
    if !down {
        let wide = if wrap { area } else { Rect { w: i32::MAX / 4, ..area } };
        return flow(wide, sizes, gap);
    }
    let mut out = Vec::with_capacity(sizes.len());
    let (mut x, mut y, mut col_w) = (area.x + gap, area.y + gap, 0);
    for &(w, h) in sizes {
        if wrap && y > area.y + gap && y + h > area.y + area.h - gap {
            y = area.y + gap;
            x += col_w + gap;
            col_w = 0;
        }
        out.push((x, y));
        y += h + gap;
        col_w = col_w.max(w);
    }
    out
}

/// A table panel's places, as WinForms' TableLayoutPanel: `cols` equal columns across the area,
/// items in order row by row, each as wide as its cell; a row as tall as its tallest item.
pub fn table(area: Rect, sizes: &[(i32, i32)], cols: usize, gap: i32) -> Vec<Rect> {
    let cols = cols.max(1);
    let cell_w = ((area.w - gap * (cols as i32 + 1)) / cols as i32).max(1);
    let mut out = Vec::with_capacity(sizes.len());
    let mut y = area.y + gap;
    for row in sizes.chunks(cols) {
        let row_h = row.iter().map(|s| s.1).max().unwrap_or(0);
        for (k, &(_, h)) in row.iter().enumerate() {
            out.push(Rect::new(area.x + gap + k as i32 * (cell_w + gap), y, cell_w, h));
        }
        y += row_h + gap;
    }
    out
}

#[cfg(test)]
mod arrange_tests {
    use super::*;

    #[test]
    fn flow_wraps_and_table_fills_its_columns() {
        let area = Rect::new(0, 0, 200, 300);
        // 80 + 80 fit a 200 row with gaps of 6; the third goes down.
        let p = flow(area, &[(80, 20), (80, 30), (80, 20)], 6);
        assert_eq!(p, vec![(6, 6), (92, 6), (6, 42)]);
        let t = table(area, &[(50, 20), (50, 25), (50, 20)], 2, 6);
        assert_eq!((t[0].x, t[0].w, t[1].x, t[2].y), (6, 91, 103, 37), "two columns of 91, the second row below the taller");
    }
}
