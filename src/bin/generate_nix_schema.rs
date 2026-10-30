


fn main() {
    let path = env!("ENV_NIX_SCHEMA_PATH");
    let nix_schema = succinct::Config::nixos_type_definition();
    
    match std::fs::write(path, nix_schema) {
        Ok(_) => println!("Generated `{}`", path),
        Err(e) => panic!("Failed to write Nix schema: {}", e),
    }
}