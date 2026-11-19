//! A form built now, in a running program: the designer's canvas and its preview. The same
//! phai-ui calls the generated code makes (`codegen.rs`), so what is designed is what runs.

use phai_ui::controls;
use phai_ui::layout::{unscale, Anchor, Dock, Item, Layout, Rect};
use phai_ui::text::t;
use phai_ui::win;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::HFONT;

use crate::model::{parse_anchor, Control, FormFile, HasProps, Kind};
use crate::props;

/// The windows of a built form: one per control, in form order, and the toolbar if it has one.
pub struct Live {
    pub controls: Vec<HWND>,
    pub toolbar: Option<HWND>,
    pub layout: Layout,
}

/// The id a control's window gets, as in the generated code.
pub fn control_id(i: usize) -> i32 {
    2000 + i as i32
}

thread_local! {
    /// Whether the form's words go through `t()`. The designer shows them as written: the
    /// program's Thai table is the program's, and the designer's own would mistranslate them.
    static TRANSLATE: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// Shows the form's words as written (`false`) or in the chosen language (`true`, the default).
pub fn set_translate(on: bool) {
    TRANSLATE.with(|t| t.set(on));
}

/// Text that is a sentence goes through `t()`; it must live as long as the program, as the
/// form's words do once compiled, so it is leaked - once per distinct sentence.
fn tr(s: &str) -> &'static str {
    use std::collections::HashSet;
    use std::sync::Mutex;
    static KEPT: Mutex<Option<HashSet<&'static str>>> = Mutex::new(None);
    let mut kept = KEPT.lock().unwrap_or_else(|e| e.into_inner());
    let set = kept.get_or_insert_with(HashSet::new);
    let key: &'static str = match set.get(s) {
        Some(k) => k,
        None => {
            let k: &'static str = Box::leak(s.to_string().into_boxed_str());
            set.insert(k);
            k
        }
    };
    if TRANSLATE.with(|t| t.get()) {
        t(key)
    } else {
        key
    }
}

fn dock_of(c: &Control) -> Dock {
    match c.dock.as_deref() {
        Some("top") => Dock::Top,
        Some("bottom") => Dock::Bottom,
        Some("left") => Dock::Left,
        Some("right") => Dock::Right,
        Some("fill") => Dock::Fill,
        _ => Dock::None,
    }
}

fn anchor_of(c: &Control) -> Anchor {
    let a = parse_anchor(c.anchor.as_deref()).unwrap_or_default();
    Anchor { left: a.left, top: a.top, right: a.right, bottom: a.bottom }
}

/// One control's window on `parent`.
pub fn make(parent: HWND, c: &Control, id: i32, font: HFONT) -> HWND {
    let text = c.text.as_deref().unwrap_or("");
    let h = match c.kind {
        Kind::Label => win::label(parent, tr(text), id, font),
        Kind::Edit => win::edit(parent, text, id, font, c.password.unwrap_or(false)),
        Kind::Button => win::button(parent, tr(text), id, font, false),
        Kind::Checkbox => win::checkbox(parent, tr(text), id, font, c.checked.unwrap_or(false)),
        Kind::Radio => win::radio(parent, tr(text), id, font, c.group.unwrap_or(0), c.checked.unwrap_or(false)),
        Kind::Combo => {
            let sorted = crate::codegen::combo_items(c);
            let items: Vec<&str> = sorted.iter().map(|s| tr(s)).collect();
            win::combo_with(parent, id, font, &items, c.flag("editable", false))
        }
        Kind::List => {
            let cols: Vec<(&str, i32)> = c.columns.iter().map(|(s, n)| (tr(s), *n)).collect();
            win::list(parent, id, font, &cols)
        }
        Kind::Group => controls::group(parent, tr(text), id, font),
        Kind::Tabs => {
            let tabs: Vec<&str> = c.tabs.iter().map(|s| tr(s)).collect();
            controls::tabs(parent, id, font, &tabs)
        }
        Kind::Statusbar => {
            let h = controls::status_bar(parent, id, font);
            if !c.parts.is_empty() {
                controls::status_parts(h, &c.parts);
            }
            if !text.is_empty() {
                controls::status_text(h, 0, tr(text));
            }
            h
        }
        Kind::Textarea => win::textarea(parent, text, id, font),
        Kind::Date => win::date(parent, id, font),
        Kind::Number => win::number(parent, id, font, c.min.unwrap_or(0), c.max.unwrap_or(100), c.value.unwrap_or(0)),
        Kind::Progress => win::progress(parent, id, c.min.unwrap_or(0), c.max.unwrap_or(100), c.value.unwrap_or(0)),
        Kind::Slider => win::slider(parent, id, c.min.unwrap_or(0), c.max.unwrap_or(100), c.value.unwrap_or(0)),
        Kind::Tree => {
            let h = win::tree(parent, id, font);
            let items: Vec<&str> = c.items.iter().map(String::as_str).collect();
            win::tree_items(h, &items);
            h
        }
        Kind::Picture => phai_ui::picture::picture(parent, id, c.image.as_deref().unwrap_or("")),
        Kind::Flow | Kind::Table => win::panel(parent, id, true),
        Kind::Rich => phai_ui::grid::richtext(parent, id, font, text),
        Kind::Grid => {
            let cols: Vec<(&str, i32)> = c.columns.iter().map(|(s, n)| (tr(s), *n)).collect();
            phai_ui::grid::grid(parent, id, font, &cols)
        }
        // A component has no window on the form: a hidden stand-in keeps the controls' order.
        Kind::Timer | Kind::ContextMenu => {
            let h = win::label(parent, "", id, font);
            win::show(h, false);
            h
        }
    };
    win::set_css(h, c.class.as_deref().unwrap_or(""), &c.id);
    if let Some(own) = c.own_css() {
        let _ = win::set_own_style(h, &own);
    }
    // The table's properties, as the generated code gives them - those the canvas shows.
    for s in props::of_kind(c.kind).filter(|s| s.how == props::How::Call && s.canvas) {
        if let Some(v) = c.prop(s.name) {
            let text = props::text_of(v);
            let text = if s.ty == props::Ty::Words { tr(&text).to_string() } else { text };
            phai_ui::props::apply(h, s.name, &text);
        }
    }
    if c.enabled == Some(false) {
        win::enable(h, false);
    }
    h
}

