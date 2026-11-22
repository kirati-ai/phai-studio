// Written by docs/help-src/build_examples.py from the *_demo.phaiform files: `cargo run -- <kind>`
// opens that Toolbox page's demo window.
#![windows_subsystem = "windows"]

mod button_demo;
mod button_demo_ui;

fn main() {
    match std::env::args().nth(1).as_deref().unwrap_or("") {
        "button" => button_demo_ui::ButtonDemoUi::run(button_demo::ButtonDemo::default()),
        other => eprintln!("no demo called {other:?}"),
    }
}
