use std::{iter, ops::Not, path::{Path, PathBuf}, str, time};

use anyhow::{anyhow, ensure};
use chrono::{DateTime, Utc};
use num_enum::{IntoPrimitive, TryFromPrimitive};


pub mod kind;
use crate::{delta::kind::DeltaKind, util::{fhasher::FHasher, relpath::RelPath}};


#[derive(Hash, Debug)]
pub struct Delta {
    pub kind: DeltaKind,
    pub path: Box<Path>,
    pub is_dir: bool,
}

impl Delta {


    pub fn to_bytes(&self) -> Box<[u8]> {
        let path = self.path.as_bytes();
        let mut v = Vec::with_capacity(1 + path.len());
        v.push(self.kind as u8);
        v.extend_from_slice(path);
        v.into_boxed_slice()
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {

        let kind = bytes.get(0).ok_or_else(|| anyhow!("Delta bytes is empty"))?;
        let kind = DeltaKind::try_from(*kind)?;
        
        let path = str::from_utf8(&bytes[1..])?;
        let path = RelPath::from_str_unchecked(path);

        Ok(Self {
            kind,
            path
        })
    }
}





#[derive(Debug, Clone, Copy)]
pub struct DeltaId(u64);

impl DeltaId {

    const ENTROPY_BITS: u32 = 16;
    const TIMESTAMP_MASK: u64 = u64::MAX << Self::ENTROPY_BITS;
    const ENTROPY_MASK: u64 = !Self::TIMESTAMP_MASK;
    
    pub fn new(delta: &Delta, timestamp: &DateTime<Utc>) -> Self {
        let timestamp = timestamp.timestamp_millis() as u64;
        let entropy = FHasher::new().hash(timestamp).hash(delta).finish();

        let timestamp_reigon = timestamp << Self::ENTROPY_BITS;
        let entropy_reigon = entropy & Self::ENTROPY_MASK;
        
        let id = timestamp_reigon | entropy_reigon;
        Self(id)
    }

    pub fn created_at(self) -> DateTime<Utc> {
        let timestamp_reigon = self.0 & Self::TIMESTAMP_MASK;
        let timestamp = timestamp_reigon >> Self::ENTROPY_BITS;
        DateTime::<Utc>::from_timestamp_millis(timestamp as i64).unwrap()
    }
}