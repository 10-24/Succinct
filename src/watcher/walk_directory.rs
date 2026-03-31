use futures::future::join_all;
use ignore::gitignore::Gitignore;
use tokio::fs::{self, DirEntry};

use crate::{config::{ROOT_ID, ROOT_PARENT_ID}, path::{AbsPath, Local}, state::file_id::{FileId, FileIdOrd}};



pub async fn walk_directory(root_path: AbsPath<Local>, ignore: &Gitignore) -> impl Iterator<Item = WalkNode> {
    let root_node = WalkNode {
        id: ROOT_ID,
        path: root_path,
        is_dir: true,
        parent_id: *ROOT_PARENT_ID,
    };
    let descendants = get_descendants(&root_node, &ignore).await;
    vec![vec![root_node]].into_iter().chain(descendants).flatten() 
}

async fn get_descendants(parent: &WalkNode, ignore: &Gitignore) -> Vec<Vec<WalkNode>> {
    
    let mut children = Vec::new();

    let mut dir = fs::read_dir(parent.path.as_ref()).await.unwrap();
    
    while let Some(entry) = dir.next_entry().await.unwrap() {
        
        let is_dir = entry.file_type().await.unwrap().is_dir();
        if !is_ignored(&entry, is_dir, ignore) {
            continue;
        }

        children.push(WalkNode {
            id: parent.id.child(entry.file_name().to_str().unwrap()),
            path: AbsPath::from_os_path(entry.path()),
            parent_id: *parent.id,
            is_dir,
        });
    }
    let descendants = children.iter().filter(|node| node.is_dir).map(|node| get_descendants(node, ignore));
    let mut descendants: Vec<Vec<WalkNode>> = join_all(descendants).await.into_iter().flatten().collect();
    descendants.push(children);
    descendants
}



fn is_ignored(entry: &DirEntry, is_dir: bool, ignore: &Gitignore) -> bool {
    ignore.matched(entry.path(), is_dir).is_ignore()
}

#[derive(Debug,Clone)]
pub struct WalkNode {
    pub id: FileIdOrd,
    pub path: AbsPath<Local>,
    pub is_dir: bool,
    pub parent_id:FileId,
}