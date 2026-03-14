use std::rc::Rc;

use std::hash::{Hash, Hasher};

use rustc_hash::FxHasher;

use crate::entry::finish;



#[derive(Debug,Clone,Eq,PartialEq)]
pub struct Path {
    full: Rc<str>,
    length: u16,
}

impl Path {
    
    // No trailing slashes
    pub fn new_relative(abs_path: &str, root: &str) -> Option<Self> {
        if !abs_path.starts_with(root) {
            return None
        }
        let mut path = &abs_path[root.len()+1..];
        
        if path.ends_with("/") {
            path = &path[..path.len() - 1];
        }
        
        Some(Self{
            length: path.len() as u16,
            full: Rc::from(path),
        })
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
    
    pub fn last(&self) -> Option<&str> {
        for (i,c) in self.as_ref().char_indices().rev() {
            if c != '/' {
              continue; 
            }
            return Some(&self.as_ref()[..i+1])
        }
        None
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