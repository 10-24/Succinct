use std::sync::Arc;

use camino::Utf8Path;
use fjall::Database;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use crate::{context::Context, notify::{Command, Notify, event::Event, walk_dir::{walk, walk_dir}}};






mod constants;
mod context;
mod db;
mod state;
mod util;
mod notify;
/*
 * Unimplemented Features:
 * Handle if you update a child then delete the parent.
 * Fix Root Id
 * Watch new files
 * Error Notifications
 * Add and remove ignored file from the command line
 * Add check in walk dir for uninitialized files
 * Graceful shutdown
 * 
 * 
 * 

 Steps
// Pull changes from remote DB, excluding those that collide with local changes
 // Push queued changes to opendal
 // Update remote db
 // Aggregate File changes
 // Save queued changes to local_db
 */
#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let ctx = Context::load().await;
    let walk_dir_stream = walk_dir(ctx.environment().sync_dir.clone(), ctx.exclude().clone()).filter_map(Result::ok);
    
    let notify = Notify::new(ctx.config().debounce_duration);
    notify.watch_dirs(walk_dir_stream);

    
    loop {

        
       
    }

   
}




pub fn filter_deltas(deltas: impl Iterator<Item = &Event>, notify_cmd_tx: mpsc::Sender<Command>) -> () {
    deltas.filter(predicate)
}