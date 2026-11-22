//! Build > Build for Web: the project as a web page that installs as an app on Android and iOS
//! (a PWA). Every form's `src/web/<form>_ui.rs` written from its `.phf`, the project told to
//! take `phai-web` for `phai-ui` when it builds for `wasm32` (its own code stays as it is), cargo
//! and wasm-bindgen run, and `web/` given the page, the theme, a manifest, a service worker and
//! icons. Run in Browser serves `web/` on this computer and opens it.

use std::io::{BufRead, BufReader, Read, Write as _};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::process::Command;

use phai_form::FormFile;

/// The wasm-bindgen that phai-web is built against: the command-line tool must be the same.
pub const WASM_BINDGEN: &str = "0.2.130";

/// Where phai-web is on this machine: beside Phai Studio when installed, else the source.
pub fn phai_web_path() -> PathBuf {
    if let Some(installed) = std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("phai-web"))).filter(|d| d.join("Cargo.toml").is_file()) {
        return installed;
    }
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("crates").join("phai-web");
    here.canonicalize().map(|p| PathBuf::from(p.to_string_lossy().trim_start_matches(r"\\?\"))).unwrap_or(here)
}

/// phai-web as the project names it for the web; main.rs calls it `phai_ui` there
/// ([`web_main_rs`]), so the program's code needs no change.
pub fn phai_web_dependency() -> String {
    crate::project::library_dependency(&phai_web_path())
}

const NATIVE: &str = "[target.'cfg(not(target_arch = \"wasm32\"))'.dependencies]";
const WEB: &str = "[target.'cfg(target_arch = \"wasm32\")'.dependencies]";

/// The project's Cargo.toml taking `phai-web` for `phai-ui` on the web: phai-ui, windows and
/// phai-data moved to Windows' own dependencies. Once is enough; a second time changes nothing.
pub fn web_cargo_toml(text: &str, web_dep: &str) -> String {
    if text.contains("target_arch = \"wasm32\"") {
        return text.to_string();
    }
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<String> = Vec::new();
    let mut native: Vec<String> = Vec::new();
    let mut in_deps = false;
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i];
        let t = l.trim();
        if t.starts_with('[') && !t.starts_with("[[") {
            in_deps = t == "[dependencies]";
        }
        let key = t.split(['=', ' ']).next().unwrap_or("");
        if in_deps && matches!(key, "phai-ui" | "windows" | "phai-data") {
            // The entry, over as many lines as its brackets take.
            let mut entry = vec![l.to_string()];
            let mut depth = balance(l);
            while depth > 0 && i + 1 < lines.len() {
                i += 1;
                entry.push(lines[i].to_string());
                depth += balance(lines[i]);
            }
            native.extend(entry);
        } else {
            out.push(l.to_string());
        }
        i += 1;
    }
    // After [dependencies], before the next table.
    let at = out.iter().position(|l| l.trim() == "[dependencies]").map(|p| out.iter().skip(p + 1).position(|l| l.trim().starts_with('[')).map(|q| p + 1 + q).unwrap_or(out.len())).unwrap_or(out.len());
    let mut block = vec![NATIVE.to_string()];
    block.extend(native);
    block.push(String::new());
    block.push(WEB.to_string());
    block.push(format!("phai-web = {web_dep}"));
    block.push(String::new());
    let mut result = out[..at].to_vec();
    if result.last().is_some_and(|l| !l.trim().is_empty()) {
        result.push(String::new());
    }
    result.extend(block);
    result.extend(out[at..].iter().cloned());
    let mut s = result.join("\n");
    s.push('\n');
    s
}

/// How many more brackets a line opens than it closes.
fn balance(l: &str) -> i32 {
    l.chars().map(|c| match c {
        '[' | '{' => 1,
        ']' | '}' => -1,
        _ => 0,
    }).sum()
}

/// What main.rs says on the web: phai-web is `phai_ui` there, for every file of the program.
const AS_PHAI_UI: &str = "#[cfg(target_arch = \"wasm32\")]\nextern crate phai_web as phai_ui;\n";

/// main.rs with phai-web called `phai_ui` on the web, each form's `_ui` module taken from
/// `src/web/` there, and the database's module left to Windows.
pub fn web_main_rs(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = String::new();
    let first_mod = if text.contains("extern crate phai_web") { None } else { lines.iter().position(|l| l.trim_start().starts_with("mod ")) };
    for (i, l) in lines.iter().enumerate() {
        let t = l.trim();
        if Some(i) == first_mod {
            out.push_str(AS_PHAI_UI);
            out.push('\n');
        }
        let before = if i > 0 { lines[i - 1].trim() } else { "" };
        if let Some(m) = t.strip_prefix("mod ").and_then(|r| r.strip_suffix(';')) {
            if m.ends_with("_ui") && !before.contains("target_arch") {
                out.push_str(&format!("#[cfg_attr(target_arch = \"wasm32\", path = \"web/{m}.rs\")]\n"));
            } else if m == "data_sources" && !before.contains("target_arch") {
                out.push_str("#[cfg(not(target_arch = \"wasm32\"))]\n");
            }
        }
        out.push_str(l);
        out.push('\n');
    }
    out
}

