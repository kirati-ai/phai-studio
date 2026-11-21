//! The Win32 every window needs: wide strings, a class, a font that follows the monitor's scale,
//! child controls, a pointer from a window to its state - from PhaiDesk's console
//! (`console/win.rs`) - and the drawing that makes them follow the theme.
//!
//! Windows draws the text, so Thai shapes the way it does everywhere else on the machine - the
//! reason this is Win32 rather than a toolkit that renders its own glyphs.
//!
//! **[`def`] does the theming.** A window procedure passes what it does not handle to [`def`],
//! and that paints the background, colours labels and edits, draws the library's buttons,
//! checkboxes and radios, and the selected row of a list - all from [`crate::theme::current`].

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{self, HDC, HFONT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi;
use windows::Win32::UI::WindowsAndMessaging::{self as wm, HMENU, WINDOW_EX_STYLE, WINDOW_STYLE};

use crate::theme::{self, Color, Query, State, Style};

pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

pub fn instance() -> HINSTANCE {
    unsafe { GetModuleHandleW(None).map(|m| HINSTANCE(m.0)).unwrap_or_default() }
}

/// Per-monitor DPI awareness and the common controls, once, before any window. Without it a
/// scaled monitor stretches every window as a bitmap and the text goes soft.
pub fn init() {
    static DONE: std::sync::Once = std::sync::Once::new();
    DONE.call_once(|| unsafe {
        let _ = HiDpi::SetProcessDpiAwarenessContext(HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let icc = windows::Win32::UI::Controls::INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<windows::Win32::UI::Controls::INITCOMMONCONTROLSEX>() as u32,
            dwICC: windows::Win32::UI::Controls::ICC_LISTVIEW_CLASSES | windows::Win32::UI::Controls::ICC_BAR_CLASSES | windows::Win32::UI::Controls::ICC_STANDARD_CLASSES | windows::Win32::UI::Controls::ICC_DATE_CLASSES | windows::Win32::UI::Controls::ICC_UPDOWN_CLASS | windows::Win32::UI::Controls::ICC_COOL_CLASSES | windows::Win32::UI::Controls::ICC_LINK_CLASS,
        };
        let _ = windows::Win32::UI::Controls::InitCommonControlsEx(&icc);
    });
}

pub fn dpi(hwnd: HWND) -> i32 {
    let d = unsafe { HiDpi::GetDpiForWindow(hwnd) } as i32;
    if d <= 0 {
        96
    } else {
        d
    }
}

/// `v` design pixels at 96 DPI, at this window's scale.
pub fn px(hwnd: HWND, v: i32) -> i32 {
    crate::layout::scale(v, dpi(hwnd))
}

/// Registers a window class once per process. Its background is painted by [`def`] in the
/// theme's window colour, so the class has none of its own.
pub fn register(class: &str, proc_: wm::WNDPROC, icon: Option<wm::HICON>) -> Vec<u16> {
    register_with(ClassStyle::Plain, class, proc_, icon)
}

/// What a class asks of Windows beyond the usual.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ClassStyle {
    Plain,
    /// The window hears `WM_LBUTTONDBLCLK`.
    DoubleClicks,
}

pub fn register_with(extra: ClassStyle, class: &str, proc_: wm::WNDPROC, icon: Option<wm::HICON>) -> Vec<u16> {
    static DONE: Mutex<Option<HashSet<String>>> = Mutex::new(None);
    let name = wide(class);
    let mut done = DONE.lock().unwrap_or_else(|e| e.into_inner());
    let set = done.get_or_insert_with(HashSet::new);
    if set.insert(class.to_string()) {
        unsafe {
            let wc = wm::WNDCLASSEXW {
                cbSize: std::mem::size_of::<wm::WNDCLASSEXW>() as u32,
                style: wm::CS_HREDRAW | wm::CS_VREDRAW | if extra == ClassStyle::DoubleClicks { wm::CS_DBLCLKS } else { Default::default() },
                lpfnWndProc: proc_,
                hInstance: instance(),
                hCursor: wm::LoadCursorW(None, wm::IDC_ARROW).unwrap_or_default(),
                hbrBackground: Gdi::HBRUSH::default(),
                lpszClassName: PCWSTR(name.as_ptr()),
                hIcon: icon.unwrap_or_default(),
                hIconSm: icon.unwrap_or_default(),
                ..Default::default()
            };
            wm::RegisterClassExW(&wc);
        }
    }
    name
}

/// The message font (Segoe UI on every current Windows) at this window's scale. `points` 0 is
/// the system's own size (9 pt).
pub fn font(hwnd: HWND, points: i32, bold: bool) -> HFONT {
    unsafe {
        let mut metrics = wm::NONCLIENTMETRICSW { cbSize: std::mem::size_of::<wm::NONCLIENTMETRICSW>() as u32, ..Default::default() };
        let _ = HiDpi::SystemParametersInfoForDpi(wm::SPI_GETNONCLIENTMETRICS.0, metrics.cbSize, Some(&mut metrics as *mut _ as *mut _), 0, dpi(hwnd) as u32);
        let mut lf = metrics.lfMessageFont;
        if points > 0 {
            lf.lfHeight = -(points * dpi(hwnd) / 72);
        }
        if bold {
            lf.lfWeight = 600;
        }
        Gdi::CreateFontIndirectW(&lf)
    }
}

pub fn set_font(hwnd: HWND, f: HFONT) {
    unsafe {
        wm::SendMessageW(hwnd, wm::WM_SETFONT, Some(WPARAM(f.0 as usize)), Some(LPARAM(1)));
    }
}

/// Sets `f` on every child of `parent` - after a DPI change, with a font made for the new scale.
pub fn set_font_all(parent: HWND, f: HFONT) {
    unsafe extern "system" fn each(h: HWND, l: LPARAM) -> windows::core::BOOL {
        set_font(h, HFONT(l.0 as *mut _));
        true.into()
    }
    unsafe {
        let _ = wm::EnumChildWindows(Some(parent), Some(each), LPARAM(f.0 as isize));
    }
}

/// The width and height of `text` in `font`, measured - never guessed, because Thai is taller
/// and often longer than the English it replaces.
pub fn measure(hwnd: HWND, font: HFONT, text: &str) -> (i32, i32) {
    if text.is_empty() {
        return (0, measure(hwnd, font, "Ag").1);
    }
    unsafe {
        let dc = Gdi::GetDC(Some(hwnd));
        let old = Gdi::SelectObject(dc, font.into());
        let w: Vec<u16> = text.encode_utf16().collect();
        let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        let mut buf = w.clone();
        Gdi::DrawTextW(dc, &mut buf, &mut r, Gdi::DT_CALCRECT | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX);
        Gdi::SelectObject(dc, old);
        Gdi::ReleaseDC(Some(hwnd), dc);
        (r.right - r.left, r.bottom - r.top)
    }
}

/// A child control.
#[allow(clippy::too_many_arguments)]
pub fn child(parent: HWND, class: &str, text: &str, style: WINDOW_STYLE, ex: WINDOW_EX_STYLE, id: i32, f: HFONT) -> HWND {
    let class = wide(class);
    let text = wide(text);
    unsafe {
        let h = wm::CreateWindowExW(
            ex,
            PCWSTR(class.as_ptr()),
            PCWSTR(text.as_ptr()),
            wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPSIBLINGS | style,
            0,
            0,
            10,
            10,
            Some(parent),
            Some(HMENU(id as isize as *mut _)),
            Some(instance()),
            None,
        )
        .unwrap_or_default();
        if !f.is_invalid() {
            set_font(h, f);
        }
        h
    }
}

// --- what CSS knows about a control: its kind, classes and id ----------------------------------

/// The kinds the library draws; the name is the CSS selector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Button,
    Checkbox,
    Radio,
    Edit,
    Label,
    List,
    Group,
    Tabs,
    Combo,
    StatusBar,
    Toolbar,
    Tree,
    Date,
    Progress,
    Slider,
    /// A month to choose a day in (MonthCalendar).
    Calendar,
}

impl Kind {
    pub fn css(self) -> &'static str {
        match self {
            Kind::Button => "button",
            Kind::Checkbox => "checkbox",
            Kind::Radio => "radio",
            Kind::Edit => "edit",
            Kind::Label => "label",
            Kind::List => "list",
            Kind::Group => "group",
            Kind::Tabs => "tabs",
            Kind::Combo => "combo",
            Kind::StatusBar => "statusbar",
            Kind::Toolbar => "toolbar",
            Kind::Tree => "tree",
            Kind::Date => "date",
            Kind::Progress => "progress",
            Kind::Slider => "slider",
            Kind::Calendar => "calendar",
        }
    }
}

#[derive(Clone, Debug)]
struct Css {
    kind: Kind,
    classes: Vec<String>,
    id: Option<String>,
    /// Radios with the same parent and group number exclude each other.
    group: i32,
    checked: bool,
    hot: bool,
    /// The control's own look, over the theme's: [`set_own_style`].
    own: Style,
    /// The same as it was written, for the Live Inspector.
    own_text: String,
    /// Where an owner-drawn button's words sit: 0 its kind's own, 1 left, 2 centre, 3 right.
    align: u8,
    /// A check box's third state (WinForms' Indeterminate), and whether a click reaches it.
    mixed: bool,
    three: bool,
    /// A check box or radio drawn as a button that stays pressed - Appearance = Button.
    push: bool,
    /// A label as wide as its words - AutoSize.
    auto: bool,
    /// The designer showing it in a state it is not in - its Style tab's preview.
    preview: Option<State>,
    /// A button's icon: a glyph of Windows' icon font before its words (0: none).
    glyph: u16,
}

/// An icon on a button, before its words - a glyph of Windows' icon font (Segoe Fluent Icons),
/// drawn in the button's text colour.
pub fn set_button_glyph(h: HWND, glyph: u16) {
    with_css(h, |c| c.glyph = glyph);
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, true);
    }
}

/// Shows control `h` as if it were in `state` (hovered, pressed...) - or as it is, for `None`.
/// The designer's Style tab uses it to preview a state's look.
pub fn set_preview_state(h: HWND, state: Option<State>) {
    with_css(h, |c| c.preview = state);
    unsafe {
        let _ = Gdi::RedrawWindow(Some(h), None, None, Gdi::RDW_INVALIDATE | Gdi::RDW_FRAME | Gdi::RDW_ERASE);
    }
}

/// Where each of control `h`'s values comes from in `state`: the theme's rules, then its own
/// (`#id`, its Properties), then its state rules - for the Style tab.
pub fn explain_of(h: HWND, kind: Kind, state: State) -> Vec<theme::Origin> {
    let (classes, id, own) = with_css(h, |c| (c.classes.clone(), c.id.clone(), c.own.clone())).unwrap_or_default();
    let refs: Vec<&str> = classes.iter().map(String::as_str).collect();
    let q = Query { kind: kind.css(), classes: &refs, id: id.as_deref(), state };
    let mut out = theme::current().explain(&q);
    let mut put = |o: theme::Origin| match out.iter_mut().find(|x| x.property == o.property) {
        Some(x) => *x = o,
        None => out.push(o),
    };
    let me = format!("#{}", id.clone().unwrap_or_default());
    for (p, set) in [("background", own.background.is_some()), ("color", own.color.is_some()), ("border-color", own.border_color.is_some()), ("border-width", own.border_width.is_some()), ("border-radius", own.border_radius.is_some()), ("font-weight", own.font_weight.is_some()), ("accent-color", own.accent_color.is_some()), ("icon-color", own.icon_color.is_some()), ("transition", own.transition.is_some()), ("background-image", own.gradient.is_some()), ("box-shadow", own.shadow.is_some())] {
        if set {
            put(theme::Origin { property: p, selector: me.clone(), source: "own".into(), line: 0 });
        }
    }
    for o in theme::state_rules_explain(&q) {
        put(theme::Origin { source: "own".into(), ..o });
    }
    out
}

/// A check box's state: 0 unticked, 1 ticked, 2 neither (WinForms' Indeterminate).
pub fn check_state(h: impl Into<HWND>) -> u8 {
    let h: HWND = h.into();
    with_css(h, |c| if c.mixed { 2 } else { c.checked as u8 }).unwrap_or(0)
}

pub fn set_check_state(h: impl Into<HWND>, state: u8) {
    let h: HWND = h.into();
    with_css(h, |c| {
        c.mixed = state == 2;
        c.checked = state != 0;
    });
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

/// ThreeState: a click goes unticked, ticked, neither.
pub fn set_three_state(h: HWND, on: bool) {
    with_css(h, |c| c.three = on);
}

/// Appearance = Button: a check box or radio drawn as a button, pressed while ticked.
pub fn set_push_like(h: HWND, on: bool) {
    with_css(h, |c| c.push = on);
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

/// AutoSize: a label as wide as its words, now and whenever they change.
pub fn set_auto_size(h: HWND, on: bool) {
    with_css(h, |c| c.auto = on);
    if on {
        fit_label(h);
    }
}

/// The width a label that sizes itself wants, in pixels; `None` for any other control.
pub fn auto_width(h: HWND) -> Option<i32> {
    if !with_css(h, |c| c.auto).unwrap_or(false) {
        return None;
    }
    let font = unsafe { HFONT(wm::SendMessageW(h, wm::WM_GETFONT, None, None).0 as *mut _) };
    let text = text_of(h);
    let (tw, _) = measure(h, font, if text.is_empty() { " " } else { &text });
    Some(tw + px(h, 3))
}

fn fit_label(h: HWND) {
    if let Some(w) = auto_width(h) {
        let r = rect_in_parent(h);
        move_to(h, r.left, r.top, w, r.bottom - r.top);
    }
}

/// Where a button's words sit: `"left"`, `"center"` or `"right"`.
pub fn set_align(h: HWND, align: &str) {
    with_css(h, |c| c.align = match align { "left" => 1, "right" => 3, _ => 2 });
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

static CSS: Mutex<Option<HashMap<isize, Css>>> = Mutex::new(None);

fn with_css<R>(h: HWND, f: impl FnOnce(&mut Css) -> R) -> Option<R> {
    let mut map = CSS.lock().unwrap_or_else(|e| e.into_inner());
    map.get_or_insert_with(HashMap::new).get_mut(&(h.0 as isize)).map(f)
}

fn mark(h: HWND, kind: Kind) {
    let mut map = CSS.lock().unwrap_or_else(|e| e.into_inner());
    map.get_or_insert_with(HashMap::new).insert(h.0 as isize, Css { kind, classes: vec![], id: None, group: 0, checked: false, hot: false, own: Style::default(), own_text: String::new(), align: 0, mixed: false, three: false, push: false, auto: false, preview: None, glyph: 0 });
}

fn forget(h: HWND) {
    let mut map = CSS.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(m) = map.as_mut() {
        m.remove(&(h.0 as isize));
    }
}

/// For controls drawn in other modules: CSS knows them by `kind`. Pair with [`forget_kind`] at
/// `WM_NCDESTROY`.
pub fn register_kind(h: HWND, kind: Kind) {
    mark(h, kind);
}

pub fn forget_kind(h: HWND) {
    forget(h);
}

/// Control `h`'s own look as it was given to [`set_own_style`]; empty when it has none.
pub fn own_style_text(h: HWND) -> String {
    with_css(h, |c| c.own_text.clone()).unwrap_or_default()
}

/// Control `h`'s kind, CSS classes and id, if the library made it.
pub fn css_names(h: HWND) -> Option<(Kind, Vec<String>, Option<String>)> {
    with_css(h, |c| (c.kind, c.classes.clone(), c.id.clone()))
}

/// The control's kind, if the library made it.
pub fn kind_of(h: HWND) -> Option<Kind> {
    with_css(h, |c| c.kind)
}

/// Gives a control CSS classes (space-separated) and an id, for `.class` and `#id` rules.
pub fn set_css(h: HWND, classes: &str, id: &str) {
    with_css(h, |c| {
        c.classes = classes.split_whitespace().map(str::to_string).collect();
        c.id = (!id.is_empty()).then(|| id.to_string());
    });
    // A rule for this class or id may change the font of a control Windows draws.
    if matches!(kind_of(h), Some(Kind::Label) | Some(Kind::Edit)) {
        theme_native(h);
    }
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, true);
    }
}

/// What the theme says about control `h` in `state`.
pub fn style_of(h: HWND, kind: Kind, state: State) -> Style {
    let (classes, id, own, preview) = with_css(h, |c| (c.classes.clone(), c.id.clone(), c.own.clone(), c.preview)).unwrap_or_default();
    let state = match preview {
        Some(p) => State { hover: state.hover || p.hover, pressed: state.pressed || p.pressed, focus: state.focus || p.focus, disabled: state.disabled || p.disabled, selected: state.selected || p.selected, checked: state.checked || p.checked },
        None => state,
    };
    let refs: Vec<&str> = classes.iter().map(String::as_str).collect();
    let q = Query { kind: kind.css(), classes: &refs, id: id.as_deref(), state };
    theme::current().style(&q).under(&own).under(&theme::state_rules_style(&q))
}

/// A colour change under way: from what was shown to what the state now asks, since `start`.
struct Anim {
    from: Style,
    to: Style,
    start: std::time::Instant,
}

static ANIMS: Mutex<Option<HashMap<isize, Anim>>> = Mutex::new(None);
const ANIM_TIMER: usize = 0x7A11;

/// [`style_of`] for the control's look as drawn now: with a CSS `transition`, a change of state
/// moves its colours over the transition's time instead of at once.
pub fn style_animated(h: HWND, kind: Kind, state: State) -> Style {
    let shown = style_transitioned(h, kind, state);
    animate(h, shown)
}

/// Controls with a CSS `animation` running: when each started, and its name.
static RUNNING: Mutex<Option<HashMap<isize, (String, std::time::Instant)>>> = Mutex::new(None);

/// `style` as its `animation` has it now (the frames of a `@keyframes`, or a built-in one), the
/// timer kept going while it runs.
fn animate(h: HWND, style: Style) -> Style {
    let mut map = RUNNING.lock().unwrap_or_else(|e| e.into_inner());
    let m = map.get_or_insert_with(HashMap::new);
    let key = h.0 as isize;
    let Some(a) = style.animation.clone() else {
        m.remove(&key);
        return style;
    };
    let start = match m.get(&key) {
        Some((name, at)) if *name == a.name => *at,
        _ => {
            let now = std::time::Instant::now();
            m.insert(key, (a.name.clone(), now));
            now
        }
    };
    let Some(frames) = theme::current().frames(&a.name, &style) else { return style };
    let (t, going) = a.at(start.elapsed().as_millis());
    if going {
        unsafe {
            wm::SetTimer(Some(h), ANIM_TIMER, 30, Some(anim_tick));
        }
    }
    if !going {
        // Done: as it was asked to look, as CSS leaves it without a fill mode.
        return style;
    }
    theme::frames_at(&frames, t)
}

fn style_transitioned(h: HWND, kind: Kind, state: State) -> Style {
    let target = style_of(h, kind, state);
    let mut map = ANIMS.lock().unwrap_or_else(|e| e.into_inner());
    let m = map.get_or_insert_with(HashMap::new);
    let key = h.0 as isize;
    let Some((ms, easing)) = target.transition.filter(|(ms, _)| *ms > 0) else {
        m.remove(&key);
        return target;
    };
    let now = std::time::Instant::now();
    let done = |a: &Anim| now.duration_since(a.start).as_millis() as i32 >= ms;
    let shown = |a: &Anim| if done(a) { a.to.clone() } else { a.from.mix(&a.to, easing.at(now.duration_since(a.start).as_millis() as f32 / ms as f32)) };
    match m.get(&key) {
        None => {
            m.insert(key, Anim { from: target.clone(), to: target.clone(), start: now - std::time::Duration::from_millis(ms as u64) });
            target
        }
        Some(a) if a.to == target => shown(a),
        Some(a) => {
            let from = shown(a);
            m.insert(key, Anim { from: from.clone(), to: target, start: now });
            unsafe {
                wm::SetTimer(Some(h), ANIM_TIMER, 15, Some(anim_tick));
            }
            from
        }
    }
}

extern "system" fn anim_tick(h: HWND, _: u32, _: usize, _: u32) {
    let running = ANIMS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|m| m.get(&(h.0 as isize)).map(|a| {
        let ms = a.to.transition.map(|t| t.0).unwrap_or(0);
        (a.start.elapsed().as_millis() as i32) < ms + 15
    })).unwrap_or(false);
    unsafe {
        // An animation keeps it going; `animate` sets it again on each frame it draws.
        if !running {
            let _ = wm::KillTimer(Some(h), ANIM_TIMER);
        }
        let _ = Gdi::RedrawWindow(Some(h), None, None, Gdi::RDW_INVALIDATE | Gdi::RDW_FRAME);
    }
}

fn window_style() -> Style {
    theme::style("window", State::default())
}

type RootColors = HashMap<isize, (Option<Color>, Option<Color>)>;
static ROOT_COLORS: Mutex<Option<RootColors>> = Mutex::new(None);

/// A window's own background and words over the theme's - a form's BackColor and ForeColor.
/// Every control in it draws on that background.
pub fn set_window_colors(hwnd: HWND, background: Option<Color>, color: Option<Color>) {
    ROOT_COLORS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).insert(hwnd.0 as isize, (background, color));
}

fn root_colors(h: HWND) -> (Option<Color>, Option<Color>) {
    let map = ROOT_COLORS.lock().unwrap_or_else(|e| e.into_inner());
    let Some(m) = map.as_ref().filter(|m| !m.is_empty()) else { return (None, None) };
    let root = unsafe { wm::GetAncestor(h, wm::GA_ROOT) };
    m.get(&(root.0 as isize)).copied().unwrap_or((None, None))
}

/// What is behind control `h` in `r` of its `dc`: the window's gradient or picture where it has
/// one (Windows asks the parent to paint into the control's place), its colour else.
pub fn paint_behind(h: HWND, dc: HDC, r: &RECT) {
    if window_patterned(h) {
        unsafe {
            let _ = windows::Win32::UI::Controls::DrawThemeParentBackground(h, dc, Some(r));
        }
    } else {
        fill(dc, r, window_background_for(h));
    }
}

/// Whether the window `h` is in shows more than one colour behind its controls: a gradient, or
/// a picture.
pub fn window_patterned(h: HWND) -> bool {
    let root = unsafe { wm::GetAncestor(h, wm::GA_ROOT) };
    crate::form::has_background_image(root) || (root_colors(h).0.is_none() && window_style().gradient.is_some())
}

