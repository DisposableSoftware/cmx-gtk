# Configuration

## Requirements

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `CONF-1` | Settings are read from `$XDG_CONFIG_HOME/template-app/config.ini` (GLib key-file / INI syntax) when the app starts. A missing file is not an error. A file that can't be read or parsed is warned about, and the defaults are used. Every key is optional. Changes apply on restart. | done | `config::tests` |
| `CONF-2` | `data/config.example.ini` lists every setting with its default, and `data/keybindings.example.ini` every binding with its default keys. Both are installed to `share/doc/template-app/`. | done | `spec_tests::example_config_matches_spec` |
| `CONF-3` | When the app starts and `config.ini` or `keybindings.ini` doesn't exist, it creates the missing file (and the folder) from the example (`CONF-2`), with every line commented out, so the file changes nothing until a line is uncommented. A file that exists is never written to, whether it is a regular file, a symlink (as with dotfiles or home-manager) or a dangling symlink. If a file can't be created, that is warned about and the app starts normally. | done | `config::tests` |
| `CONF-4` | `[window] width` and `height`: the size of new windows in logical px, clamped to 300–8192 and 200–8192. | done | `config::tests`, `spec_tests` |
| `CONF-5` | `[view] greeting`: the text the window shows (`WIN-1`). | done | `config::tests`, `spec_tests` |

## Defaults

<!-- spec:config:begin -->
| Section | Key | Default |
|---------|-----|---------|
| `window` | `width` | `800` |
| `window` | `height` | `600` |
| `view` | `greeting` | `Hello` |
<!-- spec:config:end -->

The defaults for `[keybindings]` are in [keybindings.md](keybindings.md).

## Implementation notes

- To add a key, edit the defaults table, `Config`, `spec_tests::default_value` and `CONFIG_KEY_COUNT`, `data/config.example.ini`, the man page `data/tapp.1` (`INT-32`) and, if it needs an option, `nix/hm-module.nix`. The spec tests fail until they agree.
- Runtime state the app remembers between sessions (as opposed to settings) belongs in `$XDG_STATE_HOME/template-app/state.ini`, written atomically, never in `config.ini`.
