use gtk::glib;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const CONFIG_EXAMPLE: &str = include_str!("../data/config.example.ini");
const KEYBINDINGS_EXAMPLE: &str = include_str!("../data/keybindings.example.ini");

/// `$XDG_CONFIG_HOME/cmx`.
pub fn dir() -> PathBuf {
    glib::user_config_dir().join("cmx")
}

const FLAVORS: &[&str] = &["latte", "frappe", "macchiato", "mocha", "gtk"];

pub struct Config {
    /// `latte`, `frappe`, `macchiato`, `mocha` or `gtk` (`CONF-6`).
    pub flavor: String,
    /// One of `theme::ACCENTS` (`CONF-7`).
    pub accent: String,
    /// Window background alpha (`CONF-8`).
    pub opacity: f64,
    /// Default window size in logical px.
    pub width: i32,
    pub height: i32,
    /// Overrides from `keybindings.ini`; see `keys::BINDINGS` for names and defaults.
    pub keybindings: HashMap<String, Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            flavor: "mocha".into(),
            accent: "mauve".into(),
            opacity: 0.85,
            width: 800,
            height: 1000,
            keybindings: HashMap::new(),
        }
    }
}

pub fn get() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| Config::load_from(&dir()))
}

impl Config {
    /// Reads `config.ini` and `keybindings.ini`, first creating whichever is missing
    /// (`CONF-3`); missing keys fall back to defaults.
    fn load_from(dir: &Path) -> Self {
        create_missing(dir);
        let mut cfg = read(&dir.join("config.ini"))
            .map(|kf| Self::from_keyfile(&kf))
            .unwrap_or_default();
        if let Some(kf) = read(&dir.join("keybindings.ini")) {
            cfg.keybindings = bindings(&kf);
        }
        cfg
    }

    fn from_keyfile(kf: &glib::KeyFile) -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = kf.integer("window", "width") {
            cfg.width = v.clamp(300, 8192);
        }
        if let Ok(v) = kf.integer("window", "height") {
            cfg.height = v.clamp(200, 8192);
        }
        if let Ok(v) = kf.string("appearance", "flavor") {
            if FLAVORS.contains(&v.as_str()) {
                cfg.flavor = v.to_string();
            } else {
                glib::g_warning!("cmx", "config.ini: unknown flavor “{v}”, using mocha");
            }
        }
        if let Ok(v) = kf.string("appearance", "accent") {
            if crate::theme::ACCENTS.contains(&v.as_str()) {
                cfg.accent = v.to_string();
            } else {
                glib::g_warning!("cmx", "config.ini: unknown accent “{v}”, using mauve");
            }
        }
        if let Ok(v) = kf.double("appearance", "opacity") {
            cfg.opacity = v.clamp(0.0, 1.0);
        }
        cfg
    }
}

/// The `[keybindings]` group of `keybindings.ini` (`KEY-2`).
fn bindings(kf: &glib::KeyFile) -> HashMap<String, Vec<String>> {
    let mut out = HashMap::new();
    for key in kf
        .keys("keybindings")
        .map(|k| k.to_vec())
        .unwrap_or_default()
    {
        if let Ok(list) = kf.string_list("keybindings", &key) {
            let accels = list
                .iter()
                .map(|a| a.trim().to_owned())
                .filter(|a| !a.is_empty())
                .collect();
            out.insert(key.to_string(), accels);
        }
    }
    out
}

/// The key file at `path`, or `None` when it is missing or broken; broken is warned about.
fn read(path: &Path) -> Option<glib::KeyFile> {
    let kf = glib::KeyFile::new();
    match kf.load_from_file(path, glib::KeyFileFlags::NONE) {
        Ok(_) => Some(kf),
        Err(e) => {
            if !e.matches(glib::FileError::Noent) {
                glib::g_warning!("cmx", "{}: {}", path.display(), e.message());
            }
            None
        }
    }
}

