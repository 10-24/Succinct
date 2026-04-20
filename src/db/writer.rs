use std::sync::Arc;

use chrono::{DateTime, Utc};
use redb::{ReadableTable, WriteTransaction};
use tokio::sync::MutexGuard;

use crate::{
    db::tables::{CHILDREN, FILES, QUEUED_DELETES, QUEUED_UPDATES, file::FileIdOrd, macros::OpenWriteTable},
    state::{
        file::{File, FileKV},
        file_id::FileId,
    },
};

pub struct DbWriter {
    pub(crate) txn: WriteTransaction,
    // pub(crate) guard: MutexGuard<'a, ()>,
}

impl DbWriter {
    pub fn commit(self) {
        self.txn.commit().unwrap();
    }

    pub fn ensure_tables(&self) {
        tables_mut!(self, FILES, CHILDREN, QUEUED_UPDATES, QUEUED_DELETES);
    }

    /// Removes a file from the files table and the children index.
    /// Does nothing if the file doesn't exist.
    pub fn delete_file(&self, id: FileId) {
        let (files,children) = tables_mut!(self,FILES,CHILDREN);

        if let Some(bytes) = files.remove(&id.0).unwrap() {
            let file: File = bincode::deserialize(&bytes.value()).unwrap();
            children.remove(&file.parent_id.0, &id.0).unwrap();
        }
    }

    /// Removes the file from the delete queue and adds it to the update queue.
    pub fn enqueue_update(&self, id: FileIdOrd) {
        let (queued_updates,queued_deletes) = tables_mut!(self,QUEUED_UPDATES,QUEUED_DELETES);

        queued_deletes.remove(id).unwrap();
        queued_updates.insert(id,()).unwrap();
    }

    /// Removes the file from the update queue and adds it to the delete queue.
    pub fn enqueue_delete(&self, id: FileIdOrd) {
        let (queued_updates,queued_deletes) = tables_mut!(self,QUEUED_UPDATES,QUEUED_DELETES);

        queued_updates.remove(id).unwrap();
        queued_deletes.insert(id,()).unwrap();
    }

    pub fn dequeue_update(&self, id: FileId) {
        let queued_updates = tables_mut!(self,QUEUED_UPDATES);
        queued_updates.remove(&id.0).unwrap();
    }

    pub fn dequeue_delete(&self, id: FileId) {
        let queued_deletes = tables_mut!(self,QUEUED_DELETES);
        queued_deletes.remove(&id.0).unwrap();
    }

    /// Updates the hash and modified_at fields of an existing file.
    pub fn update_file(&self, id: FileId, hash: i32, modified_at: DateTime<Utc>) {
        let mut files = self.txn.open_table(FILES).unwrap();
        let bytes = files.get(&id.0).unwrap().unwrap();
        let mut file: File = bincode::deserialize(&bytes.value()).unwrap();
        file.hash = hash;
        file.modified_at = modified_at.timestamp();

        files.insert(&id.0, &file).unwrap();
    }

    pub fn ensure_file_exists(&self, file: FileKV) {
        let mut files = self.txn.open_table(FILES).unwrap();
        let mut children = self.txn.open_multimap_table(CHILDREN).unwrap();

        if files.get(file.id).unwrap().is_none() {
            files.insert(file.id, file.as_bytes()).unwrap();
            children.insert(&file.parent_id, &file.id).unwrap();
        }
    }
}
