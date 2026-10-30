# nix/hm-module.nix
self: { config, lib, pkgs, ... }:

let
  cfg = config.services.succinct;
  tomlFormat = pkgs.formats.toml { };

  appName = cfg.package.meta.mainProgram or cfg.package.pname;
  configFile = tomlFormat.generate "${appName}.toml" cfg.settings;
in
{
  options.services.succinct = {
    enable = lib.mkEnableOption "${appName} background service";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      description = "The package to install.";
    };

    settings = lib.mkOption {
      type = lib.types.submodule (import ./config-schema.nix);
      description = "Configuration for succinct.";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];


    xdg.configFile."${appName}/config.toml".source = configFile;

    systemd.user.services.${appName} = {
      Unit = {
        Description = "${appName} background service";
        After = [ "network.target" ];
      };
      Service = {
        ExecStart = "${lib.getExe cfg.package} --config ${configFile}";
        Restart = "on-failure";
      };
      Install.WantedBy = [ "default.target" ];
    };
  };
}