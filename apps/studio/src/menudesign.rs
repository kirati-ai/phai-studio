//! The form's menus designed on the form, as Visual Studio's MenuStrip designer: the menu bar
//! over the form's own, each menu dropped down under it, "Type Here" at the end of the bar, of
//! each menu and beside an item for a menu within it. A click chooses, a click on the chosen one
//! (or Enter, F2, or typing) edits its words in place; "-" makes a separator; Delete takes it out,
//! Ctrl+Up/Down moves it; a right-click has Insert, Move, Delete and Properties (id, shortcut,
//! picture, checked, disabled). Every change is the form's at once, one undo step each; a click
//! anywhere else closes it.

use phai_form::{Menu, MenuItem};
use phai_ui::text::{t, tf};
use phai_ui::theme::{self, Color};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::Input::KeyboardAndMouse as km;
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging as wm;

use crate::studio;

/// The colour that is not there: what is outside the bar and the menus lets clicks through.
const CLEAR: COLORREF = COLORREF(0x00FF00FF);
const EDIT_ID: i32 = 0x7E10;

/// A place in the designer: a menu on the bar, or an item of the menu open at a level (`index ==
/// len`: its "Type Here"), or the "Type Here" beside an item for a menu within it.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Slot {
    Top(usize),
    Item { level: usize, index: usize },
    Sub { level: usize, index: usize },
}

/// A cell drawn and clicked.
struct Cell {
    r: RECT,
    slot: Slot,
}

struct Designer {
    menus: Vec<Menu>,
    /// The menus open: the bar's (`path[0]`), then the item whose own menu is open at each level.
    path: Vec<usize>,
    sel: Option<Slot>,
    /// The box editing a slot's words.
    edit: Option<(HWND, Slot)>,
    /// The menu bar on the screen and each menu's edges on it - the form's, from the canvas.
    bar: RECT,
    tops: Vec<(i32, i32)>,
    cells: Vec<Cell>,
    panels: Vec<RECT>,
    /// The window's top-left on the screen.
    at: POINT,
    font: Gdi::HFONT,
    /// A dialog or the form being rebuilt: losing the focus then does not close it.
    busy: bool,
    /// A click on the chosen slot: its words edited once no second click makes it a double-click.
    pending: Option<Slot>,
    done: bool,
}

fn state<'a>(h: HWND) -> Option<&'a mut Designer> {
    unsafe { win::state::<Designer>(h) }
}

static OPEN: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

/// Whether the designer is open - its chip in the tray shown chosen.
pub fn is_open() -> bool {
    OPEN.load(std::sync::atomic::Ordering::Relaxed) != 0
}

/// Opens the designer over the form's menu bar, menu `open` dropped down (the "Type Here" of an
/// empty bar chosen when there is none).
pub fn open(open: Option<usize>) {
    let Some(s) = studio() else { return };
    let Some(c) = s.canvas.as_ref() else { return };
    let old = HWND(OPEN.load(std::sync::atomic::Ordering::Relaxed) as *mut _);
    if !old.is_invalid() && unsafe { wm::IsWindow(Some(old)) }.as_bool() {
        unsafe {
            let _ = wm::DestroyWindow(old);
        }
    }
    let menus = s.doc.form.menu.clone();
    let (bar, tops) = c.menu_bar_screen(&s.doc.form);
    win::init();
    // Double-clicks heard: one on an item opens its Click method.
    let class = win::register_with(win::ClassStyle::DoubleClicks, "PhaiMenuDesigner", Some(proc_), None);
    unsafe {
        let h = wm::CreateWindowExW(wm::WS_EX_LAYERED | wm::WS_EX_TOOLWINDOW, PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_POPUP, bar.left, bar.top, 10, 10, Some(s.hwnd), None, Some(win::instance()), None).unwrap_or_default();
        let _ = wm::SetLayeredWindowAttributes(h, CLEAR, 0, wm::LWA_COLORKEY);
        let font = win::font(h, 0, false);
        let path = match open {
            Some(i) if i < menus.len() => vec![i],
            _ => vec![],
        };
        let sel = match open {
            Some(i) if i < menus.len() => Some(Slot::Top(i)),
            _ => Some(Slot::Top(menus.len())),
        };
        win::set_state(h, Box::new(Designer { menus, path, sel, edit: None, bar, tops, cells: vec![], panels: vec![], at: POINT { x: bar.left, y: bar.top }, font, busy: false, pending: None, done: false }));
        OPEN.store(h.0 as isize, std::sync::atomic::Ordering::Relaxed);
        crate::canvas::refresh_tray();
        relayout(h);
        let _ = wm::ShowWindow(h, wm::SW_SHOW);
        let _ = km::SetFocus(Some(h));
        // An empty bar: its first menu's words typed at once.
        if open.is_none_or(|i| i >= state(h).map(|d| d.menus.len()).unwrap_or(0)) {
            begin_edit(h, Slot::Top(state(h).map(|d| d.menus.len()).unwrap_or(0)), None);
        }
    }
}

