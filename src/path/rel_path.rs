
use std::rc::Rc;

use derive_more::Deref;

use crate::config::settings::INTERNAL_ROOT_NAME;


#[derive(Debug, Clone, Deref)]
pub struct RelPath(Rc<str>);

impl RelPath {
    
    pub fn new(p: impl Into<Rc<str>>) -> Option<Self> {
        let path_str = p.into();
        Self::is_valid(&path_str).then_some(Self(path_str))
    }
    
    pub fn is_valid(path_str:&str) -> bool {
        path_str.starts_with(INTERNAL_ROOT_NAME) && !path_str.ends_with('/')
    }

    pub fn depth(&self) -> usize {
        self.as_ref().chars().filter(|c| *c == '/').count()
    }

    pub fn is_root(&self) -> bool {
        INTERNAL_ROOT_NAME == self.0.as_ref()
    }
    
   
    pub fn child(&self, name: &str) -> Self {
        let path_str = format!("{self}/{name}");
        debug_assert!(Self::is_valid(&path_str), "invalid path: {path_str}");
        Self(path_str.into())
    }

    pub fn root() -> Self {
        Self(INTERNAL_ROOT_NAME.into())
    }
    
}

impl std::fmt::Display for RelPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

