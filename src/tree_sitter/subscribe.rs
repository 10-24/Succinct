use std::path::PathBuf;

use compact_str::CompactString;
use futures::future::join_all;
use ignore::gitignore::Gitignore;
use tokio::{fs::{self}, io};

use crate::{
    config::{INTERNAL_ROOT_NAME, ROOT_ID, ROOT_PARENT_ID},
    delta::{DeltaKind, FileRecord},
    path::{AbsPath, Local}, tree_sitter::tree_sitter::{FileKV, TreeSitter},
};

impl TreeSitter {
    pub(crate) async fn subscribe_root(&mut self, root: AbsPath<Local>)  {
        let root = WalkNode::root(root);
        let subtree = Self::get_descendants(&root, &self.ignore).await.into_iter().flatten();
        
        for entry in subtree {
            self.subscribe(entry).unwrap();
        }
        self.subscribe(root).unwrap();
    }
    
    pub(crate) async fn subscribe_subtree(&mut self, start: WalkNode) -> io::Result<Vec<FileKV>> {
   
        let subtree = if start.is_dir {
            Some(Self::get_descendants(&start, &self.ignore).await.into_iter().flatten())
        } else {
            None
        }.into_iter().flatten();
        
        let mut new_files = Vec::new();
      
        for entry in subtree {
            new_files.push(entry.file.clone());
            self.subscribe(entry)?;
        }
        new_files.push(start.file.clone());
        self.subscribe(start)?;
        
        Ok(new_files)
    }
    
    async fn get_descendants(
        parent: &WalkNode,
        ignore: &Gitignore,
    ) -> Vec<Vec<WalkNode>> {
        let mut children = Vec::new();
        
        let mut dir = fs::read_dir(parent.abs_path.as_ref()).await.unwrap();

        while let Some(entry) = dir.next_entry().await.unwrap() {
            let entry_name = CompactString::from(entry.file_name().to_str().unwrap());
            let is_dir = entry.file_type().await.unwrap().is_dir();
            let child_node = WalkNode::new(parent, entry_name, entry.path(), is_dir);
       
            if child_node.abs_path.is_ignored(is_dir, ignore) {
                children.push(child_node);
            }
        }

        let descendants = children.iter().filter(|node| node.is_dir).map(|node| 
            Self::get_descendants(node, ignore)
        );
        let mut descendants: Vec<Vec<_>> = join_all(descendants).await.into_iter().flatten().collect();
        descendants.push(children);
        descendants
    }
    
    pub(crate) fn subscribe(&mut self, entry:WalkNode) -> io::Result<()>{
        let descriptor = self.inotify_watch_list.add(entry.abs_path.as_ref(), DeltaKind::WATCH_MASK)?.get_watch_descriptor_id();
        
        self.descriptors_to_file_ids.insert(descriptor, entry.file.id);
        self.file_ids_to_records.insert(*entry.file.id, entry.file.record);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct WalkNode {
    pub abs_path: AbsPath<Local>,
    pub is_dir: bool,
    pub file: FileKV,
}

impl WalkNode {
    pub(crate) fn new(
        parent: &WalkNode,
        name: CompactString,
        path: PathBuf,
        is_dir: bool,
    ) -> WalkNode {
        let id = parent.file.id.child(&name);
        let abs_path = AbsPath::from_os_path(path);
        let record = FileRecord {
            name,
            parent_id: *parent.file.id,
        };
        WalkNode {
            abs_path,
            is_dir,
            file: FileKV { id, record },
        }
    }
    pub fn root(abs_path:AbsPath<Local>) -> Self {
        WalkNode {
            abs_path,
            is_dir: true,
            file: FileKV {
                id: ROOT_ID,
                record: FileRecord {
                    name: INTERNAL_ROOT_NAME.into(),
                    parent_id: *ROOT_PARENT_ID,
                },
            },
        }
    }
 
}
