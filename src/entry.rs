use std::{hash::{Hasher,Hash}, time::{SystemTime, UNIX_EPOCH}};

use chrono::{DateTime, Utc};
use compact_str::CompactString;
use rustc_hash::FxHasher;
use sqlx::{FromRow};

use crate::path::Path;

#[derive(Debug, Clone, FromRow)]
pub struct Entry {
    pub id: i64,
    pub name: CompactString,
    pub hash: i32,
    pub modified_at: DateTime<Utc>,
    pub parent_id: i64,
}

impl Entry {
    pub fn new(path:&Path,timestamp:DateTime<Utc>, mut hasher: FxHasher) -> Self {
        
        let mut self_ = Self {
            id: Entry::get_id(path),
            name: path.as_ref().to_string().into(),
            modified_at: timestamp,
            parent_id: path.parent().map(|parent| Entry::get_id(&parent)).unwrap_or_default(),
            hash: 0,
        };
        self_.hash(&mut hasher);
        self_.hash = finish(hasher) as i32;
        self_
    }
    
    pub fn get_id(path:&Path) -> i64 {
        let mut hasher = FxHasher::default();
        &path.hash(&mut hasher);
        finish(hasher)
    }
}

impl Hash for Entry {
    fn hash<F: Hasher>(&self, state: &mut F) {
        self.id.hash(state);
        self.name.hash(state);
        self.modified_at.hash(state);
    }
}



pub fn finish(hasher: FxHasher) -> i64 {
    let hash_64 = hasher.finish().to_ne_bytes();
    i64::from_ne_bytes(hash_64)
}