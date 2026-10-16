use std::{sync::Arc, time::Duration};



use camino::Utf8Path;
use object_store::aws::AmazonS3ConfigKey;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use serde_inline_default::serde_inline_default;

use crate::constants::{config_default};



#[derive(Debug,Deserialize)]
#[serde_inline_default]
pub struct Config {
    
    pub sync_dir: Arc<Utf8Path>,
    
    #[serde(default)]
    pub exclude: Vec<Box<str>>,
    
    #[serde_inline_default(config_default::DEBOUNCE_DURATION)]
    pub debounce_duration: Duration,
    
    #[serde(default)]
    pub ignore_deletions: bool,
    
    pub remote: RemoteConfig
}




#[derive(Debug,Deserialize)]
#[serde_inline_default]
pub struct RemoteConfig {
    pub endpoint: Box<str>,
    pub access_key_id: Box<str>,
    pub secret_access_key: Box<str>,
    
    /// [object_store::client::ClientOptions::with_timeout]
    #[serde_inline_default(config_default::TIMEOUT)]
    pub timeout: Duration,
    
    /// [object_store::client::retry::RetryConfig]
    #[serde_inline_default(config_default::MAX_RETRIES)]
    pub max_retries: u16,

    #[serde_inline_default(config_default::REMOTE_REGION.into())]
    pub region: Box<str>,
    
    #[serde_inline_default(config_default::REMOTE_BUCKET.into())]
    pub bucket:Box<str>
}