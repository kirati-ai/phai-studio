//! Add Connection in one window: the kinds of database as cards along the top - MySQL / MariaDB,
//! PostgreSQL, a SQLite file - and below, what the chosen one needs, changing as another card is
//! chosen, as tabs do. Test connection says whether it answers before OK keeps it.

use phai_data::{Connection, Kind};
use phai_ui::text::{t, tf};
use phai_ui::theme::{self, Color};
use phai_ui::win;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi;
use windows::Win32::UI::WindowsAndMessaging as wm;

const ID_OK: i32 = 1;
const ID_CANCEL: i32 = 2;
const ID_TEST: i32 = 3;
const ID_BROWSE: i32 = 4;
const ID_CARDS: i32 = 10;

/// The kinds, in the cards' order.
const KINDS: [Kind; 3] = [Kind::MySql, Kind::Postgres, Kind::Sqlite];

fn card_title(k: usize) -> &'static str {
    ["MySQL / MariaDB", "PostgreSQL", "SQLite"][k]
}

fn card_words(k: usize) -> &'static str {
    [t("A database server - port 3306"), t("A database server - port 5432"), t("A file on this computer, no server")][k]
}

/// The badge on a card: its letters and colour.
fn badge(k: usize) -> (&'static str, Color) {
    [("My", Color::rgb(0x00, 0x75, 0x8f)), ("Pg", Color::rgb(0x33, 0x67, 0x91)), ("Lite", Color::rgb(0x0f, 0x80, 0xcc))][k]
}

/// Design pixels.
const W: i32 = 640;
const M: i32 = 16;
const CARD_H: i32 = 76;
const ROW: i32 = 26;
const GAP: i32 = 10;

struct Fields {
    host: HWND,
    port: HWND,
    user: HWND,
    password: HWND,
    database: HWND,
    ssl: HWND,
    trust: HWND,
    file: HWND,
    browse: HWND,
    file_note: HWND,
    labels: Vec<(HWND, bool)>,
}

struct Dialog {
    cards: HWND,
    kind: usize,
    f: Fields,
    say: HWND,
    chosen: Option<Connection>,
    done: bool,
}

fn state<'a>(h: HWND) -> Option<&'a mut Dialog> {
    unsafe { win::state::<Dialog>(h) }
}

impl Dialog {
    /// The connection as the boxes say.
    fn read(&self) -> Connection {
        let kind = KINDS[self.kind];
        if kind == Kind::Sqlite {
            return Connection { host: win::text_of(self.f.file).trim().trim_matches('"').to_string(), kind, ..Default::default() };
        }
        Connection {
            host: win::text_of(self.f.host).trim().into(),
            port: win::text_of(self.f.port).trim().parse().unwrap_or(kind.default_port()),
            user: win::text_of(self.f.user).trim().into(),
            password: win::text_of(self.f.password),
            database: win::text_of(self.f.database).trim().into(),
            ssl: win::is_checked(self.f.ssl),
            trust_cert: win::is_checked(self.f.trust),
            kind,
        }
    }
}

/// Shows the window over `owner`, on `start` when a connection is being changed; the connection
/// chosen, or `None` on Cancel. (Kept by the caller once it answers.)
pub fn choose(owner: HWND, start: Option<&Connection>) -> Option<Connection> {
    let hwnd = make(owner, start);
    unsafe {
        win::enable(owner, false);
        win::show_top(hwnd);
        let mut msg = wm::MSG::default();
        while state(hwnd).is_some_and(|d| !d.done) {
            if !wm::GetMessageW(&mut msg, None, 0, 0).as_bool() {
                wm::PostQuitMessage(msg.wParam.0 as i32);
                break;
            }
            if !wm::IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = wm::TranslateMessage(&msg);
                wm::DispatchMessageW(&msg);
            }
        }
        let chosen = state(hwnd).and_then(|d| d.chosen.take());
        win::enable(owner, true);
        let _ = wm::SetForegroundWindow(owner);
        let _ = wm::DestroyWindow(hwnd);
        chosen
    }
}

