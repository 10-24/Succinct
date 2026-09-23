use std::{collections::HashMap, path::PathBuf, time::Duration};

use directories::{ProjectDirs, UserDirs};
use rustc_hash::FxHashMap;



const IGNORE_FILE_NAME: &str = ".succinctignore";
const ROOT_DIR_NAME:&str = "sync";
const DB_FILE_NAME: &str = "state.db";
const DEBOUNCE_DURATION: Duration = Duration::from_secs(12);
const ROOT_DIR_NAME: &str = "sync";

pub struct DefaultConfig {
    pub remote_drive_config: FxHashMap<String,String>,
    pub local_root_dir: PathBuf,
    pub ignore_file: PathBuf,
    pub db_file: PathBuf,
    pub debounce_duration: Duration,
}

impl DefaultConfig {
    pub fn from_dirs(project_dirs: &ProjectDirs, user_dirs: &UserDirs) -> Self {
        
        let home_dir = user_dirs.home_dir();
        let data_dir = project_dirs.data_dir();
        let config_dir = project_dirs.config_dir();
        
        let local_root_dir = home_dir.join(ROOT_DIR_NAME);
        let ignore_file = local_root_dir.join(IGNORE_FILE_NAME);
        let db_file = data_dir.join(DB_FILE_NAME);
        let remote_drive_config = HashMap::default();
        let debounce_duration = DEBOUNCE_DURATION;
        
        Self {
            local_root_dir,
            ignore_file,
            db_file,
            remote_drive_config,
            debounce_duration,
        }
    }
}