/// The project's forms: each `.phf` under `src`.
fn forms(src: &Path) -> Vec<(PathBuf, FormFile)> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(src) else { return out };
    for e in rd.flatten() {
        let p = e.path();
        if phai_form::is_form(&p) {
            if let Some(f) = std::fs::read_to_string(&p).ok().and_then(|t| FormFile::parse(&t).ok()) {
                out.push((p, f));
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// wasm-bindgen's command-line tool, on the PATH or where cargo installs.
fn wasm_bindgen() -> Option<PathBuf> {
    let name = "wasm-bindgen.exe";
    let mut places: Vec<PathBuf> = std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect()).unwrap_or_default();
    if let Some(home) = std::env::var_os("CARGO_HOME") {
        places.push(PathBuf::from(home).join("bin"));
    }
    if let Some(home) = std::env::var_os("USERPROFILE") {
        places.push(PathBuf::from(home).join(".cargo").join("bin"));
    }
    if let Some(c) = crate::cargo::find_cargo() {
        if let Some(d) = c.parent() {
            places.push(d.to_path_buf());
        }
    }
    places.into_iter().map(|d| d.join(name)).find(|p| p.is_file())
}

/// A square app icon: the theme's colour, a white leaf of Phai's bamboo across it.
pub fn icon_png(size: u32, color: (u8, u8, u8)) -> Vec<u8> {
    let s = size as f32;
    let mut px = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            // A leaf: two arcs meeting at its tips, slanted.
            let (cx, cy) = (fx - s / 2.0, fy - s / 2.0);
            let (u, v) = ((cx + cy) / std::f32::consts::SQRT_2, (cy - cx) / std::f32::consts::SQRT_2);
            let half = s * 0.34;
            let leaf = u.abs() < half && v.abs() < (1.0 - (u / half).powi(2)) * s * 0.13;
            let i = ((y * size + x) * 4) as usize;
            let (r, g, b) = if leaf { (255, 255, 255) } else { color };
            px[i..i + 4].copy_from_slice(&[r, g, b, 255]);
        }
    }
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, size, size);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        if let Ok(mut w) = enc.write_header() {
            let _ = w.write_image_data(&px);
        }
    }
    out
}

fn hex(c: &str) -> (u8, u8, u8) {
    let c = c.trim_start_matches('#');
    let p = |i: usize| u8::from_str_radix(c.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0);
    if c.len() >= 6 {
        (p(0), p(2), p(4))
    } else {
        (0x12, 0x85, 0x5f)
    }
}

/// What a build for the web made: the folder to serve, what the forms have that a browser does
/// not (left out), and cargo's words when it failed.
#[derive(Debug, Default)]
pub struct WebBuild {
    pub folder: PathBuf,
    pub left_out: Vec<String>,
    /// A grid is bound to a table: `data-server/` written, the web app's tables come from it.
    pub data_server: bool,
}

/// Builds `project` for the web, telling `say` each step. The `web/` folder ready to serve.
pub fn build(project: &Path, mut say: impl FnMut(&str)) -> Result<WebBuild, String> {
    let src = project.join("src");
    let package = crate::cargo::package_name(project).ok_or("the project's Cargo.toml has no package name")?;
    let all = forms(&src);
    if all.is_empty() {
        return Err("the project has no form".into());
    }
    let mut left_out = Vec::new();
    std::fs::create_dir_all(src.join("web")).map_err(|e| e.to_string())?;
    for (path, f) in &all {
        let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        std::fs::write(src.join("web").join(format!("{stem}_ui.rs")), phai_form::web::generate(f)).map_err(|e| e.to_string())?;
        left_out.extend(phai_form::web::left_out(f).into_iter().map(|x| format!("{}: {x}", f.form.name)));
    }
    let toml_path = project.join("Cargo.toml");
    let toml = std::fs::read_to_string(&toml_path).map_err(|e| e.to_string())?;
    // The data server, when a grid is bound: its main written again, its Cargo.toml once.
    let data_server = match phai_form::web::data_server_main(&all.iter().map(|(_, f)| f).collect::<Vec<_>>()) {
        Some(main) => {
            let dir = project.join("data-server");
            std::fs::create_dir_all(dir.join("src")).map_err(|e| e.to_string())?;
            std::fs::write(dir.join("src").join("main.rs"), main).map_err(|e| e.to_string())?;
            if !dir.join("Cargo.toml").exists() {
                let dep = toml.lines().find_map(|l| l.trim().strip_prefix("phai-data").map(|r| r.trim_start().trim_start_matches('=').trim().to_string())).unwrap_or_else(crate::project::phai_data_dependency);
                std::fs::write(dir.join("Cargo.toml"), phai_form::web::data_server_toml(&package, &dep)).map_err(|e| e.to_string())?;
            }
            true
        }
        None => false,
    };
    let new_toml = web_cargo_toml(&toml, &phai_web_dependency());
    if new_toml != toml {
        std::fs::write(&toml_path, new_toml).map_err(|e| e.to_string())?;
    }
    let main_path = src.join("main.rs");
    if let Ok(main) = std::fs::read_to_string(&main_path) {
        let new_main = web_main_rs(&main);
        if new_main != main {
            std::fs::write(&main_path, new_main).map_err(|e| e.to_string())?;
        }
    }

    say("cargo build --target wasm32-unknown-unknown --release");
    let cargo = crate::cargo::find_cargo().ok_or("cargo is not installed - rustup.rs")?;
    let out = Command::new(&cargo).args(["build", "--release", "--target", "wasm32-unknown-unknown", "--message-format=short"]).env("RUSTFLAGS", remapped_paths(project)).current_dir(project).output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).to_string();
        if err.contains("target may not be installed") || err.contains("can't find crate for `core`") || err.contains("can't find crate for `std`") {
            return Err("Rust's WebAssembly library is not installed: rustup target add wasm32-unknown-unknown".into());
        }
        let lines: Vec<&str> = err.lines().filter(|l| l.contains("error")).take(12).collect();
        return Err(if lines.is_empty() { err } else { lines.join("\n") });
    }
    let wasm = target_dir(project).join("wasm32-unknown-unknown").join("release").join(format!("{package}.wasm"));
    say("wasm-bindgen");
    let bindgen = wasm_bindgen().ok_or_else(|| format!("wasm-bindgen is not installed: cargo install wasm-bindgen-cli --version {WASM_BINDGEN}"))?;
    let web = project.join("web");
    let pkg = web.join("pkg");
    std::fs::create_dir_all(&pkg).map_err(|e| e.to_string())?;
    let out = Command::new(&bindgen).arg(&wasm).args(["--target", "web", "--no-typescript", "--out-dir"]).arg(&pkg).output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }

    say("index.html, manifest, service worker");
    let theme = std::fs::read_to_string(project.join("theme.css")).unwrap_or_else(|_| "@import \"phai:light\";".into());
    let main_form = all.iter().find(|(p, _)| p.file_stem().is_some_and(|s| s == "main_window")).or(all.first()).map(|(_, f)| f).ok_or("no form")?;
    let app = WebApp::of(project, &main_form.form.title, &theme);
    let version = version_stamp();
    let icon = |size: u32| app.icon_file(project).and_then(|f| scaled_icon(&f, size, hex(&app.color))).unwrap_or_else(|| icon_png(size, hex(&app.color)));
    let files: Vec<(&str, Vec<u8>)> = vec![
        ("index.html", phai_form::web::index_html(&app.name, &package, &(page_css(project, &theme) + "\n/* grids */\n" + &phai_form::web::grids_css(&all.iter().map(|(_, f)| f).collect::<Vec<_>>())), &app.color, &version).into_bytes()),
        ("manifest.webmanifest", phai_form::web::manifest(&app.name, &app.short, &app.color, &app.background, &app.orientation).into_bytes()),
        ("sw.js", phai_form::web::service_worker(&package, &version).into_bytes()),
        ("icon-192.png", icon(192)),
        ("icon-512.png", icon(512)),
    ];
    for (name, data) in files {
        std::fs::write(web.join(name), data).map_err(|e| e.to_string())?;
    }
    // The data server elsewhere than beside the page: Cargo.toml's `data-url`, in the page.
    let data_url = std::fs::read_to_string(&toml_path).ok().and_then(|t| crate::cargo::table_field(&t, crate::cargo::WEB_APP_TABLE, "data-url")).filter(|u| !u.trim().is_empty());
    if let Some(u) = data_url {
        let index = web.join("index.html");
        if let Ok(html) = std::fs::read_to_string(&index) {
            let meta = format!("<meta name=\"phai-data\" content=\"{}\">", u.trim().replace('"', "&quot;"));
            std::fs::write(&index, html.replacen("<head>", &format!("<head>\n{meta}"), 1)).map_err(|e| e.to_string())?;
        }
    }
    Ok(WebBuild { folder: web, left_out, data_server })
}

