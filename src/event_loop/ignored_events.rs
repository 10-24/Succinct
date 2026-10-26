use std::{
    collections::BTreeSet, ops::Range, sync::Arc, time::{Duration, SystemTime}
};

use camino::Utf8Path;
use globset::{Glob, GlobSet, GlobSetBuilder};
use log::warn;

use crate::{notify::{event::{Event, IgnoredEvent}, event_kind::IgnoredEventKind}, util::arc_path::ArcPath};



pub struct IgnoredEvents(BTreeSet<IgnoredEvent>);

impl IgnoredEvents {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    pub fn filter(&mut self, events: impl Iterator<Item = Event>) -> impl Iterator<Item = Event> {
        events.filter(|event| self.remove_matching(event).is_none())
    }

    fn remove_matching(&mut self, event: &Event) -> Option<IgnoredEvent> {
        let (path, event) = event;
        let Ok(ignore_kind) = IgnoredEventKind::try_from(event.kind) else {
            return None;
        };

       
        let timestamp = SystemTime::from(event.timestamp);
        self.0
            .extract_if(Self::matching_range(path.clone(), ignore_kind), |_| true)
            .find(|candidate| candidate.expiration >= timestamp)
    }
  
    pub fn ignore_event(&mut self, event: IgnoredEvent) {
        self.0.insert(event);
    }

    fn matching_range(path: ArcPath, kind: IgnoredEventKind) -> Range<IgnoredEvent> {
        let start_time = SystemTime::UNIX_EPOCH;
        let start = IgnoredEvent::new(path.clone(), kind, start_time);

        let end_time = SystemTime::UNIX_EPOCH + Duration::from_secs(i64::MAX as u64);
        let end = IgnoredEvent::new(path, kind, end_time);
        start..end
    }

    pub fn union(mut self, mut other: Self) -> Self {
        if self.len() >= other.len() {
            self.0.extend(other.0);
            self
        } else {
            other.0.extend(self.0);
            other
        }
    }


    pub fn len(&self) -> usize {
        self.0.len()
    }
}