use std::hash::{Hash, Hasher};

use chrono::{DateTime, Utc};
use compact_str::CompactString;
use derive_more::{Deref, From};
use rustc_hash::FxHasher;
use sqlx::{Decode, Encode, FromRow, Sqlite, prelude::Type};
use crate::database::FileId;

#[derive(Debug, Clone, FromRow, Default)]
pub struct File {
    pub id: FileId,
    pub name: CompactString,
    pub parent_id: FileId,
    pub hash: i32,
    pub modified_at: DateTime<Utc>,
}

impl File {
    pub fn new(
        name: impl Into<CompactString>,
        parent_id: FileId,
        timestamp: DateTime<Utc>,
        child_hashes: impl Iterator<Item = i32>,
    ) -> Self {
        let name = name.into();
        let id = parent_id.child(&name);
        let mut self_ = Self {
            name,
            parent_id,
            id,
            modified_at: timestamp,
            hash: i32::default(),
        };

        self_.update(child_hashes, timestamp);
        self_
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
