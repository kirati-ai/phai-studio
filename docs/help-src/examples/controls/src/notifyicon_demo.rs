//! NotifyIcon: an icon by the clock - a notice from it, and a click on it.

use crate::notifyicon_demo_ui::{NotifyiconDemoEvents, NotifyiconDemoUi};

#[derive(Default)]
pub struct NotifyiconDemo;

impl NotifyiconDemoEvents for NotifyiconDemo {
    // help: balloon
    fn tell_clicked(&mut self, ui: &NotifyiconDemoUi) {
        ui.tray.show_balloon_tip("Phai demo", "Your work is saved.");
    }
    // help: end

    // help: click
    fn tray_clicked(&mut self, ui: &NotifyiconDemoUi) {
        // A click on the icon: the window comes to the front again.
        ui.show_window();
        ui.activate();
    }
    // help: end
}
