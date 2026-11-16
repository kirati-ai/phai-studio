//! Lining controls up, as Visual Studio's Format menu does: against the *primary* control - the
//! first one chosen - for edges and sizes, and between the outermost ones for even spacing. Pure
//! arithmetic on `[x, y, w, h]`, so it is tested rather than eyeballed.
//!
//! Also the clipboard's text for controls: TOML `[[control]]` tables, the same as in a form, so
//! a copy can be pasted into another form, or into a text editor and back.

use serde::{Deserialize, Serialize};

use crate::model::{Control, FormFile};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Lefts,
    Centers,
    Rights,
    Tops,
    Middles,
    Bottoms,
    SameWidth,
    SameHeight,
    SameSize,
    SpaceAcross,
    SpaceDown,
    /// Each one's corner onto the grid.
    AlignToGrid,
    /// Each one's size onto the grid.
    SizeToGrid,
    /// The gaps across one grid step wider, narrower, or gone - the first stays.
    MoreAcross,
    LessAcross,
    NoneAcross,
    MoreDown,
    LessDown,
    NoneDown,
    /// The group moved to the middle of the form, across or down.
    CenterAcross,
    CenterDown,
}

impl Op {
    /// Whether it does anything to one control alone - the others need two or more.
    pub fn works_alone(self) -> bool {
        matches!(self, Op::AlignToGrid | Op::SizeToGrid | Op::CenterAcross | Op::CenterDown)
    }
}

/// Moves and sizes `rects` by `op` within a form of `form` size, on a grid of `grid` pixels;
/// `primary` is the index of the one the others follow.
pub fn arrange_in(rects: &mut [[i32; 4]], primary: usize, op: Op, form: [i32; 2], grid: i32) {
    let grid = grid.max(1);
    let snap = |v: i32| (v + grid / 2).div_euclid(grid) * grid;
    match op {
        Op::AlignToGrid => rects.iter_mut().for_each(|r| {
            r[0] = snap(r[0]);
            r[1] = snap(r[1]);
        }),
        Op::SizeToGrid => rects.iter_mut().for_each(|r| {
            r[2] = snap(r[2]).max(grid);
            r[3] = snap(r[3]).max(grid);
        }),
        Op::CenterAcross | Op::CenterDown if !rects.is_empty() => {
            let (pos, len, room) = if op == Op::CenterAcross { (0, 2, form[0]) } else { (1, 3, form[1]) };
            let lo = rects.iter().map(|r| r[pos]).min().unwrap_or(0);
            let hi = rects.iter().map(|r| r[pos] + r[len]).max().unwrap_or(0);
            let d = (room - (hi - lo)) / 2 - lo;
            rects.iter_mut().for_each(|r| r[pos] += d);
        }
        Op::MoreAcross | Op::LessAcross | Op::NoneAcross | Op::MoreDown | Op::LessDown | Op::NoneDown if rects.len() >= 2 => {
            let down = matches!(op, Op::MoreDown | Op::LessDown | Op::NoneDown);
            let (pos, len) = if down { (1, 3) } else { (0, 2) };
            let mut order: Vec<usize> = (0..rects.len()).collect();
            order.sort_by_key(|i| rects[*i][pos]);
            if matches!(op, Op::NoneAcross | Op::NoneDown) {
                // Side by side from the first, no room between.
                let mut at = rects[order[0]][pos];
                for i in order {
                    rects[i][pos] = at;
                    at += rects[i][len];
                }
            } else {
                // Each gap a step wider or narrower; the primary stays where it is.
                let step = if matches!(op, Op::MoreAcross | Op::MoreDown) { grid } else { -grid };
                let p = order.iter().position(|i| *i == primary).unwrap_or(0) as i32;
                for (k, i) in order.into_iter().enumerate() {
                    rects[i][pos] += (k as i32 - p) * step;
                }
            }
        }
        _ => arrange(rects, primary, op),
    }
}

