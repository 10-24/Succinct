use std::rc::Rc;
use bytemuck::TransparentWrapper;

use derive_more::Deref;

use crate::{config::ROOT_NAME, db::tables::file::{FileName}, state::file::name::FileName};

#[derive(Debug, Clone, Deref)]
pub struct RelPath(Rc<str>);

impl RelPath {
    pub fn from(p: impl AsRef<str>) -> Self {
        let path_str = p.as_ref().trim_matches('/');
        Self(Rc::from(path_str))
    }

    pub fn depth(&self) -> u16 {
        self.as_ref().chars().filter(|c| *c == '/').count() as u16
    }

    pub fn child(&self, name: &FileName) -> Self {
        let path_str = format!("{self}/{name}");
        Self::from(path_str)
    }

    pub fn from_components<T: AsRef<FileName>>(components: impl Iterator<Item = T>) -> Self {
        let mut path = String::with_capacity(72);
        for component in components {
            if !path.is_empty() {
                path.push('/');
            }
            path.push_str(component.as_ref().as_str());
        }
Self::from(path)
    }
    
    pub fn components(&self) -> impl DoubleEndedIterator<Item=&str> {
        self.split('/')
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RelPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}



