
use std::{ops::Not, path::{Path, PathBuf}};

use derive_more::Display;
use serde::{Deserialize, Serialize};


#[derive(Debug,Hash,Clone, Serialize, Deserialize)]
pub struct RelPath(Box<str>);


impl RelPath {


    pub fn from_str_unchecked(s: &str) -> Self {
        Self(s.into())
    }
    pub fn from(path: &Path, prefix: &Path) -> Result<Self, Error> {
        let rel_path = path.strip_prefix(prefix).map_err(|_| Error::missing_prefix(path, prefix))?.to_str().unwrap();
        let rel_str = Self::extract_valid(rel_path).ok_or_else(|| Error::path_is_sparse(rel_path))?;
        
        Ok(Self(rel_str.into()))
    }
    
    fn extract_valid(path: &str) -> Option<&str> {
        const DELIMITER: char = '/';
 
        let stripped = path.trim_matches(DELIMITER);
      
        if stripped.is_empty() || stripped.split(DELIMITER).any(str::is_empty) {
            return None;
        }
        Some(stripped)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Path `{path}` does not begin with `{prefix}`")]
    MissingPrefix {
        path: String,
        prefix: String,
    },
    #[error("Path `{path}` is empty or has empty segments")]
    PathIsSparse {
        path: String,
    },
}

impl Error {
    pub fn missing_prefix(path: &Path, prefix: &Path) -> Self {
        let path = path.to_string_lossy().to_string();
        let prefix = prefix.to_string_lossy().to_string();
        Self::MissingPrefix { path, prefix }
    }

    pub fn path_is_sparse(path: &str) -> Self {
        Self::PathIsSparse { path: path.to_owned() }
    }
}

