//! PictureBox: a picture file on the window; a click changes how it fits.

use crate::picture_demo_ui::{PictureDemoEvents, PictureDemoUi};

#[derive(Default)]
pub struct PictureDemo {
    mode: usize,
}

const MODES: [&str; 4] = ["zoom", "stretch", "center", "normal"];

impl PictureDemoEvents for PictureDemo {
    // help: load
    fn loaded(&mut self, ui: &PictureDemoUi) {
        // Another picture file, found from where the program runs.
        ui.photo.load("pictures/phai.png");
    }
    // help: end

    // help: mode
    fn photo_click(&mut self, ui: &PictureDemoUi) {
        self.mode = (self.mode + 1) % MODES.len();
        ui.photo.set_size_mode(MODES[self.mode]);
        ui.mode.set_text(&format!("SizeMode: {}", MODES[self.mode]));
    }
    // help: end
}
