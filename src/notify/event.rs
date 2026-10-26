use std::{collections::BTreeSet, ffi::OsString, ops::{Range, RangeInclusive}, time::SystemTime};

use chrono::{DateTime, Duration, Utc};

use crate::{notify::event_kind::{EventKind, IgnoredEventKind}, util::arc_path::ArcPath};

pub type Event = (ArcPath, EventData);

#[derive(Debug, Clone, Copy)]
pub struct EventData {
    pub kind: EventKind,
    pub timestamp: DateTime<Utc>,
    pub is_dir: bool,
}

impl EventData {
    pub fn from_inotify_event(event: &inotify::Event<OsString>, timestamp: DateTime<Utc>) -> Option<Self> {
        Some(Self {
            timestamp,
            kind: EventKind::from(event.mask)?,
            is_dir: event.mask.contains(inotify::EventMask::ISDIR)
        })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct IgnoredEvent {
    pub kind: IgnoredEventKind,
    pub path: ArcPath,
    pub expiration: SystemTime,
}

impl IgnoredEvent {
    pub fn new(path: ArcPath, kind: IgnoredEventKind, expiration: SystemTime) -> Self {
        Self { path, kind, expiration }
    }   
}

