pub mod pull_remote;
mod find_deltas;
mod write_deltas;
mod temp_file;

#[derive(Default)]
pub struct RemoteDeltas {
    pub update: Vec<Box<str>>,
    pub delete: Vec<Box<str>>,
}