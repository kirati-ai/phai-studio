//! A grid bound to a database table, on the web: a page cannot reach a database server, so it
//! asks the program's data server (`data-server/` in the project, [`phai_data::server`]) at
//! `phai-data/` beside the page - or where [`set_url`] says. The same `reload_<grid>` and
//! `save_<grid>` as on Windows; here they start the work and return at once, the grid filled (or
//! a message shown) when the answer comes.
//!
//! The access key the server asks for is asked of the person once and kept in this browser.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};

use crate::text::{t, tf};
use crate::win::{self, HWND};

thread_local! {
    static URL: RefCell<String> = RefCell::new("phai-data/".into());
    /// Each bound grid's rows as they were read, to tell the server what changed.
    static LOADED: RefCell<HashMap<u32, Vec<Vec<String>>>> = RefCell::new(HashMap::new());
}

const KEY: &str = "phai-data-key";

/// Where the data server answers - `https://data.example.com/phai-data/`; `phai-data/` beside
/// the page when not set.
pub fn set_url(url: &str) {
    let mut u = url.trim().to_string();
    if !u.ends_with('/') {
        u.push('/');
    }
    URL.with(|x| *x.borrow_mut() = u);
}

/// The server's address: [`set_url`]'s, else the page's `<meta name="phai-data">` (Phai Studio
/// writes it from Cargo.toml's `data-url`), else `phai-data/` beside the page.
fn base_url() -> String {
    let meta = win::document().query_selector("meta[name=phai-data]").ok().flatten().and_then(|m| m.get_attribute("content")).filter(|c| !c.trim().is_empty());
    match meta {
        Some(m) if URL.with(|u| u.borrow().as_str() == "phai-data/") => {
            let m = m.trim().to_string();
            if m.ends_with('/') { m } else { m + "/" }
        }
        _ => URL.with(|u| u.borrow().clone()),
    }
}

fn key() -> String {
    win::window().local_storage().ok().flatten().and_then(|s| s.get_item(KEY).ok().flatten()).unwrap_or_default()
}

fn set_key(k: &str) {
    if let Some(s) = win::window().local_storage().ok().flatten() {
        let _ = s.set_item(KEY, k);
    }
}

/// A string as JSON writes it.
fn quote(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn row_json(r: &[String]) -> String {
    format!("[{}]", r.iter().map(|c| quote(c)).collect::<Vec<_>>().join(","))
}

type Done = Box<dyn FnOnce(Result<JsValue, String>)>;

/// POSTs `body` to `what` (`load`, `save`) and gives `done` the answer's JSON - asking for the
/// access key, and trying once more, when the server says it is missing or wrong.
fn post(what: &'static str, body: String, asked: bool, done: Done) {
    let url = base_url() + what;
    let init = web_sys::RequestInit::new();
    init.set_method("POST");
    init.set_body(&JsValue::from_str(&body));
    if let Ok(h) = web_sys::Headers::new() {
        let _ = h.set("Content-Type", "application/json");
        let k = key();
        if !k.is_empty() {
            let _ = h.set("Authorization", &format!("Bearer {k}"));
        }
        init.set_headers(&h);
    }
    let done: Rc<RefCell<Option<Done>>> = Rc::new(RefCell::new(Some(done)));
    let (d1, d2) = (done.clone(), done.clone());
    let on_response = Closure::once(move |v: JsValue| {
        let r: web_sys::Response = v.unchecked_into();
        let status = r.status();
        let Ok(text) = r.text() else { return };
        let on_text = Closure::once(move |v: JsValue| {
            let text = v.as_string().unwrap_or_default();
            if status == 401 && !asked {
                let typed = win::window().prompt_with_message(t("The access key of the program's data:")).ok().flatten();
                if let Some(k) = typed.filter(|k| !k.trim().is_empty()) {
                    set_key(k.trim());
                    if let Some(f) = d1.borrow_mut().take() {
                        post(what, body, true, f);
                    }
                    return;
                }
            }
            let Some(f) = d1.borrow_mut().take() else { return };
            let json = js_sys::JSON::parse(&text).ok();
            let err = json.as_ref().and_then(|j| js_sys::Reflect::get(j, &JsValue::from_str("error")).ok()).and_then(|e| e.as_string());
            match (status, json, err) {
                (200, Some(j), None) => f(Ok(j)),
                (_, _, Some(e)) => f(Err(e)),
                _ => f(Err(tf("The data server answered {0}", &[&status]))),
            }
        });
        let _ = text.then(&on_text);
        on_text.forget();
    });
    let on_fail = Closure::once(move |_: JsValue| {
        if let Some(f) = d2.borrow_mut().take() {
            f(Err(t("The data server cannot be reached.").to_string()));
        }
    });
    let _ = win::window().fetch_with_str_and_init(&url, &init).then2(&on_response, &on_fail);
    on_response.forget();
    on_fail.forget();
}

fn rows_of(j: &JsValue) -> Vec<Vec<String>> {
    let Ok(rows) = js_sys::Reflect::get(j, &JsValue::from_str("rows")) else { return vec![] };
    js_sys::Array::from(&rows).iter().map(|r| js_sys::Array::from(&r).iter().map(|c| c.as_string().unwrap_or_default()).collect()).collect()
}

/// A grid's table on the data server, by the name the server gives it (`main_window/customers`).
#[derive(Clone, Debug)]
pub struct Binding {
    pub name: String,
}

impl Binding {
    pub fn new(name: &str) -> Binding {
        Binding { name: name.into() }
    }

    /// The table's rows read into `grid`, each tagged with its place; a message on `window` when
    /// it cannot be.
    pub fn load_into(&self, grid: HWND) {
        let body = format!("{{\"table\":{}}}", quote(&self.name));
        post(
            "load",
            body,
            false,
            Box::new(move |r| match r {
                Ok(j) => {
                    let rows = rows_of(&j);
                    crate::ctl::fill_tagged(grid, &rows);
                    LOADED.with(|l| l.borrow_mut().insert(grid.0, rows));
                }
                Err(e) => win::message(&e),
            }),
        );
    }

    /// What `grid` holds written back - changed cells, new rows, rows taken out - then read again.
    pub fn save_from(&self, grid: HWND) {
        let loaded = LOADED.with(|l| l.borrow().get(&grid.0).cloned().unwrap_or_default());
        let rows = crate::ctl::tagged_rows(grid);
        let body = format!(
            "{{\"table\":{},\"loaded\":[{}],\"rows\":[{}]}}",
            quote(&self.name),
            loaded.iter().map(|r| row_json(r)).collect::<Vec<_>>().join(","),
            rows.iter().map(|(tag, r)| format!("[{tag},{}]", row_json(r))).collect::<Vec<_>>().join(",")
        );
        let me = self.clone();
        post(
            "save",
            body,
            false,
            Box::new(move |r| match r {
                Ok(_) => me.load_into(grid),
                Err(e) => win::message(&crate::text::lines(&e)),
            }),
        );
    }
}
