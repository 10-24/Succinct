use camino::{Utf8Path, Utf8PathBuf};

use crate::util::arc_path::ArcPath;



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelPath(Box<Utf8Path>);

impl RelPath {
    pub fn from(rel_path: &str) -> Self {
        Self(rel_path.into())
    }
}





#[derive(Debug)]
pub struct RelPathManager {
    root: ArcPath,
}

impl RelPathManager {
    pub fn new(root: ArcPath) -> Self {
        Self { root }
    }

    pub fn strip(&self, abs_path: &Utf8Path) -> Option<RelPath> {
        let rel_path = abs_path.strip_prefix(self.root.as_std_path()).ok()?;
        let rel_path = RelPath(Box::from(rel_path));
        Some(rel_path)
    }

    pub fn join(&self, rel_path: impl AsRef<Utf8Path>) -> Utf8PathBuf {
        self.root.join(rel_path.as_ref())
    }

    pub fn join_str(&self, rel_path: &str) -> Utf8PathBuf {
        self.root.join(rel_path)
    }
}