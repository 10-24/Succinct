use std::hash::{Hash, Hasher};

use directories::ProjectDirs;
use globset::GlobSet;
use rustc_hash::FxHasher;
use tokio::{fs, io, join};

use crate::context::{config::Config, constants::{APP_NAME, CONFIG_FILE_NAME, ORG_NAME}};

pub mod constants;
pub mod config;
mod create_ignore;
mod app_paths;
struct Context {
    config: Config,
    machine_id: u16,
    dirs: ProjectDirs,
}

impl Context {

    pub async fn load() -> io::Result<Context>  {
        let dirs = Self::get_project_dirs().unwrap();
        let (machine_id,config) = join!(Self::fetch_machine_id(),)
    }
    
    async fn fetch_machine_id() -> io::Result<u16> {
        let id_str = fs::read_to_string("/etc/machine-id").await?;

        let mut hasher = FxHasher::default();
        id_str.trim().hash(&mut hasher);
        let hash = hasher.finish() as u16;
        Ok(hash)
    }

    pub fn get_project_dirs() -> Option<ProjectDirs> {
    }

    async fn fetch_config(path:) -> anyhow::Result<Config> {
         
        let config_file = project_dirs.config_dir().with_file_name(CONFIG_FILE_NAME);
        let config_str = fs::read_to_string(config_file).await?;
        let config = toml::from_str(&config_str)?;
        
        Ok(config)
    }
}

