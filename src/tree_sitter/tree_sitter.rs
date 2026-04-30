
use std::{collections::BTreeMap, sync::Arc};

use globset::GlobSet;
use inotify::{Inotify, Watches};
use nohash::IntMap;
use redb::{ReadableTable, ReadableTableMetadata};
use rustc_hash::{FxHashMap, FxHashSet};
use tokio::sync::{mpsc};

use crate::{
    db::{Db, tables::{FILES, file::{FileIdOrd, FileName, info::FileInfo}}}, delta::{Delta, DeltaData, DeltaReceiver, DeltaSender}, hashset, path::{AbsPath, RelPath}, tables, tree_sitter::{event::EventKV, subscribe::WalkNode}
};
use crate::db::tables::file::FileId;

pub struct TreeSitter {
    pub(crate) inotify_watch_list: Arc<std::sync::Mutex<Watches>>,
    pub(crate) descriptors: IntMap<i32, FileIdOrd>,
    pub(crate) root: AbsPath,
    pub(crate) ignore:Arc<GlobSet>,
    pub(crate) db: Db,
    pub(crate) output_tx: DeltaSender,
}

impl TreeSitter {
    
    pub fn start(root: AbsPath, ignore: GlobSet, db: Db) -> DeltaReceiver {
        
        let inotify = Inotify::init().unwrap();
        let inotify_watch_list = Arc::from(std::sync::Mutex::from(inotify.watches()));
        let ignore = Arc::from(ignore);
        
        let (output_tx, output_rx) = mpsc::channel(4);
        let mut tree_sitter = Box::from(Self {
            descriptors: FxHashMap::default(),
            inotify_watch_list,
            root: root.clone(),
            ignore,
            output_tx,
            db,
        });

        tokio::spawn(async move {
            tree_sitter.subscribe_root().await;
            tree_sitter.watch(inotify).await;
        });
        output_rx
    }

    async fn subscribe_root(&mut self) {
        let known_ids = self.all_known_ids();
        let new_files = self.subscribe_descendants(self.root.clone(), FileIdOrd::ROOT, &known_ids).await.unwrap();
        if new_files.is_empty() {
            return;
        }
        let deltas = new_files.into_iter().map(|(id,info)| (id, Delta::new_create(0, info))).collect();
        self.output_tx.send(deltas).await.unwrap();
    }
    
    pub fn get_id(&self,event:&EventKV) -> Option<FileIdOrd> {
        let parent_id = self.descriptors.get(&event.descriptor).unwrap();
        let child_name = event.name.as_ref().unwrap();
        let child_name = FileName::from_os_str(child_name)?;
        Some(parent_id.child(child_name))
    }
    
    fn all_known_ids(&self) -> FxHashSet<FileId> {
        let files_table = tables!(&self.db,FILES);
        let num_entries = files_table.len().unwrap() as usize;
        let mut known_ids = hashset(num_entries);
        for file in files_table.iter().unwrap() {
            let (id,_) = file.unwrap();
            known_ids.insert(id.value());
        }
        known_ids
    }
    
    pub fn convert_walk_node_to_file_info(&self, node: WalkNode) -> (FileIdOrd, FileInfo) {
        let (parent_id, child_id) = self.parent_child_id(&node.path);
        let file = FileInfo {
            is_dir: node.is_dir,
            name: node.path.file_name().unwrap().to_owned(),
            parent_id: *parent_id,
        };
        (child_id, file)
    }

    fn parent_child_id(&self, path: &AbsPath) -> (FileIdOrd, FileIdOrd) {
        let ext = path.as_rel(&self.root).unwrap();
        let mut ext_components = ext.components().map(|c| FileName::from(c).unwrap());
        let file_name = ext_components.next_back().unwrap();
        let parent_id = FileIdOrd::ROOT.extend(ext_components);
        let child_id = parent_id.child(file_name);
        (parent_id, child_id)
    }

}


