use redb::{MultimapTable, MultimapTableDefinition, ReadOnlyMultimapTable, ReadOnlyTable, Table, TableDefinition};
use crate::db::tables::{file::{File, FileId}, queued_deltas::QueuedDelete};


pub mod timestamp;
pub mod file;
#[macro_use]
pub mod macros;
pub mod queued_deltas;

pub const FILES: TableDefinition<FileId, &File> = TableDefinition::new("files");
pub type FilesTable<'a> = ReadOnlyTable<FileId, &'a File>;
pub type FilesTableMut<'a> = Table<'a,FileId, &'a File>;

/// Stores only immediate children of a file.
pub const CHILDREN: MultimapTableDefinition<FileId, FileId> =
    MultimapTableDefinition::new("children");
pub type ChildrenTable = ReadOnlyMultimapTable<FileId, FileId>;
pub type ChildrenTableMut<'a> = MultimapTable<'a, FileId, FileId>;


pub const QUEUED_CREATES: TableDefinition<FileId,()> = TableDefinition::new("queued_creates");
pub type QueuedCreatesTable = ReadOnlyTable<FileId, ()>;
pub type QueuedCreatesTableMut<'a> = Table<'a, FileId, ()>;

pub const QUEUED_UPDATES: TableDefinition<FileId,()> = TableDefinition::new("queued_updates");
pub type QueuedUpdatesTable = ReadOnlyTable<FileId, ()>;
pub type QueuedUpdatesTableMut<'a> = Table<'a, FileId, ()>;

pub const QUEUED_DELETES: TableDefinition<FileId,QueuedDelete> = TableDefinition::new("queued_deletes");
pub type QueuedDeletesTable = ReadOnlyTable<FileId, QueuedDelete>;
pub type QueuedDeletesTableMut<'a> = Table<'a, FileId, QueuedDelete>;