/// The background of the window `h` is in: its own, or the theme's.
pub fn window_background_for(h: HWND) -> Color {
    root_colors(h).0.unwrap_or_else(window_background)
}

/// The words' colour of the window `h` is in.
pub fn window_text_for(h: HWND) -> Color {
    root_colors(h).1.unwrap_or_else(window_text)
}

pub fn window_background() -> Color {
    window_style().background_or(Color::rgb(255, 255, 255))
}

pub fn window_text() -> Color {
    window_style().color_or(Color::rgb(27, 27, 27))
}

fn cr(c: Color) -> COLORREF {
    COLORREF(c.colorref())
}

/// A solid brush of `c`, kept for the life of the process - there are only as many as the
/// theme has colours, and a brush handed back from `WM_CTLCOLOR*` must outlive the paint.
pub fn brush(c: Color) -> Gdi::HBRUSH {
    static BRUSHES: Mutex<Option<HashMap<u32, isize>>> = Mutex::new(None);
    let mut map = BRUSHES.lock().unwrap_or_else(|e| e.into_inner());
    let b = map.get_or_insert_with(HashMap::new).entry(c.colorref()).or_insert_with(|| unsafe { Gdi::CreateSolidBrush(cr(c)).0 as isize });
    Gdi::HBRUSH(*b as *mut _)
}

/// A rounded rectangle filled with `fill` and outlined in `border`, `radius` device pixels.
pub fn round_rect(dc: HDC, r: RECT, fill: Color, border: Color, radius: i32) {
    unsafe {
        let pen = Gdi::CreatePen(Gdi::PS_SOLID, 1, cr(border));
        let old_pen = Gdi::SelectObject(dc, pen.into());
        let old_brush = if fill.a == 0 { Gdi::SelectObject(dc, Gdi::GetStockObject(Gdi::NULL_BRUSH)) } else { Gdi::SelectObject(dc, brush(fill).into()) };
        let _ = Gdi::RoundRect(dc, r.left, r.top, r.right, r.bottom, radius, radius);
        Gdi::SelectObject(dc, old_brush);
        Gdi::SelectObject(dc, old_pen);
        let _ = Gdi::DeleteObject(pen.into());
    }
}

/// How far into `r` a box with shadow `sh` (pixels) must stand for its shadow to show: left, top,
/// right, bottom - CSS draws a shadow outside the box; a window cannot draw outside itself.
pub fn shadow_room(sh: Option<theme::Shadow>) -> (i32, i32, i32, i32) {
    let Some(s) = sh else { return (0, 0, 0, 0) };
    // Half the blur: its faint outer half may fall outside, as it fades to nothing.
    let reach = s.blur / 2 + s.spread.max(0);
    ((reach - s.x).max(0), (reach - s.y).max(0), (reach + s.x).max(0), (reach + s.y).max(0))
}

/// A box with what CSS can give it - a gradient, a soft shadow, a border, round corners drawn
/// smooth - into `r` of `dc`. `radius` and the shadow are in pixels already. The box stands inside
/// `r` far enough for its shadow ([`shadow_room`]); where it stands is returned, for its words.
#[allow(clippy::too_many_arguments)]
pub fn paint_box(dc: HDC, r: RECT, fill: Color, gradient: Option<&theme::Gradient>, shadow: Option<theme::Shadow>, border: Color, border_width: i32, radius: i32) -> RECT {
    let (w, h) = (r.right - r.left, r.bottom - r.top);
    if w <= 0 || h <= 0 {
        return r;
    }
    let (rl, rt, rr, rb) = shadow_room(shadow);
    let body = RECT { left: r.left + rl, top: r.top + rt, right: r.right - rr, bottom: r.bottom - rb };
    // The signed distance from a point to a rounded box: below 0 inside.
    let sdf = |px: f32, py: f32, b: (f32, f32, f32, f32), rad: f32| {
        let (cx, cy) = ((b.0 + b.2) / 2.0, (b.1 + b.3) / 2.0);
        let (hx, hy) = (((b.2 - b.0) / 2.0).max(0.0), ((b.3 - b.1) / 2.0).max(0.0));
        let rad = rad.min(hx).min(hy).max(0.0);
        let qx = (px - cx).abs() - hx + rad;
        let qy = (py - cy).abs() - hy + rad;
        (qx.max(0.0).hypot(qy.max(0.0))) + qx.max(qy).min(0.0) - rad
    };
    let local = |b: RECT| ((b.left - r.left) as f32, (b.top - r.top) as f32, (b.right - r.left) as f32, (b.bottom - r.top) as f32);
    let bb = local(body);
    let rad = radius as f32 / 2.0;
    let sh = shadow.map(|s| {
        let b = RECT { left: body.left + s.x - s.spread, top: body.top + s.y - s.spread, right: body.right + s.x + s.spread, bottom: body.bottom + s.y + s.spread };
        (local(b), (rad + s.spread as f32).max(0.0), (s.blur as f32).max(0.5), s.color)
    });
    // The gradient's line through the box's middle, CSS's way: 0 degrees up, clockwise.
    let (dx, dy, len) = gradient.map(|g| {
        let a = g.angle.to_radians();
        let (dx, dy) = (a.sin(), -a.cos());
        let (bw, bh) = (bb.2 - bb.0, bb.3 - bb.1);
        (dx, dy, (bw * dx.abs() + bh * dy.abs()).max(1.0))
    }).unwrap_or((0.0, 1.0, 1.0));
    let bw_px = border_width.max(0) as f32;
    let mut pixels = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            // Premultiplied BGRA, painted bottom up: shadow, then the box over it.
            let mut out = [0.0f32; 4];
            let over = |out: &mut [f32; 4], c: Color, cover: f32| {
                let a = c.a as f32 / 255.0 * cover;
                for (k, ch) in [c.b, c.g, c.r].iter().enumerate() {
                    out[k] = *ch as f32 / 255.0 * a + out[k] * (1.0 - a);
                }
                out[3] = a + out[3] * (1.0 - a);
            };
            if let Some((b, srad, blur, col)) = sh {
                let d = sdf(px, py, b, srad);
                let t = (0.5 - d / (blur * 2.0)).clamp(0.0, 1.0);
                over(&mut out, col, t * t * (3.0 - 2.0 * t));
            }
            let d = sdf(px, py, bb, rad);
            let cover = (0.5 - d).clamp(0.0, 1.0);
            if cover > 0.0 {
                let c = match gradient {
                    Some(g) => {
                        let t = ((px - (bb.0 + bb.2) / 2.0) * dx + (py - (bb.1 + bb.3) / 2.0) * dy) / len + 0.5;
                        g.at(t)
                    }
                    None => fill,
                };
                over(&mut out, c, cover);
                // The border: the box's last pixels inward.
                if bw_px > 0.0 && border.a > 0 {
                    let ring = (d + bw_px + 0.5).clamp(0.0, 1.0);
                    over(&mut out, border, ring * cover);
                }
            }
            let i = ((y * w + x) * 4) as usize;
            for k in 0..4 {
                pixels[i + k] = (out[k] * 255.0).round() as u8;
            }
        }
    }
    unsafe {
        let bi = Gdi::BITMAPINFO { bmiHeader: Gdi::BITMAPINFOHEADER { biSize: std::mem::size_of::<Gdi::BITMAPINFOHEADER>() as u32, biWidth: w, biHeight: -h, biPlanes: 1, biBitCount: 32, biCompression: 0, ..Default::default() }, ..Default::default() };
        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let Ok(bmp) = Gdi::CreateDIBSection(Some(dc), &bi, Gdi::DIB_RGB_COLORS, &mut bits, None, 0) else { return body };
        std::ptr::copy_nonoverlapping(pixels.as_ptr(), bits as *mut u8, pixels.len());
        let mem = Gdi::CreateCompatibleDC(Some(dc));
        let old = Gdi::SelectObject(mem, bmp.into());
        let blend = Gdi::BLENDFUNCTION { BlendOp: 0, BlendFlags: 0, SourceConstantAlpha: 255, AlphaFormat: 1 };
        let _ = Gdi::AlphaBlend(dc, r.left, r.top, w, h, mem, 0, 0, w, h, blend);
        Gdi::SelectObject(mem, old);
        let _ = Gdi::DeleteDC(mem);
        let _ = Gdi::DeleteObject(bmp.into());
    }
    body
}

/// A style's shadow at `h`'s scale.
pub fn shadow_px(h: HWND, s: &Style) -> Option<theme::Shadow> {
    s.shadow.flatten().map(|sh| theme::Shadow { x: px(h, sh.x), y: px(h, sh.y), blur: px(h, sh.blur), spread: px(h, sh.spread), color: sh.color })
}

/// A plain area in a style's background: its gradient, or its colour.
pub fn fill_style(dc: HDC, r: &RECT, s: &Style, fallback: Color) {
    match &s.gradient {
        Some(g) => {
            fill(dc, r, s.background_or(fallback));
            paint_box(dc, *r, s.background_or(fallback), Some(g), None, Color::TRANSPARENT, 0, 0);
        }
        None => fill(dc, r, s.background_or(fallback)),
    }
}

pub fn fill(dc: HDC, r: &RECT, c: Color) {
    unsafe {
        Gdi::FillRect(dc, r, brush(c));
    }
}

/// The font a style asks for with `font-size` / `font-weight`, at `h`'s scale; `None` when it asks
/// for neither, and the control keeps its own. Kept for the life of the process, one per size,
/// weight and scale.
pub fn css_font(h: HWND, s: &Style) -> Option<HFONT> {
    /// (points, weight, DPI) to the font.
    type Fonts = HashMap<(i32, i32, i32), isize>;
    static FONTS: Mutex<Option<Fonts>> = Mutex::new(None);
    if s.font_size.is_none() && s.font_weight.is_none() {
        return None;
    }
    let d = dpi(h);
    // A weight alone keeps the size the control has - a 14 pt label made bold stays 14 pt.
    let own_height = if s.font_size.is_none() {
        unsafe {
            let f = HFONT(wm::SendMessageW(h, wm::WM_GETFONT, None, None).0 as *mut _);
            let mut lf = Gdi::LOGFONTW::default();
            if !f.is_invalid() && Gdi::GetObjectW(f.into(), std::mem::size_of::<Gdi::LOGFONTW>() as i32, Some(&mut lf as *mut _ as *mut _)) > 0 {
                Some(lf.lfHeight)
            } else {
                None
            }
        }
    } else {
        None
    };
    let key = (s.font_size.unwrap_or(0), s.font_weight.unwrap_or(0), d * 1000 + own_height.unwrap_or(0));
    let mut map = FONTS.lock().unwrap_or_else(|e| e.into_inner());
    let f = map.get_or_insert_with(HashMap::new).entry(key).or_insert_with(|| unsafe {
        let mut metrics = wm::NONCLIENTMETRICSW { cbSize: std::mem::size_of::<wm::NONCLIENTMETRICSW>() as u32, ..Default::default() };
        let _ = HiDpi::SystemParametersInfoForDpi(wm::SPI_GETNONCLIENTMETRICS.0, metrics.cbSize, Some(&mut metrics as *mut _ as *mut _), 0, d as u32);
        let mut lf = metrics.lfMessageFont;
        if let Some(height) = own_height {
            lf.lfHeight = height;
        }
        if let Some(pt) = s.font_size {
            lf.lfHeight = -(pt * d / 72);
        }
        if let Some(wt) = s.font_weight {
            lf.lfWeight = wt;
        }
        Gdi::CreateFontIndirectW(&lf).0 as isize
    });
    Some(HFONT(*f as *mut _))
}

/// Text in `colour`, in the font `s` asks for, else the control's own.
pub fn draw_text_styled(dc: HDC, h: HWND, text: &str, r: &mut RECT, colour: Color, flags: Gdi::DRAW_TEXT_FORMAT, s: &Style) {
    if text.is_empty() {
        return;
    }
    let font = css_font(h, s).unwrap_or_else(|| unsafe { HFONT(wm::SendMessageW(h, wm::WM_GETFONT, None, None).0 as *mut _) });
    unsafe {
        let old = Gdi::SelectObject(dc, font.into());
        Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        Gdi::SetTextColor(dc, cr(colour));
        let mut w: Vec<u16> = text.encode_utf16().collect();
        Gdi::DrawTextW(dc, &mut w, r, flags);
        Gdi::SelectObject(dc, old);
    }
}

/// Text in `colour`, in the control's own font.
pub fn draw_text(dc: HDC, h: HWND, text: &str, r: &mut RECT, colour: Color, flags: Gdi::DRAW_TEXT_FORMAT) {
    // An empty buffer is a dangling pointer with no length, and DrawTextW reads it all the same.
    if text.is_empty() {
        return;
    }
    unsafe {
        let font = HFONT(wm::SendMessageW(h, wm::WM_GETFONT, None, None).0 as *mut _);
        let old = Gdi::SelectObject(dc, font.into());
        Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        Gdi::SetTextColor(dc, cr(colour));
        let mut w: Vec<u16> = text.encode_utf16().collect();
        Gdi::DrawTextW(dc, &mut w, r, flags);
        Gdi::SelectObject(dc, old);
    }
}

// --- buttons, checkboxes and radios -------------------------------------------------------------
//
// Owner-drawn, as Windows 11's own are: the standard push button on a classic Win32 window is
// still the old grey one, and a themed checkbox cannot take the theme's colours. These remember
// hover and checked themselves, and answer BM_GETCHECK / BM_SETCHECK as the standard ones do.

const BS_OWNERDRAW: u32 = 0x0B;
const BM_GETCHECK: u32 = 0x00F0;
const BM_SETCHECK: u32 = 0x00F1;
const BM_GETSTATE: u32 = 0x00F2;
const BST_PUSHED: isize = 0x0004;
const WM_MOUSELEAVE: u32 = 0x02A3;

fn owner_drawn(parent: HWND, text: &str, id: i32, f: HFONT, kind: Kind) -> HWND {
    let h = child(parent, "BUTTON", text, WINDOW_STYLE(BS_OWNERDRAW) | wm::WS_TABSTOP, WINDOW_EX_STYLE(0), id, f);
    mark(h, kind);
    unsafe {
        let _ = windows::Win32::UI::Shell::SetWindowSubclass(h, Some(button_proc), 1, 0);
    }
    h
}

/// A push button. `primary` is the one that answers a dialog, in the accent colour (CSS
/// `button.primary`).
pub fn button(parent: HWND, text: &str, id: i32, f: HFONT, primary: bool) -> HWND {
    let h = owner_drawn(parent, text, id, f, Kind::Button);
    if primary {
        set_css(h, "primary", "");
    }
    h
}

pub fn checkbox(parent: HWND, text: &str, id: i32, f: HFONT, checked: bool) -> HWND {
    let h = owner_drawn(parent, text, id, f, Kind::Checkbox);
    set_checked(h, checked);
    h
}

/// A radio; radios with the same parent and `group` exclude each other.
pub fn radio(parent: HWND, text: &str, id: i32, f: HFONT, group: i32, checked: bool) -> HWND {
    let h = owner_drawn(parent, text, id, f, Kind::Radio);
    with_css(h, |c| c.group = group);
    if checked {
        set_checked(h, true);
    }
    h
}

pub fn is_checked(h: impl Into<HWND>) -> bool {
    let h: HWND = h.into();
    unsafe { wm::SendMessageW(h, BM_GETCHECK, None, None).0 != 0 }
}

pub fn set_checked(h: impl Into<HWND>, on: bool) {
    let h: HWND = h.into();
    unsafe {
        wm::SendMessageW(h, BM_SETCHECK, Some(WPARAM(on as usize)), None);
    }
}

