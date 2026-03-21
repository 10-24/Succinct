use std::{cmp::Ordering, ffi::{OsStr, OsString}};

use anyhow::bail;
use compact_str::CompactString;
use inotify::{Event, EventMask};

use crate::path::{AbsPath, Path};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Delta {
    pub path: Path,
    pub kind: DeltaKind,
    pub depth: u16,
}

impl Ord for Delta {
    fn cmp(&self, other: &Self) -> Ordering {
        self.depth
            .cmp(&other.depth)
            .then_with(|| self.path.as_ref().cmp(other.path.as_ref()))
    }
}

impl PartialOrd for Delta {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Delta {
    pub fn from(event: Event<OsString>,root: &AbsPath) -> Self {
      
        let path = Path::new_relative(event.name.unwrap().to_str().unwrap(), root).unwrap();
        let depth = path.depth() as u16;
        let kind = event.mask.into();
        Self {
            depth,
            path,
            kind,
        }
    }
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
