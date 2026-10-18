# Product

cmx (pronounced "see-mix") is a minimal comic reader for `.cbz` files on Wayland. It is a GTK 4 port of the macOS app [cmx](https://github.com/DisposableSoftware/cmx). Binary `cmx`, app ID `io.github.DisposableSoftware.cmx`.

It shows one page at a time, or two side by side, fit to the window, with nothing drawn over the image. It has no library: it opens whatever file it is given and remembers where you stopped reading.

## Requirements

| ID | Requirement | Status |
|----|-------------|--------|
| `PROD-1` | Lightweight and minimal: few dependencies, no libadwaita, fast startup. | done (ongoing constraint) |
| `PROD-2` | Follow a freedesktop/XDG spec wherever one exists for the behaviour (see [integration.md](integration.md)). | done (ongoing constraint) |
| `PROD-3` | Simple toolchain: `cargo` inside a Nix devshell, nothing else. | done |
| `PROD-4` | Easy NixOS integration. | done (see `INT-20`…`INT-24`) |
| `PROD-5` | Every action has a keybinding users can change (see [keybindings.md](keybindings.md)). | done (ongoing constraint) |
| `PROD-6` | Minimal UI is a hard constraint. The page fills the window, and everything else (the info panel, errors) stays hidden until asked for. There is no menu bar, toolbar or on-screen control. | planned (ongoing constraint) |
| `PROD-7` | Archive contents are untrusted input. A broken, hostile or mislabeled file shows an error. It never crashes or hangs the app. | planned (ongoing constraint) |

## Non-goals

- No libadwaita. It fights custom theming and transparency.
- No library, shelf or collection. cmx opens the files it is given.
- No CBR/RAR reading. A RAR is detected and explained (`READ-1`), never read.
- No reading-direction toggle, cloud sync or annotations.
- No menu bar. `--help` and the man page document the keys.
- No preview pane like macOS Quick Look's space-bar preview. File managers get thumbnails only (`INT-50`).

## Stack constraints

- Rust and gtk4-rs (`gtk4` 0.11 with feature `gnome_49`).
- Use GIO/GLib for anything they provide (XDG dirs, MIME, files, D-Bus) rather than reimplementing it or adding a crate.
- Ask before adding a crate. Approved: `zip` (deflate only) for archives, and `quick-xml` for `ComicInfo.xml`. GdkTexture and gdk-pixbuf decode the images.
- File I/O is async through GIO, or runs on a worker thread. The main loop never blocks on I/O or on decoding.
