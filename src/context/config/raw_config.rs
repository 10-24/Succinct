use std::{path::PathBuf, time::Duration};

use rustc_hash::FxHashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RawConfig {
    pub remote: RawRemoteConfig,
    pub local: Option<RawLocalConfig>,

    pub debounce_duration: Option<Duration>,
    pub db_file: Option<PathBuf>,
    pub ignore_file: Option<PathBuf>,
}

/// See [SUPPORTED_DRIVES_LINK]
#[derive(Debug, Deserialize)]
struct RawRemoteConfig {
    pub kind: String,
    pub config: Option<FxHashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct RawLocalConfig {
    pub root_dir: Option<PathBuf>,
}