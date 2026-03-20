use std::ops::Deref;
use std::rc::Rc;

use std::hash::{Hash, Hasher};

use compact_str::{CompactString, format_compact};
use derive_more::{Deref, From, Into};


#[derive(Debug,Clone,Eq,PartialEq)]
pub struct Path {
    full: Rc<str>,
    length: u16,
}

/// Never starts or ends with a '/'
impl Path {
    
    pub fn new_relative(abs_path: &str, root: &AbsPath) -> Option<Self> {
        if !abs_path.starts_with(root.as_ref()) {
            return None
        }
        let rel_path = abs_path[root.len()+2..].trim_matches('/');
        
        Some(Self{
            length: rel_path.len() as u16,
            full: Rc::from(rel_path),
        })
    }
    
    pub fn as_absolute(&self, root: &str) -> CompactString {
        format_compact!("{}/{}", root, self.as_ref())
    }
    
    pub fn parent(&self) -> Option<Self> {
        for (i,c) in self.as_ref().char_indices().rev() {
            if c != '/' {
              continue; 
            }
            return Some(Self {
                full: self.full.clone(),
                length: (i + 1) as u16,
            })
        }
        None
    }
    
    /// from this parent to root
    pub fn ancestors(&self) -> impl Iterator<Item=Self> {
        let mut current = self.parent();
        std::iter::from_fn(move || {
            let val = current.clone()?;
            current = val.parent();
            Some(val)
        })
    }
    
    pub fn ancestor_depths(&self) -> impl Iterator<Item=(usize,Self)> {
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
        for (i,c) in self.as_ref().char_indices().rev() {
            if c != '/' {
              continue; 
            }
            return &self.as_ref()[i+1..]
        }
        self.as_ref()
    }
    
    pub fn from(full: impl Into<Rc<str>>) -> Self {
        let full = full.into();
        Self {
            length: full.len() as u16,
            full,
        }
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

/// Never ends with a '/'
#[derive(Debug,Clone, Deref)]
pub struct AbsPath (CompactString);
impl AbsPath {
    pub fn new(mut path: &str) -> Self {
        path = path.trim_end_matches("/");
        let path = CompactString::from(path);
        Self(path)
    }
    pub fn join(&self, path: &Path) -> Self {
        if path.is_root() {
            return self.clone();
        }
        Self(format_compact!("{}/{}", self.as_ref(), path.as_ref()))
    }
}
impl AsRef<str> for AbsPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}