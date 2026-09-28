use std::{path::{Path, PathBuf}, sync::Arc};

use inotify::{EventMask, Inotify};
use tokio::{select, sync::{self, mpsc}};

use crate::delta::DeltaKind;



pub fn spawn_notify(mut cmd_rx: mpsc::Receiver<PathBuf>) -> mpsc::Receiver<(DeltaKind, Arc<Path>)> {
    const MASK: u32 = EventMask::CREATE.union(EventMask::MODIFY)
    
    let mut buf = vec![0u8; 1024].into_boxed_slice();

    let mut notify_stream = Inotify::init().unwrap().into_event_stream(&mut buf).unwrap();
    
    tokio::spawn(async move {
        select! {
            new_path = cmd_rx.recv() => {
                notify_stream.watches().add(new_path, mask)
            }
        }
    });
    todo!()
}

