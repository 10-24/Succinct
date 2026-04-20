use std::{iter::Rev, mem, sync::Arc, time::Duration, vec::IntoIter};

use futures::Stream;
use tokio::sync::{Mutex, mpsc::{self, Receiver}};
use tokio_stream::StreamExt;

use crate::{hashmap, hashset, tree_sitter::{event::EventKV, predelta_buffer::debouncer::DebouncedTx}};



/// Queues predeltas until for a specified time (debounced). Then outputs them.
pub struct PredeltaBuffer {
    debounced_tx: DebouncedTx,
    buffer: Arc<Mutex<Vec<EventKV>>>,
}

impl PredeltaBuffer {
    pub fn new(debounce_duration: Duration) -> (Self, Receiver<UniquePredeltas>) {
        let (output_tx, rx) = mpsc::channel(2);
        let (debounced_tx, debounced_rx) = DebouncedTx::new_graceful(debounce_duration);
        let buffer = Arc::from(Mutex::default());
        tokio::spawn( Self::emitter(buffer.clone(), output_tx, debounced_rx));
       
        (Self { debounced_tx, buffer }, rx)
    }

    pub async fn add(&mut self,  predelta_kv: EventKV) {
        self.debounced_tx.restart();
        {
            let mut buffer = self.buffer.lock().await;
            buffer.push(predelta_kv);
        }
    }

    async fn emitter(
        buffer: Arc<Mutex<Vec<EventKV>>>,
        output_tx: mpsc::Sender<UniquePredeltas>,
        mut debounced_rx: impl Stream<Item = ()> + Unpin,
    ) {
        
        loop {
            debounced_rx.next().await;
            
            let prev_buffer = {
                let mut buffer = buffer.lock().await;
                let buffer_len = buffer.len(); 
                mem::replace(&mut *buffer,Vec::with_capacity(buffer_len*3/2))
            };
            let unique = UniquePredeltas::new(prev_buffer);
            output_tx.send(unique).await;
        }
    }
    

    
   
}

/// Use `into_iter` to iterate in proper order of creation
#[derive(Debug)]
pub struct UniquePredeltas(Vec<EventKV>);

impl UniquePredeltas {
    fn new(buffer: Vec<EventKV>) -> Self {
        let mut prev = hashset(buffer.len());
        let mut unique = Vec::with_capacity(buffer.len());
        for predelta in buffer.into_iter().rev() {
            let new = prev.insert(predelta.descriptor);
            if new {
                unique.push(predelta);
            }
        }
        Self(unique)
    }
}

impl IntoIterator for UniquePredeltas {
    type Item = EventKV;
    type IntoIter = Rev<IntoIter<Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().rev()
    }
}