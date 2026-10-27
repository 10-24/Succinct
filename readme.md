# Installation with Nix Home Manager
Add the module as an input in `flake.nix`
```nix
{
  inputs = {
    # ...
    succinct.url = "github:1024/succinct";
  };
}
```

Import and Configure the Module in `home.nix`
```nix
{ inputs, ... }: {
  imports = [
    inputs.succinct.homeManagerModules.default
  ];
  
  programs.succinct = {
    enable = true;

    settings = {
      sync_dir = /home/your-username/sync; 
      
      remote = {
        endpoint = ""; # Eg. https://<ACCOUNT_ID>.r2.cloudflarestorage.com
        access_key_id = "";
        secret_access_key = "";
      };
      
      exclude = [
        "target/"
        "*.lock"
        ".git/"
        "*~"
        "*.bak"
        ".env"
        ".env.*"
        
        ".venv/",
        ".uv/",
        "__pycache__/",
        "*.pyc",
        ".mypy_cache/",
        ".ruff_cache/",
      ];
    };
  };
}
```