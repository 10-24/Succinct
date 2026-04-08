use std::{ops::Deref, path::{Path, PathBuf}};

use ignore::gitignore::Gitignore;
use serde::{Deserialize, Serialize};

use crate::{config::INTERNAL_ROOT_NAME, path::RelPath};

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
        let new_path = format!("{}/{}", self.as_ref(), file_name);
        Self::new(new_path)
    }

    pub fn join(&self, path: &RelPath) -> Self {
        let excess = &path.as_ref()[INTERNAL_ROOT_NAME.len() + 1..];
        let new_path = format!("{}/{}", self.as_ref(), excess);
        Self::new(new_path)
    }

    pub fn file_name(&self) -> &str {
        self.inner.split('/').last().unwrap()
    }
    
    pub fn is_ignored(&self, is_dir: bool, ignore: &Gitignore,) -> bool {
        ignore.matched(self.as_ref(), is_dir).is_ignore()
    }

}

impl<K: PathKind> AsRef<str> for AbsPath<K> {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}
impl<K: PathKind> Deref for AbsPath<K> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
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
