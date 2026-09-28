use inotify::EventMask;

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive, Hash)]
#[repr(u8)]
pub enum DeltaKind {
    Modify,
    Delete,
}


impl DeltaKind {
    pub const fn as_inotify_mask(self) -> inotify::WatchMask {
        match self {
            DeltaKind::Modify => {
                inotify::WatchMask::CREATE.union(
                    inotify::WatchMask::MOVED_TO
                ).union(
                    inotify::WatchMask::MODIFY
                ).union(
                    inotify::WatchMask::CLOSE_WRITE
                )                
            }
            DeltaKind::Delete => {
                inotify::WatchMask::DELETE.union(
                    inotify::WatchMask::MOVED_FROM
                )
            },
        }
    }
}