/// The page's own CSS: the theme's, made the web's, then the project's `web.css` as written -
/// CSS for the web alone (`@media`, `.phai-stacked`, a control's `#id`), after the theme so it wins.
pub fn page_css(project: &Path, theme: &str) -> String {
    let mut css = phai_form::web::theme_css(theme);
    if let Ok(own) = std::fs::read_to_string(project.join("web.css")) {
        css.push_str("\n/* web.css */\n");
        // Never the end of the page's <style>.
        css.push_str(&own.replace("</style", "<\\/style"));
    }
    css
}

/// How the program shows as an app on a phone - Project > Web App (PWA): kept in Cargo.toml's
/// `[package.metadata.phai-web]`, each one the form's and the theme's when not set.
#[derive(Clone, Debug, PartialEq)]
pub struct WebApp {
    /// Its name: the page's title, the app's name when installed.
    pub name: String,
    /// The name under its icon on the home screen; the first 12 letters of `name` when empty.
    pub short: String,
    /// The bar's colour, `#rrggbb`.
    pub color: String,
    /// The screen while it starts.
    pub background: String,
    /// any, portrait or landscape.
    pub orientation: String,
    /// A PNG of the project's for its icon; the leaf in `color` when empty.
    pub icon: String,
}

/// The keys in Cargo.toml, in [`WebApp`]'s order.
pub const WEB_APP_KEYS: [&str; 6] = ["name", "short-name", "theme-color", "background-color", "orientation", "icon"];

impl WebApp {
    pub fn of(project: &Path, title: &str, theme: &str) -> WebApp {
        let text = std::fs::read_to_string(project.join("Cargo.toml")).unwrap_or_default();
        let get = |k: &str| crate::cargo::table_field(&text, crate::cargo::WEB_APP_TABLE, k).filter(|v| !v.trim().is_empty());
        WebApp {
            name: get("name").unwrap_or_else(|| title.to_string()),
            short: get("short-name").unwrap_or_default(),
            color: get("theme-color").filter(|c| is_colour(c)).or_else(|| phai_form::web::theme_var(theme, "--accent")).unwrap_or_else(|| "#12855f".into()),
            background: get("background-color").filter(|c| is_colour(c)).or_else(|| phai_form::web::theme_var(theme, "--surface")).unwrap_or_else(|| "#ffffff".into()),
            orientation: get("orientation").unwrap_or_else(|| "any".into()),
            icon: get("icon").unwrap_or_default(),
        }
    }

    /// The icon's file, when one is set and is there.
    pub fn icon_file(&self, project: &Path) -> Option<PathBuf> {
        let i = self.icon.trim();
        if i.is_empty() {
            return None;
        }
        let p = Path::new(i);
        let p = if p.is_absolute() { p.to_path_buf() } else { project.join(p) };
        p.is_file().then_some(p)
    }
}

/// `#rrggbb`.
pub fn is_colour(c: &str) -> bool {
    c.len() == 7 && c.starts_with('#') && c[1..].chars().all(|d| d.is_ascii_hexdigit())
}

/// A PNG of any size made a square icon of `size`: fitted in the middle on `back`.
pub fn scaled_icon(file: &Path, size: u32, back: (u8, u8, u8)) -> Option<Vec<u8>> {
    let mut decoder = png::Decoder::new(std::fs::File::open(file).ok()?);
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let (sw, sh) = (info.width as usize, info.height as usize);
    let channels = match info.color_type {
        png::ColorType::Rgba => 4,
        png::ColorType::Rgb => 3,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Grayscale => 1,
        _ => return None,
    };
    if sw == 0 || sh == 0 {
        return None;
    }
    let src = |x: usize, y: usize| -> [f32; 4] {
        let i = (y * sw + x) * channels;
        let p = &buf[i..i + channels];
        match channels {
            4 => [p[0] as f32, p[1] as f32, p[2] as f32, p[3] as f32],
            3 => [p[0] as f32, p[1] as f32, p[2] as f32, 255.0],
            2 => [p[0] as f32, p[0] as f32, p[0] as f32, p[1] as f32],
            _ => [p[0] as f32, p[0] as f32, p[0] as f32, 255.0],
        }
    };
    let size_u = size as usize;
    // Fitted: the longer side fills the icon.
    let scale = sw.max(sh) as f32 / size as f32;
    let (dw, dh) = ((sw as f32 / scale).round() as usize, (sh as f32 / scale).round() as usize);
    let (ox, oy) = ((size_u - dw.min(size_u)) / 2, (size_u - dh.min(size_u)) / 2);
    let mut px = vec![0u8; size_u * size_u * 4];
    for y in 0..size_u {
        for x in 0..size_u {
            let mut c = [0f32; 4];
            if x >= ox && x < ox + dw && y >= oy && y < oy + dh {
                // The average of the source pixels this one covers.
                let (x0, x1) = (((x - ox) as f32 * scale) as usize, ((((x - ox) + 1) as f32 * scale) as usize).clamp(1, sw));
                let (y0, y1) = (((y - oy) as f32 * scale) as usize, ((((y - oy) + 1) as f32 * scale) as usize).clamp(1, sh));
                let (x0, y0) = (x0.min(x1 - 1), y0.min(y1 - 1));
                let n = ((x1 - x0) * (y1 - y0)) as f32;
                for sy in y0..y1 {
                    for sx in x0..x1 {
                        let p = src(sx, sy);
                        for k in 0..4 {
                            c[k] += p[k] / n;
                        }
                    }
                }
            }
            // On the background: a phone's icon has no see-through parts.
            let a = c[3] / 255.0;
            let i = (y * size_u + x) * 4;
            let mix = |v: f32, b: u8| (v * a + b as f32 * (1.0 - a)).round().clamp(0.0, 255.0) as u8;
            px[i..i + 4].copy_from_slice(&[mix(c[0], back.0), mix(c[1], back.1), mix(c[2], back.2), 255]);
        }
    }
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, size, size);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header().ok()?;
        w.write_image_data(&px).ok()?;
    }
    Some(out)
}

