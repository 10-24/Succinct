use std::ops::Deref;

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::db::tables::file::FileName;

#[derive(Debug, Clone, Copy, Pod, Zeroable, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct FileNameBuf {
    length: u8,
    bytes: [u8; FileNameBuf::MAX_LEN],
}

impl FileNameBuf {
    pub const MAX_LEN: usize = 31;

    pub fn from(s: impl AsRef<str>) -> Option<Self> {
        if !FileName::is_valid(s.as_ref()) {
            return None;
        }
        Some(Self::from_unchecked(s))
    }

    pub fn from_os_str(s: impl AsRef<std::ffi::OsStr>) -> Option<Self> {
        let s = s.as_ref().to_str()?;
        Self::from(s)
    }
    
    pub(super) fn from_unchecked(s: impl AsRef<str>) -> Self {
        let s = s.as_ref();
        let mut bytes = [0u8; Self::MAX_LEN];
        bytes[..s.len()].copy_from_slice(s.as_bytes());
        Self {
            length: s.len() as u8,
            bytes,
        }
    }
    
    pub const fn constant(s: &'static str) -> Self {
        assert!(FileName::is_valid(s), "Invalid file name");

        let mut bytes = [0u8; Self::MAX_LEN];
        let s = s.as_bytes();
        let mut i = 0;
        while i < s.len() {
            bytes[i] = s[i];
            i += 1;
        }
        Self {
            length: s.len() as u8,
            bytes,
        }
    } 
}



impl AsRef<FileName> for FileNameBuf {
    fn as_ref(&self) -> &FileName {
        FileName::from_unchecked(self.as_str())
    }
}

impl AsRef<str> for FileNameBuf {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}



impl Serialize for FileNameBuf {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for FileNameBuf {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from(&s).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "file name too long: {} bytes (max {})",
                s.len(),
                Self::MAX_LEN
            ))
        })
    }
}

impl std::borrow::Borrow<FileName> for FileNameBuf {
    fn borrow(&self) -> &FileName {
        self.as_ref()
    }
}

impl Deref for FileNameBuf {
    type Target = FileName;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}