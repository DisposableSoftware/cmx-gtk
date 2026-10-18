# Integration

## XDG / freedesktop

| ID | Spec | Requirement | Status |
|----|------|-------------|--------|
| `INT-1` | Base Directory | Config, cache and state paths come from `glib::user_config_dir()` / `user_cache_dir()` / `user_state_dir()`. They are never hard-coded as `~/.config`. | done |
| `INT-2` | Desktop Entry | `io.github.DisposableSoftware.cmx.desktop`: `Name=cmx`, `Exec=cmx`, `Icon=io.github.DisposableSoftware.cmx`, `StartupWMClass=io.github.DisposableSoftware.cmx`, and a New Window action. It passes `desktop-file-validate`. | done |

## Nix

| ID | Requirement | Status |
|----|-------------|--------|
| `INT-20` | The flake exports `packages.<system>.default`, `devShells.<system>.default`, `overlays.default` (attr `cmx`), `nixosModules.default` and `homeManagerModules.default`, for x86_64-linux and aarch64-linux. | done |
| `INT-21` | The package installs `bin/cmx` (wrapped by `wrapGAppsHook4`), the desktop entry, the app icon, the AppStream metainfo (`INT-41`), both example files (`CONF-2`) and the man page (`INT-32`). `nix build` runs the unit tests and the spec tests. | done |
| `INT-22` | NixOS module `programs.cmx`: `enable`, `package`. | done |
| `INT-23` | Home-manager module `programs.cmx`: `enable`, `package`, `settings` (generates `config.ini`), `keybindings` (binding name to a list of keys; generates `keybindings.ini`, lists joined with `;`). | done |
| `INT-24` | `nix flake check` passes, and it runs `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` as well as the tests. | done |

## Command line and manual

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `INT-30` | `cmx --help` (or `-h`) prints a short help to stdout and exits with status 0, without opening a window or reaching a running instance: the usage line, the options, the paths of `config.ini` and `keybindings.ini`, and a pointer to `cmx(1)`. | done | `cli::tests::help_and_version` |
| `INT-31` | `cmx --version` (or `-V`) prints `cmx <version>`, the version in `Cargo.toml`, and exits with status 0 in the same way. | done | `cli::tests::help_and_version` |
| `INT-32` | The man page `cmx(1)` documents the synopsis and options, every binding in the default bindings table with its description and default keys, how to override one (`KEY-2`), where the config files live and how they are created (`CONF-3`), every key in the config defaults table with its allowed values and default, and files and environment variables. It renders with `man` without warnings. | done | `spec_tests::man_page_matches_spec`, manual (`man --warnings=all -l data/cmx.1`) |
| `INT-41` | AppStream metainfo, `share/metainfo/io.github.DisposableSoftware.cmx.metainfo.xml`: ID, name, summary, a description that keeps the README's warning, the launchable desktop entry, licences, developer, homepage and the current release. It passes `appstreamcli validate --no-net`. | done | manual (`appstreamcli validate --no-net`) |
