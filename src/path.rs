use std::ops::Deref;
use std::sync::Arc;

use std::hash::{Hash, Hasher};

use compact_str::{CompactString, format_compact};
use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Path {
    full: Arc<str>,
    length: u16,
}

/// Never starts or ends with a '/'
impl Path {
    pub fn new_relative(abs_path: &str, root: &AbsPath) -> Option<Self> {
        
        let rel_path = Path::as_relative(abs_path, root)?;
    
        Some(Self {
            length: rel_path.len() as u16,
            full: Arc::from(rel_path),
        })
    }

    pub fn as_absolute(&self, root: &str) -> CompactString {
        format_compact!("{}/{}", root, self.as_ref())
    }

    pub fn parent(&self) -> Option<Self> {
        if self.is_root() {
            return None;
        }

        let mut length = self.length;
        for c in self.as_ref().chars().rev() {
            if c == '/' {
                break;
            }
            length -= 1;
        }
        Some(Self {
            full: self.full.clone(),
            length,
        })
    }

    /// from this parent to root
    pub fn ancestors(&self) -> impl Iterator<Item = Self> {
        let mut current = self.parent();
        std::iter::from_fn(move || {
            let val = current.clone()?;
            current = val.parent();
            Some(val)
        })
    }

    pub fn ancestor_depths(&self) -> impl Iterator<Item = (usize, Self)> {
        let mut current_depth = self.depth();
        self.ancestors().map(move |path| {
            current_depth -= 1;
            (current_depth, path)
        })
    }

    pub fn len(&self) -> usize {
        self.length as usize
    }

    pub fn depth(&self) -> usize {
        self.as_ref().chars().filter(|c| *c == '/').count()
    }

    pub fn is_root(&self) -> bool {
        self.as_ref().len() == 0
    }

    /// Returns the file or folder name
    pub fn last(&self) -> &str {
        for (i, c) in self.as_ref().char_indices().rev() {
            if c != '/' {
                continue;
            }
            return &self.as_ref()[i + 1..];
        }
        self.as_ref()
    }

    pub fn from(full: impl Into<Arc<str>>) -> Self {
        let full = full.into();
        Self {
            length: full.len() as u16,
            full,
        }
    }
    
    pub fn as_relative<'a>(mut path: &'a str, root: &AbsPath) -> Option<&'a str> {
        if !path.starts_with(root.as_ref()) {
            return None;
        }
        path = &path[root.len()+2..].trim_matches('/');
        
        Some(path)
    }
}

impl AsRef<str> for Path {
    fn as_ref(&self) -> &str {
        &self.full[..self.len()]
    }
}

impl Hash for Path {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Deref for Path {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Eq for Path {}

/// Never ends with a '/'
#[derive(Debug, Clone, Deref, Serialize, Deserialize, Default)]
pub struct AbsPath(Arc<str>);
impl AbsPath {
    pub fn new(mut path: &str) -> Self {
        path = path.trim_end_matches("/");
        let path = Arc::from(path);
        Self(path)
    }

    pub fn join(&self, path: &Path) -> Self {
        if path.is_root() {
            return self.clone();
        }
        let new_path = format!("{}/{}", self.as_ref(), path.as_ref());
        Self::new(&new_path)
    }

    pub fn as_relative(&self, root: &Self) -> &str {
        &self.0[root.len() + 1..]
    }
}
impl AsRef<str> for AbsPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
