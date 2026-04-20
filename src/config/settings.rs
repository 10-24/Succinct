use std::path::PathBuf;

use opendal::{Operator};

use crate::{
    config::{APP_NAME, LOCAL_DATABASE_DIR, SUPPORTED_DRIVES_LINK, config::Config},
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

        match Operator::via_iter(*drive_cfg.kind, drive_cfg.config) {
            Ok(operator) => operator,
            Err(err) => panic!("
                Failed to connect remote-drive operator.
                    Error: {err}
                    Config: {:?}
                See {SUPPORTED_DRIVES_LINK}
            ",
            &self.remote.drive),
        }
        
    }
}
