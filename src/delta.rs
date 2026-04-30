use std::{collections::BTreeMap};

use inotify::{ EventMask, WatchMask};


use crate::{db::tables::file::{FileIdOrd, info::FileInfo}, state::file::{id::{FileId, FileIdOrd}, name::FileName}};

pub type Deltas = BTreeMap<FileIdOrd,Delta>;

#[derive(Debug,Clone, Copy)]
pub struct Delta {
    pub data: DeltaData,
    pub index: u16,
}

impl Delta {
    pub fn new_create(index:u16,info:FileInfo,) -> Self {
        Delta {
            index,
            data: DeltaData::Create(info)
        }
    }
    pub fn new_update(index:u16) -> Self {
        Delta {
            index,
            data: DeltaData::Update
        }
    }
    pub fn new_delete(index:u16) -> Self {
        Delta {
            index,
            data: DeltaData::Delete
        }
    }
}
#[derive(Debug,Clone, Copy)]
#[repr(u8)]
pub enum DeltaData {
    Create(FileInfo),
    Update,
    Delete,
}

impl DeltaData {
    pub fn kind(&self) -> DeltaKind {
        match self {
            DeltaData::Create(_) => DeltaKind::Create,
            DeltaData::Update => DeltaKind::Update,
            DeltaData::Delete => DeltaKind::Delete,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Copy, Clone,FromPrimitive)]
pub enum DeltaKind {
    /// index: 0
    Create,
    /// index: 1
    Update,
    /// index: 2
    Delete,
}

impl DeltaKind {
    pub const WATCH_MASK: WatchMask = Self::CREATE_MASKS.union(Self::UPDATE_MASKS).union(Self::DELETE_MASKS);
        
    const CREATE_MASKS: WatchMask = WatchMask::CREATE.union(WatchMask::MOVED_TO);
    const UPDATE_MASKS: WatchMask = WatchMask::CLOSE_WRITE;
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

impl From<u8> for DeltaKind {
    fn from(value: u8) -> Self {
        match value {
            0 => DeltaKind::Create,
            1 => DeltaKind::Update,
            2 => DeltaKind::Delete,
            _ => unreachable!(),
        }
    }
}

impl From<DeltaKind> for u8 {
    fn from(value: DeltaKind) -> Self {
        match value {
            DeltaKind::Create => 0,
            DeltaKind::Update => 1,
            DeltaKind::Delete => 2,
        }
    }
}



#[derive(Debug,Clone)]
pub struct FileRecord {
    pub name: FileName,
    pub parent_id: FileId
}

pub type DeltaSender = mpsc::Sender<Deltas>;
pub type DeltaReceiver = mpsc::Receiver<Deltas>;

pub type DeltaKV = (FileIdOrd, Delta);