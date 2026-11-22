//! Import WinForms: a `Form1.Designer.cs` read into a form file, so a screen made in Visual Studio
//! opens in Phai Studio looking as it did, to go on with by drag and drop. Only the screen comes
//! over - the C# in `Form1.cs` is not turned into Rust - and what has no place in a form file is
//! named in the warnings rather than dropped in silence.
//!
//! Not a C# parser: `InitializeComponent` is a list of simple statements - `x = new Button();`,
//! `x.Text = "OK";`, `x.Click += Ok_Click;`, `Controls.Add(x);` - read as such, old style
//! (`this.x`, full type names, casts) and new (short names) alike.

use crate::model::{anchor_text, Anchors, Control, Form, FormFile, HasProps, Kind, Menu, MenuItem, Props, Tool, FORMAT};
use crate::props::{self, Ty};
use std::collections::{BTreeSet, HashMap, HashSet};

/// A form read from WinForms, and what did not come over.
pub struct Imported {
    pub form: FormFile,
    pub warnings: Vec<String>,
}

/// Reads a `.Designer.cs`. An error only when it has no `InitializeComponent` to read.
pub fn import(designer_cs: &str) -> Result<Imported, String> {
    let toks = lex(designer_cs);
    let class = toks.windows(2).find_map(|w| match (&w[0], &w[1]) {
        (Tok::Id(c), Tok::Id(n)) if c == "class" => Some(n.clone()),
        _ => None,
    });
    let body = init_body(&toks).ok_or("no InitializeComponent() in it - is it a .Designer.cs of a form?")?;
    let mut r = Reader::default();
    for st in split_statements(body) {
        if let Some(s) = statement(st) {
            r.take(s);
        }
    }
    // Visual Studio before .NET Core wrote `this.` everywhere, and its forms' letters were
    // Microsoft Sans Serif 8.25 pt rather than Segoe UI 9.
    let framework = designer_cs.contains("this.ClientSize") || designer_cs.contains("this.AutoScaleDimensions");
    let mut b = Build::new(&r, class.as_deref().unwrap_or("Form1"), framework);
    b.run();
    Ok(Imported { form: b.form, warnings: b.warn })
}

// ---- reading the C# ----

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Id(String),
    Num(f64),
    Str(String),
    P(&'static str),
}

const PUNCT: &[&str] = &["+=", "-=", "==", "!=", "=>", "&&", "||", "<=", ">=", "::", "(", ")", "{", "}", "[", "]", ".", ",", ";", "=", "|", "+", "-", "*", "/", "<", ">", "!", "?", ":", "&", "~", "^", "%"];

