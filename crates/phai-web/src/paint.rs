//! A control the program draws itself - WinForms' Paint event, in a browser. The panel or picture
//! gets a `<canvas>` filling it; the program's Paint method draws on it with the same calls as on
//! Windows (`e.clear`, `e.fill_rect`, `e.draw_text`, `e.draw_pixels` ...), in the control's own
//! pixels, its top left `0, 0`. It is drawn when the page shows, when the control changes size,
//! and after `ui.x.invalidate()`.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement};

use crate::win::{self, el, HWND};

/// What a Paint method draws on - WinForms' PaintEventArgs with its Graphics.
pub struct PaintEventArgs {
    cx: CanvasRenderingContext2d,
    font: String,
    /// The control's size in pixels.
    pub width: i32,
    pub height: i32,
}

impl PaintEventArgs {
    /// The whole control in one colour - `#2a2a2c`, `white`, any CSS colour.
    pub fn clear(&self, css: &str) {
        self.fill_rect(0, 0, self.width, self.height, css);
    }

    pub fn fill_rect(&self, x: i32, y: i32, w: i32, h: i32, css: &str) {
        self.cx.set_fill_style_str(css);
        self.cx.fill_rect(x as f64, y as f64, w as f64, h as f64);
    }

    /// A line one pixel wide.
    pub fn draw_line(&self, x1: i32, y1: i32, x2: i32, y2: i32, css: &str) {
        self.cx.set_stroke_style_str(css);
        self.cx.set_line_width(1.0);
        self.cx.begin_path();
        // Through the middle of the pixels, as Windows draws a one-pixel line.
        self.cx.move_to(x1 as f64 + 0.5, y1 as f64 + 0.5);
        self.cx.line_to(x2 as f64 + 0.5, y2 as f64 + 0.5);
        self.cx.stroke();
    }

    /// Words in the control's font, their top left at `x, y`; each line under the last.
    pub fn draw_text(&self, text: &str, x: i32, y: i32, css: &str) {
        self.words(css, "left");
        for (k, line) in text.lines().enumerate() {
            let _ = self.cx.fill_text(line, x as f64, (y + k as i32 * self.line_height()) as f64);
        }
    }

    /// Words centred in the control - "Connecting..." - their lines kept, long ones wrapped
    /// inside a margin, the block in the middle.
    pub fn draw_text_centered(&self, text: &str, css: &str) {
        if text.is_empty() {
            return;
        }
        self.words(css, "center");
        let margin = (self.width / 10).min(40);
        let room = (self.width - 2 * margin).max(1) as f64;
        let lines = wrap(text, room, |s| self.cx.measure_text(s).map(|m| m.width()).unwrap_or(0.0));
        let lh = self.line_height();
        let top = ((self.height - lh * lines.len() as i32) / 2).max(0);
        for (k, line) in lines.iter().enumerate() {
            let _ = self.cx.fill_text(line, self.width as f64 / 2.0, (top + k as i32 * lh) as f64);
        }
    }

    fn words(&self, css: &str, align: &str) {
        self.cx.set_font(&self.font);
        self.cx.set_fill_style_str(css);
        self.cx.set_text_align(align);
        self.cx.set_text_baseline("top");
    }

    fn line_height(&self) -> i32 {
        self.cx.measure_text("Mg").map(|m| (m.font_bounding_box_ascent() + m.font_bounding_box_descent()).ceil() as i32).unwrap_or(16).max(1)
    }

    /// A picture of `size` (width, height) pixels, 4 bytes each (blue, green, red, unused - what a
    /// screen capture gives), drawn into `place` (x, y, w, h).
    pub fn draw_pixels(&self, bgra: &[u8], size: (i32, i32), place: (i32, i32, i32, i32)) {
        self.picture(bgra, size, place, false);
    }

    /// As [`Self::draw_pixels`], the picture's own transparency kept (its fourth byte) - a mouse
    /// pointer, an icon over what is drawn already.
    pub fn draw_pixels_alpha(&self, bgra: &[u8], size: (i32, i32), place: (i32, i32, i32, i32)) {
        self.picture(bgra, size, place, true);
    }

