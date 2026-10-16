use std::sync::Arc;
use async_walkdir::{DirEntry, Error, Filtering, WalkDir};
use futures::{FutureExt, Stream};
use globset::GlobSet;


use crate::util::arc_path::ArcPath;




pub fn walk_dir(root: ArcPath, exclude: Arc<GlobSet>) -> impl Stream<Item = Result<DirEntry, Error>> {
    let filter_fn = move |entry| {
        let exclude = exclude.clone();
        async move { filter(&exclude, entry).await }.boxed()
    };
    WalkDir::new(root.as_std_path()).filter(filter_fn)
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