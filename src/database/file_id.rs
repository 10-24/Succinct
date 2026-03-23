use std::hash::{Hash, Hasher};

use derive_more::{Deref, From};
use rustc_hash::FxHasher;
use sqlx::{Decode, Encode, FromRow, Sqlite, prelude::Type};

use crate::config::ROOT_ID;

#[derive(Clone, Copy, Type, From, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[sqlx(transparent)]
#[derive(Debug)]
pub struct FileId(pub i64);

impl FileId {
    pub fn child(&self, name: &str) -> Self {
        let mut hasher = FxHasher::default();
        name.hash(&mut hasher);

        let hash_64: [u8; 8] = hasher.finish().to_ne_bytes();
        let int_64 = i64::from_ne_bytes(hash_64);
        Self(int_64)
    }

    pub fn is_root(&self) -> bool {
        self == &ROOT_ID
    }
}

impl Into<usize> for &FileId {
    fn into(self) -> usize {
        let int_64 = self.0.to_ne_bytes();
        usize::from_ne_bytes(int_64)
    }
}

fn as_i64(n: usize) -> i64 {
    let hash_64: [u8; 8] = n.to_ne_bytes();
    i64::from_ne_bytes(hash_64)
}

#[derive(Debug,PartialEq, Eq, PartialOrd, Ord,Deref)]
/// Can be used like a typical FileId, but has an additional depth field for ordering.
pub struct FileIdOrd {
    pub depth: u16, // Must be first
    #[deref]
    pub file_id: FileId,
}