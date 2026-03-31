use std::hash::{Hash, Hasher};

use crate::database::FileId;
use chrono::{DateTime, Utc};
use compact_str::CompactString;
use derive_more::{Deref, From};
use rustc_hash::FxHasher;
use sqlx::{Decode, Encode, FromRow, Sqlite, prelude::Type};

#[derive(Debug, Clone, FromRow, Default)]
pub struct File {
    pub id: FileId,
    pub name: CompactString,
    pub parent_id: FileId,
    pub hash: i32,
    pub modified_at: DateTime<Utc>,
    pub depth: u16,
    pub created_at: DateTime<Utc>,
}

impl File {
    pub fn empty(
        name: impl Into<CompactString>,
        depth: u16,
        timestamp: DateTime<Utc>,
        parent_id: FileId,
    ) -> Self {
        Self::new(name, depth, timestamp, None.into_iter(), parent_id)
    }
    pub fn new(
        name: impl Into<CompactString>,
        depth: u16,
        timestamp: DateTime<Utc>,
        child_hashes: impl Iterator<Item = i32>,
        parent_id: FileId,
    ) -> Self {
        let name = name.into();
        let id = parent_id.child(&name);
        let hash = File::calculate_hash(id, timestamp, child_hashes);
        Self {
            id,
            name,
            parent_id,
            depth,
            modified_at: timestamp,
            created_at: timestamp,
            hash,
        }
    }

    pub fn update(&mut self, timestamp: DateTime<Utc>,child_hashes: impl Iterator<Item = i32>,) {
        self.modified_at = timestamp;
        self.hash = Self::calculate_hash(self.id,self.modified_at,child_hashes) 
    }
    
   
    pub fn calculate_hash(id: FileId, modified_at: DateTime<Utc>, child_hashes: impl Iterator<Item = i32>) -> i32 {
        
        let mut state = FxHasher::default();
        id.hash(&mut state);
        modified_at.hash(&mut state);
        for child_hash in child_hashes {
            child_hash.hash(&mut state);
        }
        
        let hash_64 = state.finish().to_ne_bytes();
        let hash_32 = hash_64[..4].try_into().unwrap();
        i32::from_ne_bytes(hash_32)
    }
    pub fn is_new(&self) -> bool {
        self.created_at == self.modified_at
    }
}

impl Hash for File {
    fn hash<F: Hasher>(&self, state: &mut F) {
        self.id.hash(state);
        self.name.hash(state);
        self.modified_at.hash(state);
    }
}

#[derive(Debug, Clone, Default)]
pub struct FileData {
    pub name: CompactString,
    pub parent_id: FileId,
    pub depth: u16,
}
