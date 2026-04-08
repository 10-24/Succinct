use std::{time::Duration};
use futures::Stream;
use tokio::{signal::unix::{self, SignalKind}, sync::{mpsc, watch}, time::sleep};
use tokio_stream::{StreamExt, adapters::Skip, wrappers::{ReceiverStream, SignalStream, WatchStream}};

#[derive(Debug)]
pub struct DebouncedTx {
    duration: Duration,
    task: tokio::task::AbortHandle,
    tx: watch::Sender<()>,
}
impl DebouncedTx {
    fn new(duration: Duration) -> (DebouncedTx, DebouncedRx) {
        let task = tokio::spawn(async {}).abort_handle();
        let (tx, rx) = watch::channel(());
        let rx = WatchStream::new(rx).skip(1); // So the listener doesn't need to skip the initial value

        let tx = Self { duration, task, tx };
        (tx,rx)
    }
    
    pub fn restart(&mut self) {
        self.task.abort();
        
        let duration = self.duration;
        let tx = self.tx.clone();
        self.task = tokio::spawn(async move {
            sleep(duration).await;
            tx.send(());
        }).abort_handle();
    }
    
    /// Combines debounce signal with override for task exist and kill
    pub fn new_graceful(duration: Duration) -> (DebouncedTx, impl Stream<Item = ()>) {
        let (debounce_tx,debounce_rx) = Self::new(duration);
        
        let sigint_signal = unix::signal(SignalKind::interrupt()).unwrap().into();
        let sigint_stream = SignalStream::new(sigint_signal).map(|_| ());
        
        let sigterm_signal = unix::signal(SignalKind::terminate()).unwrap();
        let sigterm_stream = SignalStream::new(sigterm_signal).map(|_| ());
        
        let rx = debounce_rx.merge(sigint_stream).merge(sigterm_stream);
        
        (debounce_tx, rx)
    }
    

}


pub type DebouncedRx = Skip<WatchStream<()>>;

