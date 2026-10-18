use crate::config;
use gtk::{gio, glib, prelude::*};

/// Opens a main window (`WIN-1`).
pub fn open(app: &gtk::Application) {
    let cfg = config::get();
    let label = gtk::Label::builder()
        .label(&cfg.greeting)
        .css_classes(["title-1"])
        .build();
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Template App")
        .default_width(cfg.width)
        .default_height(cfg.height)
        .child(&label)
        .build();

    let new_window = gio::ActionEntry::builder("new-window")
        .activate(glib::clone!(
            #[weak]
            app,
            move |_: &gtk::ApplicationWindow, _, _| open(&app)
        ))
        .build();
    window.add_action_entries([new_window]);
    window.present();
}
