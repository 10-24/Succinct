use std::array;

use inotify::EventMask;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum::VariantArray;

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive, Hash, VariantArray, PartialEq, Eq)]
#[repr(u8)]
pub enum EventKind {
    Create,
    Update,
    Delete,
}


impl EventKind {

    pub fn from(mask: inotify::EventMask) -> Option<Self> {
        Self::VARIANTS.iter().copied().find(|variant| variant.matches(mask))
    }

    
    pub const fn as_event_mask(self) -> inotify::EventMask {
        match self {
            EventKind::Create => {
                inotify::EventMask::CREATE.union(
                    inotify::EventMask::MOVED_TO
                )
            }
            EventKind::Update => {
                inotify::EventMask::MODIFY
                .union(
                    inotify::EventMask::CLOSE_WRITE
                )                
            }
            EventKind::Delete => {
                inotify::EventMask::DELETE.union(
                    inotify::EventMask::MOVED_FROM
                )
            },
        }
    }
    
    /// Only watches directories
    pub const WATCH_MASKS: inotify::WatchMask = inotify::WatchMask::CREATE.union(
            inotify::WatchMask::MOVED_TO
        ).union(
            inotify::WatchMask::MODIFY
        ).union(
            inotify::WatchMask::CLOSE_WRITE
        ).union(
            inotify::WatchMask::DELETE,
        ).union(
            inotify::WatchMask::MOVED_FROM,
        );
    

    pub fn is_create(&self) -> bool {
        Self::Create == *self
    }

    pub fn is_delete(&self) -> bool {
        Self::Delete == *self
    }

    pub fn matches(&self, mask: inotify::EventMask) -> bool {
        mask.contains(self.as_event_mask())
    }

    pub fn matches_dir(mask: inotify::EventMask) -> bool {
        mask.contains(inotify::EventMask::ISDIR)
    }

    pub fn matches_moved_to(mask: inotify::EventMask) -> bool {
        mask.contains(inotify::EventMask::MOVED_TO)
    }
}