/// The window, filled, not yet shown - for [`choose`], and for a picture in a test.
pub fn make(owner: HWND, start: Option<&Connection>) -> HWND {
    win::init();
    let class = win::register("PhaiAddConnection", Some(proc_), None);
    let cards_class = win::register("PhaiConnectionCards", Some(cards_proc), None);
    let title = win::wide(&if start.is_some() { t("Change Connection").to_string() } else { t("Add Connection").to_string() });
    unsafe {
        let hwnd = wm::CreateWindowExW(wm::WS_EX_DLGMODALFRAME, PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), wm::WS_POPUP | wm::WS_CAPTION | wm::WS_SYSMENU | wm::WS_CLIPCHILDREN, 0, 0, 100, 100, Some(owner), None, Some(win::instance()), None).unwrap_or_default();
        let font = win::font(hwnd, 0, false);
        let px = |v| win::px(hwnd, v);
        let at = |h: HWND, x: i32, y: i32, w: i32, ht: i32| win::move_to(h, px(x), px(y), px(w), px(ht));
        let pick = win::label(hwnd, t("Which database?"), 0, win::font(hwnd, 0, true));
        at(pick, M, M, W - 2 * M, 22);
        let cards = wm::CreateWindowExW(Default::default(), PCWSTR(cards_class.as_ptr()), PCWSTR::null(), wm::WS_CHILD | wm::WS_VISIBLE | wm::WS_TABSTOP, 0, 0, 10, 10, Some(hwnd), Some(wm::HMENU(ID_CARDS as isize as *mut _)), Some(win::instance()), None).unwrap_or_default();
        at(cards, M, M + 26, W - 2 * M, CARD_H);
        // The fields: a server's, or a file's - laid out on the same rows.
        let top = M + 26 + CARD_H + 18;
        let (lx, lw, fx) = (M, 150, M + 160);
        let fw = W - M - fx;
        let mut labels = Vec::new();
        let row_y = |i: i32| top + i * (ROW + GAP);
        let mut label = |text: &str, i: i32, server: bool| {
            let h = win::label(hwnd, text, 0, font);
            at(h, lx, row_y(i), lw, ROW);
            labels.push((h, server));
        };
        label(t("Server"), 0, true);
        label(t("Port"), 1, true);
        label(t("User"), 2, true);
        label(t("Password"), 3, true);
        label(t("Database"), 4, true);
        label(t("File"), 0, false);
        let c = start.cloned().unwrap_or_default();
        let field = |text: &str, i: i32, w: i32, password: bool| {
            let h = win::edit(hwnd, text, 0, font, password);
            at(h, fx, row_y(i), w, ROW);
            h
        };
        let host = field(if c.host.is_empty() { "localhost" } else { &c.host }, 0, fw, false);
        let port = field(&if c.port == 0 { String::new() } else { c.port.to_string() }, 1, 120, false);
        let user = field(&c.user, 2, fw, false);
        let password = field(&c.password, 3, fw, true);
        let database = field(&c.database, 4, fw, false);
        phai_ui::props::placeholder(database, t("Empty: none to start in"));
        let ssl = win::checkbox(hwnd, t("Encrypt the connection (TLS)"), 0, font, c.ssl);
        at(ssl, fx, row_y(5), fw, ROW);
        let trust = win::checkbox(hwnd, t("Trust the server's own certificate"), 0, font, c.trust_cert);
        at(trust, fx, row_y(6), fw, ROW);
        let file = field(if c.kind == Kind::Sqlite { &c.host } else { "" }, 0, fw - 110, false);
        phai_ui::props::placeholder(file, t("A .db file - a new name makes a new database"));
        let browse = win::button(hwnd, t("Browse..."), ID_BROWSE, font, false);
        at(browse, W - M - 100, row_y(0), 100, ROW);
        let file_note = win::label(hwnd, t("SQLite keeps the whole database in this one file. Nothing to install, no password."), 0, font);
        win::set_css(file_note, "muted", "");
        at(file_note, fx, row_y(1), fw, ROW * 2);
        // Below: Test, what it said, OK, Cancel.
        // What Test connection said: a line of its own, the whole width, above the buttons.
        let foot = row_y(7) + 34;
        let test = win::button(hwnd, t("Test connection"), ID_TEST, font, false);
        phai_ui::props::tooltip(test, t("Connect once, to see that it answers"));
        at(test, M, foot, 150, 30);
        let say = win::label(hwnd, "", 0, font);
        at(say, M, row_y(7) + 2, W - 2 * M, 24);
        let ok = win::button(hwnd, t("OK"), ID_OK, font, true);
        at(ok, W - M - 200, foot, 96, 30);
        let cancel = win::button(hwnd, t("Cancel"), ID_CANCEL, font, false);
        at(cancel, W - M - 96, foot, 96, 30);
        let kind = KINDS.iter().position(|k| *k == c.kind).filter(|_| start.is_some()).unwrap_or(0);
        let f = Fields { host, port, user, password, database, ssl, trust, file, browse, file_note, labels };
        win::set_state(hwnd, Box::new(Dialog { cards, kind: usize::MAX, f, say, chosen: None, done: false }));
        set_kind(hwnd, kind);
        let mut r = RECT { left: 0, top: 0, right: px(W), bottom: px(foot + 30 + M) };
        let style = wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32;
        let exs = wm::GetWindowLongW(hwnd, wm::GWL_EXSTYLE) as u32;
        let _ = windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi(&mut r, wm::WINDOW_STYLE(style), false, wm::WINDOW_EX_STYLE(exs), win::dpi(hwnd) as u32);
        let _ = wm::SetWindowPos(hwnd, None, 0, 0, r.right - r.left, r.bottom - r.top, wm::SWP_NOMOVE | wm::SWP_NOZORDER);
        win::center_on(hwnd, owner);
        win::theme_frame(hwnd);
        hwnd
    }
}

