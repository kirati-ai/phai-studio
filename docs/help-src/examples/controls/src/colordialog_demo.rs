//! ColorDialog: Windows' own window for choosing a colour - shown on a panel.

use crate::colordialog_demo_ui::{ColordialogDemoEvents, ColordialogDemoUi};
use phai_ui::paint::PaintEventArgs;

pub struct ColordialogDemo {
    colour: String,
}

impl Default for ColordialogDemo {
    fn default() -> Self {
        ColordialogDemo { colour: "#12855f".into() }
    }
}

impl ColordialogDemoEvents for ColordialogDemo {
    fn loaded(&mut self, ui: &ColordialogDemoUi) {
        ui.code.set_text(&self.colour);
    }

    // help: pick
    fn pick_clicked(&mut self, ui: &ColordialogDemoUi) {
        // Starts on the colour we have; gives "#rrggbb", or None for Cancel.
        if let Some(c) = ui.colours.show_dialog(&self.colour) {
            self.colour = c;
            ui.code.set_text(&self.colour);
            ui.swatch.invalidate();
        }
    }
    // help: end

    fn swatch_paint(&mut self, _ui: &ColordialogDemoUi, e: &PaintEventArgs) {
        e.clear(&self.colour);
    }
}
