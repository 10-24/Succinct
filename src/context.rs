use std::{hash::{Hash, Hasher}, path::Path};

use anyhow::{Context as _, Result};

use fjall::Database;
use tokio::{fs, try_join};

use crate::{constants::CONFIG_FILE_NAME, context::{app_paths::AppPaths, config::Config, environment::Environment}, util::fhasher::FHasher};

pub mod config;
mod create_ignore;
mod app_paths;
mod environment;

pub struct Context {
    config: Config,
    paths: AppPaths,
    machine_id: u16,
    environment: Environment,
    db: Database
}

impl Context {

    pub async fn load() -> Result<Context>  {
        let enviroment = Environment::new()?;
        
        let config_path = enviroment.sync_dir.join(CONFIG_FILE_NAME);
        let (config, machine_id) = try_join!(Self::read_config(&config_path), Self::read_machine_id())?;
        
        Ok(Self {
            config,
            machine_id,
            environment: enviroment,
        })
    }


    async fn read_config(path: &Path) -> Result<Config> {
        let config_str = fs::read_to_string(path).await?;
        
        toml::from_str(&config_str).with_context(|| format!("Failed to deserialize {} ({})", CONFIG_FILE_NAME, path.to_string_lossy()))
    }
    
    async fn read_machine_id() -> Result<u16> {
        const MACHINE_ID_PATH: &str = "/etc/machine-id";
        let id_str = fs::read_to_string(MACHINE_ID_PATH).await.with_context(|| format!("Failed to read machine id ({MACHINE_ID_PATH})"))?;

   
        let hash = FHasher::new().hash(&id_str.trim()).finish();
        Ok(hash as u16)
    }

 

  
}

