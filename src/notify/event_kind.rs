use inotify::EventMask;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum::VariantArray;

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive, Hash, VariantArray)]
#[repr(u8)]
pub enum EventKind {
    Modify,
    Delete,
}


impl EventKind {
    pub const fn as_event_mask(self) -> inotify::EventMask {
        match self {
            EventKind::Modify => {
                inotify::EventMask::CREATE.union(
                    inotify::EventMask::MOVED_TO
                ).union(
                    inotify::EventMask::MODIFY
                ).union(
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
    

    
}