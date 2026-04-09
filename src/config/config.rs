use crate::{
    config::{
        APP_NAME, DEFAULT_DEBOUNCE_DURATION, DEFAULT_ROOT_DIR_NAME, SETTINGS_FILE_NAME,
        SUPPORTED_DRIVES_LINK,
    },
    path::{AbsPath, Local, Remote},
};
use colored::Colorize;
use derive_more::Deref;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::{str::FromStr, sync::Arc, time::Duration};

use tokio::io;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub remote: RemoteConfig,
    pub local: LocalConfig,

    /// Optional
    #[serde(default = "default_debounce_duration")]
    pub debounce_duration: Duration,
}

#[derive(Debug, Deserialize, Clone)]
struct RemoteConfig {
    pub drive: RemoteDriveConfig,
}

/// See [SUPPORTED_DRIVES_LINK]
#[derive(Debug, Deserialize, Clone)]
struct RemoteDriveConfig {
    pub kind: DriveKind,
    pub config: FxHashMap<Box<str>, Box<str>>,
}

#[derive(Default, Debug, Deserialize)]
struct LocalConfig {
    /// Optional
    #[serde(default = "default_local_root_dir")]
    pub root_path: AbsPath<Local>,
}

impl Config {
    pub async fn load() -> Arc<Config> {
        let path = dirs::config_dir()
            .unwrap()
            .join(APP_NAME)
            .join(SETTINGS_FILE_NAME);

        let file_content = tokio::fs::read_to_string(&path)
            .await
            .unwrap_or_else(panic_required_file(path.as_path()));
        Arc::new(toml::from_str(&file_content).unwrap())
    }
}

#[derive(Debug, Deserialize, Deref, Clone)]
#[serde(try_from = "String")]
pub struct DriveKind(opendal::Scheme);
impl From<String> for DriveKind {
    fn from(s: String) -> Self {
        let parse_res = opendal::Scheme::from_str(&s);
        if let Ok(scheme) = parse_res
            && !matches!(scheme, opendal::Scheme::Custom(_))
        {
            return Self(scheme);
        }
        panic!(
            "
            Failed to parse drive kind `{}`.
                Full list of supported drives: ({})
                Err: {}
        ",
            s.red(),
            SUPPORTED_DRIVES_LINK,
            parse_res.unwrap_err()
        )
    }
}

fn default_debounce_duration() -> Duration {
    DEFAULT_DEBOUNCE_DURATION
}

fn default_local_root_dir() -> AbsPath<Local> {
    let path = dirs::home_dir().unwrap().join(DEFAULT_ROOT_DIR_NAME);
    AbsPath::new(path.to_string_lossy())
}

fn default_remote_root_dir() -> AbsPath<Remote> {
    AbsPath::new(DEFAULT_ROOT_DIR_NAME)
}

pub fn panic_required_file<T>(path: impl Into<AbsPath<Local>>) -> impl FnOnce(io::Error) -> T {
    move |err: io::Error| {
        let path = path.into();
        let file_name = path.file_name();
        match err.kind() {
            io::ErrorKind::NotFound => {
                panic!("File not found: {path}\n Are sure {file_name} exists?")
            }
            _ => panic!("Failed to read file: {path}\n Error: {err}",),
        }
    }
}
