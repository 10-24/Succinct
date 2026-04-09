use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Pod, Zeroable, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(C)]
pub struct FileName {
    pub len: u8,
    pub bytes: [u8; FileName::MAX_LEN],
}

impl FileName {
    pub const MAX_LEN: usize = 31;
    
    pub fn from(s: impl AsRef<str>) -> Self {
        let s = s.as_ref();
        assert!(s.len() <= Self::MAX_LEN);
        let mut bytes = [0u8; Self::MAX_LEN];
        bytes[..s.len()].copy_from_slice(s.as_bytes());
        Self { len: s.len() as u8, bytes }
    }

    pub fn as_str(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(&self.bytes[..self.len as usize])
        }
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
