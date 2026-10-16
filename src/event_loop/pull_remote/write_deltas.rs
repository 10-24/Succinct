use std::{error::Error, os::fd::{AsFd, AsRawFd, OwnedFd}, sync::Arc};



use anyhow::Context as _;
use log::error;
use object_store::{GetResult, ObjectStoreExt as _, aws::AmazonS3, path::Path as ObjPath};

use nix::fcntl::{linkat};
use tokio::{join, task::{JoinError, JoinSet}};
use crate::{context::Context, event_loop::pull_remote::{RemoteDeltas, temp_file::{self, TempFile}}, notify::ephemeral_channel::EphemeralSender, util::arc_path::ArcPath};



pub async fn write_deltas(ctx: &Context, remote_deltas: RemoteDeltas, ignore_events: EphemeralSender<ArcPath>) -> Vec<ArcPath> {
    
    let mut set = JoinSet::new();
    for update in remote_deltas.update {
        let obj_path = ObjPath::from(update.to_string());
        set.spawn(download_file(ctx.clone(), obj_path));
    }
    
    let mut successful_deltas = RemoteDeltas::default();
    while let Some(res) = set.join_next().await {
        if let Some(tmp_file) = handle_task_result(res) {
            let dest = tmp_file.destination().clone();
            ignore_events.insert(dest.clone()).await;
            tmp_file.replace().unwrap();
            successful_deltas.update.push(dest);
        }
    }
}

async fn download_file(ctx: Context, remote_path: ObjPath) -> anyhow::Result<TempFile> {
    
    
    let local_path = ctx.config.sync_dir.join(remote_path.as_ref());
    let (get_result, tmp_file) = join!(
        ctx.object_store.get(&remote_path),
        TempFile::create(local_path)
    );
    let get_result = get_result.with_context(|| format!("failed to download `{remote_path}`"));
    let tmp_file = tmp_file.with_context(|| format!("failed to create temp file for `{remote_path}`"));
    
    tmp_file?.write_chunks(get_result?.into_stream()).await
}





fn handle_task_result(res: Result<anyhow::Result<TempFile>, JoinError>) -> Option<TempFile> {
    res.unwrap().inspect_err(|e| error!("Skipping download reason={e}")).ok()   
}



