{
  lib,
  rustPlatform,
  pkg-config,
  wrapGAppsHook4,
  installShellFiles,
  gtk4,
  gnome,
  webp-pixbuf-loader,
  libavif,
}:
let
  manifest = (lib.importTOML ../Cargo.toml).package;
in
rustPlatform.buildRustPackage {
  pname = manifest.name;
  inherit (manifest) version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../src
      ../data
      ../spec
      ../tests
    ];
  };
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
    wrapGAppsHook4
    installShellFiles
  ];
  buildInputs = [ gtk4 ];

  postInstall = ''
    install -Dm644 data/io.github.DisposableSoftware.cmx.desktop -t $out/share/applications
    for size in 16 32 64 128 256 512; do
      install -Dm644 data/icons/$size.png \
        $out/share/icons/hicolor/''${size}x''${size}/apps/io.github.DisposableSoftware.cmx.png
    done
    install -Dm644 data/io.github.DisposableSoftware.cmx.metainfo.xml -t $out/share/metainfo
    install -Dm644 data/cmx.thumbnailer -t $out/share/thumbnailers
    install -Dm644 data/config.example.ini data/keybindings.example.ini -t $out/share/doc/cmx
    installManPage data/cmx.1

    # INT-21: WebP and AVIF pages decode through gdk-pixbuf loaders. Set here so
    # gappsWrapperArgsHook puts it in the wrapper.
    export GDK_PIXBUF_MODULE_FILE="${
      gnome._gdkPixbufCacheBuilder_DO_NOT_USE {
        extraLoaders = [
          webp-pixbuf-loader
          libavif
        ];
      }
    }"
  '';

  meta = {
    description = "A minimal comic reader for .cbz files";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
    mainProgram = "cmx";
  };
}
