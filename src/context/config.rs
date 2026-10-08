use std::{collections::HashMap, error::Error, path::{Path, PathBuf}, sync::Arc, time::Duration};



use camino::Utf8Path;
use object_store::aws::AmazonS3ConfigKey;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use serde_inline_default::serde_inline_default;

use crate::constants::{self, DEFAULT_BUCKET_NAME, DEFAULT_BUCKET_REGION, DEFAULT_DEBOUNCE_DURATION};

#[derive(Debug,Deserialize)]
#[serde_inline_default]
pub struct Config {
    
    pub sync_dir: Arc<Utf8Path>,
    
    #[serde(default)]
    pub exclude: Vec<Box<str>>,
    
    #[serde_inline_default(DEFAULT_DEBOUNCE_DURATION)]
    pub debounce_duration: Duration,
    
    /// #[AmazonS3ConfigKey]: 
    pub remote: RemoteConfig,
}





pub type RemoteConfig = FxHashMap<AmazonS3ConfigKey, Box<str>>;