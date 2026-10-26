use std::{error::Error, io, os::fd::{AsFd, AsRawFd, OwnedFd}, sync::Arc};



use anyhow::Context as _;
use log::error;
use object_store::{GetResult, ObjectStoreExt as _, aws::AmazonS3, path::Path as ObjPath};

use tokio::{join, task::{JoinError, JoinSet}};
use crate::{context::Context, event_loop::{ignored_events::IgnoredEvents, pull_remote::{Delta, RemoteDeltas, temp_file::{self, TempFile}}}, notify::ephemeral_channel::EphemeralSender, util::arc_path::{ArcPath, RelPath}};



pub async fn write_deltas(ctx: &Context, remote_deltas: RemoteDeltas) -> RemoteDeltas {
    
    let mut set = JoinSet::new();
    for delta in remote_deltas {
        let obj_path = ObjPath::from(delta.path.to_string());
        let ctx_clone = ctx.clone();
        set.spawn(async move {
            (delta, download_file(&ctx_clone, obj_path).await)
        });
    }
    
    let mut successful_deltas = RemoteDeltas::default();
    
    while let Some(res) = set.join_next().await {
        if let Some((delta, tmp_file)) = handle_task_result(res) {
            tmp_file.save().unwrap();
            successful_deltas.push(delta);
        }
    }

    successful_deltas
}

async fn write_delta(ctx: &Context, delta: Delta, tmp_file: TempFile) -> anyhow::Result<()> {
    
}

async fn download_file(ctx: &Context, rel_path: RelPath) -> anyhow::Result<TempFile> {
    
    
    let local_path = ctx.config.sync_dir.join(&);
    let (get_result, tmp_file) = join!(
        ctx.object_store.get(&remote_path),
        TempFile::create(local_path)
    );
    let get_result = get_result.with_context(|| format!("failed to download `{remote_path}`"));
    let tmp_file = tmp_file.with_context(|| format!("failed to create temp file for `{remote_path}`"));
    
    tmp_file?.write_chunks(get_result?.into_stream()).await
}





fn handle_task_result(res: Result<(Delta, anyhow::Result<TempFile>), JoinError>) -> Option<(Delta, TempFile)> {
    let (delta, tmp_file) = res.unwrap();
    let tmp_file = tmp_file.inspect_err(|e| error!("Skipping download reason={e}")).ok()?;
    Some((delta, tmp_file))
}



async fn delete_locally(ctx: &Context, paths: impl Iterator<Item = RelPath>) -> io::Result<()> {
    
    let abs_paths = paths.map(|p| ctx.paths.from_sync_dir(&p));
    
    todo!()
}