use std::{path::Path, sync::Arc};



use camino::Utf8Path;
use object_store::aws::AmazonS3ConfigKey;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use serde_inline_default::serde_inline_default;
use serde_nixos::NixosType;

use crate::constants::{config_default};



#[derive(Debug,Deserialize, NixosType)]
#[nixos(auto_doc)]
#[serde_inline_default]
pub struct Config {
    
    pub sync_dir: Box<Path>,

    /// If a path matches a glob pattern, it will not be synced.
    #[serde(default)]
    #[nixos(default = "[]")]
    pub exclude: Vec<Box<str>>,

    /// Duration waited after the last file event before syncing.
    #[serde_inline_default(config_default::DEBOUNCE_DURATION_SECONDS)]
    pub debounce_duration_seconds: u16,

    /// Whether to delete the file locally if it was deleted in the remote.
    #[serde(default)]
    pub ignore_deletions: bool,
    
    pub remote: RemoteConfig
}




#[derive(Debug,Deserialize, NixosType)]
#[nixos(auto_doc)]
#[serde_inline_default]
pub struct RemoteConfig {
    pub endpoint: Box<str>,
    pub access_key_id: Box<str>,
    pub secret_access_key: Box<str>,
    
    /// [object_store::client::ClientOptions::with_timeout]
    #[serde_inline_default(config_default::TIMEOUT_SECONDS)]
    pub timeout_seconds: u16,
    
    /// [object_store::client::retry::RetryConfig]
    #[serde_inline_default(config_default::MAX_RETRIES)]
    pub max_retries: u16,

    #[serde_inline_default(config_default::REMOTE_REGION.into())]
    pub region: Box<str>,
    
    #[serde_inline_default(config_default::REMOTE_BUCKET.into())]
    pub bucket: Box<str>
}

