use std::io;
use std::{future, iter, path::PathBuf, sync::Arc};

use async_walkdir::{DirEntry, Filtering, WalkDir};

use futures::{Stream, StreamExt, pin_mut, stream};
use globset::{GlobMatcher, GlobSet};
use rustc_hash::FxHashSet;
use smallvec::{SmallVec, smallvec};
use tokio::task;

use crate::db::tables::FILES;
use crate::db::tables::file::info::FileInfo;
use crate::db::tables::file::{FileIdOrd, FileName};
use crate::tables;



use crate::{
    db::tables::file::{FileId, FileNameBuf},
    delta::{DeltaKind},
    path::{AbsPath},
    state::file::{id::FileId, name::FileName},
    tree_sitter::tree_sitter::{TreeSitter},
};

impl TreeSitter {
    pub(crate) async fn subscribe_descendants(&mut self, start_path: AbsPath, start_id: FileIdOrd, known_ids: &FxHashSet<FileId>) -> io::Result<Vec<(FileIdOrd,FileInfo)>> {
        let subtree = Self::stream_descendants(start_path.clone(),self.ignore.clone(), self.root.clone());
        pin_mut!(subtree);
    
        let mut new_files = Vec::new();
        
        while let Some(node) = subtree.next().await {
            let ext = node.path.as_str().strip_prefix(self.root.as_str()).unwrap();
            let ext_components = ext.split('/').map(|c| FileName::from(c).unwrap());
            let child_id = start_id.extend(ext_components);
            self.subscribe(*child_id, node.path.to_owned()).await?;
            
            if known_ids.contains(&child_id) {
               continue; 
            }
            let file = self.convert_walk_node_to_file_info(node);
            new_files.push(file);
        }
        Ok(new_files)
    }

    fn stream_descendants(start_path: AbsPath,ignore: Arc<GlobSet>,global_root: AbsPath) -> impl Stream<Item = WalkNode> {
        let root_node = WalkNode {
            path: start_path,
            is_dir: true,
        };
       
        let global_root_prefix_len = global_root.len() + 1;
        let descendants = WalkDir::new(&root_node.path)
            .filter(move |file| {
                let ignore = ignore.clone();
                async move {
                    let abs_path = file.path();
                    let abs_path = abs_path.to_str().unwrap();
                    let rel_path = &abs_path[global_root_prefix_len..];
                    match ignore.is_match(rel_path) {
                        true => Filtering::IgnoreDir,
                        false => Filtering::Continue,
                    }
                }
            })
            .filter_map(async |f| f.ok())
            .then(WalkNode::from_entry);

        descendants
    }

    pub async fn subscribe(&mut self, id: FileId, path: AbsPath) -> io::Result<()> {
        let watch_list_handle = Arc::clone(&self.inotify_watch_list);
        let descriptor = task::spawn_blocking(move || {
            let mut list = watch_list_handle.lock().unwrap();
            list.add(path, DeltaKind::WATCH_MASK)
                .map(|d| d.get_watch_descriptor_id())
        })
        .await 
        .expect("The background thread panicked")?;
    
        self.descriptors.insert(descriptor, id);
        Ok(())
    }
    
    
 
}

#[derive(Debug, Clone)]
pub struct WalkNode {
    pub path: AbsPath,
    pub is_dir: bool,
}

impl WalkNode {
    pub async fn from_entry(entry: DirEntry) -> Self {
        WalkNode {
            path: AbsPath::from_os_path(entry.path()),
            is_dir: entry.file_type().await.unwrap().is_dir(),
        }
    }
}


