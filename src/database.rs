use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::state::file_id::FileId;

pub mod file;
pub mod file_id;
pub mod local_reader;
pub mod local_writer;
pub mod remote_writer;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueuedUpdate {
    pub file_id: FileId,
    pub depth: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedDeletion {
    pub file_id: FileId,
    pub deleted_at: DateTime<Utc>,
}
