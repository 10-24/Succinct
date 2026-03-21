use crate::config::{APP_NAME, Config, IGNORE_FILE_NAME, SETTINGS_FILE_NAME};


mod delta;
mod path;
mod state;
mod delta_emitter;
mod config;
mod watcher;
mod path_cache;
/*
 * Unimplemented Features:
    * Error Notifications
    * Watch new files
 */


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load().await;
    let fs_walk = ignore::WalkBuilder::new(config.local_root_path.as_ref()).add_custom_ignore_filename(IGNORE_FILE_NAME).build();
    
    Ok(())
}
