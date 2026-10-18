use gtk::glib;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const CONFIG_EXAMPLE: &str = include_str!("../data/config.example.ini");
const KEYBINDINGS_EXAMPLE: &str = include_str!("../data/keybindings.example.ini");

/// `$XDG_CONFIG_HOME/template-app`.
pub fn dir() -> PathBuf {
    glib::user_config_dir().join("template-app")
}

pub struct Config {
    /// Default window size in logical px.
    pub width: i32,
    pub height: i32,
    pub greeting: String,
    /// Overrides from `keybindings.ini`; see `keys::BINDINGS` for names and defaults.
    pub keybindings: HashMap<String, Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            greeting: "Hello".into(),
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
        if let Ok(v) = kf.string("view", "greeting") {
            cfg.greeting = v.to_string();
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
                glib::g_warning!("tapp", "{}: {}", path.display(), e.message());
            }
            None
        }
    }
}

/// `example` with every setting commented out, so the file documents the defaults
/// without pinning them.
fn commented(example: &str) -> String {
    let mut out = String::from(
        "# Created by Template App with every default commented out. To change one,\n\
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
            glib::g_warning!("tapp", "could not create {}: {e}", path.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("tapp-test-config-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn reads_settings_and_keybindings_from_their_files() {
        let dir = temp("read");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("config.ini"),
            "[window]\nwidth=100\nheight=700\n[view]\ngreeting=Hi there\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("keybindings.ini"),
            "[keybindings]\nquit=<Ctrl>q;<Ctrl><Shift>q\nnew-window=\n",
        )
        .unwrap();
        let cfg = Config::load_from(&dir);
        assert_eq!(cfg.width, 300, "clamped");
        assert_eq!(cfg.height, 700);
        assert_eq!(cfg.greeting, "Hi there");
        assert_eq!(cfg.keybindings["quit"], ["<Ctrl>q", "<Ctrl><Shift>q"]);
        assert!(cfg.keybindings["new-window"].is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn creates_missing_files_with_the_defaults_commented_out() {
        let dir = temp("create").join("template-app");
        let cfg = Config::load_from(&dir);
        assert_eq!(cfg.width, Config::default().width);
        assert!(cfg.keybindings.is_empty());

        let config = std::fs::read_to_string(dir.join("config.ini")).unwrap();
        let keys = std::fs::read_to_string(dir.join("keybindings.ini")).unwrap();
        assert!(config.contains("\n# width = 800\n"));
        assert!(keys.contains("\n# quit = <Ctrl>q\n"));
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
        let cfg = Config::load_from(&blocker.join("template-app"));
        assert_eq!(cfg.width, Config::default().width);
        std::fs::remove_file(&blocker).unwrap();
    }
}
