use chrono::{DateTime, Utc};

use crate::state::file::FileId;

pub mod local_reader;
pub mod local_writer;
pub mod remote_writer;
pub mod file;
pub mod file_id;


#[derive(Debug)]
pub struct QueuedUpdate {
    pub file_id: FileId,
    pub depth: u16,
}

#[derive(Debug, Clone)]
pub struct QueuedDeletion {
    pub file_id: FileId,
    pub deleted_at: DateTime<Utc>,
}
