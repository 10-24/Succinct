use std::{collections::HashMap, path::Path, time::Duration};

use anyhow::Result;
use directories::{ProjectDirs, UserDirs};
use serde::Deserialize;


use crate::{constants::{DEFAULT_BUCKET_NAME, DEFAULT_DEBOUNCE_DURATION}, context::{app_paths::BPath, config::default_config::DefaultConfig, constants::{DEFAULT_BUCKET_NAME, DEFAULT_ROOT_DIR_NAME}, raw_config::RawConfig}};




#[derive(Debug,Deserialize)]
pub struct Config {
    pub debounce_duration: Duration,
    #[serde(default)]
    pub exclude: Option<Vec<String>>,
    pub remote: RemoteConfig,
}


#[derive(Debug, Deserialize, Clone)]
struct RemoteConfig {
    account_id: String,
    key_id: String,
    #[serde(default = "default_bucket")]
    bucket: String,
}





fn default_debounce_duration() -> Duration {
    DEFAULT_DEBOUNCE_DURATION
}
fn default_bucket() -> String {
    DEFAULT_BUCKET_NAME.into()
}