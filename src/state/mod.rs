pub use crate::database::file;
pub use crate::database::file_id;
use crate::{FxBuildHasher, FxHashMap};
mod push;
mod remote_drive;
mod state;
mod prepare_deltas;

pub trait WithCapacity {
    fn with_capacity(cap: usize) -> Self;
}

impl<K, V> WithCapacity for FxHashMap<K, V> {
    fn with_capacity(cap: usize) -> Self {
        Self::with_capacity_and_hasher(cap, FxBuildHasher::default())
    }
}
