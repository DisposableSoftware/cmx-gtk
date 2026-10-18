# cmx: specification

This folder is the source of truth for what cmx does. The code implements the spec. If the code and the spec disagree, the code is wrong, or the spec must be changed first.

## Files

| File | Prefix | Covers |
|------|--------|--------|
| [product.md](product.md) | `PROD` | Goals, non-goals, platform and stack constraints |
| [reader.md](reader.md) | `READ` | Archives, pages, metadata, navigation, dual-page layout, progress |
| [window.md](window.md) | `WIN` | Windows and what they show |
| [keybindings.md](keybindings.md) | `KEY` | Default bindings, user overrides |
| [appearance.md](appearance.md) | `LOOK` | Styling and display scale |
| [configuration.md](configuration.md) | `CONF` | `config.ini` and `keybindings.ini` |
| [integration.md](integration.md) | `INT` | XDG/freedesktop, desktop entry, Nix, command line, man page |
| [roadmap.md](roadmap.md) | — | The order of planned work, which `/implement` follows |

Add a file with a new prefix when a feature area doesn't fit these, and list it here.

## Requirement format

Each requirement is one line in a table:

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `WIN-1` | One sentence that can be checked. | done | `module::tests::name` |

- **IDs are permanent.** Do not renumber them or reuse one. When a requirement is dropped, keep its row, set the status to `dropped`, and give the reason.
- **Status** is one of:
  - `planned`: agreed but not built.
  - `partial`: built, but with a known gap. The gap is written in the row.
  - `done`: built and verified.
  - `dropped`: no longer wanted.
- Write behaviour, not implementation. Keep the "how" short, and put it under **Implementation notes** when it is not obvious or a past attempt failed.
- **Verified by** names the test, or says `manual` with what to check.

`scripts/release-check.sh` reads the status column: a requirement that became `done` or `partial` since the last release means a minor bump. `/release` pushes `main` and acts on that recommendation.

## Machine-checked sections

Tables between `<!-- spec:NAME:begin -->` and `<!-- spec:NAME:end -->` markers are parsed by `src/spec_tests.rs`. `cargo test` (and so `nix build`) fails when they disagree with the code, `data/config.example.ini`, `data/keybindings.example.ini` or the man page. Keep their column layout unchanged.
