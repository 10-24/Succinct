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

    pub const CONFIG_NAME: &str = "config.toml";
    
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
    use crate::constants::fs;

    pub const DEBOUNCE: Duration = Duration::from_secs(12);
    pub const REQUIRED_EXCLUDE_GLOBS: [&str;1] = [
        fs::sync::sync_data::PATH
    ];
    pub const IGNORE_DELETIONS: bool = false;
    
    pub mod remote {
        use std::time::Duration;

        pub const BUCKET: &str = "sync";
        pub const REGION: &str = "auto";
        pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
        pub const MAX_RETRIES: u64 = 0;
    }
}

