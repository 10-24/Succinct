use std::{collections::LinkedList, sync::Arc};

use derive_more::Deref;
use redb::{AccessGuard, ReadableDatabase, ReadableTable, ReadableTableMetadata};
use rustc_hash::FxHashSet;
use tokio::sync::Mutex;

use tables::{CHILDREN, ChildrenTable, FILES, FilesTable, QUEUED_DELETES, QueuedDeletion};
use writer::DbWriter;

use crate::{
     db::tables::macros::OpenReadTable, hashset, path::RelPath, state::file::{
        File,
        bytes::FileBytes,
        id::{FileId, FileIdOrd},
        name::FileName,
     }

};

#[macro_use]
pub mod tables;
pub mod writer;

#[derive(Debug, Clone)]
pub struct Db {
    conn: Arc<redb::Database>,
}

impl Db {
    pub async fn init(database: redb::Database) -> Self {
        let db = Self {
            conn: Arc::new(database),
        };
        let writer = db.begin_write().await;
        writer.ensure_tables();
        writer.commit();
        db
    }

    pub async fn begin_write(&self) -> DbWriter {
        let txn = self.conn.begin_write().unwrap();
        DbWriter { txn }
    }
    
    /// Prefer using the [`tables!`] macro
    pub fn begin_read(&self) -> redb::ReadTransaction {
        self.conn.begin_read().unwrap()
    }

    pub fn get_file_child_hashes(&self, parent_id: FileId) -> Vec<i32> {
        let (files,children) = tables!(self,FILES,CHILDREN);

        children
            .get(&parent_id)
            .unwrap()
            .map(|child_id| {
                let child_id = child_id.unwrap().value();
                let child = files.get(&child_id).unwrap().unwrap().value();
                child.hash
            })
            .collect()
    }


    /// Excludes parent
    pub fn get_file_descendants(&self, parent_id: FileIdOrd) -> Vec<FileIdOrd> {
        fn descendants(
            parent_id: FileIdOrd,
            children_table: &ChildrenTable,
            output: &mut Vec<FileIdOrd>,
        ) {
            let mut children = children_table.get(*parent_id).unwrap();
            while let Some(Ok(child_id)) = children.next() {
                let child_id = child_id.value().into_ord(parent_id.depth + 1);
                output.push(child_id);
                descendants(child_id, children_table, output);
            }
        }
        
        let children_table = tables!(self,CHILDREN);
        let mut output = Vec::new();
        descendants(parent_id, &children_table, &mut output);
        output
    }

    fn all_ids(&self) -> FxHashSet<FileId> {
        let files = tables!(self,FILES);

        let mut ids = hashset(files.len().unwrap() as usize);
        
        for Ok((id, _)) in files.iter().unwrap() {
            ids.insert(id.value());
        }
        ids
    }

    pub fn get_file_path(&self, file_id: FileId) -> RelPath {
        fn path_components<'a>(
            file_id: FileId,
            files: &'a FilesTable,
            components: &mut Vec<&'a FileName>,
        ) {
            let file = files.get(file_id).unwrap().unwrap().value();
            components.push(&file.value().name);
            if !file_id.is_root() {
                path_components(file.parent_id, &files, components);
            }
        }
        let files = tables!(self,FILES);
        let mut components_rev = Vec::with_capacity(8);
        path_components(file_id, &files, &mut components_rev);

        RelPath::from_components(components_rev.into_iter().rev()).unwrap()
    }

    /// Returns ancestors starting from root
    pub fn get_file_ancestors(&self, file_id: FileId) -> impl Iterator<Item = FileIdOrd> {
        fn add_ancestors(
            file_id: FileId,
            files: &FilesTable,
            ancestors: &mut Vec<FileIdOrd>,
        ) {
            let file = files.get(file_id).unwrap().unwrap().value();
            let parent_id = file.parent_id.into_ord(file.depth - 1);
            ancestors.push(parent_id);
            if !file_id.is_root() {
                add_ancestors(*parent_id, &files, ancestors);
            }
        }

        let files = tables!(self,FILES);
        let mut ancestors = Vec::with_capacity(8);
        add_ancestors(file_id, &files, &mut ancestors);
        ancestors.into_iter().rev()
    }
}
