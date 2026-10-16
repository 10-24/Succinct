use std::{path::PathBuf, sync::Arc};

use async_tempfile::TempDir;
use camino::{Utf8Path, Utf8PathBuf};
use const_format::concatcp;
use directories::{BaseDirs, ProjectDirs};
use object_store::path::Path as ObjPath;
use tokio::{fs, io};

use crate::{constants, context::config::Config, util::arc_path::{ArcPath, IntoArcPath}};


type FilePath = Box<Utf8Path>;
type DirPath = Utf8PathBuf;

#[derive(Debug, Clone)]
pub struct AppPaths {
    sync_dir: ArcPath,
    remote_state: ObjPath,
    tmp_dir: ArcPath,
    tmp_dir_handle: Arc<TempDir>,
}

impl AppPaths {
    pub async fn new(config: &Config) -> io::Result<Self> {
        let sync_dir = config.sync_dir.clone();
        let remote_state = ObjPath::from(constants::fs::sync::sync_data::state_db::PATH);
        
        let sync_data_dir = sync_dir.join(constants::fs::sync::sync_data::PATH);
        let tmp_dir_handle = Arc::from(Self::create_tmp_dir(&sync_data_dir).await?);
        let tmp_dir = tmp_dir_handle.to_path_buf().into_arc_path();
        
        Ok(Self {
            tmp_dir,
            remote_state,
            tmp_dir_handle,
            sync_dir,
        })
    }

    
    fn project_dirs() -> ProjectDirs  {
        ProjectDirs::from("", constants::ORGANIZATION, constants::APP_NAME).unwrap()
    }
    
    fn with_sync_data_dir(sync_dir: &Utf8Path) -> DirPath {
        sync_dir.join(constants::fs::sync::sync_data::PATH)
    }
    
    async fn create_tmp_dir(sync_data_dir: &Utf8Path) -> io::Result<TempDir> {
        
        match TempDir::new_in(sync_data_dir.as_std_path()).await {
            Ok(dir) => Ok(dir),
            Err(async_tempfile::Error::Io(e)) => Err(e),
            Err(e) => panic!("Unhandled TempDir Error: {e}")
        }
    }
    pub fn remote_state(&self) -> &ObjPath {
        &self.remote_state
    }

    fn base_dirs() -> BaseDirs {
        BaseDirs::new().unwrap()
    }
    
    pub fn staged_deltas_db(&self) -> FilePath {
        self.sync_dir.join(constants::fs::sync::sync_data::staged_deltas_db::PATH).into()
    }
    
    pub fn state_db(&self) -> FilePath {
        self.sync_dir.join(constants::fs::sync::sync_data::state_db::PATH).into()
    }

    pub fn config_file() -> PathBuf {
        Self::base_dirs().home_dir().join(constants::fs::CONFIG_NAME)
    }

    pub fn sync_data_dir(&self) -> DirPath {
        Self::with_sync_data_dir(&self.sync_dir)
    }

    pub fn tmp_dir(&self) -> &Utf8Path {
        &self.tmp_dir
    }

  
}


/// For functional extending without redundant clones
pub trait JoinWith {
    fn join_with(self, path: impl AsRef<Utf8Path>) -> Utf8PathBuf;
}

impl JoinWith for Utf8PathBuf {
    fn join_with(mut self, path: impl AsRef<Utf8Path>) -> Utf8PathBuf {
        self.push(path);
        self
    }
}