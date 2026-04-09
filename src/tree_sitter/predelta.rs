use std::ffi::OsString;

use inotify::{Event, EventMask};
use rustc_hash::FxHashMap;

use crate::{delta::DeltaKind,};



#[derive(Debug)]
pub struct Predelta {
    pub name: Option<String>,
    pub kind: DeltaKind,
    pub is_dir: bool,
}

pub type WatchDescriptor = i32;

#[derive(Debug)]
pub struct PredeltaKV {
    pub descriptor: WatchDescriptor,
    pub value: Predelta,
}

impl Into<PredeltaKV> for (WatchDescriptor, Predelta) {
    fn into(self) -> PredeltaKV {
        PredeltaKV {
            descriptor: self.0,
            value: self.1,
        }
    }
}

impl PredeltaKV {
    pub fn from_event(e: Event<OsString>) -> Self {
        let descriptor = e.wd.get_watch_descriptor_id();
        let name = e.name.map(|name| name.to_str().unwrap().into());
        let delta_kind = DeltaKind::from_event_mask(e.mask);
        let is_dir = e.mask.contains(EventMask::ISDIR);
        let value = Predelta {
            name,
            kind: delta_kind,
            is_dir,
        };
        Self { descriptor, value }
    }
}

