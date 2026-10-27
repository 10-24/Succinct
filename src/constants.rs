use std::time::Duration;

use const_format::concatcp;
use object_store::aws::AmazonS3ConfigKey;

use crate::constants::fs::app_data;


pub const APP_NAME:&str = env!("CARGO_PKG_NAME");
pub const ORGANIZATION: &str = "10-24";
pub const NIX_SCHEMA_PATH: &str = "nix/config-schema.nix";


pub mod fs {
    use const_format::concatcp;
    use crate::{constants::APP_NAME, declare_paths};

    pub const CONFIG_NAME: &str = concatcp!(APP_NAME, ".toml");
    
    declare_paths! {
        sync {
            // ... User Files
            sync_data: ".sync-data" {
                // tmp_dir. (Randomly Named)
                state_db: "state.db";
                staged_deltas_db: "staged-deltas.db";
            }
        }
    }
}






pub mod config_default {
    use std::time::Duration;
    use const_format::concatcp;
    use crate::constants::fs;

    pub const DEBOUNCE_DURATION_SECONDS: u16 = 12;
    pub const MAX_RETRIES: u16 = 0;
    pub const TIMEOUT_SECONDS: u16 = 12;
    pub const REQUIRED_EXCLUDE_GLOBS: [&str;1] = [
        fs::sync::sync_data::PATH
    ];

    pub const REMOTE_BUCKET: &str = "sync";
    pub const REMOTE_REGION: &str = "auto";
}

