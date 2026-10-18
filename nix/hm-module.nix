self:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.template-app;
  ini = pkgs.formats.ini { listToValue = lib.concatMapStringsSep ";" toString; };
in
{
  options.programs.template-app = {
    enable = lib.mkEnableOption "Template App";
    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      defaultText = lib.literalExpression "template-app.packages.\${system}.default";
    };
    settings = lib.mkOption {
      inherit (ini) type;
      default = { };
      example = {
        window.width = 1000;
        view.greeting = "Hi";
      };
      description = "Written to $XDG_CONFIG_HOME/template-app/config.ini.";
    };
    keybindings = lib.mkOption {
      type = lib.types.attrsOf (lib.types.listOf lib.types.str);
      default = { };
      example = {
        quit = [
          "<Ctrl>q"
          "<Alt>F4"
        ];
        new-window = [ ];
      };
      description = "Binding name to keys, written to $XDG_CONFIG_HOME/template-app/keybindings.ini. An empty list unbinds it; see tapp(1) for the names.";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];
    xdg.configFile."template-app/config.ini" = lib.mkIf (cfg.settings != { }) {
      source = ini.generate "template-app-config.ini" cfg.settings;
    };
    xdg.configFile."template-app/keybindings.ini" = lib.mkIf (cfg.keybindings != { }) {
      source = ini.generate "template-app-keybindings.ini" { inherit (cfg) keybindings; };
    };
  };
}
