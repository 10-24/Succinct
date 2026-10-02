use std::fs::ReadDir;
use std::sync::Arc;
use std::{fs::DirEntry, path::Path};
use std::os::unix::fs::DirEntryExt;
use futures::future::join_all;
use globset::GlobSet;
use tokio::{join, task};
use tokio::{io, sync::mpsc};



async fn walk_dir(dir_path: Arc<Path>, exclude: &GlobSet, output: mpsc::Sender<Arc<Path>>) -> io::Result<()> {
    
    let (_, children) = join!(output.send(dir_path.clone()), read_dir_async(dir_path));
   
    let child_dirs = children?.filter_map(Result::ok).filter_map(|child| {
        let path = child.path().into();
        if child.file_type().unwrap().is_file() {
            return None
        }
        if exclude.is_match(&path) {
            return None;
        }
        Some(path)
    });

    join_all(child_dirs.map(|path| walk_dir(path, exclude, output.clone()))).await.into_iter().collect()
}



async fn read_dir_async(path: Arc<Path>) -> std::io::Result<ReadDir> {
    task::spawn_blocking(|| {
        std::fs::read_dir(path)
    }).await.unwrap()
}