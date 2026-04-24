use std::time::Duration;

use inotify::Inotify;
use tokio::{select};
use tokio_stream::StreamExt;

use crate::{delta::{DeltaKind, Deltas}, tree_sitter::{predelta_buffer::predelta_buffer::{PredeltaBuffer, UniqueEvents}, tree_sitter::{PredeltaKV, Predeltas, TreeSitter}}};

const INOTIFY_BUFFER_SIZE: usize = 1024;
const DEBOUNCE_INTERVAL: Duration = Duration::from_secs(3);

impl TreeSitter {
 
    pub(crate) async fn watch(mut self,inotify:Inotify) {
        let buffer = Box::from([0; INOTIFY_BUFFER_SIZE]);
        let mut inotify_stream = inotify.into_event_stream(buffer).unwrap();
        
        let (mut predelta_buffer,mut predelta_drain) = PredeltaBuffer::new(DEBOUNCE_INTERVAL);
        loop {
            select! {
                Some(Ok(event)) = inotify_stream.next() => {
                    let predelta = PredeltaKV::from_event(event);
                    predelta_buffer.add(predelta);
                },
                Some(predeltas) = predelta_drain.recv() => {
                    self.output_deltas(predeltas);
                },
            }
        }
    }
    
    async fn output_deltas(&mut self,events:UniqueEvents){
        let mut deltas = Deltas::default();
        for (i,event) in events.into_iter().enumerate() {
            let index = i as u16;
            match event.kind {
                DeltaKind::Create => {
                    deltas.extend( self.handle_create(event, index).await);
                }
                DeltaKind::Update => {
                    deltas.extend(self.handle_update(event, index));
                }
                DeltaKind::Delete => {
                    deltas.extend(self.handle_delete(event, index));
                }
            }
        }
        
        if !deltas.is_empty() {
            self.output_tx.send(deltas).await;
        }
    }
    
    
 
}



