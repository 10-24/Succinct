use std::{collections::BTreeSet, mem, ops::DerefMut, sync::Arc, time::Duration};


use tokio::{sync::{Mutex, mpsc}, time::sleep};

use crate::delta::Delta;

pub struct DeltaEmitter {
    tx: mpsc::Sender<Deltas>,
    debounce: Duration,
    debounce_task: tokio::task::AbortHandle,
    
    deltas: Arc<Mutex<Deltas>>
}

impl DeltaEmitter {
    pub fn new(debounce: Duration) -> (Self,DeltaReceiver) {
        let (sender, receiver) = mpsc::channel(4);
        let debounce_task = tokio::spawn(async move {}).abort_handle();
        let pool = Arc::new(Mutex::new(BTreeSet::new()));
        (Self { tx: sender, debounce, debounce_task, deltas: pool }, receiver)
    }
    
    pub async fn send(&mut self, delta:Delta) {
        self.debounce_task.abort();
        self.deltas.lock().await.insert(delta);
        
        let tx = self.tx.to_owned();
        let pool = self.deltas.to_owned();
        let duration = self.debounce;
        
        self.debounce_task = tokio::spawn(async move {
            sleep(duration).await;
            let prev_pool = mem::take(&mut *pool.lock().await);
            _ = tx.send(prev_pool).await;
        }).abort_handle()
    }
}

pub type DeltaReceiver = mpsc::Receiver<Deltas>;

pub type Deltas = BTreeSet<Delta>;