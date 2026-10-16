use std::{error::Error as StdError, iter, str::FromStr};

use camino::{Utf8Path, Utf8PathBuf};
use object_store::{self as obj, ObjectStoreExt, aws::AmazonS3};
use redb::{Database, StorageBackend};
use reqwest::StatusCode;
use strum::Display;
use tokio_util::bytes::Bytes;

use crate::{context::ContextInner, database::db::Db};

pub async fn get_bytes(drive: &AmazonS3, path: &obj::path::Path) -> Result<Option<Bytes>, RemoteDriveError> {
    match drive.get(&path).await {
        Ok(data) => Ok(Some(data.bytes().await?)),
        Err(obj::Error::NotFound { .. }) => Ok(None),
        Err(e) => Err(e.into()),
    }
}



type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;
#[derive(Debug, thiserror::Error)]
pub enum RemoteDriveError {
    #[error("Network error: {message} (status={status:?})")]
    Network {
        message: String,
        is_timeout: bool,
        is_connect: bool,
        status: Option<StatusCode>,
    },
    #[error("Authentication error: {0}")]
    Auth(obj::Error),
    #[error("File not found in remote drive: `{0}`")]
    FileNotFound(Box<str>),
    
    #[error("Unknown error: {0}")]
    Unknown(obj::Error)
}

impl From<obj::Error> for RemoteDriveError {
    fn from(err: obj::Error) -> Self {
       
        match err {
            obj::Error::Generic { ref source, .. }  => {
                Self::try_extract_network_err(source.as_ref()).unwrap_or(Self::Unknown(err))
            }
            obj::Error::PermissionDenied {..} => Self::Auth(err),
            obj::Error::Unauthenticated {..} => Self::Auth(err),
            obj::Error::UnknownConfigurationKey {..} => Self::Auth(err),
            obj::Error::NotFound { path, .. } => Self::FileNotFound(path.into()),
     
            _ => Self::Unknown(err),
        }
    }
}



impl RemoteDriveError {
    fn try_extract_network_err(err:&(dyn StdError + 'static)) -> Option<Self> {
      
        if let Some(reqwest_err) = err.downcast_ref::<reqwest::Error>() {
            return Some(Self::from_reqwest_err(reqwest_err));
        }
        if let Some(source_err) = err.source() {
            return Self::try_extract_network_err(source_err)
        }
        None
        
    }

    fn from_reqwest_err(err: &reqwest::Error) -> Self {
        Self::Network { message: err.to_string(), is_timeout: err.is_timeout(), is_connect: err.is_connect(), status: err.status() }
    }

    pub fn is_network_err(&self) -> bool {
        matches!(self, Self::Network {..})
    }
}