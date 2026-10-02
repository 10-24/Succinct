use std::ffi::OsString;


use inotify::{Event, Inotify};
use rustc_hash::FxHashMap;
use strum::VariantArray;
use tokio::{select, sync::mpsc};
use tokio_stream::StreamExt;

use crate::{delta::{Delta, kind::DeltaKind}, notify::{Command, Descriptors, INotifyStream, Notify}};


pub struct NotifyTask {
    descriptors: Descriptors,
    notify_stream: INotifyStream,
}
impl NotifyTask {
    pub fn spawn(cmd_rx: mpsc::Receiver<Command>) -> mpsc::Receiver<Delta> {
        
        let buf = vec![0u8; 2048].into_boxed_slice();
        let (delta_tx, delta_rx) = mpsc::channel(48);
        let task = Self {
            descriptors: FxHashMap::default(),
            notify_stream: Inotify::init().unwrap().into_event_stream(buf).unwrap(),
        };
        
        tokio::spawn(task.run(cmd_rx, delta_tx));
        delta_rx
    }

    async fn run(mut self, mut cmd_rx: mpsc::Receiver<Command>, delta_tx: mpsc::Sender<Delta>) {
        loop {
            select! {
                Some(command) = cmd_rx.recv() => 
                    self.execute_command(command),
                Some(event) = self.notify_stream.next() => {
                    let event = event.unwrap();
                    let delta = self.convert_event_into_delta(event);
                    delta_tx.send(delta).await;
                }
            }
        }
    }

    fn execute_command(&mut self, command: Command) {
        const MASK: inotify::WatchMask = DeltaKind::WATCH_MASKS;
        
        match command {
            Command::WatchDir(path) => {
                let descriptor = self.notify_stream.watches().add(&path, MASK).unwrap();
                self.descriptors.insert(descriptor,path);
            }
            Command::UnwatchDir(target_path) => {
                let Some((descriptor,_)) = self.descriptors.iter().find(|(_, path)| **path == target_path) else {
                    return
                };
                let descriptor = descriptor.clone();
                self.descriptors.remove(&descriptor);
                self.notify_stream.watches().remove(descriptor);
            }
        }
    }
    
    fn convert_event_into_delta(&self, e: Event<OsString>,) -> Delta {
    
        let parent_path = self.descriptors.get(&e.wd).unwrap();
        let child_name = e.name.unwrap();
        let path = parent_path.join(child_name).into();
        
        let kind = DeltaKind::VARIANTS.iter().copied().find(|delta_kind| e.mask.contains(delta_kind.as_event_mask())).unwrap();
        let is_dir = e.mask.contains(inotify::EventMask::ISDIR);
        
        Delta {
            kind,
            path,
            is_dir,
        }
    }
}