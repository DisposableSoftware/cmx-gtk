<h1 align="center">cmx</h1>

<p align="center">A starting point for GTK 4 apps on Wayland.</p>

> [!WARNING]
> **Certified, artisanal, vibe-coded slop.**
>
> No human has read this code. An AI wrote all of it while a person said "yeah, looks
> good" and pressed Enter a lot.


## Install

```sh
nix run github:DisposableSoftware/cmx-gtk
```

Or as a flake input:

```nix
{
  inputs.cmx.url = "github:DisposableSoftware/cmx-gtk";

  # NixOS
  imports = [ inputs.cmx.nixosModules.default ];
  programs.cmx.enable = true;

  # or home-manager, which can also write the settings
  imports = [ inputs.cmx.homeManagerModules.default ];
  programs.cmx = {
    enable = true;
    settings.window.width = 1000;
    keybindings.quit = [ "<Ctrl>q" "<Alt>F4" ];
  };
}
```

## Usage

```sh
cmx            # open a window
cmx --help
man cmx        # every binding and setting
```

Settings live in `~/.config/cmx/config.ini` and keybindings in
`~/.config/cmx/keybindings.ini`. Both are created on first start with every
default commented out; remove the `# ` from a line to change it.

## Development

What the app does is written down in [`spec/`](spec/README.md); see
[`CLAUDE.md`](CLAUDE.md) for how changes are made.

```sh
nix develop                 # cargo, clippy, rustfmt, rust-analyzer, GTK 4
cargo run
cargo clippy --all-targets -- -D warnings
cargo test
nix build                   # ./result/bin/cmx
scripts/manual/headless.sh scripts/manual/smoke.sh
```

## License

MIT, see [LICENSE](LICENSE).
