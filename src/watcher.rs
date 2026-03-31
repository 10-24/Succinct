mod walk_directory;
use std::time::Duration;

use futures::StreamExt;
use ignore::gitignore::Gitignore;
use inotify::{Inotify, WatchMask};
use rustc_hash::FxHashMap;
use walk_directory::walk_directory;

use crate::{delta::{Delta, FileRecord}, delta_emitter::{DeltaEmitter, DeltaReceiver}, path::{AbsPath, Local}};

const DEBOUNCE_INTERVAL: Duration = Duration::from_secs(3);

const INOTIFY_BUFFER_SIZE: usize = 1024;




async fn spawn_watcher(root: AbsPath<Local>, ignore: Gitignore) -> DeltaReceiver  {
    let inotify = Inotify::init().unwrap();
    let mut watch_list = inotify.watches();
    let mut watch_descriptors = FxHashMap::default();
    for entry in walk_directory(root.to_owned(), &ignore).await {
        let descriptor_id = watch_list.add(entry.path.as_ref(), WATCH_MASK).unwrap().get_watch_descriptor_id();
        let name = entry.path.file_name().into();
        let parent_id = entry.parent_id;
        watch_descriptors.insert(descriptor_id, FileRecord { name, parent_id });
    }
    watcher(inotify)
}

fn watcher(inotify: Inotify, descriptors: FxHashMap<i32, FileRecord>) -> DeltaReceiver {
    
    let mut buffer = [0; INOTIFY_BUFFER_SIZE];
    let mut stream = inotify.into_event_stream(&mut buffer).unwrap();
  
    let (mut tx,rx) = DeltaEmitter::new(DEBOUNCE_INTERVAL);
    
    tokio::spawn(async move {
        while let Some(event) = stream.next().await {
            let Ok(event) = event else {
                eprintln!("Watch error {:?}",event);
                continue;
            };
            // Todo handle new file/directory
            
            let file = descriptors.get(&event.wd.get_watch_descriptor_id()).unwrap().to_owned();
            
        }
    });
    rx
}