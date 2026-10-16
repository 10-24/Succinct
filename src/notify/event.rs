use std::ffi::OsString;

use chrono::{DateTime, Utc};

use crate::{notify::event_kind::EventKind, util::arc_path::ArcPath};

pub type Event = (ArcPath, EventData);

#[derive(Debug, Clone, Copy)]
pub struct EventData {
    pub kind: EventKind,
    pub timestamp: DateTime<Utc>,
    pub is_dir: bool,
    pub is_move_to: bool,
}

impl EventData {
    pub fn from_inotify_event(event: &inotify::Event<OsString>, timestamp: DateTime<Utc>) -> Option<Self> {
        Some(Self {
            timestamp,
            kind: EventKind::from(event.mask)?,
            is_dir: EventKind::matches_dir(event.mask),
            is_move_to: EventKind::matches_moved_to(event.mask)
        })
    }
}