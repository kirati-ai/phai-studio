//! Panel: a frame for other controls - or a place the program draws on itself (Paint).

use crate::panel_demo_ui::{PanelDemoEvents, PanelDemoUi};
use phai_ui::paint::PaintEventArgs;

pub struct PanelDemo {
    size: i32,
}

impl Default for PanelDemo {
    fn default() -> Self {
        PanelDemo { size: 40 }
    }
}

impl PanelDemoEvents for PanelDemo {
    // help: paint
    fn board_paint(&mut self, _ui: &PanelDemoUi, e: &PaintEventArgs) {
        // Called whenever the panel is drawn: the program draws what it shows.
        e.clear("#f6f8fa");
        let (x, y) = ((e.width - self.size) / 2, (e.height - self.size) / 2);
        e.fill_rect(x, y, self.size, self.size, "#12855f");
        e.draw_text(&format!("{} px", self.size), 8, 8, "#5f6368");
    }
    // help: end

    // help: redraw
    fn bigger_clicked(&mut self, ui: &PanelDemoUi) {
        self.size = if self.size >= 110 { 40 } else { self.size + 20 };
        ui.board.invalidate(); // draw it again - board_paint runs
    }
    // help: end
}
