use std::{path::{Path, PathBuf}, sync::Arc};

use inotify::{EventMask, Inotify};
use tokio::{select, sync::{self, mpsc}};

use crate::delta::DeltaKind;



pub fn spawn_notify(mut cmd_rx: mpsc::Receiver<PathBuf>) -> mpsc::Receiver<(DeltaKind, Arc<Path>)> {
    
    let mut buf = vec![0u8; 1024].into_boxed_slice();
    let watch_mask = DeltaKind::all_inotify_masks();
    let mut notify_stream = Inotify::init().unwrap().into_event_stream(&mut buf).unwrap();
    
    tokio::spawn(async move {
        select! {
            new_path = cmd_rx.recv() => {
                notify_stream.watches().add(new_path, watch_mask)
            }
        }
    });
    todo!()
}

