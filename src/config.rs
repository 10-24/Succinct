use std::time::Duration;

use crate::path::AbsPath;
use serde::{Deserialize, Serialize};

pub const APP_NAME: &str = "succinct";
pub const SETTINGS_FILE_NAME: &str = "succinct.toml"; // In config directory
pub const IGNORE_FILE_NAME: &str = ".succinctignore";
const LOCAL_DATABASE_NAME: &str = "state.db";
const DEFAULT_DEBOUNCE_DURATION: Duration = Duration::from_secs(3);
const DEFAULT_ROOT_NAME: &str = "sync";

#[derive(Default,Debug, Serialize, Deserialize)]
pub struct Config {
  
    pub remote: RemoteConfig,
    pub local: LocalConfig,
    
    /// Optional
    #[serde(default = "default_debounce_duration")]
    pub debounce_duration: Duration,
    
    /// Optional
    #[serde(default = "default_root_dir")]
    pub local_root_path: AbsPath,
    
  
}

#[derive(Default,Debug, Serialize, Deserialize)]
struct RemoteConfig {
    /// Required
    pub database_url: Box<str>,
    
    /// Optional
    #[serde(default = "default_root_dir")]
    pub root_path: AbsPath,
}

#[derive(Default,Debug, Serialize, Deserialize)]
struct LocalConfig {
    /// Optional
    #[serde(default = "default_root_dir")]
    pub root_path: AbsPath,
}

impl Config {
    pub async fn load() -> Config {
        let path = dirs::config_dir().unwrap().join(APP_NAME).join(SETTINGS_FILE_NAME);
        let config_exists = tokio::fs::try_exists(&path).await.unwrap();
        if !config_exists {
            panic!("Config file ({}) not found. Path: {:?}", SETTINGS_FILE_NAME, path);
        }
        let file_content = tokio::fs::read_to_string(&path).await.unwrap();
        toml::from_str(&file_content).unwrap()
    }
}


fn default_debounce_duration() -> Duration {
    DEFAULT_DEBOUNCE_DURATION
}

fn default_root_dir() -> AbsPath {
    let path = dirs::home_dir().unwrap().join(DEFAULT_ROOT_NAME);
    AbsPath::new(&path.to_string_lossy())
}

