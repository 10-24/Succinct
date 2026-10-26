use std::{collections::HashMap, io, sync::Arc};

use futures::stream;
use globset::GlobSet;

use rustc_hash::FxHashMap;
use tokio_stream::StreamExt;

use crate::{context::Context, event_loop::{deduplicate_events::deduplicate_events, find_missed_child_create_events::find_missed_child_create_events, ignored_events::IgnoredEvents, stage_deltas::stage_deltas}, notify::{Notify, event::{Event, EventData}, event_kind::DeltaKind, walk_dir::walk_dir}, util::arc_path::{ArcPath, IntoArcPath}};






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
    let mut ignored_events = IgnoredEvents::new();
    
    loop {
        // Pull
        // …
  
        // Push
        


        // Await events
        let events = notify.drain_deltas().await;
        let events = reconcile_events(&ctx, events, &mut ignored_events).await;
        for (path, event) in events.iter() {
            if !event.is_dir {
                continue;
            }
            match event.kind.delta() {
                DeltaKind::Create => notify.watch_dir(path.clone()).await,
                DeltaKind::Delete => notify.unwatch_dir(path.clone()).await,
                _ => {},
            }
        }

        
        stage_deltas(&ctx, events);
    }

   
}


fn start_notify(ctx: &Context) -> Notify {
    let notify = Notify::new(ctx.config.debounce_duration);
    let walk_dir_stream = walk_dir(ctx.config.sync_dir.clone(), ctx.exclude.clone()).filter_map(Result::ok).map(|entry| entry.path().into_arc_path());
    notify.watch_dirs_stream(walk_dir_stream);
    notify
}


async fn reconcile_events(ctx: &Context, events: impl Iterator<Item = Event>, ignored_events: &mut IgnoredEvents,) -> FxHashMap<ArcPath, EventData> {
    let events = events.filter(|(path,_)| !ctx.exclude.matches(path));
    let events = ignored_events.filter(events);
    let events = deduplicate_events(events, Default::default());
    let missed_events = find_missed_child_create_events(&events, ctx.exclude.clone()).await;
    let events = deduplicate_events( missed_events, events);
    events
}