/// A click on a radio: it is checked, and the others of its group are not.
fn check_radio(h: HWND) {
    let Some(group) = with_css(h, |c| c.group) else { return };
    let parent = unsafe { wm::GetParent(h).unwrap_or_default() };
    let mut map = CSS.lock().unwrap_or_else(|e| e.into_inner());
    let others: Vec<isize> = map
        .get_or_insert_with(HashMap::new)
        .iter()
        .filter(|(k, c)| c.kind == Kind::Radio && c.group == group && c.checked && **k != h.0 as isize && unsafe { wm::GetParent(HWND(**k as *mut _)) }.unwrap_or_default() == parent)
        .map(|(k, _)| *k)
        .collect();
    let m = map.as_mut().unwrap();
    for k in &others {
        if let Some(c) = m.get_mut(k) {
            c.checked = false;
        }
    }
    if let Some(c) = m.get_mut(&(h.0 as isize)) {
        c.checked = true;
    }
    drop(map);
    unsafe {
        for k in others {
            let _ = Gdi::InvalidateRect(Some(HWND(k as *mut _)), None, false);
        }
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

unsafe extern "system" fn button_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _id: usize, _data: usize) -> LRESULT {
    use windows::Win32::UI::Input::KeyboardAndMouse::{TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT};
    match msg {
        wm::WM_MOUSEMOVE => {
            if with_css(h, |c| !std::mem::replace(&mut c.hot, true)).unwrap_or(false) {
                let mut t = TRACKMOUSEEVENT { cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32, dwFlags: TME_LEAVE, hwndTrack: h, dwHoverTime: 0 };
                let _ = TrackMouseEvent(&mut t);
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
        }
        WM_MOUSELEAVE => {
            with_css(h, |c| c.hot = false);
            let _ = Gdi::InvalidateRect(Some(h), None, false);
        }
        BM_GETCHECK => return LRESULT(check_state(h) as isize),
        BM_SETCHECK => {
            let on = w.0 != 0;
            with_css(h, |c| c.mixed = w.0 == 2);
            if on && kind_of(h) == Some(Kind::Radio) {
                check_radio(h);
            } else {
                with_css(h, |c| c.checked = on);
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            return LRESULT(0);
        }
        // The release that makes a click: toggle before the parent hears BN_CLICKED, so it reads
        // the new state.
        wm::WM_LBUTTONUP | wm::WM_KEYUP => {
            let space = msg == wm::WM_KEYUP && w.0 == 0x20;
            let pushed = wm::SendMessageW(h, BM_GETSTATE, None, None).0 & BST_PUSHED != 0;
            if (msg == wm::WM_LBUTTONUP || space) && pushed {
                match kind_of(h) {
                    Some(Kind::Checkbox) => {
                        // Unticked, ticked, then - with ThreeState - neither.
                        let three = with_css(h, |c| c.three).unwrap_or(false);
                        set_check_state(h, match check_state(h) {
                            0 => 1,
                            1 if three => 2,
                            _ => 0,
                        });
                    }
                    Some(Kind::Radio) => check_radio(h),
                    _ => {}
                }
            }
        }
        wm::WM_NCDESTROY => {
            forget(h);
            let _ = windows::Win32::UI::Shell::RemoveWindowSubclass(h, Some(button_proc), 1);
        }
        _ => {}
    }
    windows::Win32::UI::Shell::DefSubclassProc(h, msg, w, l)
}

/// `WM_DRAWITEM` for the library's buttons, checkboxes and radios. False for anything else.
pub fn draw_item(l: LPARAM) -> bool {
    use windows::Win32::UI::Controls::{DRAWITEMSTRUCT, ODS_DISABLED, ODS_FOCUS, ODS_SELECTED, ODT_BUTTON};
    let d = unsafe { &*(l.0 as *const DRAWITEMSTRUCT) };
    if d.CtlType != ODT_BUTTON {
        return false;
    }
    let Some((real_kind, hot, checked, mixed, push)) = with_css(d.hwndItem, |c| (c.kind, c.hot, c.checked, c.mixed, c.push)) else { return false };
    // Appearance = Button: drawn as a button, pressed while ticked.
    let push = push && matches!(real_kind, Kind::Checkbox | Kind::Radio);
    let kind = if push { Kind::Button } else { real_kind };
    let state = State {
        hover: hot,
        pressed: d.itemState.0 & ODS_SELECTED.0 != 0 || (push && checked),
        focus: d.itemState.0 & ODS_FOCUS.0 != 0,
        disabled: d.itemState.0 & ODS_DISABLED.0 != 0,
        checked,
        selected: false,
    };
    let s = style_animated(d.hwndItem, kind, state);
    let dc = d.hDC;
    let r = d.rcItem;
    let h = d.hwndItem;
    // The parent's colour behind the rounded corners and around the box.
    paint_behind(h, dc, &r);
    let text = text_of(h);
    let fg = s.color_or(window_text_for(h));
    match kind {
        Kind::Checkbox | Kind::Radio => {
            // The box as Windows and Visual Studio size it beside 9-point text; the tick scales with it.
            let size = px(h, 14);
            let top = r.top + (r.bottom - r.top - size) / 2;
            let bx = RECT { left: r.left, top, right: r.left + size, bottom: top + size };
            let bg = s.background_or(Color::rgb(255, 255, 255));
            let border = s.border_or(Color::rgb(138, 138, 138));
            unsafe {
                if kind == Kind::Radio {
                    let pen = Gdi::CreatePen(Gdi::PS_SOLID, 1, cr(border));
                    let op = Gdi::SelectObject(dc, pen.into());
                    let ob = Gdi::SelectObject(dc, brush(bg).into());
                    let _ = Gdi::Ellipse(dc, bx.left, bx.top, bx.right, bx.bottom);
                    if checked {
                        let inset = size * 3 / 10;
                        let dot = brush(s.accent_or(Color::rgb(255, 255, 255)));
                        Gdi::SelectObject(dc, dot.into());
                        Gdi::SelectObject(dc, Gdi::GetStockObject(Gdi::NULL_PEN));
                        let _ = Gdi::Ellipse(dc, bx.left + inset, bx.top + inset, bx.right - inset + 1, bx.bottom - inset + 1);
                    }
                    Gdi::SelectObject(dc, ob);
                    Gdi::SelectObject(dc, op);
                    let _ = Gdi::DeleteObject(pen.into());
                } else {
                    if s.gradient.is_some() || s.shadow.flatten().is_some() {
                        paint_box(dc, bx, bg, s.gradient.as_ref(), shadow_px(h, &s), border, px(h, 1), px(h, s.border_radius.unwrap_or(3) * 2));
                    } else {
                        round_rect(dc, bx, bg, border, px(h, s.border_radius.unwrap_or(3) * 2));
                    }
                    if mixed {
                        // Neither: a square of the accent in the box.
                        let inset = size * 3 / 10;
                        round_rect(dc, RECT { left: bx.left + inset, top: bx.top + inset, right: bx.right - inset, bottom: bx.bottom - inset }, s.accent_or(Color::rgb(0x12, 0x85, 0x5f)), s.accent_or(Color::rgb(0x12, 0x85, 0x5f)), px(h, 2));
                    } else if checked {
                        // The tick: two strokes, as Windows 11 draws it.
                        let pen = Gdi::CreatePen(Gdi::PS_SOLID, px(h, 2).max(1), cr(s.accent_or(Color::rgb(255, 255, 255))));
                        let op = Gdi::SelectObject(dc, pen.into());
                        let (x, y, u) = (bx.left, bx.top, size as f32 / 18.0);
                        let p = |a: f32, b: f32| (x + (a * u) as i32, y + (b * u) as i32);
                        let (a, b, c) = (p(4.5, 9.5), p(7.5, 12.5), p(13.5, 5.5));
                        let _ = Gdi::MoveToEx(dc, a.0, a.1, None);
                        let _ = Gdi::LineTo(dc, b.0, b.1);
                        let _ = Gdi::LineTo(dc, c.0, c.1);
                        Gdi::SelectObject(dc, op);
                        let _ = Gdi::DeleteObject(pen.into());
                    }
                }
            }
            let label_fg = if state.disabled { fg } else { style_of(h, kind, State::default()).color_or(window_text_for(h)) };
            let mut tr = RECT { left: bx.right + px(h, 6), ..r };
            draw_text_styled(dc, h, &text, &mut tr, label_fg, Gdi::DT_LEFT | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE, &s);
            if state.focus {
                let mut fr = tr;
                let (w, th) = measure(h, unsafe { HFONT(wm::SendMessageW(h, wm::WM_GETFONT, None, None).0 as *mut _) }, &text);
                fr.right = fr.left + w;
                fr.top = r.top + (r.bottom - r.top - th) / 2;
                fr.bottom = fr.top + th;
                let line = RECT { top: fr.bottom, bottom: fr.bottom + 1, ..fr };
                fill(dc, &line, s.border_or(fg));
            }
        }
        _ => {
            let radius = px(h, s.border_radius.unwrap_or(4) * 2);
            // A gradient or a shadow: drawn as CSS draws them, the corners smooth; else plainly.
            let body = if s.gradient.is_some() || s.shadow.flatten().is_some() {
                paint_box(dc, r, s.background_or(Color::rgb(255, 255, 255)), s.gradient.as_ref(), shadow_px(h, &s), s.border_or(Color::rgb(212, 212, 212)), px(h, s.border_width.unwrap_or(1)), radius)
            } else {
                round_rect(dc, r, s.background_or(Color::rgb(255, 255, 255)), s.border_or(Color::rgb(212, 212, 212)), radius);
                r
            };
            let pad = px(h, 8);
            let mut tr = RECT { left: body.left + pad, right: body.right - pad, top: body.top, bottom: body.bottom };
            let align = match with_css(h, |c| c.align).unwrap_or(0) {
                1 => Gdi::DT_LEFT,
                3 => Gdi::DT_RIGHT,
                _ => Gdi::DT_CENTER,
            };
            let glyph = with_css(h, |c| c.glyph).unwrap_or(0);
            if glyph != 0 {
                // The icon and the words together, centred as one (or at the side they are put).
                let size = px(h, 16);
                let gap = if text.is_empty() { 0 } else { px(h, 6) };
                let (tw, _) = measure(h, unsafe { HFONT(wm::SendMessageW(h, wm::WM_GETFONT, None, None).0 as *mut _) }, &text);
                let whole = size + gap + tw;
                let x = match align {
                    Gdi::DT_LEFT => tr.left,
                    Gdi::DT_RIGHT => tr.right - whole,
                    _ => tr.left + ((tr.right - tr.left) - whole).max(0) / 2,
                };
                let y = tr.top + (tr.bottom - tr.top - size) / 2;
                if let Some(i) = crate::icon::glyph_icon(glyph, size, fg) {
                    unsafe {
                        let _ = wm::DrawIconEx(dc, x, y, i, size, size, 0, None, wm::DI_NORMAL);
                        let _ = wm::DestroyIcon(i);
                    }
                }
                tr.left = x + size + gap;
                draw_text_styled(dc, h, &text, &mut tr, fg, Gdi::DT_LEFT | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE, &s);
            } else {
                draw_text_styled(dc, h, &text, &mut tr, fg, align | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE, &s);
            }
        }
    }
    true
}

// --- labels, edits, combos: what Windows draws, in the theme's colours ---------------------------

pub fn label(parent: HWND, text: &str, id: i32, f: HFONT) -> HWND {
    const SS_NOPREFIX: u32 = 0x80;
    const SS_CENTERIMAGE: u32 = 0x200;
    let h = child(parent, "STATIC", text, WINDOW_STYLE(SS_NOPREFIX | SS_CENTERIMAGE), WINDOW_EX_STYLE(0), id, f);
    mark(h, Kind::Label);
    theme_native(h);
    h
}

/// A one-line edit with a thin border in the theme's colour - the accent while it has focus.
pub fn edit(parent: HWND, text: &str, id: i32, f: HFONT, password: bool) -> HWND {
    const ES_AUTOHSCROLL: u32 = 0x0080;
    const ES_PASSWORD: u32 = 0x0020;
    const EM_SETMARGINS: u32 = 0x00D3;
    // No Windows border: the subclass keeps an edge and draws the theme's frame on it, the words
    // in the middle of the box.
    let style = WINDOW_STYLE(ES_AUTOHSCROLL | if password { ES_PASSWORD } else { 0 }) | wm::WS_TABSTOP;
    let h = child(parent, "EDIT", text, style, WINDOW_EX_STYLE(0), id, f);
    mark(h, Kind::Edit);
    unsafe {
        let m = px(parent, 6) as isize;
        wm::SendMessageW(h, EM_SETMARGINS, Some(WPARAM(3)), Some(LPARAM((m << 16) | m)));
        let _ = windows::Win32::UI::Shell::SetWindowSubclass(h, Some(edit_proc), 2, 0);
    }
    h
}

/// A box of several lines, in a fixed-width font so columns of text line up, scrolling down.
pub fn edit_lines(parent: HWND, text: &str, id: i32, f: HFONT) -> HWND {
    const ES_MULTILINE: u32 = 0x0004;
    const ES_AUTOVSCROLL: u32 = 0x0040;
    const ES_WANTRETURN: u32 = 0x1000;
    const EM_SETMARGINS: u32 = 0x00D3;
    let style = WINDOW_STYLE(ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN) | wm::WS_TABSTOP | wm::WS_BORDER | wm::WS_VSCROLL;
    let h = child(parent, "EDIT", text, style, WINDOW_EX_STYLE(0), id, f);
    mark(h, Kind::Edit);
    unsafe {
        // 9 points: as large as the window's own words look - Cascadia at 10 stood out above them.
        let mut lf = Gdi::LOGFONTW { lfHeight: -(9 * dpi(parent) / 72), lfCharSet: Gdi::DEFAULT_CHARSET, lfQuality: Gdi::CLEARTYPE_QUALITY, ..Default::default() };
        for (i, u) in "Cascadia Mono".encode_utf16().enumerate() {
            lf.lfFaceName[i] = u;
        }
        let _ = f;
        set_font(h, Gdi::CreateFontIndirectW(&lf));
        let m = px(parent, 6) as isize;
        wm::SendMessageW(h, EM_SETMARGINS, Some(WPARAM(3)), Some(LPARAM((m << 16) | m)));
        let _ = windows::Win32::UI::Shell::SetWindowSubclass(h, Some(edit_proc), 2, 0);
    }
    h
}

unsafe extern "system" fn edit_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _id: usize, _data: usize) -> LRESULT {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetFocus, TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT};
    match msg {
        wm::WM_NCPAINT => {
            // A box with its words in the middle (see WM_NCCALCSIZE): its whole edge painted
            // here - Windows would frame the smaller middle too. Any other: Windows first.
            let centred = {
                let (mut wr0, mut cr0) = (RECT::default(), RECT::default());
                let _ = wm::GetWindowRect(h, &mut wr0);
                let _ = wm::GetClientRect(h, &mut cr0);
                (wr0.bottom - wr0.top) - (cr0.bottom - cr0.top) > 4
                    && wm::GetWindowLongW(h, wm::GWL_STYLE) as u32 & 0x0004 /* ES_MULTILINE */ == 0
            };
            let r = if centred { LRESULT(0) } else { windows::Win32::UI::Shell::DefSubclassProc(h, msg, w, l) };
            let state = State {
                hover: with_css(h, |c| c.hot).unwrap_or(false),
                focus: GetFocus() == h,
                disabled: !windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled(h).as_bool(),
                ..Default::default()
            };
            let s = style_animated(h, Kind::Edit, state);
            let dc = Gdi::GetWindowDC(Some(h));
            let mut wr = RECT::default();
            let _ = wm::GetWindowRect(h, &mut wr);
            let rr = RECT { left: 0, top: 0, right: wr.right - wr.left, bottom: wr.bottom - wr.top };
            // The room above and below the centred line: the box's own colour.
            let mut cr = RECT::default();
            let _ = wm::GetClientRect(h, &mut cr);
            let mut at = [windows::Win32::Foundation::POINT { x: cr.left, y: cr.top }, windows::Win32::Foundation::POINT { x: cr.right, y: cr.bottom }];
            Gdi::MapWindowPoints(Some(h), None, &mut at);
            let (top, bottom) = (at[0].y - wr.top, at[1].y - wr.top);
            if top > 1 || bottom < rr.bottom - 1 {
                let bg = s.background_or(Color::rgb(255, 255, 255));
                fill(dc, &RECT { left: 1, top: 1, right: rr.right - 1, bottom: top }, bg);
                fill(dc, &RECT { left: 1, top: bottom, right: rr.right - 1, bottom: rr.bottom - 1 }, bg);
            }
            let b = brush(s.border_or(Color::rgb(212, 212, 212)));
            Gdi::FrameRect(dc, &rr, b);
            Gdi::ReleaseDC(Some(h), dc);
            return r;
        }
        // A one-line box taller than its words: the words in the middle, as a browser's text
        // box has them - not against the top, as Windows leaves them. (A number box's arrows
        // live in it: that one stays as it is.)
        wm::WM_NCCALCSIZE if w.0 != 0 => {
            let r = windows::Win32::UI::Shell::DefSubclassProc(h, msg, w, l);
            let style = wm::GetWindowLongW(h, wm::GWL_STYLE) as u32;
            let multi = style & 0x0004 /* ES_MULTILINE */ != 0;
            let has_child = wm::GetWindow(h, wm::GW_CHILD).is_ok_and(|c| !c.is_invalid());
            // A one-line box has no Windows border (it would frame the middle too): the edge for
            // the frame WM_NCPAINT draws is kept here.
            if !multi && style & wm::WS_BORDER.0 == 0 {
                let p = &mut *(l.0 as *mut wm::NCCALCSIZE_PARAMS);
                p.rgrc[0].left += 1;
                p.rgrc[0].top += 1;
                p.rgrc[0].right -= 1;
                p.rgrc[0].bottom -= 1;
            }
            if !multi && !has_child {
                let p = &mut *(l.0 as *mut wm::NCCALCSIZE_PARAMS);
                let dc = Gdi::GetDC(Some(h));
                let font = Gdi::HGDIOBJ(wm::SendMessageW(h, wm::WM_GETFONT, None, None).0 as *mut _);
                let old = Gdi::SelectObject(dc, font);
                let mut tm = Gdi::TEXTMETRICW::default();
                let _ = Gdi::GetTextMetricsW(dc, &mut tm);
                Gdi::SelectObject(dc, old);
                Gdi::ReleaseDC(Some(h), dc);
                let room = p.rgrc[0].bottom - p.rgrc[0].top;
                let line = tm.tmHeight + 2;
                if room > line {
                    let pad = (room - line) / 2;
                    p.rgrc[0].top += pad;
                    p.rgrc[0].bottom -= room - line - pad;
                }
            }
            return r;
        }
        // A new font: the middle worked out again.
        wm::WM_SETFONT => {
            let r = windows::Win32::UI::Shell::DefSubclassProc(h, msg, w, l);
            let _ = wm::SetWindowPos(h, None, 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOZORDER | wm::SWP_NOACTIVATE | wm::SWP_FRAMECHANGED);
            return r;
        }
        wm::WM_SETFOCUS | wm::WM_KILLFOCUS | wm::WM_ENABLE => {
            let r = windows::Win32::UI::Shell::DefSubclassProc(h, msg, w, l);
            let _ = Gdi::RedrawWindow(Some(h), None, None, Gdi::RDW_FRAME | Gdi::RDW_INVALIDATE);
            return r;
        }
        wm::WM_MOUSEMOVE => {
            if with_css(h, |c| !std::mem::replace(&mut c.hot, true)).unwrap_or(false) {
                let mut t = TRACKMOUSEEVENT { cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32, dwFlags: TME_LEAVE, hwndTrack: h, dwHoverTime: 0 };
                let _ = TrackMouseEvent(&mut t);
                let _ = Gdi::RedrawWindow(Some(h), None, None, Gdi::RDW_FRAME | Gdi::RDW_INVALIDATE);
            }
        }
        WM_MOUSELEAVE => {
            with_css(h, |c| c.hot = false);
            let _ = Gdi::RedrawWindow(Some(h), None, None, Gdi::RDW_FRAME | Gdi::RDW_INVALIDATE);
        }
        wm::WM_NCDESTROY => {
            forget(h);
            let _ = windows::Win32::UI::Shell::RemoveWindowSubclass(h, Some(edit_proc), 2);
        }
        _ => {}
    }
    windows::Win32::UI::Shell::DefSubclassProc(h, msg, w, l)
}

/// A drop-down list to choose one of `items`.
pub fn combo(parent: HWND, id: i32, f: HFONT, items: &[&str]) -> HWND {
    combo_with(parent, id, f, items, false)
}

/// A drop-down list; `editable`, words can be typed in too - WinForms' DropDownStyle = DropDown.
pub fn combo_with(parent: HWND, id: i32, f: HFONT, items: &[&str], editable: bool) -> HWND {
    const CBS_DROPDOWN: u32 = 0x0002;
    const CBS_DROPDOWNLIST: u32 = 0x0003;
    const CBS_AUTOHSCROLL: u32 = 0x0040;
    const CB_ADDSTRING: u32 = 0x0143;
    const CB_SETCURSEL: u32 = 0x014E;
    let style = if editable { CBS_DROPDOWN | CBS_AUTOHSCROLL } else { CBS_DROPDOWNLIST };
    let h = child(parent, "COMBOBOX", "", WINDOW_STYLE(style) | wm::WS_TABSTOP | wm::WS_VSCROLL, WINDOW_EX_STYLE(0), id, f);
    mark(h, Kind::Combo);
    unsafe {
        for it in items {
            let w = wide(it);
            wm::SendMessageW(h, CB_ADDSTRING, None, Some(LPARAM(w.as_ptr() as isize)));
        }
        wm::SendMessageW(h, CB_SETCURSEL, Some(WPARAM(0)), None);
    }
    theme_native(h);
    h
}

/// A drop-down's box as tall as `height` pixels, borders and all - never shorter than its words
/// need, as Windows first made it. (Windows sizes the box by its font; the window's own height
/// is the room for its list.)
pub fn combo_box_height(h: HWND, height: i32) {
    const CB_GETITEMHEIGHT: u32 = 0x0154;
    const CB_SETITEMHEIGHT: u32 = 0x0153;
    unsafe {
        let key = windows::core::w!("PhaiComboNatural");
        let mut natural = wm::GetPropW(h, key).0 as i32;
        if natural == 0 {
            natural = wm::SendMessageW(h, CB_GETITEMHEIGHT, Some(WPARAM(usize::MAX)), None).0 as i32;
            let _ = wm::SetPropW(h, key, Some(windows::Win32::Foundation::HANDLE(natural as isize as *mut _)));
        }
        // The box's border and the room around its words: what the box is beyond its field.
        let mut r = RECT::default();
        let _ = wm::GetWindowRect(h, &mut r);
        let field = wm::SendMessageW(h, CB_GETITEMHEIGHT, Some(WPARAM(usize::MAX)), None).0 as i32;
        // The box's window is its box (the list is a window of its own): beyond the field, its frame.
        let frame = ((r.bottom - r.top) - field).clamp(2, 16);
        let want = (height - frame).max(natural);
        if want != field {
            wm::SendMessageW(h, CB_SETITEMHEIGHT, Some(WPARAM(usize::MAX)), Some(LPARAM(want as isize)));
        }
        // Taller than Windows makes it: its words drawn again in the middle of the box (Windows
        // leaves them at the top). A box that is typed in keeps its own.
        if want > natural && wm::GetWindowLongW(h, wm::GWL_STYLE) as u32 & 0x3 == 0x3 {
            let _ = windows::Win32::UI::Shell::SetWindowSubclass(h, Some(tall_combo_proc), 12, 0);
        }
    }
}

unsafe extern "system" fn tall_combo_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _id: usize, _data: usize) -> LRESULT {
    use windows::Win32::UI::Shell::DefSubclassProc;
    let r = unsafe { DefSubclassProc(h, msg, w, l) };
    if msg == wm::WM_PAINT || msg == wm::WM_ENABLE || msg == wm::WM_SETFOCUS || msg == wm::WM_KILLFOCUS {
        unsafe {
            let mut cr = RECT::default();
            let _ = wm::GetClientRect(h, &mut cr);
            let arrow = wm::GetSystemMetrics(wm::SM_CXVSCROLL) + 2;
            let inner = RECT { left: cr.left + 2, top: cr.top + 2, right: cr.right - arrow, bottom: cr.bottom - 2 };
            let dc = Gdi::GetDC(Some(h));
            let s = style_of(h, Kind::Combo, State { focus: windows::Win32::UI::Input::KeyboardAndMouse::GetFocus() == h, ..Default::default() });
            let bg = s.background_or(window_background_for(h));
            let fg = if windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled(h).as_bool() { s.color_or(window_text_for(h)) } else { Color::rgb(150, 150, 150) };
            fill(dc, &inner, bg);
            let font = Gdi::HGDIOBJ(wm::SendMessageW(h, wm::WM_GETFONT, None, None).0 as *mut _);
            let old = Gdi::SelectObject(dc, font);
            Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
            Gdi::SetTextColor(dc, windows::Win32::Foundation::COLORREF(fg.colorref()));
            let mut words: Vec<u16> = text_of(h).encode_utf16().collect();
            let mut tr = RECT { left: inner.left + 2, ..inner };
            Gdi::DrawTextW(dc, &mut words, &mut tr, Gdi::DT_LEFT | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE | Gdi::DT_NOPREFIX | Gdi::DT_END_ELLIPSIS);
            Gdi::SelectObject(dc, old);
            Gdi::ReleaseDC(Some(h), dc);
        }
    }
    r
}

pub fn combo_selected(h: impl Into<HWND>) -> i32 {
    let h: HWND = h.into();
    unsafe { wm::SendMessageW(h, 0x0147 /* CB_GETCURSEL */, None, None).0 as i32 }
}

/// The colours for `WM_CTLCOLOR*`: labels, edits, the list of an open combo.
pub fn ctl_color(msg: u32, w: WPARAM, l: LPARAM) -> Option<LRESULT> {
    let dc = HDC(w.0 as *mut _);
    let h = HWND(l.0 as *mut _);
    let kind = match msg {
        wm::WM_CTLCOLORSTATIC => kind_of(h).unwrap_or(Kind::Label),
        wm::WM_CTLCOLOREDIT => kind_of(h).unwrap_or(Kind::Edit),
        // A list box of the library's is a list; an open combo's list is the combo's.
        wm::WM_CTLCOLORLISTBOX => kind_of(h).unwrap_or(Kind::Combo),
        wm::WM_CTLCOLORBTN => return Some(LRESULT(brush(window_background_for(h)).0 as isize)),
        _ => return None,
    };
    let disabled = !unsafe { windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled(h) }.as_bool();
    let s = style_of(h, kind, State { disabled, ..Default::default() });
    // A read-only or disabled edit comes as STATIC; it is still an edit.
    // A label stands on the window, unless it was given a background of its own.
    let own_bg = with_css(h, |c| c.own.background).flatten();
    // A label on a window with a gradient or a picture behind it: the window shows through.
    if kind == Kind::Label && own_bg.is_none() && window_patterned(h) {
        unsafe {
            let mut lr = RECT::default();
            let _ = wm::GetClientRect(h, &mut lr);
            paint_behind(h, dc, &lr);
            Gdi::SetTextColor(dc, cr(s.color_or(window_text_for(h))));
            Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
            return Some(LRESULT(Gdi::GetStockObject(Gdi::NULL_BRUSH).0 as isize));
        }
    }
    let bg = match kind {
        Kind::Label => own_bg.unwrap_or(window_background_for(h)),
        _ => s.background_or(window_background_for(h)),
    };
    unsafe {
        Gdi::SetTextColor(dc, cr(s.color_or(window_text_for(h))));
        Gdi::SetBkColor(dc, cr(bg));
        Gdi::SetBkMode(dc, Gdi::OPAQUE);
    }
    Some(LRESULT(brush(bg).0 as isize))
}

// --- lists ---------------------------------------------------------------------------------------

/// A details list: `(heading, width in design pixels)` per column, whole rows selected.
/// Lines between a list's cells, as a grid of data is drawn (LVS_EX_GRIDLINES).
pub fn grid_lines(h: HWND) {
    unsafe {
        wm::SendMessageW(h, 0x1036 /* LVM_SETEXTENDEDLISTVIEWSTYLE */, Some(WPARAM(0x1)), Some(LPARAM(0x1)));
    }
}

pub fn list(parent: HWND, id: i32, f: HFONT, columns: &[(&str, i32)]) -> HWND {
    use windows::Win32::UI::Controls as cc;
    const LVS_REPORT: u32 = 0x0001;
    const LVS_SHOWSELALWAYS: u32 = 0x0008;
    const LVS_EX_FULLROWSELECT: isize = 0x20;
    const LVS_EX_DOUBLEBUFFER: isize = 0x10000;
    let h = child(parent, "SysListView32", "", WINDOW_STYLE(LVS_REPORT | LVS_SHOWSELALWAYS) | wm::WS_TABSTOP, WINDOW_EX_STYLE(0), id, f);
    mark(h, Kind::List);
    unsafe {
        let _ = windows::Win32::UI::Shell::SetWindowSubclass(h, Some(list_frame_proc), 9, 0);
        wm::SendMessageW(h, cc::LVM_SETEXTENDEDLISTVIEWSTYLE, Some(WPARAM(0)), Some(LPARAM(LVS_EX_FULLROWSELECT | LVS_EX_DOUBLEBUFFER)));
        for (i, (heading, width)) in columns.iter().enumerate() {
            let mut text = wide(heading);
            let col = cc::LVCOLUMNW { mask: cc::LVCF_TEXT | cc::LVCF_WIDTH, cx: px(parent, *width), pszText: windows::core::PWSTR(text.as_mut_ptr()), ..Default::default() };
            wm::SendMessageW(h, cc::LVM_INSERTCOLUMNW, Some(WPARAM(i)), Some(LPARAM(&col as *const _ as isize)));
        }
    }
    theme_native(h);
    h
}

