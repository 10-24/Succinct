use std::time::Duration;

use crate::{
    path::{AbsPath, Local, Remote},
    state::file::FileId,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};

pub const APP_NAME: &str = "succinct";
pub const SETTINGS_FILE_NAME: &str = "succinct.toml"; // In config directory
pub const IGNORE_FILE_NAME: &str = ".succinctignore";
pub const ROOT_PARENT_ID: FileId = FileId(0);
pub const ROOT_ID: FileId = FileId(-4513623453135682776);
pub const DATABASE_READ_CONNECTIONS:u32 = 4;
pub const DATABASE_WRITE_CONNECTIONS:u32 = 1;
const LOCAL_DATABASE_FILE_NAME: &str = "local_state.db";
const LOCAL_DATABASE_BUSY_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_DEBOUNCE_DURATION: Duration = Duration::from_secs(3);
const DEFAULT_ROOT_NAME: &str = "sync";

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    pub remote: RemoteConfig,
    pub local: LocalConfig,

    /// Optional
    #[serde(default = "default_debounce_duration")]
    pub debounce_duration: Duration,

    /// Optional
    #[serde(default = "default_root_dir")]
    pub local_root_path: AbsPath<Local>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct RemoteConfig {
    /// Required
    pub database_url: Box<str>,

    /// Optional
    #[serde(default = "default_root_dir")]
    pub root_path: AbsPath<Remote>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct LocalConfig {
    /// Optional
    #[serde(default = "default_root_dir")]
    pub root_path: AbsPath<Local>,
}

impl Config {
    pub async fn load() -> Config {
        let path = dirs::config_dir()
            .unwrap()
            .join(APP_NAME)
            .join(SETTINGS_FILE_NAME);
        let config_exists = tokio::fs::try_exists(&path).await.unwrap();
        if !config_exists {
            panic!(
                "Config file ({}) not found. Path: {:?}",
                SETTINGS_FILE_NAME, path
            );
        }
        let file_content = tokio::fs::read_to_string(&path).await.unwrap();
        toml::from_str(&file_content).unwrap()
    }
}

fn default_debounce_duration() -> Duration {
    DEFAULT_DEBOUNCE_DURATION
}

fn default_root_dir() -> AbsPath<Local> {
    let path = dirs::home_dir().unwrap().join(DEFAULT_ROOT_NAME);
    AbsPath::new(&path.to_string_lossy())
}

pub fn database_config() -> SqliteConnectOptions {
    let path = dirs::data_dir()
        .unwrap()
        .join(APP_NAME)
        .join(LOCAL_DATABASE_FILE_NAME);
    
    SqliteConnectOptions::new()
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5))
        .filename(path)
}
