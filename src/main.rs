use std::{collections::HashMap, io, sync::Arc};

use futures::stream;
use globset::GlobSet;
use tokio_stream::StreamExt;

use crate::{context::{Context, ContextInner, Databases}, event_loop::{filter_events::{FilteredEvents, filter_events}, find_remote_deltas::fetch_remote_state, stage_deltas::stage_deltas}, notify::{Notify, walk_dir::walk_dir}, util::arc_path::IntoArcPath as _};





mod remote_drive;
mod event_loop;
mod database;
mod constants;
mod context;
mod util;
mod notify;
/*
 * Unimplemented Features:
 * 
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

    let ctx = Context::load().await?;
    
    
    let mut notify = start_notify(&ctx);
    
    
    loop {
        // Pull
        
        


        // Await events
        let events = notify.drain_deltas().await;
        let FilteredEvents { events, new_dirs } = filter_events(&ctx, events).await?;
        notify.watch_dirs(new_dirs).await;

        // Stage Events
       
        stage_deltas(&ctx, events);
    
    }

   
}


fn start_notify(ctx: &ContextInner) -> Notify {
    let notify = Notify::new(ctx.config.debounce_duration);
    let walk_dir_stream = walk_dir(ctx.config.sync_dir.clone(), ctx.exclude.clone()).filter_map(Result::ok).map(|entry| entry.path().into_arc_path());
    notify.watch_dirs_stream(walk_dir_stream);
    notify
}