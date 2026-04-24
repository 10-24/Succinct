use crate::db::tables::file::{FileId, name_buf::FileNameBuf};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FileInfo {
    pub name: FileNameBuf,
    pub parent_id: FileId,
    pub is_dir: bool,
}

