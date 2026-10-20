use std::{collections::{BTreeMap, VecDeque}, default, hash::Hash, time::SystemTime};
use inotify::WatchDescriptor;
use rustc_hash::FxHashMap;

use crate::{notify::EventKind, util::arc_path::ArcPath};

#[derive(Default)]
pub struct Descriptors {
    forwards: FxHashMap<WatchDescriptor,Entry>,
    backwards: FxHashMap<ArcPath,WatchDescriptor>
}

impl Descriptors {
    pub fn new() -> Self {
        Self {
            forwards: FxHashMap::default(),
            backwards: FxHashMap::default(),
        }
    }

    pub fn insert(&mut self, desc: WatchDescriptor, path: ArcPath) {
        self.forwards.insert(desc.to_owned(), Entry {
            path: path.clone(),
            ignored_events: Vec::new()
        });
        self.backwards.insert(path,desc);
    }

    pub fn get_mut(&mut self, key: &WatchDescriptor) -> Option<&mut Entry> {
        self.forwards.get_mut(key)
    }

    pub fn extract_ignored_event(entry: &mut Entry) -> bool {
        let now = SystemTime::now();
        entry.ignored_events.retain(|entry| entry.expiration <= now);
        
        for i in 0..entry.ignored_events.len() {
            if 
        }
        
    }
}



struct Entry {
    path: ArcPath,
    ignored_events: Vec<IgnoredEvent>,
}


pub enum IgnoredEventKind {
    MoveTo,
    Delete,
}

struct IgnoredEvent {
    kind: IgnoredEventKind,
    expiration: SystemTime,
}

