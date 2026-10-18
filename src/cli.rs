use gtk::glib;
use std::ffi::OsString;

/// What `--help` or `--version` prints (`INT-30`, `INT-31`). GLib's own help would
/// say "Usage: io.github.DisposableSoftware.cmx", because the prgname must stay the app ID.
pub fn early(args: impl IntoIterator<Item = OsString>) -> Option<String> {
    for arg in args.into_iter().skip(1) {
        match arg.to_str() {
            Some("--") => return None,
            Some("-h" | "--help") => return Some(help()),
            Some("-V" | "--version") => return Some(version()),
            _ => {}
        }
    }
    None
}

fn version() -> String {
    format!("cmx {}\n", env!("CARGO_PKG_VERSION"))
}

fn help() -> String {
    let dir = glib::user_config_dir().join("cmx");
    let config = dir.join("config.ini").display().to_string();
    let keys = dir.join("keybindings.ini").display().to_string();
    let w = config.chars().count().max(keys.chars().count());
    format!(
        "\
Usage: cmx [OPTION…]

Opens a cmx window. If cmx is already running, the window opens in it.

Options:
  -h, --help     Show this help and exit
  -V, --version  Show the version and exit

Files:
  {config:<w$}  settings
  {keys:<w$}  keybindings

See cmx(1) for the keybindings and every setting.
",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(args: &[&str]) -> Option<String> {
        early(
            std::iter::once("cmx")
                .chain(args.iter().copied())
                .map(OsString::from),
        )
    }

    #[test]
    fn help_and_version() {
        let help = run(&["--help"]).unwrap();
        assert!(help.starts_with("Usage: cmx [OPTION…]\n"));
        for part in [
            "--help",
            "--version",
            "config.ini",
            "keybindings.ini",
            "cmx(1)",
        ] {
            assert!(help.contains(part), "help lacks {part}");
        }
        assert_eq!(run(&["x", "-h"]), Some(help));
        assert_eq!(
            run(&["-V"]).unwrap(),
            format!("cmx {}\n", env!("CARGO_PKG_VERSION"))
        );
        assert_eq!(run(&["--version"]), run(&["-V"]));
        assert_eq!(run(&[]), None);
        assert_eq!(run(&["--", "--help"]), None);
    }
}
