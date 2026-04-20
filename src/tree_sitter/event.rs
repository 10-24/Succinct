use std::ffi::OsString;

use derive_more::Deref;
use inotify::{EventMask};
use rustc_hash::FxHashMap;

use crate::{delta::DeltaKind,};



#[derive(Debug)]
pub struct Event {
    pub name: Option<OsString>,
    pub kind: DeltaKind,
    pub is_dir: bool,
}

pub type WatchDescriptor = i32;

#[derive(Debug,Deref)]
pub struct EventKV {
    pub descriptor: WatchDescriptor,
    #[deref]
    pub value: Event,
}

impl Into<EventKV> for (WatchDescriptor, Event) {
    fn into(self) -> EventKV {
        EventKV {
            descriptor: self.0,
            value: self.1,
        }
    }
}

impl EventKV {
    pub fn from(e: inotify::Event<OsString>) -> Self {
        let descriptor = e.wd.get_watch_descriptor_id();

        let delta_kind = DeltaKind::from_event_mask(e.mask);
        let is_dir = e.mask.contains(EventMask::ISDIR);
        let value = Event {
            name:e.name,
            kind: delta_kind,
            is_dir,
        };
        Self { descriptor, value }
    }
}

