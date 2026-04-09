use std::{collections::LinkedList, sync::Arc};

use derive_more::Deref;
use redb::{
    AccessGuard, MultimapTableDefinition, ReadOnlyMultimapTable, ReadOnlyTable, ReadTransaction, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition
};
use rustc_hash::FxHashSet;
use tokio::sync::Mutex;

use crate::{
    config::{INTERNAL_ROOT_NAME, ROOT_ID, ROOT_PARENT_ID}, database::{QueuedDeletion, local_writer::DbWriter}, hashset, path::RelPath, state::file::{File, bytes::FileBytes, id::{FileId, FileIdOrd}, name::FileName}, tree_sitter::tree_sitter::FileKV
};

pub const FILES: TableDefinition<FileId, &FileBytes> = TableDefinition::new("files");
pub type FilesTable<'a> = ReadOnlyTable<FileId, &'a FileBytes>;
pub const CHILDREN: MultimapTableDefinition<FileId, FileId> = MultimapTableDefinition::new("children");
pub type ChildrenTable = ReadOnlyMultimapTable<FileId, FileId>;
pub const QUEUED_UPDATES: TableDefinition<FileId, ()> = TableDefinition::new("queued_updates");
pub const QUEUED_DELETES: TableDefinition<FileId, Vec<u8>> = TableDefinition::new("queued_deletes");

#[derive(Debug, Clone)]
pub struct Db {
    database: Arc<redb::Database>,
    write_lock: Arc<Mutex<()>>
}

impl Db {
    pub async fn init(database: redb::Database) -> Self {
        let db = Self { database: Arc::new(database), write_lock: Arc::default() };
        let writer = db.begin_write().await;
        writer.ensure_tables();
        writer.commit();
        db
    }
    
    pub async fn begin_write(&self) -> DbWriter {
        let guard = self.write_lock.lock().await;
        let txn = self.database.begin_write().unwrap();
        DbWriter { txn, guard }
    }
    
    pub fn db(&self) -> &Arc<redb::Database> {
        &self.database
    }

    pub fn get_file_child_hashes(&self, parent_id: FileId) -> Vec<i32> {
        let txn = self.database.begin_read().unwrap();
        let children_table = txn.open_multimap_table(CHILDREN).unwrap();
        let files_table = txn.open_table(FILES).unwrap();

        children_table
            .get(&parent_id)
            .unwrap()
            .map(|guard| {
                let child_id = guard.unwrap().value();
                let bytes = files_table.get(&child_id).unwrap().unwrap();
                bytes.value().hash
            })
            .collect()
    }
    pub fn get_file_ref<'a>(id: FileId,table: &'a FilesTable,) -> Option<&'a File> {
        let bytes = table.get(id).unwrap()?.value();
        Some(File::from_bytes(bytes))
    }
    
    pub fn get_file_owned(&self, id: FileId) -> Option<File> {
        let txn = self.database.begin_read().unwrap();
        let table = txn.open_table(FILES).unwrap();
    
        Self::get_file_ref(id, &table).map(|f| f.to_owned())
    }

    /// Excludes parent
    pub fn get_file_descendants(&self, parent_id: FileIdOrd) -> Vec<FileIdOrd> {
        
        fn descendants(parent_id: FileIdOrd, children_table: &ChildrenTable, output: &mut Vec<FileIdOrd>) {
            let mut children = children_table.get(*parent_id).unwrap();
            while let Some(Ok(child_id)) = children.next(){
                let child_id = child_id.value().into_ord(parent_id.depth + 1);
                output.push(child_id);
                descendants(child_id, children_table, output);
            }
        }
        
        let txn = self.database.begin_read().unwrap();
        let children_table = txn.open_multimap_table(CHILDREN).unwrap();
        let mut output = Vec::new();
        descendants(parent_id, &children_table, &mut output);
        output
    }

    fn all_ids(&self) -> FxHashSet<FileId> {
        let txn = self.database.begin_read().unwrap();
        let files = txn.open_table(FILES).unwrap();
        
        let mut ids = hashset(files.len().unwrap() as usize);
        while let Some(Ok((id,_))) = files.iter().unwrap().next() {
            ids.insert(id.value());
        }
        ids
    }

    pub fn get_update_queue(&self) -> ReadGuard<impl Iterator<Item = (FileId, &File)>> {
        let txn = self.database.begin_read().unwrap();
        let updates_table = txn.open_table(QUEUED_UPDATES).unwrap();
        let files_table = txn.open_table(FILES).unwrap();
        
        let files = updates_table
            .iter()
            .unwrap()
            .map(|entry| {
                let (id, _value) = entry.unwrap();
                let id = id.value();
                let file = Self::get_file_ref(id, &files_table).unwrap();
                (id,file)
            });
        ReadGuard::new(files, txn)
    }

    pub fn get_delete_queue(&self) -> Vec<QueuedDeletion> {
        let txn = self.database.begin_read().unwrap();
        let table = txn.open_table(QUEUED_DELETES).unwrap();

        table
            .iter()
            .unwrap()
            .map(|entry| {
                let (_, bytes) = entry.unwrap();
                bincode::deserialize(&bytes.value()).unwrap()
            })
            .collect()
    }

    pub fn get_file_path(&self, file_id: FileId) -> RelPath {
        fn path_components<'a>(file_id:FileId, files_table: &'a FilesTable, components: &mut Vec<&'a FileName>){
            let file = Db::get_file_ref(file_id, &files_table).unwrap();
            components.push(&file.name);
            if !file_id.is_root() {
                path_components(file.parent_id, &files_table, components);
            }
        }
        let txn = self.database.begin_read().unwrap();
        
        let files_table = txn.open_table(FILES).unwrap();
        let mut components_rev = Vec::with_capacity(8);
        path_components(file_id, &files_table, &mut components_rev);
        
        
        RelPath::from_components(components_rev.into_iter().rev()).unwrap()
    }

    /// Returns ancestors starting from root
    pub fn get_file_ancestors(&self, file_id: FileId) -> impl Iterator<Item = FileIdOrd> {
        
        fn add_ancestors(file_id:FileId, files_table: &FilesTable, ancestors: &mut Vec<FileIdOrd>){
            let file = Db::get_file_ref(file_id, &files_table).unwrap();
            let parent_id = file.parent_id.into_ord(file.depth-1);
            ancestors.push(parent_id);
            if !file_id.is_root() {
                add_ancestors(*parent_id, &files_table, ancestors);
            }
        }
        
        let txn = self.database.begin_read().unwrap();
        
        let files_table = txn.open_table(FILES).unwrap();
        let mut ancestors = Vec::with_capacity(8);
        add_ancestors(file_id, &files_table, &mut ancestors);
        ancestors.into_iter().rev()
    }


}

