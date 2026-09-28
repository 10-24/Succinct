use std::path::{Path, PathBuf};

use crate::constants::CONFIG_FILE_NAME;





pub type BPath = Box<Path>;

pub struct AppPaths {
    config_file: BPath,
    db_file: BPath,
}

impl AppPaths {
    pub fn from(sync_dir: &Path) -> Self {

        let config_file = sync_dir.join(CONFIG_FILE_NAME).into();
        let db_file = sync_dir.join("").into();
        Self {
            config_file,
            db_file,
        }
    }
}

