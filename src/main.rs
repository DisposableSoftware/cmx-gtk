mod cli;
mod config;
mod keys;
#[cfg(test)]
mod spec_tests;
mod window;

use gtk::{gio, glib, prelude::*};

pub const APP_ID: &str = "io.github.DisposableSoftware.cmx";

fn main() -> glib::ExitCode {
    if let Some(text) = cli::early(std::env::args_os()) {
        print!("{text}");
        return glib::ExitCode::SUCCESS;
    }
    // Without this, windows other than ApplicationWindows get the binary name as their
    // Wayland app_id, so compositor window rules miss dialogs.
    glib::set_prgname(Some(APP_ID));
    let app = gtk::Application::builder().application_id(APP_ID).build();

    app.connect_startup(|app| {
        let quit = gio::ActionEntry::builder("quit")
            .activate(|app: &gtk::Application, _, _| app.quit())
            .build();
        app.add_action_entries([quit]);
        keys::apply(app);
    });
    app.connect_activate(window::open);

    app.run()
}