/// For a picture: card `k` chosen in a window from [`make`].
pub fn show_kind(h: HWND, k: usize) {
    set_kind(h, k.min(KINDS.len() - 1));
}

/// What a failed connection means, in words a person can act on: nothing answering at the
/// address (os error 10061) is the server not running there, or another port.
pub fn said_of(c: &Connection, e: &str) -> String {
    if e.contains("10061") || e.contains("actively refused") || e.contains("Connection refused") {
        return tf("Nothing answers at {0}:{1} - is {2} running there, on that port?", &[&c.host, &c.port, &c.kind.name()]);
    }
    if e.contains("11001") || e.contains("No such host") {
        return tf("No computer is named {0} - check the Server box.", &[&c.host]);
    }
    e.to_string()
}

/// For checking: Test connection pressed.
pub fn press_test(h: HWND) {
    unsafe {
        wm::SendMessageW(h, wm::WM_COMMAND, Some(WPARAM(ID_TEST as usize)), None);
    }
}

/// For checking: a click in the middle of card `k`, sent as the mouse sends it.
pub fn click_card(h: HWND, k: usize) {
    let Some(cards) = state(h).map(|d| d.cards) else { return };
    let mut r = RECT::default();
    unsafe {
        let _ = wm::GetClientRect(cards, &mut r);
        let cw = r.right / KINDS.len() as i32;
        let x = cw * k as i32 + cw / 2;
        wm::SendMessageW(cards, wm::WM_LBUTTONDOWN, Some(WPARAM(1)), Some(LPARAM(((r.bottom / 2) << 16 | x) as isize)));
        wm::SendMessageW(cards, wm::WM_LBUTTONUP, Some(WPARAM(0)), Some(LPARAM(((r.bottom / 2) << 16 | x) as isize)));
    }
}

/// Card `k` chosen: its fields shown, a server's user and port as its kind usually has them
/// unless they were changed.
fn set_kind(h: HWND, k: usize) {
    let Some(d) = state(h) else { return };
    let was = d.kind;
    d.kind = k;
    let file = KINDS[k] == Kind::Sqlite;
    for (l, server) in &d.f.labels {
        win::show(*l, *server != file);
    }
    for c in [d.f.host, d.f.port, d.f.user, d.f.password, d.f.database, d.f.ssl, d.f.trust] {
        win::show(c, !file);
    }
    for c in [d.f.file, d.f.browse, d.f.file_note] {
        win::show(c, file);
    }
    // A port and user as some database has them by default (or none) follow the card; ones the
    // person typed stay. Compared with every server's defaults, not only the card before - that
    // may be SQLite, which has neither.
    if !file && was != k {
        let (port, user) = (win::text_of(d.f.port), win::text_of(d.f.user));
        let ports = [Kind::MySql.default_port().to_string(), Kind::Postgres.default_port().to_string()];
        if port.trim().is_empty() || ports.iter().any(|p| p == port.trim()) {
            win::set_text(d.f.port, &KINDS[k].default_port().to_string());
        }
        if user.trim().is_empty() || ["root", "postgres"].contains(&user.trim()) {
            win::set_text(d.f.user, if KINDS[k] == Kind::Postgres { "postgres" } else { "root" });
        }
    }
    win::set_text(d.say, "");
    unsafe {
        let _ = Gdi::InvalidateRect(Some(d.cards), None, false);
    }
}

