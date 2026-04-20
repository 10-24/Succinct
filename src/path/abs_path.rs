use std::{ops::Deref, path::{Path}};

use ignore::gitignore::Gitignore;
use serde::{Deserialize, Serialize};

use crate::{config::ROOT_NAME, path::RelPath};

#[derive(Debug, Clone, Copy, Default)]
pub struct Local;
#[derive(Debug, Clone, Copy, Default)]
pub struct Remote;
pub trait PathKind: Clone {}
impl PathKind for Local {}
impl PathKind for Remote {}

/// Never ends with a '/'
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AbsPath<K: PathKind> {
    inner: Box<str>,
    _marker: std::marker::PhantomData<K>,
}
impl<K: PathKind> AbsPath<K> {
    pub fn new(path: impl AsRef<str>) -> Self {
        Self {
            inner: path.as_ref().trim_end_matches('/').into(),
            _marker: std::marker::PhantomData,
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

    pub fn file_name(&self) -> &str {
        for (i,c) in self.inner.char_indices().rev() {
            if c == '/' {
                return &self.inner[i + 1..];
            }
        }
        &self.inner
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
    
    pub fn starts_with(&self, other: &AbsPath<K>) -> bool {
        self.as_str().starts_with(other.as_str())
    }
}

impl<K: PathKind> AsRef<str> for AbsPath<K> {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl<K: PathKind> AsRef<Path> for AbsPath<K> {
    fn as_ref(&self) -> &Path {
        Path::new(self.as_str())
    }
}


impl From<&Path> for AbsPath<Local> {
    fn from(path: &Path) -> Self {
        debug_assert!(path.is_absolute());
        AbsPath::new(path.to_string_lossy())
    }
}

impl<T:PathKind> std::fmt::Display for AbsPath<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
