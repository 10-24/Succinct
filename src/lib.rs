
mod entry;
mod delta;
mod path;
mod state;

pub const STATE_FILE_NAME: &str = "fs_state.db";

pub fn get_root() -> &'static str {
    "/home/ryan/sync/"
}
