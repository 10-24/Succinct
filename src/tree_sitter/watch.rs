use std::time::Duration;

use inotify::Inotify;
use tokio::{select};
use tokio_stream::StreamExt;

use crate::{delta::Deltas, tree_sitter::{predelta_buffer::predelta_buffer::{PredeltaBuffer, UniquePredeltas}, tree_sitter::{PredeltaKV, Predeltas, TreeSitter}}};

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
    
    async fn output_deltas(&mut self,predeltas:UniquePredeltas){
        let mut deltas = Deltas::default();
        for (i,predelta) in predeltas.into_iter().enumerate() {
            let predelta = predelta.into();
            let new_deltas = self.convert_predelta(predelta).await;
            deltas.extend(new_deltas);
        }
    }
    
 
}



