self:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.cmx;
in
{
  options.programs.cmx = {
    enable = lib.mkEnableOption "cmx";
    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      defaultText = lib.literalExpression "cmx.packages.\${system}.default";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
  };
}
