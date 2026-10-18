mod archive;
mod cli;
mod config;
mod decode;
mod fit;
mod info;
mod keys;
mod layout;
mod metadata;
mod page_view;
mod pages;
mod progress;
mod reader;
mod session;
#[cfg(test)]
mod spec_tests;
mod thumbnail;
mod window;

use gtk::{gdk, gio, glib, prelude::*};
use std::cell::Cell;

pub const APP_ID: &str = "io.github.DisposableSoftware.cmx";

const CSS: &str = "
window.cmx .page-area { background-color: black; }
window.cmx .placeholder-title { font-size: 1.5em; color: #c0c0c0; }
window.cmx .placeholder-hint { color: #808080; }
window.cmx .page-area spinner { color: #c0c0c0; }
";

fn main() -> glib::ExitCode {
    match cli::early(std::env::args_os()) {
        Some(cli::Early::Print(text)) => {
            print!("{text}");
            return glib::ExitCode::SUCCESS;
        }
        Some(cli::Early::Fail(message)) => {
            eprintln!("cmx: {message}");
            return glib::ExitCode::FAILURE;
        }
        Some(cli::Early::Thumbnail {
            size,
            input,
            output,
        }) => {
            return match thumbnail::run(size, &input, &output) {
                Ok(()) => glib::ExitCode::SUCCESS,
                Err(message) => {
                    eprintln!("cmx: {message}");
                    glib::ExitCode::FAILURE
                }
            };
        }
        None => {}
    }
    // Without this, windows other than ApplicationWindows get the binary name as their
    // Wayland app_id, so compositor window rules miss dialogs.
    glib::set_prgname(Some(APP_ID));
    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_startup(|app| {
        let quit = gio::ActionEntry::builder("quit")
            .activate(|app: &gtk::Application, _, _| app.quit())
            .build();
        let about = gio::ActionEntry::builder("about")
            .activate(|app: &gtk::Application, _, _| about(app))
            .build();
        app.add_action_entries([quit, about]);
        keys::apply(app);
        let provider = gtk::CssProvider::new();
        provider.load_from_string(CSS);
        if let Some(display) = gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    });

    // Only the first window of the process reopens the last comic (`READ-41`).
    let restored = Cell::new(false);
    app.connect_activate(move |app| {
        let win = window::Win::new(app);
        if !restored.replace(true)
            && let Some(last) = progress::get().last()
        {
            win.load(last, true);
        }
    });
    app.connect_open(|app, files, _| {
        // Held from here: the app would quit before the future first runs.
        let hold = app.hold();
        let open = window::open_files(app.clone(), None, files.to_vec());
        glib::spawn_future_local(async move {
            open.await;
            drop(hold);
        });
    });

    app.run()
}

/// `WIN-15`.
fn about(app: &gtk::Application) {
    let dialog = gtk::AboutDialog::builder()
        .program_name("cmx")
        .version(env!("CARGO_PKG_VERSION"))
        .comments("cmx is pronounced “see-mix”.")
        .logo_icon_name(APP_ID)
        .license_type(gtk::License::MitX11)
        .website("https://github.com/DisposableSoftware/cmx-gtk")
        .modal(true)
        .build();
    dialog.add_credit_section(
        "Third-party crates",
        &[
            "zip (MIT) https://github.com/zip-rs/zip2",
            "quick-xml (MIT) https://github.com/tafia/quick-xml",
        ],
    );
    dialog.set_transient_for(app.active_window().as_ref());
    dialog.present();
}
