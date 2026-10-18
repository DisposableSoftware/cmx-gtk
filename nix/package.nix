{
  lib,
  rustPlatform,
  pkg-config,
  wrapGAppsHook4,
  installShellFiles,
  gtk4,
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
    install -Dm644 data/io.github.DisposableSoftware.TemplateApp.desktop -t $out/share/applications
    install -Dm644 data/io.github.DisposableSoftware.TemplateApp.svg -t $out/share/icons/hicolor/scalable/apps
    install -Dm644 data/io.github.DisposableSoftware.TemplateApp.metainfo.xml -t $out/share/metainfo
    install -Dm644 data/config.example.ini data/keybindings.example.ini -t $out/share/doc/template-app
    installManPage data/tapp.1
  '';

  meta = {
    description = "A starting point for GTK 4 apps";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
    mainProgram = "tapp";
  };
}