/// Adds a row of `cells` at the end.
/// Gives a list new columns - a query's result has its own - and no rows.
pub fn list_columns(h: HWND, columns: &[(&str, i32)]) {
    use windows::Win32::UI::Controls as cc;
    unsafe {
        wm::SendMessageW(h, cc::LVM_DELETEALLITEMS, None, None);
        while wm::SendMessageW(h, cc::LVM_DELETECOLUMN, Some(WPARAM(0)), None).0 != 0 {}
        for (i, (heading, width)) in columns.iter().enumerate() {
            let mut text = wide(heading);
            let col = cc::LVCOLUMNW { mask: cc::LVCF_TEXT | cc::LVCF_WIDTH, cx: px(h, *width), pszText: windows::core::PWSTR(text.as_mut_ptr()), ..Default::default() };
            wm::SendMessageW(h, cc::LVM_INSERTCOLUMNW, Some(WPARAM(i)), Some(LPARAM(&col as *const _ as isize)));
        }
    }
}

pub fn list_add(h: impl Into<HWND>, cells: &[&str]) {
    let h: HWND = h.into();
    use windows::Win32::UI::Controls as cc;
    unsafe {
        let row = wm::SendMessageW(h, cc::LVM_GETITEMCOUNT, None, None).0 as i32;
        for (i, cell) in cells.iter().enumerate() {
            let mut text = wide(cell);
            let item = cc::LVITEMW { mask: cc::LVIF_TEXT, iItem: row, iSubItem: i as i32, pszText: windows::core::PWSTR(text.as_mut_ptr()), ..Default::default() };
            let m = if i == 0 { cc::LVM_INSERTITEMW } else { cc::LVM_SETITEMW };
            wm::SendMessageW(h, m, None, Some(LPARAM(&item as *const _ as isize)));
        }
    }
}

/// A list's small icons, one per glyph by index, in the list's text colour; rows then show
/// [`list_set_image`]'s - the first, unless set.
pub fn list_icons(h: HWND, glyphs: &[u16]) {
    use windows::Win32::UI::Controls as cc;
    unsafe {
        let size = px(h, 16);
        let colour = style_of(h, Kind::List, State::default()).icon_color.unwrap_or(window_text_for(h));
        let list = cc::ImageList_Create(size, size, cc::ILC_COLOR32 | cc::ILC_MASK, glyphs.len() as i32, 0);
        for g in glyphs {
            if let Some(i) = crate::icon::glyph_icon(*g, size, colour) {
                cc::ImageList_ReplaceIcon(list, -1, i);
                let _ = wm::DestroyIcon(i);
            }
        }
        let old = wm::SendMessageW(h, 0x1003 /* LVM_SETIMAGELIST */, Some(WPARAM(1 /* LVSIL_SMALL */)), Some(LPARAM(list.0 as isize))).0;
        if old != 0 {
            let _ = cc::ImageList_Destroy(Some(cc::HIMAGELIST(old as _)));
        }
    }
}

/// The icon a list's row shows: an index of [`list_icons`]'s, or -2 for none.
pub fn list_set_image(h: HWND, row: usize, image: i32) {
    use windows::Win32::UI::Controls as cc;
    let item = cc::LVITEMW { mask: cc::LVIF_IMAGE, iItem: row as i32, iImage: image, ..Default::default() };
    unsafe {
        wm::SendMessageW(h, cc::LVM_SETITEMW, None, Some(LPARAM(&item as *const _ as isize)));
    }
}

/// Selects one row of a list view - the others not - and scrolls it into view.
/// The words in a list's cell: row and column from 0.
pub fn list_text(h: impl Into<HWND>, row: usize, col: i32) -> String {
    let h: HWND = h.into();
    use windows::Win32::UI::Controls as cc;
    let mut buf = vec![0u16; 512];
    let mut item = cc::LVITEMW { iSubItem: col, pszText: windows::core::PWSTR(buf.as_mut_ptr()), cchTextMax: 512, ..Default::default() };
    let n = unsafe { wm::SendMessageW(h, 0x1073 /* LVM_GETITEMTEXTW */, Some(WPARAM(row)), Some(LPARAM(&mut item as *mut _ as isize))).0 } as usize;
    String::from_utf16_lossy(&buf[..n.min(512)])
}

pub fn list_select(h: impl Into<HWND>, row: usize) {
    let h: HWND = h.into();
    use windows::Win32::UI::Controls as cc;
    unsafe {
        let all = cc::LVITEMW { stateMask: cc::LIST_VIEW_ITEM_STATE_FLAGS(cc::LVIS_SELECTED.0 | cc::LVIS_FOCUSED.0), ..Default::default() };
        wm::SendMessageW(h, cc::LVM_SETITEMSTATE, Some(WPARAM(usize::MAX)), Some(LPARAM(&all as *const _ as isize)));
        let one = cc::LVITEMW { stateMask: cc::LIST_VIEW_ITEM_STATE_FLAGS(cc::LVIS_SELECTED.0 | cc::LVIS_FOCUSED.0), state: cc::LIST_VIEW_ITEM_STATE_FLAGS(cc::LVIS_SELECTED.0 | cc::LVIS_FOCUSED.0), ..Default::default() };
        wm::SendMessageW(h, cc::LVM_SETITEMSTATE, Some(WPARAM(row)), Some(LPARAM(&one as *const _ as isize)));
        wm::SendMessageW(h, cc::LVM_ENSUREVISIBLE, Some(WPARAM(row)), Some(LPARAM(0)));
    }
}

/// `NM_CUSTOMDRAW` from a list view: the selected row in the theme's `list:selected` colours
/// instead of Windows' blue. A grid's check-box columns are drawn here too ([`crate::grid`]).
pub fn list_custom_draw(l: LPARAM) -> LRESULT {
    use windows::Win32::UI::Controls as cc;
    // A grid with properties of its own draws its cells itself.
    if let Some(r) = crate::grid::custom_draw(l) {
        return r;
    }
    unsafe {
        let cd = &mut *(l.0 as *mut cc::NMLVCUSTOMDRAW);
        let h = cd.nmcd.hdr.hwndFrom;
        let selected = |cd: &mut cc::NMLVCUSTOMDRAW| {
            let state = wm::SendMessageW(h, cc::LVM_GETITEMSTATE, Some(WPARAM(cd.nmcd.dwItemSpec)), Some(LPARAM(cc::LVIS_SELECTED.0 as isize)));
            if state.0 as u32 & cc::LVIS_SELECTED.0 != 0 {
                let s = style_of(h, Kind::List, State { selected: true, ..Default::default() });
                cd.nmcd.uItemState = cc::NMCUSTOMDRAW_DRAW_STATE_FLAGS(cd.nmcd.uItemState.0 & !(cc::CDIS_SELECTED.0 | cc::CDIS_FOCUS.0));
                cd.clrTextBk = cr(s.background_or(Color::rgb(231, 243, 239)));
                cd.clrText = cr(s.color_or(window_text_for(h)));
                true
            } else {
                false
            }
        };
        match cd.nmcd.dwDrawStage {
            cc::CDDS_PREPAINT => LRESULT(cc::CDRF_NOTIFYITEMDRAW as isize),
            cc::CDDS_ITEMPREPAINT => {
                selected(cd);
                // A grid with check-box columns: each cell asked about.
                if crate::grid::has_check_columns(h) {
                    return LRESULT(cc::CDRF_NOTIFYSUBITEMDRAW as isize);
                }
                LRESULT(cc::CDRF_DODEFAULT as isize)
            }
            s if s.0 == cc::CDDS_ITEMPREPAINT.0 | cc::CDDS_SUBITEM.0 => {
                let sel = selected(cd);
                let bg = if sel { style_of(h, Kind::List, State { selected: true, ..Default::default() }).background_or(Color::rgb(231, 243, 239)) } else { window_background_for(h) };
                if crate::grid::draw_check_cell(h, cd.nmcd.hdc, cd.nmcd.dwItemSpec, cd.iSubItem as usize, bg, window_text_for(h)) {
                    return LRESULT(cc::CDRF_SKIPDEFAULT as isize);
                }
                LRESULT(cc::CDRF_DODEFAULT as isize)
            }
            _ => LRESULT(cc::CDRF_DODEFAULT as isize),
        }
    }
}

/// A list's chosen row's colours: the theme's `list:selected` background and words.
pub fn list_selection_colors(h: HWND) -> (Color, Color) {
    let s = style_of(h, Kind::List, State { selected: true, ..Default::default() });
    (s.background_or(Color::rgb(231, 243, 239)), s.color_or(window_text_for(h)))
}

/// A list's own colours - its CSS's, else the window's.
pub fn list_colors(h: HWND) -> (Color, Color) {
    let s = style_of(h, Kind::List, State::default());
    (s.background_or(window_background_for(h)), s.color_or(window_text_for(h)))
}

/// A list's border colour, for lines drawn to match it.
pub fn list_border_color(h: HWND) -> Color {
    list_border(h).1
}

static FRAMED: Mutex<Vec<isize>> = Mutex::new(Vec::new());

/// A list framed whatever its style says - a grid, as WinForms' DataGridView has its single
/// border. Its colour and width are its CSS's `border-color` and `border-width`.
pub fn list_framed(h: HWND) {
    let mut v = FRAMED.lock().unwrap_or_else(|e| e.into_inner());
    if !v.contains(&(h.0 as isize)) {
        v.push(h.0 as isize);
    }
    drop(v);
    frame_changed(h);
}

fn frame_changed(h: HWND) {
    unsafe {
        let _ = wm::SetWindowPos(h, None, 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOZORDER | wm::SWP_NOACTIVATE | wm::SWP_FRAMECHANGED);
    }
}

/// A list's border: its width in pixels (0: none) and colour - its own `border-width` and
/// `border-color` when it has them; a framed list's 1 pixel otherwise.
fn list_border(h: HWND) -> (i32, Color) {
    let own = with_css(h, |c| (c.own.border_width, c.own.border_color)).unwrap_or((None, None));
    let framed = FRAMED.lock().unwrap_or_else(|e| e.into_inner()).contains(&(h.0 as isize));
    let width = match own {
        (Some(w), _) => w,
        (None, Some(_)) => 1,
        _ if framed => 1,
        _ => 0,
    };
    let colour = own.1.unwrap_or_else(|| style_of(h, Kind::List, State::default()).border_or(Color::rgb(212, 212, 212)));
    (px(h, width.max(0)), colour)
}

unsafe extern "system" fn list_frame_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _id: usize, _data: usize) -> LRESULT {
    use windows::Win32::UI::Shell::DefSubclassProc;
    match msg {
        wm::WM_NCCALCSIZE if w.0 != 0 => {
            let r = unsafe { DefSubclassProc(h, msg, w, l) };
            let (bw, _) = list_border(h);
            if bw > 0 {
                let p = unsafe { &mut *(l.0 as *mut wm::NCCALCSIZE_PARAMS) };
                p.rgrc[0].left += bw;
                p.rgrc[0].top += bw;
                p.rgrc[0].right -= bw;
                p.rgrc[0].bottom -= bw;
            }
            r
        }
        wm::WM_NCPAINT => {
            let r = unsafe { DefSubclassProc(h, msg, w, l) };
            let (bw, colour) = list_border(h);
            if bw > 0 {
                unsafe {
                    let dc = Gdi::GetWindowDC(Some(h));
                    let mut wr = RECT::default();
                    let _ = wm::GetWindowRect(h, &mut wr);
                    let (ww, wh) = (wr.right - wr.left, wr.bottom - wr.top);
                    for edge in [RECT { left: 0, top: 0, right: ww, bottom: bw }, RECT { left: 0, top: wh - bw, right: ww, bottom: wh }, RECT { left: 0, top: 0, right: bw, bottom: wh }, RECT { left: ww - bw, top: 0, right: ww, bottom: wh }] {
                        fill(dc, &edge, colour);
                    }
                    Gdi::ReleaseDC(Some(h), dc);
                }
            }
            r
        }
        wm::WM_NCDESTROY => {
            FRAMED.lock().unwrap_or_else(|e| e.into_inner()).retain(|x| *x != h.0 as isize);
            unsafe { DefSubclassProc(h, msg, w, l) }
        }
        _ => unsafe { DefSubclassProc(h, msg, w, l) },
    }
}

// --- trees ----------------------------------------------------------------------------------------

/// A tree, as Explorer's folders: lines off, buttons to open and close, whole rows chosen. Its
/// icons are glyphs of Windows' icon font in the theme's `tree { icon-color }` ([`tree_icons`]).
pub fn tree(parent: HWND, id: i32, f: HFONT) -> HWND {
    const TVS_HASBUTTONS: u32 = 0x0001;
    const TVS_LINESATROOT: u32 = 0x0004;
    const TVS_SHOWSELALWAYS: u32 = 0x0020;
    const TVS_TRACKSELECT: u32 = 0x0200;
    const TVS_FULLROWSELECT: u32 = 0x1000;
    const TVM_SETEXTENDEDSTYLE: u32 = 0x112C;
    const TVS_EX_DOUBLEBUFFER: isize = 0x0004;
    let style = TVS_HASBUTTONS | TVS_LINESATROOT | TVS_SHOWSELALWAYS | TVS_TRACKSELECT | TVS_FULLROWSELECT;
    let h = child(parent, "SysTreeView32", "", WINDOW_STYLE(style) | wm::WS_TABSTOP, WINDOW_EX_STYLE(0), id, f);
    mark(h, Kind::Tree);
    unsafe {
        wm::SendMessageW(h, TVM_SETEXTENDEDSTYLE, Some(WPARAM(TVS_EX_DOUBLEBUFFER as usize)), Some(LPARAM(TVS_EX_DOUBLEBUFFER)));
    }
    theme_native(h);
    h
}

type TreeGlyphs = Vec<u16>;
static TREE_ICONS: Mutex<Option<HashMap<isize, TreeGlyphs>>> = Mutex::new(None);

/// The tree's icons, one per glyph, by index - drawn again when the theme changes.
pub fn tree_icons(h: HWND, glyphs: &[u16]) {
    use windows::Win32::UI::Controls as cc;
    const TVM_SETIMAGELIST: u32 = 0x1109;
    TREE_ICONS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).insert(h.0 as isize, glyphs.to_vec());
    unsafe {
        let size = px(h, 16);
        let colour = style_of(h, Kind::Tree, State::default()).icon_color.unwrap_or(window_text_for(h));
        let list = cc::ImageList_Create(size, size, cc::ILC_COLOR32 | cc::ILC_MASK, glyphs.len() as i32, 0);
        for g in glyphs {
            if let Some(i) = crate::icon::glyph_icon(*g, size, colour) {
                cc::ImageList_ReplaceIcon(list, -1, i);
                let _ = wm::DestroyIcon(i);
            }
        }
        let old = wm::SendMessageW(h, TVM_SETIMAGELIST, Some(WPARAM(0)), Some(LPARAM(list.0 as isize))).0;
        if old != 0 {
            let _ = cc::ImageList_Destroy(Some(cc::HIMAGELIST(old as _)));
        }
    }
}

/// Adds an item under `parent` (0 for the top), with icon `image` and a number kept with it.
/// Returns the item, to add others under.
pub fn tree_add(h: HWND, parent: isize, text: &str, image: i32, data: isize) -> isize {
    use windows::Win32::UI::Controls as cc;
    const TVM_INSERTITEMW: u32 = 0x1132;
    let mut w = wide(text);
    let item = cc::TVITEMW {
        mask: cc::TVIF_TEXT | cc::TVIF_IMAGE | cc::TVIF_SELECTEDIMAGE | cc::TVIF_PARAM,
        pszText: windows::core::PWSTR(w.as_mut_ptr()),
        iImage: image,
        iSelectedImage: image,
        lParam: LPARAM(data),
        ..Default::default()
    };
    let ins = cc::TVINSERTSTRUCTW {
        hParent: cc::HTREEITEM(parent),
        hInsertAfter: cc::HTREEITEM(-0xFFFE), // TVI_LAST
        Anonymous: cc::TVINSERTSTRUCTW_0 { item },
    };
    unsafe { wm::SendMessageW(h, TVM_INSERTITEMW, None, Some(LPARAM(&ins as *const _ as isize))).0 }
}

/// Where a child window is in its parent's client area.
pub fn rect_in_parent(h: impl Into<HWND>) -> RECT {
    let h: HWND = h.into();
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetWindowRect(h, &mut r);
        if let Ok(p) = wm::GetParent(h) {
            let mut pts = [windows::Win32::Foundation::POINT { x: r.left, y: r.top }, windows::Win32::Foundation::POINT { x: r.right, y: r.bottom }];
            Gdi::MapWindowPoints(None, Some(p), &mut pts);
            r = RECT { left: pts[0].x, top: pts[0].y, right: pts[1].x, bottom: pts[1].y };
        }
    }
    r
}

/// A panel that holds others laid out by it (flow, table): the window's colour; `outline` draws
/// a faint dashed edge, as a designer shows an empty panel.
pub fn panel(parent: HWND, id: i32, outline: bool) -> HWND {
    let class = register(if outline { "PhaiPanelOutlined" } else { "PhaiPanel" }, Some(if outline { panel_outlined_proc } else { panel_proc }), None);
    unsafe {
        let h = wm::CreateWindowExW(Default::default(), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPSIBLINGS, 0, 0, 10, 10, Some(parent), Some(wm::HMENU(id as isize as *mut _)), Some(instance()), None).unwrap_or_default();
        // Under the controls it holds, which are made after it and would lie below - and kept
        // there by the layout, as it keeps a group frame.
        let _ = wm::SetWindowPos(h, Some(wm::HWND_BOTTOM), 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOACTIVATE);
        register_kind(h, Kind::Group);
        h
    }
}

fn panel_paint(h: HWND, outline: bool) {
    unsafe {
        let mut ps = Gdi::PAINTSTRUCT::default();
        let dc = Gdi::BeginPaint(h, &mut ps);
        let mut r = RECT::default();
        let _ = wm::GetClientRect(h, &mut r);
        // A panel with a colour of its own (its BackColor) shows it; else what is behind it.
        match with_css(h, |c| c.own.background).flatten() {
            Some(c) => fill(dc, &r, c),
            None => paint_behind(h, dc, &r),
        }
        if outline {
            let line = crate::theme::current().var_color("--border").unwrap_or(Color::rgb(0xb0, 0xb0, 0xb0));
            let mut x = r.left;
            while x < r.right {
                fill(dc, &RECT { left: x, top: r.top, right: (x + 4).min(r.right), bottom: r.top + 1 }, line);
                fill(dc, &RECT { left: x, top: r.bottom - 1, right: (x + 4).min(r.right), bottom: r.bottom }, line);
                x += 8;
            }
            let mut y = r.top;
            while y < r.bottom {
                fill(dc, &RECT { left: r.left, top: y, right: r.left + 1, bottom: (y + 4).min(r.bottom) }, line);
                fill(dc, &RECT { left: r.right - 1, top: y, right: r.right, bottom: (y + 4).min(r.bottom) }, line);
                y += 8;
            }
        }
        let _ = Gdi::EndPaint(h, &ps);
    }
}

extern "system" fn panel_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_PAINT => {
            panel_paint(h, false);
            LRESULT(0)
        }
        _ => def(h, msg, w, l),
    }
}

extern "system" fn panel_outlined_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_PAINT => {
            panel_paint(h, true);
            LRESULT(0)
        }
        _ => def(h, msg, w, l),
    }
}

/// Fills a tree from lines: two spaces (or a tab) before a line put it under the line above with
/// one less. The items' numbers are their line numbers.
pub fn tree_items(h: impl Into<HWND>, lines: &[&str]) {
    let h: HWND = h.into();
    tree_clear(h);
    let mut parents: Vec<isize> = Vec::new();
    for (n, line) in lines.iter().enumerate() {
        let raw = line.replace('\t', "  ");
        let depth = (raw.len() - raw.trim_start().len()) / 2;
        parents.truncate(depth);
        let under = parents.last().copied().unwrap_or(0);
        let item = tree_add(h, under, raw.trim(), 0, n as isize);
        parents.push(item);
        if under != 0 {
            tree_expand(h, under);
        }
    }
}

/// A timer on window `h`: every `ms` milliseconds its window hears `WM_TIMER` with `id`, from now
/// when `on`. [`timer_start`] and [`timer_stop`] turn it on and off later.
pub fn timer(h: HWND, id: i32, ms: u32, on: bool) {
    if on {
        timer_start(h, id, ms);
    }
}

pub fn timer_start(h: HWND, id: i32, ms: u32) {
    unsafe {
        wm::SetTimer(Some(h), id as usize, ms.max(1), None);
    }
}

pub fn timer_stop(h: HWND, id: i32) {
    unsafe {
        let _ = wm::KillTimer(Some(h), id as usize);
    }
}

/// Empties the tree.
pub fn tree_clear(h: impl Into<HWND>) {
    let h: HWND = h.into();
    unsafe {
        wm::SendMessageW(h, 0x1101 /* TVM_DELETEITEM */, None, Some(LPARAM(-0x10000 /* TVI_ROOT */)));
    }
}

/// Opens an item to show what is under it.
pub fn tree_expand(h: HWND, item: isize) {
    unsafe {
        wm::SendMessageW(h, 0x1102 /* TVM_EXPAND */, Some(WPARAM(2 /* TVE_EXPAND */)), Some(LPARAM(item)));
    }
}

