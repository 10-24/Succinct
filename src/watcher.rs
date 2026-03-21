use std::collections::{BTreeSet, HashMap};
use std::ffi::OsStr;
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use ignore::Walk;
use ignore::gitignore::Gitignore;
use inotify::{Event, EventMask, Inotify, WatchMask};
use tokio::sync::mpsc;
use tokio::time::{MissedTickBehavior, interval};

use crate::delta_emitter::{DeltaEmitter, DeltaReceiver};
use crate::{
    delta::{Delta, DeltaKind},
    path::{AbsPath, Path},
};

/// Time window to collect and merge file system events before sending
const DEBOUNCE_INTERVAL = Duration::from_secs(3);

/// Channel buffer size for the final delta output
const OUTPUT_CHANNEL_SIZE: usize = 100;

/// Channel buffer size for internal event passing between threads
const EVENT_CHANNEL_SIZE: usize = 1000;

/// Buffer size for reading inotify events
const INOTIFY_BUFFER_SIZE: usize = 4096;

/// Watch mask for monitoring file system events
const WATCH_MASK: WatchMask = WatchMask::CREATE
    .union(WatchMask::MODIFY)
    .union(WatchMask::DELETE)
    .union(WatchMask::MOVED_FROM)
    .union(WatchMask::MOVED_TO);



fn watch<F>(root: AbsPath, ignore: Gitignore) -> DeltaReceiver {
    
    let mut inotify = Inotify::init().unwrap();
    
    todo!()
}

fn walk_dir(root: AbsPath, ignore: &Gitignore) -> impl Iterator<Item = Box<str>> {
    
    vec![].into_iter()
}

fn spawn_watcher(root: AbsPath, inotify: Inotify) -> DeltaReceiver {
    let mut buffer = [0; 1024];
    let mut stream = inotify.into_event_stream(&mut buffer).unwrap();
    
    let (mut tx,rx) = DeltaEmitter::new(DEBOUNCE_INTERVAL);
    
    tokio::spawn(async move {
        while let Some(e) = stream.next().await {
            let Ok(event) = e else {
                eprintln!("Watch error {:?}",e);
                continue;
            };
            
            let delta = Delta::from(event,&root);
            tx.send(delta);
        }
    });
    rx
}