fn lex(src: &str) -> Vec<Tok> {
    let c: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    let mut line_start = true;
    while i < c.len() {
        let ch = c[i];
        if ch == '\n' {
            line_start = true;
            i += 1;
            continue;
        }
        if ch.is_whitespace() {
            i += 1;
            continue;
        }
        // `#region`, `#nullable` - a line of its own.
        if ch == '#' && line_start {
            while i < c.len() && c[i] != '\n' {
                i += 1;
            }
            continue;
        }
        line_start = false;
        if ch == '/' && c.get(i + 1) == Some(&'/') {
            while i < c.len() && c[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if ch == '/' && c.get(i + 1) == Some(&'*') {
            i += 2;
            while i + 1 < c.len() && !(c[i] == '*' && c[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            continue;
        }
        if ch == '@' && c.get(i + 1) == Some(&'"') {
            // A verbatim string: "" is a quote, nothing else escapes.
            i += 2;
            let mut s = String::new();
            while i < c.len() {
                if c[i] == '"' {
                    if c.get(i + 1) == Some(&'"') {
                        s.push('"');
                        i += 2;
                        continue;
                    }
                    i += 1;
                    break;
                }
                s.push(c[i]);
                i += 1;
            }
            out.push(Tok::Str(s));
            continue;
        }
        if ch == '"' || ch == '\'' {
            let q = ch;
            i += 1;
            let mut s = String::new();
            while i < c.len() && c[i] != q {
                if c[i] == '\\' && i + 1 < c.len() {
                    i += 1;
                    match c[i] {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '0' => s.push('\0'),
                        'u' => {
                            let hex: String = c[i + 1..(i + 5).min(c.len())].iter().collect();
                            if let Some(x) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                                s.push(x);
                                i += 4;
                            }
                        }
                        other => s.push(other),
                    }
                } else {
                    s.push(c[i]);
                }
                i += 1;
            }
            i += 1;
            out.push(Tok::Str(s));
            continue;
        }
        if ch.is_ascii_digit() || (ch == '.' && c.get(i + 1).is_some_and(|d| d.is_ascii_digit())) {
            let start = i;
            if ch == '0' && matches!(c.get(i + 1), Some('x' | 'X')) {
                i += 2;
                while i < c.len() && c[i].is_ascii_hexdigit() {
                    i += 1;
                }
                let hex: String = c[start + 2..i].iter().collect();
                out.push(Tok::Num(i64::from_str_radix(&hex, 16).unwrap_or(0) as f64));
            } else {
                while i < c.len() && (c[i].is_ascii_digit() || c[i] == '.' || ((c[i] == 'e' || c[i] == 'E') && c.get(i + 1).is_some_and(|d| d.is_ascii_digit() || *d == '-' || *d == '+'))) {
                    if c[i] == 'e' || c[i] == 'E' {
                        i += 1;
                    }
                    i += 1;
                }
                let text: String = c[start..i].iter().collect();
                out.push(Tok::Num(text.parse().unwrap_or(0.0)));
            }
            while i < c.len() && c[i].is_ascii_alphabetic() {
                i += 1; // 10F, 3.5D, 2M, 7L
            }
            continue;
        }
        if ch.is_alphabetic() || ch == '_' || ch == '@' {
            let start = if ch == '@' { i + 1 } else { i };
            i += 1;
            while i < c.len() && (c[i].is_alphanumeric() || c[i] == '_') {
                i += 1;
            }
            out.push(Tok::Id(c[start..i].iter().collect()));
            continue;
        }
        let rest: String = c[i..(i + 2).min(c.len())].iter().collect();
        match PUNCT.iter().find(|p| rest.starts_with(**p)) {
            Some(p) => {
                out.push(Tok::P(p));
                i += p.chars().count();
            }
            None => i += 1,
        }
    }
    out
}

/// The tokens between `InitializeComponent() {` and its `}`.
fn init_body(t: &[Tok]) -> Option<&[Tok]> {
    let at = t.windows(4).position(|w| w[0] == Tok::Id("InitializeComponent".into()) && w[1] == Tok::P("(") && w[2] == Tok::P(")") && w[3] == Tok::P("{"))?;
    let start = at + 4;
    let mut depth = 1;
    for (i, tok) in t.iter().enumerate().skip(start) {
        match tok {
            Tok::P("{") => depth += 1,
            Tok::P("}") => {
                depth -= 1;
                if depth == 0 {
                    return Some(&t[start..i]);
                }
            }
            _ => {}
        }
    }
    None
}

fn split_statements(t: &[Tok]) -> Vec<&[Tok]> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0i32, 0);
    for (i, tok) in t.iter().enumerate() {
        match tok {
            Tok::P("(" | "{" | "[") => depth += 1,
            Tok::P(")" | "}" | "]") => depth -= 1,
            Tok::P(";") if depth == 0 => {
                if i > start {
                    out.push(&t[start..i]);
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    out
}

/// A C# value, as far as a designer file needs.
#[derive(Clone, Debug, PartialEq)]
enum E {
    Str(String),
    Num(f64),
    Bool(bool),
    /// `a.b.c`, `this.` taken off.
    Path(Vec<String>),
    Call(Vec<String>, Vec<E>),
    /// `new T(args) { items }`.
    New(String, Vec<E>, Vec<E>),
    /// `A | B`.
    Or(Vec<E>),
    /// `"a" + "b"`, `1 + 2`.
    Add(Vec<E>),
    Neg(Box<E>),
    Other,
}

enum Stmt {
    Assign(Vec<String>, E),
    Event(Vec<String>, E),
    Call(Vec<String>, Vec<E>),
}

struct P<'a> {
    t: &'a [Tok],
    i: usize,
}

impl P<'_> {
    fn peek(&self) -> Option<&Tok> {
        self.t.get(self.i)
    }
    fn at(&self, p: &str) -> bool {
        matches!(self.peek(), Some(Tok::P(x)) if *x == p)
    }
    fn eat(&mut self, p: &str) -> bool {
        if self.at(p) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn id(&mut self) -> Option<String> {
        match self.peek() {
            Some(Tok::Id(s)) => {
                let s = s.clone();
                self.i += 1;
                Some(s)
            }
            _ => None,
        }
    }

    fn expr(&mut self) -> E {
        let mut parts = vec![self.add()];
        while self.eat("|") {
            parts.push(self.add());
        }
        if parts.len() == 1 { parts.pop().unwrap() } else { E::Or(parts) }
    }

    fn add(&mut self) -> E {
        let mut parts = vec![self.unary()];
        loop {
            if self.eat("+") {
                parts.push(self.unary());
            } else if self.eat("-") {
                parts.push(E::Neg(Box::new(self.unary())));
            } else {
                break;
            }
        }
        if parts.len() == 1 { parts.pop().unwrap() } else { E::Add(parts) }
    }

    fn unary(&mut self) -> E {
        if self.eat("-") {
            return E::Neg(Box::new(self.unary()));
        }
        if self.eat("!") {
            return match self.unary() {
                E::Bool(b) => E::Bool(!b),
                _ => E::Other,
            };
        }
        if self.eat("~") {
            self.unary();
            return E::Other;
        }
        self.postfix()
    }

    /// A type's name after `new` or in a cast: `System.Windows.Forms.Button`, `List<int>`, `int[]`.
    fn type_name(&mut self) -> Option<String> {
        let mut name = self.id()?;
        while self.at(".") && matches!(self.t.get(self.i + 1), Some(Tok::Id(_))) {
            self.i += 1;
            name = format!("{name}.{}", self.id()?);
        }
        if self.at("<") {
            let mut depth = 0;
            while let Some(t) = self.peek() {
                match t {
                    Tok::P("<") => depth += 1,
                    Tok::P(">") => depth -= 1,
                    _ => {}
                }
                self.i += 1;
                if depth == 0 {
                    break;
                }
            }
        }
        while self.at("[") && matches!(self.t.get(self.i + 1), Some(Tok::P("]" | ","))) {
            while !self.eat("]") && self.peek().is_some() {
                self.i += 1;
            }
            name.push_str("[]");
        }
        self.eat("?");
        Some(name)
    }

    fn args(&mut self, close: &str) -> Vec<E> {
        let mut out = Vec::new();
        while !self.eat(close) {
            if self.peek().is_none() {
                break;
            }
            let e = self.expr();
            // An object initializer's `Name = value`: the value is what counts.
            if self.eat("=") {
                out.push(self.expr());
            } else {
                out.push(e);
            }
            if !self.eat(",") && !self.at(close) {
                // Something this reader does not follow: skip to the closing bracket.
                let mut depth = 0;
                while let Some(t) = self.peek() {
                    match t {
                        Tok::P("(" | "{" | "[") => depth += 1,
                        Tok::P(")" | "}" | "]") if depth > 0 => depth -= 1,
                        Tok::P(x) if *x == close && depth == 0 => break,
                        _ => {}
                    }
                    self.i += 1;
                }
            }
        }
        out
    }

    fn primary(&mut self) -> E {
        match self.peek().cloned() {
            Some(Tok::Num(n)) => {
                self.i += 1;
                E::Num(n)
            }
            Some(Tok::Str(s)) => {
                self.i += 1;
                E::Str(s)
            }
            Some(Tok::Id(w)) => {
                self.i += 1;
                match w.as_str() {
                    "true" => E::Bool(true),
                    "false" => E::Bool(false),
                    "null" => E::Other,
                    "this" | "base" => E::Path(vec![]),
                    "new" => {
                        let ty = self.type_name().unwrap_or_default();
                        let args = if self.eat("(") { self.args(")") } else { vec![] };
                        let items = if self.eat("{") { self.args("}") } else { vec![] };
                        E::New(ty, args, items)
                    }
                    _ => E::Path(vec![w]),
                }
            }
            Some(Tok::P("(")) => {
                // A cast - `(Type)x`, `((Type)(x))` - is the value cast.
                let save = self.i;
                self.i += 1;
                if self.type_name().is_some() && self.eat(")") && matches!(self.peek(), Some(Tok::Id(_) | Tok::Num(_) | Tok::Str(_) | Tok::P("("))) {
                    return self.unary();
                }
                self.i = save + 1;
                let e = self.expr();
                self.eat(")");
                e
            }
            _ => {
                self.i += 1;
                E::Other
            }
        }
    }

    fn postfix(&mut self) -> E {
        let mut e = self.primary();
        loop {
            if self.at(".") {
                self.i += 1;
                let Some(name) = self.id() else { return E::Other };
                e = match e {
                    E::Path(mut p) => {
                        p.push(name);
                        E::Path(p)
                    }
                    _ => E::Other,
                };
            } else if self.eat("(") {
                let args = self.args(")");
                e = match e {
                    E::Path(p) => E::Call(p, args),
                    _ => E::Other,
                };
            } else if self.eat("[") {
                self.args("]");
                e = E::Other;
            } else {
                return e;
            }
        }
    }
}

fn statement(t: &[Tok]) -> Option<Stmt> {
    // A local: `Type name = value` (`DataGridViewCellStyle style1 = new ...`, `var x = ...`).
    {
        let mut p = P { t, i: 0 };
        if p.type_name().is_some() {
            if let Some(name) = p.id() {
                if p.eat("=") {
                    return Some(Stmt::Assign(vec![name], p.expr()));
                }
            }
        }
    }
    let mut p = P { t, i: 0 };
    let lhs = p.postfix();
    if p.eat("=") {
        if let E::Path(path) = lhs {
            return Some(Stmt::Assign(path, p.expr()));
        }
    } else if p.eat("+=") {
        if let E::Path(path) = lhs {
            return Some(Stmt::Event(path, p.expr()));
        }
    } else if let E::Call(path, args) = lhs {
        return Some(Stmt::Call(path, args));
    }
    None
}

/// A control added to another: its name, the split pane it went in (1, 2; 0 the control itself),
/// its table cell.
type Kid = (String, u8, Option<(i32, i32)>);

/// An object `InitializeComponent` makes, and what it does to it.
#[derive(Default, Debug)]
struct Obj {
    ty: String,
    args: Vec<E>,
    props: Vec<(String, E)>,
    /// Controls added to it: name, the split pane (1, 2; 0 itself), the table cell.
    kids: Vec<Kid>,
    items: Vec<E>,
    drop: Vec<E>,
    cols: Vec<E>,
    nodes: Vec<E>,
    events: Vec<(String, String)>,
    tip: Option<String>,
}

impl Obj {
    fn get(&self, name: &str) -> Option<&E> {
        self.props.iter().rev().find(|(n, _)| n == name).map(|(_, e)| e)
    }
    fn short(&self) -> &str {
        self.ty.rsplit('.').next().unwrap_or(&self.ty)
    }
}

/// Types made with `new` that are values, not objects: `ClientSize = new Size(...)`.
const VALUES: &[&str] = &["Point", "Size", "SizeF", "PointF", "Font", "Padding", "Rectangle", "decimal", "Decimal", "Cursor", "Bitmap", "Icon", "string", "String", "int", "object", "ToolStripItem[]"];

#[derive(Default)]
struct Reader {
    objs: HashMap<String, Obj>,
    order: Vec<String>,
    form: Obj,
    resources: bool,
}

impl Reader {
    fn obj(&mut self, key: &str) -> Option<&mut Obj> {
        if key.is_empty() { Some(&mut self.form) } else { self.objs.get_mut(key) }
    }

    fn take(&mut self, s: Stmt) {
        match s {
            Stmt::Assign(path, e) => {
                if path.is_empty() {
                    return;
                }
                if path.len() == 1 {
                    if let E::New(ty, args, _) = &e {
                        let short = ty.rsplit('.').next().unwrap_or(ty);
                        if !VALUES.contains(&short) && !self.objs.contains_key(&path[0]) {
                            self.order.push(path[0].clone());
                            self.objs.insert(path[0].clone(), Obj { ty: ty.clone(), args: args.clone(), ..Default::default() });
                            return;
                        }
                    }
                }
                if self.objs.contains_key(&path[0]) && path.len() >= 2 {
                    let o = self.objs.get_mut(&path[0]).unwrap();
                    o.props.push((path[1..].join("."), e));
                } else if !self.objs.contains_key(&path[0]) {
                    self.form.props.push((path.join("."), e));
                }
            }
            Stmt::Event(path, e) => {
                let Some((ev, target)) = path.split_last() else { return };
                let handler = match e {
                    E::Path(p) => p.last().cloned(),
                    E::New(_, args, _) => match args.first() {
                        Some(E::Path(p)) => p.last().cloned(),
                        _ => None,
                    },
                    _ => None,
                };
                if let (Some(h), Some(o)) = (handler, self.obj(&target.join("."))) {
                    o.events.push((ev.clone(), h));
                }
            }
            Stmt::Call(path, args) => {
                let n = path.len();
                let tail = |k: usize| path[n.saturating_sub(k)..].iter().map(String::as_str).collect::<Vec<_>>();
                let refs = |e: Option<&E>| -> Vec<String> {
                    match e {
                        Some(E::New(_, _, items)) => items.iter().filter_map(|i| if let E::Path(p) = i { Some(p.join(".")) } else { None }).collect(),
                        Some(E::Path(p)) => vec![p.join(".")],
                        _ => vec![],
                    }
                };
                if path.last().is_some_and(|m| m == "ApplyResources") {
                    self.resources = true;
                    return;
                }
                if n >= 2 && (tail(2) == ["Controls", "Add"] || tail(2) == ["Controls", "AddRange"] || tail(2) == ["TabPages", "Add"] || tail(2) == ["TabPages", "AddRange"]) {
                    let owner = &path[..n - 2];
                    let (key, pane) = match owner {
                        [o, p] if p == "Panel1" => (o.clone(), 1),
                        [o, p] if p == "Panel2" => (o.clone(), 2),
                        _ => (owner.join("."), 0),
                    };
                    let cell = match (args.get(1), args.get(2)) {
                        (Some(E::Num(c)), Some(E::Num(r))) => Some((*c as i32, *r as i32)),
                        _ => None,
                    };
                    let kids = refs(args.first());
                    if let Some(o) = self.obj(&key) {
                        for k in kids {
                            o.kids.push((k, pane, cell));
                        }
                    }
                    return;
                }
                if n >= 3 {
                    let key = path[..n - 2].join(".");
                    let what = tail(2);
                    if let Some(o) = self.objs.get_mut(&key) {
                        let list = |e: Option<&E>| match e {
                            Some(E::New(_, _, items)) => items.clone(),
                            Some(other) => vec![other.clone()],
                            None => vec![],
                        };
                        match (what[0], what[1]) {
                            ("Items", "AddRange") => o.items.extend(list(args.first())),
                            ("Items", "Add") => o.items.extend(args.first().cloned()),
                            ("DropDownItems", "AddRange") => o.drop.extend(list(args.first())),
                            ("DropDownItems", "Add") => o.drop.extend(args.first().cloned()),
                            ("Columns", "AddRange") => o.cols.extend(list(args.first())),
                            ("Columns", "Add") => o.cols.push(E::New("Column".into(), args.clone(), vec![])),
                            ("Nodes", "AddRange") => o.nodes.extend(list(args.first())),
                            ("Nodes", "Add") => o.nodes.extend(args.first().cloned()),
                            _ => {}
                        }
                        return;
                    }
                }
                if n == 2 && path[1] == "SetToolTip" {
                    if let (Some(E::Path(c)), Some(E::Str(s))) = (args.first(), args.get(1)) {
                        if let Some(o) = self.objs.get_mut(&c.join(".")) {
                            o.tip = Some(s.clone());
                        }
                    }
                }
            }
        }
    }
}

// ---- values ----

fn num(e: &E) -> Option<f64> {
    match e {
        E::Num(n) => Some(*n),
        E::Neg(x) => num(x).map(|n| -n),
        E::Add(parts) => parts.iter().map(num).sum(),
        // `new decimal(new int[] { 100, 0, 0, 0 })`: lo, mid, hi, flags (scale, sign).
        E::New(t, args, _) if t.ends_with("ecimal") => match args.first() {
            Some(E::New(_, _, bits)) => {
                let b: Vec<f64> = bits.iter().filter_map(num).collect();
                let lo = *b.first()? as i64 as u32 as f64;
                let flags = *b.get(3).unwrap_or(&0.0) as i64 as u32;
                let scale = (flags >> 16) & 0xff;
                let v = lo / 10f64.powi(scale as i32);
                Some(if flags & 0x8000_0000 != 0 { -v } else { v })
            }
            Some(x) => num(x),
            None => None,
        },
        _ => None,
    }
}

fn text(e: &E) -> Option<String> {
    match e {
        E::Str(s) => Some(s.clone()),
        E::Add(parts) => parts.iter().map(text).collect(),
        _ => None,
    }
}

/// The last word of an enum value: `DockStyle.Fill` -> `Fill`.
fn word(e: &E) -> Option<&str> {
    match e {
        E::Path(p) => p.last().map(String::as_str),
        _ => None,
    }
}

/// Every flag of `A.X | A.Y`.
fn flags(e: &E) -> Vec<&str> {
    match e {
        E::Or(parts) => parts.iter().flat_map(flags).collect(),
        E::Path(p) => p.last().map(|s| vec![s.as_str()]).unwrap_or_default(),
        _ => vec![],
    }
}

fn pair(e: &E) -> Option<(f64, f64)> {
    match e {
        E::New(_, a, _) if a.len() >= 2 => Some((num(&a[0])?, num(&a[1])?)),
        _ => None,
    }
}

/// A colour as CSS writes it; `None` for Windows' own (SystemColors), see-through and unknown.
fn color(e: &E) -> Option<String> {
    match e {
        E::Call(p, a) if p.last().is_some_and(|m| m == "FromArgb") => {
            let n: Vec<f64> = a.iter().filter_map(num).collect();
            let (al, r, g, b) = match n.as_slice() {
                [r, g, b] => (255.0, *r, *g, *b),
                [al, r, g, b] => (*al, *r, *g, *b),
                [packed] => {
                    let v = *packed as i64 as u32;
                    ((v >> 24) as f64, ((v >> 16) & 0xff) as f64, ((v >> 8) & 0xff) as f64, (v & 0xff) as f64)
                }
                // FromArgb(alpha, Color.X)
                _ => return None,
            };
            (al > 0.0).then(|| format!("#{:02x}{:02x}{:02x}", r as u8, g as u8, b as u8))
        }
        E::Path(p) if p.len() >= 2 && p[p.len() - 2] == "Color" => {
            let name = p.last()?.to_ascii_lowercase();
            (name != "transparent" && name != "empty" && phai_ui::theme::parse_color(&name).is_some()).then_some(name)
        }
        _ => None,
    }
}

/// `MainForm` -> `main_form`, `_btnOK` -> `btn_ok`, `IBeam` -> `i_beam`: a Rust name.
pub fn ident(name: &str) -> String {
    let c: Vec<char> = name.chars().collect();
    let mut out = String::new();
    for (i, ch) in c.iter().enumerate() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && i > 0 {
                let prev = c[i - 1];
                let next_lower = c.get(i + 1).is_some_and(|n| n.is_ascii_lowercase());
                if prev.is_ascii_lowercase() || prev.is_ascii_digit() || (prev.is_ascii_uppercase() && next_lower) {
                    out.push('_');
                }
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    let mut s = String::new();
    for ch in out.chars() {
        if ch == '_' && (s.is_empty() || s.ends_with('_')) {
            continue;
        }
        s.push(ch);
    }
    let mut s = s.trim_end_matches('_').to_string();
    if s.is_empty() {
        s = "control".into();
    }
    if s.starts_with(|c: char| c.is_ascii_digit()) {
        s = format!("c{s}");
    }
    if KEYWORDS.contains(&s.as_str()) {
        s.push('_');
    }
    s
}

const KEYWORDS: &[&str] = &["as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "gen"];

fn pascal(name: &str) -> String {
    let s: String = ident(name).split('_').map(|w| {
        let mut c = w.chars();
        c.next().map(|f| f.to_ascii_uppercase().to_string() + c.as_str()).unwrap_or_default()
    }).collect();
    if s.starts_with(|c: char| c.is_ascii_uppercase()) { s } else { format!("Form{s}") }
}

/// An enum value as one of a property's choices: `FixedSingle` -> `fixed_single`, `LargeIcon` ->
/// `large_icons`, `MiddleRight` -> `right`.
fn choice(value: &str, choices: &[&str]) -> Option<String> {
    let s = ident(value);
    let mapped = match s.as_str() {
        "large_icon" => "large_icons",
        "small_icon" => "small_icons",
        "i_beam" => "ibeam",
        "wait_cursor" => "wait",
        "stretch_image" => "stretch",
        "center_image" => "center",
        "auto_size" => "normal",
        "multi_extended" | "multi_simple" => "multi",
        "nonclickable" => "clickable",
        "fixed_tool_window" => "fixed_tool",
        "sizable_tool_window" => "sizable_tool",
        "fixed3_d" | "fixed_single" => "fixed",
        _ => "",
    };
    let unaligned = ["top_", "middle_", "bottom_"].iter().find_map(|p| s.strip_prefix(p)).unwrap_or("");
    let found = [s.as_str(), mapped, unaligned].into_iter().find(|c| !c.is_empty() && choices.contains(c)).map(str::to_string);
    found
}

/// A toolbar button's picture, guessed from its words.
fn glyph_for(words: &str) -> &'static str {
    let w = words.to_ascii_lowercase();
    const BY: &[(&[&str], &str)] = &[
        (&["save all"], "SAVE_ALL"),
        (&["download", "receive", "to this computer"], "DOWNLOAD"),
        (&["upload", "send"], "UPLOAD"),
        (&["new folder"], "NEW_FOLDER"),
        (&["save"], "SAVE"),
        (&["open", "load"], "OPEN"),
        (&["new", "add", "create"], "NEW"),
        (&["undo"], "UNDO"),
        (&["redo"], "REDO"),
        (&["cut"], "CUT"),
        (&["copy"], "COPY"),
        (&["paste"], "PASTE"),
        (&["delete", "remove", "clear"], "DELETE"),
        (&["refresh", "reload"], "REFRESH"),
        (&["search", "find"], "SEARCH"),
        (&["setting", "option", "config", "preference"], "SETTINGS"),
        (&["stop", "disconnect", "cancel"], "STOP"),
        (&["pause"], "PAUSE"),
        (&["run", "play", "start", "connect"], "PLAY"),
        (&["build"], "BUILD"),
        (&["folder", "browse"], "FOLDER"),
        (&["rename"], "RENAME"),
        (&["chat", "message"], "CHAT"),
        (&["screen", "monitor", "display", "view"], "MONITOR"),
        (&["back", "previous"], "BACK"),
        (&["forward", "next"], "FORWARD"),
        (&["up"], "UP"),
        (&["font"], "FONT"),
        (&["colo"], "COLOR"),
        (&["print", "file", "document", "report"], "DOCUMENT"),
    ];
    BY.iter().find(|(keys, _)| keys.iter().any(|k| w.contains(k))).map(|(_, g)| *g).unwrap_or("BUTTON")
}

// ---- building the form ----

/// How an edge follows its container: (near edge kept, far edge kept), as anchors are.
type Axis = (bool, bool);

fn compose(parent: Axis, child: Axis) -> Axis {
    match parent {
        (true, false) => (true, false),
        (false, true) => (false, true),
        (true, true) => child,
        (false, false) => (false, false),
    }
}

/// What the Toolbar's buttons are in Phai: their strip's height.
const TOOLBAR_HEIGHT: i32 = 34;

struct Build<'a> {
    r: &'a Reader,
    form: FormFile,
    warn: Vec<String>,
    scale: f64,
    form_font: f64,
    form_family: String,
    /// Object key -> control id.
    ids: HashMap<String, String>,
    taken: HashSet<String>,
    placed: HashSet<String>,
    left_out: BTreeSet<String>,
    /// Handler -> what it is given; one name must not handle events of two kinds.
    handlers: HashMap<String, crate::events::Args>,
    /// Pixels the menu took at the top of WinForms' window, which Phai's menu does not.
    menu_h: i32,
    /// Where the toolbar ended, and what Phai's taller toolbar adds below it.
    tool_bottom: i32,
    tool_delta: i32,
    /// The colour of what a control lies on: WinForms gives it to the panels and labels on it that
    /// set none of their own.
    ambient: Option<String>,
}

/// Properties with nothing to carry: where WinForms keeps its own bookkeeping.
const QUIET: &[&str] = &["Name", "Location", "Size", "Margin", "UseVisualStyleBackColor", "ImeMode", "AutoScaleDimensions", "AutoScaleMode", "MainMenuStrip", "ImageScalingSize", "UseCompatibleTextRendering", "FormattingEnabled", "TabStop", "ClientSize", "Text", "Font", "Dock", "Anchor", "Visible", "Enabled", "BackColor", "ForeColor", "Checked", "Minimum", "Maximum", "Value", "Interval", "Items", "Multiline", "PasswordChar", "UseSystemPasswordChar", "DropDownStyle", "CheckState", "Style", "Image", "ImageLocation", "Icon", "ContextMenuStrip", "SplitterDistance", "SplitterWidth", "ColumnCount", "RowCount", "Orientation", "HeaderText", "Width", "StartPosition", "WindowState", "Opacity", "SelectedIndex", "RowHeadersWidthSizeMode", "ColumnHeadersHeightSizeMode", "RowHeadersWidth", "ColumnHeadersHeight", "AutoScroll", "AutoSize", "AutoSizeMode", "MinimumSize", "MaximumSize", "FlatAppearance.BorderColor", "FlatAppearance.BorderSize", "TabIndex", "ToolTipText", "ShortcutKeys", "CheckOnClick", "DisplayStyle", "ImageTransparentColor", "Spring", "GripStyle", "RenderMode", "ColumnHeadersDefaultCellStyle", "AlternatingRowsDefaultCellStyle", "DefaultCellStyle", "RowHeadersDefaultCellStyle", "RowTemplate.Height", "AutoSizeColumnsMode", "Padding", "SizeMode", "Cursor", "ReadOnly", "ShowItemToolTips", "TextImageRelation", "ImageAlign", "CausesValidation", "SizingGrip", "Stretch", "RowHeadersVisible", "BackgroundColor", "FlatStyle", "Width", "Height", "UseCompatibleStateImageBehavior", "ShowIcon", "UseMnemonic", "GridColor", "SelectionMode", "ScrollBars", "DecimalPlaces", "MinimumWidth", "SortMode"];

impl<'a> Build<'a> {
    fn new(r: &'a Reader, class: &str, framework: bool) -> Build<'a> {
        let f = &r.form;
        // The form's font: Windows' own unless it says.
        let own = if framework { ("Microsoft Sans Serif".to_string(), 8.25) } else { ("Segoe UI".to_string(), 9.0) };
        let set = match f.get("Font") {
            Some(E::New(_, a, _)) => Some((a.first().and_then(text).unwrap_or_else(|| own.0.clone()), a.get(1).and_then(num).unwrap_or(own.1))),
            _ => None,
        };
        let (family, size) = set.clone().unwrap_or(own);
        // WinForms wrote sizes at the designer's screen: 200% writes them twice as large. Its
        // AutoScaleDimensions say how large - a letter's size for AutoScaleMode.Font, the dots per
        // inch for Dpi.
        let mode = f.get("AutoScaleMode").and_then(word).unwrap_or("");
        let dims = f.get("AutoScaleDimensions").and_then(pair);
        let scale = match (mode, dims) {
            ("Font", Some((w, h))) => {
                // A letter of Microsoft Sans Serif 8.25 pt is 6 x 13 at 96 dpi, of Segoe UI 9 pt 7 x 15.
                let sans = |pt: f64| (6.0 * pt / 8.25, 13.0 * pt / 8.25);
                let segoe = |pt: f64| (7.0 * pt / 9.0, 15.0 * pt / 9.0);
                let bases = match &set {
                    Some((fam, pt)) if fam.to_ascii_lowercase().contains("sans serif") => vec![sans(*pt)],
                    Some((_, pt)) => vec![segoe(*pt)],
                    None if framework => vec![sans(size), segoe(9.0)],
                    None => vec![segoe(size), sans(8.25)],
                };
                // The screen's scale is a quarter step: 100%, 125%, 150%...
                let tries: Vec<(f64, f64)> = bases.iter().map(|(bw, bh)| (w / bw + h / bh) / 2.0).map(|s| (s, (s * 4.0).round() / 4.0)).collect();
                tries.iter().find(|(s, q)| (s - q).abs() < 0.04).or(tries.first()).map(|(s, q)| if (s - q).abs() < 0.1 { *q } else { *s }).unwrap_or(1.0)
            }
            ("Dpi", Some((w, _))) => w / 96.0,
            _ => 1.0,
        }
        .max(0.25);
        let mut form = FormFile {
            format: FORMAT,
            form: Form { name: pascal(class), title: class.to_string(), size: [480, 320], theme: None, font_size: None, start: None, maximized: None, events: Default::default(), more: Props::new() },
            menu: vec![],
            toolbar: vec![],
            control: vec![],
        };
        if set.is_some() && (size - 9.0).abs() >= 0.5 {
            form.form.font_size = Some((size.round() as i32).clamp(6, 48));
        }
        Build { r, form, warn: vec![], scale, form_font: size, form_family: family, ids: HashMap::new(), taken: HashSet::new(), placed: HashSet::new(), left_out: BTreeSet::new(), handlers: HashMap::new(), menu_h: 0, tool_bottom: 0, tool_delta: 0, ambient: None }
    }

    fn px(&self, v: f64) -> i32 {
        (v / self.scale).round() as i32
    }

    fn free_id(&mut self, base: &str) -> String {
        let mut id = base.to_string();
        let mut n = 2;
        while self.taken.contains(&id) {
            id = format!("{base}{n}");
            n += 1;
        }
        self.taken.insert(id.clone());
        id
    }

    fn o(&self, key: &str) -> Option<&'a Obj> {
        self.r.objs.get(key)
    }

    /// A handler's Rust name, made unique to its kind of arguments.
    fn handler(&mut self, h: &str, args: crate::events::Args, fallback: &str) -> String {
        let name = ident(h);
        match self.handlers.get(&name) {
            Some(a) if *a != args => {
                let alt = ident(fallback);
                self.handlers.insert(alt.clone(), args);
                alt
            }
            _ => {
                self.handlers.insert(name.clone(), args);
                name
            }
        }
    }

    fn run(&mut self) {
        let r = self.r;
        let f = &r.form;
        if r.resources {
            self.warn.push("This form keeps its words and places in its .resx (Localizable = true) - only what InitializeComponent sets itself came over".into());
        }
        if let Some(t) = f.get("Text").and_then(text) {
            self.form.form.title = t;
        }
        let user_control = f.get("ClientSize").is_none() && f.get("Text").is_none();
        let size = f.get("ClientSize").or_else(|| f.get("Size")).and_then(pair).map(|(w, h)| (self.px(w), self.px(h)));
        let (mut w, mut h) = size.unwrap_or((480, 320));
        if f.get("ClientSize").is_none() && !user_control {
            (w, h) = (w - 16, h - 39);
        }
        // Ids for everything first, so AcceptButton or ContextMenuStrip can name any of them.
        for key in &r.order {
            let o = &r.objs[key];
            if kind_of(o).is_some() {
                let base = ident(o.get("Name").and_then(text).as_deref().unwrap_or(key));
                let id = self.free_id(&base);
                self.ids.insert(key.clone(), id);
            }
        }
        // The menu and the toolbar along the top: Phai's menu is outside the window's area, its
        // toolbar a little taller than WinForms'.
        let top_kids: Vec<&String> = f.kids.iter().map(|(k, _, _)| k).collect();
        for k in &top_kids {
            let Some(o) = self.o(k) else { continue };
            match o.short() {
                "MenuStrip" if self.form.menu.is_empty() => {
                    self.menu_h = o.get("Size").and_then(pair).map(|(_, h)| self.px(h)).unwrap_or(24);
                    self.menus(o);
                    self.placed.insert((*k).clone());
                }
                "ToolStrip" if self.form.toolbar.is_empty() => {
                    let y = o.get("Location").and_then(pair).map(|(_, y)| self.px(y)).unwrap_or(0);
                    let th = o.get("Size").and_then(pair).map(|(_, h)| self.px(h)).unwrap_or(25);
                    self.tool_bottom = y + th;
                    self.tool_delta = TOOLBAR_HEIGHT - th;
                    self.toolbar(o);
                    self.placed.insert((*k).clone());
                }
                _ => {}
            }
        }
        self.tool_bottom -= self.menu_h;
        h = h - self.menu_h + if self.form.toolbar.is_empty() { 0 } else { self.tool_delta };
        self.form.form.size = [w.max(50), h.max(30)];
        // The controls, the form's last added first: that is the order WinForms docks them in,
        // and the order back to front.
        let raw_h = h + self.menu_h - if self.form.toolbar.is_empty() { 0 } else { self.tool_delta };
        for (k, _, _) in f.kids.iter().rev() {
            self.ambient = f.get("BackColor").and_then(color);
            self.place(k, (0, 0), (0, 0, w, raw_h), ((true, false), (true, false)), true, false, None);
        }
        for key in &r.order {
            if self.placed.contains(key) {
                continue;
            }
            let o = &r.objs[key];
            match kind_of(o) {
                Some(k) if k.is_component() => self.component(key, o, k),
                _ => {}
            }
        }
        self.form_props();
        self.radio_groups();
        if !self.left_out.is_empty() {
            let list: Vec<String> = self.left_out.iter().cloned().collect();
            self.warn.push(format!("Not brought over: {}", list.join(", ")));
        }
        // What the checker would refuse is said, and taken out, rather than left to break the form.
        for _ in 0..8 {
            let problems = self.form.check();
            let Some((msg, _)) = problems.first() else { break };
            self.warn.push(format!("Left out to keep the form valid: {msg}"));
            if !self.mend(msg) {
                break;
            }
        }
    }

    /// Takes out what one of the checker's complaints is about; false when it cannot tell what.
    fn mend(&mut self, msg: &str) -> bool {
        let before = self.form.clone();
        for c in &mut self.form.control {
            if msg.starts_with(&format!("{}:", c.id)) || msg.contains(&format!(" {} ", c.id)) {
                c.more.retain(|n, _| !msg.contains(n.as_str()));
                c.events.clear();
            }
        }
        if msg.contains("handles events with different arguments") {
            let h = msg.split_whitespace().next().unwrap_or("").to_string();
            for c in &mut self.form.control {
                c.events.retain(|_, v| *v != h);
            }
        }
        self.form != before
    }

    fn form_props(&mut self) {
        let f = &self.r.form;
        let names: Vec<(String, E)> = f.props.clone();
        for (name, e) in &names {
            match name.as_str() {
                "StartPosition" => {
                    if matches!(word(e), Some("Manual" | "WindowsDefaultLocation" | "WindowsDefaultBounds")) {
                        self.form.form.start = Some("windows".into());
                    }
                }
                "WindowState" => {
                    if word(e) == Some("Maximized") {
                        self.form.form.maximized = Some(true);
                    }
                }
                "Icon" => self.warn.push("The form's icon was in the .resx - set it in Properties (Icon)".into()),
                "Opacity" => {
                    if let Some(n) = num(e) {
                        let v = (n * 100.0).round() as i64;
                        if v < 100 {
                            self.form.form.set_prop("opacity", Some(toml::Value::Integer(v.clamp(10, 100))));
                        }
                    }
                }
                _ if QUIET.contains(&name.as_str()) || name == "Padding" => {}
                _ => {
                    match props::FORM.iter().find(|s| s.label == name) {
                        Some(s) => {
                            if let Some(v) = self.value(s, e) {
                                self.form.form.set_prop(s.name, Some(v));
                            }
                        }
                        None => {
                            if name != "Load" {
                                self.left_out.insert(format!("Form.{name}"));
                            }
                        }
                    }
                }
            }
        }
        for c in ["BackColor", "ForeColor"] {
            if let Some(v) = f.get(c).and_then(color) {
                self.form.form.set_prop(if c == "BackColor" { "back_color" } else { "fore_color" }, Some(toml::Value::String(v)));
            }
        }
        if f.get("ClientSize").is_none() && f.get("Text").is_none() {
            self.form.form.set_prop("user_control", Some(toml::Value::Boolean(true)));
        }
        let events = f.events.clone();
        for (ev, h) in events {
            if ev == "Load" {
                continue; // `loaded`, which every form has
            }
            match crate::events::FORM.iter().find(|s| s.label == ev) {
                Some(s) => {
                    let name = self.handler(&h, s.args, &format!("form_{}", s.name));
                    self.form.form.events.insert(s.name.to_string(), name);
                }
                None => {
                    self.left_out.insert(format!("Form.{ev} event"));
                }
            }
        }
    }

    /// A table property's value from C#, or none when it is the default or will not do.
    fn value(&self, s: &props::Spec, e: &E) -> Option<toml::Value> {
        let v = match s.ty {
            Ty::Bool => match e {
                E::Bool(b) => toml::Value::Boolean(*b),
                _ => return None,
            },
            Ty::Int(lo, hi) => {
                let n = num(e)?;
                let n = if matches!(s.name, "splitter_distance" | "row_height") { self.px(n) as i64 } else { n.round() as i64 };
                toml::Value::Integer(n.clamp(lo as i64, hi as i64))
            }
            Ty::Text | Ty::Words => toml::Value::String(text(e).filter(|t| !t.is_empty())?),
            Ty::Choice(c) => toml::Value::String(choice(word(e)?, c)?),
            Ty::Color => toml::Value::String(color(e)?),
            Ty::Size => {
                let (w, h) = pair(e)?;
                let (w, h) = (self.px(w), self.px(h));
                if w <= 0 || h <= 0 {
                    return None;
                }
                toml::Value::Array(vec![toml::Value::Integer(w as i64), toml::Value::Integer(h as i64)])
            }
            Ty::Button => {
                let key = match e {
                    E::Path(p) => p.join("."),
                    _ => return None,
                };
                let o = self.o(&key)?;
                if kind_of(o) != Some(Kind::Button) {
                    return None;
                }
                toml::Value::String(self.ids.get(&key)?.clone())
            }
            Ty::Date => return None,
        };
        if props::text_of(&v) == s.default || props::check(s, &v).is_some() {
            return None;
        }
        Some(v)
    }

    fn menus(&mut self, strip: &Obj) {
        for it in &strip.items {
            let E::Path(p) = it else { continue };
            let Some(o) = self.o(&p.join(".")) else { continue };
            if o.short() != "ToolStripMenuItem" {
                continue;
            }
            let label = o.get("Text").and_then(text).unwrap_or_else(|| p.join("."));
            let items = self.menu_items(&o.drop);
            let mut m = Menu { label, items, events: Default::default() };
            for (ev, h) in &o.events {
                if let Some(s) = crate::events::MENU.iter().find(|s| s.label == ev) {
                    let name = self.handler(h, s.args, &format!("{}_{}", ident(&p.join(".")), s.name));
                    m.events.insert(s.name.to_string(), name);
                }
            }
            self.form.menu.push(m);
        }
    }

    fn menu_items(&mut self, list: &[E]) -> Vec<MenuItem> {
        let mut out = Vec::new();
        for it in list {
            let E::Path(p) = it else { continue };
            let key = p.join(".");
            let Some(o) = self.o(&key) else { continue };
            match o.short() {
                "ToolStripSeparator" => out.push(MenuItem { separator: true, ..Default::default() }),
                "ToolStripMenuItem" => {
                    let label = o.get("Text").and_then(text).filter(|t| !t.is_empty()).unwrap_or_else(|| key.clone());
                    let mut m = MenuItem { label: Some(label), ..Default::default() };
                    if !o.drop.is_empty() {
                        m.items = self.menu_items(&o.drop);
                    } else {
                        let click = o.events.iter().find(|(e, _)| e == "Click").map(|(_, h)| ident(h)).unwrap_or_else(|| ident(&key));
                        let id = if self.taken.contains(&click) && !self.handlers.contains_key(&click) { self.free_id(&click) } else { click };
                        self.taken.insert(id.clone());
                        self.handlers.insert(id.clone(), crate::events::Args::None);
                        m.id = Some(id);
                    }
                    m.key = o.get("ShortcutKeys").map(keys).filter(|k| !k.is_empty());
                    m.checked = o.get("Checked") == Some(&E::Bool(true)) || o.get("CheckState").and_then(word) == Some("Checked");
                    m.check_on_click = o.get("CheckOnClick") == Some(&E::Bool(true));
                    m.disabled = o.get("Enabled") == Some(&E::Bool(false));
                    for (ev, h) in &o.events {
                        if ev == "Click" {
                            continue;
                        }
                        match crate::events::MENU_ITEM.iter().find(|s| s.label == ev) {
                            Some(s) => {
                                let name = self.handler(h, s.args, &format!("{}_{}", ident(&key), s.name));
                                m.events.insert(s.name.to_string(), name);
                            }
                            None => {
                                self.left_out.insert(format!("ToolStripMenuItem.{ev} event"));
                            }
                        }
                    }
                    out.push(m);
                }
                other => {
                    self.left_out.insert(format!("{other} in a menu"));
                }
            }
        }
        out
    }

    fn toolbar(&mut self, strip: &Obj) {
        for it in &strip.items {
            let E::Path(p) = it else { continue };
            let key = p.join(".");
            let Some(o) = self.o(&key) else { continue };
            match o.short() {
                "ToolStripSeparator" => self.form.toolbar.push(Tool { separator: true, ..Default::default() }),
                "ToolStripButton" => {
                    let words = o.get("ToolTipText").and_then(text).or_else(|| o.get("Text").and_then(text)).unwrap_or_else(|| key.clone());
                    let click = o.events.iter().find(|(e, _)| e == "Click").map(|(_, h)| ident(h)).unwrap_or_else(|| ident(&key));
                    let id = if self.form.toolbar.iter().any(|t| t.id == click) { self.free_id(&click) } else { click };
                    self.handlers.insert(id.clone(), crate::events::Args::None);
                    let glyph = glyph_for(&format!("{words} {key}")).to_string();
                    self.form.toolbar.push(Tool {
                        id,
                        label: words,
                        glyph,
                        toggle: o.get("CheckOnClick") == Some(&E::Bool(true)),
                        separator: false,
                        checked: o.get("Checked") == Some(&E::Bool(true)),
                        disabled: o.get("Enabled") == Some(&E::Bool(false)),
                        events: Default::default(),
                    });
                }
                other => {
                    self.left_out.insert(format!("{other} on a toolbar"));
                }
            }
        }
        if !self.form.toolbar.is_empty() {
            self.warn.push("The toolbar's pictures were chosen from the buttons' words - change them in the toolbar editor".into());
        }
    }

    fn component(&mut self, key: &str, o: &Obj, kind: Kind) {
        self.placed.insert(key.to_string());
        let id = self.ids.get(key).cloned().unwrap_or_else(|| self.free_id(&ident(key)));
        let mut c = blank(kind, &id, [0, 0, 32, 32]);
        c.bounds = None;
        match kind {
            Kind::Timer => {
                c.value = Some(o.get("Interval").and_then(num).map(|n| n as i32).unwrap_or(100));
            }
            Kind::ContextMenu => {
                for it in &o.items {
                    let E::Path(p) = it else { continue };
                    let k = p.join(".");
                    let Some(m) = self.o(&k) else { continue };
                    match m.short() {
                        "ToolStripSeparator" => c.items.push("-".into()),
                        "ToolStripMenuItem" => {
                            let label = m.get("Text").and_then(text).unwrap_or_else(|| k.clone());
                            let method = m.events.iter().find(|(e, _)| e == "Click").map(|(_, h)| ident(h)).unwrap_or_else(|| ident(&k));
                            self.handlers.insert(method.clone(), crate::events::Args::None);
                            if !m.drop.is_empty() {
                                self.left_out.insert("a menu within a context menu".into());
                            }
                            c.items.push(format!("{method} | {label}"));
                        }
                        other => {
                            self.left_out.insert(format!("{other} in a context menu"));
                        }
                    }
                }
            }
            Kind::NotifyIcon => {
                c.text = o.get("Text").and_then(text);
            }
            _ => {}
        }
        self.common_props(&mut c, o);
        self.events(&mut c, o);
        self.form.control.push(c);
    }

    /// One control and those it holds. `origin`: where its container's inside begins on the form;
    /// `axes`: how that container follows the window; `top`: it is on the form itself.
    #[allow(clippy::too_many_arguments)]
    fn place(&mut self, key: &str, origin: (i32, i32), clip: (i32, i32, i32, i32), axes: (Axis, Axis), top: bool, hidden: bool, page: Option<usize>) {
        if self.placed.contains(key) {
            return;
        }
        let Some(o) = self.o(key) else { return };
        self.placed.insert(key.to_string());
        let short = o.short();
        if matches!(short, "MenuStrip" | "ToolStrip") {
            self.warn.push(format!("{key}: a second {short}, or one inside a panel - Phai Studio has one menu and one toolbar along the top"));
            return;
        }
        let kind = match kind_of(o) {
            Some(k) => k,
            None => {
                self.warn.push(format!("{key}: {} is not a control Phai Studio knows - it became a panel", o.ty));
                Kind::Panel
            }
        };
        if kind.is_component() {
            self.component(key, o, kind);
            return;
        }
        let id = match self.ids.get(key) {
            Some(id) => id.clone(),
            None => {
                let id = self.free_id(&ident(key));
                self.ids.insert(key.to_string(), id.clone());
                id
            }
        };
        let (lx, ly) = o.get("Location").and_then(pair).map(|(x, y)| (self.px(x), self.px(y))).unwrap_or((0, 0));
        let (dw, dh) = kind.default_size();
        let (mut w, mut h) = o.get("Size").and_then(pair).map(|(w, h)| (self.px(w), self.px(h))).unwrap_or((dw, dh));
        if let Some(n) = o.get("Width").and_then(num) {
            w = self.px(n);
        }
        if let Some(n) = o.get("Height").and_then(num) {
            h = self.px(n);
        }
        let (ax, ay) = (origin.0 + lx, origin.1 + ly);
        // What lies outside its container is not seen in WinForms: cut to it.
        let (x1, y1) = ((ax + w).min(clip.2), (ay + h).min(clip.3));
        let (ax, ay) = (ax.max(clip.0), ay.max(clip.1));
        let (w, h) = if x1 > ax && y1 > ay { (x1 - ax, y1 - ay) } else { (w, h) };
        let inner = (ax, ay, ax + w, ay + h);
        let dock = match o.get("Dock").and_then(word) {
            Some(d) if d != "None" => Some(d.to_ascii_lowercase()),
            Some(_) => None,
            None if kind == Kind::Statusbar => Some("bottom".into()),
            None => None,
        };
        let rel: (Axis, Axis) = match dock.as_deref() {
            Some("top") => ((true, true), (true, false)),
            Some("bottom") => ((true, true), (false, true)),
            Some("left") => ((true, false), (true, true)),
            Some("right") => ((false, true), (true, true)),
            Some("fill") => ((true, true), (true, true)),
            _ => match o.get("Anchor") {
                Some(e) => {
                    let f = flags(e);
                    ((f.contains(&"Left"), f.contains(&"Right")), (f.contains(&"Top"), f.contains(&"Bottom")))
                }
                None => ((true, false), (true, false)),
            },
        };
        let eff = if top { rel } else { (compose(axes.0, rel.0), compose(axes.1, rel.1)) };
        // On the form, below the toolbar: Phai's menu takes no room, its toolbar a little more.
        let fy = |y: i32, b: &Build| {
            let y = y - b.menu_h;
            if !b.form.toolbar.is_empty() && y >= b.tool_bottom { y + b.tool_delta } else { y }
        };
        let y = fy(ay, self);
        let mut c = blank(kind, &id, [ax, y, w.max(0), h.max(0)]);
        if top && dock.is_some() {
            c.dock = dock.clone();
        } else {
            c.anchor = anchor_text(Anchors { left: eff.0 .0, right: eff.0 .1, top: eff.1 .0, bottom: eff.1 .1 });
        }
        c.page = page;
        let hidden = hidden || o.get("Visible") == Some(&E::Bool(false));
        if hidden {
            c.visible = Some(false);
        }
        if kind.has_text() {
            c.text = o.get("Text").and_then(text);
        }
        self.kind_props(&mut c, o);
        self.common_props(&mut c, o);
        self.events(&mut c, o);
        let outer = self.ambient.clone();
        if c.back_color.is_none() && matches!(kind, Kind::Panel | Kind::Flow | Kind::Table | Kind::Split | Kind::Label | Kind::Checkbox | Kind::Radio | Kind::Group | Kind::Link | Kind::Picture | Kind::Slider) && !top {
            c.back_color = outer.clone();
        }
        if c.back_color.is_some() {
            self.ambient = c.back_color.clone();
        }
        let at = self.form.control.len();
        self.form.control.push(c);
        // Those it holds.
        match kind {
            Kind::Tabs => {
                let mut tabs = Vec::new();
                for (i, (pk, _, _)) in o.kids.iter().enumerate() {
                    let Some(p) = self.o(pk) else { continue };
                    self.placed.insert(pk.clone());
                    tabs.push(p.get("Text").and_then(text).unwrap_or_else(|| format!("Page {}", i + 1)));
                    for (k, _, _) in p.kids.iter() {
                        self.place(k, (ax + 4, ay + TOOLBAR_HEIGHT), inner, eff, false, hidden, Some(i));
                    }
                }
                let c = &mut self.form.control[at];
                c.tabs = tabs;
                if let Some([_, _, _, th]) = c.bounds.as_mut() {
                    *th = TOOLBAR_HEIGHT;
                }
            }
            Kind::Split => {
                let d = o.get("SplitterDistance").and_then(num).map(|n| self.px(n)).unwrap_or(w / 2);
                let bar = o.get("SplitterWidth").and_then(num).map(|n| self.px(n)).unwrap_or(4);
                let across = o.get("Orientation").and_then(word) != Some("Horizontal");
                for (k, pane, _) in o.kids.iter() {
                    let off = match (pane, across) {
                        (2, true) => (d + bar, 0),
                        (2, false) => (0, d + bar),
                        _ => (0, 0),
                    };
                    self.place(k, (ax + off.0, ay + off.1), inner, eff, false, hidden, page);
                }
            }
            Kind::Flow | Kind::Table => {
                let mut kids = o.kids.clone();
                if kind == Kind::Table {
                    kids.sort_by_key(|(_, _, cell)| cell.map(|(c, r)| (r, c)).unwrap_or((i32::MAX, i32::MAX)));
                }
                for (k, _, _) in &kids {
                    self.place(k, (ax, ay), inner, eff, false, hidden, page);
                }
            }
            // Front to back, as WinForms keeps them: the first in Phai's form is in front too.
            _ => {
                for (k, _, _) in o.kids.iter() {
                    self.place(k, (ax, ay), inner, eff, false, hidden, page);
                }
            }
        }
        self.ambient = outer;
    }

    /// What only some kinds of control have.
    fn kind_props(&mut self, c: &mut Control, o: &Obj) {
        let is_true = |n: &str| o.get(n) == Some(&E::Bool(true));
        match c.kind {
            Kind::Edit | Kind::Masked => {
                let pc = o.get("PasswordChar").and_then(text).is_some_and(|s| !s.is_empty() && s != "\0");
                if c.kind == Kind::Edit && (pc || is_true("UseSystemPasswordChar")) {
                    c.password = Some(true);
                }
            }
            Kind::Checkbox | Kind::Radio => {
                if is_true("Checked") || o.get("CheckState").and_then(word) == Some("Checked") {
                    c.checked = Some(true);
                }
                if c.kind == Kind::Checkbox && o.get("CheckState").and_then(word) == Some("Indeterminate") {
                    c.set_prop("indeterminate", Some(toml::Value::Boolean(true)));
                }
            }
            Kind::Combo | Kind::ListBox | Kind::CheckedList => {
                c.items = o.items.iter().filter_map(text).collect();
                if c.kind == Kind::Combo && o.get("DropDownStyle").and_then(word) != Some("DropDownList") {
                    c.set_prop("editable", Some(toml::Value::Boolean(true)));
                }
            }
            Kind::Number | Kind::Slider | Kind::Progress => {
                let get = |n: &str| o.get(n).and_then(num).map(|v| v.round().clamp(i32::MIN as f64, i32::MAX as f64) as i32);
                c.min = get("Minimum").filter(|v| *v != 0);
                c.max = get("Maximum").filter(|v| *v != 100);
                c.value = get("Value").filter(|v| *v != 0);
                if c.min.unwrap_or(0) > c.max.unwrap_or(100) {
                    (c.min, c.max) = (None, None);
                }
                if c.kind == Kind::Progress && o.get("Style").and_then(word) == Some("Marquee") {
                    c.set_prop("marquee", Some(toml::Value::Boolean(true)));
                }
            }
            Kind::Picture => {
                if let Some(path) = o.get("ImageLocation").and_then(text) {
                    c.image = Some(path);
                } else if o.get("Image").is_some() {
                    self.warn.push(format!("{}: its picture was in the .resx - set image in Properties", c.id));
                }
            }
            Kind::List => {
                let mut aligns = Vec::new();
                for col in &o.cols {
                    let (t, w, a) = match col {
                        E::Path(p) => match self.o(&p.join(".")) {
                            Some(h) => (h.get("Text").and_then(text).unwrap_or_default(), h.get("Width").and_then(num).map(|n| self.px(n)).unwrap_or(60), h.get("TextAlign").and_then(word).and_then(|a| choice(a, &["left", "center", "right"]))),
                            None => continue,
                        },
                        E::New(_, args, _) => (args.first().and_then(text).unwrap_or_default(), args.get(1).and_then(num).map(|n| self.px(n)).unwrap_or(60), None),
                        _ => continue,
                    };
                    c.columns.push((t, w.max(10)));
                    aligns.push(a.unwrap_or_else(|| "left".into()));
                }
                if aligns.iter().any(|a| a != "left") {
                    c.set_prop("column_align", Some(toml::Value::String(aligns.join(", "))));
                }
            }
            Kind::Grid => {
                for col in &o.cols {
                    let (t, w, hide) = match col {
                        E::Path(p) => match self.o(&p.join(".")) {
                            Some(h) => (h.get("HeaderText").and_then(text).or_else(|| h.get("Name").and_then(text)).unwrap_or_default(), h.get("Width").and_then(num).map(|n| self.px(n)).unwrap_or(100), h.get("Visible") == Some(&E::Bool(false))),
                            None => continue,
                        },
                        E::New(_, args, _) => (args.get(1).or(args.first()).and_then(text).unwrap_or_default(), 100, false),
                        _ => continue,
                    };
                    c.columns.push((t, w.max(10)));
                    let mut cp = crate::model::ColumnProps::default();
                    if hide {
                        cp.visible = Some(false);
                    }
                    c.column_props.push(cp);
                }
                if c.column_props.iter().all(|p| p.is_default()) {
                    c.column_props.clear();
                }
                let style = |n: &str, what: &str| -> Option<String> {
                    let E::Path(p) = o.get(n)? else { return None };
                    self.o(&p.join("."))?.get(what).and_then(color)
                };
                for (n, what, prop) in [("AlternatingRowsDefaultCellStyle", "BackColor", "alternate_row_color"), ("ColumnHeadersDefaultCellStyle", "BackColor", "header_back_color"), ("ColumnHeadersDefaultCellStyle", "ForeColor", "header_fore_color")] {
                    if let Some(v) = style(n, what) {
                        c.set_prop(prop, Some(toml::Value::String(v)));
                    }
                }
                if let Some(v) = o.get("RowTemplate.Height").and_then(num) {
                    let h = self.px(v);
                    if let Some(s) = props::find(Kind::Grid, "row_height") {
                        let v = toml::Value::Integer(h as i64);
                        if props::check(s, &v).is_none() {
                            c.set_prop("row_height", Some(v));
                        }
                    }
                }
                if o.get("AutoSizeColumnsMode").and_then(word) == Some("Fill") {
                    c.set_prop("column_auto_width", Some(toml::Value::Boolean(true)));
                }
                if is_true("ReadOnly") {
                    c.set_prop("read_only", Some(toml::Value::Boolean(true)));
                }
            }
            Kind::Tree => {
                let mut lines = Vec::new();
                fn walk(b: &Build, e: &E, depth: usize, out: &mut Vec<String>) {
                    let (label, kids): (Option<String>, Vec<E>) = match e {
                        E::Path(p) => match b.o(&p.join(".")) {
                            Some(n) => (n.get("Text").and_then(text).or_else(|| n.args.first().and_then(text)), match n.args.get(1) {
                                Some(E::New(_, _, items)) => items.clone(),
                                _ => vec![],
                            }),
                            None => (None, vec![]),
                        },
                        E::New(_, args, _) => (args.first().and_then(text), match args.get(1) {
                            Some(E::New(_, _, items)) => items.clone(),
                            _ => vec![],
                        }),
                        E::Str(s) => (Some(s.clone()), vec![]),
                        _ => (None, vec![]),
                    };
                    if let Some(l) = label {
                        out.push(format!("{}{l}", "  ".repeat(depth)));
                        for k in &kids {
                            walk(b, k, depth + 1, out);
                        }
                    }
                }
                for n in &o.nodes {
                    walk(self, n, 0, &mut lines);
                }
                c.items = lines;
            }
            Kind::Table => {
                if let Some(n) = o.get("ColumnCount").and_then(num) {
                    c.cols = Some((n as i32).max(1));
                }
            }
            Kind::Split => {
                if let Some(d) = o.get("SplitterDistance").and_then(num) {
                    c.set_prop("splitter_distance", Some(toml::Value::Integer(self.px(d).max(0) as i64)));
                }
                if o.get("Orientation").and_then(word) == Some("Horizontal") {
                    c.set_prop("orientation", Some(toml::Value::String("horizontal".into())));
                }
            }
            Kind::Statusbar => {
                let parts: Vec<&Obj> = o.items.iter().filter_map(|e| if let E::Path(p) = e { self.o(&p.join(".")) } else { None }).collect();
                let words: Vec<String> = parts.iter().filter_map(|p| p.get("Text").and_then(text)).filter(|t| !t.is_empty()).collect();
                c.parts = parts.iter().skip(1).map(|p| p.get("Size").and_then(pair).map(|(w, _)| self.px(w)).unwrap_or(100).max(10)).collect();
                if !words.is_empty() {
                    self.warn.push(format!("{}: the status bar's words ({}) are set from code", c.id, words.join(", ")));
                }
            }
            _ => {}
        }
    }

    /// What every control may have: fonts, colours, Enabled, a tooltip, the table's properties.
    fn common_props(&mut self, c: &mut Control, o: &Obj) {
        if o.get("Enabled") == Some(&E::Bool(false)) && !c.kind.is_component() {
            c.enabled = Some(false);
        }
        if let Some(E::New(_, a, _)) = o.get("Font") {
            let family = a.first().and_then(text);
            let mut size = a.get(1).and_then(num).unwrap_or(self.form_font);
            let rest: Vec<&str> = a.iter().skip(2).flat_map(flags).collect();
            if rest.contains(&"Pixel") {
                size *= 0.75;
            }
            if (size - self.form_font).abs() >= 0.5 {
                c.font_size = Some((size.round() as i32).clamp(6, 48));
            }
            if rest.contains(&"Bold") {
                c.bold = Some(true);
            }
            if let Some(fam) = family {
                if !fam.eq_ignore_ascii_case(&self.form_family) && !c.kind.is_component() {
                    c.set_prop("font_family", Some(toml::Value::String(fam)));
                }
            }
        }
        if !c.kind.is_component() {
            if matches!(c.kind, Kind::Edit | Kind::Textarea | Kind::Rich | Kind::Masked | Kind::Grid | Kind::List | Kind::Tree | Kind::ListBox) && o.get("BorderStyle").and_then(word) == Some("None") {
                c.set_prop("border_width", Some(toml::Value::Integer(0)));
            }
            c.back_color = o.get("BackColor").and_then(color);
            c.fore_color = o.get("ForeColor").and_then(color);
            if let Some(v) = o.get("FlatAppearance.BorderColor").and_then(color) {
                c.border_color = Some(v);
            }
            if let Some(n) = o.get("FlatAppearance.BorderSize").and_then(num) {
                if n as i32 != 1 {
                    c.set_prop("border_width", Some(toml::Value::Integer((n as i64).clamp(0, 8))));
                }
            }
            if let Some(t) = &o.tip {
                c.set_prop("tooltip", Some(toml::Value::String(t.clone())));
            }
            if let Some(E::Path(p)) = o.get("ContextMenuStrip") {
                if let Some(id) = self.ids.get(&p.join(".")) {
                    c.context_menu = Some(id.clone());
                }
            }
        }
        fn alias(n: &str) -> &str {
            match n {
            "AllowUserToAddRows" => "AllowAddRows",
            "AllowUserToDeleteRows" => "AllowDeleteRows",
            "AllowUserToOrderColumns" => "AllowColumnMoving",
            "ColumnHeadersVisible" => "ShowColumnHeaders",
            other => other,
            }
        }
        let short = o.short().to_string();
        for (name, e) in &o.props {
            let label = alias(name);
            let spec = props::of_kind(c.kind).find(|s| s.label == label);
            match spec {
                Some(s) => {
                    // Done by kind_props, or not the same thing (WinForms' Padding is inside, Phai's gap between).
                    if c.kind == Kind::Split && matches!(s.name, "splitter_distance" | "orientation") || name == "Padding" {
                        continue;
                    }
                    if let Some(v) = self.value(s, e) {
                        c.set_prop(s.name, Some(v));
                    }
                }
                None => {
                    if !QUIET.contains(&name.as_str()) && !name.starts_with("FlatAppearance.") {
                        self.left_out.insert(format!("{short}.{name}"));
                    }
                }
            }
        }
    }

    fn events(&mut self, c: &mut Control, o: &Obj) {
        let main = c.kind.event();
        for (ev, h) in &o.events {
            let is_main = matches!(
                (c.kind, ev.as_str()),
                (Kind::Button | Kind::Checkbox | Kind::Radio | Kind::NotifyIcon, "Click")
                    | (Kind::Link, "LinkClicked" | "Click")
                    | (Kind::Edit | Kind::Textarea | Kind::Rich | Kind::Masked, "TextChanged")
                    | (Kind::Combo | Kind::Tabs, "SelectedIndexChanged")
                    | (Kind::Date | Kind::Number | Kind::Slider, "ValueChanged")
                    | (Kind::Grid, "CellValueChanged")
                    | (Kind::Calendar, "DateChanged")
                    | (Kind::CheckedList, "ItemCheck")
                    | (Kind::Split, "SplitterMoved")
                    | (Kind::List | Kind::ListBox, "SelectedIndexChanged")
                    | (Kind::Tree, "AfterSelect")
                    | (Kind::Timer, "Tick")
            );
            if is_main && main.is_some() && c.handler().is_none() {
                let name = self.handler(h, crate::events::Args::None, &format!("{}_{}", c.id, ident(ev)));
                c.set_handler(Some(name));
                continue;
            }
            match crate::events::of_kind(c.kind).find(|s| s.label == ev) {
                Some(s) if !c.events.contains_key(s.name) => {
                    let name = self.handler(h, s.args, &crate::events::method_name(&c.id, s.name));
                    c.events.insert(s.name.to_string(), name);
                }
                Some(_) => {}
                None => {
                    self.left_out.insert(format!("{}.{ev} event", o.short()));
                }
            }
        }
    }

    /// Radio buttons that WinForms grouped by their panel: a number for each panel that holds some,
    /// when there is more than one.
    fn radio_groups(&mut self) {
        // The container each radio lies in, by the form's rectangles.
        let holders: Vec<Option<usize>> = (0..self.form.control.len()).map(|i| if self.form.control[i].kind == Kind::Radio { self.form.container_of(i) } else { None }).collect();
        let mut seen: Vec<Option<usize>> = Vec::new();
        for (i, h) in holders.iter().enumerate() {
            if self.form.control[i].kind == Kind::Radio && !seen.contains(h) {
                seen.push(*h);
            }
        }
        if seen.len() > 1 {
            for (i, h) in holders.iter().enumerate() {
                if self.form.control[i].kind == Kind::Radio {
                    self.form.control[i].group = seen.iter().position(|x| x == h).map(|n| n as i32 + 1);
                }
            }
        }
    }
}

/// A control with none of the samples a new one dropped in the designer gets ("One", "Two"...).
fn blank(kind: Kind, id: &str, bounds: [i32; 4]) -> Control {
    let mut c = Control::new(kind, id, bounds);
    c.text = None;
    c.items.clear();
    c.columns.clear();
    c.tabs.clear();
    c.value = None;
    c.cols = None;
    c.dock = None;
    c
}

/// `Keys.Control | Keys.S` -> `Ctrl+S`.
fn keys(e: &E) -> String {
    let f = flags(e);
    let mut out: Vec<String> = Vec::new();
    for (k, w) in [("Control", "Ctrl"), ("Shift", "Shift"), ("Alt", "Alt")] {
        if f.contains(&k) {
            out.push(w.into());
        }
    }
    for k in f.iter().filter(|k| !matches!(**k, "Control" | "Shift" | "Alt" | "None")) {
        let k = match *k {
            d if d.len() == 2 && d.starts_with('D') && d[1..].chars().all(|c| c.is_ascii_digit()) => d[1..].to_string(),
            "Delete" => "Del".into(),
            "Return" | "Enter" => "Enter".into(),
            "Escape" => "Esc".into(),
            other => other.into(),
        };
        out.push(k);
    }
    out.join("+")
}

/// The Phai control a WinForms type becomes; `None` for what is not a control (a TabPage is its
/// TabControl's, a ToolStripMenuItem its menu's) or unknown.
fn kind_of(o: &Obj) -> Option<Kind> {
    Some(match o.short() {
        "Label" => Kind::Label,
        "LinkLabel" => Kind::Link,
        "TextBox" => {
            if o.get("Multiline") == Some(&E::Bool(true)) {
                Kind::Textarea
            } else {
                Kind::Edit
            }
        }
        "MaskedTextBox" => Kind::Masked,
        "RichTextBox" => Kind::Rich,
        "Button" => Kind::Button,
        "CheckBox" => Kind::Checkbox,
        "RadioButton" => Kind::Radio,
        "ComboBox" => Kind::Combo,
        "ListBox" => Kind::ListBox,
        "CheckedListBox" => Kind::CheckedList,
        "ListView" => Kind::List,
        "DataGridView" => Kind::Grid,
        "TreeView" => Kind::Tree,
        "PictureBox" => Kind::Picture,
        "ProgressBar" => Kind::Progress,
        "TrackBar" => Kind::Slider,
        "NumericUpDown" => Kind::Number,
        "DateTimePicker" => Kind::Date,
        "MonthCalendar" => Kind::Calendar,
        "GroupBox" => Kind::Group,
        "Panel" => Kind::Panel,
        "FlowLayoutPanel" => Kind::Flow,
        "TableLayoutPanel" => Kind::Table,
        "TabControl" => Kind::Tabs,
        "SplitContainer" => Kind::Split,
        "StatusStrip" | "StatusBar" => Kind::Statusbar,
        "Timer" => Kind::Timer,
        "ContextMenuStrip" => Kind::ContextMenu,
        "OpenFileDialog" => Kind::OpenFile,
        "SaveFileDialog" => Kind::SaveFile,
        "FolderBrowserDialog" => Kind::FolderDialog,
        "ColorDialog" => Kind::ColorDialog,
        "FontDialog" => Kind::FontDialog,
        "ErrorProvider" => Kind::ErrorProvider,
        "NotifyIcon" => Kind::NotifyIcon,
        "PrintDocument" => Kind::PrintDocument,
        "BindingSource" => Kind::BindingSource,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const NEW_STYLE: &str = r#"
namespace Shop
{
    partial class OrderForm
    {
        private void InitializeComponent()
        {
            components = new System.ComponentModel.Container();
            _menu = new MenuStrip();
            _menuFile = new ToolStripMenuItem();
            _menuFileSave = new ToolStripMenuItem();
            _menuFileSep = new ToolStripSeparator();
            _menuFileExit = new ToolStripMenuItem();
            _top = new Panel();
            _title = new Label();
            _name = new TextBox();
            _ok = new Button();
            _kind = new ComboBox();
            _clock = new System.Windows.Forms.Timer(components);
            _menu.SuspendLayout();
            SuspendLayout();
            //
            // _menu
            //
            _menu.Items.AddRange(new ToolStripItem[] { _menuFile });
            _menu.Location = new Point(0, 0);
            _menu.Size = new Size(800, 48);
            _menuFile.DropDownItems.AddRange(new ToolStripItem[] { _menuFileSave, _menuFileSep, _menuFileExit });
            _menuFile.Text = "&File";
            _menuFileSave.ShortcutKeys = Keys.Control | Keys.S;
            _menuFileSave.Text = "Save";
            _menuFileSave.Click += Save_Click;
            _menuFileExit.Text = "Exit";
            //
            // _top
            //
            _top.BackColor = Color.FromArgb(18, 133, 95);
            _top.Controls.Add(_title);
            _top.Dock = DockStyle.Top;
            _top.Location = new Point(0, 48);
            _top.Size = new Size(800, 120);
            _title.Font = new Font("Segoe UI", 14F, FontStyle.Bold);
            _title.ForeColor = Color.White;
            _title.Location = new Point(24, 30);
            _title.Size = new Size(400, 60);
            _title.Text = "New order";
            _name.Anchor = AnchorStyles.Top | AnchorStyles.Left | AnchorStyles.Right;
            _name.Location = new Point(24, 200);
            _name.Size = new Size(500, 40);
            _name.TextChanged += Name_TextChanged;
            _name.KeyDown += Name_KeyDown;
            _ok.Anchor = AnchorStyles.Bottom | AnchorStyles.Right;
            _ok.Location = new Point(600, 500);
            _ok.Size = new Size(150, 46);
            _ok.Text = "OK";
            _ok.DialogResult = DialogResult.OK;
            _ok.Click += Ok_Click;
            _kind.Items.AddRange(new object[] { "Shop", "Phone" });
            _kind.DropDownStyle = ComboBoxStyle.DropDownList;
            _kind.Location = new Point(24, 260);
            _kind.Size = new Size(240, 40);
            _clock.Interval = 500;
            _clock.Tick += Clock_Tick;
            //
            // OrderForm
            //
            AutoScaleDimensions = new SizeF(13F, 32F);
            AutoScaleMode = AutoScaleMode.Font;
            ClientSize = new Size(800, 600);
            Controls.Add(_kind);
            Controls.Add(_ok);
            Controls.Add(_name);
            Controls.Add(_top);
            Controls.Add(_menu);
            AcceptButton = _ok;
            FormBorderStyle = FormBorderStyle.FixedDialog;
            MaximizeBox = false;
            Text = "Shop order";
            FormClosing += OrderForm_FormClosing;
            ResumeLayout(false);
        }
    }
}
"#;

    #[test]
    fn a_dot_net_form_designed_at_200_percent_comes_over_at_100() {
        let im = import(NEW_STYLE).unwrap();
        let f = &im.form;
        assert_eq!(f.check(), vec![], "{}", f.to_toml());
        assert_eq!(f.form.name, "OrderForm");
        assert_eq!(f.form.title, "Shop order");
        // 800 x 600 at 200%, the 48 px menu (24 at 100%) outside Phai's window area.
        assert_eq!(f.form.size, [400, 276]);
        assert_eq!(f.form.word("border"), Some("fixed_dialog"));
        assert_eq!(f.form.prop("maximize_box"), Some(&toml::Value::Boolean(false)));
        assert_eq!(f.form.word("accept_button"), Some("ok"));
        assert_eq!(f.form.events.get("closing").map(String::as_str), Some("order_form_form_closing"));
        // The menu.
        assert_eq!(f.menu.len(), 1);
        assert_eq!(f.menu[0].label, "&File");
        let items = &f.menu[0].items;
        assert_eq!(items[0].id.as_deref(), Some("save_click"));
        assert_eq!(items[0].key.as_deref(), Some("Ctrl+S"));
        assert!(items[1].separator);
        assert_eq!(items[2].id.as_deref(), Some("menu_file_exit"));
        // Docked first, in WinForms' docking order; what it holds after it.
        let ids: Vec<&str> = f.control.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, ["top", "title", "name", "ok", "kind", "clock"]);
        let c = |id: &str| f.control.iter().find(|c| c.id == id).unwrap();
        assert_eq!(c("top").dock.as_deref(), Some("top"));
        assert_eq!(c("top").back_color.as_deref(), Some("#12855f"));
        assert_eq!(c("title").bounds, Some([12, 15, 200, 30]));
        assert_eq!(c("title").font_size, Some(14));
        assert_eq!(c("title").bold, Some(true));
        assert_eq!(c("title").fore_color.as_deref(), Some("white"));
        // Inside a panel docked along the top: it keeps to the left and the top.
        assert_eq!(c("title").anchor, None);
        assert_eq!(c("name").bounds, Some([12, 76, 250, 20]));
        assert_eq!(c("name").anchor.as_deref(), Some("left top right"));
        assert_eq!(c("name").on_change.as_deref(), Some("name_text_changed"));
        assert_eq!(c("name").events.get("key_down").map(String::as_str), Some("name_key_down"));
        assert_eq!(c("ok").anchor.as_deref(), Some("right bottom"));
        assert_eq!(c("ok").on_click.as_deref(), Some("ok_click"));
        assert_eq!(c("ok").word("dialog_result"), Some("ok"));
        assert_eq!(c("kind").items, ["Shop", "Phone"]);
        assert_eq!(c("kind").prop("editable"), None);
        assert_eq!(c("clock").value, Some(500));
        assert_eq!(c("clock").on_tick.as_deref(), Some("clock_tick"));
        // It writes and reads back as any form does.
        let back = FormFile::parse(&f.to_toml()).unwrap();
        assert_eq!(&back, f);
    }

    #[test]
    fn an_old_style_form_with_this_casts_and_full_names_reads_the_same() {
        let src = r#"
partial class SetupForm {
    private void InitializeComponent() {
        this.groupBox1 = new System.Windows.Forms.GroupBox();
        this.radioA = new System.Windows.Forms.RadioButton();
        this.radioB = new System.Windows.Forms.RadioButton();
        this.count = new System.Windows.Forms.NumericUpDown();
        this.tabs = new System.Windows.Forms.TabControl();
        this.page1 = new System.Windows.Forms.TabPage();
        this.page2 = new System.Windows.Forms.TabPage();
        this.check = new System.Windows.Forms.CheckBox();
        ((System.ComponentModel.ISupportInitialize)(this.count)).BeginInit();
        this.groupBox1.Controls.Add(this.radioA);
        this.groupBox1.Controls.Add(this.radioB);
        this.groupBox1.Location = new System.Drawing.Point(12, 12);
        this.groupBox1.Size = new System.Drawing.Size(200, 100);
        this.groupBox1.Text = "Mode";
        this.radioA.Checked = true;
        this.radioA.Location = new System.Drawing.Point(10, 20);
        this.radioA.Size = new System.Drawing.Size(100, 20);
        this.radioA.Text = "Automatic";
        this.radioB.Location = new System.Drawing.Point(10, 50);
        this.radioB.Size = new System.Drawing.Size(100, 20);
        this.radioB.Text = @"By ""hand""";
        this.count.Location = new System.Drawing.Point(230, 12);
        this.count.Maximum = new decimal(new int[] { 500, 0, 0, 0 });
        this.count.Value = new decimal(new int[] { 25, 0, 0, 0 });
        this.count.ValueChanged += new System.EventHandler(this.Count_ValueChanged);
        this.tabs.Controls.Add(this.page1);
        this.tabs.Controls.Add(this.page2);
        this.tabs.Location = new System.Drawing.Point(12, 130);
        this.tabs.Size = new System.Drawing.Size(300, 150);
        this.page1.Text = "General";
        this.page2.Controls.Add(this.check);
        this.page2.Text = "More";
        this.check.Location = new System.Drawing.Point(6, 6);
        this.check.Size = new System.Drawing.Size(120, 20);
        this.check.Text = "Dark\tcolours";
        this.check.ForeColor = System.Drawing.Color.FromArgb(((int)(((byte)(11)))), ((int)(((byte)(61)))), ((int)(((byte)(32)))));
        this.check.Font = new System.Drawing.Font("Consolas", 9F, System.Drawing.FontStyle.Regular, System.Drawing.GraphicsUnit.Point, ((byte)(0)));
        this.AutoScaleDimensions = new System.Drawing.SizeF(6F, 13F);
        this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
        this.ClientSize = new System.Drawing.Size(340, 300);
        this.Controls.Add(this.tabs);
        this.Controls.Add(this.count);
        this.Controls.Add(this.groupBox1);
        this.Name = "SetupForm";
        this.Text = "Setup";
        ((System.ComponentModel.ISupportInitialize)(this.count)).EndInit();
        this.ResumeLayout(false);
    }
}
"#;
        let im = import(src).unwrap();
        let f = &im.form;
        assert_eq!(f.check(), vec![], "{}", f.to_toml());
        assert_eq!(f.form.size, [340, 300]);
        let c = |id: &str| f.control.iter().find(|c| c.id == id).unwrap();
        assert_eq!(c("group_box1").text.as_deref(), Some("Mode"));
        assert_eq!(c("radio_a").bounds, Some([22, 32, 100, 20]));
        assert_eq!(c("radio_a").checked, Some(true));
        assert_eq!(c("radio_b").text.as_deref(), Some("By \"hand\""));
        assert_eq!((c("count").max, c("count").value), (Some(500), Some(25)));
        assert_eq!(c("count").on_change.as_deref(), Some("count_value_changed"));
        assert_eq!(c("tabs").tabs, ["General", "More"]);
        assert_eq!(c("tabs").bounds, Some([12, 130, 300, 34]));
        assert_eq!(c("check").page, Some(1));
        assert_eq!(c("check").bounds, Some([22, 170, 120, 20]));
        assert_eq!(c("check").text.as_deref(), Some("Dark\tcolours"));
        assert_eq!(c("check").fore_color.as_deref(), Some("#0b3d20"));
        assert_eq!(c("check").word("font_family"), Some("Consolas"));
        assert!(FormFile::parse(&f.to_toml()).is_ok());
    }

    #[test]
    fn names_become_rust_names() {
        assert_eq!(ident("_btnOK"), "btn_ok");
        assert_eq!(ident("MainForm"), "main_form");
        assert_eq!(ident("dataGridViewTextBoxColumn1"), "data_grid_view_text_box_column1");
        assert_eq!(ident("Fixed3D"), "fixed3_d");
        assert_eq!(ident("type"), "type_");
        assert_eq!(pascal("main_form"), "MainForm");
        assert_eq!(choice("MiddleRight", &["left", "center", "right"]).as_deref(), Some("right"));
        assert_eq!(choice("LargeIcon", &["details", "list", "small_icons", "large_icons"]).as_deref(), Some("large_icons"));
        assert_eq!(choice("FixedSingle", &["none", "fixed"]).as_deref(), Some("fixed"));
        assert_eq!(choice("FixedSingle", &["sizable", "fixed_single"]).as_deref(), Some("fixed_single"));
        assert_eq!(keys(&E::Or(vec![E::Path(vec!["Keys".into(), "Control".into()]), E::Path(vec!["Keys".into(), "D1".into()])])), "Ctrl+1");
    }

    #[test]
    fn what_is_not_a_designer_file_is_said() {
        assert!(import("class X { void Foo() {} }").is_err());
    }

    /// The 21 forms of an older C# program, when they are on this computer: each comes over and
    /// is a form the designer opens.
    #[test]
    fn real_designer_files_all_come_over() {
        let dir = std::path::Path::new(r"D:\Project.NET\phai-desk");
        let Ok(list) = std::fs::read_dir(dir) else { return };
        let mut n = 0;
        for e in list.flatten() {
            let p = e.path();
            if !p.to_string_lossy().ends_with(".Designer.cs") {
                continue;
            }
            let src = std::fs::read_to_string(&p).unwrap();
            let im = import(&src).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            let toml = im.form.to_toml();
            assert_eq!(im.form.check(), vec![], "{}:\n{toml}", p.display());
            FormFile::parse(&toml).unwrap_or_else(|e| panic!("{}: {e}\n{toml}", p.display()));
            n += 1;
        }
        assert!(n == 0 || n >= 20, "{n}");
    }
}

#[cfg(test)]
mod dump {
    /// `PHAI_WF_OUT=folder cargo test -p phai-form dump_real -- --ignored`: every real form written
    /// out with its warnings, to look at.
    #[test]
    #[ignore]
    fn dump_real() {
        let out = std::path::PathBuf::from(std::env::var("PHAI_WF_OUT").unwrap());
        std::fs::create_dir_all(&out).unwrap();
        for e in std::fs::read_dir(r"D:\Project.NET\phai-desk").unwrap().flatten() {
            let p = e.path();
            let name = p.file_name().unwrap().to_string_lossy().to_string();
            let Some(stem) = name.strip_suffix(".Designer.cs") else { continue };
            let im = super::import(&std::fs::read_to_string(&p).unwrap()).unwrap();
            std::fs::write(out.join(format!("{}.phf", super::ident(stem))), im.form.to_toml()).unwrap();
            println!("== {stem}: {}", im.warnings.join("\n   "));
        }
    }
}