/// RUSTFLAGS that keep this computer's folders out of the program a web page hands to anyone:
/// the project, the Phai libraries, cargo's downloads and Rust's own library each by a short name
/// in the places a panic would name.
fn remapped_paths(project: &Path) -> String {
    let mut pairs: Vec<(PathBuf, &str)> = vec![(project.to_path_buf(), "app")];
    if let Some(libs) = phai_web_path().parent().map(Path::to_path_buf) {
        pairs.push((libs, "phai"));
    }
    let cargo_home = std::env::var_os("CARGO_HOME").map(PathBuf::from).or_else(|| std::env::var_os("USERPROFILE").map(|h| PathBuf::from(h).join(".cargo")));
    if let Some(h) = cargo_home {
        pairs.push((h, "cargo"));
    }
    if let Ok(out) = Command::new("rustc").args(["--print", "sysroot"]).output() {
        let root = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !root.is_empty() {
            pairs.push((PathBuf::from(root), "rust"));
        }
    }
    let mut flags: Vec<String> = std::env::var("RUSTFLAGS").ok().filter(|s| !s.is_empty()).into_iter().collect();
    for (from, to) in pairs {
        let from = from.canonicalize().map(|p| PathBuf::from(p.to_string_lossy().trim_start_matches(r"\\?\"))).unwrap_or(from);
        // Spaces would split RUSTFLAGS; such a folder keeps its name.
        if !from.to_string_lossy().contains(' ') {
            flags.push(format!("--remap-path-prefix={}={to}", from.display()));
        }
    }
    flags.join(" ")
}

/// Where cargo puts the project's builds: `CARGO_TARGET_DIR`, else `target`.
fn target_dir(project: &Path) -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR").map(PathBuf::from).unwrap_or_else(|| project.join("target"))
}

fn version_stamp() -> String {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_default()
}

fn mime(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "wasm" => "application/wasm",
        "css" => "text/css; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "webmanifest" | "json" => "application/manifest+json",
        _ => "application/octet-stream",
    }
}

/// The file a request asks for under `root`, never above it.
pub fn file_for(root: &Path, request_line: &str) -> Option<PathBuf> {
    let path = request_line.split_whitespace().nth(1)?;
    let path = path.split(['?', '#']).next()?;
    let mut out = root.to_path_buf();
    for part in path.split('/').filter(|p| !p.is_empty()) {
        if part == ".." || part.contains('\\') || part.contains(':') {
            return None;
        }
        out.push(part);
    }
    if out.is_dir() {
        out.push("index.html");
    }
    out.is_file().then_some(out)
}

/// Serves `root` on this computer only: the port. Its own page's data requests are given the
/// session's key ([`session_key`]).
pub fn serve(root: PathBuf) -> std::io::Result<u16> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    serve_with(listener, Arc::new(Mutex::new(root)), false, true);
    Ok(port)
}

/// The port Run on Phone tries first, so a phone keeps its address from one day to the next.
pub const PHONE_PORT: u16 = 8790;

/// The server for phones, once started: its folder (changed for another project) and port.
static LAN: Mutex<Option<(Arc<Mutex<PathBuf>>, u16)>> = Mutex::new(None);

/// Serves `root` to the computers and phones on this network - one server for the whole run of
/// Studio, the folder changed when another project is built. Its pages reload when the build
/// changes. The port.
pub fn serve_lan(root: PathBuf) -> std::io::Result<u16> {
    let mut lan = LAN.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((folder, port)) = lan.as_ref() {
        *folder.lock().unwrap_or_else(|e| e.into_inner()) = root;
        return Ok(*port);
    }
    let listener = std::net::TcpListener::bind(("0.0.0.0", PHONE_PORT)).or_else(|_| std::net::TcpListener::bind("0.0.0.0:0"))?;
    let port = listener.local_addr()?.port();
    let folder = Arc::new(Mutex::new(root));
    serve_with(listener, folder.clone(), true, false);
    *lan = Some((folder, port));
    Ok(port)
}

/// This computer's address on its network - the one a phone on the same Wi-Fi reaches. Found by
/// asking which address would be used to reach the internet; nothing is sent.
pub fn lan_address() -> Option<std::net::Ipv4Addr> {
    let s = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect("8.8.8.8:80").ok()?;
    match s.local_addr().ok()?.ip() {
        std::net::IpAddr::V4(a) if !a.is_loopback() && !a.is_unspecified() => Some(a),
        _ => None,
    }
}

/// Where the page asks whether the build changed.
const STAMP_PATH: &str = "/__phai_stamp";

/// What a live page runs: it asks for the build's stamp and reloads when it changes.
const RELOAD_SCRIPT: &str = r#"<script>(function(){var s=null;setInterval(function(){fetch("/__phai_stamp",{cache:"no-store"}).then(function(r){return r.text()}).then(function(t){if(s!==null&&t!==s)location.reload();s=t}).catch(function(){})},1500)})();</script>"#;

