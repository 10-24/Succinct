use std::{env::VarError, fmt, path::PathBuf};

use anyhow::{Result, bail};
use directories::UserDirs;

use crate::{BStr, constants::{self, DEFAULT_SYNC_DIR_NAME}};

pub struct Environment {
    pub sync_dir: PathBuf,
    pub r2_secret: String,
}

impl Environment {
    pub fn new() -> Result<Self> {
        let enviroment = Self {
            sync_dir: Self::get_sync_dir()?,
            r2_secret: Self::get_r2_secret()?,
        };
        Ok(enviroment)
    }

    fn get_sync_dir() -> Result<PathBuf> {
        match std::env::var(constants::env::SYNC_DIR) {
            Ok(path_str) => {
                Ok(PathBuf::from(path_str))
            }
            Err(VarError::NotPresent) => Ok(Self::default_sync_dir()),
            Err(e) => Err(Self::read_var_failed_err(constants::env::SYNC_DIR, e))
        }
    }

    fn get_r2_secret() -> Result<String> {
        std::env::var(constants::env::SYNC_DIR).map_err(|e| Self::read_var_failed_err(constants::env::SYNC_DIR, e))
    }

    fn default_sync_dir() -> PathBuf {
        let user_dir = UserDirs::new().unwrap();
        user_dir.home_dir().join(DEFAULT_SYNC_DIR_NAME)
    }


    fn read_var_failed_err(var: &str, error: impl fmt::Display) -> anyhow::Error {
        anyhow::format_err!("Failed to read enviroment variable {var}: {error}")
    }

}
