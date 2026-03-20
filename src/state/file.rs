use std::{
    hash::{Hash, Hasher},
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, Utc};
use compact_str::CompactString;
use rustc_hash::FxHasher;
use sqlx::FromRow;

use crate::path::Path;

#[derive(Debug, Clone, FromRow, Default)]
pub struct File {
    pub id: i64,
    pub name: CompactString,
    pub parent_id: i64,
    pub hash: i32,
    pub modified_at: DateTime<Utc>,
}

impl File {
    pub fn new(
        path: &Path,
        timestamp: DateTime<Utc>,
        child_hashes: impl Iterator<Item = i32>,
    ) -> Self {
        let mut self_ = Self {
            id: File::get_id(path),
            parent_id: path
                .parent()
                .map(|parent| File::get_id(&parent))
                .unwrap_or_default(),
            name: path
                .last()
                .map(|l| CompactString::from(l))
                .unwrap_or_default(),
            
            modified_at: DateTime::default(),
            hash: i32::default(),
        };

        self_.update(child_hashes, timestamp);
        self_
    }

    pub fn get_id(path: &Path) -> i64 {
        let mut hasher = FxHasher::default();
        path.hash(&mut hasher);
        let hash_64 = hasher.finish().to_ne_bytes();
        i64::from_ne_bytes(hash_64)
    }

    pub fn update(&mut self, child_hashes: impl Iterator<Item = i32>, timestamp: DateTime<Utc>) {
        self.modified_at = timestamp;

        let mut state = FxHasher::default();
        for child_hash in child_hashes {
            child_hash.hash(&mut state);
        }
        self.id.hash(&mut state);
        self.modified_at.hash(&mut state);

        let hash_64 = state.finish().to_ne_bytes();
        let hash_32 = hash_64[..4].try_into().unwrap();
        self.hash = i32::from_ne_bytes(hash_32);
    }
}

impl Hash for File {
    fn hash<F: Hasher>(&self, state: &mut F) {
        self.id.hash(state);
        self.name.hash(state);
        self.modified_at.hash(state);
    }
}