/// The build's stamp: when the newest of its files was written.
pub fn stamp_of(root: &Path) -> String {
    fn newest(dir: &Path, depth: u32) -> u128 {
        let Ok(entries) = std::fs::read_dir(dir) else { return 0 };
        entries
            .flatten()
            .map(|e| {
                let p = e.path();
                if p.is_dir() {
                    if depth < 2 {
                        newest(&p, depth + 1)
                    } else {
                        0
                    }
                } else {
                    e.metadata().ok().and_then(|m| m.modified().ok()).and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_millis()).unwrap_or(0)
                }
            })
            .max()
            .unwrap_or(0)
    }
    newest(root, 0).to_string()
}

/// `html` with the reload script, before `</body>` when it has one.
pub fn with_reload(html: &str) -> String {
    match html.rfind("</body>") {
        Some(i) => format!("{}{RELOAD_SCRIPT}{}", &html[..i], &html[i..]),
        None => format!("{html}{RELOAD_SCRIPT}"),
    }
}

/// The project's data server while its page is being tried: the project, whether it asks for a
/// key, its port, the process, and the job that ends it when Studio ends.
struct DataServer {
    project: PathBuf,
    port: u16,
    child: std::process::Child,
    _job: isize,
}

static DATA: Mutex<Option<DataServer>> = Mutex::new(None);

/// The data server's port the page servers pass `/phai-data/` on to; 0 when none runs.
fn data_port() -> u16 {
    let mut data = DATA.lock().unwrap_or_else(|e| e.into_inner());
    let Some(d) = data.as_mut() else { return 0 };
    if d.child.try_wait().ok().flatten().is_none() { d.port } else { 0 }
}

/// A job that ends its processes when Studio ends (its handle is never closed before that).
fn job_for(child: &std::process::Child) -> isize {
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::System::JobObjects as jo;
    unsafe {
        let Ok(job) = jo::CreateJobObjectW(None, None) else { return 0 };
        let mut info = jo::JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags = jo::JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let _ = jo::SetInformationJobObject(job, jo::JobObjectExtendedLimitInformation, &info as *const _ as *const _, std::mem::size_of_val(&info) as u32);
        let _ = jo::AssignProcessToJobObject(job, windows::Win32::Foundation::HANDLE(child.as_raw_handle()));
        job.0 as isize
    }
}

/// The access key of the data servers Studio starts, one for its whole run: Run in Browser's
/// page is given it by the page server, a phone asks for it once (it is on the status bar).
pub fn session_key() -> &'static str {
    static KEY: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    KEY.get_or_init(|| phai_data::server::random_key(6))
}

/// The project's data server built and started on this computer, with [`session_key`] - kept
/// while it is the same project. What went wrong, if it could not.
pub fn start_data_server(project: &Path) -> Result<(), String> {
    let key = session_key();
    let mut data = DATA.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(d) = data.as_mut() {
        if d.project == project && d.child.try_wait().ok().flatten().is_none() {
            return Ok(());
        }
        let _ = d.child.kill();
        *data = None;
    }
    let dir = project.join("data-server");
    let cargo = crate::cargo::find_cargo().ok_or("cargo is not installed - rustup.rs")?;
    let target = target_dir(project).join("data-server");
    use std::os::windows::process::CommandExt;
    let out = Command::new(&cargo).args(["build", "--release", "--quiet", "--target-dir"]).arg(&target).current_dir(&dir).creation_flags(0x0800_0000).output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).to_string();
        let lines: Vec<&str> = err.lines().filter(|l| l.contains("error")).take(8).collect();
        return Err(format!("data-server: {}", if lines.is_empty() { err } else { lines.join("\n") }));
    }
    let package = crate::cargo::package_name(&dir).unwrap_or_default();
    let exe = target.join("release").join(format!("{package}.exe"));
    let port = std::net::TcpListener::bind("127.0.0.1:0").and_then(|l| l.local_addr()).map(|a| a.port()).map_err(|e| e.to_string())?;
    let child = Command::new(&exe)
        .env("PHAI_DATA_ADDR", format!("127.0.0.1:{port}"))
        .env("PHAI_DATA_TOKEN", key)
        .current_dir(project)
        .creation_flags(0x0800_0000)
        .stdout(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("{}: {e}", exe.display()))?;
    let job = job_for(&child);
    // Ready when it listens.
    for _ in 0..50 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    *data = Some(DataServer { project: project.to_path_buf(), port, child, _job: job });
    Ok(())
}

/// The value of a request's header, if it has it.
fn header<'a>(head: &'a [String], name: &str) -> Option<&'a str> {
    head.iter().find_map(|l| l.split_once(':').filter(|(k, _)| k.trim().eq_ignore_ascii_case(name)).map(|(_, v)| v.trim()))
}

/// Whether a request is this computer's own page asking: addressed to `localhost` or `127.0.0.1`
/// (not a name of another site that leads here) and from that page, not another site's.
fn own_page(head: &[String]) -> bool {
    let Some(host) = header(head, "host") else { return false };
    let name = host.rsplit_once(':').map_or(host, |(n, _)| n);
    let local = matches!(name.to_ascii_lowercase().as_str(), "localhost" | "127.0.0.1");
    local && header(head, "origin").is_none_or(|o| o.eq_ignore_ascii_case(&format!("http://{host}")))
}

/// A request for `/phai-data/...` passed on to the data server as it came, its answer back -
/// given the session's key when `give_key` and it is this computer's own page asking.
fn pass_to_data(first: &str, head: &[String], reader: &mut BufReader<std::net::TcpStream>, give_key: bool) -> Vec<u8> {
    let port = data_port();
    let unavailable = b"HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json\r\nContent-Length: 44\r\nConnection: close\r\n\r\n{\"error\":\"the data server is not running\"}".to_vec();
    if port == 0 {
        return unavailable;
    }
    let length: usize = header(head, "content-length").and_then(|v| v.parse().ok()).unwrap_or(0);
    let mut body = vec![0u8; length.min(64 << 20)];
    if reader.read_exact(&mut body).is_err() {
        return unavailable;
    }
    let Ok(mut up) = std::net::TcpStream::connect(("127.0.0.1", port)) else { return unavailable };
    let mut req = first.as_bytes().to_vec();
    for l in head {
        req.extend_from_slice(l.as_bytes());
    }
    if give_key && header(head, "authorization").is_none() && own_page(head) {
        req.extend_from_slice(format!("Authorization: Bearer {}\r\n", session_key()).as_bytes());
    }
    req.extend_from_slice(b"\r\n");
    req.extend_from_slice(&body);
    if up.write_all(&req).is_err() {
        return unavailable;
    }
    let mut answer = Vec::new();
    let _ = up.read_to_end(&mut answer);
    if answer.is_empty() { unavailable } else { answer }
}