/// The designer's copy of the menus taken again from the form - changed in Properties.
pub fn reload() {
    let h = HWND(OPEN.load(std::sync::atomic::Ordering::Relaxed) as *mut _);
    if h.is_invalid() {
        return;
    }
    if let (Some(d), Some(s)) = (state(h), studio()) {
        d.menus = s.doc.form.menu.clone();
        relayout(h);
    }
}

/// What is chosen, for Properties: a menu, an item, or the bar.
fn tell(d: &Designer) {
    let sel = match d.sel {
        Some(Slot::Top(i)) if i < d.menus.len() => crate::BarSel::Menu(i),
        Some(Slot::Item { level, index }) if list(&d.menus, &d.path, level).is_some_and(|l| index < l.len()) => {
            let mut p: Vec<usize> = d.path[..=level].to_vec();
            p.push(index);
            crate::BarSel::MenuItem(p)
        }
        _ => crate::BarSel::MenuStrip,
    };
    if let Some(s) = studio() {
        if s.bar_sel.as_ref() != Some(&sel) {
            s.choose_bar(Some(sel));
        }
    }
}

/// The items of the menu open at `level`.
fn list<'a>(menus: &'a [Menu], path: &[usize], level: usize) -> Option<&'a Vec<MenuItem>> {
    let mut items = &menus.get(*path.first()?)?.items;
    for k in 1..=level {
        items = &items.get(*path.get(k)?)?.items;
    }
    Some(items)
}

fn list_mut<'a>(menus: &'a mut [Menu], path: &[usize], level: usize) -> Option<&'a mut Vec<MenuItem>> {
    let mut items = &mut menus.get_mut(*path.first()?)?.items;
    for k in 1..=level {
        items = &mut items.get_mut(*path.get(k)?)?.items;
    }
    Some(items)
}

fn px(h: HWND, v: i32) -> i32 {
    win::px(h, v)
}

fn text_w(h: HWND, font: Gdi::HFONT, s: &str) -> i32 {
    win::measure(h, font, &phai_form::bars::shown(s)).0
}

