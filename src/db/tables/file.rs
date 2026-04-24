use std::{
    hash::{Hash, Hasher}, iter
};

use crate::{
    db::tables::{file::info::FileInfo, timestamp::Timestamp},
    state::file::{id::FileId, name::FileName},
};

use bytemuck::{Pod, Zeroable};
use derive_more::Deref;
use rustc_hash::FxHasher;
use serde::{Deserialize, Serialize};
mod bytes;
pub mod info;
mod id;
mod name;
mod name_buf;
pub use id::FileId;
pub use id::FileIdOrd;
pub use name_buf::FileNameBuf;
pub use name::FileName;

mod child_id;
#[derive(Debug, Copy, Pod, Zeroable, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct File {
    name: FileNameBuf,
    modified_at:Timestamp,
    parent_id: FileId,
    hash: i32,
    depth: u16,
    is_dir_u16: u16,
}

impl File {
    pub const SIZE: usize = std::mem::size_of::<File>();

    pub fn empty(id:FileIdOrd, info:FileInfo,now:Timestamp) -> Self {
        let hash = Self::calculate_hash(*id, now,iter::empty());
        Self {
            name: info.name,
            parent_id: info.parent_id,
            modified_at: now,
            hash,
            depth: id.depth,
            is_dir_u16: u16::from(info.is_dir),
        }
    }
   

    pub fn calculate_hash(
        id: FileId,
        timestamp: Timestamp,
        child_hashes: impl Iterator<Item = i32>,
    ) -> i32 {
        let mut state = FxHasher::default();
        id.hash(&mut state);
        timestamp.hash(&mut state);
        for child_hash in child_hashes {
            child_hash.hash(&mut state);
        }

        let hash_64 = state.finish().to_ne_bytes();
        let hash_32 = hash_64[..4].try_into().unwrap();
        i32::from_ne_bytes(hash_32)
    }
    
    pub fn is_dir(&self) -> bool {
        self.is_dir_u16 == 1
    }
    
    pub fn file_name(&self) -> &FileNameBuf {
        &self.name
    }
    
    pub fn parent_id(&self) -> FileId {
        self.parent_id
    }
    
    pub fn modified_at(&self) -> Timestamp {
        self.modified_at
    }
    
    pub fn hash(&self) -> i32 {
        self.hash
    }
    
    pub fn depth(&self) -> u16 {
        self.depth
    }
    
    pub fn modify(self, id: FileId, timestamp: Timestamp,child_hashes: impl Iterator<Item = i32>) -> Self {
        Self {
            modified_at:timestamp,
            hash: Self::calculate_hash(id, timestamp, child_hashes),
            ..self
        }
    }
    
}
#[derive(Debug, Deref)]
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
