use compact_str::CompactString;
use inotify::{ EventMask};

use crate::state::file_id::FileId;

#[derive(Debug,Clone)]
pub struct Delta {
    pub file_name: CompactString,
    pub kind: DeltaKind,
    pub depth: u16,
    pub ord: u16,
    pub parent_id: FileId,
}


#[derive(Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Copy, Clone)]
pub enum DeltaKind {
    Update,
    Delete,
}

impl Into<DeltaKind> for EventMask {
    fn into(self) -> DeltaKind {
        if self.contains(EventMask::DELETE) || self.contains(EventMask::MOVED_FROM) {
            return DeltaKind::Delete
        } 
        DeltaKind::Update
        
    }
}
