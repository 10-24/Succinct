use std::{
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result};

use camino::{Utf8Path, Utf8PathBuf};
use directories::{BaseDirs, ProjectDirs};
use fjall::Database;
use futures::future::try_join_all;
use globset::{Glob, GlobSet, GlobSetBuilder};
use tokio::{fs, join, try_join};
use xxhash_rust::xxh32::xxh32;

use crate::{
    constants::{self, CONFIG_FILE_NAME}, context::{config::Config, create_ignore::create_exclude, machine_id::MachineId}, util::fhasher::FHasher,
};
pub mod machine_id;
pub mod config;
mod create_ignore;

pub struct Context {
    config: Config,
    machine_id: MachineId,
    exclude: GlobSet,
}

impl Context {
    pub async fn load() -> Result<Context> {
        let dirs = BaseDirs::new().unwrap();
        let home_dir = Utf8PathBuf::try_from(dirs.home_dir().to_owned()).unwrap();
        let config_file = home_dir.join(CONFIG_FILE_NAME);
        
        let (config, machine_id) = join!(Self::read_config(&config_file), MachineId::read());
        let (config,machine_id) = (config?, machine_id?);
        
        let exclude = Self::create_exclude(&config.exclude)?;
        Ok(Self { config, machine_id, exclude })
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
   
}




