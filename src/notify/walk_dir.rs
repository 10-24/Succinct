use std::sync::Arc;
use async_walkdir::{DirEntry, Error, Filtering, WalkDir};
use futures::{FutureExt, Stream};


use crate::util::{arc_path::ArcPath, globs::Globs};


pub fn walk_dir(root: ArcPath, exclude: Arc<Globs>) -> impl Stream<Item = Result<DirEntry, Error>> {
    let filter_fn = move |entry| {
        let exclude = exclude.clone();
        async move { filter(&exclude, entry).await }.boxed()
    };
    WalkDir::new(root.as_std_path()).filter(filter_fn)
}


async fn filter(exclude: &Globs, entry: DirEntry) -> Filtering {
    if exclude.matches_std_path(entry.path()) {
        return Filtering::IgnoreDir;
    }
    
    match entry.file_type().await {
        Ok(t) if t.is_dir() => Filtering::Continue,
        _ => Filtering::Ignore,                     
    }
}