/// The item kept with number `data` chosen and scrolled into view, its folders opened; whether
/// there was one.
pub fn tree_select_data(h: HWND, data: isize) -> bool {
    use windows::Win32::UI::Controls as cc;
    unsafe fn find(h: HWND, mut item: isize, data: isize) -> isize {
        while item != 0 {
            let mut it = cc::TVITEMW { mask: cc::TVIF_PARAM, hItem: cc::HTREEITEM(item), ..Default::default() };
            wm::SendMessageW(h, 0x113E /* TVM_GETITEMW */, None, Some(LPARAM(&mut it as *mut _ as isize)));
            if it.lParam.0 == data {
                return item;
            }
            let child = wm::SendMessageW(h, 0x110A, Some(WPARAM(4 /* TVGN_CHILD */)), Some(LPARAM(item))).0;
            let deeper = find(h, child, data);
            if deeper != 0 {
                return deeper;
            }
            item = wm::SendMessageW(h, 0x110A, Some(WPARAM(1 /* TVGN_NEXT */)), Some(LPARAM(item))).0;
        }
        0
    }
    unsafe {
        let root = wm::SendMessageW(h, 0x110A, Some(WPARAM(0 /* TVGN_ROOT */)), None).0;
        let item = find(h, root, data);
        if item == 0 {
            return false;
        }
        wm::SendMessageW(h, 0x110B /* TVM_SELECTITEM */, Some(WPARAM(9)), Some(LPARAM(item)));
        wm::SendMessageW(h, 0x1114 /* TVM_ENSUREVISIBLE */, None, Some(LPARAM(item)));
        true
    }
}

/// The number kept with the chosen item, if one is chosen.
pub fn tree_selected(h: impl Into<HWND>) -> Option<isize> {
    let h: HWND = h.into();
    use windows::Win32::UI::Controls as cc;
    unsafe {
        let item = wm::SendMessageW(h, 0x110A /* TVM_GETNEXTITEM */, Some(WPARAM(9 /* TVGN_CARET */)), None).0;
        if item == 0 {
            return None;
        }
        let mut it = cc::TVITEMW { mask: cc::TVIF_PARAM, hItem: cc::HTREEITEM(item), ..Default::default() };
        wm::SendMessageW(h, 0x113E /* TVM_GETITEMW */, None, Some(LPARAM(&mut it as *mut _ as isize)));
        Some(it.lParam.0)
    }
}

/// `NM_CUSTOMDRAW` from a tree: the chosen row in the theme's `tree:selected` colours.
pub fn tree_custom_draw(l: LPARAM) -> LRESULT {
    use windows::Win32::UI::Controls as cc;
    unsafe {
        let cd = &mut *(l.0 as *mut cc::NMTVCUSTOMDRAW);
        match cd.nmcd.dwDrawStage {
            cc::CDDS_PREPAINT => LRESULT(cc::CDRF_NOTIFYITEMDRAW as isize),
            cc::CDDS_ITEMPREPAINT => {
                let selected = cd.nmcd.uItemState.0 & cc::CDIS_SELECTED.0 != 0;
                let hot = cd.nmcd.uItemState.0 & cc::CDIS_HOT.0 != 0;
                if selected || hot {
                    let s = style_of(cd.nmcd.hdr.hwndFrom, Kind::Tree, State { selected, hover: hot && !selected, ..Default::default() });
                    cd.nmcd.uItemState = cc::NMCUSTOMDRAW_DRAW_STATE_FLAGS(cd.nmcd.uItemState.0 & !(cc::CDIS_SELECTED.0 | cc::CDIS_FOCUS.0 | cc::CDIS_HOT.0));
                    cd.clrTextBk = cr(s.background_or(window_background_for(cd.nmcd.hdr.hwndFrom)));
                    cd.clrText = cr(s.color_or(window_text_for(cd.nmcd.hdr.hwndFrom)));
                }
                LRESULT(cc::CDRF_DODEFAULT as isize)
            }
            _ => LRESULT(cc::CDRF_DODEFAULT as isize),
        }
    }
}

/// Puts a control Windows draws - list, combo - in the theme: its colours, and Windows' own dark
/// look for scroll bars and headers when the theme is dark.
pub fn theme_native(h: HWND) {
    use windows::Win32::UI::Controls as cc;
    let dark = theme::current().is_dark();
    unsafe {
        match kind_of(h) {
            Some(Kind::List) => {
                let s = style_of(h, Kind::List, State::default());
                let bg = s.background_or(window_background_for(h));
                wm::SendMessageW(h, cc::LVM_SETBKCOLOR, None, Some(LPARAM(bg.colorref() as isize)));
                wm::SendMessageW(h, cc::LVM_SETTEXTBKCOLOR, None, Some(LPARAM(bg.colorref() as isize)));
                wm::SendMessageW(h, cc::LVM_SETTEXTCOLOR, None, Some(LPARAM(s.color_or(window_text_for(h)).colorref() as isize)));
                // The body keeps the plain list look: the Explorer look draws its own selection and
                // the theme's `list:selected` would never show.
                let header = HWND(wm::SendMessageW(h, cc::LVM_GETHEADER, None, None).0 as *mut _);
                if !header.is_invalid() {
                    let _ = cc::SetWindowTheme(header, if dark { windows::core::w!("DarkMode_ItemsView") } else { windows::core::w!("ItemsView") }, PCWSTR::null());
                }
            }
            Some(Kind::Tree) => {
                let s = style_of(h, Kind::Tree, State::default());
                wm::SendMessageW(h, 0x111D /* TVM_SETBKCOLOR */, None, Some(LPARAM(s.background_or(window_background_for(h)).colorref() as isize)));
                wm::SendMessageW(h, 0x111E /* TVM_SETTEXTCOLOR */, None, Some(LPARAM(s.color_or(window_text_for(h)).colorref() as isize)));
                // Explorer's arrows instead of + and - boxes; dark ones in a dark theme.
                let _ = cc::SetWindowTheme(h, if dark { windows::core::w!("DarkMode_Explorer") } else { windows::core::w!("Explorer") }, PCWSTR::null());
                let glyphs = TREE_ICONS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|m| m.get(&(h.0 as isize)).cloned());
                if let Some(g) = glyphs {
                    tree_icons(h, &g);
                }
            }
            Some(Kind::Label) | Some(Kind::Edit) => {
                let kind = kind_of(h).unwrap_or(Kind::Label);
                if let Some(f) = css_font(h, &style_of(h, kind, State::default())) {
                    set_font(h, f);
                }
                // A box of several lines scrolls; its bar follows a dark theme.
                let _ = cc::SetWindowTheme(h, if dark { windows::core::w!("DarkMode_Explorer") } else { windows::core::w!("Explorer") }, PCWSTR::null());
            }
            Some(Kind::Combo) => {
                let _ = cc::SetWindowTheme(h, if dark { windows::core::w!("DarkMode_CFD") } else { PCWSTR::null() }, PCWSTR::null());
            }
            Some(Kind::Date) => {
                if let Some(f) = css_font(h, &style_of(h, Kind::Date, State::default())) {
                    set_font(h, f);
                }
                // The calendar that drops down: the theme's colours, its chosen day in the accent.
                let s = style_of(h, Kind::Date, State::default());
                let bg = s.background_or(window_background_for(h));
                let set = |part: usize, c: Color| {
                    wm::SendMessageW(h, 0x1006 /* DTM_SETMCCOLOR */, Some(WPARAM(part)), Some(LPARAM(c.colorref() as isize)));
                };
                set(0 /* MCSC_BACKGROUND */, bg);
                set(4 /* MCSC_MONTHBK */, bg);
                set(1 /* MCSC_TEXT */, s.color_or(window_text_for(h)));
                set(2 /* MCSC_TITLEBK */, s.accent_or(Color::rgb(0x12, 0x85, 0x5f)));
                set(3 /* MCSC_TITLETEXT */, Color::rgb(255, 255, 255));
                let _ = cc::SetWindowTheme(h, if dark { windows::core::w!("DarkMode_CFD") } else { PCWSTR::null() }, PCWSTR::null());
            }
            Some(Kind::Calendar) => {
                let s = style_of(h, Kind::Calendar, State::default());
                crate::extra::calendar_colors(h, s.background_or(window_background_for(h)), s.color_or(window_text_for(h)), s.accent_or(Color::rgb(0x12, 0x85, 0x5f)), dark);
            }
            Some(Kind::Progress) | Some(Kind::Slider) => {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
            Some(_) => {}
            None => {
                // A window of a program's own that scrolls (a toolbox, an editor): its scroll bars
                // dark in a dark theme, as Windows' own controls have them.
                let style = wm::GetWindowLongW(h, wm::GWL_STYLE) as u32;
                if style & (wm::WS_VSCROLL.0 | wm::WS_HSCROLL.0) != 0 {
                    let _ = cc::SetWindowTheme(h, if dark { windows::core::w!("DarkMode_Explorer") } else { windows::core::w!("Explorer") }, PCWSTR::null());
                    let _ = wm::SetWindowPos(h, None, 0, 0, 0, 0, wm::SWP_NOMOVE | wm::SWP_NOSIZE | wm::SWP_NOZORDER | wm::SWP_NOACTIVATE | wm::SWP_FRAMECHANGED);
                }
            }
        }
    }
}

// --- dates, numbers, progress, sliders -------------------------------------------------------
//
// A date is Windows' own date picker. A number is an edit with Windows' up-down arrows inside it.
// A progress bar and a slider are drawn here, so the theme reaches every pixel of them:
// `background` the track, `accent-color` what is done or chosen.

const DTN_DATETIMECHANGE: u32 = (-759i32) as u32;
/// What a date, number or slider tells its parent when its value changes: `WM_COMMAND` with this
/// code, as an edit's `EN_CHANGE`.
pub const VALUE_CHANGED: u16 = 0x0300;

/// A date, today unless set with [`set_date`]; the parent hears [`VALUE_CHANGED`].
pub fn date(parent: HWND, id: i32, f: HFONT) -> HWND {
    date_with(parent, id, f, false, false)
}

/// A date with WinForms' ShowCheckBox (a tick before it: no date when unticked) and ShowUpDown
/// (arrows in place of the calendar) - styles Windows takes only when it is made.
pub fn date_with(parent: HWND, id: i32, f: HFONT, show_check_box: bool, show_up_down: bool) -> HWND {
    const DTS_SHORTDATECENTURYFORMAT: u32 = 0x000C;
    const DTS_UPDOWN: u32 = 0x0001;
    const DTS_SHOWNONE: u32 = 0x0002;
    let style = DTS_SHORTDATECENTURYFORMAT | if show_up_down { DTS_UPDOWN } else { 0 } | if show_check_box { DTS_SHOWNONE } else { 0 };
    let h = child(parent, "SysDateTimePick32", "", WINDOW_STYLE(style) | wm::WS_TABSTOP, WINDOW_EX_STYLE(0), id, f);
    mark(h, Kind::Date);
    theme_native(h);
    h
}

/// The date shown, as `(year, month, day)`.
pub fn date_value(h: impl Into<HWND>) -> (i32, i32, i32) {
    let h: HWND = h.into();
    let mut st = windows::Win32::Foundation::SYSTEMTIME::default();
    unsafe {
        wm::SendMessageW(h, 0x1001 /* DTM_GETSYSTEMTIME */, None, Some(LPARAM(&mut st as *mut _ as isize)));
    }
    (st.wYear as i32, st.wMonth as i32, st.wDay as i32)
}

/// With ShowCheckBox: whether it is ticked - a date given.
pub fn date_checked(h: impl Into<HWND>) -> bool {
    let h: HWND = h.into();
    let mut st = windows::Win32::Foundation::SYSTEMTIME::default();
    unsafe { wm::SendMessageW(h, 0x1001 /* DTM_GETSYSTEMTIME */, None, Some(LPARAM(&mut st as *mut _ as isize))).0 == 0 }
}

pub fn set_date(h: impl Into<HWND>, year: i32, month: i32, day: i32) {
    let h: HWND = h.into();
    let st = windows::Win32::Foundation::SYSTEMTIME { wYear: year as u16, wMonth: month as u16, wDay: day as u16, ..Default::default() };
    unsafe {
        wm::SendMessageW(h, 0x1002 /* DTM_SETSYSTEMTIME */, Some(WPARAM(0 /* GDT_VALID */)), Some(LPARAM(&st as *const _ as isize)));
    }
}

/// A box for a whole number from `min` to `max`, with arrows; the parent hears `EN_CHANGE`.
pub fn number(parent: HWND, id: i32, f: HFONT, min: i32, max: i32, value: i32) -> HWND {
    const ES_NUMBER: u32 = 0x2000;
    const UDS_SETBUDDYINT: u32 = 0x0002;
    const UDS_ARROWKEYS: u32 = 0x0020;
    const UDS_NOTHOUSANDS: u32 = 0x0080;
    const UDS_HOTTRACK: u32 = 0x0100;
    const EM_SETMARGINS: u32 = 0x00D3;
    let h = edit(parent, &value.to_string(), id, f, false);
    unsafe {
        let style = wm::GetWindowLongW(h, wm::GWL_STYLE) as u32;
        // A negative range needs the minus sign, which ES_NUMBER refuses.
        let numeric = if min >= 0 { ES_NUMBER } else { 0 };
        wm::SetWindowLongW(h, wm::GWL_STYLE, (style | numeric | wm::WS_CLIPCHILDREN.0) as i32);
        // The arrows live inside the box, so they move, hide and ghost with it.
        let ud = child(h, "msctls_updown32", "", WINDOW_STYLE(UDS_SETBUDDYINT | UDS_ARROWKEYS | UDS_NOTHOUSANDS | UDS_HOTTRACK), WINDOW_EX_STYLE(0), 0, HFONT::default());
        wm::SendMessageW(ud, 0x0469 /* UDM_SETBUDDY */, Some(WPARAM(h.0 as usize)), None);
        wm::SendMessageW(ud, 0x046F /* UDM_SETRANGE32 */, Some(WPARAM(min as isize as usize)), Some(LPARAM(max as isize)));
        wm::SendMessageW(ud, 0x0471 /* UDM_SETPOS32 */, None, Some(LPARAM(value as isize)));
        let m = px(parent, 6) as isize;
        let right = px(parent, 22) as isize;
        wm::SendMessageW(h, EM_SETMARGINS, Some(WPARAM(3)), Some(LPARAM((right << 16) | m)));
        let _ = windows::Win32::UI::Shell::SetWindowSubclass(h, Some(number_proc), 3, ud.0 as usize);
        place_arrows(h, ud);
    }
    h
}

fn place_arrows(edit: HWND, ud: HWND) {
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(edit, &mut r);
    }
    let w = px(edit, 18);
    move_to(ud, r.right - w, 0, w, r.bottom);
}

unsafe extern "system" fn number_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM, _id: usize, ud: usize) -> LRESULT {
    let r = unsafe { windows::Win32::UI::Shell::DefSubclassProc(h, msg, w, l) };
    match msg {
        wm::WM_SIZE => place_arrows(h, HWND(ud as *mut _)),
        wm::WM_ENABLE => unsafe {
            let _ = windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow(HWND(ud as *mut _), w.0 != 0);
        },
        wm::WM_NCDESTROY => unsafe {
            let _ = windows::Win32::UI::Shell::RemoveWindowSubclass(h, Some(number_proc), 3);
        },
        _ => {}
    }
    r
}

pub(crate) struct Range {
    pub(crate) min: i32,
    pub(crate) max: i32,
    pub(crate) value: i32,
    drag: bool,
    /// What an arrow key moves a slider by; Page Up and Down, 0 a tenth of the range.
    pub(crate) small: i32,
    pub(crate) large: i32,
    /// A tick under a slider every so many; 0 none.
    pub(crate) ticks: i32,
    /// What a progress bar's perform_step adds.
    pub(crate) step: i32,
    /// A progress bar's running block, and where it is, in hundredths of the width.
    pub(crate) marquee: bool,
    pub(crate) phase: i32,
    /// A slider standing up, its least at the bottom - Orientation = Vertical.
    pub(crate) vertical: bool,
}

/// A slider's or progress bar's settings, changed by `f`, then shown.
pub(crate) fn with_range(h: HWND, f: impl FnOnce(&mut Range)) {
    if let Some(r) = unsafe { state::<Range>(h) } {
        f(r);
    }
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    }
}

fn range_class(name: &str, proc_: wm::WNDPROC) -> Vec<u16> {
    register(name, proc_, None)
}

#[allow(clippy::too_many_arguments)]
fn make_range(parent: HWND, class: &str, proc_: wm::WNDPROC, id: i32, kind: Kind, min: i32, max: i32, value: i32, tab: bool) -> HWND {
    let class = range_class(class, proc_);
    let h = unsafe {
        wm::CreateWindowExW(WINDOW_EX_STYLE(0), PCWSTR(class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_CLIPSIBLINGS | if tab { wm::WS_TABSTOP } else { WINDOW_STYLE(0) }, 0, 0, 10, 10, Some(parent), Some(HMENU(id as isize as *mut _)), Some(instance()), None).unwrap_or_default()
    };
    let (lo, hi) = (min.min(max), max.max(min));
    set_state(h, Box::new(Range { min: lo, max: hi, value: value.clamp(lo, hi), drag: false, small: 1, large: 0, ticks: 0, step: 10, marquee: false, phase: 0, vertical: false }));
    mark(h, kind);
    h
}

/// A bar that fills from `min` to `max` as work is done.
pub fn progress(parent: HWND, id: i32, min: i32, max: i32, value: i32) -> HWND {
    make_range(parent, "PhaiProgress", Some(progress_proc), id, Kind::Progress, min, max, value, false)
}

/// A slider from `min` to `max`, by mouse or arrow keys; the parent hears [`VALUE_CHANGED`].
pub fn slider(parent: HWND, id: i32, min: i32, max: i32, value: i32) -> HWND {
    make_range(parent, "PhaiSlider", Some(slider_proc), id, Kind::Slider, min, max, value, true)
}

/// The value of a number box, slider or progress bar.
pub fn value(h: impl Into<HWND>) -> i32 {
    let h: HWND = h.into();
    match kind_of(h) {
        Some(Kind::Edit) => text_of(h).trim().parse().unwrap_or(0),
        _ => unsafe { state::<Range>(h) }.map(|r| r.value).unwrap_or(0),
    }
}

/// Sets a number box's, slider's or progress bar's value, kept within its range.
pub fn set_value(h: impl Into<HWND>, v: i32) {
    let h: HWND = h.into();
    match kind_of(h) {
        Some(Kind::Edit) => set_text(h, &v.to_string()),
        _ => {
            if let Some(r) = unsafe { state::<Range>(h) } {
                r.value = v.clamp(r.min, r.max);
            }
            unsafe {
                let _ = Gdi::InvalidateRect(Some(h), None, false);
            }
        }
    }
}

fn range_paint(h: HWND, slider: bool) {
    let Some(r) = (unsafe { state::<Range>(h) }) else { return };
    let (min, max, value, ticks, marquee, phase, vertical) = (r.min, r.max, r.value, r.ticks, r.marquee, r.phase, r.vertical);
    unsafe {
        let mut ps = Gdi::PAINTSTRUCT::default();
        let dc = Gdi::BeginPaint(h, &mut ps);
        let mut rc = RECT::default();
        let _ = wm::GetClientRect(h, &mut rc);
        let enabled = windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled(h).as_bool();
        let hot = with_css(h, |c| c.hot).unwrap_or(false);
        let focus = windows::Win32::UI::Input::KeyboardAndMouse::GetFocus() == h;
        let kind = if slider { Kind::Slider } else { Kind::Progress };
        let s = style_animated(h, kind, State { hover: hot, focus, disabled: !enabled, ..Default::default() });
        // Drawn off screen and copied once, so dragging does not flicker.
        let mem = Gdi::CreateCompatibleDC(Some(dc));
        let bmp = Gdi::CreateCompatibleBitmap(dc, rc.right.max(1), rc.bottom.max(1));
        let old = Gdi::SelectObject(mem, bmp.into());
        paint_behind(h, mem, &rc);
        let accent = s.accent_or(theme::current().var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f)));
        let track = s.background_or(Color::rgb(0xe5, 0xe5, 0xe5));
        let span = (max - min).max(1) as i64;
        let frac = |w: i32| ((value - min) as i64 * w as i64 / span) as i32;
        if slider && vertical {
            // Standing up: the least at the bottom, as a vertical TrackBar.
            let thumb = px(h, 18);
            let th = px(h, 4);
            let cx = rc.right / 2;
            let (y0, y1) = (thumb / 2, rc.bottom - thumb / 2);
            let at = y1 - frac(y1 - y0);
            if ticks > 0 {
                let tick = s.border_or(Color::rgb(0x8a, 0x8a, 0x8a));
                let mut v = min;
                while v <= max {
                    let y = y1 - ((v - min) as i64 * (y1 - y0) as i64 / span) as i32;
                    fill(mem, &RECT { left: rc.right - px(h, 5), top: y, right: rc.right - 1, bottom: y + 1 }, tick);
                    v += ticks;
                }
            }
            round_rect(mem, RECT { left: cx - th / 2, top: y0, right: cx - th / 2 + th, bottom: y1 }, track, track, th);
            round_rect(mem, RECT { left: cx - th / 2, top: at.min(y1 - 1), right: cx - th / 2 + th, bottom: y1 }, accent, accent, th);
            let ring = RECT { left: cx - thumb / 2, top: at - thumb / 2, right: cx - thumb / 2 + thumb, bottom: at - thumb / 2 + thumb };
            round_rect(mem, ring, window_background_for(h), s.border_or(Color::rgb(0x8a, 0x8a, 0x8a)), thumb);
            let dot = thumb * if hot || r.drag { 5 } else { 4 } / 10;
            round_rect(mem, RECT { left: cx - dot / 2, top: at - dot / 2, right: cx - dot / 2 + dot, bottom: at - dot / 2 + dot }, accent, accent, dot);
            if focus {
                let _ = Gdi::DrawFocusRect(mem, &rc);
            }
        } else if slider {
            let thumb = px(h, 18);
            let th = px(h, 4);
            let cy = rc.bottom / 2;
            let (x0, x1) = (thumb / 2, rc.right - thumb / 2);
            let at = x0 + frac(x1 - x0);
            if ticks > 0 {
                // Tick marks along the bottom, as a TrackBar's TickFrequency.
                let tick = s.border_or(Color::rgb(0x8a, 0x8a, 0x8a));
                let mut v = min;
                while v <= max {
                    let x = x0 + ((v - min) as i64 * (x1 - x0) as i64 / span) as i32;
                    fill(mem, &RECT { left: x, top: rc.bottom - px(h, 5), right: x + 1, bottom: rc.bottom - 1 }, tick);
                    v += ticks;
                }
            }
            round_rect(mem, RECT { left: x0, top: cy - th / 2, right: x1, bottom: cy - th / 2 + th }, track, track, th);
            round_rect(mem, RECT { left: x0, top: cy - th / 2, right: at.max(x0 + 1), bottom: cy - th / 2 + th }, accent, accent, th);
            // The thumb: a ring in the border colour around a dot in the accent, as Windows 11's.
            let ring = RECT { left: at - thumb / 2, top: cy - thumb / 2, right: at - thumb / 2 + thumb, bottom: cy - thumb / 2 + thumb };
            round_rect(mem, ring, window_background_for(h), s.border_or(Color::rgb(0x8a, 0x8a, 0x8a)), thumb);
            let dot = thumb * if hot || r.drag { 5 } else { 4 } / 10;
            round_rect(mem, RECT { left: at - dot / 2, top: cy - dot / 2, right: at - dot / 2 + dot, bottom: cy - dot / 2 + dot }, accent, accent, dot);
            if focus {
                let fr = RECT { left: 0, top: 0, right: rc.right, bottom: rc.bottom };
                let _ = Gdi::DrawFocusRect(mem, &fr);
            }
        } else {
            let radius = px(h, s.border_radius.unwrap_or(3) * 2);
            round_rect(mem, rc, track, track, radius);
            let done = frac(rc.right);
            if marquee {
                // A block a quarter of the width, running from the left edge past the right.
                let bw = (rc.right / 4).max(radius * 2);
                let x = (rc.right + bw) * phase / 100 - bw;
                round_rect(mem, RECT { left: x.max(0), right: (x + bw).min(rc.right), ..rc }, accent, accent, radius);
            } else if done > 0 {
                round_rect(mem, RECT { right: done.max(radius), ..rc }, accent, accent, radius);
            }
        }
        let _ = Gdi::BitBlt(dc, 0, 0, rc.right, rc.bottom, Some(mem), 0, 0, Gdi::SRCCOPY);
        Gdi::SelectObject(mem, old);
        let _ = Gdi::DeleteObject(bmp.into());
        let _ = Gdi::DeleteDC(mem);
        let _ = Gdi::EndPaint(h, &ps);
    }
}

