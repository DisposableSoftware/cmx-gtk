self:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.template-app;
in
{
  options.programs.template-app = {
    enable = lib.mkEnableOption "Template App";
    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      defaultText = lib.literalExpression "template-app.packages.\${system}.default";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
  };
}