/// Serves `root` on `listener`: `live` pages reload when the build changes; `give_key` gives this
/// computer's own page the data server's key.
fn serve_with(listener: std::net::TcpListener, root: Arc<Mutex<PathBuf>>, live: bool, give_key: bool) {
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let root = root.lock().unwrap_or_else(|e| e.into_inner()).clone();
            std::thread::spawn(move || {
                let mut stream = stream;
                let mut reader = BufReader::new(match stream.try_clone() {
                    Ok(s) => s,
                    Err(_) => return,
                });
                let mut first = String::new();
                if reader.read_line(&mut first).is_err() {
                    return;
                }
                // The rest of the request's head.
                let mut head: Vec<String> = Vec::new();
                let mut line = String::new();
                while reader.read_line(&mut line).is_ok_and(|n| n > 2) {
                    head.push(std::mem::take(&mut line));
                }
                // The data server's requests, passed on: the page and its data at one address.
                if first.split_whitespace().nth(1).is_some_and(|p| p.contains("/phai-data")) {
                    let reply = pass_to_data(&first, &head, &mut reader, give_key);
                    let _ = stream.write_all(&reply);
                    let _ = stream.flush();
                    return;
                }
                let ok = |kind: &str, data: Vec<u8>| {
                    let mut head = format!("HTTP/1.1 200 OK\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n", data.len()).into_bytes();
                    head.extend(data);
                    head
                };
                let asked = first.split_whitespace().nth(1).unwrap_or("");
                let reply = if live && asked.split('?').next() == Some(STAMP_PATH) {
                    ok("text/plain", stamp_of(&root).into_bytes())
                } else {
                    match file_for(&root, &first).and_then(|p| std::fs::read(&p).ok().map(|d| (p, d))) {
                        Some((p, data)) if live && p.extension().is_some_and(|e| e == "html") => ok(mime(&p), with_reload(&String::from_utf8_lossy(&data)).into_bytes()),
                        Some((p, data)) => ok(mime(&p), data),
                        None => b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec(),
                    }
                };
                let _ = stream.write_all(&reply);
                let mut sink = [0u8; 1];
                let _ = stream.flush();
                let _ = stream.shutdown(std::net::Shutdown::Write);
                let _ = stream.read(&mut sink);
            });
        }
    });
}

/// Where Build > Publish Web App puts `web/`: a folder (a web server's, a shared drive), a server
/// reached by SSH (`user@host:/var/www/app`, through Windows' own `scp`, its key - never a
/// password typed here), or the project's GitHub repository's Pages (`github-pages`: its
/// `gh-pages` branch, which GitHub serves over HTTPS).
#[derive(Clone, Debug, PartialEq)]
pub enum Destination {
    Folder(PathBuf),
    Ssh(String),
    GitHubPages,
}

impl Destination {
    pub fn parse(s: &str) -> Option<Destination> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }
        if matches!(s.to_ascii_lowercase().as_str(), "github-pages" | "github" | "gh-pages") {
            return Some(Destination::GitHubPages);
        }
        // `user@host:path` - not `C:\...`, which is a folder.
        if let Some((who, path)) = s.split_once(':') {
            if who.contains('@') && !who.contains(['\\', '/']) && !path.is_empty() {
                return Some(Destination::Ssh(s.to_string()));
            }
        }
        Some(Destination::Folder(PathBuf::from(s)))
    }
}

/// The GitHub Pages address of a repository's `origin`: `https://user.github.io/repo/`.
pub fn pages_url(remote: &str) -> Option<String> {
    let r = remote.trim().trim_end_matches('/').trim_end_matches(".git");
    let path = r.strip_prefix("https://github.com/").or_else(|| r.strip_prefix("git@github.com:")).or_else(|| r.strip_prefix("ssh://git@github.com/"))?;
    let (user, repo) = path.split_once('/')?;
    if repo.eq_ignore_ascii_case(&format!("{user}.github.io")) {
        return Some(format!("https://{}.github.io/", user.to_ascii_lowercase()));
    }
    Some(format!("https://{}.github.io/{repo}/", user.to_ascii_lowercase()))
}

fn copy_tree(from: &Path, to: &Path) -> std::io::Result<usize> {
    std::fs::create_dir_all(to)?;
    let mut n = 0;
    for e in std::fs::read_dir(from)? {
        let e = e?;
        let target = to.join(e.file_name());
        if e.file_type()?.is_dir() {
            n += copy_tree(&e.path(), &target)?;
        } else {
            std::fs::copy(e.path(), &target)?;
            n += 1;
        }
    }
    Ok(n)
}

fn run(dir: &Path, program: &str, args: &[&str]) -> Result<String, String> {
    use std::os::windows::process::CommandExt;
    let out = Command::new(program).args(args).current_dir(dir).creation_flags(0x0800_0000).output().map_err(|e| format!("{program}: {e}"))?;
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    if out.status.success() {
        Ok(text)
    } else {
        Err(format!("{program} {}:\n{}", args.join(" "), text.trim()))
    }
}

