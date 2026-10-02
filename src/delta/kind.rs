use inotify::{EventMask, WatchMask};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum::VariantArray;

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive, Hash, VariantArray)]
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

    pub fn all_inotify_masks() -> WatchMask {
        Self::VARIANTS.into_iter().copied().map(Self::as_inotify_mask).reduce(WatchMask::union).unwrap()
 
    }
}