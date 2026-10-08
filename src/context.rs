
use std::{default, sync::Arc};

use anyhow::{Context as _, Result};
use camino::{Utf8Path, Utf8PathBuf};
use directories::BaseDirs;
use globset::{Glob, GlobSet, GlobSetBuilder};
use object_store::aws::{AmazonS3, AmazonS3Builder, AmazonS3ConfigKey};
use tokio::{fs, join};
use crate::{
    constants::{CONFIG_FILE_NAME, DEFAULT_REMOTE_CONFIG}, context::{config::{Config, RemoteConfig}, machine_id::MachineId},
};
pub mod machine_id;
pub mod config;

pub struct Context {
    pub config: Config,
    pub machine_id: MachineId,
    pub exclude: Arc<GlobSet>,
    pub object_store: AmazonS3
}

impl Context {
    pub async fn load() -> Result<Context> {
        let dirs = BaseDirs::new().unwrap();
        let home_dir = Utf8PathBuf::try_from(dirs.home_dir().to_owned()).unwrap();
        let config_file = home_dir.join(CONFIG_FILE_NAME);
        
        let (config, machine_id) = join!(Self::read_config(&config_file), MachineId::read());
        let (config,machine_id) = (config?, machine_id?);
        
        let object_store = Self::create_object_store(&config.remote).with_context(|| "Failed to create object store.")?;
        let exclude = Self::create_exclude(&config.exclude)?.into();
        Ok(Self { config, machine_id, exclude, object_store })
    }

    async fn read_config(path: &Utf8Path) -> Result<Config> {
        let config_str = fs::read_to_string(path).await?;

        toml::from_str(&config_str).with_context(|| {
            format!(
                "Failed to deserialize config ({path})",
            )
        })
    }

    fn create_exclude(globs: &[impl AsRef<str>]) -> Result<GlobSet> {
        let mut builder = GlobSetBuilder::new();
        for glob_str in globs {
            let glob = Glob::new(glob_str.as_ref())?;
            builder.add(glob);
        }
        Ok(builder.build()?)
    }

    fn create_object_store(config: &RemoteConfig) -> object_store::Result<AmazonS3> {
    
        let entries = DEFAULT_REMOTE_CONFIG
            .iter()
            .map(|(k, v)| (*k, v.as_ref()))
            .chain(config.iter().map(|(k, v)| (*k, v.as_ref())));
    
        let mut builder = AmazonS3Builder::new();
        for (key, value) in entries {
            builder = builder.with_config(key, value);
        }
        builder.build()
    }
}




