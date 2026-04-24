use std::{path::Path, sync::Arc};
use serde::{Deserialize, Serialize};

use crate::{config::ROOT_NAME, db::tables::file::FileName, path::RelPath};



/// Never ends with a '/'
#[derive(Debug, Clone, Default,Serialize,Deserialize)]
pub struct AbsPath {
    inner: Arc<str>,
}
impl AbsPath {
    pub fn new(path: impl AsRef<str>) -> Self {
        Self {
            inner: path.as_ref().trim_end_matches('/').into(),
        }
    }

    pub fn child(&self, file_name: &str) -> Self {
        let new_path = format!("{self}/{file_name}");
        Self::new(new_path)
    }

    pub fn join(&self, path: &RelPath) -> Self {
        let excess = &path.as_ref()[ROOT_NAME.len() + 1..];
        let new_path = format!("{self}/{excess}");
        Self::new(new_path)
    }

    pub fn file_name(&self) -> Option<&FileName> {
        for (i,c) in self.inner.char_indices().rev() {
            if c == '/' {
                return FileName::from(&self.inner[i + 1..]);
            }
        }
        FileName::from(&self.inner)
    }
    
    
    pub fn from_os_path(path: impl AsRef<Path> ) -> Self {
        let path = path.as_ref();
        debug_assert!(path.is_absolute());
        Self::new(path.to_string_lossy())
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }
    
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn as_rel(&self,root:&Self) -> Option<RelPath> {
        let s = self.as_str().strip_prefix(root.as_str())?;
        Some(RelPath::from(s))
    }
  
}

impl AsRef<str> for AbsPath {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl AsRef<Path> for AbsPath {
    fn as_ref(&self) -> &Path {
        Path::new(self.as_str())
    }
}


impl From<&Path> for AbsPath {
    fn from(path: &Path) -> Self {
        debug_assert!(path.is_absolute());
        AbsPath::new(path.to_string_lossy())
    }
}

impl std::fmt::Display for AbsPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}