/// Where everything is, worked out again: the bar's menus and "Type Here", each open menu under
/// or beside its parent, the window as large as all of them.
fn relayout(h: HWND) {
    let Some(d) = state(h) else { return };
    let mut cells = Vec::new();
    let mut panels = Vec::new();
    let bar = d.bar;
    // The bar: the menus where the form shows them, then "Type Here".
    let mut x = bar.left + px(h, 8);
    let mut tops = Vec::new();
    for m in &d.menus {
        let w = text_w(h, d.font, &m.label) + px(h, 16);
        tops.push((x, x + w));
        x += w;
    }
    d.tops = tops;
    for (i, (l, r)) in d.tops.iter().enumerate() {
        cells.push(Cell { r: RECT { left: *l, top: bar.top, right: *r, bottom: bar.bottom }, slot: Slot::Top(i) });
    }
    let new_w = text_w(h, d.font, t("Type Here")) + px(h, 20);
    cells.push(Cell { r: RECT { left: x, top: bar.top + px(h, 2), right: x + new_w, bottom: bar.bottom - px(h, 2) }, slot: Slot::Top(d.menus.len()) });
    // The menus open, one a level.
    let row = px(h, 26);
    let sep = px(h, 9);
    let (mut px0, mut py0) = (0, bar.bottom);
    if let Some(&top) = d.path.first() {
        px0 = d.tops.get(top).map(|t| t.0).unwrap_or(bar.left);
    }
    let mut level = 0;
    while let Some(items) = (!d.path.is_empty()).then(|| list(&d.menus, &d.path, level)).flatten() {
        let mut w = text_w(h, d.font, t("Type Here")) + px(h, 60);
        for it in items {
            let words = text_w(h, d.font, it.label.as_deref().unwrap_or("")) + it.key.as_deref().map(|k| text_w(h, d.font, k) + px(h, 30)).unwrap_or(0);
            w = w.max(words + px(h, 70));
        }
        w = w.max(px(h, 180));
        let mut y = py0 + px(h, 3);
        let mut open_row = None;
        for (i, it) in items.iter().enumerate() {
            let hh = if it.separator { sep } else { row };
            cells.push(Cell { r: RECT { left: px0 + px(h, 3), top: y, right: px0 + w - px(h, 3), bottom: y + hh }, slot: Slot::Item { level, index: i } });
            if d.path.get(level + 1) == Some(&i) {
                open_row = Some(y);
            }
            // The chosen item at the deepest level, with no menu of its own: "Type Here" beside it.
            if d.sel == Some(Slot::Item { level, index: i }) && !it.separator && it.items.is_empty() && d.path.len() == level + 1 {
                let sw = text_w(h, d.font, t("Type Here")) + px(h, 24);
                let r = RECT { left: px0 + w - px(h, 1), top: y, right: px0 + w + sw, bottom: y + row };
                panels.push(RECT { top: r.top - px(h, 3), bottom: r.bottom + px(h, 3), ..r });
                cells.push(Cell { r: RECT { left: r.left + px(h, 3), right: r.right - px(h, 3), ..r }, slot: Slot::Sub { level, index: i } });
            }
            y += hh;
        }
        cells.push(Cell { r: RECT { left: px0 + px(h, 3), top: y, right: px0 + w - px(h, 3), bottom: y + row }, slot: Slot::Item { level, index: items.len() } });
        y += row + px(h, 3);
        panels.insert(level, RECT { left: px0, top: py0, right: px0 + w, bottom: y });
        match open_row {
            Some(oy) => {
                px0 += w - px(h, 2);
                py0 = oy - px(h, 3);
                level += 1;
            }
            None => break,
        }
    }
    // The window: everything, the clear colour between.
    let mut all = bar;
    for r in cells.iter().map(|c| c.r).chain(panels.iter().copied()) {
        all.left = all.left.min(r.left);
        all.top = all.top.min(r.top);
        all.right = all.right.max(r.right);
        all.bottom = all.bottom.max(r.bottom);
    }
    let margin = px(h, 4);
    all.right += margin;
    all.bottom += margin;
    d.cells = cells;
    d.panels = panels;
    tell(d);
    d.at = POINT { x: all.left, y: all.top };
    unsafe {
        let _ = wm::SetWindowPos(h, Some(wm::HWND_TOP), all.left, all.top, all.right - all.left, all.bottom - all.top, wm::SWP_NOACTIVATE);
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
    place_edit(h);
}

/// A screen rectangle in the window's own coordinates.
fn local(d: &Designer, r: RECT) -> RECT {
    RECT { left: r.left - d.at.x, top: r.top - d.at.y, right: r.right - d.at.x, bottom: r.bottom - d.at.y }
}

fn paint(h: HWND) {
    let Some(d) = state(h) else { return };
    let mut ps = Gdi::PAINTSTRUCT::default();
    let dc = unsafe { Gdi::BeginPaint(h, &mut ps) };
    let mut cr = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut cr);
    }
    // Drawn off screen, then shown at once.
    let mem = unsafe { Gdi::CreateCompatibleDC(Some(dc)) };
    let bmp = unsafe { Gdi::CreateCompatibleBitmap(dc, cr.right, cr.bottom) };
    let old_bmp = unsafe { Gdi::SelectObject(mem, bmp.into()) };
    unsafe {
        let b = Gdi::CreateSolidBrush(CLEAR);
        Gdi::FillRect(mem, &cr, b);
        let _ = Gdi::DeleteObject(b.into());
    }
    let window = win::window_background();
    let text = win::window_text();
    let grey = Color::rgb(140, 140, 140);
    let border = theme::style("group", phai_ui::theme::State::default()).border_or(Color::rgb(212, 212, 212));
    let accent = theme::current().var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f));
    let tint = if window.is_dark() { Color::rgb(40, 64, 56) } else { Color::rgb(231, 243, 239) };
    // The bar, over the form's.
    win::fill(mem, &local(d, d.bar), window);
    win::fill(mem, &RECT { top: local(d, d.bar).bottom - 1, ..local(d, d.bar) }, border);
    for p in &d.panels {
        let r = local(d, *p);
        win::fill(mem, &r, border);
        win::fill(mem, &RECT { left: r.left + 1, top: r.top + 1, right: r.right - 1, bottom: r.bottom - 1 }, window);
    }
    let old_font = unsafe { Gdi::SelectObject(mem, d.font.into()) };
    unsafe {
        Gdi::SetBkMode(mem, Gdi::TRANSPARENT);
    }
    let draw = |r: RECT, s: &str, colour: Color, flags: Gdi::DRAW_TEXT_FORMAT| {
        let mut w: Vec<u16> = s.encode_utf16().collect();
        let mut rr = r;
        unsafe {
            Gdi::SetTextColor(mem, COLORREF(colour.colorref()));
            Gdi::DrawTextW(mem, &mut w, &mut rr, flags | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_END_ELLIPSIS);
        }
    };
    let glyph = |g: u16, x: i32, r: RECT, colour: Color| {
        let size = px(h, 14);
        if let Some(i) = phai_ui::icon::glyph_icon(g, size, colour) {
            unsafe {
                let _ = wm::DrawIconEx(mem, x, r.top + (r.bottom - r.top - size) / 2, i, size, size, 0, None, wm::DI_NORMAL);
                let _ = wm::DestroyIcon(i);
            }
        }
    };
    for c in &d.cells {
        let r = local(d, c.r);
        let chosen = d.sel == Some(c.slot);
        let open = match c.slot {
            Slot::Top(i) => d.path.first() == Some(&i),
            Slot::Item { level, index } => d.path.get(level + 1) == Some(&index),
            _ => false,
        };
        let type_here = match c.slot {
            Slot::Top(i) => i >= d.menus.len(),
            Slot::Item { level, index } => list(&d.menus, &d.path, level).is_none_or(|l| index >= l.len()),
            Slot::Sub { .. } => true,
        };
        if chosen || open {
            win::fill(mem, &r, if chosen { accent } else { border });
            win::fill(mem, &RECT { left: r.left + 1, top: r.top + 1, right: r.right - 1, bottom: r.bottom - 1 }, tint);
        } else if type_here {
            // A dashed-looking box: a light frame.
            win::fill(mem, &r, border);
            win::fill(mem, &RECT { left: r.left + 1, top: r.top + 1, right: r.right - 1, bottom: r.bottom - 1 }, window);
        }
        if type_here {
            draw(RECT { left: r.left + px(h, 8), ..r }, t("Type Here"), grey, Gdi::DT_LEFT | Gdi::DT_NOPREFIX);
            continue;
        }
        match c.slot {
            Slot::Top(i) => draw(r, &d.menus[i].label, text, Gdi::DT_CENTER),
            Slot::Item { level, index } => {
                let Some(it) = list(&d.menus, &d.path, level).and_then(|l| l.get(index)) else { continue };
                if it.separator {
                    let y = (r.top + r.bottom) / 2;
                    win::fill(mem, &RECT { left: r.left + px(h, 28), top: y, right: r.right - px(h, 6), bottom: y + 1 }, border);
                    continue;
                }
                let colour = if it.disabled { grey } else { text };
                if it.checked {
                    glyph(0xE73E, r.left + px(h, 6), r, colour);
                } else if let Some(g) = it.glyph.as_deref().and_then(phai_ui::icon::glyph::by_name) {
                    glyph(g, r.left + px(h, 6), r, colour);
                }
                draw(RECT { left: r.left + px(h, 28), right: r.right - px(h, 24), ..r }, it.label.as_deref().unwrap_or(""), colour, Gdi::DT_LEFT);
                if let Some(k) = &it.key {
                    draw(RECT { right: r.right - px(h, 24), ..r }, k, grey, Gdi::DT_RIGHT | Gdi::DT_NOPREFIX);
                }
                if !it.items.is_empty() {
                    glyph(0xE76C, r.right - px(h, 20), r, colour);
                }
            }
            Slot::Sub { .. } => {}
        }
    }
    unsafe {
        Gdi::SelectObject(mem, old_font);
        let _ = Gdi::BitBlt(dc, 0, 0, cr.right, cr.bottom, Some(mem), 0, 0, Gdi::SRCCOPY);
        Gdi::SelectObject(mem, old_bmp);
        let _ = Gdi::DeleteObject(bmp.into());
        let _ = Gdi::DeleteDC(mem);
        let _ = Gdi::EndPaint(h, &ps);
    }
}

