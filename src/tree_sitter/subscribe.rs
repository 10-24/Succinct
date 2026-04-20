use std::io;
use std::{future, iter, path::PathBuf, sync::Arc};

use async_walkdir::{DirEntry, Filtering, WalkDir};

use futures::{Stream, StreamExt, pin_mut, stream};
use smallvec::{SmallVec, smallvec};

use crate::db::tables::FILES;
use crate::tables;



use crate::{
    db::tables::file::{FileId, FileName},
    delta::{DeltaKind},
    path::{AbsPath, Local},
    state::file::{id::FileId, name::FileName},
    tree_sitter::tree_sitter::{TreeSitter},
};

impl TreeSitter {
    pub(crate) async fn subscribe_subtree(&mut self, start_path: AbsPath<Local>, start_id: FileId) -> io::Result<Vec<(FileId,WalkNode)>> {
        let subtree = self.stream_subtree(self.root.clone());
        pin_mut!(subtree);
        
        let start_prefix_len = start_path.len();
        let existing_files = tables!(&self.db, FILES);
        let mut new_files = Vec::new();
        
        while let Some(node) = subtree.next().await {
            debug_assert!(node.abs_path.starts_with(&start_path));
            let extra_components = node.abs_path.as_str()[start_prefix_len..]
                .split('/')
                .filter_map(FileName::from);
            let id = start_id.extend(extra_components);
            
            self.subscribe(id, &node.abs_path)?;
            
            let existing_file = existing_files.get(&id).unwrap();
            if existing_file.is_none() {
                new_files.push((id, node));
            }
        }
        Ok(new_files)
    }

    fn stream_subtree(&self, root_path: AbsPath<Local>) -> impl Stream<Item = WalkNode> {
        let root_node = WalkNode {
            abs_path: root_path,
            is_dir: true,
        };
        let ignore = self.ignore.clone();
        let root_prefix_len = self.root.len() + 1;
        let descendants = WalkDir::new(&root_node.abs_path)
            .filter(move |file| {
                let ignore = ignore.clone();
                async move {
                    let abs_path = file.path();
                    let abs_path = abs_path.to_str().unwrap();
                    let rel_path = &abs_path[root_prefix_len..];
                    match ignore.is_match(rel_path) {
                        true => Filtering::IgnoreDir,
                        false => Filtering::Continue,
                    }
                }
            })
            .filter_map(async |f| f.ok())
            .then(WalkNode::from_entry);

        stream::once(future::ready(root_node)).chain(descendants)
    }

    fn subscribe(&mut self, id: FileId, path: &AbsPath<Local>) -> io::Result<()> {
        let descriptor = self
            .inotify_watch_list
            .add(path, DeltaKind::WATCH_MASK)?
            .get_watch_descriptor_id();
        self.descriptors.insert(descriptor, id);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct WalkNode {
    pub abs_path: AbsPath<Local>,
    pub is_dir: bool,
}

impl WalkNode {
    pub async fn from_entry(entry: DirEntry) -> Self {
        WalkNode {
            abs_path: AbsPath::from_os_path(entry.path()),
            is_dir: entry.file_type().await.unwrap().is_dir(),
        }
    }
}