    fn picture(&self, bgra: &[u8], (pw, ph): (i32, i32), (x, y, w, h): (i32, i32, i32, i32), alpha: bool) {
        if pw <= 0 || ph <= 0 || bgra.len() < (pw * ph * 4) as usize {
            return;
        }
        let rgba = to_rgba(&bgra[..(pw * ph * 4) as usize], alpha);
        let Ok(data) = web_sys::ImageData::new_with_u8_clamped_array_and_sh(Clamped(&rgba), pw as u32, ph as u32) else { return };
        // Drawn on a canvas of its own first: only a canvas can be shrunk or grown into the place.
        let Ok(c) = win::document().create_element("canvas") else { return };
        let c: HtmlCanvasElement = c.unchecked_into();
        c.set_width(pw as u32);
        c.set_height(ph as u32);
        let Some(pcx) = context(&c) else { return };
        let _ = pcx.put_image_data(&data, 0.0, 0.0);
        self.cx.set_image_smoothing_enabled(w < pw);
        let _ = self.cx.draw_image_with_html_canvas_element_and_dw_and_dh(&c, x as f64, y as f64, w as f64, h as f64);
    }

    /// Where a picture of `pw` x `ph` fits the control whole, its shape kept and centred:
    /// `(x, y, w, h)` - never larger than it is.
    pub fn fit(&self, pw: i32, ph: i32) -> (i32, i32, i32, i32) {
        fit(self.width, self.height, pw, ph)
    }
}

/// Where `pw` x `ph` fits `cw` x `ch` whole, its shape kept, centred, never grown.
pub fn fit(cw: i32, ch: i32, pw: i32, ph: i32) -> (i32, i32, i32, i32) {
    if pw <= 0 || ph <= 0 || cw <= 0 || ch <= 0 {
        return (0, 0, 0, 0);
    }
    let s = (cw as f64 / pw as f64).min(ch as f64 / ph as f64).min(1.0);
    let (w, h) = ((pw as f64 * s).round() as i32, (ph as f64 * s).round() as i32);
    ((cw - w) / 2, (ch - h) / 2, w, h)
}

/// Blue, green, red, unused as a browser's red, green, blue, alpha - opaque unless `alpha`.
fn to_rgba(bgra: &[u8], alpha: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(bgra.len());
    for &[b, g, r, a] in bgra.as_chunks::<4>().0 {
        out.extend_from_slice(&[r, g, b, if alpha { a } else { 255 }]);
    }
    out
}

/// `text`'s lines, each one broken at spaces where it is wider than `room`.
fn wrap(text: &str, room: f64, width: impl Fn(&str) -> f64) -> Vec<String> {
    let mut out = vec![];
    for line in text.lines() {
        let mut now = String::new();
        for word in line.split(' ') {
            let next = if now.is_empty() { word.to_string() } else { format!("{now} {word}") };
            if !now.is_empty() && width(&next) > room {
                out.push(std::mem::replace(&mut now, word.to_string()));
            } else {
                now = next;
            }
        }
        out.push(now);
    }
    out
}

fn context(c: &HtmlCanvasElement) -> Option<CanvasRenderingContext2d> {
    c.get_context("2d").ok().flatten()?.dyn_into().ok()
}

type Painter = Rc<RefCell<dyn FnMut(&PaintEventArgs)>>;

thread_local! {
    static PAINTERS: RefCell<HashMap<u32, Painter>> = RefCell::new(HashMap::new());
    /// Controls asked to draw again, before the browser's next frame.
    static ASKED: RefCell<HashSet<u32>> = RefCell::new(HashSet::new());
}

