use std::time::Duration;

use crate::db::tables::file::{FileId, FileIdOrd, FileName};


pub mod config;
pub mod create_ignore;
pub mod settings;

pub const APP_NAME: &str = "succinct";
pub const SETTINGS_FILE_NAME: &str = "succinct.toml"; // In config directory
pub const IGNORE_FILE_NAME: &str = ".succinctignore";
pub const ROOT_PARENT_ID: FileIdOrd = FileIdOrd {
    depth: 1,
    value: FileId::constant(&0),
};
pub const ROOT_ID: FileIdOrd = FileIdOrd {
    depth: 2,
    value: FileId::constant(&10399743216161163099),
};
pub const ROOT_NAME: FileName = FileName::constant("sync");
pub const LOCAL_DATABASE_DIR: &str = "local_state_db";
pub const DEFAULT_DEBOUNCE_DURATION: Duration = Duration::from_secs(3);
pub const DEFAULT_ROOT_DIR_NAME: &str = "sync";
pub const SUPPORTED_DRIVES_LINK: &str =
    "https://opendal.apache.org/docs/rust/opendal/services/index.html";
