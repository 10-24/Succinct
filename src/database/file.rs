use std::hash::{Hash, Hasher};

use crate::{database::FileId, state::file::{id::FileId, name::FileName}};

use bytemuck::{Pod, Zeroable};
use chrono::{DateTime, Utc};
use derive_more::Deref;
use rustc_hash::FxHasher;
use serde::{Deserialize, Serialize};

pub mod id;
pub mod name;
pub mod bytes;
#[derive(Debug,Copy, Pod, Zeroable, PartialEq, Eq, Clone, Serialize,Deserialize)]
#[repr(C)]
pub struct File {
    pub name: FileName,
    pub parent_id: FileId,
    pub modified_at: i64,
    pub hash: i32,
    pub depth: u16,
}

impl File {
    pub const SIZE:usize = std::mem::size_of::<File>();
    
    pub fn empty(
        name: FileName,
        depth: u16,
        timestamp: DateTime<Utc>,
        parent_id: FileId,
    ) -> Self {
        Self::new(name, depth, timestamp, None.into_iter(), parent_id)
    }
    pub fn new(
        name: FileName,
        depth: u16,
        timestamp: DateTime<Utc>,
        child_hashes: impl Iterator<Item = i32>,
        parent_id: FileId,
    ) -> Self {
        let id = parent_id.child(&name);
        let hash = File::calculate_hash(id, timestamp, child_hashes);
        Self {
            name,
            parent_id,
            depth,
            modified_at: timestamp.timestamp(),
            hash,
        }
    }


    pub fn calculate_hash(
        id: FileId,
        modified_at: DateTime<Utc>,
        child_hashes: impl Iterator<Item = i32>,
    ) -> i32 {
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
    
}
#[derive(Debug,Deref)]
pub struct FileKV {
    pub id: FileId,
    #[deref]
    pub file: File,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileData {
    pub name: String,
    pub parent_id: FileId,
    pub depth: u16,
}

