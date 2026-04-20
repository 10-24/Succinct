use crate::db::tables::file::{FileId, FileName};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FileInfo {
    pub name: FileName,
    pub parent_id: FileId,
    pub is_dir: bool,
}

