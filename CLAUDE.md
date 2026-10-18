# cmx

A minimal comic reader for `.cbz` files, a GTK 4 port of the macOS app in `../cmx`. Display name **cmx**, binary `cmx`, app ID `io.github.DisposableSoftware.cmx`.

**What the app does is defined in [`spec/`](spec/README.md).** This file covers how to work on it.

## Spec-first workflow (mandatory)

Every change follows these steps, in order. For feature requests, use the `/feature` skill (`.claude/skills/feature/`), which adds a user-approval step between the spec and the code. For bugs, use `/bug` (`.claude/skills/bug/`): it reproduces the bug, finds the spec gap that let it happen, then commits the spec delta and the fix without an approval step, unless the fix changes intended behaviour. To build what is already agreed, use `/implement` (`.claude/skills/implement/`): it works through `spec/roadmap.md`, the verification backlog first and then the `planned` rows in order, with one commit per requirement.

1. **Spec.** Before touching code, update the relevant `spec/*.md`.
   - **New behaviour or a changed requirement:** add or edit a requirement row. A new row gets the next free ID in its prefix, with status `planned`. If the request is unclear, settle the wording with the user before coding.
   - **Bug fix:** find the requirement the bug breaks. If no requirement covers the behaviour, the bug is a gap in the spec, so add one.
   - **Refactor with no behaviour change:** no spec edit, but it must still pass step 3.
   - Commit the spec change on its own, before the code: `spec: <summary>`.
2. **Code.** Implement only what the spec says. If the code needs behaviour the spec doesn't describe, go back to step 1.
3. **Confirm against the spec.** Before the code commit:
   - Run `cargo test`. It includes `src/spec_tests.rs`, which checks the keybinding and config tables in the spec against the code and `data/config.example.ini`.
   - For every requirement ID touched, verify it as its **Verified by** column says (unit test, or a manual or screenshot check, headless by default). Prefer adding a unit test where one is possible.
   - Re-read the changed code against the requirement text and look for behaviour the spec doesn't mention.
   - Update the row: set the status to `done` (or `partial` with the gap written in), and fill in **Verified by**. Update `roadmap.md` if a planned item landed.
4. **Commit** the code with a `Spec: ID, ID` trailer listing the requirements it implements or affects, and tell the user which IDs were verified and how.

When reporting to the user, say plainly if something could only be partly verified (e.g. the check needs Hyprland and the user had the app open, so the manual test was skipped).

## Commands

```bash
nix develop                    # devshell: cargo, clippy, rustfmt, rust-analyzer, gtk4
cargo run                      # run (inside devshell)
cargo clippy --all-targets -- -D warnings    # must pass
cargo fmt
cargo test                     # unit tests + spec conformance tests
nix build                      # package -> ./result/bin/cmx (also runs the tests)
nix flake check
git config core.hooksPath .githooks   # once per clone: enables the pre-push release check
```

The flake only sees git-tracked files. Run `git add` on new files before `nix build` or `nix develop`. After adding crates, commit `Cargo.lock`, which `nix/package.nix` reads through `cargoLock.lockFile`. `spec/` is part of the package source, because the spec tests `include_str!` it.

## Code layout

Keep this table current: one line per module, with the spec IDs it implements.

```
src/main.rs        GtkApplication, open/activate, app actions, About, CSS     WIN-1, WIN-2, WIN-3, WIN-15
src/cli.rs         --help, --version, --thumbnail args, before GTK starts    INT-30, INT-31, INT-51
src/config.rs      config.ini + keybindings.ini, first-run files, OnceLock   CONF KEY-2
src/keys.rs        keys::BINDINGS, overrides, accelerators                   KEY
src/archive.rs     magic-byte sniffing, the ZIP reader, user-facing errors   READ-1, READ-3..5, PROD-7
src/pages.rs       page list: junk filter, natural sort                      READ-2
src/metadata.rs    ComicInfo.xml: locate, lenient parse, display title      READ-10..14
src/decode.rs      bytes -> texture/pixbuf, loader picked by extension      READ-5, INT-21
src/session.rs     off-thread decode, radius-2 cache, prefetch, cancel      READ-21, READ-32
src/reader.rs      open a book, navigation rules, spreads, titles            READ-20, READ-30, WIN-7
src/progress.rs    state.ini: page per comic, last comic, atomic writes      READ-40, READ-41
src/window.rs      reader window, win.* actions, opening files, fit, errors  WIN, READ-42
src/page_view.rs   widget that draws one or two pages                        WIN-6
src/layout.rs      where pages go in the view (pure)                         WIN-6
src/fit.rs         window-fit geometry (pure)                                WIN-11
src/info.rs        the metadata panel                                         WIN-13
src/thumbnail.rs   cmx --thumbnail                                           INT-51
src/spec_tests.rs  spec <-> code conformance tests
tests/fixtures/    comics for unit and manual tests (six.cbz: page 3 wide;
                   demo.cbz: drawn by scripts/make-demo.sh for screenshots)
scripts/manual/    UI checks: smoke, open, paging, info, scales, screenshot (lib.sh helpers;
                   headless.sh runs one in a headless sway)
scripts/release-check.sh  recommended version bump since the last v* tag (spec delta)
scripts/release.sh  bump, commit "cmx X.Y.Z", tag, draft notes (no push)
.githooks/pre-push  on a push of main, offers to release
data/              desktop entry, metainfo, cmx.thumbnailer,
                   icons/ (the macOS icons per size: regenerate them in ../cmx
                   with scripts/generate-icons.sh, never edit the PNGs)       INT-4
                   screenshots/reader.png (scripts/manual/screenshot.sh)    INT-41
                   config.example.ini + keybindings.example.ini (first-run)  CONF-2, INT-50
                   cmx.1 (man page, checked by spec_tests)                  INT-32
nix/               package.nix (+ WebP/AVIF pixbuf loaders), modules        INT-2x
spec/              the specification
```

