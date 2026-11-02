use std::{path::PathBuf};

use camino::{Utf8Path, Utf8PathBuf};
use directories::{ProjectDirs};
use tokio::{io};

use crate::{constants, util::{arc_path::{ArcPath}, rel_path::{RelPath, RelPathManager}}};


type FilePath = Box<Utf8Path>;
type DirPath = Utf8PathBuf;

#[derive(Debug)]
pub struct AppPaths {
    pub sync_dir: RelPathManager,
    remote_state: RelPath,
    project_dirs: ProjectDirs,
}

impl AppPaths {
    async fn new(project_dirs: ProjectDirs, sync_dir: ArcPath) -> io::Result<Self> {
        
        let sync_dir = RelPathManager::new(sync_dir.clone());
        
        let remote_state = RelPath::from(constants::fs::sync::sync_data::state_db::PATH);
        
        Ok(Self {
            project_dirs,
            remote_state,
            sync_dir,
        })
    }



    pub fn remote_state(&self) -> &RelPath {
        &self.remote_state
    }

    
    pub fn staged_deltas_db(&self) -> PathBuf {
        self.project_dirs.data_dir().join(constants::fs::app_data::staged_deltas::PATH)
    }
    
    pub fn state_db(&self) -> FilePath {
        self.sync_dir.join(constants::fs::sync::sync_data::state_db::PATH).into()
    }
}



pub struct AppPathsBuilder {
    project_dirs: ProjectDirs,
}

impl AppPathsBuilder {
    pub fn new() -> Self {
        let project_dirs = ProjectDirs::from("", constants::ORGANIZATION, constants::APP_NAME).unwrap();
        Self {
            project_dirs,
        }
    }

    pub async fn build(self, sync_dir: ArcPath) -> io::Result<AppPaths> {
        AppPaths::new(self.project_dirs, sync_dir).await
    }

    pub fn config_file(&self) -> PathBuf {
        self.project_dirs.config_dir().join(constants::fs::app_config::config::PATH)
    }

    
}