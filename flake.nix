# Flake.nix
{ config, lib, pkgs, ... }:

let
  inherit (lib) mkEnableOption mkOption mkIf types getExe;
  cfg = config.services.succinct;
in {
  options.services.succinct = {
    enable = mkEnableOption "Succinct background service";

    package = mkOption {
      type = types.package;
      description = "Automatically synchronizes a folder to the cloud.";
    };

  };

  config = mkIf cfg.enable {
    # Install package to user environment
    home.packages = [ cfg.package ];

    # Background service (systemd user daemon)
    systemd.user.services.my-rust-app = {
      Unit = {
        Description = cfg.package.description;
        After = [ "network.target" ];
      };

      Service = {
        ExecStart = ''
          ${getExe cfg.package} \
            --port ${toString cfg.port} \
            --log-level ${cfg.verbosity} \
            ${lib.escapeShellArgs cfg.extraArgs}
        '';
        Restart = "on-failure";
        RestartSec = "5s";
      };

      Install = {
        WantedBy = [ "default.target" ];
      };
    };
  };
}