/// `example` with every setting commented out, so the file documents the defaults
/// without pinning them.
fn commented(example: &str) -> String {
    let mut out = String::from(
        "# Created by cmx with every default commented out. To change one,\n\
         # remove the leading '# ' and edit the value.\n",
    );
    for line in example.lines() {
        if line.starts_with(|c: char| c.is_ascii_lowercase()) && line.contains(" =") {
            out.push_str("# ");
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Writes the commented examples for files missing from `dir` (`CONF-3`). Anything
/// already at the path, a dangling symlink included, is left alone.
fn create_missing(dir: &Path) {
    for (name, example) in [
        ("config.ini", CONFIG_EXAMPLE),
        ("keybindings.ini", KEYBINDINGS_EXAMPLE),
    ] {
        let path = dir.join(name);
        if path.symlink_metadata().is_ok() {
            continue;
        }
        let written = std::fs::create_dir_all(dir).and_then(|()| {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)?
                .write_all(commented(example).as_bytes())
        });
        if let Err(e) = written {
            glib::g_warning!("cmx", "could not create {}: {e}", path.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("cmx-test-config-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn reads_settings_and_keybindings_from_their_files() {
        let dir = temp("read");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("config.ini"), "[window]\nwidth=100\nheight=700\n").unwrap();
        std::fs::write(
            dir.join("keybindings.ini"),
            "[keybindings]\nquit=<Ctrl>q;<Ctrl><Shift>q\nnew-window=\n",
        )
        .unwrap();
        let cfg = Config::load_from(&dir);
        assert_eq!(cfg.width, 300, "clamped");
        assert_eq!(cfg.height, 700);
        assert_eq!(cfg.keybindings["quit"], ["<Ctrl>q", "<Ctrl><Shift>q"]);
        assert!(cfg.keybindings["new-window"].is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn reads_appearance_and_falls_back_on_bad_values() {
        let read = |data: &str| {
            let kf = glib::KeyFile::new();
            kf.load_from_data(data, glib::KeyFileFlags::NONE).unwrap();
            Config::from_keyfile(&kf)
        };
        let cfg = read("[appearance]\nflavor=latte\naccent=teal\nopacity=0.5\n");
        assert_eq!(
            (cfg.flavor.as_str(), cfg.accent.as_str(), cfg.opacity),
            ("latte", "teal", 0.5)
        );
        let cfg = read("[appearance]\nflavor=gtk\nopacity=7\n");
        assert_eq!((cfg.flavor.as_str(), cfg.opacity), ("gtk", 1.0));
        assert_eq!(read("[appearance]\nopacity=-1\n").opacity, 0.0);
        let cfg = read("[appearance]\nflavor=nord\naccent=chartreuse\nopacity=lots\n");
        let default = Config::default();
        assert_eq!(cfg.flavor, default.flavor);
        assert_eq!(cfg.accent, default.accent);
        assert_eq!(cfg.opacity, default.opacity);
    }

    #[test]
    fn creates_missing_files_with_the_defaults_commented_out() {
        let dir = temp("create").join("cmx");
        let cfg = Config::load_from(&dir);
        assert_eq!(cfg.width, Config::default().width);
        assert!(cfg.keybindings.is_empty());

        let config = std::fs::read_to_string(dir.join("config.ini")).unwrap();
        let keys = std::fs::read_to_string(dir.join("keybindings.ini")).unwrap();
        assert!(config.contains("\n# width = 800\n"));
        assert!(keys.contains("\n# quit = q;<Ctrl>q\n"));
        for line in config.lines().chain(keys.lines()) {
            assert!(
                line.is_empty() || line.starts_with('#') || line.starts_with('['),
                "not commented out: {line}"
            );
        }

        // Uncommenting a line takes effect, and a second start leaves the files alone.
        std::fs::write(
            dir.join("config.ini"),
            config.replace("# width = 800", "width = 1000"),
        )
        .unwrap();
        let cfg = Config::load_from(&dir);
        assert_eq!(cfg.width, 1000);
        std::fs::remove_dir_all(dir.parent().unwrap()).unwrap();
    }

    #[test]
    fn never_writes_to_existing_files_or_symlinks() {
        let dir = temp("keep");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("config.ini"), "[window]\nwidth=1200\n").unwrap();
        // Like a dotfiles link whose target isn't there yet.
        std::os::unix::fs::symlink(dir.join("elsewhere.ini"), dir.join("keybindings.ini")).unwrap();
        let cfg = Config::load_from(&dir);
        assert_eq!(cfg.width, 1200);
        assert_eq!(
            std::fs::read_to_string(dir.join("config.ini")).unwrap(),
            "[window]\nwidth=1200\n"
        );
        assert!(!dir.join("elsewhere.ini").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn starts_with_defaults_when_the_files_cant_be_created() {
        let blocker = temp("blocked");
        std::fs::write(&blocker, "a file, not a folder").unwrap();
        let cfg = Config::load_from(&blocker.join("cmx"));
        assert_eq!(cfg.width, Config::default().width);
        std::fs::remove_file(&blocker).unwrap();
    }
}
