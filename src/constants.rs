use std::time::Duration;

use const_format::concatcp;

pub mod env {
    pub const SYNC_DIR:&str = "SUCCINCT_DIR";
    pub const R2_SECRET:&str = "SUCCINCT_R2_SECRET";
}

pub const DEFAULT_SYNC_DIR_NAME:&str = "sync";
pub const DEFAULT_DEBOUNCE_DURATION: Duration = Duration::from_secs(12);
pub const DEFAULT_BUCKET_NAME: &str = DEFAULT_SYNC_DIR_NAME;
pub const CONFIG_FILE_NAME: &str = "succinct.toml";
pub const DB_FILE_NAME: &str = "succinct.db";
pub const REQUIRED_EXCLUDE_GLOBS: &[&str] = &[
    concatcp!('/', CONFIG_FILE_NAME),
];