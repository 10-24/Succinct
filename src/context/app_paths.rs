use std::path::{Path};

use directories::ProjectDirs;

use crate::context::{config::Config, constants::{APP_NAME, CONFIG_FILE_NAME, ORG_NAME}};



pub type ConstPath = Box<Path>;

pub struct AppPaths {
    config: Box<Path>,
    ignore: Box<Path>,
    db: Box<Path>,
}

impl AppPaths {
    pub fn resolve(config) -> Option<Self> {
        let dirs = ProjectDirs::from("com", ORG_NAME, APP_NAME)?;
        Self {
            config: dirs.config_local_dir().join(CONFIG_FILE_NAME).into(),

            db: dirs.state_dir().unwrap().join(DB)
        }
    }
}
