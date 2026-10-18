use gtk::glib;
use std::ffi::OsString;
use std::path::PathBuf;

/// What to do before GTK starts.
#[derive(Debug, PartialEq)]
pub enum Early {
    /// `--help` or `--version` (`INT-30`, `INT-31`).
    Print(String),
    /// `--thumbnail SIZE INPUT OUTPUT` (`INT-51`).
    Thumbnail {
        size: i32,
        input: PathBuf,
        output: PathBuf,
    },
    Fail(String),
}

/// GLib's own help would say "Usage: io.github.DisposableSoftware.cmx", because the
/// prgname must stay the app ID.
pub fn early(args: impl IntoIterator<Item = OsString>) -> Option<Early> {
    let args: Vec<OsString> = args.into_iter().skip(1).collect();
    if args.first().is_some_and(|a| a == "--thumbnail") {
        return Some(thumbnail(&args[1..]));
    }
    for arg in &args {
        match arg.to_str() {
            Some("--") => return None,
            Some("-h" | "--help") => return Some(Early::Print(help())),
            Some("-V" | "--version") => return Some(Early::Print(version())),
            _ => {}
        }
    }
    None
}

fn thumbnail(args: &[OsString]) -> Early {
    let [size, input, output] = args else {
        return Early::Fail("usage: cmx --thumbnail SIZE INPUT OUTPUT".into());
    };
    match size.to_str().and_then(|s| s.parse::<i32>().ok()) {
        Some(size) if size > 0 => Early::Thumbnail {
            size,
            input: input.into(),
            output: output.into(),
        },
        _ => Early::Fail(format!(
            "--thumbnail: '{}' is not a size in px",
            size.to_string_lossy()
        )),
    }
}

fn version() -> String {
    format!("cmx {}\n", env!("CARGO_PKG_VERSION"))
}

fn help() -> String {
    let dir = glib::user_config_dir().join("cmx");
    let config = dir.join("config.ini").display().to_string();
    let keys = dir.join("keybindings.ini").display().to_string();
    let state = crate::progress::file().display().to_string();
    let w = [&config, &keys, &state]
        .iter()
        .map(|p| p.chars().count())
        .max()
        .unwrap_or(0);
    format!(
        "\
Usage: cmx [OPTION…] [FILE…]

Opens each FILE, a .cbz comic, in its own window. With no FILE, reopens the comic
read last. If cmx is already running, the windows open in it.

Options:
  -h, --help                         Show this help and exit
  -V, --version                      Show the version and exit
      --thumbnail SIZE INPUT OUTPUT  Write INPUT's first page as a PNG of at most
                                     SIZE×SIZE px to OUTPUT, and exit

Files:
  {config:<w$}  settings
  {keys:<w$}  keybindings
  {state:<w$}  reading progress

See cmx(1) for the keybindings and every setting.
",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(args: &[&str]) -> Option<Early> {
        early(
            std::iter::once("cmx")
                .chain(args.iter().copied())
                .map(OsString::from),
        )
    }

    #[test]
    fn help_and_version() {
        let Some(Early::Print(help)) = run(&["--help"]) else {
            panic!("no help")
        };
        assert!(help.starts_with("Usage: cmx [OPTION…] [FILE…]\n"));
        for part in [
            "--help",
            "--version",
            "--thumbnail",
            "config.ini",
            "keybindings.ini",
            "state.ini",
            "cmx(1)",
        ] {
            assert!(help.contains(part), "help lacks {part}");
        }
        assert_eq!(run(&["x.cbz", "-h"]), Some(Early::Print(help)));
        assert_eq!(
            run(&["-V"]),
            Some(Early::Print(format!("cmx {}\n", env!("CARGO_PKG_VERSION"))))
        );
        assert_eq!(run(&["--version"]), run(&["-V"]));
        assert_eq!(run(&[]), None);
        assert_eq!(run(&["a.cbz"]), None);
        assert_eq!(run(&["--", "--help"]), None);
    }

    #[test]
    fn thumbnail_arguments() {
        assert_eq!(
            run(&["--thumbnail", "256", "in.cbz", "out.png"]),
            Some(Early::Thumbnail {
                size: 256,
                input: "in.cbz".into(),
                output: "out.png".into()
            })
        );
        for bad in [
            &["--thumbnail"][..],
            &["--thumbnail", "256", "in.cbz"],
            &["--thumbnail", "big", "in.cbz", "out.png"],
            &["--thumbnail", "0", "in.cbz", "out.png"],
            &["--thumbnail", "1", "a", "b", "c"],
        ] {
            assert!(matches!(run(bad), Some(Early::Fail(_))), "{bad:?}");
        }
    }
}
