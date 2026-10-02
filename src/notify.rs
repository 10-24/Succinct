
use debounced::{Debounced, debounced};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use std::{iter, path::Path, time::Duration};

use crate::{delta::{Delta, kind::DeltaKind}, notify::task::NotifyTask};

use rustc_hash::FxHashMap;

#[derive(Debug)]
pub enum Command {
    WatchDir(Box<Path>),
    UnwatchDir(Box<Path>)
}

type INotifyStream = EventStream<Box<[u8]>>;
type Descriptors = FxHashMap<WatchDescriptor, Box<Path>>;

mod task;

pub struct Notify {
    cmd_tx: mpsc::Sender<Command>,
    delta_stream: Debounced<ReceiverStream<Delta>>
}

impl Notify {
    pub fn new(debounce: Duration) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel(16);
        
        let delta_rx = NotifyTask::spawn(cmd_rx);
        let delta_rx = ReceiverStream::new(delta_rx);
        let delta_stream = debounced(delta_rx, debounce);
        Self { cmd_tx, delta_stream }
    }

    pub async fn watch_dir(&self, path: Box<Path>) {
        let cmd = Command::WatchDir(path);
        self.cmd_tx.send(cmd).await.unwrap();
    }

    pub async fn unwatch_dir(&self, path: Box<Path>) {
        let cmd = Command::UnwatchDir(path);
        self.cmd_tx.send(cmd).await.unwrap();
    }

    pub async fn drain_deltas(&mut self) -> impl Iterator<Item = Delta> + '_ {
        let first = self.delta_stream.next().await;
        let rest = iter::from_fn(move || self.delta_stream.next().now_or_never().flatten());
        first.into_iter().chain(rest)
    }
}




