use log::{info, warn};
use redb::{Database, ReadableDatabase, ReadableTable};
use tokio::io;
use object_store::{self as obj, GetResult, ObjectStoreExt};
use tokio_util::bytes::Bytes;
use crate::{
    context::Context, database::{db::Db, table::State, table_trait::Txn as _}, event_loop::{ignored_events::IgnoredEvents, pull_remote::{find_deltas::find_remote_deltas, write_deltas::write_deltas}}, remote_drive::{self, RemoteDriveError}
};


pub async fn pull_remote_deltas(ctx: &Context) -> IgnoredEvents {
    
    let remote_state_opt = fetch_remote_state(ctx).await.map(Some).or_else(handle_remote_state_error).unwrap();
    let Some(remote_state) = remote_state_opt else {
        return IgnoredEvents::new();
    };

    let local_txn = ctx.db.state.begin_write().unwrap();
    {
        let local_state_table = local_txn.open::<State>().unwrap();
        let remote_deltas = {
            let remote_txn = remote_state.begin_read().unwrap();
            let remote_state_table = remote_txn.open::<State>().unwrap();
            find_remote_deltas(&remote_state_table,&local_state_table)
        };
        
        let successful_deltas = write_deltas(ctx, remote_deltas).await;
        
    }
    local_txn.commit().unwrap();
    Ok(true)
}

async fn fetch_remote_state(ctx: &Context) -> obj::Result<Database> {
    let path = ctx.paths.remote_state();
    let bytes: &Bytes = ctx.object_store.get(path).await?.bytes().await?;
    Ok(Db::from_bytes(&bytes).unwrap())
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Sync failed locally: {0}")]
    Local(io::Error),
    #[error("Sync failed remotely: {0}")]
    Remote(RemoteDriveError)
}

fn handle_remote_state_error(err: obj::Error) -> Result<Option<Database>, SyncError> {
    let err = RemoteDriveError::from(err);
    match err {
        RemoteDriveError::FileNotFound(_) => {
            info!("skipping pull_remote_deltas reason=remote_state_not_found");
            Ok(None)
        },
        RemoteDriveError::Network{..} => {
            warn!("skipping pull_remote_deltas reason=network_error err={err}");
            Ok(None)
        },
        err => Err(SyncError::Remote(err))
    }
}