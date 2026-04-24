use crate::db::tables::CHILDREN;
use crate::db::tables::ChildrenTableMut;
use crate::db::tables::FILES;
use crate::db::tables::FilesTableMut;
use crate::db::tables::file::File;
use crate::db::tables::file::FileId;
use crate::db::tables::file::FileIdOrd;
use crate::db::tables::timestamp::Timestamp;
use crate::state::file_id::FileId;

use crate::state::file_id::FileIdOrd;
use crate::state::prepare_deltas::DeltaKV;
use crate::state::prepare_deltas::DeltaValue;
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
    pub(crate) remote_drive: opendal::Operator,
    pub(crate) local_root: AbsPath,
}

impl State {
    pub fn new(
        local_db: Db,
        remote_drive: opendal::Operator,
        local_root: AbsPath,
    ) -> Self {
        Self {
            local_db,
            remote_drive,
            local_root,
        }
    }

    /// File and Folder creations cannot be implicit (Eg. If a parent and child file/folder are created, both must be included)
    pub async fn push_deltas(&mut self, deltas: BTreeMap<FileIdOrd, Delta>) {
        let txn = self.local_db.begin_write();
        let (files_table, children_table) = tables_mut!(&txn, FILES, CHILDREN);
        let now = Timestamp::now();
        
        let mut complete_deltas = self.instantiate_files_and_complete_tree(deltas, now, &mut files_table, &mut children_table);
        while let Some((id,delta)) = complete_deltas.pop_last() {
            match delta.kind {
                DeltaKind::Create => self.handle_modify(id,delta, now,&mut files_table,&children_table),
                DeltaKind::Update => self.handle_modify(id,delta, now,&mut files_table,&children_table),
                DeltaKind::Delete => self.handle_delete(id,&mut files_table,&mut children_table),
            }
        }
    }

    /// Handles both Update and Create
    fn handle_modify(&self, id:FileIdOrd, delta:DeltaValue, now:Timestamp, files_table: &mut FilesTableMut, children_table: &ChildrenTableMut) {

        let mut file_entry = files_table.get_mut(*id).unwrap().unwrap();
        let prev_file:&File = file_entry.value();
        let updated_file = if prev_file.is_dir() {
            let child_hashes = get_child_hashes(*id, files_table, children_table);
            prev_file.modify(*id, now, child_hashes)
        } else {
            prev_file.modify(*id, now, None)
        };
        file_entry.insert(updated_file);
    }
    
    

    fn handle_delete(&self, id:FileIdOrd,files_table: &mut FilesTableMut, children_table: &mut ChildrenTableMut) {
        // let file_id = delta.0.value;

        // let descendants = self.local_db.get_file_descendants(file_id);
        // let subtree = descendants.into_iter().chain(Some(delta.0));

        // for file_id in subtree {
        //     self.local_writer.delete_file(*file_id);

        //     let queued_deletion = QueuedDeletion {
        //         deleted_at,
        //         file_id: *file_id,
        //     };
        //     self.local_writer.enqueue_delete(queued_deletion);
        // }
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