extern "system" fn progress_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_TIMER => {
            with_range(h, |r| r.phase = (r.phase + 2) % 100);
            LRESULT(0)
        }
        wm::WM_PAINT => {
            range_paint(h, false);
            LRESULT(0)
        }
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_NCDESTROY => {
            forget(h);
            drop(unsafe { take_state::<Range>(h) });
            unsafe { wm::DefWindowProcW(h, msg, w, l) }
        }
        _ => unsafe { wm::DefWindowProcW(h, msg, w, l) },
    }
}

/// Moves a slider to `v` and, when that is a change, tells its parent.
fn slide_to(h: HWND, v: i32) {
    let Some(r) = (unsafe { state::<Range>(h) }) else { return };
    let v = v.clamp(r.min, r.max);
    if v == r.value {
        return;
    }
    r.value = v;
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
        let id = wm::GetDlgCtrlID(h) as usize;
        if let Ok(parent) = wm::GetParent(h) {
            wm::SendMessageW(parent, wm::WM_COMMAND, Some(WPARAM(id | (VALUE_CHANGED as usize) << 16)), Some(LPARAM(h.0 as isize)));
        }
    }
}

fn slider_value_at(h: HWND, (x, y): (i32, i32)) -> i32 {
    let Some(r) = (unsafe { state::<Range>(h) }) else { return 0 };
    let mut rc = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(h, &mut rc);
    }
    let thumb = px(h, 18);
    if r.vertical {
        let (y0, y1) = (thumb / 2, rc.bottom - thumb / 2);
        let span = (y1 - y0).max(1) as i64;
        return r.min + (((y1 - y.clamp(y0, y1)) as i64 * (r.max - r.min) as i64 + span / 2) / span) as i32;
    }
    let (x0, x1) = (thumb / 2, rc.right - thumb / 2);
    let span = (x1 - x0).max(1) as i64;
    r.min + (((x - x0).clamp(0, x1 - x0) as i64 * (r.max - r.min) as i64 + span / 2) / span) as i32
}

extern "system" fn slider_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture, SetFocus, TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT};
    let redraw = || unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, false);
    };
    match msg {
        wm::WM_PAINT => {
            range_paint(h, true);
            LRESULT(0)
        }
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_GETDLGCODE => LRESULT(0x0001 /* DLGC_WANTARROWS */),
        wm::WM_SETFOCUS | wm::WM_KILLFOCUS | wm::WM_ENABLE => {
            redraw();
            LRESULT(0)
        }
        wm::WM_LBUTTONDOWN => {
            unsafe {
                let _ = SetFocus(Some(h));
                SetCapture(h);
            }
            if let Some(r) = unsafe { state::<Range>(h) } {
                r.drag = true;
            }
            slide_to(h, slider_value_at(h, point_of(l)));
            redraw();
            LRESULT(0)
        }
        wm::WM_MOUSEMOVE => {
            if with_css(h, |c| !std::mem::replace(&mut c.hot, true)).unwrap_or(false) {
                let mut t = TRACKMOUSEEVENT { cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32, dwFlags: TME_LEAVE, hwndTrack: h, dwHoverTime: 0 };
                let _ = unsafe { TrackMouseEvent(&mut t) };
                redraw();
            }
            if unsafe { state::<Range>(h) }.is_some_and(|r| r.drag) {
                slide_to(h, slider_value_at(h, point_of(l)));
            }
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            with_css(h, |c| c.hot = false);
            redraw();
            LRESULT(0)
        }
        wm::WM_LBUTTONUP | wm::WM_CAPTURECHANGED => {
            if let Some(r) = unsafe { state::<Range>(h) } {
                if std::mem::replace(&mut r.drag, false) && msg == wm::WM_LBUTTONUP {
                    let _ = unsafe { ReleaseCapture() };
                }
            }
            redraw();
            LRESULT(0)
        }
        wm::WM_KEYDOWN => {
            let Some(r) = (unsafe { state::<Range>(h) }) else { return LRESULT(0) };
            let page = if r.large > 0 { r.large } else { ((r.max - r.min) / 10).max(1) };
            let (v, small) = (r.value, r.small.max(1));
            let to = match w.0 as u16 {
                0x25 | 0x28 => v - small, // left, down
                0x27 | 0x26 => v + small, // right, up
                0x21 => v + page,     // page up
                0x22 => v - page,     // page down
                0x24 => r.min,        // home
                0x23 => r.max,        // end
                _ => return unsafe { wm::DefWindowProcW(h, msg, w, l) },
            };
            slide_to(h, to);
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            forget(h);
            drop(unsafe { take_state::<Range>(h) });
            unsafe { wm::DefWindowProcW(h, msg, w, l) }
        }
        _ => unsafe { wm::DefWindowProcW(h, msg, w, l) },
    }
}

/// A box of several lines in the control font, wrapping words - a form's text area.
pub fn textarea(parent: HWND, text: &str, id: i32, f: HFONT) -> HWND {
    textarea_with(parent, text, id, f, true, "vertical")
}

/// A box of several lines with WinForms' WordWrap and ScrollBars (`none`, `vertical`,
/// `horizontal`, `both`) - styles Windows takes only when the box is made.
pub fn textarea_with(parent: HWND, text: &str, id: i32, f: HFONT, word_wrap: bool, scroll_bars: &str) -> HWND {
    const ES_MULTILINE: u32 = 0x0004;
    const ES_AUTOVSCROLL: u32 = 0x0040;
    const ES_AUTOHSCROLL: u32 = 0x0080;
    const ES_WANTRETURN: u32 = 0x1000;
    const EM_SETMARGINS: u32 = 0x00D3;
    let mut style = WINDOW_STYLE(ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN | if word_wrap { 0 } else { ES_AUTOHSCROLL }) | wm::WS_TABSTOP | wm::WS_BORDER;
    if matches!(scroll_bars, "vertical" | "both") {
        style |= wm::WS_VSCROLL;
    }
    if matches!(scroll_bars, "horizontal" | "both") {
        style |= wm::WS_HSCROLL;
    }
    let h = child(parent, "EDIT", text, style, WINDOW_EX_STYLE(0), id, f);
    mark(h, Kind::Edit);
    unsafe {
        let m = px(parent, 6) as isize;
        wm::SendMessageW(h, EM_SETMARGINS, Some(WPARAM(3)), Some(LPARAM((m << 16) | m)));
        let _ = windows::Win32::UI::Shell::SetWindowSubclass(h, Some(edit_proc), 2, 0);
    }
    theme_native(h);
    h
}

/// A control's own look over the theme - `background: #12855f; color: white`, what a `#id { }`
/// rule would say. Phai Studio writes it from a control's colour properties.
pub fn set_own_style(h: HWND, decls: &str) -> Result<(), theme::ThemeError> {
    let own = theme::declarations_style(decls)?;
    with_css(h, |c| {
        c.own = own;
        c.own_text = decls.trim().to_string();
    });
    theme_native(h);
    // A list's border may be thicker or thinner now: its edge worked out again.
    if kind_of(h) == Some(Kind::List) {
        frame_changed(h);
    }
    unsafe {
        let _ = Gdi::InvalidateRect(Some(h), None, true);
        let _ = Gdi::RedrawWindow(Some(h), None, None, Gdi::RDW_FRAME | Gdi::RDW_INVALIDATE);
    }
    Ok(())
}

// --- toolbar --------------------------------------------------------------------------------------
//
// Windows' own `ToolbarWindow32`, icons only - the label is the tooltip (PhaiDesk's owner,
// 2026-11-16) - with glyphs of Windows' icon font in the theme's `toolbar { icon-color }`.

const TB_BUTTONSTRUCTSIZE: u32 = 0x041E;
const TB_ADDBUTTONSW: u32 = 0x0444;
const TB_AUTOSIZE: u32 = 0x0421;
const TB_DELETEBUTTON: u32 = 0x0416;
const TB_BUTTONCOUNT: u32 = 0x0418;
const TB_SETIMAGELIST: u32 = 0x0430;
const TB_GETIMAGELIST: u32 = 0x0431;
const TB_SETBUTTONWIDTH: u32 = 0x043B;
const TB_SETPADDING: u32 = 0x0457;
const BTNS_SEP: u8 = 0x01;
const TBSTATE_ENABLED: u8 = 0x04;

/// A toolbar: `(command, label, glyph)` per button, `(0, "", 0)` a separator. Commands arrive as
/// `WM_COMMAND` with the id.
pub fn toolbar(parent: HWND, id: i32, items: &[(i32, &str, u16)]) -> HWND {
    const TBSTYLE_FLAT: u32 = 0x0800;
    const TBSTYLE_TOOLTIPS: u32 = 0x0100;
    const TBSTYLE_TRANSPARENT: u32 = 0x8000;
    const TBSTYLE_LIST: u32 = 0x1000;
    const CCS_NODIVIDER: u32 = 0x0040;
    const CCS_NORESIZE: u32 = 0x0004;
    const CCS_NOPARENTALIGN: u32 = 0x0008;
    const TB_SETEXTENDEDSTYLE: u32 = 0x0454;
    const TBSTYLE_EX_MIXEDBUTTONS: isize = 0x0008;
    let style = TBSTYLE_FLAT | TBSTYLE_LIST | TBSTYLE_TOOLTIPS | TBSTYLE_TRANSPARENT | CCS_NODIVIDER | CCS_NORESIZE | CCS_NOPARENTALIGN;
    let h = child(parent, "ToolbarWindow32", "", WINDOW_STYLE(style), WINDOW_EX_STYLE(0), id, HFONT::default());
    mark(h, Kind::Toolbar);
    unsafe {
        wm::SendMessageW(h, TB_BUTTONSTRUCTSIZE, Some(WPARAM(std::mem::size_of::<windows::Win32::UI::Controls::TBBUTTON>())), None);
        wm::SendMessageW(h, TB_SETEXTENDEDSTYLE, None, Some(LPARAM(TBSTYLE_EX_MIXEDBUTTONS)));
        wm::SendMessageW(h, 0x042F /* TB_SETINDENT */, Some(WPARAM(px(parent, 4) as usize)), None);
    }
    set_toolbar(h, items);
    h
}

/// Toolbars that stand as a column of buttons, one of which is chosen ([`toolbar_column`]).
static COLUMNS: Mutex<Option<HashSet<isize>>> = Mutex::new(None);

/// A column of icon buttons of which one is chosen at a time - a toolbox. The labels are the
/// tooltips; the chosen one shows pressed. Commands arrive as `WM_COMMAND`.
pub fn toolbar_column(parent: HWND, id: i32, items: &[(i32, &str, u16)]) -> HWND {
    const CCS_VERT: u32 = 0x0080;
    const TBSTYLE_WRAPABLE: u32 = 0x0200;
    let h = toolbar(parent, id, &[]);
    unsafe {
        let style = wm::GetWindowLongW(h, wm::GWL_STYLE) as u32;
        wm::SetWindowLongW(h, wm::GWL_STYLE, (style | CCS_VERT | TBSTYLE_WRAPABLE) as i32);
    }
    COLUMNS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashSet::new).insert(h.0 as isize);
    set_toolbar(h, items);
    h
}

/// Each column's chosen command, kept so a rebuilt column shows it still.
static CHOSEN: Mutex<Option<HashMap<isize, i32>>> = Mutex::new(None);

/// Shows `command` as the chosen button of a [`toolbar_column`].
pub fn toolbar_choose(h: HWND, command: i32) {
    CHOSEN.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).insert(h.0 as isize, command);
    unsafe {
        wm::SendMessageW(h, 0x0402 /* TB_CHECKBUTTON */, Some(WPARAM(command as usize)), Some(LPARAM(1)));
    }
}

// --- the toolbar tray: toolbars that can be dragged about, as Visual Studio's ----------------------

/// Windows' rebar: a tray of toolbars, each with a grip to drag it along its row, onto another
/// row, or up beside another toolbar. A toolbar too long for its room shows » for the rest.
pub fn rebar(parent: HWND, id: i32) -> HWND {
    const RBS_VARHEIGHT: u32 = 0x0200;
    const RBS_BANDBORDERS: u32 = 0x0400;
    const CCS_NODIVIDER: u32 = 0x0040;
    const CCS_NOPARENTALIGN: u32 = 0x0008;
    const CCS_NORESIZE: u32 = 0x0004;
    let style = WINDOW_STYLE(RBS_VARHEIGHT | RBS_BANDBORDERS | CCS_NODIVIDER | CCS_NOPARENTALIGN | CCS_NORESIZE) | wm::WS_CLIPCHILDREN | wm::WS_CLIPSIBLINGS;
    let h = child(parent, "ReBarWindow32", "", style, WINDOW_EX_STYLE(0), id, HFONT::default());
    rebar_colours(h);
    h
}

/// A rebar's bands drawn: on a dark tray, each band's gripper again in the theme's colours -
/// Windows draws it in its 3-D white.
fn rebar_custom_draw(l: LPARAM) -> LRESULT {
    use windows::Win32::UI::Controls as cc;
    let cd = unsafe { &mut *(l.0 as *mut cc::NMCUSTOMDRAW) };
    if !theme::current().is_dark() {
        return LRESULT(cc::CDRF_DODEFAULT as isize);
    }
    match cd.dwDrawStage {
        cc::CDDS_PREPAINT => LRESULT(cc::CDRF_NOTIFYITEMDRAW as isize),
        cc::CDDS_ITEMPREPAINT => LRESULT(cc::CDRF_NOTIFYPOSTPAINT as isize),
        cc::CDDS_ITEMPOSTPAINT => {
            let h = cd.hdr.hwndFrom;
            let r = cd.rc;
            let grip = RECT { left: r.left, top: r.top, right: r.left + px(h, 9), bottom: r.bottom };
            fill(cd.hdc, &grip, window_background_for(h));
            // Dots down the middle, as Visual Studio's grip.
            let dot = theme::current().var_color("--muted").unwrap_or(Color::rgb(0x80, 0x80, 0x80));
            let (x, s) = (grip.left + px(h, 3), px(h, 2).max(1));
            let mut y = grip.top + px(h, 6);
            while y + s <= grip.bottom - px(h, 5) {
                fill(cd.hdc, &RECT { left: x, top: y, right: x + s, bottom: y + s }, dot);
                y += s * 2;
            }
            LRESULT(cc::CDRF_DODEFAULT as isize)
        }
        _ => LRESULT(cc::CDRF_DODEFAULT as isize),
    }
}

/// The tray in the theme's colours.
pub fn rebar_colours(h: HWND) {
    unsafe {
        // Windows' own look paints the tray light whatever colour it is given: in a dark theme it is
        // drawn plain, in the theme's colour.
        let dark = theme::current().is_dark();
        let _ = windows::Win32::UI::Controls::SetWindowTheme(h, if dark { windows::core::w!(" ") } else { PCWSTR::null() }, if dark { windows::core::w!(" ") } else { PCWSTR::null() });
        wm::SendMessageW(h, 0x0413 /* RB_SETBKCOLOR */, None, Some(LPARAM(window_background_for(h).colorref() as isize)));
        wm::SendMessageW(h, 0x0415 /* RB_SETTEXTCOLOR */, None, Some(LPARAM(window_text_for(h).colorref() as isize)));
        // Drawn plain, the bands' borders and grippers are Windows' 3-D white and grey: a bright
        // line under each row on a dark tray. The theme's quiet border colour for both, then.
        #[repr(C)]
        struct ColorScheme {
            size: u32,
            highlight: u32,
            shadow: u32,
        }
        const CLR_DEFAULT: u32 = 0xFF00_0000;
        let line = theme::style("group", State::default()).border_or(Color::rgb(0x3a, 0x3a, 0x3a)).colorref();
        let scheme = if dark { ColorScheme { size: 12, highlight: line, shadow: line } } else { ColorScheme { size: 12, highlight: CLR_DEFAULT, shadow: CLR_DEFAULT } };
        wm::SendMessageW(h, 0x2002 /* RB_SETCOLORSCHEME */, None, Some(LPARAM(&scheme as *const _ as isize)));
        let _ = Gdi::InvalidateRect(Some(h), None, true);
    }
}

fn band_info(tb: HWND, id: u32, new_row: bool) -> windows::Win32::UI::Controls::REBARBANDINFOW {
    use windows::Win32::UI::Controls as cc;
    const RBBS_BREAK: u32 = 0x0001;
    const RBBS_GRIPPERALWAYS: u32 = 0x0080;
    const RBBS_USECHEVRON: u32 = 0x0200;
    let mut size = windows::Win32::Foundation::SIZE::default();
    unsafe {
        wm::SendMessageW(tb, 0x0453 /* TB_GETMAXSIZE */, None, Some(LPARAM(&mut size as *mut _ as isize)));
    }
    let button = px(tb, tool_sizes(tb).1);
    cc::REBARBANDINFOW {
        cbSize: std::mem::size_of::<cc::REBARBANDINFOW>() as u32,
        fMask: cc::RBBIM_CHILD | cc::RBBIM_CHILDSIZE | cc::RBBIM_SIZE | cc::RBBIM_STYLE | cc::RBBIM_ID | cc::RBBIM_IDEALSIZE,
        fStyle: RBBS_GRIPPERALWAYS | RBBS_USECHEVRON | if new_row { RBBS_BREAK } else { 0 },
        hwndChild: tb,
        // Shrunk as far as three buttons; the rest go behind ».
        cxMinChild: (button * 3) as u32,
        cyMinChild: (size.cy.max(button) + px(tb, 4)) as u32,
        cx: (size.cx + px(tb, 16)) as u32,
        cxIdeal: size.cx as u32,
        wID: id,
        ..Default::default()
    }
}

/// Puts a toolbar into the tray as band `id`, on a row of its own when `new_row`.
pub fn rebar_add(rb: HWND, tb: HWND, id: u32, new_row: bool) {
    let info = band_info(tb, id, new_row);
    unsafe {
        wm::SendMessageW(rb, 0x0401 /* RB_INSERTBANDW */, Some(WPARAM(usize::MAX)), Some(LPARAM(&info as *const _ as isize)));
    }
}

/// Every band sized again for its toolbar - after a language or DPI change made its buttons anew.
pub fn rebar_fit(rb: HWND) {
    use windows::Win32::UI::Controls as cc;
    unsafe {
        let n = wm::SendMessageW(rb, 0x040C /* RB_GETBANDCOUNT */, None, None).0.max(0) as usize;
        for i in 0..n {
            let mut get = cc::REBARBANDINFOW { cbSize: std::mem::size_of::<cc::REBARBANDINFOW>() as u32, fMask: cc::RBBIM_CHILD | cc::RBBIM_ID | cc::RBBIM_STYLE | cc::RBBIM_IDEALSIZE, ..Default::default() };
            wm::SendMessageW(rb, 0x041C /* RB_GETBANDINFOW */, Some(WPARAM(i)), Some(LPARAM(&mut get as *mut _ as isize)));
            if get.hwndChild.is_invalid() {
                continue;
            }
            // Its place in the tray stays: only the sizes change.
            let mut info = band_info(get.hwndChild, get.wID, false);
            // Unchanged: nothing to lay out again.
            if info.cxIdeal == get.cxIdeal {
                continue;
            }
            info.fMask = cc::RBBIM_CHILDSIZE | cc::RBBIM_IDEALSIZE;
            wm::SendMessageW(rb, 0x040B /* RB_SETBANDINFOW */, Some(WPARAM(i)), Some(LPARAM(&info as *const _ as isize)));
        }
    }
}

/// The tray's height now, device pixels.
pub fn rebar_height(rb: HWND) -> i32 {
    unsafe { wm::SendMessageW(rb, 0x041B /* RB_GETBARHEIGHT */, None, None).0 as i32 }
}

/// The buttons of a toolbar that lie past `width` device pixels - hidden behind its » - as
/// `(command, name)`.
pub fn toolbar_hidden(tb: HWND, width: i32) -> Vec<(i32, String)> {
    let items: Vec<(i32, String, u16)> = TOOLBARS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|m| m.get(&(tb.0 as isize)).cloned()).unwrap_or_default();
    let mut out = Vec::new();
    for (k, (cmd, label, _)) in items.iter().enumerate() {
        if *cmd == 0 {
            continue;
        }
        let r = toolbar_item_rect(tb, k);
        if r.right > width {
            out.push((*cmd, label.clone()));
        }
    }
    out
}

