{ config, lib, pkgs, ... }:

let
  inherit (lib) mkEnableOption mkOption mkIf types getExe;
  cfg = config.services.my-rust-app;
in {
  options.services.succinct = {
    enable = mkEnableOption "Succinct background service";

    package = mkOption {
      type = types.package;
      description = "The succinct package to use.";
    };

    verbosity = mkOption {
      type = types.enum [ "error" "warn" "info" "debug" ];
      default = "info";
      description = "Logging verbosity.";
    };

    extraArgs = mkOption {
      type = types.listOf types.str;
      default = [ ];
      description = "Extra CLI arguments to pass to the binary.";
    };
  };

  config = mkIf cfg.enable {
    # Install package to user environment
    home.packages = [ cfg.package ];

    # Background service (systemd user daemon)
    systemd.user.services.my-rust-app = {
      Unit = {
        Description = "My Rust App Background Daemon";
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