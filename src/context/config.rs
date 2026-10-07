use std::{collections::HashMap, error::Error, path::{Path, PathBuf}, sync::Arc, time::Duration};



use camino::Utf8Path;
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
    
    pub remote: RemoteConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[serde_inline_default]
struct RemoteConfig {
    key_id: Box<str>,
    key_secret: Box<str>,
    #[serde_inline_default(DEFAULT_BUCKET_REGION.into())]
    region: Box<str>,
    #[serde_inline_default(DEFAULT_BUCKET_NAME.into())]
    bucket: Box<str>,
}