/// Moves and sizes `rects` by `op`; `primary` is the index of the one the others follow.
pub fn arrange(rects: &mut [[i32; 4]], primary: usize, op: Op) {
    if rects.len() < 2 || primary >= rects.len() {
        return;
    }
    let [px, py, pw, ph] = rects[primary];
    match op {
        Op::Lefts => rects.iter_mut().for_each(|r| r[0] = px),
        Op::Rights => rects.iter_mut().for_each(|r| r[0] = px + pw - r[2]),
        Op::Centers => rects.iter_mut().for_each(|r| r[0] = px + (pw - r[2]) / 2),
        Op::Tops => rects.iter_mut().for_each(|r| r[1] = py),
        Op::Bottoms => rects.iter_mut().for_each(|r| r[1] = py + ph - r[3]),
        Op::Middles => rects.iter_mut().for_each(|r| r[1] = py + (ph - r[3]) / 2),
        Op::SameWidth => rects.iter_mut().for_each(|r| r[2] = pw),
        Op::SameHeight => rects.iter_mut().for_each(|r| r[3] = ph),
        Op::SameSize => rects.iter_mut().for_each(|r| {
            r[2] = pw;
            r[3] = ph;
        }),
        Op::SpaceAcross | Op::SpaceDown => {
            let (pos, len) = if op == Op::SpaceAcross { (0, 2) } else { (1, 3) };
            if rects.len() < 3 {
                return;
            }
            // The outermost stay; the rest share the room between them equally.
            let mut order: Vec<usize> = (0..rects.len()).collect();
            order.sort_by_key(|i| rects[*i][pos]);
            let first = rects[order[0]];
            let last = rects[*order.last().unwrap()];
            let total: i32 = order.iter().map(|i| rects[*i][len]).sum();
            let room = last[pos] + last[len] - first[pos] - total;
            let gaps = (order.len() - 1) as i32;
            let mut at = first[pos];
            for (k, i) in order.iter().enumerate() {
                if k > 0 && k < order.len() - 1 {
                    rects[*i][pos] = at;
                }
                // Spread the remainder over the first gaps, so the last one still lands exactly.
                at += rects[*i][len] + room / gaps + if (k as i32) < room % gaps { 1 } else { 0 };
            }
        }
        _ => arrange_in(rects, primary, op, [0, 0], 8),
    }
}

/// How close an edge must come to another, design pixels, to be drawn to it.
pub const SNAP: i32 = 5;

/// Snaplines, as Visual Studio's designer draws them: where a moving rectangle's edges come
/// within [`SNAP`] of another control's edges or of the form's margin. Returns how far to move it
/// on to touch them, and the lines to draw - `(true, x)` a vertical line at x, `(false, y)` a
/// horizontal one.
pub fn snaplines(moving: [i32; 4], others: &[[i32; 4]], form: [i32; 2], margin: i32) -> ((i32, i32), Vec<(bool, i32)>) {
    let [x, y, w, h] = moving;
    let mut xs: Vec<i32> = vec![margin, form[0] - margin];
    let mut ys: Vec<i32> = vec![margin, form[1] - margin];
    for o in others {
        xs.extend([o[0], o[0] + o[2]]);
        ys.extend([o[1], o[1] + o[3]]);
    }
    // The nearest line to either edge, if near enough: (shift, line).
    let best = |a: i32, b: i32, lines: &[i32]| {
        lines
            .iter()
            .flat_map(|&l| [(l - a, l), (l - b, l)])
            .filter(|(d, _)| d.abs() <= SNAP)
            .min_by_key(|(d, _)| d.abs())
    };
    let mut lines = Vec::new();
    let dx = best(x, x + w, &xs).map(|(d, l)| {
        lines.push((true, l));
        d
    });
    let dy = best(y, y + h, &ys).map(|(d, l)| {
        lines.push((false, l));
        d
    });
    ((dx.unwrap_or(0), dy.unwrap_or(0)), lines)
}

/// Ctrl+arrow, as Visual Studio's designer: how far to go in direction `dir` (-1 or 1, across
/// when `across`) for an edge of `moving` to reach the next line another control's edge or the
/// form's margin makes. Sizing (`size`) moves only the right or bottom edge. Without a line ahead,
/// one pixel.
pub fn next_stop(moving: [i32; 4], others: &[[i32; 4]], form: [i32; 2], margin: i32, across: bool, dir: i32, size: bool) -> i32 {
    let (pos, len, extent) = if across { (moving[0], moving[2], form[0]) } else { (moving[1], moving[3], form[1]) };
    let mut lines = vec![margin, extent - margin];
    for o in others {
        let (p, l) = if across { (o[0], o[2]) } else { (o[1], o[3]) };
        lines.extend([p, p + l]);
    }
    let edges: Vec<i32> = if size { vec![pos + len] } else { vec![pos, pos + len] };
    lines
        .iter()
        .flat_map(|l| edges.iter().map(move |e| (l - e) * dir))
        .filter(|d| *d > 0)
        .min()
        .map(|d| d * dir)
        .unwrap_or(dir)
}

