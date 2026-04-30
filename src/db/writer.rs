use chrono::{DateTime, Utc};
use redb::{ReadableTable, WriteTransaction};
use crate::db::tables::{CHILDREN, FILES, QUEUED_DELETES, QUEUED_UPDATES, file::{File, FileId, FileIdOrd}};

pub struct DbWriter {
    pub(crate) txn: WriteTransaction,
    // pub(crate) guard: MutexGuard<'a, ()>,
}

impl DbWriter {
    pub fn commit(self) {
        self.txn.commit().unwrap();
    }

}