fn cards_paint(h: HWND) {
    let Some(d) = (unsafe { wm::GetParent(h) }).ok().and_then(state) else { return };
    let cur = theme::current();
    unsafe {
        let mut ps = Gdi::PAINTSTRUCT::default();
        let dc = Gdi::BeginPaint(h, &mut ps);
        let mut r = RECT::default();
        let _ = wm::GetClientRect(h, &mut r);
        let surface = cur.var_color("--surface").unwrap_or(Color::rgb(255, 255, 255));
        win::fill(dc, &r, surface);
        let px = |v| win::px(h, v);
        let n = KINDS.len() as i32;
        let gap = px(12);
        let cw = (r.right - (n - 1) * gap) / n;
        let accent = cur.var_color("--accent").unwrap_or(Color::rgb(0x12, 0x85, 0x5f));
        let tint = cur.var_color("--accent-tint").unwrap_or(Color::rgb(0xe7, 0xf3, 0xef));
        let border = cur.var_color("--border").unwrap_or(Color::rgb(0xd4, 0xd4, 0xd4));
        let text = cur.var_color("--text").unwrap_or(Color::rgb(0x1b, 0x1b, 0x1b));
        let muted = cur.var_color("--muted").unwrap_or(Color::rgb(0x5f, 0x5f, 0x5f));
        let bold = win::font(h, 0, true);
        let font = win::font(h, 0, false);
        Gdi::SetBkMode(dc, Gdi::TRANSPARENT);
        for k in 0..KINDS.len() {
            let x = k as i32 * (cw + gap);
            let card = RECT { left: x, top: 0, right: x + cw, bottom: r.bottom };
            let on = d.kind == k;
            // The chosen one: framed in the accent, on the light accent.
            win::fill(dc, &card, if on { accent } else { border });
            let edge = px(if on { 2 } else { 1 });
            let inner = RECT { left: card.left + edge, top: card.top + edge, right: card.right - edge, bottom: card.bottom - edge };
            win::fill(dc, &inner, if on { tint } else { surface });
            // Its badge.
            let (letters, colour) = badge(k);
            let b = RECT { left: inner.left + px(10), top: inner.top + px(12), right: inner.left + px(48), bottom: inner.top + px(50) };
            win::fill(dc, &b, colour);
            let old = Gdi::SelectObject(dc, bold.into());
            Gdi::SetTextColor(dc, COLORREF(Color::rgb(255, 255, 255).colorref()));
            let mut br = b;
            let mut w: Vec<u16> = letters.encode_utf16().collect();
            Gdi::DrawTextW(dc, &mut w, &mut br, Gdi::DT_CENTER | Gdi::DT_VCENTER | Gdi::DT_SINGLELINE);
            // Its name, and what it is.
            Gdi::SetTextColor(dc, COLORREF(text.colorref()));
            let mut tr = RECT { left: b.right + px(10), top: inner.top + px(10), right: inner.right - px(6), bottom: inner.top + px(32) };
            let mut w: Vec<u16> = card_title(k).encode_utf16().collect();
            Gdi::DrawTextW(dc, &mut w, &mut tr, Gdi::DT_LEFT | Gdi::DT_SINGLELINE | Gdi::DT_END_ELLIPSIS);
            Gdi::SelectObject(dc, font.into());
            Gdi::SetTextColor(dc, COLORREF(muted.colorref()));
            let mut wr = RECT { left: b.right + px(10), top: inner.top + px(32), right: inner.right - px(6), bottom: inner.bottom - px(4) };
            let mut w: Vec<u16> = card_words(k).encode_utf16().collect();
            Gdi::DrawTextW(dc, &mut w, &mut wr, Gdi::DT_LEFT | Gdi::DT_WORDBREAK | Gdi::DT_END_ELLIPSIS);
            Gdi::SelectObject(dc, old);
        }
        let _ = Gdi::EndPaint(h, &ps);
    }
}

