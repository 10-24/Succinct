use std::io::ErrorKind;

use opendal::{Operator, OperatorBuilder};
use sqlx::{SqlitePool, sqlite::{SqliteConnectOptions, SqliteJournalMode}};
use tokio::fs::{self, OpenOptions};

use crate::{config::{APP_NAME, LOCAL_DATABASE_BUSY_TIMEOUT, LOCAL_DATABASE_FILE_NAME, config::Config}, database::{local_reader::LocalReader, local_writer::LocalWriter}, path::{AbsPath, Local}};



impl Config {
    
    pub async fn init_local_database() -> SqliteConnectOptions {
        let (file_path, opts) = Self::local_database_options();
        Self::ensure_db_file(&file_path).await;
        opts
    }
    
    fn local_database_options() -> (AbsPath<Local>,SqliteConnectOptions) {
        let path = dirs::data_dir()
            .unwrap()
            .join(APP_NAME)
            .join(LOCAL_DATABASE_FILE_NAME);
        let path = AbsPath::from(path.as_path());
        
        let db_opts = SqliteConnectOptions::new()
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(LOCAL_DATABASE_BUSY_TIMEOUT)
            .filename(path.as_ref());
        (path, db_opts)
    }
    
    pub async fn ensure_db_file(path:&AbsPath<Local>){
        let try_create_res = OpenOptions::new().write(true).create(true).open(path.as_ref()).await;
        if let Err(e) = try_create_res {
            if e.kind() != ErrorKind::AlreadyExists {
                panic!("Failed to ensure database file. Path: {path}\n Err: {e}")
            }
        }
    }
    
    pub async fn connect_remote_drive(&self) -> opendal::Operator {
        let drive_cfg = self.remote.drive.to_owned();
        
        Operator::via_iter(drive_cfg.kind.as_ref(), drive_cfg.config.into_iter()).unwrap()
    }
}
