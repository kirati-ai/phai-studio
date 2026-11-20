//! Window layouts, as Visual Studio's Window > Save Window Layout, Apply Window Layout
//! (Ctrl+Alt+1 to 0) and Reset Window Layout: which panes show, where they are docked, how big,
//! and which hide themselves - kept as a line of text in the settings, ten of them by name.

use phai_ui::layout::Dock;

/// How many layouts can be kept: Ctrl+Alt+1 to Ctrl+Alt+0.
pub const SLOTS: usize = 10;

#[derive(Clone, Debug, PartialEq)]
pub struct Arrangement {
    pub toolbox: bool,
    pub explorer: bool,
    pub properties: bool,
    pub bottom: bool,
    pub toolbox_dock: Dock,
    pub side_dock: Dock,
    /// Design pixels.
    pub toolbox_w: i32,
    pub toolbox_h: i32,
    pub side_w: i32,
    pub side_h: i32,
    pub bottom_h: i32,
    /// Auto Hide: the toolbox, Solution Explorer, Properties.
    pub toolbox_auto: bool,
    pub explorer_auto: bool,
    pub properties_auto: bool,
}

impl Arrangement {
    /// Studio as it first opens: the toolbox at the left, Solution Explorer and Properties at the
    /// right, the panel below closed, nothing hiding itself.
    pub fn standard(toolbox_w: i32, bottom_h: i32) -> Arrangement {
        Arrangement {
            toolbox: true,
            explorer: true,
            properties: true,
            bottom: false,
            toolbox_dock: Dock::Left,
            side_dock: Dock::Right,
            toolbox_w,
            toolbox_h: 200,
            side_w: 280,
            side_h: 260,
            bottom_h,
            toolbox_auto: false,
            explorer_auto: false,
            properties_auto: false,
        }
    }

    pub fn to_text(&self) -> String {
        let b = |v: bool| if v { "1" } else { "0" };
        format!(
            "toolbox={};explorer={};properties={};bottom={};toolbox_dock={};side_dock={};toolbox_w={};toolbox_h={};side_w={};side_h={};bottom_h={};toolbox_auto={};explorer_auto={};properties_auto={}",
            b(self.toolbox),
            b(self.explorer),
            b(self.properties),
            b(self.bottom),
            dock_name(self.toolbox_dock),
            dock_name(self.side_dock),
            self.toolbox_w,
            self.toolbox_h,
            self.side_w,
            self.side_h,
            self.bottom_h,
            b(self.toolbox_auto),
            b(self.explorer_auto),
            b(self.properties_auto)
        )
    }

    /// Read back from [`to_text`]; what is missing or wrong is the standard's.
    pub fn from_text(text: &str, standard: &Arrangement) -> Arrangement {
        let mut a = standard.clone();
        for pair in text.split(';') {
            let Some((k, v)) = pair.split_once('=') else { continue };
            let (k, v) = (k.trim(), v.trim());
            let on = v == "1";
            let n = v.parse::<i32>().ok();
            match k {
                "toolbox" => a.toolbox = on,
                "explorer" => a.explorer = on,
                "properties" => a.properties = on,
                "bottom" => a.bottom = on,
                "toolbox_dock" => a.toolbox_dock = dock_of(v).unwrap_or(a.toolbox_dock),
                "side_dock" => a.side_dock = dock_of(v).unwrap_or(a.side_dock),
                "toolbox_w" => a.toolbox_w = n.unwrap_or(a.toolbox_w).clamp(140, 600),
                "toolbox_h" => a.toolbox_h = n.unwrap_or(a.toolbox_h).clamp(80, 600),
                "side_w" => a.side_w = n.unwrap_or(a.side_w).clamp(180, 900),
                "side_h" => a.side_h = n.unwrap_or(a.side_h).clamp(120, 900),
                "bottom_h" => a.bottom_h = n.unwrap_or(a.bottom_h).clamp(60, 900),
                "toolbox_auto" => a.toolbox_auto = on,
                "explorer_auto" => a.explorer_auto = on,
                "properties_auto" => a.properties_auto = on,
                _ => {}
            }
        }
        a
    }
}

fn dock_name(d: Dock) -> &'static str {
    match d {
        Dock::Left => "left",
        Dock::Right => "right",
        Dock::Top => "top",
        Dock::Bottom => "bottom",
        _ => "left",
    }
}

fn dock_of(s: &str) -> Option<Dock> {
    Some(match s {
        "left" => Dock::Left,
        "right" => Dock::Right,
        "top" => Dock::Top,
        "bottom" => Dock::Bottom,
        _ => return None,
    })
}

/// The settings' names for slot `i` (0 is Ctrl+Alt+1): its name and its arrangement.
pub fn keys(i: usize) -> (String, String) {
    (format!("window_layout_{}_name", i + 1), format!("window_layout_{}", i + 1))
}

/// The shortcut of slot `i`: Ctrl+Alt+1 ... Ctrl+Alt+9, Ctrl+Alt+0.
pub fn shortcut(i: usize) -> String {
    format!("Ctrl+Alt+{}", (i + 1) % 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_layout_comes_back_as_it_was_kept() {
        let std = Arrangement::standard(220, 160);
        let mut a = std.clone();
        a.toolbox = false;
        a.side_dock = Dock::Left;
        a.side_w = 400;
        a.explorer_auto = true;
        a.bottom = true;
        assert_eq!(Arrangement::from_text(&a.to_text(), &std), a);
        assert_eq!(Arrangement::from_text("side_w=5;nonsense;toolbox_dock=up", &std).side_w, 180, "kept in bounds");
        assert_eq!(Arrangement::from_text("", &std), std);
        assert_eq!(shortcut(0), "Ctrl+Alt+1");
        assert_eq!(shortcut(9), "Ctrl+Alt+0");
    }
}
