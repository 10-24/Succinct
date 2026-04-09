use std::sync::Arc;

use chrono::{DateTime, Utc};
use redb::{ReadableDatabase, ReadableTable, WriteTransaction};
use tokio::sync::MutexGuard;

use crate::{
    database::{
        QueuedDeletion, QueuedUpdate,
        local_reader::{CHILDREN, FILES, QUEUED_DELETES, QUEUED_UPDATES},
    },
    state::{file::{File, FileKV}, file_id::FileId},
};

pub struct DbWriter<'a> {
    pub(crate) txn: WriteTransaction,
    pub(crate) guard: MutexGuard<'a,()>
}



impl DbWriter {
    
    
    pub fn commit(self) {
        self.txn.commit().unwrap();
    }

    pub fn ensure_tables(&self) {
        self.txn.open_table(FILES).unwrap();
        self.txn.open_multimap_table(CHILDREN).unwrap();
        self.txn.open_table(QUEUED_UPDATES).unwrap();
        self.txn.open_table(QUEUED_DELETES).unwrap();
    }

    /// Removes a file from the files table and the children index.
    /// Does nothing if the file doesn't exist.
    pub fn delete_file(&self, id: FileId) {
        let mut files = self.txn.open_table(FILES).unwrap();
        let mut children = self.txn.open_multimap_table(CHILDREN).unwrap();

        if let Some(bytes) = files.remove(&id.0).unwrap() {
            let file: File = bincode::deserialize(&bytes.value()).unwrap();
            children.remove(&file.parent_id.0, &id.0).unwrap();
        }
    }

    /// Removes the file from the delete queue and adds it to the update queue.
    pub fn enqueue_update(&self, update: QueuedUpdate) {
        let mut queued_updates = self.txn.open_table(QUEUED_UPDATES).unwrap();
        let mut queued_deletes = self.txn.open_table(QUEUED_DELETES).unwrap();

        queued_deletes.remove(&update.file_id.0).unwrap();
        queued_updates.insert(&update.file_id.0, &()).unwrap();
    }

    /// Removes the file from the update queue and adds it to the delete queue.
    pub fn enqueue_delete(&self, deletion: QueuedDeletion) {
        let mut queued_updates = self.txn.open_table(QUEUED_UPDATES).unwrap();
        let mut queued_deletes = self.txn.open_table(QUEUED_DELETES).unwrap();

        queued_updates.remove(&deletion.file_id.0).unwrap();
        let bytes = bincode::serialize(&deletion).unwrap();
        queued_deletes.insert(&deletion.file_id.0, &bytes).unwrap();
    }

    pub fn dequeue_update(&self, id: FileId) {
        let mut queued_updates = self.txn.open_table(QUEUED_UPDATES).unwrap();
        queued_updates.remove(&id.0).unwrap();
    }

    pub fn dequeue_delete(&self, id: FileId) {
        let mut queued_deletes = self.txn.open_table(QUEUED_DELETES).unwrap();
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

