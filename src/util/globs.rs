use std::{ops::Deref, path::Path};

use camino::Utf8Path;
use derive_more::{AsRef, Deref};
use globset::{GlobSet, GlobSetBuilder};
use log::warn;
use serde::{Deserialize, Deserializer, de};

use crate::util::arc_path::ArcPath;
#[derive(Debug)]
pub struct Globs {
    set: GlobSet,
    root: ArcPath,
}

impl Globs {
    pub fn new(root: ArcPath, globs: impl IntoIterator<Item = Glob>) -> Result<Self, globset::Error>  {
        
        let set = GlobSet::new(globs)?;
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


#[derive(Debug, AsRef)]
pub struct Glob(globset::Glob);


impl<'de> Deserialize<'de> for Glob {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let pattern = <&str>::deserialize(deserializer)?;
                
        Self::try_from(pattern).map_err(|err| {
            de::Error::custom(format!("invalid glob pattern {pattern}: {err}"))
        })
    }
}

impl serde_nixos::NixosTypeGenerator for Glob {
    fn nixos_type_definition() -> String {
        // Doesn't need a separate type definition block
        String::new()
    }

    fn nixos_options() -> String {
        // No nested options
        String::new()
    }

    fn nixos_type() -> String {
        // Treat as a plain Nix string
        "types.str".to_string()
    }
}


impl TryFrom<&str> for Glob {
    type Error = globset::Error;

    fn try_from(pattern: &str) -> Result<Self, Self::Error> {
        globset::Glob::new(pattern).map(Self)
    }
}
