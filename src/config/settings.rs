use std::path::PathBuf;

use opendal::{Operator, OperatorBuilder};

use crate::{
    config::{APP_NAME, LOCAL_DATABASE_DIR, config::Config},
    path::{AbsPath, Local},
};

impl Config {
    pub async fn init_local_database() -> redb::Database {
        let path = Self::local_database_path();
        redb::Database::create(&path)
            .unwrap_or_else(|e| panic!("Failed to open redb database at {path:?}: {e}"))
    }

    fn local_database_path() -> PathBuf {
        dirs::data_dir()
            .unwrap()
            .join(APP_NAME)
            .join(LOCAL_DATABASE_DIR)
    }

    pub async fn connect_remote_drive(&self) -> opendal::Operator {
        let drive_cfg = self.remote.drive.to_owned();

        Operator::via_iter(drive_cfg.kind.as_ref(), drive_cfg.config.into_iter()).unwrap()
    }
}
