use std::{path::Path, sync::Arc};



use camino::Utf8Path;
use globset::GlobSetBuilder;
use object_store::aws::AmazonS3ConfigKey;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use serde_nixos::{NixosType,NixosTypeGenerator};

use crate::{util::globs::Glob};




#[derive(Debug,Deserialize, NixosType)]
#[nixos(auto_doc)]
pub struct RawConfig {
    
    pub sync_dir: Box<Path>,

    /// If a path matches a glob pattern, it will not be synced.
    pub exclude: Vec<Glob>,

    /// Duration waited after the last file event before syncing.
    pub debounce_seconds: Option<u64>,

    /// Whether to delete the file locally if it was deleted in the remote.
    pub ignore_deletions: Option<bool>,
    
    pub remote: RawRemote
}




#[derive(Debug,Deserialize, NixosType)]
#[nixos(auto_doc)]
pub struct RawRemote {
    pub endpoint: Box<str>,
    pub access_key_id: Box<str>,
    pub secret_access_key: Box<str>,

    /// [object_store::client::ClientOptions::with_timeout]
    /// 
    /// Defaults to [crate::constants::config_default::remote::TIMEOUT].
    pub timeout_seconds: Option<u64>,
    
    /// [object_store::client::retry::RetryConfig::max_retries]
    /// 
    /// Defaults to [`crate::constants::config_default::remote::MAX_RETRIES`].
    pub max_retries: Option<u16>,
    
    /// [object_store::client::ClientOptions::with_region]
    /// 
    /// Defaults to [`crate::constants::config_default::remote::REGION`].
    pub region: Option<Box<str>>,

    /// [object_store::client::ClientOptions::with_bucket]
    /// 
    /// Defaults to [`crate::constants::config_default::remote::BUCKET`].
    pub bucket: Option<Box<str>>
}

macro_rules! declare_config {
    () => {
        
    };
}