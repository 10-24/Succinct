use std::{collections::BTreeMap, ops::Not};

use compact_str::CompactString;
use inotify::{ EventMask, WatchMask};

use crate::state::file_id::{FileId, FileIdOrd};

pub type Deltas = BTreeMap<FileIdOrd,Delta>;
#[derive(Debug,Clone)]
pub struct Delta {
    pub file: FileRecord,
    pub kind: DeltaKind,
}


#[derive(Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Copy, Clone)]
pub enum DeltaKind {
    Create,
    Update,
    Delete,
}

impl DeltaKind {
    pub const WATCH_MASK: WatchMask = Self::CREATE_MASKS.union(Self::UPDATE_MASKS).union(Self::DELETE_MASKS);
        
    const CREATE_MASKS: WatchMask = WatchMask::CREATE.union(WatchMask::MOVED_TO);
    const UPDATE_MASKS: WatchMask = WatchMask::MODIFY;
    const DELETE_MASKS: WatchMask = WatchMask::DELETE.union(WatchMask::MOVED_FROM);
    
    pub fn from_event_mask(mask:EventMask) -> DeltaKind {
        
        let is_create = has_any(&mask, &Self::CREATE_MASKS);
        let is_update = has_any(&mask, &Self::UPDATE_MASKS);
        let is_delete = has_any(&mask, &Self::DELETE_MASKS);
        
        match (is_create, is_update, is_delete) {
            (true, _, _) => DeltaKind::Create,
            (_, true, _) => DeltaKind::Update,
            (_, _, true) => DeltaKind::Delete,
            _ => unreachable!()
        }
    }
}

fn has_any(event: &EventMask, target: &WatchMask) -> bool {
    let target = target.bits();
    let event = event.bits();
    (target & event) != 0
}

#[derive(Debug,Clone)]
pub struct FileRecord {
    pub name: CompactString,
    pub parent_id: FileId
}