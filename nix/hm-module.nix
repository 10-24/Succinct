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
      type = tomlFormat.type;
      default = { };
      description = "Configuration written to the `${appName}.toml`";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];

    # For interactive CLI use (the app finds it via ProjectDirs)
    xdg.configFile."${home}/${appName}.toml".source = configFile;

    systemd.user.services.${appName} = {
      Unit = {
        Description = "${appName} background service";
        After = [ "network.target" ];
      };
      Service = {
        # Explicit path: the service doesn't depend on directory lookup at all
        ExecStart = "${lib.getExe cfg.package} --config ${configFile}";
        Restart = "on-failure";
      };
      Install.WantedBy = [ "default.target" ];
    };
  };
}