/// Shows a toolbar button pressed in (`on`) or not - a toggle, as Visual Studio's Categorized
/// and Alphabetical are. The theme's pressed colour is drawn behind it.
pub fn toolbar_check(h: HWND, command: i32, on: bool) {
    const BTNS_CHECK: u8 = 0x02;
    const TBIF_STYLE: u32 = 0x0008;
    let info = windows::Win32::UI::Controls::TBBUTTONINFOW {
        cbSize: std::mem::size_of::<windows::Win32::UI::Controls::TBBUTTONINFOW>() as u32,
        dwMask: windows::Win32::UI::Controls::TBBUTTONINFOW_MASK(TBIF_STYLE),
        fsStyle: BTNS_CHECK,
        ..Default::default()
    };
    unsafe {
        wm::SendMessageW(h, 0x0440 /* TB_SETBUTTONINFOW */, Some(WPARAM(command as usize)), Some(LPARAM(&info as *const _ as isize)));
        wm::SendMessageW(h, 0x0402 /* TB_CHECKBUTTON */, Some(WPARAM(command as usize)), Some(LPARAM(on as isize)));
    }
}

/// Each toolbar's buttons, so [`retheme`] can draw their icons again in the new colour.
type ToolItems = Vec<(i32, String, u16)>;
static TOOLBARS: Mutex<Option<HashMap<isize, ToolItems>>> = Mutex::new(None);

/// A toolbar button in design pixels: Windows' small toolbar - a 16 px icon in a 28 px square.
const TOOL_ICON: i32 = 16;
const TOOL_BUTTON: i32 = 28;

/// Toolbars made smaller than Windows' small toolbar: their icon and button, design pixels.
static COMPACT: Mutex<Option<HashMap<isize, (i32, i32)>>> = Mutex::new(None);

/// A toolbar's icon and button size, design pixels: Windows' small toolbar unless made compact.
fn tool_sizes(h: HWND) -> (i32, i32) {
    COMPACT.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|m| m.get(&(h.0 as isize)).copied()).unwrap_or((TOOL_ICON, TOOL_BUTTON))
}

/// Makes a toolbar's icons and buttons smaller - a pane's own row of icons, as Visual Studio's
/// Properties has - and draws them again.
pub fn toolbar_compact(h: HWND, icon: i32, button: i32, items: &[(i32, &str, u16)]) {
    COMPACT.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).insert(h.0 as isize, (icon, button));
    set_toolbar(h, items);
}

/// Replaces every button - for a language switch, a theme switch or a DPI change. The icons are
/// drawn again in the theme's colour at the toolbar's scale.
/// Toolbar buttons that stay down until clicked again (CheckOnClick), by toolbar.
static TOGGLES: Mutex<Option<HashMap<isize, Vec<i32>>>> = Mutex::new(None);

/// Toolbar `h`'s button `id` stays down when clicked, until clicked again.
pub fn toolbar_toggle_style(h: HWND, id: i32) {
    {
        let mut t = TOGGLES.lock().unwrap_or_else(|e| e.into_inner());
        let v = t.get_or_insert_with(HashMap::new).entry(h.0 as isize).or_default();
        if !v.contains(&id) {
            v.push(id);
        }
    }
    check_style(h, id);
}

fn check_style(h: HWND, id: i32) {
    use windows::Win32::UI::Controls as cc;
    let mut info = cc::TBBUTTONINFOW { cbSize: std::mem::size_of::<cc::TBBUTTONINFOW>() as u32, dwMask: cc::TBIF_STYLE, ..Default::default() };
    unsafe {
        wm::SendMessageW(h, 0x043F /* TB_GETBUTTONINFOW */, Some(WPARAM(id as usize)), Some(LPARAM(&mut info as *mut _ as isize)));
        info.fsStyle |= 0x02; /* BTNS_CHECK */
        wm::SendMessageW(h, 0x0440 /* TB_SETBUTTONINFOW */, Some(WPARAM(id as usize)), Some(LPARAM(&info as *const _ as isize)));
    }
}

pub fn set_toolbar(h: HWND, items: &[(i32, &str, u16)]) {
    // Made again (a new theme, the other language): its buttons that stay down keep being so.
    let toggles = TOGGLES.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|m| m.get(&(h.0 as isize)).cloned()).unwrap_or_default();
    let down: Vec<i32> = toggles.iter().copied().filter(|id| unsafe { wm::SendMessageW(h, 0x040A /* TB_ISBUTTONCHECKED */, Some(WPARAM(*id as usize)), None).0 } != 0).collect();
    set_toolbar_buttons(h, items);
    for id in &toggles {
        check_style(h, *id);
    }
    for id in down {
        unsafe {
            wm::SendMessageW(h, 0x0402 /* TB_CHECKBUTTON */, Some(WPARAM(id as usize)), Some(LPARAM(1)));
        }
    }
}

fn set_toolbar_buttons(h: HWND, items: &[(i32, &str, u16)]) {
    use windows::Win32::UI::Controls::{self as cc, TBBUTTON};
    TOOLBARS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).insert(h.0 as isize, items.iter().map(|(c, l, g)| (*c, l.to_string(), *g)).collect());
    unsafe {
        let (icon_dp, button_dp) = tool_sizes(h);
        let icon = px(h, icon_dp);
        let colour = theme::style("toolbar", State::default()).icon_color.unwrap_or(window_text_for(h));
        let list = cc::ImageList_Create(icon, icon, cc::ILC_COLOR32 | cc::ILC_MASK, items.len() as i32, 0);
        for (cmd, _, g) in items {
            if *cmd != 0 {
                if let Some(i) = crate::icon::glyph_icon(*g, icon, colour) {
                    cc::ImageList_ReplaceIcon(list, -1, i);
                    let _ = wm::DestroyIcon(i);
                }
            }
        }
        let old = wm::SendMessageW(h, TB_GETIMAGELIST, None, None).0;
        wm::SendMessageW(h, TB_SETIMAGELIST, Some(WPARAM(0)), Some(LPARAM(list.0 as isize)));
        if old != 0 {
            let _ = cc::ImageList_Destroy(Some(cc::HIMAGELIST(old as _)));
        }
        // Greyed buttons get their icons in the muted colour, not Windows' dithering.
        let muted = theme::current().var_color("--muted").unwrap_or(Color::rgb(0xa0, 0xa0, 0xa0));
        let grey = Color::rgb(((muted.r as u16 + 2 * window_background_for(h).r as u16) / 3) as u8, ((muted.g as u16 + 2 * window_background_for(h).g as u16) / 3) as u8, ((muted.b as u16 + 2 * window_background_for(h).b as u16) / 3) as u8);
        let dim = cc::ImageList_Create(icon, icon, cc::ILC_COLOR32 | cc::ILC_MASK, items.len() as i32, 0);
        for (cmd, _, g) in items {
            if *cmd != 0 {
                if let Some(i) = crate::icon::glyph_icon(*g, icon, grey) {
                    cc::ImageList_ReplaceIcon(dim, -1, i);
                    let _ = wm::DestroyIcon(i);
                }
            }
        }
        let old_dim = wm::SendMessageW(h, 0x0436 /* TB_SETDISABLEDIMAGELIST */, Some(WPARAM(0)), Some(LPARAM(dim.0 as isize))).0;
        if old_dim != 0 {
            let _ = cc::ImageList_Destroy(Some(cc::HIMAGELIST(old_dim as _)));
        }
        // Hidden buttons stay hidden across the rebuild.
        // (A button not there yet answers -1, not "no": ask only of those that are.)
        let there = |c: i32| wm::SendMessageW(h, 0x0419 /* TB_COMMANDTOINDEX */, Some(WPARAM(c as usize)), None).0 >= 0;
        let hidden: Vec<i32> = items.iter().map(|(c, _, _)| *c).filter(|c| *c != 0 && there(*c) && wm::SendMessageW(h, 0x040C /* TB_ISBUTTONHIDDEN */, Some(WPARAM(*c as usize)), None).0 > 0).collect();
        // Greyed buttons stay greyed across the rebuild, as toggles pressed in stay pressed in.
        let greyed: Vec<i32> = items.iter().map(|(c, _, _)| *c).filter(|c| *c != 0 && wm::SendMessageW(h, 0x0409 /* TB_ISBUTTONENABLED */, Some(WPARAM(*c as usize)), None).0 == 0 && wm::SendMessageW(h, 0x0419 /* TB_COMMANDTOINDEX */, Some(WPARAM(*c as usize)), None).0 >= 0).collect();
        let pressed: Vec<i32> = items.iter().map(|(c, _, _)| *c).filter(|c| *c != 0 && wm::SendMessageW(h, 0x040A /* TB_ISBUTTONCHECKED */, Some(WPARAM(*c as usize)), None).0 != 0).collect();
        let toggles: Vec<i32> = items.iter().map(|(c, _, _)| *c).filter(|c| *c != 0 && {
            let mut info = windows::Win32::UI::Controls::TBBUTTONINFOW { cbSize: std::mem::size_of::<windows::Win32::UI::Controls::TBBUTTONINFOW>() as u32, dwMask: windows::Win32::UI::Controls::TBBUTTONINFOW_MASK(0x0008), ..Default::default() };
            wm::SendMessageW(h, 0x043F /* TB_GETBUTTONINFOW */, Some(WPARAM(*c as usize)), Some(LPARAM(&mut info as *mut _ as isize))).0 >= 0 && info.fsStyle & 0x02 != 0
        }).collect();
        while wm::SendMessageW(h, TB_BUTTONCOUNT, None, None).0 > 0 {
            wm::SendMessageW(h, TB_DELETEBUTTON, Some(WPARAM(0)), None);
        }
        // The strings must outlive the call only; the toolbar copies them.
        let labels: Vec<Vec<u16>> = items.iter().map(|(_, l, _)| wide(l)).collect();
        let mut image = 0;
        let buttons: Vec<TBBUTTON> = items
            .iter()
            .zip(&labels)
            .map(|((cmd, _, _), label)| {
                if *cmd == 0 {
                    // `gap:N` - room N design pixels wide for a box the owner lays over it.
                    let lbl = String::from_utf16_lossy(&label[..label.len().saturating_sub(1)]);
                    let wide_gap = lbl.strip_prefix("gap:").and_then(|n| n.parse::<i32>().ok()).unwrap_or(6);
                    TBBUTTON { iBitmap: px(h, wide_gap), idCommand: 0, fsState: 0, fsStyle: BTNS_SEP, ..Default::default() }
                } else {
                    image += 1;
                    TBBUTTON { iBitmap: image - 1, idCommand: *cmd, fsState: TBSTATE_ENABLED, iString: label.as_ptr() as isize, ..Default::default() }
                }
            })
            .collect();
        wm::SendMessageW(h, TB_ADDBUTTONSW, Some(WPARAM(buttons.len())), Some(LPARAM(buttons.as_ptr() as isize)));
        for c in toggles {
            toolbar_check(h, c, pressed.contains(&c));
        }
        for c in greyed {
            wm::SendMessageW(h, 0x0401 /* TB_ENABLEBUTTON */, Some(WPARAM(c as usize)), Some(LPARAM(0)));
        }
        for c in hidden {
            wm::SendMessageW(h, 0x0404 /* TB_HIDEBUTTON */, Some(WPARAM(c as usize)), Some(LPARAM(1)));
        }
        // A column: every button on a row of its own, one of the group chosen at a time.
        if COLUMNS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().is_some_and(|c| c.contains(&(h.0 as isize))) {
            const TBSTATE_WRAP: u8 = 0x20;
            const BTNS_CHECKGROUP: u8 = 0x06;
            const TBIF_STYLE: u32 = 0x0008;
            const TBIF_STATE: u32 = 0x0004;
            for (cmd, _, _) in items.iter().filter(|(c, _, _)| *c != 0) {
                let info = windows::Win32::UI::Controls::TBBUTTONINFOW {
                    cbSize: std::mem::size_of::<windows::Win32::UI::Controls::TBBUTTONINFOW>() as u32,
                    dwMask: windows::Win32::UI::Controls::TBBUTTONINFOW_MASK(TBIF_STYLE | TBIF_STATE),
                    fsStyle: BTNS_CHECKGROUP,
                    fsState: TBSTATE_ENABLED | TBSTATE_WRAP,
                    ..Default::default()
                };
                wm::SendMessageW(h, 0x0440 /* TB_SETBUTTONINFOW */, Some(WPARAM(*cmd as usize)), Some(LPARAM(&info as *const _ as isize)));
            }
            let chosen = CHOSEN.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|m| m.get(&(h.0 as isize)).copied());
            if let Some(c) = chosen {
                wm::SendMessageW(h, 0x0402 /* TB_CHECKBUTTON */, Some(WPARAM(c as usize)), Some(LPARAM(1)));
            }
        }
        let b = px(h, button_dp) as isize;
        let pad = (px(h, button_dp) - icon) as isize;
        wm::SendMessageW(h, TB_SETPADDING, None, Some(LPARAM((pad << 16) | (pad & 0xFFFF))));
        wm::SendMessageW(h, TB_SETBUTTONWIDTH, None, Some(LPARAM((b << 16) | (b & 0xFFFF))));
        wm::SendMessageW(h, 0x041F /* TB_SETBUTTONSIZE */, None, Some(LPARAM((b << 16) | (b & 0xFFFF))));
        wm::SendMessageW(h, TB_AUTOSIZE, None, None);
    }
    // Buttons that show their words keep them across a rebuild.
    let shown: Vec<(i32, String)> = SHOWN.lock().unwrap_or_else(|e| e.into_inner()).as_ref().map(|m| m.iter().filter(|((t, _), _)| *t == h.0 as isize).map(|((_, c), s)| (*c, s.clone())).collect()).unwrap_or_default();
    for (c, text) in shown {
        apply_text(h, c, &text);
    }
}

/// Words shown on toolbar buttons beside their icons, by toolbar and command.
static SHOWN: Mutex<Option<HashMap<(isize, i32), String>>> = Mutex::new(None);

/// Shows `text` on a toolbar button beside its icon - Visual Studio's "▶ Project" Start button.
pub fn toolbar_text(h: HWND, command: i32, text: &str) {
    SHOWN.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).insert((h.0 as isize, command), text.to_string());
    apply_text(h, command, text);
}

fn apply_text(h: HWND, command: i32, text: &str) {
    const BTNS_SHOWTEXT: u8 = 0x40;
    const BTNS_AUTOSIZE: u8 = 0x10;
    const TBIF_TEXT: u32 = 0x0002;
    const TBIF_STYLE: u32 = 0x0008;
    const TBIF_SIZE: u32 = 0x0040;
    let mut w = wide(text);
    let font = unsafe { HFONT(wm::SendMessageW(h, wm::WM_GETFONT, None, None).0 as *mut _) };
    let (tw, _) = measure(h, font, text);
    let info = windows::Win32::UI::Controls::TBBUTTONINFOW {
        cbSize: std::mem::size_of::<windows::Win32::UI::Controls::TBBUTTONINFOW>() as u32,
        dwMask: windows::Win32::UI::Controls::TBBUTTONINFOW_MASK(TBIF_TEXT | TBIF_STYLE | TBIF_SIZE),
        fsStyle: BTNS_SHOWTEXT | BTNS_AUTOSIZE,
        cx: (px(h, TOOL_BUTTON) + tw + px(h, 10)) as u16,
        pszText: windows::core::PWSTR(w.as_mut_ptr()),
        ..Default::default()
    };
    unsafe {
        wm::SendMessageW(h, 0x0440 /* TB_SETBUTTONINFOW */, Some(WPARAM(command as usize)), Some(LPARAM(&info as *const _ as isize)));
    }
}

/// Where a toolbar's item `index` is, in its client pixels.
pub fn toolbar_item_rect(h: HWND, index: usize) -> RECT {
    let mut r = RECT::default();
    unsafe {
        wm::SendMessageW(h, 0x041D /* TB_GETITEMRECT */, Some(WPARAM(index)), Some(LPARAM(&mut r as *mut _ as isize)));
    }
    r
}

/// A toolbar button shown or hidden - Add or Remove Buttons.
pub fn toolbar_show_button(h: HWND, command: i32, on: bool) {
    unsafe {
        wm::SendMessageW(h, 0x0404 /* TB_HIDEBUTTON */, Some(WPARAM(command as usize)), Some(LPARAM(!on as isize)));
    }
}

/// Whether a toolbar button is shown.
pub fn toolbar_button_shown(h: HWND, command: i32) -> bool {
    unsafe { wm::SendMessageW(h, 0x040C /* TB_ISBUTTONHIDDEN */, Some(WPARAM(command as usize)), None).0 == 0 }
}

/// Where a toolbar button is, by its command, in the toolbar's client pixels.
pub fn toolbar_button_rect(h: HWND, command: i32) -> RECT {
    let mut r = RECT::default();
    unsafe {
        wm::SendMessageW(h, 0x0433 /* TB_GETRECT */, Some(WPARAM(command as usize)), Some(LPARAM(&mut r as *mut _ as isize)));
    }
    r
}

/// A toolbar button usable or greyed.
pub fn toolbar_enable(h: HWND, command: i32, on: bool) {
    unsafe {
        wm::SendMessageW(h, 0x0401 /* TB_ENABLEBUTTON */, Some(WPARAM(command as usize)), Some(LPARAM(on as isize)));
    }
}

/// A toolbar's height in design pixels: its buttons and 3 px above and below.
pub const TOOLBAR_HEIGHT: i32 = TOOL_BUTTON + 6;

/// The toolbar's height at its scale, for docking.
pub fn toolbar_height(h: HWND) -> i32 {
    px(h, tool_sizes(h).1 + 6)
}

/// `NM_CUSTOMDRAW` from a toolbar: the button under the pointer in the theme's hover colour.
pub fn toolbar_custom_draw(l: LPARAM) -> LRESULT {
    use windows::Win32::UI::Controls as cc;
    const TBCDRF_NOEDGES: isize = 0x0001_0000;
    const TBCDRF_HILITEHOTTRACK: isize = 0x0002_0000;
    unsafe {
        let cd = &mut *(l.0 as *mut cc::NMTBCUSTOMDRAW);
        match cd.nmcd.dwDrawStage {
            cc::CDDS_PREPAINT => LRESULT(cc::CDRF_NOTIFYITEMDRAW as isize),
            cc::CDDS_ITEMPREPAINT => {
                let hover = theme::style("button", State { hover: true, ..Default::default() });
                cd.clrHighlightHotTrack = cr(hover.background_or(Color::rgb(231, 243, 239)));
                // A button's words (Start) in the theme's, not Windows' black on a dark bar: the
                // visual style's own colour unless told to use these.
                cd.clrText = cr(window_text_for(cd.nmcd.hdr.hwndFrom));
                cd.clrTextHighlight = cr(hover.color_or(window_text_for(cd.nmcd.hdr.hwndFrom)));
                cd.clrBtnFace = cr(window_background_for(cd.nmcd.hdr.hwndFrom));
                let own_colours: isize = if theme::current().is_dark() { 0x0080_0000 /* TBCDRF_USECDCOLORS */ } else { 0 };
                // A chosen button of a column: the theme's pressed colour behind it, no 3-D edge.
                if cd.nmcd.uItemState.0 & cc::CDIS_CHECKED.0 != 0 {
                    let chosen = theme::style("button", State { pressed: true, ..Default::default() });
                    let r = cd.nmcd.rc;
                    round_rect(cd.nmcd.hdc, r, chosen.background_or(Color::rgb(208, 231, 223)), chosen.border_or(Color::rgb(18, 133, 95)), px(cd.nmcd.hdr.hwndFrom, 8));
                    cd.nmcd.uItemState = cc::NMCUSTOMDRAW_DRAW_STATE_FLAGS(cd.nmcd.uItemState.0 & !cc::CDIS_CHECKED.0);
                    return LRESULT(cc::CDRF_DODEFAULT as isize | TBCDRF_NOEDGES | 0x0040_0000 /* TBCDRF_NOBACKGROUND */ | own_colours);
                }
                LRESULT(cc::CDRF_DODEFAULT as isize | TBCDRF_HILITEHOTTRACK | TBCDRF_NOEDGES | own_colours)
            }
            _ => LRESULT(cc::CDRF_DODEFAULT as isize),
        }
    }
}

// --- the window frame -------------------------------------------------------------------------

/// Pop-up menus and scroll bars dark or light with the theme. Windows does this for Explorer
/// through two functions of uxtheme.dll it exports only by number (SetPreferredAppMode, 135;
/// FlushMenuThemes, 136) - the way Notepad++ and others follow a dark theme. Where they are
/// missing nothing changes.
fn app_dark(dark: bool) {
    use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
    type SetMode = unsafe extern "system" fn(i32) -> i32;
    type Flush = unsafe extern "system" fn();
    unsafe {
        let Ok(ux) = LoadLibraryW(windows::core::w!("uxtheme.dll")) else { return };
        if let Some(f) = GetProcAddress(ux, windows::core::PCSTR(135usize as *const u8)) {
            let set: SetMode = std::mem::transmute(f);
            set(if dark { 2 } else { 3 }); // ForceDark / ForceLight
        }
        if let Some(f) = GetProcAddress(ux, windows::core::PCSTR(136usize as *const u8)) {
            let flush: Flush = std::mem::transmute(f);
            flush();
        }
    }
}

/// The title bar follows the theme: dark when it is dark (Windows 10 2004 and later), and the
/// corners rounded on Windows 11. Older Windows ignore what they do not know.
pub fn theme_frame(hwnd: HWND) {
    use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};
    const DWMWA_USE_IMMERSIVE_DARK_MODE: i32 = 20;
    const DWMWA_WINDOW_CORNER_PREFERENCE: i32 = 33;
    const DWMWCP_ROUND: u32 = 2;
    let dark: u32 = theme::current().is_dark() as u32;
    app_dark(dark != 0);
    unsafe {
        let _ = DwmSetWindowAttribute(hwnd, DWMWINDOWATTRIBUTE(DWMWA_USE_IMMERSIVE_DARK_MODE), &dark as *const _ as *const _, 4);
        let _ = DwmSetWindowAttribute(hwnd, DWMWINDOWATTRIBUTE(DWMWA_WINDOW_CORNER_PREFERENCE), &DWMWCP_ROUND as *const _ as *const _, 4);
    }
}

