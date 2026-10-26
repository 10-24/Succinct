use std::path::Path;

use camino::Utf8Path;
use globset::{Glob, GlobSet, GlobSetBuilder};
use log::warn;

use crate::util::arc_path::ArcPath;

pub struct Globs {
    set: GlobSet,
    root: ArcPath,
}

impl Globs {
    pub fn new(root: ArcPath, patterns: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Self, globset::Error>  {
        let mut builder = GlobSetBuilder::new();
        for pat in patterns {
            let glob = Glob::new(pat.as_ref())?;
            builder.add(glob);
        }
        let set = builder.build()?;
        Ok(Self{set, root})
    }

    
    pub fn matches(&self, path: impl AsRef<Utf8Path>) -> bool {
        self.matches_std_path(path.as_ref().as_std_path())
    }
    
    pub fn matches_std_path(&self, path: impl AsRef<Path>) -> bool {
        let Ok(relpath) = path.as_ref().strip_prefix(self.root.as_ref()) else {
            warn!("Cannot match glob-patterns on paths outside the root. \n path: {} \n root: {}", path.as_ref().to_string_lossy(), self.root);
            return false;
        };
        self.set.is_match(relpath)
    }
}
