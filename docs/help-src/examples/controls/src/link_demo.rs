//! LinkLabel: words with a link in them that opens a web page.

use crate::link_demo_ui::{LinkDemoEvents, LinkDemoUi};

#[derive(Default)]
pub struct LinkDemo;

impl LinkDemoEvents for LinkDemo {
    // help: click
    fn site_clicked(&mut self, ui: &LinkDemoUi) {
        // Windows opens the address in the browser.
        let _ = std::process::Command::new("explorer").arg("https://phai-studio.monrak.net/").spawn();
        ui.note.set_text("Opening the website...");
    }
    // help: end
}
