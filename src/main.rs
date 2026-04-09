#[cfg(target_os = "linux")]
use rustc_hash::FxHashSet;

use rustc_hash::{FxBuildHasher, FxHashMap};
use tokio::join;

use crate::{
    config::config::Config,
    database::{local_reader::Db, local_writer::DbWriter},
    state::state::State,
    tree_sitter::tree_sitter::TreeSitter,
};

mod delta;
mod path;
mod state;

mod config;
mod database;
mod tree_sitter;
mod tests;
mod utils;
/*
 * Unimplemented Features:
 * Handle if you update a child then delete the parent.
 * Fix Root Id
 * Watch new files
 * Error Notifications
 * Add and remove ignored file from the command line
 * Add check in walk dir for uninitialized files
 * Graceful shutdown
 */

#[tokio::main]
async fn main() {
    let (config, db) = join!(Config::load(), Config::init_local_database());
    let local_reader = Db::init(db).await;
    let local_writer = DbWriter::new(local_reader.db().clone());
    let (ignore, remote_drive) = join!(config.create_ignore(), config.connect_remote_drive());

    let mut delta_rx =
        TreeSitter::start(config.local.root_path.clone(), ignore, local_reader.clone());
    let mut state = State::new(
        local_writer,
        local_reader,
        remote_drive,
        config.local.root_path.clone(),
    );
    while let Some(deltas) = delta_rx.recv().await {
        state.push_deltas(deltas).await;
    }
}

pub fn hashmap<K, V>(capacity: usize) -> FxHashMap<K, V> {
    FxHashMap::with_capacity_and_hasher(capacity, FxBuildHasher::default())
}
pub fn hashset<T>(capacity: usize) -> FxHashSet<T> {
    FxHashSet::with_capacity_and_hasher(capacity, FxBuildHasher::default())
}
