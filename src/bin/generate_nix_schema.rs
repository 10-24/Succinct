


fn main() {
    // Call the macro-generated function from your library
    let nix_schema = succinct::Config::nixos_type_definition();
    
    // Write it to your nix folder
    std::fs::write(env!("ENV_NIX_SCHEMA_PATH"),nix_schema)
        .expect("Failed to write Nix schema");
        
    println!("Successfully generated nix/config-schema.nix!");
}