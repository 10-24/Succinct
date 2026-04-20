use redb::{MultimapTableDefinition, ReadOnlyMultimapTable, ReadOnlyTable, TableDefinition};
use crate::db::tables::file::{File, FileId, FileIdOrd};


pub mod timestamp;
pub mod file;
#[macro_use]
pub mod macros;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueuedUpdate {
    pub file_id: FileId,
    pub depth: FileIdOrd,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueuedDeletion {
    pub deleted_at: chrono::DateTime<chrono::Utc>,
    pub file_id: FileId,
}

pub const FILES: TableDefinition<FileId, &File> = TableDefinition::new("files");
pub type FilesTable<'a> = ReadOnlyTable<FileId, &'a File>;

/// Stores only immediate children of a file.
pub const CHILDREN: MultimapTableDefinition<FileId, FileId> =
    MultimapTableDefinition::new("children");
pub type ChildrenTable = ReadOnlyMultimapTable<FileId, FileId>;

pub const QUEUED_UPDATES: TableDefinition<FileIdOrd, ()> = TableDefinition::new("queued_updates");
pub const QUEUED_DELETES: TableDefinition<FileIdOrd, ()> = TableDefinition::new("queued_deletes");
