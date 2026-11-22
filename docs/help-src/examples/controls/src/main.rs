// Written by docs/help-src/build_examples.py from the *_demo.phf files: `cargo run -- <kind>`
// opens that Toolbox page's demo window.
#![windows_subsystem = "windows"]

mod bindingsource_demo;
mod bindingsource_demo_ui;
mod button_demo;
mod button_demo_ui;
mod calendar_demo;
mod calendar_demo_ui;
mod checkbox_demo;
mod checkbox_demo_ui;
mod checkedlist_demo;
mod checkedlist_demo_ui;
mod colordialog_demo;
mod colordialog_demo_ui;
mod combo_demo;
mod combo_demo_ui;
mod contextmenu_demo;
mod contextmenu_demo_ui;
mod date_demo;
mod date_demo_ui;
mod edit_demo;
mod edit_demo_ui;
mod errorprovider_demo;
mod errorprovider_demo_ui;
mod flow_demo;
mod flow_demo_ui;
mod folderbrowserdialog_demo;
mod folderbrowserdialog_demo_ui;
mod fontdialog_demo;
mod fontdialog_demo_ui;
mod grid_demo;
mod grid_demo_ui;
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
mod openfiledialog_demo;
mod openfiledialog_demo_ui;
mod panel_demo;
mod panel_demo_ui;
mod picture_demo;
mod picture_demo_ui;
mod printdocument_demo;
mod printdocument_demo_ui;
mod progress_demo;
mod progress_demo_ui;
mod radio_demo;
mod radio_demo_ui;
mod richtext_demo;
mod richtext_demo_ui;
mod savefiledialog_demo;
mod savefiledialog_demo_ui;
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
mod timer_demo;
mod timer_demo_ui;
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
        "bindingsource" => bindingsource_demo_ui::BindingsourceDemoUi::run(bindingsource_demo::BindingsourceDemo::default()),
        "button" => button_demo_ui::ButtonDemoUi::run(button_demo::ButtonDemo::default()),
        "calendar" => calendar_demo_ui::CalendarDemoUi::run(calendar_demo::CalendarDemo::default()),
        "checkbox" => checkbox_demo_ui::CheckboxDemoUi::run(checkbox_demo::CheckboxDemo::default()),
        "checkedlist" => checkedlist_demo_ui::CheckedlistDemoUi::run(checkedlist_demo::CheckedlistDemo::default()),
        "colordialog" => colordialog_demo_ui::ColordialogDemoUi::run(colordialog_demo::ColordialogDemo::default()),
        "combo" => combo_demo_ui::ComboDemoUi::run(combo_demo::ComboDemo::default()),
        "contextmenu" => contextmenu_demo_ui::ContextmenuDemoUi::run(contextmenu_demo::ContextmenuDemo::default()),
        "date" => date_demo_ui::DateDemoUi::run(date_demo::DateDemo::default()),
        "edit" => edit_demo_ui::EditDemoUi::run(edit_demo::EditDemo::default()),
        "errorprovider" => errorprovider_demo_ui::ErrorproviderDemoUi::run(errorprovider_demo::ErrorproviderDemo::default()),
        "flow" => flow_demo_ui::FlowDemoUi::run(flow_demo::FlowDemo::default()),
        "folderbrowserdialog" => folderbrowserdialog_demo_ui::FolderbrowserdialogDemoUi::run(folderbrowserdialog_demo::FolderbrowserdialogDemo::default()),
        "fontdialog" => fontdialog_demo_ui::FontdialogDemoUi::run(fontdialog_demo::FontdialogDemo::default()),
        "grid" => grid_demo_ui::GridDemoUi::run(grid_demo::GridDemo::default()),
        "group" => group_demo_ui::GroupDemoUi::run(group_demo::GroupDemo::default()),
        "label" => label_demo_ui::LabelDemoUi::run(label_demo::LabelDemo::default()),
        "link" => link_demo_ui::LinkDemoUi::run(link_demo::LinkDemo::default()),
        "list" => list_demo_ui::ListDemoUi::run(list_demo::ListDemo::default()),
        "listbox" => listbox_demo_ui::ListboxDemoUi::run(listbox_demo::ListboxDemo::default()),
        "masked" => masked_demo_ui::MaskedDemoUi::run(masked_demo::MaskedDemo::default()),
        "menubar" => menubar_demo_ui::MenubarDemoUi::run(menubar_demo::MenubarDemo::default()),
        "notifyicon" => notifyicon_demo_ui::NotifyiconDemoUi::run(notifyicon_demo::NotifyiconDemo::default()),
        "number" => number_demo_ui::NumberDemoUi::run(number_demo::NumberDemo::default()),
        "openfiledialog" => openfiledialog_demo_ui::OpenfiledialogDemoUi::run(openfiledialog_demo::OpenfiledialogDemo::default()),
        "panel" => panel_demo_ui::PanelDemoUi::run(panel_demo::PanelDemo::default()),
        "picture" => picture_demo_ui::PictureDemoUi::run(picture_demo::PictureDemo::default()),
        "printdocument" => printdocument_demo_ui::PrintdocumentDemoUi::run(printdocument_demo::PrintdocumentDemo::default()),
        "progress" => progress_demo_ui::ProgressDemoUi::run(progress_demo::ProgressDemo::default()),
        "radio" => radio_demo_ui::RadioDemoUi::run(radio_demo::RadioDemo::default()),
        "richtext" => richtext_demo_ui::RichtextDemoUi::run(richtext_demo::RichtextDemo::default()),
        "savefiledialog" => savefiledialog_demo_ui::SavefiledialogDemoUi::run(savefiledialog_demo::SavefiledialogDemo::default()),
        "slider" => slider_demo_ui::SliderDemoUi::run(slider_demo::SliderDemo::default()),
        "split" => split_demo_ui::SplitDemoUi::run(split_demo::SplitDemo::default()),
        "statusbar" => statusbar_demo_ui::StatusbarDemoUi::run(statusbar_demo::StatusbarDemo::default()),
        "table" => table_demo_ui::TableDemoUi::run(table_demo::TableDemo::default()),
        "tabs" => tabs_demo_ui::TabsDemoUi::run(tabs_demo::TabsDemo::default()),
        "textarea" => textarea_demo_ui::TextareaDemoUi::run(textarea_demo::TextareaDemo::default()),
        "timer" => timer_demo_ui::TimerDemoUi::run(timer_demo::TimerDemo::default()),
        "toolbar" => toolbar_demo_ui::ToolbarDemoUi::run(toolbar_demo::ToolbarDemo::default()),
        "tree" => tree_demo_ui::TreeDemoUi::run(tree_demo::TreeDemo::default()),
        "usercontrol" => usercontrol_demo_ui::UsercontrolDemoUi::run(usercontrol_demo::UsercontrolDemo::default()),
        other => eprintln!("no demo called {other:?}"),
    }
}
