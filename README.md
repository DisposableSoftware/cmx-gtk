<h1 align="center">Template App</h1>

<p align="center">A starting point for GTK 4 apps on Wayland.</p>

> [!WARNING]
> **Certified, artisanal, vibe-coded slop.**
>
> No human has read this code. An AI wrote all of it while a person said "yeah, looks
> good" and pressed Enter a lot.

<!-- template:begin -->
## Using this template

This is a Rust + GTK 4 + Nix app skeleton for building apps with Claude Code, spec first.
It is the workflow from [Junk Drawer](https://github.com/DisposableSoftware/junk-drawer),
with the file manager taken out: a window that says hello, a config file, keybindings, a
man page, Nix modules and the tooling that keeps them honest.

1. On GitHub, **Use this template**, then clone your new repo.
2. Rename everything:

   ```sh
   scripts/rename.sh "My App" myapp io.github.you.MyApp
   nix develop -c cargo test
   git config core.hooksPath .githooks
   git commit -am "Rename the template to My App"
   ```

   The crate name defaults to the display name in kebab case and the GitHub repo to
   `origin`; pass them as the 4th and 5th arguments to override. The script removes
   itself and this section.
3. Edit `spec/product.md` to say what your app is, then start Claude Code and ask for
   features with `/feature` and fixes with `/bug`.

What you get:

- **`spec/`**: every behaviour as a numbered requirement with a status and how it is
  verified. `CLAUDE.md` makes every change go spec first: a `spec:` commit, then code
  with a `Spec: ID` trailer.
- **`/feature` and `/bug`** (`.claude/skills/`): `/feature` drafts the spec change and
  waits for your approval before writing code; `/bug` reproduces, finds the spec gap,
  fixes and verifies without stopping.
- **Spec conformance tests** (`src/spec_tests.rs`): the keybinding and config tables in
  the spec are checked against the code, the example config files and the man page.
- **First-run config**: `config.ini` and `keybindings.ini` are created with every default
  commented out, and never overwritten.
- **Headless UI testing** (`scripts/manual/`): `headless.sh` runs a check in a private
  headless sway, D-Bus session and XDG dirs, so it works with the screen locked and never
  touches your desktop. `scales.sh` checks the layout at scales 1, 1.25, 1.5 and 2.
- **Releases**: `release-check.sh` recommends a semver bump from the spec's status
  changes, `release.sh` bumps, tags and drafts notes, and the pre-push hook offers to do
  it all when you push main.
- **Nix**: package, devshell, `nix flake check` (tests, clippy, rustfmt), overlay, and
  NixOS and home-manager modules.
<!-- template:end -->

## Install

```sh
nix run github:DisposableSoftware/template-app
```

Or as a flake input:

```nix
{
  inputs.template-app.url = "github:DisposableSoftware/template-app";

  # NixOS
  imports = [ inputs.template-app.nixosModules.default ];
  programs.template-app.enable = true;

  # or home-manager, which can also write the settings
  imports = [ inputs.template-app.homeManagerModules.default ];
  programs.template-app = {
    enable = true;
    settings.window.width = 1000;
    keybindings.quit = [ "<Ctrl>q" "<Alt>F4" ];
  };
}
```

## Usage

```sh
tapp            # open a window
tapp --help
man tapp        # every binding and setting
```

Settings live in `~/.config/template-app/config.ini` and keybindings in
`~/.config/template-app/keybindings.ini`. Both are created on first start with every
default commented out; remove the `# ` from a line to change it.

## Development

What the app does is written down in [`spec/`](spec/README.md); see
[`CLAUDE.md`](CLAUDE.md) for how changes are made.

```sh
nix develop                 # cargo, clippy, rustfmt, rust-analyzer, GTK 4
cargo run
cargo clippy --all-targets -- -D warnings
cargo test
nix build                   # ./result/bin/tapp
scripts/manual/headless.sh scripts/manual/smoke.sh
```

## License

MIT, see [LICENSE](LICENSE).
