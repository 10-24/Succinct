use std::{collections::LinkedList, sync::Arc};

use derive_more::Deref;
use redb::{AccessGuard, ReadableDatabase, ReadableTable, ReadableTableMetadata};
use rustc_hash::FxHashSet;
use tables::{CHILDREN, ChildrenTable, FILES, FilesTable};
use writer::DbWriter;

use crate::{
     config::ROOT_NAME, db::tables::{file::{File, FileId, FileIdOrd, FileName, FileNameBuf}, macros::OpenReadTable}, hashset, path::RelPath

};

#[macro_use]
pub mod tables;
pub mod writer;
pub mod remote;

#[derive(Debug, Clone)]
pub struct Db {
    conn: Arc<redb::Database>,
}

impl Db {
    pub async fn init(database: redb::Database) -> Self {
        let db = Self {
            conn: Arc::new(database),
        };
        let writer = db.begin_write();
        writer.ensure_tables();
        writer.commit();
        db
    }

    pub fn begin_write(&self) -> DbWriter {
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
            children_tbl: &ChildrenTable,
            mut result: Vec<FileIdOrd>,
        ) -> Vec<FileIdOrd> {
            let mut children = children_tbl.get(*parent_id).unwrap();
            while let Some(child_id) = children.next() {
                let child_id = child_id.unwrap().value().into_ord(parent_id.depth + 1);
                result = vec![..result, child_id];
                result = descendants(child_id, children_tbl, result);
            }
            result
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

    pub fn get_file_path(id: FileId, files: &FilesTable) -> RelPath {
        fn comp<'a>(
            id: FileId,
            files: &FilesTable,
            components: Vec<FileNameBuf>,
        ) -> Vec<FileNameBuf> {
            if id == FileId::ROOT {
                return vec![..components,ROOT_NAME];
            }
            let f:&File = files.get(id).unwrap().unwrap().value();
            let new_components = vec![..components, *f.name()];
            comp(f.parent_id(), &files, new_components)
        }
   
        let components_rev = comp(id, &files,  vec![_;8]);
        let components = components_rev.into_iter().rev();
        RelPath::from_components(components)
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
    
    pub fn contains_file(&self, file_id: FileId) -> bool {
        let entry = tables!(self,FILES).get(file_id).unwrap();
        entry.is_some()
    }
    
    pub fn is_dir(&self,file_id: FileId) -> Option<bool> {
        let files = tables!(self,FILES);
        
        let file = files.get(&file_id).unwrap()?;
        Some(file.value().is_dir)
    }
}