fn cell_rect(d: &Designer, slot: Slot) -> Option<RECT> {
    d.cells.iter().find(|c| c.slot == slot).map(|c| c.r)
}

/// The words of a slot, as typed (`&` and all).
fn words_of(d: &Designer, slot: Slot) -> String {
    match slot {
        Slot::Top(i) => d.menus.get(i).map(|m| m.label.clone()).unwrap_or_default(),
        Slot::Item { level, index } => list(&d.menus, &d.path, level).and_then(|l| l.get(index)).map(|it| if it.separator { "-".to_string() } else { it.label.clone().unwrap_or_default() }).unwrap_or_default(),
        Slot::Sub { .. } => String::new(),
    }
}

/// The box over a slot to type its words in; `first` replaces them (a key typed on a chosen item).
fn begin_edit(h: HWND, slot: Slot, first: Option<char>) {
    end_edit(h, true);
    let Some(d) = state(h) else { return };
    if matches!(slot, Slot::Item { level, index } if list(&d.menus, &d.path, level).and_then(|l| l.get(index)).is_some_and(|it| it.separator)) {
        return;
    }
    d.sel = Some(slot);
    let text = match first {
        Some(c) => c.to_string(),
        None => words_of(d, slot),
    };
    let e = win::edit(h, &text, EDIT_ID, d.font, false);
    d.edit = Some((e, slot));
    place_edit(h);
    unsafe {
        let _ = SetWindowSubclass(e, Some(edit_keys), 1, h.0 as usize);
        let _ = km::SetFocus(Some(e));
        let n = text.encode_utf16().count();
        if first.is_some() {
            wm::SendMessageW(e, 0x00B1, Some(WPARAM(n)), Some(LPARAM(n as isize)));
        } else {
            wm::SendMessageW(e, 0x00B1, Some(WPARAM(0)), Some(LPARAM(-1)));
        }
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

fn place_edit(h: HWND) {
    let Some(d) = state(h) else { return };
    let Some((e, slot)) = d.edit else { return };
    if let Some(r) = cell_rect(d, slot) {
        let mut r = local(d, r);
        if matches!(slot, Slot::Top(i) if i < d.menus.len()) {
            // A menu's own words may grow as they are typed: room to grow.
            r.right = r.left.max(r.right).max(r.left + px(h, 110));
        }
        win::move_to(e, r.left + 1, r.top + 1, r.right - r.left - 2, r.bottom - r.top - 2);
    }
}

/// The box closed: its words the slot's when `keep`.
fn end_edit(h: HWND, keep: bool) {
    let Some(d) = state(h) else { return };
    let Some((e, slot)) = d.edit.take() else { return };
    let text = win::text_of(e).trim().to_string();
    unsafe {
        let _ = wm::DestroyWindow(e);
        let _ = km::SetFocus(Some(h));
    }
    if keep && !text.is_empty() && text != words_of(d, slot) {
        set_words(h, slot, &text);
    }
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

/// Words typed into a slot: a menu or an item renamed, or a new one made ("-": a separator).
fn set_words(h: HWND, slot: Slot, text: &str) {
    let Some(d) = state(h) else { return };
    let taken = taken_ids(d);
    let item = |text: &str| if text == "-" { MenuItem { separator: true, ..Default::default() } } else { MenuItem { id: Some(new_id(text, &taken)), label: Some(text.to_string()), ..Default::default() } };
    match slot {
        Slot::Top(i) if i < d.menus.len() => d.menus[i].label = text.to_string(),
        Slot::Top(i) => {
            d.menus.push(Menu { label: text.to_string(), items: vec![], events: Default::default() });
            d.path = vec![i];
            // Its first item next.
            d.sel = Some(Slot::Item { level: 0, index: 0 });
        }
        Slot::Item { level, index } => {
            let path = d.path.clone();
            let Some(l) = list_mut(&mut d.menus, &path, level) else { return };
            if index < l.len() {
                if text == "-" {
                    l[index] = MenuItem { separator: true, ..Default::default() };
                } else {
                    l[index].label = Some(text.to_string());
                }
            } else {
                l.push(item(text));
                // The next "Type Here", as Visual Studio's moves on.
                d.sel = Some(Slot::Item { level, index: index + 1 });
            }
        }
        Slot::Sub { level, index } => {
            if text == "-" {
                return;
            }
            let path = d.path.clone();
            let Some(l) = list_mut(&mut d.menus, &path, level) else { return };
            let Some(parent) = l.get_mut(index) else { return };
            // A menu within it now: it opens that menu, its own id let go.
            parent.id = None;
            parent.items.push(item(text));
            d.path.truncate(level + 1);
            d.path.push(index);
            d.sel = Some(Slot::Item { level: level + 1, index: 1 });
        }
    }
    apply(h);
}

/// Every id the form has - its controls', its toolbar's, its menus' - a new one must be another.
fn taken_ids(d: &Designer) -> Vec<String> {
    let mut out: Vec<String> = d.menus.iter().flat_map(|m| m.all_items()).filter_map(|it| it.id.clone()).collect();
    if let Some(s) = studio() {
        out.extend(s.doc.form.control.iter().map(|c| c.id.clone()));
        out.extend(s.doc.form.toolbar.iter().map(|t| t.id.clone()));
    }
    out
}

/// An item's id from its words: `&Save As...` - `save_as`; a word of Rust's, or one taken, gets a
/// number; words with no Latin letters - `item`.
pub fn new_id(words: &str, taken: &[String]) -> String {
    const KEYWORDS: &[&str] = &["as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "new", "loaded"];
    let mut base = String::new();
    for c in phai_form::bars::shown(words).chars() {
        if c.is_ascii_alphanumeric() {
            base.push(c.to_ascii_lowercase());
        } else if !base.ends_with('_') && !base.is_empty() {
            base.push('_');
        }
    }
    let mut base = base.trim_matches('_').to_string();
    if base.is_empty() {
        base = "item".into();
    }
    if base.starts_with(|c: char| c.is_ascii_digit()) {
        base = format!("item_{base}");
    }
    if KEYWORDS.contains(&base.as_str()) {
        base = format!("{base}_item");
    }
    let mut id = base.clone();
    let mut n = 2;
    while taken.contains(&id) {
        id = format!("{base}{n}");
        n += 1;
    }
    id
}

/// The menus the form's now - one undo step - and drawn again.
fn apply(h: HWND) {
    let Some(d) = state(h) else { return };
    let menus = d.menus.clone();
    d.busy = true;
    if let Some(s) = studio() {
        if s.doc.form.menu != menus {
            s.edit(|f| f.menu = menus);
            s.rebuild();
            if let Some(c) = s.canvas.as_ref() {
                let (bar, _) = c.menu_bar_screen(&s.doc.form);
                d.bar = bar;
            }
        }
    }
    unsafe {
        let _ = km::SetActiveWindow(h);
        let _ = km::SetFocus(Some(d.edit.map(|(e, _)| e).unwrap_or(h)));
    }
    d.busy = false;
    relayout(h);
}

fn close(h: HWND) {
    end_edit(h, true);
    if let Some(d) = state(h) {
        if d.done {
            return;
        }
        d.done = true;
    }
    OPEN.store(0, std::sync::atomic::Ordering::Relaxed);
    crate::canvas::refresh_tray();
    unsafe {
        let _ = wm::DestroyWindow(h);
    }
}

/// A click on a slot: chosen (its menu opened); on the one already chosen, or on a "Type Here",
/// its words edited.
fn click(h: HWND, slot: Slot) {
    end_edit(h, true);
    let Some(d) = state(h) else { return };
    let again = d.sel == Some(slot);
    d.sel = Some(slot);
    let mut type_here = false;
    match slot {
        Slot::Top(i) => {
            if i < d.menus.len() {
                d.path = vec![i];
            } else {
                type_here = true;
            }
        }
        Slot::Item { level, index } => {
            d.path.truncate(level + 1);
            match list(&d.menus, &d.path, level).map(|l| (l.len(), l.get(index).is_some_and(|it| !it.items.is_empty()))) {
                Some((len, _)) if index >= len => type_here = true,
                Some((_, true)) => d.path.push(index),
                _ => {}
            }
        }
        Slot::Sub { .. } => type_here = true,
    }
    relayout(h);
    if type_here {
        begin_edit(h, slot, None);
    } else if again {
        // Edited only if this was not the first half of a double-click.
        if let Some(d) = state(h) {
            d.pending = Some(slot);
        }
        unsafe {
            wm::SetTimer(Some(h), 1, windows::Win32::UI::Input::KeyboardAndMouse::GetDoubleClickTime(), None);
        }
    }
}

/// The chosen item moved one up or down among its own.
fn move_item(h: HWND, down: bool) {
    let Some(d) = state(h) else { return };
    match d.sel {
        Some(Slot::Top(i)) if i < d.menus.len() => {
            let j = if down { i + 1 } else { i.wrapping_sub(1) };
            if j < d.menus.len() {
                d.menus.swap(i, j);
                d.path = vec![j];
                d.sel = Some(Slot::Top(j));
                apply(h);
            }
        }
        Some(Slot::Item { level, index }) => {
            let path = d.path.clone();
            let Some(l) = list_mut(&mut d.menus, &path, level) else { return };
            let j = if down { index + 1 } else { index.wrapping_sub(1) };
            if index < l.len() && j < l.len() {
                l.swap(index, j);
                d.path.truncate(level + 1);
                d.sel = Some(Slot::Item { level, index: j });
                apply(h);
            }
        }
        _ => {}
    }
}

/// The chosen menu or item taken out, with what is in it.
fn delete(h: HWND) {
    let Some(d) = state(h) else { return };
    match d.sel {
        Some(Slot::Top(i)) if i < d.menus.len() => {
            d.menus.remove(i);
            d.path.clear();
            d.sel = Some(Slot::Top(i.min(d.menus.len())));
            apply(h);
        }
        Some(Slot::Item { level, index }) => {
            let path = d.path.clone();
            let Some(l) = list_mut(&mut d.menus, &path, level) else { return };
            if index < l.len() {
                l.remove(index);
                let len = l.len();
                d.path.truncate(level + 1);
                d.sel = Some(Slot::Item { level, index: index.min(len) });
                apply(h);
            }
        }
        _ => {}
    }
}

/// A separator put before the chosen item.
fn insert(h: HWND, separator: bool) {
    let Some(d) = state(h) else { return };
    let Some(Slot::Item { level, index }) = d.sel else { return };
    let taken = taken_ids(d);
    let path = d.path.clone();
    let Some(l) = list_mut(&mut d.menus, &path, level) else { return };
    let at = index.min(l.len());
    if separator {
        l.insert(at, MenuItem { separator: true, ..Default::default() });
    } else {
        // The English word, as every text of a form: the Thai table translates it.
        let label = "New item".to_string();
        l.insert(at, MenuItem { id: Some(new_id(&label, &taken)), label: Some(label), ..Default::default() });
    }
    d.path.truncate(level + 1);
    d.sel = Some(Slot::Item { level, index: at });
    apply(h);
    if !separator {
        begin_edit(h, Slot::Item { level, index: at }, None);
    }
}

/// The chosen item's properties: its id (the method a click runs), words, shortcut, picture,
/// ticks and whether it may be chosen.
fn properties(h: HWND) {
    let Some(d) = state(h) else { return };
    let Some(Slot::Item { level, index }) = d.sel else {
        if let Some(Slot::Top(i)) = d.sel.filter(|s| matches!(s, Slot::Top(i) if *i < d.menus.len())) {
            begin_edit(h, Slot::Top(i), None);
        }
        return;
    };
    let Some(it) = list(&d.menus, &d.path, level).and_then(|l| l.get(index)).cloned() else { return };
    if it.separator {
        return;
    }
    d.busy = true;
    use phai_ui::dialogs::Field;
    let id = it.id.clone().unwrap_or_default();
    let label = it.label.clone().unwrap_or_default();
    let key = it.key.clone().unwrap_or_default();
    let glyph = it.glyph.clone().unwrap_or_default();
    let answer = phai_ui::dialogs::ask_fields(
        Some(h),
        &tf("Menu item {0}", &[&phai_form::bars::shown(&label)]),
        &[
            Field::Text(t("Id (its Click method)"), &id),
            Field::Text(t("Text (& before the underlined letter)"), &label),
            Field::Text(t("Shortcut, e.g. Ctrl+S"), &key),
            Field::Text(t("Picture (a glyph's name, e.g. SAVE)"), &glyph),
            Field::Check(t("Checked"), it.checked),
            Field::Check(t("Check on click"), it.check_on_click),
            Field::Check(t("Disabled"), it.disabled),
        ],
    );
    let Some(d) = state(h) else { return };
    d.busy = false;
    let Some(a) = answer else { return };
    let new_glyph = a[3].trim().to_uppercase();
    if !new_glyph.is_empty() && phai_ui::icon::glyph::by_name(&new_glyph).is_none() {
        let names: Vec<&str> = phai_ui::icon::glyph::NAMES.iter().map(|(n, _)| *n).collect();
        win::message(Some(h), &tf("{0} is not a glyph - one of {1}", &[&new_glyph, &names.join(", ")]), t("Menus"), wm::MB_ICONWARNING);
        return;
    }
    let new_id = a[0].trim().to_string();
    if !it.items.is_empty() || new_id.is_empty() {
        // A menu within it has no id of its own.
    } else if !new_id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') || new_id.starts_with(|c: char| c.is_ascii_digit()) {
        win::message(Some(h), &tf("{0}: an id is snake_case, like save_as", &[&new_id]), t("Menus"), wm::MB_ICONWARNING);
        return;
    }
    let path = d.path.clone();
    let Some(l) = list_mut(&mut d.menus, &path, level) else { return };
    let x = &mut l[index];
    if x.items.is_empty() && !new_id.is_empty() {
        x.id = Some(new_id);
    }
    x.label = Some(a[1].trim().to_string()).filter(|s| !s.is_empty()).or(x.label.take());
    x.key = Some(a[2].trim().to_string()).filter(|s| !s.is_empty());
    x.glyph = Some(new_glyph).filter(|s| !s.is_empty());
    x.checked = a[4] == "true";
    x.check_on_click = a[5] == "true";
    x.disabled = a[6] == "true";
    apply(h);
}

fn right_click(h: HWND, slot: Slot) {
    click_choose(h, slot);
    const EDIT: u32 = 1;
    const PROPS: u32 = 2;
    const SEP: u32 = 3;
    const INSERT: u32 = 4;
    const UP: u32 = 5;
    const DOWN: u32 = 6;
    const DELETE: u32 = 7;
    let Some(d) = state(h) else { return };
    let item = matches!(slot, Slot::Item { level, index } if list(&d.menus, &d.path, level).is_some_and(|l| index < l.len()));
    let top = matches!(slot, Slot::Top(i) if i < d.menus.len());
    if !item && !top {
        return;
    }
    let mut items: Vec<(u32, &str)> = vec![(EDIT, t("Edit Text")), (PROPS, t("Properties..."))];
    if item {
        items.extend([(0, ""), (INSERT, t("Insert Item")), (SEP, t("Insert Separator"))]);
    }
    items.extend([(0, ""), (UP, t("Move Up")), (DOWN, t("Move Down")), (0, ""), (DELETE, t("Delete"))]);
    d.busy = true;
    let chosen = win::popup(h, &items);
    if let Some(d) = state(h) {
        d.busy = false;
    }
    match chosen {
        EDIT => begin_edit(h, slot, None),
        PROPS => properties(h),
        SEP => insert(h, true),
        INSERT => insert(h, false),
        UP => move_item(h, false),
        DOWN => move_item(h, true),
        DELETE => delete(h),
        _ => {}
    }
}

/// Chosen without editing (a right-click).
fn click_choose(h: HWND, slot: Slot) {
    end_edit(h, true);
    let Some(d) = state(h) else { return };
    d.sel = Some(slot);
    if let Slot::Top(i) = slot {
        if i < d.menus.len() {
            d.path = vec![i];
        }
    }
    if let Slot::Item { level, .. } = slot {
        d.path.truncate(level + 1);
    }
    relayout(h);
}

/// The arrows: up and down a menu, right into a menu within, left back out; along the bar.
fn arrow(h: HWND, key: usize) {
    let Some(d) = state(h) else { return };
    let Some(sel) = d.sel else { return };
    match (sel, key) {
        (Slot::Top(i), 0x25) => click_choose(h, Slot::Top(i.saturating_sub(1))),
        (Slot::Top(i), 0x27) => click_choose(h, Slot::Top((i + 1).min(d.menus.len()))),
        (Slot::Top(i), 0x28) if i < d.menus.len() => {
            d.path = vec![i];
            click_choose(h, Slot::Item { level: 0, index: 0 });
        }
        (Slot::Item { level, index }, 0x26) => {
            if index == 0 && level == 0 {
                let top = d.path.first().copied().unwrap_or(0);
                click_choose(h, Slot::Top(top));
            } else {
                click_choose(h, Slot::Item { level, index: index.saturating_sub(1) });
            }
        }
        (Slot::Item { level, index }, 0x28) => {
            let len = list(&d.menus, &d.path, level).map(|l| l.len()).unwrap_or(0);
            click_choose(h, Slot::Item { level, index: (index + 1).min(len) });
        }
        (Slot::Item { level, index }, 0x27) => {
            let has = list(&d.menus, &d.path, level).and_then(|l| l.get(index)).is_some_and(|it| !it.items.is_empty());
            if has {
                d.path.truncate(level + 1);
                d.path.push(index);
                click_choose(h, Slot::Item { level: level + 1, index: 0 });
            }
        }
        (Slot::Item { level, .. }, 0x25) if level > 0 => {
            let parent = d.path.get(level).copied().unwrap_or(0);
            d.path.truncate(level);
            click_choose(h, Slot::Item { level: level - 1, index: parent });
        }
        _ => {}
    }
}

unsafe extern "system" fn edit_keys(e: HWND, msg: u32, w: WPARAM, l: LPARAM, _: usize, owner: usize) -> LRESULT {
    let h = HWND(owner as *mut _);
    match msg {
        wm::WM_GETDLGCODE => LRESULT(4),
        wm::WM_KEYDOWN if w.0 == 0x0D => {
            end_edit(h, true);
            // A new item made: the next "Type Here" takes the typing on.
            LRESULT(0)
        }
        wm::WM_KEYDOWN if w.0 == 0x1B => {
            end_edit(h, false);
            LRESULT(0)
        }
        wm::WM_CHAR if w.0 == 0x0D || w.0 == 0x1B => LRESULT(0),
        wm::WM_KILLFOCUS => {
            let r = unsafe { DefSubclassProc(e, msg, w, l) };
            unsafe {
                let _ = wm::PostMessageW(Some(h), wm::WM_APP + 1, WPARAM(0), LPARAM(0));
            }
            r
        }
        _ => unsafe { DefSubclassProc(e, msg, w, l) },
    }
}

fn hit(h: HWND, l: LPARAM) -> Option<Slot> {
    let d = state(h)?;
    let (x, y) = win::point_of(l);
    let p = POINT { x: x + d.at.x, y: y + d.at.y };
    d.cells.iter().find(|c| p.x >= c.r.left && p.x < c.r.right && p.y >= c.r.top && p.y < c.r.bottom).map(|c| c.slot)
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(d) = state(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_PAINT => {
            paint(h);
            LRESULT(0)
        }
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_MOUSEACTIVATE => LRESULT(1 /* MA_ACTIVATE */),
        wm::WM_LBUTTONDOWN => {
            match hit(h, l) {
                Some(slot) => click(h, slot),
                None => close(h),
            }
            LRESULT(0)
        }
        wm::WM_TIMER => {
            unsafe {
                let _ = wm::KillTimer(Some(h), 1);
            }
            if let Some(slot) = d.pending.take() {
                begin_edit(h, slot, None);
            }
            LRESULT(0)
        }
        // A double-click on an item: its Click method, as Visual Studio's; on a menu, its words.
        wm::WM_LBUTTONDBLCLK => {
            d.pending = None;
            unsafe {
                let _ = wm::KillTimer(Some(h), 1);
            }
            if let Some(slot) = hit(h, l) {
                let id = match slot {
                    Slot::Item { level, index } => list(&d.menus, &d.path, level).and_then(|l| l.get(index)).filter(|it| it.items.is_empty()).and_then(|it| it.id.clone()),
                    _ => None,
                };
                match id {
                    Some(id) => {
                        close(h);
                        if let Some(s) = studio() {
                            s.open_item_handler(&id);
                        }
                    }
                    None => begin_edit(h, slot, None),
                }
            }
            LRESULT(0)
        }
        wm::WM_RBUTTONUP => {
            if let Some(slot) = hit(h, l) {
                right_click(h, slot);
            }
            LRESULT(0)
        }
        wm::WM_KEYDOWN => {
            let ctrl = unsafe { km::GetKeyState(0x11) } < 0;
            match w.0 {
                0x1B => close(h),
                0x0D | 0x71 => {
                    if let Some(s) = d.sel {
                        begin_edit(h, s, None);
                    }
                }
                0x2E => delete(h),
                0x73 => properties(h),
                0x26 | 0x28 if ctrl => move_item(h, w.0 == 0x28),
                0x25..=0x28 => arrow(h, w.0),
                _ => {}
            }
            LRESULT(0)
        }
        // A letter typed on a chosen slot: its words typed anew, as Visual Studio's.
        wm::WM_CHAR if w.0 >= 0x20 && w.0 != 0x7F => {
            if let (Some(s), Some(c)) = (d.sel, char::from_u32(w.0 as u32)) {
                begin_edit(h, s, Some(c));
            }
            LRESULT(0)
        }
        m if m == wm::WM_APP + 1 => {
            // The box lost the focus: kept, unless the focus is still inside.
            if let Some((e, _)) = d.edit {
                if unsafe { km::GetFocus() } != e {
                    end_edit(h, true);
                }
            }
            LRESULT(0)
        }
        // Another window chosen - a click on the form, another pane: done.
        wm::WM_ACTIVATE if win::loword(w.0) == 0 => {
            if !d.busy {
                unsafe {
                    let _ = wm::PostMessageW(Some(h), wm::WM_APP + 2, WPARAM(0), LPARAM(0));
                }
            }
            LRESULT(0)
        }
        m if m == wm::WM_APP + 2 => {
            let active = unsafe { km::GetActiveWindow() };
            if active != h && !d.busy {
                close(h);
            }
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            if let Some(d) = unsafe { win::take_state::<Designer>(h) } {
                unsafe {
                    let _ = Gdi::DeleteObject(d.font.into());
                }
            }
            LRESULT(0)
        }
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An item's id comes from its words, a Rust name that is not taken.
    #[test]
    fn ids_from_words() {
        let taken = vec!["save".to_string()];
        assert_eq!(new_id("&Save As...", &taken), "save_as");
        assert_eq!(new_id("&Save", &taken), "save2");
        assert_eq!(new_id("New", &[]), "new_item");
        assert_eq!(new_id("เปิดไฟล์", &[]), "item");
        assert_eq!(new_id("3D View", &[]), "item_3d_view");
    }
}
