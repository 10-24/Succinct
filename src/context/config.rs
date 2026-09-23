use std::path::Path;

use anyhow::Result
use directories::{ProjectDirs, UserDirs};

use crate::context::{constants::DEFAULT_ROOT_DIR_NAME, raw_config::RawConfig};



mod default_config;
mod raw_config;

impl Config {

    pub async fn from_file(path:&Path, project_dirs: &ProjectDirs) -> Result<Self> {
        
    }

    pub async fn from_str(s:&str, project_dirs: &ProjectDirs) -> Result<Self> {
        let user_dirs = UserDirs::new().unwrap();
        let raw_config = toml::from_str::<RawConfig>(s)?;
        
        let default_root_dir = user_dirs.home_dir().join(DEFAULT_ROOT_DIR_NAME);
        let default_db_file = project_dirs.config_dir().join(DEFAULT_DB)
    }
    pub fn default(project_dirs: &ProjectDirs, user_dirs: &UserDirs) -> Self {


        Config {
            remote: RemoteConfig {
                drive: RemoteDriveConfig {
                    kind: DriveKind(opendal::Scheme::Gdrive),
                    config: Some(HashMap::default()),
                },
            },
            local: LocalConfig { 
                root_dir: user_dirs.home_dir().join(&DEFAULT_ROOT_DIR_NAME).into(),
            },
            debounce_duration: DEFAULT_DEBOUNCE_DURATION.into(),
            db_file: project_dirs.data_dir().join(DATABASE_FILE_NAME).into(),
        }
    }

    pub fn ignore_file(&self) -> PathBuf {
        
    }
    
}

#[derive(Debug, Deserialize, Clone)]
struct RemoteConfig {
    pub drive: RemoteDriveConfig,
}







fn default_debounce_duration() -> Duration {
    DEFAULT_DEBOUNCE_DURATION
}