extern "system" fn cards_proc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        wm::WM_PAINT => {
            cards_paint(h);
            LRESULT(0)
        }
        wm::WM_ERASEBKGND => LRESULT(1),
        wm::WM_LBUTTONDOWN => {
            let (x, _) = win::point_of(l);
            let mut r = RECT::default();
            unsafe {
                let _ = wm::GetClientRect(h, &mut r);
            }
            let n = KINDS.len() as i32;
            let gap = win::px(h, 12);
            let cw = (r.right - (n - 1) * gap) / n;
            let k = (x / (cw + gap)) as usize;
            if k < KINDS.len() && x % (cw + gap) < cw {
                if let Ok(p) = unsafe { wm::GetParent(h) } {
                    set_kind(p, k);
                }
            }
            LRESULT(0)
        }
        // The keyboard: left and right move between the cards.
        wm::WM_GETDLGCODE => LRESULT(0x0001 /* DLGC_WANTARROWS */),
        wm::WM_KEYDOWN if w.0 == 0x25 || w.0 == 0x27 => {
            if let Some((p, d)) = unsafe { wm::GetParent(h) }.ok().and_then(|p| state(p).map(|d| (p, d.kind))) {
                let k = if w.0 == 0x25 { d.saturating_sub(1) } else { (d + 1).min(KINDS.len() - 1) };
                set_kind(p, k);
            }
            LRESULT(0)
        }
        wm::WM_SETCURSOR => {
            unsafe {
                let _ = wm::SetCursor(wm::LoadCursorW(None, wm::IDC_HAND).ok());
            }
            LRESULT(1)
        }
        _ => win::def(h, msg, w, l),
    }
}

extern "system" fn proc_(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let Some(d) = state(h) else { return win::def(h, msg, w, l) };
    match msg {
        wm::WM_COMMAND => {
            match win::loword(w.0) as i32 {
                ID_OK => {
                    let c = d.read();
                    if c.host.is_empty() {
                        win::set_text(d.say, if c.kind == Kind::Sqlite { t("Choose or name a file first") } else { t("Name the server first") });
                    } else {
                        d.chosen = Some(c);
                        d.done = true;
                    }
                }
                ID_CANCEL => d.done = true,
                ID_BROWSE => {
                    let start = win::text_of(d.f.file);
                    if let Some(p) = crate::file_dialog_pub(h, true, "SQLite (*.db; *.sqlite; *.sqlite3)", "*.db;*.sqlite;*.sqlite3", if start.trim().is_empty() { "data.db" } else { start.trim() }) {
                        win::set_text(d.f.file, &p.to_string_lossy());
                    }
                }
                ID_TEST => {
                    let c = d.read();
                    let probe_words = c.clone();
                    let name = c.kind.name();
                    match phai_ui::dialogs::wait_for(Some(h), &tf("Add Connection - {0}", &[&name]), t("Connecting..."), move |_| c.test()) {
                        Some(Ok(version)) => win::set_text(d.say, &tf("It answers: {0} {1}", &[&name, &version])),
                        Some(Err(e)) => win::set_text(d.say, &said_of(&probe_words, &e)),
                        None => {}
                    }
                }
                _ => return win::def(h, msg, w, l),
            }
            LRESULT(0)
        }
        wm::WM_CLOSE => {
            d.done = true;
            LRESULT(0)
        }
        wm::WM_NCDESTROY => {
            drop(unsafe { win::take_state::<Dialog>(h) });
            win::def(h, msg, w, l)
        }
        _ => win::def(h, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_refused_connection_says_what_to_check() {
        let c = Connection { host: "localhost".into(), port: 5432, kind: Kind::Postgres, ..Default::default() };
        let said = said_of(&c, "No connection could be made because the target machine actively refused it. (os error 10061)");
        assert!(said.contains("localhost:5432") && said.contains("PostgreSQL"), "{said}");
        assert_eq!(said_of(&c, "password authentication failed"), "password authentication failed", "other words as they are");
    }
}