/// Flow and table panels place what they hold, as the generated program's `arrange` does; after
/// the layout has put the panels themselves where they go.
pub fn arrange(f: &FormFile, controls: &[HWND]) {
    for (p, c) in f.control.iter().enumerate().filter(|(_, c)| c.kind.arranges()) {
        let Some(&ph) = controls.get(p) else { continue };
        let dpi = win::dpi(ph);
        let r = win::rect_in_parent(ph);
        let area = Rect::new(r.left, r.top, r.right - r.left, r.bottom - r.top);
        let kids = f.arranged_in(p);
        let sizes: Vec<(i32, i32)> = kids.iter().map(|k| {
            let [_, _, w, h] = f.control[*k].rect();
            (phai_ui::layout::scale(w, dpi), phai_ui::layout::scale(h, dpi))
        }).collect();
        let gap = phai_ui::layout::scale(c.int("gap").unwrap_or(6), dpi);
        if c.kind == Kind::Flow {
            for (k, (x, y)) in kids.iter().zip(phai_ui::layout::flow_with(area, &sizes, gap, c.word("direction") == Some("top_down"), c.flag("wrap", true))) {
                let (w, h) = sizes[kids.iter().position(|q| q == k).unwrap_or(0)];
                if let Some(&kh) = controls.get(*k) {
                    win::move_to(kh, x, y, w, h);
                }
            }
        } else {
            for (k, cell) in kids.iter().zip(phai_ui::layout::table(area, &sizes, c.cols.unwrap_or(2).max(1) as usize, gap)) {
                if let Some(&kh) = controls.get(*k) {
                    win::move_to(kh, cell.x, cell.y, cell.w, cell.h);
                }
            }
        }
    }
}

/// The layout item for control `c`, as the generated code places it.
pub fn item(parent: HWND, c: &Control, h: HWND) -> Item {
    let [x, y, w, hgt] = c.rect();
    let hgt = if c.kind == Kind::Statusbar { unscale(controls::status_height(h), win::dpi(parent)) } else { hgt };
    Item { bounds: Rect::new(x, y, w, hgt), dock: dock_of(c), anchor: anchor_of(c) }
}

/// Builds every control of `f` on `parent` - the toolbar first, as the generated code does.
/// `page` is the tab page to show; controls on others are hidden.
pub fn build(parent: HWND, f: &FormFile, font: HFONT, page: usize) -> Live {
    let mut layout = Layout::new((f.form.size[0], f.form.size[1]));
    let toolbar = (!f.toolbar.is_empty()).then(|| {
        let items: Vec<(i32, &str, u16)> = f.toolbar.iter().enumerate().map(|(i, tool)| (1000 + i as i32, tr(&tool.label), phai_ui::icon::glyph::by_name(&tool.glyph).unwrap_or(0))).collect();
        let h = win::toolbar(parent, 1999, &items);
        layout.add(h, Item { bounds: Rect::new(0, 0, 0, win::TOOLBAR_HEIGHT), dock: Dock::Top, anchor: Anchor::default() });
        h
    });
    let mut out = Vec::new();
    let form_font = match f.form.font_size {
        Some(pt) => win::font(parent, pt, false),
        None => font,
    };
    for (i, c) in f.control.iter().enumerate() {
        let own = c.font_size.map(|pt| win::font(parent, pt, false)).unwrap_or(form_font);
        let h = make(parent, c, control_id(i), own);
        if c.kind == Kind::Tabs && page > 0 {
            controls::tabs_select(h, page);
        }
        if c.visible == Some(false) || c.page.is_some_and(|p| p != page) || c.kind.is_component() {
            win::show(h, false);
        }
        layout.add(h, item(parent, c, h));
        out.push(h);
    }
    Live { controls: out, toolbar, layout }
}
