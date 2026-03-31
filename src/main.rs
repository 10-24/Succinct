#[cfg(target_os = "linux")]
use crate::config::{APP_NAME, Config, IGNORE_FILE_NAME, SETTINGS_FILE_NAME};
use rustc_hash::{FxHashMap,FxBuildHasher};

mod delta;
mod path;
mod state;
mod delta_emitter;
mod config;
mod watcher;
mod path_cache;
mod database;
/*
 * Unimplemented Features:
    * Handle if you update a child then delete the parent.
    * Fix Root Id
    * Watch new files
    * Error Notifications
    * Add and remove ignored file from the command line
    * Add check in walk dir for uninitialized files
 */


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load().await;
    let fs_walk = ignore::WalkBuilder::new(config.local_root_path.as_ref()).add_custom_ignore_filename(IGNORE_FILE_NAME).build();
    
    Ok(())
}


