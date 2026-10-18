self:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.cmx;
  ini = pkgs.formats.ini { listToValue = lib.concatMapStringsSep ";" toString; };
in
{
  options.programs.cmx = {
    enable = lib.mkEnableOption "cmx";
    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      defaultText = lib.literalExpression "cmx.packages.\${system}.default";
    };
    settings = lib.mkOption {
      inherit (ini) type;
      default = { };
      example = {
        window.width = 1000;
      };
      description = "Written to $XDG_CONFIG_HOME/cmx/config.ini.";
    };
    keybindings = lib.mkOption {
      type = lib.types.attrsOf (lib.types.listOf lib.types.str);
      default = { };
      example = {
        quit = [
          "<Ctrl>q"
          "<Alt>F4"
        ];
        toggle-dual-page = [ ];
      };
      description = "Binding name to keys, written to $XDG_CONFIG_HOME/cmx/keybindings.ini. An empty list unbinds it; see cmx(1) for the names.";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];
    xdg.configFile."cmx/config.ini" = lib.mkIf (cfg.settings != { }) {
      source = ini.generate "cmx-config.ini" cfg.settings;
    };
    xdg.configFile."cmx/keybindings.ini" = lib.mkIf (cfg.keybindings != { }) {
      source = ini.generate "cmx-keybindings.ini" { inherit (cfg) keybindings; };
    };
  };
}
