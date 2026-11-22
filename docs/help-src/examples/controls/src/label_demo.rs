//! Label: words on the window - changed by code, and clicked.

use crate::label_demo_ui::{LabelDemoEvents, LabelDemoUi};

#[derive(Default)]
pub struct LabelDemo {
    next: usize,
}

const WORDS: [&str; 4] = ["Bamboo", "River", "Mango", "Elephant"];

impl LabelDemoEvents for LabelDemo {
    // help: click
    fn word_click(&mut self, ui: &LabelDemoUi) {
        // A label can hear a click too (its Click event).
        ui.word.set_text(WORDS[self.next]);
        self.next = (self.next + 1) % WORDS.len();
    }
    // help: end
}
