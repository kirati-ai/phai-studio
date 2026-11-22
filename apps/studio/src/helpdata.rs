//! `--help-data <folder>`: what the Thai guide's control pages are made from, written by Phai
//! Studio itself so they never drift from it - every Toolbox icon as a PNG (`icons/<kind>.png`,
//! the glyph the Toolbox draws) and `controls.json`: each tool's name as the Toolbox shows it, and
//! the form's properties and events as the Properties pane lists them. `docs/help-src/help_gen.py`
//! reads them.

use std::path::Path;

use phai_form::Kind;
use phai_ui::theme::Color;
use serde_json::{json, Value};

use crate::{glyph, kind_label, props, toolbox_items};

/// The PNG size of an icon: twice what the page shows, sharp on a high-DPI screen.
const ICON_PX: i32 = 64;

pub fn export(dir: &Path) -> Result<(), String> {
    phai_ui::text::set_thai(true);
    let icons = dir.join("icons");
    std::fs::create_dir_all(&icons).map_err(|e| format!("{}: {e}", icons.display()))?;

    // The Toolbox's tools in its order, and the form itself (Solution Explorer's form picture).
    let mut tools = vec![("form", glyph::FORM, phai_ui::text::t("Form"))];
    tools.push(("menubar", glyph::MENU_BAR, phai_ui::text::t("Menu bar")));
    tools.push(("toolbar", glyph::TOOLBAR, phai_ui::text::t("Toolbar")));
    let kinds = Kind::ALL;
    for (i, (_, _, g)) in toolbox_items().into_iter().skip(3).enumerate() {
        let k = kinds[i];
        tools.push((k.name(), g, kind_label(k)));
    }

    // The Toolbox's groups, in English (the guide's menu) and Thai (its pages).
    phai_ui::text::set_thai(false);
    let groups_en = crate::toolbox::groups();
    phai_ui::text::set_thai(true);
    let groups_th = crate::toolbox::groups();
    let groups: Vec<Value> = (1..groups_en.len()).map(|i| json!({ "index": i, "en": groups_en[i], "th": groups_th[i] })).collect();

    let ink = Color::rgb(0x1f, 0x29, 0x37);
    let mut list = Vec::new();
    for (tag, g, label) in &tools {
        let px = phai_ui::icon::glyph_pixels(*g, ICON_PX, ink).ok_or("no icon font on this computer")?;
        write_png(&icons.join(format!("{tag}.png")), &px, ICON_PX)?;
        let mut tool = json!({ "tag": tag, "label": label });
        if let Some(k) = kinds.iter().find(|k| k.name() == *tag) {
            let (rows, events) = props::control_for_help(*k);
            tool["component"] = json!(k.is_component());
            tool["group"] = json!(crate::toolbox::group_of(*k));
            tool["type"] = json!(phai_form::codegen::ctl_type(*k));
            tool["event"] = json!(k.event());
            tool["properties"] = rows.into_iter().map(|(category, name, value, help, part)| json!({ "category": category, "name": name, "default": value, "help": help, "part": part })).collect();
            tool["events"] = events.into_iter().map(|(category, name, key, help, arg)| json!({ "category": category, "name": name, "key": key, "help": help, "arg": arg })).collect();
        } else if *tag != "form" {
            // The menu bar and the toolbar are in Menus and toolbars.
            tool["group"] = json!(3);
        }
        list.push(tool);
    }

    let form_props: Vec<Value> = props::form_rows_for_help()
        .into_iter()
        .map(|(category, name, value, help, part)| json!({ "category": category, "name": name, "default": value, "help": help, "part": part }))
        .collect();
    let form_events: Vec<Value> = props::form_events_for_help()
        .into_iter()
        .map(|(category, name, method, help, arg)| json!({ "category": category, "name": name, "method": method, "help": help, "arg": arg }))
        .collect();

    let data = json!({ "groups": groups, "tools": list, "form": { "properties": form_props, "events": form_events } });
    let text = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    let file = dir.join("controls.json");
    std::fs::write(&file, text).map_err(|e| format!("{}: {e}", file.display()))
}

/// BGRA pixels with straight alpha, as `glyph_pixels` gives them, as a PNG.
fn write_png(path: &Path, bgra: &[u8], size: i32) -> Result<(), String> {
    let rgba: Vec<u8> = bgra.as_chunks::<4>().0.iter().flat_map(|p| [p[2], p[1], p[0], p[3]]).collect();
    let file = std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), size as u32, size as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut w = enc.write_header().map_err(|e| e.to_string())?;
    w.write_image_data(&rgba).map_err(|e| e.to_string())
}
