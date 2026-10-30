use std::{path::PathBuf, sync::Arc};

use async_tempfile::TempDir;
use camino::{Utf8Path, Utf8PathBuf};
use const_format::concatcp;
use directories::{BaseDirs, ProjectDirs};
use object_store::path::Path as ObjPath;
use tokio::{fs, io};

use crate::{constants, context::config::Config, util::{arc_path::{ArcPath, IntoArcPath}, rel_path::{RelPath, RelPathManager}}};


type FilePath = Box<Utf8Path>;
type DirPath = Utf8PathBuf;

#[derive(Debug)]
pub struct AppPaths {
    pub sync_dir: RelPathManager,
    remote_state: RelPath,
    tmp_dir: ArcPath,
    tmp_dir_handle: Arc<TempDir>,
}

impl AppPaths {
    async fn new(project_dirs: ProjectDirs, sync_dir: ArcPath) -> io::Result<Self> {
        
        let sync_dir = RelPathManager::new(sync_dir.clone());
        
        let remote_state = RelPath::from(constants::fs::sync::sync_data::state_db::PATH);
        
        let sync_data_dir = sync_dir.join_str(constants::fs::sync::sync_data::PATH);
        let tmp_dir_handle = Arc::from(Self::create_tmp_dir(&sync_data_dir).await?);
        let tmp_dir = tmp_dir_handle.to_path_buf().into_arc_path();
        
        Ok(Self {
            tmp_dir,
            remote_state,
            tmp_dir_handle,
            sync_dir,
        })
    }


    async fn create_tmp_dir(sync_data_dir: &Utf8Path) -> io::Result<TempDir> {
        
        match TempDir::new_in(sync_data_dir.as_std_path()).await {
            Ok(dir) => Ok(dir),
            Err(async_tempfile::Error::Io(e)) => Err(e),
            Err(e) => panic!("Unhandled TempDir Error: {e}")
        }
    }
    pub fn remote_state(&self) -> &RelPath {
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



    pub fn tmp_dir(&self) -> &Utf8Path {
        &self.tmp_dir
    }

    pub fn from_sync_dir(&self, relpath: &str) -> FilePath {
        self.sync_dir.join(relpath).into()
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

pub struct AppPathsBuilder {
    project_dirs: ProjectDirs,
    config_file: PathBuf,
}
impl AppPathsBuilder {
    pub fn new() -> Self {
        let project_dirs = ProjectDirs::from("", constants::ORGANIZATION, constants::APP_NAME).unwrap();
        let config_file = project_dirs.config_dir().join(constants::fs::CONFIG_NAME);
        Self {
            project_dirs,
            config_file,
        }
    }



    pub async fn build(self, sync_dir: ArcPath) -> io::Result<AppPaths> {
        AppPaths::new(self.project_dirs, sync_dir).await
    }

    pub fn config(&self) -> &std::path::Path {
        &self.config_file
    }
}