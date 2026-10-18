use crate::config;
use gtk::{glib, prelude::*};
use std::collections::HashMap;

pub type Binding = (
    &'static str,
    &'static str,
    &'static str,
    &'static [&'static str],
);

/// Config name, GAction, description, default accelerators. The config name is the key
/// in the `[keybindings]` group of keybindings.ini.
pub const BINDINGS: &[Binding] = &[
    ("open", "win.open", "Open comics", &["<Ctrl>o"]),
    ("new-window", "win.new-window", "New window", &["<Ctrl>n"]),
    ("close-window", "window.close", "Close window", &["<Ctrl>w"]),
    ("quit", "app.quit", "Quit", &["q", "<Ctrl>q"]),
    (
        "next-page",
        "win.next-page",
        "Next page",
        &["space", "Right", "Down", "Page_Down", "l", "j"],
    ),
    (
        "previous-page",
        "win.previous-page",
        "Previous page",
        &["<Shift>space", "Left", "Up", "Page_Up", "h", "k"],
    ),
    ("first-page", "win.first-page", "First page", &["Home"]),
    ("last-page", "win.last-page", "Last page", &["End"]),
    (
        "toggle-dual-page",
        "win.toggle-dual-page",
        "Toggle dual-page layout",
        &["d"],
    ),
    (
        "toggle-info",
        "win.toggle-info",
        "Toggle the info panel",
        &["<Ctrl>i"],
    ),
    (
        "toggle-fullscreen",
        "win.toggle-fullscreen",
        "Toggle fullscreen",
        &["f", "F11"],
    ),
    (
        "exit-fullscreen",
        "win.exit-fullscreen",
        "Leave fullscreen",
        &["Escape"],
    ),
    ("about", "app.about", "About cmx", &["F1"]),
];

fn resolve(overrides: &HashMap<String, Vec<String>>) -> Vec<(&'static Binding, Vec<String>)> {
    for name in overrides.keys() {
        if !BINDINGS.iter().any(|(n, _, _, _)| n == name) {
            glib::g_warning!("cmx", "keybindings.ini: unknown keybinding '{name}'");
        }
    }
    BINDINGS
        .iter()
        .map(|binding| {
            let (name, _, _, defaults) = binding;
            let accels = match overrides.get(*name) {
                Some(custom) => custom
                    .iter()
                    .filter(|a| {
                        let ok = gtk::accelerator_parse(a.as_str()).is_some();
                        if !ok {
                            glib::g_warning!(
                                "cmx",
                                "keybindings.ini: bad accelerator '{a}' for {name}"
                            );
                        }
                        ok
                    })
                    .cloned()
                    .collect(),
                None => defaults.iter().map(|a| a.to_string()).collect(),
            };
            (binding, accels)
        })
        .collect()
}

pub fn apply(app: &gtk::Application) {
    for ((_, action, _, _), accels) in resolve(&config::get().keybindings) {
        let accels: Vec<&str> = accels.iter().map(String::as_str).collect();
        app.set_accels_for_action(action, &accels);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One test: GTK may only be initialised on one thread, and needs a display.
    #[test]
    fn accelerators() {
        if gtk::init().is_err() {
            eprintln!("skipping: no display");
            return;
        }
        for (name, _, _, accels) in BINDINGS {
            for a in *accels {
                assert!(gtk::accelerator_parse(*a).is_some(), "{name}: {a}");
            }
        }
        let overrides = HashMap::from([
            (
                "quit".to_owned(),
                vec!["<Alt>F4".to_owned(), "nonsense<".to_owned()],
            ),
            ("new-window".to_owned(), vec![]),
        ]);
        let resolved = resolve(&overrides);
        let keys = |n: &str| {
            resolved
                .iter()
                .find(|((name, _, _, _), _)| *name == n)
                .map(|(_, k)| k.clone())
                .unwrap()
        };
        assert_eq!(keys("quit"), ["<Alt>F4"]);
        assert_eq!(keys("next-page")[0], "space");
        assert!(keys("new-window").is_empty());
        assert_eq!(keys("close-window"), ["<Ctrl>w"]);
    }
}