## Conventions

- Keep the dependency list short. Ask before adding a crate, and prefer GIO/GLib equivalents (`PROD-1`).
- Use async GIO APIs for file I/O. Never block the main loop.
- Avoid strong reference cycles in signal closures: use `glib::clone!` with `#[weak]` for widgets and models.
- Look up gtk4-rs and GTK docs before guessing at APIs, because the bindings change between minor versions. Read the vendored source in `~/.cargo/registry/src/*/gtk4-0.11.*` for exact signatures.
  - https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/
  - https://docs.gtk.org/gtk4/
- To match another app's behaviour, read its source (`nix build nixpkgs#<app>.src`) rather than guessing.
- Comments only when the reason is not obvious.
- Unit tests that use GTK must call `gtk::init()` and skip if it fails, because the `nix build` sandbox has no display. GTK may only be initialised on one thread, so keep GTK checks in one test.
- Commits: `spec: <summary>` for spec changes, a `Spec: ID, ID` trailer on code commits, `BREAKING` in the body for incompatible changes (`scripts/release-check.sh` reads all three). No Claude-Session links in commit messages.

## Releasing

`scripts/release-check.sh` recommends a bump from the commits since the last `v*` tag: major for `BREAKING` or a removed config key or binding, minor when a requirement became `done` or `partial`, patch for other package changes. `scripts/release.sh LEVEL NOTES` bumps `Cargo.toml`, `Cargo.lock`, the metainfo and the man page, runs the tests, commits and tags, and drafts notes; then `git push origin main vX.Y.Z` and `gh release create vX.Y.Z -F NOTES`. With the hook enabled, pushing main offers all of this (`NO_RELEASE_CHECK=1` skips it). Ask before pushing.

## Manual testing

Prefer adding a script to `scripts/manual/` (source `lib.sh`, which has the guards below built in) over ad-hoc commands. Each script names the spec IDs it checks. Run it after `nix build`.

### Headless (default)

Run checks with `./scripts/manual/headless.sh ./scripts/manual/<name>.sh`. It starts a headless sway with its own D-Bus session and runtime dir, so it works while the screen is locked or the user has the app open, and never touches their windows. `lib.sh` switches to `swaymsg` when `HEADLESS` is set, so scripts don't change.

- The output defaults to 3840×2160 at scale 2, a common 4K HiDPI setup. Set `HEADLESS_RES` and `HEADLESS_SCALE` to test other scales. Windows float, so they open at their default size.
- Send screenshots (`shot`, into `$OUT`) to the user when a change is visual.
- Headless sway drops the first key after `wtype` loads its keymap; `keys` sends a Shift tap first to absorb it.
- The runner keeps tests out of the user's session. It uses a private bus without portals (a portal started there attaches to the user's compositor and mounts a document portal), temp `XDG_STATE_HOME`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME` and runtime dir, no `$DISPLAY` (a GTK app that misses the headless display can open on the user's desktop through Xwayland), and `GDK_BACKEND=wayland`. The user's `gtk-4.0/settings.ini` comes in through `XDG_CONFIG_DIRS`, so fonts and icons match. Scripts still set their own config and state dirs: never rely on the runner to protect the user's files.
- In headless mode `hyprctl` only answers `cursorpos` (tracked from `wlrctl` moves) and `monitors`; anything else, and `ydotool` (kernel-level input), fails instead of reaching the real session. Pointer clicks at fixed positions don't always register; run those on Hyprland.
- Headless sway is not Hyprland. Use a real Hyprland session (below) for compositor behaviour: window states, focus rules, drag icons.
- `scripts/manual/scales.sh` runs `scale-check.sh` at scales 1, 1.25, 1.5 and 2 by itself (`LOOK-1`).

### On Hyprland

Run `./scripts/manual/<name>.sh` directly.

- Screenshots: `hyprctl clients -j` gives the window geometry, and `grim -g` captures it. Use an `XDG_CONFIG_HOME` in a temp dir to test config variants.
- Check `pidof hyprlock` first: if the screen is locked, grim captures the lock screen. `systemd-inhibit --what=idle` works, because hypridle respects inhibitors.
- New windows don't take focus. Run `hyprctl dispatch 'hl.dsp.focus({ window = "class:io.github.DisposableSoftware.cmx" })'` (Hyprland 0.56 uses a Lua config), then send keys with `wtype`. Always check `hyprctl activewindow` first, so keys never land in the user's other apps.
- The Nix wrapper renames the process to `.cmx-wrapped`, so `pgrep -x cmx` never matches, and `pgrep -f` matches the test's own shell. Find and kill test windows through `hyprctl clients -j` (`.pid`).
- The app is single-instance. If the user already has it open, a test launch only opens a window in *their* process, and killing that pid closes their windows. Check there are no `io.github.DisposableSoftware.cmx` clients before testing, and don't go ahead if there are.

### Test comics

Use `tests/fixtures/` first. For a real-world check, copy one comic from your own library into a temp dir and open the copy. Never open library files directly: cmx writes reading progress and recent-file entries for whatever it opens.
