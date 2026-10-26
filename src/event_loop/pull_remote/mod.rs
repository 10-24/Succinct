use crate::{database::table::Entry, util::arc_path::RelPath};

pub mod pull_remote;
mod find_deltas;
mod write_deltas;
mod temp_file;

type RemoteDeltas = Vec<Delta>;

pub struct Delta {
    path: RelPath,
    entry: Entry,
}

impl Delta {
    fn new(path: RelPath, entry: Entry) -> Self {
        Self { path, entry }
    }
}