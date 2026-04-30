use redb_derive::{Key, Value};
use redb::Value;

use crate::{db::tables::{file::FileId, timestamp::Timestamp}};

#[derive(Debug,PartialEq, Eq, PartialOrd, Ord,Key,Value)]
pub struct QueuedDelete {
    parent_id: FileId,
    timestamp: Timestamp,
}