/// Control `h` drawn by `f`: a canvas over all of it, drawn now and whenever it changes size.
pub fn on_paint(h: HWND, f: impl FnMut(&PaintEventArgs) + 'static) {
    let Some(mut e) = el(h) else { return };
    // A picture is an image on the page, which holds nothing: a box in its place, with its
    // classes, id and place, to hold the canvas.
    if e.tag_name().eq_ignore_ascii_case("img") {
        let b = win::make("div");
        b.set_class_name(&e.class_name());
        b.set_id(&e.id());
        b.style().set_css_text(&e.style().css_text());
        win::replace(h, b.clone());
        e = b;
    }
    let c: HtmlCanvasElement = win::make("canvas").unchecked_into();
    c.set_class_name("phai-paint");
    c.style().set_css_text("position:absolute;left:0;top:0;width:100%;height:100%;display:block;pointer-events:none;");
    if e.style().get_property_value("position").ok().as_deref().unwrap_or("").is_empty() {
        let _ = e.style().set_property("position", "absolute");
    }
    let _ = e.append_child(&c);
    PAINTERS.with(|p| p.borrow_mut().insert(h.0, Rc::new(RefCell::new(f))));
    // Its size changes with the window: drawn again then.
    let me = h;
    let resized = Closure::<dyn FnMut()>::new(move || invalidate(me));
    let _ = win::window().add_event_listener_with_callback("resize", resized.as_ref().unchecked_ref());
    resized.forget();
    invalidate(h);
}

/// Control `h` drawn again before the browser's next frame - `Invalidate()`; nothing when its
/// Paint is not the program's.
pub fn invalidate(h: HWND) {
    if !PAINTERS.with(|p| p.borrow().contains_key(&h.0)) {
        return;
    }
    let first = ASKED.with(|a| {
        let mut a = a.borrow_mut();
        let first = a.is_empty();
        a.insert(h.0);
        first
    });
    if first {
        let run = Closure::once_into_js(|| {
            let asked: Vec<u32> = ASKED.with(|a| a.borrow_mut().drain().collect());
            for k in asked {
                paint_now(HWND(k));
            }
        });
        let _ = win::window().request_animation_frame(run.unchecked_ref());
    }
}

fn paint_now(h: HWND) {
    let Some(e) = el(h) else { return };
    let Some(painter) = PAINTERS.with(|p| p.borrow().get(&h.0).cloned()) else { return };
    let Ok(Some(c)) = e.query_selector(":scope > canvas.phai-paint") else { return };
    let c: HtmlCanvasElement = c.unchecked_into();
    let (w, hh) = (e.client_width().max(1), e.client_height().max(1));
    // As many canvas pixels as the screen has, so it is sharp on a phone; drawn in the
    // control's own pixels.
    let scale = win::window().device_pixel_ratio().max(1.0);
    c.set_width((w as f64 * scale).round() as u32);
    c.set_height((hh as f64 * scale).round() as u32);
    let Some(cx) = context(&c) else { return };
    let _ = cx.scale(scale, scale);
    let font = font_of(&e);
    let args = PaintEventArgs { cx, font, width: w, height: hh };
    // The control's own colour under whatever is drawn, as on Windows.
    if let Ok(Some(style)) = win::window().get_computed_style(&e) {
        if let Ok(bg) = style.get_property_value("background-color") {
            if !bg.is_empty() && bg != "rgba(0, 0, 0, 0)" {
                args.clear(&bg);
            }
        }
    }
    if let Ok(mut f) = painter.try_borrow_mut() {
        f(&args);
    };
}

/// The CSS font of element `e` - its size and family, as its words have.
fn font_of(e: &HtmlElement) -> String {
    let style = win::window().get_computed_style(e).ok().flatten();
    let get = |p: &str| style.as_ref().and_then(|s| s.get_property_value(p).ok()).filter(|v| !v.is_empty());
    format!("{} {} {}", get("font-weight").unwrap_or_else(|| "400".into()), get("font-size").unwrap_or_else(|| "14px".into()), get("font-family").unwrap_or_else(|| "sans-serif".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picture_fits_whole_and_centred_never_grown() {
        assert_eq!(fit(1000, 600, 1920, 1080), (0, 18, 1000, 563));
        assert_eq!(fit(2000, 2000, 800, 600), (600, 700, 800, 600));
    }

    #[test]
    fn screen_pixels_become_browser_pixels() {
        assert_eq!(to_rgba(&[1, 2, 3, 0], false), vec![3, 2, 1, 255]);
        assert_eq!(to_rgba(&[1, 2, 3, 9], true), vec![3, 2, 1, 9]);
    }

    #[test]
    fn long_lines_wrap_at_spaces() {
        let w = |s: &str| s.chars().count() as f64;
        assert_eq!(wrap("one two three", 7.0, w), vec!["one two", "three"]);
        assert_eq!(wrap("a\nb", 7.0, w), vec!["a", "b"]);
    }
}
