use crate::db::remote::RemoteDb;
use crate::db::tables::CHILDREN;
use crate::db::tables::ChildrenTableMut;
use crate::db::tables::FILES;
use crate::db::tables::FilesTableMut;
use crate::db::tables::QUEUED_CREATES;
use crate::db::tables::QUEUED_DELETES;
use crate::db::tables::QUEUED_UPDATES;
use crate::db::tables::QueuedCreatesTableMut;
use crate::db::tables::QueuedDeletesTableMut;
use crate::db::tables::QueuedUpdatesTableMut;
use crate::db::tables::file::File;
use crate::db::tables::file::FileId;
use crate::db::tables::file::FileIdOrd;
use crate::db::tables::queued_deltas::QueuedDelete;
use crate::db::tables::timestamp::Timestamp;
use crate::state::file_id::FileId;

use crate::state::file_id::FileIdOrd;
use crate::state::prepare_deltas::DeltaKV;
use crate::state::prepare_deltas::DeltaValue;
use crate::state::remote_drive::remote_drive::RemoteDrive;
use crate::tables_mut;
use crate::{
    db::{Db, tables::QueuedDeletion},
    delta::{Delta, DeltaKind},
    path::{AbsPath},
    state::file::File,
};
use chrono::{DateTime, Utc};
use redb::ReadableMultimapTable;
use redb::ReadableTable;
use std::collections::BTreeMap;

pub struct State {
    pub(crate) local_db: Db,
    pub(crate) remote_db: RemoteDb,
    pub(crate) drive: RemoteDrive,
    pub(crate) local_root: AbsPath,
}

impl State {
    

    /// File and Folder creations cannot be implicit (Eg. If a parent and child file/folder are created, both must be included)
    pub async fn push_deltas(&mut self, deltas: BTreeMap<FileIdOrd, Delta>) {
        let txn = self.local_db.begin_write();
        let (files_table, children_table, creates, updates, deletes) = tables_mut!(&txn, FILES, CHILDREN, QUEUED_CREATES, QUEUED_UPDATES, QUEUED_DELETES);
        let queues = Queues { creates, updates, deletes };
        let now = Timestamp::now();
        
        let mut complete_deltas = self.instantiate_files_and_complete_tree(deltas, now, &mut files_table, &mut children_table);
        while let Some((id,delta)) = complete_deltas.pop_last() {
            match delta.kind {
                DeltaKind::Create => {
                    self.update_entry(id,now,&mut files_table,&mut children_table);
                    Self::enqueue_create(id, &mut queues);
                },
                DeltaKind::Update => {
                    self.update_entry(id,now,&mut files_table,&children_table);
                    Self::enqueue_update(id, &mut queues);
                },
                DeltaKind::Delete => {
                    let parent_id = self.delete_entry(id,&mut files_table,&mut children_table);
                    Self::enqueue_delete(id, &mut queues, now, parent_id);
                },
            }
        }
        txn.commit();
    }

    fn create_entry(&self, id:FileIdOrd, now:Timestamp, files_table: &mut FilesTableMut, children_table: &mut ChildrenTableMut) {

        let mut file_entry = files_table.get_mut(*id).unwrap().unwrap();
        let prev_file:&File = file_entry.value();
        children_table.insert(prev_file.parent_id(), *id);
        match prev_file.is_dir() {
            true => {
                let child_hashes = get_child_hashes(*id, files_table, children_table);
                let modified_file = prev_file.modify(*id, now, child_hashes);
                file_entry.insert(modified_file);
            }
            false => {
                let modified_file = prev_file.modify(*id, now, None);
                file_entry.insert(modified_file);
            }
        }
    }

    
    fn update_entry(&self, id:FileIdOrd, now:Timestamp, files_table: &mut FilesTableMut, children_table: &ChildrenTableMut) {

        let mut file_entry = files_table.get_mut(*id).unwrap().unwrap();
        let prev_file:&File = file_entry.value();
        match prev_file.is_dir() {
            true => {
                let child_hashes = get_child_hashes(*id, files_table, children_table);
                let modified_file = prev_file.modify(*id, now, child_hashes);
                file_entry.insert(modified_file);
            }
            false => {
                let modified_file = prev_file.modify(*id, now, None);
                file_entry.insert(modified_file);
            }
        }
    }
    
    

    fn delete_entry(&self, id:FileIdOrd,files_table: &mut FilesTableMut, children_table: &mut ChildrenTableMut) -> FileId {
        files_table.remove(*id);
    }

    fn enqueue_create(id: FileIdOrd, queues: &mut Queues<'_>) {
        queues.deletes.remove(*id);
        queues.updates.remove(*id);
        queues.creates.insert(*id, ());
    }
    fn enqueue_update(id: FileIdOrd, queues: &mut Queues<'_>) {
        queues.deletes.remove(*id);
        queues.creates.remove(*id);
        queues.updates.insert(*id, ());
    }
    fn enqueue_delete(id: FileIdOrd, queues: &mut Queues<'_>, now: Timestamp, parent_id: FileId) {
        queues.creates.remove(*id);
        queues.updates.remove(*id);
        queues.deletes.insert(*id, QueuedDelete { timestamp: now, parent_id });
    }
    
}

fn get_child_hashes(parent_id: FileId, files_table: &FilesTableMut, children_table: &ChildrenTableMut) -> impl Iterator<Item = i32> {
    let children = children_table.get(parent_id).unwrap();
    children.into_iter().map(|child_id| {
        let child_id = child_id.unwrap().value();
        let child = files_table.get(child_id).unwrap().unwrap();
        child.value().hash()
    })
}

struct Queues<'a> {
    creates: QueuedCreatesTableMut<'a>,
    updates: QueuedUpdatesTableMut<'a>,
    deletes: QueuedDeletesTableMut<'a>,
}