/// After [`theme::set`]: the frame, every native control, every toolbar's icons and every pixel
/// of `top` again.
pub fn retheme(top: HWND) {
    unsafe extern "system" fn each(h: HWND, _: LPARAM) -> windows::core::BOOL {
        theme_native(h);
        if kind_of(h) == Some(Kind::Toolbar) {
            let items = TOOLBARS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|m| m.get(&(h.0 as isize)).cloned());
            if let Some(items) = items {
                let refs: Vec<(i32, &str, u16)> = items.iter().map(|(c, l, g)| (*c, l.as_str(), *g)).collect();
                set_toolbar(h, &refs);
            }
        }
        true.into()
    }
    theme_frame(top);
    unsafe {
        let _ = wm::EnumChildWindows(Some(top), Some(each), LPARAM(0));
        let _ = Gdi::RedrawWindow(Some(top), None, None, Gdi::RDW_INVALIDATE | Gdi::RDW_ERASE | Gdi::RDW_FRAME | Gdi::RDW_ALLCHILDREN);
    }
}

// --- a theme file, watched ----------------------------------------------------------------------
//
// Drop a `.css` beside the program, or save one being edited: every window follows within half a
// second. A sheet with a mistake keeps the theme that was working and says what is wrong.

const TIMER_THEME: usize = 0x7A11;

/// Posted to a watching window after its theme file was read: `WPARAM` 1 when it applied, 0 when
/// it had a mistake ([`theme_error`] says which).
pub const WM_THEME_CHANGED: u32 = wm::WM_APP + 0x3A11;

struct Watch {
    path: std::path::PathBuf,
    stamp: Option<std::time::SystemTime>,
}

static WATCHES: Mutex<Option<HashMap<isize, Watch>>> = Mutex::new(None);
static THEME_ERROR: Mutex<Option<String>> = Mutex::new(None);

/// What was wrong with the last theme file read, if anything.
pub fn theme_error() -> Option<String> {
    THEME_ERROR.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// Applies the theme in `path` to `top` now and again whenever the file changes, until the window
/// is destroyed. [`def`] does the watching; the window's procedure must pass `WM_TIMER` to it.
pub fn watch_theme(top: HWND, path: &std::path::Path) {
    WATCHES.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).insert(top.0 as isize, Watch { path: path.to_path_buf(), stamp: None });
    check_theme(top);
    unsafe {
        wm::SetTimer(Some(top), TIMER_THEME, 500, None);
    }
}

/// Stops watching; the theme in use stays.
pub fn unwatch_theme(top: HWND) {
    if let Some(m) = WATCHES.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
        m.remove(&(top.0 as isize));
    }
    unsafe {
        let _ = wm::KillTimer(Some(top), TIMER_THEME);
    }
}

fn check_theme(top: HWND) {
    let path = {
        let mut map = WATCHES.lock().unwrap_or_else(|e| e.into_inner());
        let Some(w) = map.get_or_insert_with(HashMap::new).get_mut(&(top.0 as isize)) else { return };
        let stamp = std::fs::metadata(&w.path).and_then(|m| m.modified()).ok();
        if stamp.is_none() || stamp == w.stamp {
            return;
        }
        w.stamp = stamp;
        w.path.clone()
    };
    let ok = match theme::load(&path) {
        Ok(t) => {
            theme::set(t);
            *THEME_ERROR.lock().unwrap_or_else(|e| e.into_inner()) = None;
            retheme(top);
            true
        }
        Err(e) => {
            *THEME_ERROR.lock().unwrap_or_else(|e| e.into_inner()) = Some(e.to_string());
            false
        }
    };
    unsafe {
        let _ = wm::PostMessageW(Some(top), WM_THEME_CHANGED, WPARAM(ok as usize), LPARAM(0));
    }
}

// --- small things ---------------------------------------------------------------------------------

pub fn move_to(h: impl Into<HWND>, x: i32, y: i32, w: i32, height: i32) {
    let h: HWND = h.into();
    unsafe {
        let _ = wm::MoveWindow(h, x, y, w.max(0), height.max(0), true);
    }
}

pub fn text_of(h: impl Into<HWND>) -> String {
    let h: HWND = h.into();
    unsafe {
        let n = wm::GetWindowTextLengthW(h);
        let mut buf = vec![0u16; n as usize + 1];
        let got = wm::GetWindowTextW(h, &mut buf);
        // A SysLink counts the end of its words as one of them.
        String::from_utf16_lossy(&buf[..got as usize]).trim_end_matches(' ').to_string()
    }
}

/// Fits a label that sizes itself once its words are in.
struct AutoFit(HWND);

impl Drop for AutoFit {
    fn drop(&mut self) {
        // A label the window shows through: the window under it painted again, or the old
        // words stay.
        if kind_of(self.0) == Some(Kind::Label) && window_patterned(self.0) {
            unsafe {
                if let Ok(parent) = wm::GetParent(self.0) {
                    let r = rect_in_parent(self.0);
                    let _ = Gdi::InvalidateRect(Some(parent), Some(&r), true);
                }
            }
        }
        if with_css(self.0, |c| c.auto).unwrap_or(false) {
            fit_label(self.0);
        }
    }
}

pub fn set_text(h: impl Into<HWND>, s: &str) {
    let h: HWND = h.into();
    let _fit = AutoFit(h);
    let w = wide(s);
    unsafe {
        let _ = wm::SetWindowTextW(h, PCWSTR(w.as_ptr()));
    }
}

pub fn enable(h: impl Into<HWND>, on: bool) {
    let h: HWND = h.into();
    unsafe {
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow(h, on);
    }
}

/// Shows a top-level window the first time. While a program is being photographed (`PHAI_SHOT`)
/// it shows without taking the foreground: a test must never catch what a person is typing into
/// another window.
pub fn show_top(h: HWND) {
    let quiet = std::env::var_os("PHAI_SHOT").is_some();
    unsafe {
        let _ = wm::ShowWindow(h, if quiet { wm::SW_SHOWNOACTIVATE } else { wm::SW_SHOW });
    }
}

pub fn show(h: impl Into<HWND>, on: bool) {
    let h: HWND = h.into();
    unsafe {
        let _ = wm::ShowWindow(h, if on { wm::SW_SHOWNA } else { wm::SW_HIDE });
    }
}

pub fn message(owner: Option<HWND>, text: &str, caption: &str, flags: wm::MESSAGEBOX_STYLE) -> wm::MESSAGEBOX_RESULT {
    let t = wide(text);
    let c = wide(caption);
    unsafe { wm::MessageBoxW(owner, PCWSTR(t.as_ptr()), PCWSTR(c.as_ptr()), flags) }
}

/// Stores `state` behind the window; [`state`] gets it back, [`take_state`] frees it.
pub fn set_state<T>(hwnd: HWND, state: Box<T>) {
    unsafe {
        wm::SetWindowLongPtrW(hwnd, wm::GWLP_USERDATA, Box::into_raw(state) as isize);
    }
}

/// # Safety
/// `T` must be the type given to [`set_state`] for this window, and the reference must not outlive
/// the window's own message handling.
pub unsafe fn state<'a, T>(hwnd: HWND) -> Option<&'a mut T> {
    let p = wm::GetWindowLongPtrW(hwnd, wm::GWLP_USERDATA) as *mut T;
    p.as_mut()
}

/// # Safety
/// As [`state`]; call once, at `WM_NCDESTROY`.
pub unsafe fn take_state<T>(hwnd: HWND) -> Option<Box<T>> {
    let p = wm::SetWindowLongPtrW(hwnd, wm::GWLP_USERDATA, 0) as *mut T;
    (!p.is_null()).then(|| Box::from_raw(p))
}

/// The message loop of one window's thread. `dialog` makes Tab move between controls.
///
/// With `PHAI_SHOT=out.png` in the environment, the window is photographed after it has drawn
/// (`PHAI_SHOT_DELAY` milliseconds, 800 by default) and the program ends - so any program made
/// with phai-ui, generated ones included, can be checked by its picture.
pub fn run_loop(dialog: Option<HWND>) {
    #[cfg(feature = "shot")]
    if let (Some(h), Ok(path)) = (dialog, std::env::var("PHAI_SHOT")) {
        static TARGET: Mutex<Option<(isize, String)>> = Mutex::new(None);
        unsafe extern "system" fn snap(_: HWND, _: u32, id: usize, _: u32) {
            unsafe {
                let _ = wm::KillTimer(None, id);
            }
            if let Some((h, path)) = TARGET.lock().unwrap_or_else(|e| e.into_inner()).take() {
                // A dialog of ours in front - a modal one open - is what is on screen: take that.
                let front = unsafe { wm::GetForegroundWindow() };
                let mut pid = 0u32;
                unsafe {
                    wm::GetWindowThreadProcessId(front, Some(&mut pid));
                }
                let h = if !front.is_invalid() && pid == std::process::id() { front } else { HWND(h as *mut _) };
                // A modal window over it, shown without taking the focus: that is what is on screen.
                let h = unsafe { wm::GetWindow(h, wm::GW_ENABLEDPOPUP) }.ok().filter(|p| !p.is_invalid() && p.0 != h.0).unwrap_or(h);
                if let Err(e) = crate::shot::save(h, std::path::Path::new(&path)) {
                    eprintln!("{e}");
                }
            }
            std::process::exit(0);
        }
        *TARGET.lock().unwrap_or_else(|e| e.into_inner()) = Some((h.0 as isize, path));
        let delay = std::env::var("PHAI_SHOT_DELAY").ok().and_then(|d| d.parse().ok()).unwrap_or(800);
        unsafe {
            wm::SetTimer(None, 0, delay, Some(snap));
        }
    }
    unsafe {
        let quiet = std::env::var_os("PHAI_SHOT").is_some();
        let mut msg = wm::MSG::default();
        while wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
            // Being photographed, a program takes no keys and no mouse: a person typing or moving the
            // mouse elsewhere must not reach it. Tests send theirs straight to the window, past this queue.
            if quiet && ((wm::WM_KEYFIRST..=wm::WM_KEYLAST).contains(&msg.message) || (wm::WM_MOUSEFIRST..=wm::WM_MOUSELAST).contains(&msg.message)) {
                continue;
            }
            if let Some(d) = dialog {
                if wm::IsDialogMessageW(d, &msg).as_bool() {
                    continue;
                }
            }
            // Ctrl+F4, Ctrl+Tab among MDI child forms.
            if crate::mdi::translate(&msg) {
                continue;
            }
            let _ = wm::TranslateMessage(&msg);
            wm::DispatchMessageW(&msg);
        }
    }
}

pub fn loword(v: usize) -> u16 {
    (v & 0xFFFF) as u16
}

pub fn hiword(v: usize) -> u16 {
    ((v >> 16) & 0xFFFF) as u16
}

/// Signed coordinates out of an `LPARAM` (`GET_X_LPARAM`).
pub fn point_of(l: LPARAM) -> (i32, i32) {
    ((l.0 & 0xFFFF) as i16 as i32, ((l.0 >> 16) & 0xFFFF) as i16 as i32)
}

// --- the menu bar, in the theme -----------------------------------------------------------------
//
// Windows draws the menu bar light whatever the theme says. Since Windows 10 it asks the window
// first, with two messages it does not document (WM_UAHDRAWMENU, WM_UAHDRAWMENUITEM) - the way
// Notepad++ and others draw a dark one. A sheet with a `menubar` rule answers them; one without
// leaves the bar to Windows. A Windows that stops sending them draws its own bar, nothing worse.

const WM_UAHDRAWMENU: u32 = 0x0091;
const WM_UAHDRAWMENUITEM: u32 = 0x0092;

#[repr(C)]
struct UahMenu {
    hmenu: HMENU,
    hdc: HDC,
    flags: u32,
}

#[repr(C)]
struct UahDrawMenuItem {
    dis: windows::Win32::UI::Controls::DRAWITEMSTRUCT,
    um: UahMenu,
    position: i32,
}

/// The menu bar's rectangle in window coordinates.
fn menu_bar_rect(hwnd: HWND) -> Option<RECT> {
    unsafe {
        let mut mbi = wm::MENUBARINFO { cbSize: std::mem::size_of::<wm::MENUBARINFO>() as u32, ..Default::default() };
        wm::GetMenuBarInfo(hwnd, wm::OBJID_MENU, 0, &mut mbi).ok()?;
        let mut w = RECT::default();
        wm::GetWindowRect(hwnd, &mut w).ok()?;
        let mut r = mbi.rcBar;
        let _ = Gdi::OffsetRect(&mut r, -w.left, -w.top);
        Some(r)
    }
}

fn themed_menu(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> Option<LRESULT> {
    use windows::Win32::UI::Controls::{ODS_DISABLED, ODS_GRAYED, ODS_HOTLIGHT, ODS_NOACCEL, ODS_SELECTED};
    let bar = theme::style("menubar", State::default());
    // A sheet that says nothing about the menu bar leaves it to Windows.
    bar.background?;
    let bg = bar.background_or(window_background_for(hwnd));
    match msg {
        WM_UAHDRAWMENU => {
            let um = unsafe { &*(l.0 as *const UahMenu) };
            let r = menu_bar_rect(hwnd)?;
            fill(um.hdc, &r, bg);
            Some(LRESULT(1))
        }
        WM_UAHDRAWMENUITEM => unsafe {
            let it = &*(l.0 as *const UahDrawMenuItem);
            let mut buf = [0u16; 256];
            let mut mii = wm::MENUITEMINFOW {
                cbSize: std::mem::size_of::<wm::MENUITEMINFOW>() as u32,
                fMask: wm::MIIM_STRING,
                dwTypeData: windows::core::PWSTR(buf.as_mut_ptr()),
                cch: buf.len() as u32 - 1,
                ..Default::default()
            };
            let _ = wm::GetMenuItemInfoW(it.um.hmenu, it.position as u32, true, &mut mii);
            let st = it.dis.itemState.0;
            let hot = st & (ODS_HOTLIGHT.0 | ODS_SELECTED.0) != 0;
            let off = st & (ODS_DISABLED.0 | ODS_GRAYED.0) != 0;
            let item = theme::style("menubar", State { hover: hot, disabled: off, ..Default::default() });
            fill(it.um.hdc, &it.dis.rcItem, if hot { item.background_or(bg) } else { bg });
            Gdi::SetBkMode(it.um.hdc, Gdi::TRANSPARENT);
            Gdi::SetTextColor(it.um.hdc, cr(item.color_or(window_text_for(hwnd))));
            let mut r = it.dis.rcItem;
            let mut flags = Gdi::DT_CENTER | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE;
            if st & ODS_NOACCEL.0 != 0 {
                flags |= Gdi::DT_HIDEPREFIX;
            }
            Gdi::DrawTextW(it.um.hdc, &mut buf[..mii.cch as usize], &mut r, flags);
            Some(LRESULT(1))
        },
        // After Windows paints the frame it leaves a light line under the bar; cover it.
        wm::WM_NCPAINT | wm::WM_NCACTIVATE => unsafe {
            let r = wm::DefWindowProcW(hwnd, msg, w, l);
            if let Some(bar_r) = menu_bar_rect(hwnd) {
                let dc = Gdi::GetWindowDC(Some(hwnd));
                let line = RECT { top: bar_r.bottom, bottom: bar_r.bottom + 1, ..bar_r };
                fill(dc, &line, bg);
                Gdi::ReleaseDC(Some(hwnd), dc);
            }
            Some(r)
        },
        _ => None,
    }
}

/// What a window procedure does not handle: the theme's background and colours, the library's
/// owner-drawn controls and custom-drawn lists and toolbars, then Windows' default.
pub fn def(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    use windows::Win32::UI::Controls as cc;
    if let Some(r) = themed_menu(hwnd, msg, w, l) {
        return r;
    }
    match msg {
        wm::WM_TIMER if w.0 == TIMER_THEME => {
            check_theme(hwnd);
            return LRESULT(0);
        }
        wm::WM_NCDESTROY => {
            if let Some(m) = WATCHES.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
                m.remove(&(hwnd.0 as isize));
            }
        }
        wm::WM_ERASEBKGND => {
            let dc = HDC(w.0 as *mut _);
            let mut r = RECT::default();
            unsafe {
                let _ = wm::GetClientRect(hwnd, &mut r);
            }
            fill(dc, &r, window_background_for(hwnd));
            // A gradient for the window, where its own colour does not stand over it.
            if root_colors(hwnd).0.is_none() {
                if let Some(g) = window_style().gradient {
                    paint_box(dc, r, window_background(), Some(&g), None, Color::TRANSPARENT, 0, 0);
                }
            }
            crate::form::paint_background(hwnd, dc, &r);
            return LRESULT(1);
        }
        wm::WM_CTLCOLORSTATIC | wm::WM_CTLCOLOREDIT | wm::WM_CTLCOLORLISTBOX | wm::WM_CTLCOLORBTN => {
            if let Some(r) = ctl_color(msg, w, l) {
                return r;
            }
        }
        wm::WM_DRAWITEM => {
            if draw_item(l) {
                return LRESULT(1);
            }
        }
        wm::WM_NOTIFY => {
            let hdr = unsafe { &*(l.0 as *const cc::NMHDR) };
            // A date that changed speaks as an edit does, so one `WM_COMMAND` arm hears both.
            if hdr.code == DTN_DATETIMECHANGE && kind_of(hdr.hwndFrom) == Some(Kind::Date) {
                unsafe {
                    wm::SendMessageW(hwnd, wm::WM_COMMAND, Some(WPARAM(hdr.idFrom | (VALUE_CHANGED as usize) << 16)), Some(LPARAM(hdr.hwndFrom.0 as isize)));
                }
                return LRESULT(0);
            }
            // LabelEdit: a name typed into a tree's item stands (Escape gives no words).
            if hdr.code == (-459i32) as u32 /* TVN_ENDLABELEDITW */ {
                let it = unsafe { &*(l.0 as *const cc::NMTVDISPINFOW) };
                return LRESULT((!it.item.pszText.is_null()) as isize);
            }
            if hdr.code == cc::NM_CUSTOMDRAW {
                match kind_of(hdr.hwndFrom) {
                    Some(Kind::List) => return list_custom_draw(l),
                    Some(Kind::Toolbar) => return toolbar_custom_draw(l),
                    Some(Kind::Tree) => return tree_custom_draw(l),
                    _ => {
                        let mut buf = [0u16; 16];
                        let n = unsafe { wm::GetClassNameW(hdr.hwndFrom, &mut buf) } as usize;
                        if String::from_utf16_lossy(&buf[..n]) == "ReBarWindow32" {
                            return rebar_custom_draw(l);
                        }
                    }
                }
            }
        }
        _ => {}
    }
    // An MDI frame or child form: the handling Windows asks of those.
    if let Some(r) = crate::mdi::def(hwnd, msg, w, l) {
        return r;
    }
    unsafe { wm::DefWindowProcW(hwnd, msg, w, l) }
}

/// A pop-up menu of `(id, label)` at the cursor; `0` separates. Returns the chosen id or 0.
pub fn popup(owner: HWND, items: &[(u32, &str)]) -> u32 {
    unsafe {
        let Ok(menu) = wm::CreatePopupMenu() else { return 0 };
        for (id, label) in items {
            if *id == 0 {
                let _ = wm::AppendMenuW(menu, wm::MF_SEPARATOR, 0, None);
            } else {
                let w = wide(label);
                let _ = wm::AppendMenuW(menu, wm::MF_STRING, *id as usize, PCWSTR(w.as_ptr()));
            }
        }
        let mut pt = Default::default();
        let _ = wm::GetCursorPos(&mut pt);
        let _ = wm::SetForegroundWindow(owner);
        let chosen = wm::TrackPopupMenu(menu, wm::TPM_RETURNCMD | wm::TPM_RIGHTBUTTON, pt.x, pt.y, Some(0), owner, None);
        let _ = wm::DestroyMenu(menu);
        chosen.0 as u32
    }
}

/// Puts a window over the middle of `owner` - where a second window or a dialog is expected.
pub fn center_on(h: HWND, owner: HWND) {
    unsafe {
        let (mut r, mut o) = (RECT::default(), RECT::default());
        if wm::GetWindowRect(h, &mut r).is_ok() && wm::GetWindowRect(owner, &mut o).is_ok() {
            let (w, ht) = (r.right - r.left, r.bottom - r.top);
            let x = o.left + ((o.right - o.left) - w) / 2;
            let y = o.top + ((o.bottom - o.top) - ht) / 2;
            let _ = wm::SetWindowPos(h, None, x.max(0), y.max(0), 0, 0, wm::SWP_NOSIZE | wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
        }
    }
}

/// Puts a window in the middle of the screen it is on (its work area, the taskbar left out) - as
/// WinForms' StartPosition = CenterScreen.
pub fn center_on_screen(h: HWND) {
    unsafe {
        let mon = Gdi::MonitorFromWindow(h, Gdi::MONITOR_DEFAULTTONEAREST);
        let mut mi = Gdi::MONITORINFO { cbSize: std::mem::size_of::<Gdi::MONITORINFO>() as u32, ..Default::default() };
        let mut r = RECT::default();
        if Gdi::GetMonitorInfoW(mon, &mut mi).as_bool() && wm::GetWindowRect(h, &mut r).is_ok() {
            let a = mi.rcWork;
            let (w, ht) = (r.right - r.left, r.bottom - r.top);
            let x = a.left + ((a.right - a.left) - w).max(0) / 2;
            let y = a.top + ((a.bottom - a.top) - ht).max(0) / 2;
            let _ = wm::SetWindowPos(h, None, x, y, 0, 0, wm::SWP_NOSIZE | wm::SWP_NOZORDER | wm::SWP_NOACTIVATE);
        }
    }
}

/// Shows a window filling the screen - WindowState = Maximized. Restored, it is its designed size,
/// where it was put. (Photographed for a test, it is shown as designed and not made active.)
pub fn show_maximized(h: HWND) {
    let quiet = std::env::var_os("PHAI_SHOT").is_some();
    unsafe {
        let _ = wm::ShowWindow(h, if quiet { wm::SW_SHOWNOACTIVATE } else { wm::SW_SHOWMAXIMIZED });
    }
}

/// Runs a modal window: `owner` takes no input until `dialog` is closed, then has it back and is
/// in front again - as `ShowDialog` does in WinForms. Returns when `dialog` is gone.
pub fn modal_loop(dialog: HWND, owner: HWND) {
    unsafe {
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow(owner, false);
        let mut msg = wm::MSG::default();
        while wm::IsWindow(Some(dialog)).as_bool() {
            if !wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
                // WM_QUIT belongs to the program's own loop: put it back for that one.
                wm::PostQuitMessage(msg.wParam.0 as i32);
                break;
            }
            if wm::IsDialogMessageW(dialog, &msg).as_bool() {
                continue;
            }
            let _ = wm::TranslateMessage(&msg);
            wm::DispatchMessageW(&msg);
        }
        let _ = windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow(owner, true);
        let _ = wm::SetForegroundWindow(owner);
    }
}
