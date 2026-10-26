use std::{collections::BTreeSet, ffi::OsString, io, ops::{Bound, Range, RangeInclusive}, time::{Duration, SystemTime}};

use chrono::{DateTime, Utc};
use globset::Candidate;
use inotify::{Inotify, WatchDescriptor};
use log::{error, warn};
use rustc_hash::FxHashMap;
use strum::VariantArray;
use tokio::{select, sync::mpsc};
use tokio_stream::StreamExt;

use crate::{notify::{Command, EventKind, INotifyStream, event::{Event, EventData, IgnoredEvent}}, util::{arc_path::{ArcPath, IntoArcPath}, bimap::BiMap}};

pub struct NotifyTask {
    descriptors: BiMap<WatchDescriptor, ArcPath>,
    ignored_events: BTreeSet<IgnoredEvent>,
    notify_stream: INotifyStream,
}

impl NotifyTask {
    pub fn spawn(cmd_rx: mpsc::Receiver<Command>) -> mpsc::Receiver<Event> {
        let buf = vec![0u8; 2048].into_boxed_slice();
        let (event_tx, event_rx) = mpsc::channel(48);
        let task = Self {
            descriptors: BiMap::new(),
            ignored_events: BTreeSet::new(),
            notify_stream: Inotify::init().unwrap().into_event_stream(buf).unwrap(),
        };

        tokio::spawn(task.run(cmd_rx, event_tx));
        event_rx
    }

    async fn run(mut self, mut cmd_rx: mpsc::Receiver<Command>, event_tx: mpsc::Sender<Event>) {
        loop {
            select! {
                Some(command) = cmd_rx.recv() =>
                    self.execute_command(command),
                Some(inotify_event) = self.notify_stream.next() => {
                    if let Some(event) = self.handle_event(inotify_event) {
                        event_tx.send(event).await;
                    }
                }
            }
        }
    }

    fn execute_command(&mut self, command: Command) {
        const MASK: inotify::WatchMask = EventKind::WATCH_MASKS;

        match command {
            Command::WatchDir(path) => {
                let descriptor = self
                    .notify_stream
                    .watches()
                    .add(path.as_std_path(), MASK)
                    .unwrap();
                self.descriptors.insert(descriptor, path);
            }
            Command::UnwatchDir(target_path) => {
                self.descriptors
                    .remove_by_value(&target_path);
            }
            
        }
    }
    
    fn handle_event(&mut self, inotify_event: Result<inotify::Event<OsString>, io::Error>) -> Option<Event> {
        match inotify_event {
            Ok(event) => Some(self.convert_event(&event)),
            Err(err) => {
                warn!("failed to handle inotify event: {}", err);
                None
            },
        }
    }
    
    fn convert_event(&self, e: &inotify::Event<OsString>) -> Event {
        let timestamp = Utc::now();
        let event_data = EventData::from_inotify_event(e, timestamp).unwrap();

        let child_name = e.name.as_ref().unwrap().to_str().unwrap();
        let parent_path = self.descriptors.get_value(&e.wd).unwrap();
        let path = parent_path.join(child_name).into_arc_path();

        (path, event_data)
    }

}






