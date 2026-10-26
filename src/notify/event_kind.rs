use derive_more::Deref;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum::VariantArray;

#[derive(Debug, Clone, Copy, Hash, VariantArray, PartialEq, Eq, PartialOrd, Ord )]
pub enum EventKind {
    Create,
    MovedTo,

    Modify,
    CloseWrite,

    Delete,
    MovedFrom,
}

impl EventKind {
    pub fn from(mask: inotify::EventMask) -> Option<Self> {
        Self::VARIANTS
            .iter()
            .copied()
            .find(|variant| variant.matches(mask))
    }

    pub const fn as_event_mask(self) -> inotify::EventMask {
        match self {
            EventKind::Create => inotify::EventMask::CREATE,
            EventKind::MovedTo => inotify::EventMask::MOVED_TO,
            EventKind::Modify => inotify::EventMask::MODIFY,
            EventKind::CloseWrite => inotify::EventMask::CLOSE_WRITE,
            EventKind::Delete => inotify::EventMask::DELETE,
            EventKind::MovedFrom => inotify::EventMask::MOVED_FROM,
        }
    }

    /// Only watches directories
    pub const WATCH_MASKS: inotify::WatchMask = inotify::WatchMask::CREATE
        .union(inotify::WatchMask::MOVED_TO)
        .union(inotify::WatchMask::MODIFY)
        .union(inotify::WatchMask::CLOSE_WRITE)
        .union(inotify::WatchMask::DELETE)
        .union(inotify::WatchMask::MOVED_FROM);

    pub fn matches(&self, mask: inotify::EventMask) -> bool {
        mask.contains(self.as_event_mask())
    }

    pub fn delta(&self) -> DeltaKind {
        match self {
            Self::Create | Self::MovedTo => DeltaKind::Create,
            EventKind::Modify | EventKind::CloseWrite => DeltaKind::Update,
            Self::Delete | Self::MovedFrom => DeltaKind::Delete,
        }
    }

 
}

impl TryFrom<inotify::EventMask> for EventKind {
    type Error = ();

    fn try_from(mask: inotify::EventMask) -> Result<Self, Self::Error> {
        Self::VARIANTS
            .iter()
            .copied()
            .find(|variant| variant.matches(mask))
            .ok_or(())
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IgnoredEventKind {
    MovedTo,
    Delete,
}

impl TryFrom<EventKind> for IgnoredEventKind {
    type Error = ();

    fn try_from(event_kind: EventKind) -> Result<Self, Self::Error> {
        match event_kind {
            EventKind::MovedTo => Ok(Self::MovedTo),
            EventKind::Delete => Ok(Self::Delete),
            _ => Err(()),
        }
    }
}



#[derive(Debug, Clone, Copy, Hash, VariantArray)]
pub enum DeltaKind {
    Create,
    Update,
    Delete,
}

impl DeltaKind {
    pub fn is_create(&self) -> bool {
        matches!(self, Self::Create)
    }
    pub fn is_update(&self) -> bool {
        matches!(self, Self::Update)
    }
    pub fn is_delete(&self) -> bool {
        matches!(self, Self::Delete)
    }
}