/// `web/` (the folder a build for the web made) put at `to`: what it now is at, to tell the person.
pub fn publish(project: &Path, web: &Path, to: &Destination) -> Result<String, String> {
    match to {
        Destination::Folder(dir) => {
            let dir = if dir.is_absolute() { dir.clone() } else { project.join(dir) };
            let n = copy_tree(web, &dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            Ok(format!("{} ({n})", dir.display()))
        }
        Destination::Ssh(target) => {
            // Each thing in web/, so they land in the folder itself; no question ever waits for an
            // answer (BatchMode) - the key does the signing in.
            let mut args: Vec<String> = vec!["-r".into(), "-q".into(), "-o".into(), "BatchMode=yes".into()];
            for e in std::fs::read_dir(web).map_err(|e| e.to_string())?.flatten() {
                args.push(e.path().to_string_lossy().to_string());
            }
            args.push(target.clone());
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            run(project, "scp", &refs)?;
            Ok(target.clone())
        }
        Destination::GitHubPages => {
            let remote = run(project, "git", &["remote", "get-url", "origin"]).map_err(|_| "The project has no GitHub repository (git remote origin) to publish to.".to_string())?;
            let remote = remote.trim().to_string();
            let url = pages_url(&remote).ok_or_else(|| format!("{remote}: not a GitHub repository"))?;
            // The gh-pages branch in a folder of its own: as it is on GitHub when it is there, then
            // web/ in place of what it held, one commit on top - nothing else of the project moves.
            let tmp = std::env::temp_dir().join(format!("phai-pages-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&tmp);
            std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
            let tmp_s = tmp.to_string_lossy().to_string();
            let fresh = run(project, "git", &["clone", "--quiet", "--depth", "1", "--branch", "gh-pages", &remote, &tmp_s]).is_err();
            if fresh {
                let _ = std::fs::remove_dir_all(&tmp);
                std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
                run(&tmp, "git", &["init", "--quiet"])?;
                run(&tmp, "git", &["checkout", "--quiet", "--orphan", "gh-pages"])?;
                run(&tmp, "git", &["remote", "add", "origin", &remote])?;
            }
            for e in std::fs::read_dir(&tmp).map_err(|e| e.to_string())?.flatten() {
                if e.file_name() != ".git" {
                    let p = e.path();
                    let _ = if p.is_dir() { std::fs::remove_dir_all(&p) } else { std::fs::remove_file(&p) };
                }
            }
            copy_tree(web, &tmp).map_err(|e| e.to_string())?;
            // GitHub's Jekyll left out: the files served as they are.
            std::fs::write(tmp.join(".nojekyll"), "").map_err(|e| e.to_string())?;
            run(&tmp, "git", &["add", "-A"])?;
            let changed = run(&tmp, "git", &["status", "--porcelain"])?;
            if !changed.trim().is_empty() {
                run(&tmp, "git", &["-c", "user.name=Phai Studio", "-c", "user.email=phai-studio@users.noreply.github.com", "commit", "--quiet", "-m", "Publish the web app"])?;
                run(&tmp, "git", &["push", "--quiet", "origin", "gh-pages"])?;
            }
            let _ = std::fs::remove_dir_all(&tmp);
            Ok(url)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(port: u16, path: &str) -> String {
        let mut c = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        write!(c, "GET {path} HTTP/1.1\r\nHost: x\r\n\r\n").unwrap();
        let mut out = String::new();
        c.read_to_string(&mut out).unwrap();
        out
    }

    #[test]
    fn the_web_app_takes_what_the_project_set_else_the_form_and_theme() {
        let dir = std::env::temp_dir().join(format!("phai-webapp-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"shop\"\n").unwrap();
        let theme = "@import \"phai:light\";";
        let a = WebApp::of(&dir, "My Shop", theme);
        assert_eq!((a.name.as_str(), a.short.as_str(), a.orientation.as_str()), ("My Shop", "", "any"));
        assert!(is_colour(&a.color) && is_colour(&a.background));
        let mut t = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        for (k, v) in [("name", "ร้านค้า"), ("theme-color", "#ff0000"), ("orientation", "portrait"), ("icon", "art/logo.png")] {
            t = crate::cargo::with_table_field(&t, crate::cargo::WEB_APP_TABLE, k, v);
        }
        std::fs::write(dir.join("Cargo.toml"), t).unwrap();
        let a = WebApp::of(&dir, "My Shop", theme);
        assert_eq!((a.name.as_str(), a.color.as_str(), a.orientation.as_str()), ("ร้านค้า", "#ff0000", "portrait"));
        assert_eq!(a.icon_file(&dir), None, "not there yet");
        std::fs::create_dir_all(dir.join("art")).unwrap();
        // A wide picture: fitted in the middle of a square on the colour.
        let mut wide = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut wide, 40, 20);
            enc.set_color(png::ColorType::Rgb);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header().unwrap().write_image_data(&[0u8; 40 * 20 * 3]).unwrap();
        }
        std::fs::write(dir.join("art/logo.png"), wide).unwrap();
        let file = a.icon_file(&dir).unwrap();
        let icon = scaled_icon(&file, 192, (255, 0, 0)).unwrap();
        let mut r = png::Decoder::new(std::io::Cursor::new(icon)).read_info().unwrap();
        let mut buf = vec![0u8; r.output_buffer_size()];
        let info = r.next_frame(&mut buf).unwrap();
        assert_eq!((info.width, info.height), (192, 192));
        let at = |x: usize, y: usize| &buf[(y * 192 + x) * 4..(y * 192 + x) * 4 + 3];
        assert_eq!(at(96, 5), &[255, 0, 0], "above the picture: the colour");
        assert_eq!(at(96, 96), &[0, 0, 0], "the picture");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn only_this_computer_s_own_page_is_given_the_data_key() {
        let h = |lines: &[&str]| lines.iter().map(|l| format!("{l}\r\n")).collect::<Vec<_>>();
        assert!(own_page(&h(&["Host: localhost:5000"])));
        assert!(own_page(&h(&["Host: 127.0.0.1:5000", "Origin: http://127.0.0.1:5000"])));
        assert!(!own_page(&h(&["Host: localhost:5000", "Origin: https://evil.example"])), "another site open in the browser");
        assert!(!own_page(&h(&["Host: evil.example:5000", "Origin: http://evil.example:5000"])), "a name of another site leading here");
        assert!(!own_page(&h(&["Host: 192.168.1.5:8790"])), "another computer");
        assert!(!own_page(&h(&["Origin: http://localhost:5000"])), "no address");
    }

    #[test]
    fn the_project_s_web_css_comes_after_the_theme() {
        let dir = std::env::temp_dir().join(format!("phai-webcss-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let theme = "@import \"phai:light\";";
        assert!(!page_css(&dir, theme).contains("web.css"), "none: the theme alone");
        std::fs::write(dir.join("web.css"), "@media (max-width: 600px) { #ok { background: red; } }</style>").unwrap();
        let css = page_css(&dir, theme);
        let at = css.find("#ok").unwrap();
        assert!(at > css.find(".phai-button").unwrap_or(0) && css.ends_with("<\\/style>"), "{css}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_phone_server_reloads_its_pages_when_the_build_changes() {
        let dir = std::env::temp_dir().join(format!("phai-lan-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("index.html"), "<html><body>hi</body></html>").unwrap();
        std::fs::write(dir.join("app.js"), "x()").unwrap();
        // As Run on Phone serves, on this computer only - no firewall question in a test.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        serve_with(listener, Arc::new(Mutex::new(dir.clone())), true, false);
        let page = get(port, "/");
        assert!(page.contains("hi") && page.contains("__phai_stamp") && page.find("__phai_stamp") < page.find("</body>"), "{page}");
        assert!(!get(port, "/app.js").contains("__phai_stamp"), "only pages have it");
        let before = get(port, "/__phai_stamp");
        std::thread::sleep(std::time::Duration::from_millis(30));
        std::fs::write(dir.join("app.js"), "y()").unwrap();
        let body = |r: String| r.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
        assert_ne!(body(get(port, "/__phai_stamp")), body(before));
        assert!(!get(serve(dir.clone()).unwrap(), "/").contains("__phai_stamp"), "Run in Browser's has no reload");
        let _ = std::fs::remove_dir_all(&dir);
    }

    const TOML: &str = r#"[package]
name = "shop"
version = "0.1.0"
edition = "2021"

[dependencies]
phai-ui = { path = "D:/phai/crates/phai-ui" }
windows = { version = "0.62", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
] }
serde = "1"

[profile.release]
opt-level = "s"
"#;

    #[test]
    fn the_project_takes_phai_web_for_phai_ui_on_the_web() {
        let t = web_cargo_toml(TOML, "{ path = \"D:/phai/crates/phai-web\" }");
        let deps = t.find("[dependencies]").unwrap();
        let native = t.find(NATIVE).unwrap();
        let web = t.find(WEB).unwrap();
        let profile = t.find("[profile.release]").unwrap();
        assert!(deps < native && native < web && web < profile, "{t}");
        assert!(t[deps..native].contains("serde = \"1\"") && !t[deps..native].contains("phai-ui"), "{t}");
        assert!(t[native..web].contains("phai-ui = { path") && t[native..web].contains("\"Win32_UI_WindowsAndMessaging\",\n] }"), "{t}");
        assert!(t[web..profile].contains("phai-web = { path"), "{t}");
        assert_eq!(web_cargo_toml(&t, "x"), t, "a second time changes nothing");
    }

    #[test]
    fn main_takes_each_forms_web_module() {
        let m = web_main_rs("mod main_window;\nmod main_window_ui;\nmod data_sources;\nmod text;\n");
        assert_eq!(m, "#[cfg(target_arch = \"wasm32\")]\nextern crate phai_web as phai_ui;\n\nmod main_window;\n#[cfg_attr(target_arch = \"wasm32\", path = \"web/main_window_ui.rs\")]\nmod main_window_ui;\n#[cfg(not(target_arch = \"wasm32\"))]\nmod data_sources;\nmod text;\n");
        assert_eq!(web_main_rs(&m), m);
    }

    #[test]
    fn requests_stay_in_the_folder() {
        let dir = std::env::temp_dir().join(format!("phai-serve-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("pkg")).unwrap();
        std::fs::write(dir.join("index.html"), "hi").unwrap();
        std::fs::write(dir.join("pkg").join("a.wasm"), "w").unwrap();
        assert_eq!(file_for(&dir, "GET / HTTP/1.1"), Some(dir.join("index.html")));
        assert_eq!(file_for(&dir, "GET /pkg/a.wasm?v=2 HTTP/1.1"), Some(dir.join("pkg").join("a.wasm")));
        assert_eq!(file_for(&dir, "GET /../secret HTTP/1.1"), None);
        assert_eq!(file_for(&dir, "GET /nothing HTTP/1.1"), None);
        assert_eq!(mime(Path::new("x.wasm")), "application/wasm");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_icon_is_a_png_of_its_size() {
        let png = icon_png(48, (0x12, 0x85, 0x5f));
        assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
        assert_eq!(hex("#2563eb"), (0x25, 0x63, 0xeb));
    }

    #[test]
    fn a_publish_destination_is_a_folder_a_server_or_github_pages() {
        assert_eq!(Destination::parse("github-pages"), Some(Destination::GitHubPages));
        assert_eq!(Destination::parse("me@example.com:/var/www/app"), Some(Destination::Ssh("me@example.com:/var/www/app".into())));
        assert_eq!(Destination::parse(r"C:\inetpub\wwwroot\app"), Some(Destination::Folder(PathBuf::from(r"C:\inetpub\wwwroot\app"))));
        assert_eq!(Destination::parse("  "), None);
        assert_eq!(pages_url("https://github.com/kirati-ai/phai-studio.git").as_deref(), Some("https://kirati-ai.github.io/phai-studio/"));
        assert_eq!(pages_url("git@github.com:Someone/someone.github.io.git").as_deref(), Some("https://someone.github.io/"));
        assert_eq!(pages_url("https://gitlab.com/a/b"), None);
        let web = std::env::temp_dir().join(format!("phai-pub-web-{}", std::process::id()));
        let to = std::env::temp_dir().join(format!("phai-pub-to-{}", std::process::id()));
        std::fs::create_dir_all(web.join("pkg")).unwrap();
        std::fs::write(web.join("index.html"), "x").unwrap();
        std::fs::write(web.join("pkg").join("a.wasm"), "y").unwrap();
        let said = publish(&web, &web, &Destination::Folder(to.clone())).unwrap();
        assert!(said.ends_with("(2)") && to.join("pkg").join("a.wasm").is_file(), "{said}");
        let _ = std::fs::remove_dir_all(&web);
        let _ = std::fs::remove_dir_all(&to);
    }
}
