use std::{path::PathBuf, sync::Arc};

use camino::{Utf8Path, Utf8PathBuf};

pub type APath = Arc<Utf8Path>;

pub fn buf_into_apath(path: PathBuf) -> APath {
    Utf8PathBuf::try_from(path).expect("All paths must be UTF-8").into()
}

pub fn into_apath(path: impl Into<Utf8PathBuf>) -> APath {
    path.into().into()
}