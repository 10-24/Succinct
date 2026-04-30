use futures::future::try_join_all;
use redb::{ReadableTable, ReadableTableMetadata};

use crate::{db::{Db, tables::{CHILDREN, ChildrenTableMut, FILES, FilesTable, FilesTableMut, QUEUED_UPDATES, QueuedDeletesTableMut, QueuedUpdatesTable, QueuedUpdatesTableMut, file::{File, FileId}, queued_deltas::QueuedDelete}}, delta::DeltaKind, state::{remote_drive::save::{DriveErr, UpdatePath}, state::State}, tables, tables_mut};

impl State {
    pub async fn push_remote(&self) {
        let local_writer = self.local_db.begin_write();
        let queued_deltas = tables_mut!(&local_writer,QUEUED_UPDATES);
        let local_files = tables!(self.local_db,FILES);
        let remote_writer = self.remote_db.begin_write();
        let (remote_files,remote_children) = tables_mut!(&remote_writer, FILES, CHILDREN);
        let tables = PushTables {
            local_files,
            remote_files,
            remote_children,
        };
        
        if tables.queued_updates.len().unwrap() > 0 {
            self.sync_updates(&tables);
            self.push_updates(&mut tables);
        } 
        if tables.queued_deletes.len().unwrap() > 0 {
            
            self.push_deletes(&mut tables);
        }
        remote_writer.commit();
        local_writer.commit();
    }

    async fn sync_updates<'a>(&self, tables: &PushTables<'a>) -> Result<(),DriveErr>  {
        let updated_paths = tables.queued_updates.iter().unwrap().map(|entry| {
            let id = entry.unwrap().0.value();
            let rel = Db::get_file_path(id, &tables.local_files);
            let abs = self.local_root.join(&rel);
            UpdatePath { id, rel, abs }
        });
        self.drive.upload_all(updated_paths).await?;
        Ok(())
    }
    
    async fn push_updates<'a>(&self, tables: &mut PushTables<'a>) {
        for entry in tables.queued_updates.iter().unwrap() {
            let id = entry.unwrap().0.value();
            let file:&File = tables.local_files.get(id).unwrap().unwrap().value();
            if file.is_dir() {
                continue;
            }
            tables.remote_files.insert(id, file).unwrap();
            tables.remote_children.insert(file.parent_id(), id).unwrap();
        }
        tables.queued_updates.retain(|_,_| false).unwrap(); // clear all
    }
    
    async fn sync_deletes<'a>(&self,tables: &PushTables<'a>) -> Result<(),DriveErr> {
        let deleted_files = tables.queued_deletes.iter().unwrap().map(|e| e.unwrap().0.value());
        try_join_all(deleted_files.map(async |id|{
            let rel_path = Db::get_file_path(id, &tables.local_files);
            self.drive.delete(&rel_path).await
        }))
   
    }

    fn push_deletes<'a>(&self, tables: &mut PushTables<'a>) {
        tables.queued_deletes
    }
}


struct PushTables<'a> {
    local_files:FilesTable<'a>,
    remote_files: FilesTableMut<'a>,
    remote_children: ChildrenTableMut<'a>,
    queued_updates: QueuedUpdatesTableMut<'a>,
    queued_deletes: QueuedDeletesTableMut<'a>,
}
