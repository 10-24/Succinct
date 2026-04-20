use std::{hash::Hash, path::Path};

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::path::{AbsPath, Local};

#[derive(Debug, Clone, Copy, Pod, Zeroable, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct FileName {
    length: u8,
    bytes: [u8; FileName::MAX_LEN],
}

impl FileName {
    pub const MAX_LEN: usize = 39;

    pub fn from(s: impl AsRef<str>) -> Option<Self> {
        if !Self::is_valid(s.as_ref()) {
            return None;
        }
        Some(Self::from_unchecked(s))
    }

    pub fn from_abs_path(path: AbsPath<Local>) -> Option<Self> {
        Self::from(path.file_name())
    }

    pub fn from_unchecked(s: impl AsRef<str>) -> Self {
        let s = s.as_ref();
        let mut bytes = [0u8; Self::MAX_LEN];
        bytes[..s.len()].copy_from_slice(s.as_bytes());
        Self {
            length: s.len() as u8,
            bytes,
        }
    }

    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.bytes[..self.len()]) }
    }

    pub const fn constant(s: &'static str) -> Self {
        assert!(Self::is_valid(s), "Invalid file name");

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

    pub const fn is_valid(s: &str) -> bool {
        (0 < s.len()) && (s.len() <= Self::MAX_LEN)
    }
    
    pub fn len(&self) -> usize {
        self.length as usize
    }
}

impl AsRef<FileName> for FileName {
    fn as_ref(&self) -> &FileName {
        &self
    }
}

impl AsRef<str> for FileName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Hash for FileName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl Serialize for FileName {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for FileName {
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

impl std::fmt::Display for FileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
