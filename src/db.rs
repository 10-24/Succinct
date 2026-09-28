use std::{default, path::Path};

use fjall::{Database,  Result, SingleWriterTxDatabase};


use tokio::{io::{self, AsyncWriteExt}, task::spawn_blocking};

use crate::db::keyspace::{Keyspace, Ks};


pub mod keyspace;

#[derive(Clone)]
pub struct Db(Database);

impl Db {
    pub async fn new(path: &Path) -> Result<Self> {
        let db_builder = Database::builder(path).worker_threads(1);
        spawn_blocking(|| db_builder.open()).await.unwrap().map(Self)
        
    }
    pub fn keyspace<K: Ks>(&self) -> Keyspace<K> {
        let raw_keyspace = self.0.keyspace(K::NAME, Default::default).unwrap();
        Keyspace::<K>::new(raw_keyspace)
    }


}



