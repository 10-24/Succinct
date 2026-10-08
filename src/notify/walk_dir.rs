
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use async_walkdir::{DirEntry, Filtering, WalkDir};
use futures::{FutureExt, Stream, TryStreamExt};
use futures::future::{BoxFuture, join_all};
use globset::GlobSet;
use tokio::task::JoinSet;
use tokio::{join, task};
use tokio::{io, sync::mpsc};
use tokio_stream::StreamExt;


use crate::notify::walk_dir;
use crate::util::apath::{APath, buf_into_apath, into_apath, path_to_apath};


type StreamEntry = async_walkdir::Result<APath>;
pub fn walk_dir(root: APath, exclude: Arc<GlobSet>) -> impl Stream<Item = StreamEntry> {
    let filter_fn = move |entry| {
        let exclude = exclude.clone();
        async move { filter(&exclude, entry).await }.boxed()
    };
    WalkDir::new(root.as_std_path()).filter(filter_fn).map_ok(|entry| buf_into_apath(entry.path()))
}





async fn filter(exclude: &GlobSet, entry: DirEntry) -> Filtering {
    if exclude.is_match(entry.path()) {
        return Filtering::IgnoreDir;
    }
    match entry.file_type().await {
        Ok(t) if t.is_dir() => Filtering::Continue,
        _ => Filtering::Ignore,                     
    }
}