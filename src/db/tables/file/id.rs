use std::hash::{Hash, Hasher};

use bytemuck::{Pod, Zeroable};
use derive_more::{Deref, From};
use redb_derive::{Key, Value};
use rustc_hash::FxHasher;
use serde::{Deserialize, Serialize};

use crate::{
    config::{ROOT_ID, ROOT_NAME},
    db::tables::file::FileName,
    path::RelPath,
    state::file::name::FileName,
};
use redb::Value;

#[derive(
    Clone,
    Copy,
    From,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Debug,
    Serialize,
    Deserialize,
    Key,
    Value,
    Pod,
    Zeroable,
)]
#[repr(transparent)]
pub struct FileId(pub(crate) u64);

impl FileId {
   

    pub fn is_root(&self) -> bool {
        *self == *ROOT_ID
    }

    pub fn into_ord(self, depth: u16) -> FileIdOrd {
        FileIdOrd { depth, value: self }
    }

    pub const fn constant(id: &'static u64) -> Self {
        Self(*id)
    }
    
    pub fn extend<T: AsRef<FileName>>(self, mut components: impl Iterator<Item = T>) -> Self {
        let Some(name) = components.next() else {
            return self;
        };
        self.child(name.as_ref()).extend(components)
    }
}

impl Into<usize> for &FileId {
    fn into(self) -> usize {
        let int_64 = self.0.to_ne_bytes();
        usize::from_ne_bytes(int_64)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deref, Serialize, Deserialize, Key, Value,
)]
/// Can be used like a typical FileId, but has an additional depth field for ordering.
pub struct FileIdOrd {
    pub depth: u16, // 1
    #[deref]
    pub value: FileId, // 2
}

impl FileIdOrd {
    pub fn child(&self, name: &FileName) -> Self {
        let child_id = self.value.child(name);
        Self {
            depth: self.depth + 1,
            value: child_id,
        }
    }

 
}
