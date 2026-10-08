mod task;
mod event_kind;
mod event;
pub mod walk_dir;

use camino::Utf8Path;
use debounced::{Debounced, debounced};
use futures::{FutureExt, Stream, StreamExt};
use inotify::{EventStream, WatchDescriptor};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use std::{iter, path::Path, sync::Arc, time::Duration};
use crate::{notify::task::NotifyTask, util::apath::APath};
use tokio_util::sync::PollSender;
use futures::SinkExt;
use rustc_hash::FxHashMap;
pub use event_kind::EventKind;
pub use event::{Event, EventData};

#[derive(Debug)]
pub enum Command {
    WatchDir(Arc<Utf8Path>),
    UnwatchDir(Arc<Utf8Path>)
}

type INotifyStream = EventStream<Box<[u8]>>;
type Descriptors = FxHashMap<WatchDescriptor, Arc<Utf8Path>>;


pub struct Notify {
    cmd_tx: mpsc::Sender<Command>,
    event_stream: Debounced<ReceiverStream<Event>>
}

impl Notify {
    pub fn new(debounce: Duration) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel(16);
        
        let delta_rx = NotifyTask::spawn(cmd_rx);
        let delta_rx = ReceiverStream::new(delta_rx);
        let event_stream = debounced(delta_rx, debounce);
        Self { cmd_tx, event_stream }
    }

    pub async fn watch_dir(&self, path: Arc<Utf8Path>) {
        let cmd = Command::WatchDir(path);
        self.cmd_tx.send(cmd).await.unwrap();
    }

    pub async fn unwatch_dir(&self, path: Arc<Utf8Path>) {
        let cmd = Command::UnwatchDir(path);
        self.cmd_tx.send(cmd).await.unwrap();
    }

    pub async fn drain_deltas(&mut self) -> impl Iterator<Item = Delta> + '_ {
        let first = self.event_stream.next().await;
        let rest = iter::from_fn(move || self.event_stream.next().now_or_never().flatten());
        first.into_iter().chain(rest)
    }

    pub fn watch_dirs(&self, stream: impl Stream<Item = APath> + Send + 'static) {
        let cmd_sink = PollSender::new(self.cmd_tx.clone());
    
        tokio::spawn(async move {
            _ = stream
                .map(|p| Ok(Command::WatchDir(p)))
                .forward(cmd_sink)
                .await;
        });
    }
}





