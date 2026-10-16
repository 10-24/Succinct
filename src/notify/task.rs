use std::ffi::{OsString};

use chrono::Utc;
use inotify::Inotify;
use log::warn;
use rustc_hash::FxHashMap;
use strum::VariantArray;
use tokio::{select, sync::mpsc};
use tokio_stream::StreamExt;

use crate::{notify::{Command, Descriptors, EventKind, INotifyStream, event::{Event, EventData}}, util::arc_path::IntoArcPath};

pub struct NotifyTask {
    descriptors: Descriptors,
    notify_stream: INotifyStream,
}

impl NotifyTask {
    pub fn spawn(cmd_rx: mpsc::Receiver<Command>) -> mpsc::Receiver<Event> {
        let buf = vec![0u8; 2048].into_boxed_slice();
        let (event_tx, event_rx) = mpsc::channel(48);
        let task = Self {
            descriptors: FxHashMap::default(),
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
                    let event = self.handle_event(inotify_event.unwrap());
                    event_tx.send(event).await;
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
                    .extract_if(|_, path| 
                        path.starts_with(target_path.as_std_path())
                    ).for_each(|(descriptor, _)| {
                        _ = self.notify_stream.watches().remove(descriptor)
                    });
            }
        }
    }
    
    fn handle_event(&mut self, inotify_event: inotify::Event<OsString>) -> Event {
    
        let event = self.convert_event(&inotify_event);
        if event.1.kind.is_delete() {
            self.descriptors.remove(&inotify_event.wd);
        }
        event
    }
    
    fn convert_event(&self, e: &inotify::Event<OsString>) -> Event {
        let timestamp = Utc::now();
        let event_data = EventData::from_inotify_event(e, timestamp).unwrap();

        let child_name = e.name.as_ref().unwrap().to_str().unwrap();
        let parent_path = self.descriptors.get(&e.wd).unwrap();
        let path = parent_path.join(child_name).into_arc_path();

        (path, event_data)
    }

   
}
