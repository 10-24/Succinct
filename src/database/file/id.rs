use std::hash::{Hash, Hasher};

use bytemuck::{Pod, Zeroable};
use derive_more::{Deref, From};
use redb_derive::{Key, Value};
use rustc_hash::FxHasher;
use serde::{Deserialize, Serialize};

use crate::{config::ROOT_ID, state::file::name::FileName};
use redb::Value;

#[derive(
    Clone, Copy, From, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug, Serialize, Deserialize, Key,Value,Pod,Zeroable
)]
#[repr(transparent)]
pub struct FileId(pub i64);

impl FileId {
    pub fn child(&self, name: &FileName) -> Self {
        let mut hasher = FxHasher::default();
        name.as_str().hash(&mut hasher);

        let hash_64: [u8; 8] = hasher.finish().to_ne_bytes();
        let int_64 = i64::from_ne_bytes(hash_64);
        Self(int_64)
    }

    pub fn is_root(&self) -> bool {
        *self == *ROOT_ID
    }
    
    pub fn into_ord(self, depth: u16) -> FileIdOrd {
        FileIdOrd { depth, value: self }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deref, Serialize, Deserialize)]
/// Can be used like a typical FileId, but has an additional depth field for ordering.
pub struct FileIdOrd {
    pub depth: u16, // Must be first
    #[deref]
    pub value: FileId,
}

impl FileIdOrd {
    pub fn child(&self, name: &str) -> Self {
        let child_id = self.value.child(name);
        Self {
            depth: self.depth + 1,
            value: child_id,
        }
    }
}
