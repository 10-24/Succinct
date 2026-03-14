use std::cmp::Ordering;

use compact_str::CompactString;

use crate::path::Path;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Delta {
    pub path: Path,
    pub kind: DeltaKind,
    pub depth: u16,
}

impl Ord for Delta {
    fn cmp(&self, other: &Self) -> Ordering {
        self.depth
            .cmp(&other.depth)
            .then(self.kind.cmp(&other.kind))
            .then_with(|| self.path.as_ref().cmp(other.path.as_ref()))
    }
}

impl PartialOrd for Delta {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Copy, Clone)]
pub enum DeltaKind {
    Update,
    Delete,
}

