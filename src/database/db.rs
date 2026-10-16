use std::{marker::PhantomData, path::Path};

use derive_more::{Deref, DerefMut};
use redb::{CompactionError, DatabaseError, StorageBackend, WriteTransaction, backends::InMemoryBackend};
use tokio::{fs::{self, File}, io};

use crate::util::arc_path::{ArcPath, IntoArcPath};


#[derive(Debug, Deref, DerefMut)]
pub struct Db {
    #[deref]
    #[deref_mut]
    inner: redb::Database,
    path: ArcPath,
}

impl Db {
    pub async fn new(path: impl IntoArcPath) -> io::Result<Self> {
        let path = path.into_arc_path();
        let file = File::options()
            .write(true)
            .create(true)
            .open(path.as_ref())
            .await?;
        let file = file.try_into_std().unwrap();
        let inner = redb::Database::builder().create_file(file).unwrap();
        Ok(Self { inner, path })
    }


    pub async fn into_bytes(&mut self) -> anyhow::Result<Vec<u8>> {
        self.compact()?;
        fs::read(self.path.as_ref()).await.map_err(Into::into)
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<redb::Database, DatabaseError> {
        let backend = InMemoryBackend::new();
        backend.write(0, bytes).unwrap();
        redb::Database::builder().create_with_backend(backend)
    }
    
}


