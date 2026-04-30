use std::ffi::{OsStr};

use bytemuck::{TransparentWrapper};
use crate::{db::tables::file::name_buf::FileNameBuf};


#[derive(Debug)]
#[repr(transparent)]
pub struct FileName(str);
unsafe impl TransparentWrapper<str> for FileName {}

impl FileName {
    
    pub fn from(s: &str) -> Option<&Self> {
        FileName::is_valid(s).then(|| FileName::from_unchecked(s))
    }
    
    pub fn from_os_str(s: &OsStr) -> Option<&Self> {
        Self::from(s.to_str().unwrap())
    }
    
    pub(super) fn from_unchecked(s: &str) -> &Self {
        FileName::wrap_ref(s)
    }
    
    pub fn as_str(&self) -> &str {
        FileName::peel_ref(self)
    }
    
    pub const fn is_valid(s: &str) -> bool {
        (0 < s.len()) && (s.len() <= FileNameBuf::MAX_LEN)
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
    
    pub fn len(&self) -> usize {
        self.as_str().len()
    }
}

impl AsRef<FileName> for FileName {
    fn as_ref(&self) -> &FileName {
        self
    }
}

impl AsRef<str> for FileName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}


impl ToOwned for FileName {
    type Owned = FileNameBuf;
    fn to_owned(&self) -> Self::Owned {
        FileNameBuf::from_unchecked(self.as_str())
    }
}

impl std::fmt::Display for &FileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Into<String> for &FileName {
    fn into(self) -> String {
        self.as_str().to_string()
    }
}