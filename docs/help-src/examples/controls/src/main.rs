// Written by docs/help-src/build_examples.py from the *_demo.phaiform files: `cargo run -- <kind>`
// opens that Toolbox page's demo window.
#![windows_subsystem = "windows"]

mod button_demo;
mod button_demo_ui;
mod calendar_demo;
mod calendar_demo_ui;
mod checkbox_demo;
mod checkbox_demo_ui;
mod checkedlist_demo;
mod checkedlist_demo_ui;
mod combo_demo;
mod combo_demo_ui;
mod contextmenu_demo;
mod contextmenu_demo_ui;
mod date_demo;
mod date_demo_ui;
mod edit_demo;
mod edit_demo_ui;
mod flow_demo;
mod flow_demo_ui;
mod group_demo;
mod group_demo_ui;
mod label_demo;
mod label_demo_ui;
mod link_demo;
mod link_demo_ui;
mod list_demo;
mod list_demo_ui;
mod listbox_demo;
mod listbox_demo_ui;
mod masked_demo;
mod masked_demo_ui;
mod menubar_demo;
mod menubar_demo_ui;
mod notifyicon_demo;
mod notifyicon_demo_ui;
mod number_demo;
mod number_demo_ui;
mod panel_demo;
mod panel_demo_ui;
mod picture_demo;
mod picture_demo_ui;
mod progress_demo;
mod progress_demo_ui;
mod radio_demo;
mod radio_demo_ui;
mod richtext_demo;
mod richtext_demo_ui;
mod slider_demo;
mod slider_demo_ui;
mod split_demo;
mod split_demo_ui;
mod statusbar_demo;
mod statusbar_demo_ui;
mod table_demo;
mod table_demo_ui;
mod tabs_demo;
mod tabs_demo_ui;
mod textarea_demo;
mod textarea_demo_ui;
mod toolbar_demo;
mod toolbar_demo_ui;
mod tree_demo;
mod tree_demo_ui;
mod usercontrol_demo;
mod usercontrol_demo_ui;
mod counter_box;
mod counter_box_ui;

fn main() {
    match std::env::args().nth(1).as_deref().unwrap_or("") {
        "button" => button_demo_ui::ButtonDemoUi::run(button_demo::ButtonDemo::default()),
        "calendar" => calendar_demo_ui::CalendarDemoUi::run(calendar_demo::CalendarDemo::default()),
        "checkbox" => checkbox_demo_ui::CheckboxDemoUi::run(checkbox_demo::CheckboxDemo::default()),
        "checkedlist" => checkedlist_demo_ui::CheckedlistDemoUi::run(checkedlist_demo::CheckedlistDemo::default()),
        "combo" => combo_demo_ui::ComboDemoUi::run(combo_demo::ComboDemo::default()),
        "contextmenu" => contextmenu_demo_ui::ContextmenuDemoUi::run(contextmenu_demo::ContextmenuDemo::default()),
        "date" => date_demo_ui::DateDemoUi::run(date_demo::DateDemo::default()),
        "edit" => edit_demo_ui::EditDemoUi::run(edit_demo::EditDemo::default()),
        "flow" => flow_demo_ui::FlowDemoUi::run(flow_demo::FlowDemo::default()),
        "group" => group_demo_ui::GroupDemoUi::run(group_demo::GroupDemo::default()),
        "label" => label_demo_ui::LabelDemoUi::run(label_demo::LabelDemo::default()),
        "link" => link_demo_ui::LinkDemoUi::run(link_demo::LinkDemo::default()),
        "list" => list_demo_ui::ListDemoUi::run(list_demo::ListDemo::default()),
        "listbox" => listbox_demo_ui::ListboxDemoUi::run(listbox_demo::ListboxDemo::default()),
        "masked" => masked_demo_ui::MaskedDemoUi::run(masked_demo::MaskedDemo::default()),
        "menubar" => menubar_demo_ui::MenubarDemoUi::run(menubar_demo::MenubarDemo::default()),
        "notifyicon" => notifyicon_demo_ui::NotifyiconDemoUi::run(notifyicon_demo::NotifyiconDemo::default()),
        "number" => number_demo_ui::NumberDemoUi::run(number_demo::NumberDemo::default()),
        "panel" => panel_demo_ui::PanelDemoUi::run(panel_demo::PanelDemo::default()),
        "picture" => picture_demo_ui::PictureDemoUi::run(picture_demo::PictureDemo::default()),
        "progress" => progress_demo_ui::ProgressDemoUi::run(progress_demo::ProgressDemo::default()),
        "radio" => radio_demo_ui::RadioDemoUi::run(radio_demo::RadioDemo::default()),
        "richtext" => richtext_demo_ui::RichtextDemoUi::run(richtext_demo::RichtextDemo::default()),
        "slider" => slider_demo_ui::SliderDemoUi::run(slider_demo::SliderDemo::default()),
        "split" => split_demo_ui::SplitDemoUi::run(split_demo::SplitDemo::default()),
        "statusbar" => statusbar_demo_ui::StatusbarDemoUi::run(statusbar_demo::StatusbarDemo::default()),
        "table" => table_demo_ui::TableDemoUi::run(table_demo::TableDemo::default()),
        "tabs" => tabs_demo_ui::TabsDemoUi::run(tabs_demo::TabsDemo::default()),
        "textarea" => textarea_demo_ui::TextareaDemoUi::run(textarea_demo::TextareaDemo::default()),
        "toolbar" => toolbar_demo_ui::ToolbarDemoUi::run(toolbar_demo::ToolbarDemo::default()),
        "tree" => tree_demo_ui::TreeDemoUi::run(tree_demo::TreeDemo::default()),
        "usercontrol" => usercontrol_demo_ui::UsercontrolDemoUi::run(usercontrol_demo::UsercontrolDemo::default()),
        other => eprintln!("no demo called {other:?}"),
    }
}
