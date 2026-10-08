use std::{path::Path, sync::Arc};

use chrono::{DateTime, Utc};

use crate::notify::event_kind::EventKind;

pub type Event = (Arc<Path>, EventData);

pub struct EventData {
    pub timestamp: DateTime<Utc>,
    pub kind: EventKind,
    pub is_dir: bool,
}