#[derive(Serialize, Deserialize)]
struct Clip {
    control: Vec<Control>,
}

/// Controls as clipboard text.
pub fn to_clipboard(controls: &[Control]) -> String {
    toml::to_string(&Clip { control: controls.to_vec() }).unwrap_or_default()
}

/// Controls from clipboard text, ready to add to `form`: ids already in it get new ones, and each
/// moves `offset` right and down so the copy does not hide the original. `None` when the text is
/// not controls.
pub fn from_clipboard(text: &str, form: &FormFile, offset: i32) -> Option<Vec<Control>> {
    let clip: Clip = toml::from_str(text).ok()?;
    if clip.control.is_empty() {
        return None;
    }
    let mut taken: Vec<String> = form.control.iter().map(|c| c.id.clone()).collect();
    let mut out = Vec::new();
    for mut c in clip.control {
        if taken.contains(&c.id) {
            let base = c.id.trim_end_matches(|ch: char| ch.is_ascii_digit()).to_string();
            let base = if base.is_empty() { c.kind.name().to_string() } else { base };
            c.id = (1..).map(|n| format!("{base}{n}")).find(|id| !taken.contains(id)).unwrap();
        }
        taken.push(c.id.clone());
        if c.dock.is_none() {
            if let Some(b) = c.bounds.as_mut() {
                b[0] += offset;
                b[1] += offset;
            }
        }
        out.push(c);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Kind;

    fn r() -> Vec<[i32; 4]> {
        vec![[10, 10, 75, 23], [40, 50, 100, 30], [20, 100, 50, 20]]
    }

    #[test]
    fn edges_and_centres_follow_the_primary() {
        let mut v = r();
        arrange(&mut v, 1, Op::Lefts);
        assert!(v.iter().all(|x| x[0] == 40));
        let mut v = r();
        arrange(&mut v, 1, Op::Rights);
        assert_eq!(v.iter().map(|x| x[0] + x[2]).collect::<Vec<_>>(), vec![140, 140, 140]);
        let mut v = r();
        arrange(&mut v, 0, Op::Middles);
        assert_eq!(v[1][1], 10 + (23 - 30) / 2);
        let mut v = r();
        arrange(&mut v, 2, Op::Bottoms);
        assert!(v.iter().all(|x| x[1] + x[3] == 120));
    }

    #[test]
    fn sizes_follow_the_primary() {
        let mut v = r();
        arrange(&mut v, 0, Op::SameSize);
        assert!(v.iter().all(|x| x[2] == 75 && x[3] == 23));
        let mut v = r();
        arrange(&mut v, 1, Op::SameWidth);
        assert!(v.iter().all(|x| x[2] == 100));
        assert_eq!(v[0][3], 23, "height untouched");
    }

    #[test]
    fn spacing_leaves_the_outer_ones_and_evens_the_gaps() {
        // Three buttons across: 0..75, 100..175 (crowded right), 300..375.
        let mut v = vec![[0, 0, 75, 23], [100, 0, 75, 23], [300, 0, 75, 23]];
        arrange(&mut v, 0, Op::SpaceAcross);
        assert_eq!(v[0][0], 0);
        assert_eq!(v[2][0], 300);
        let gap1 = v[1][0] - (v[0][0] + 75);
        let gap2 = v[2][0] - (v[1][0] + 75);
        assert!((gap1 - gap2).abs() <= 1, "{gap1} vs {gap2}");
        let mut two = vec![[0, 0, 10, 10], [50, 0, 10, 10]];
        arrange(&mut two, 0, Op::SpaceAcross);
        assert_eq!(two, vec![[0, 0, 10, 10], [50, 0, 10, 10]], "two need no spacing");
    }

    #[test]
    fn edges_are_drawn_to_other_edges_and_the_margin() {
        // A button 3 px right of another's left edge: drawn back to it, a vertical line there.
        let ((dx, dy), lines) = snaplines([103, 50, 75, 23], &[[100, 10, 60, 23]], [480, 320], 11);
        assert_eq!((dx, dy), (-3, 0));
        assert_eq!(lines, vec![(true, 100)]);
        // Near the form's right margin with its right edge.
        let ((dx, _), _) = snaplines([392, 200, 75, 23], &[], [480, 320], 11);
        assert_eq!(392 + dx + 75, 480 - 11);
        // Far from everything: nothing.
        assert_eq!(snaplines([200, 150, 50, 20], &[[10, 10, 20, 20]], [480, 320], 11), ((0, 0), vec![]));
    }

    #[test]
    fn ctrl_arrows_go_to_the_next_line() {
        let others = [[200, 50, 60, 23]];
        // Right: the left edge (100) reaches no line before the other's left at 200, but the right
        // edge (175) reaches 200 first.
        assert_eq!(next_stop([100, 10, 75, 23], &others, [480, 320], 11, true, 1, false), 25);
        // Left from 100: the margin at 11.
        assert_eq!(next_stop([100, 10, 75, 23], &others, [480, 320], 11, true, -1, false), -89);
        // Sizing right: only the right edge, to 200.
        assert_eq!(next_stop([100, 10, 75, 23], &others, [480, 320], 11, true, 1, true), 25);
        // Down: the top (20) to the other's top at 50 is 30, the bottom (43) to 50 is 7.
        assert_eq!(next_stop([100, 20, 75, 23], &others, [480, 320], 11, false, 1, false), 7);
        // Past every line: a pixel.
        assert_eq!(next_stop([470, 10, 10, 10], &[], [480, 320], 11, true, 1, false), 1);
    }

    #[test]
    fn a_copy_comes_back_with_free_ids_and_a_little_lower() {
        let mut f = FormFile::new("Main");
        f.control.push(Control::new(Kind::Button, "ok", [10, 10, 75, 23]));
        f.control.push(Control::new(Kind::Button, "button1", [10, 40, 75, 23]));
        let text = to_clipboard(&f.control);
        let pasted = from_clipboard(&text, &f, 8).unwrap();
        assert_eq!(pasted.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), vec!["ok1", "button2"]);
        assert_eq!(pasted[0].bounds, Some([18, 18, 75, 23]));
        assert!(from_clipboard("hello", &f, 8).is_none());
        assert!(from_clipboard("", &f, 8).is_none());
    }

    #[test]
    fn grid_spacing_and_centring_as_visual_studios_layout_bar() {
        let mut v = vec![[13, 18, 75, 21], [100, 18, 75, 23], [200, 18, 75, 23]];
        arrange_in(&mut v, 0, Op::AlignToGrid, [480, 360], 8);
        assert_eq!((v[0][0], v[0][1]), (16, 16));
        arrange_in(&mut v, 0, Op::SizeToGrid, [480, 360], 8);
        assert_eq!((v[0][2], v[0][3]), (72, 24));
        let mut v = vec![[10, 0, 50, 20], [70, 0, 50, 20], [130, 0, 50, 20]];
        arrange_in(&mut v, 0, Op::MoreAcross, [480, 360], 8);
        assert_eq!([v[0][0], v[1][0], v[2][0]], [10, 78, 146]);
        arrange_in(&mut v, 1, Op::LessAcross, [480, 360], 8);
        assert_eq!([v[0][0], v[1][0], v[2][0]], [18, 78, 138]);
        arrange_in(&mut v, 0, Op::NoneAcross, [480, 360], 8);
        assert_eq!([v[0][0], v[1][0], v[2][0]], [18, 68, 118]);
        // The group's span (18..168) put in the middle of 480.
        arrange_in(&mut v, 0, Op::CenterAcross, [480, 360], 8);
        assert_eq!(v[0][0], 165);
        assert_eq!(v[2][0] + v[2][2] - v[0][0], 150);
        let mut one = vec![[0, 100, 40, 20]];
        arrange_in(&mut one, 0, Op::CenterDown, [480, 360], 8);
        assert_eq!(one[0][1], 170);
        assert!(Op::CenterDown.works_alone() && !Op::Lefts.works_alone());
    }
}
