use colored::Colorize;
use smallvec::{SmallVec, smallvec};

use crate::{
    db::tables::file::{FileName, info::FileInfo}, delta::{Delta, DeltaKV, DeltaKind}, path::{AbsPath, Local}, state::file_id::FileIdOrd, tree_sitter::{
        event::{EventKV, WatchDescriptor},
        subscribe::WalkNode,
        tree_sitter::{FileKV, TreeSitter},
    }
};

impl TreeSitter {
    pub(crate) async fn convert_predelta(
        &mut self,
        event: EventKV,
        index: u16,
    ) -> SmallVec<[DeltaKV;2]> {
        let kind = event.value.kind;
        let files = match kind {
            DeltaKind::Update => self.handle_update(event),
            DeltaKind::Create => self.handle_create(event).await,
            DeltaKind::Delete => self.handle_delete(event),
        };
  
    }
    
    async fn handle_create(&mut self, event: EventKV) -> SmallVec<[DeltaKV;2]> {
        
        let parent_id = *self.descriptors.get(&event.descriptor).unwrap();
        let parent_rel_path = self.db.get_file_path(parent_id);
        let name = event.name.unwrap().to_str().unwrap();
        let rel_path = &parent_rel_path.child(name);
        if self.ignore.is_match(rel_path.as_str()) {
            return smallvec![];
        }
        let id = parent_id.child(&name);
        let abs_path = self.root.join(rel_path);
        let Some(name) = FileName::from(name) else {
            panic!("Unignored file has invalid name: {}",name);
        };
        
        let new_subtree_root = WalkNode {
            abs_path,
            id,
            info: FileInfo {
                is_dir: event.value.is_dir,
                name,
                parent_id,
            }
        };


        let files = match self.subscribe_new_subtree(&new_subtree_root).await {
            Ok(files) => files,
            Err(e) => {
                eprintln!("Failed to watch entry: {:?}", e);
                smallvec![new_subtree_root.info]
            }
        };
        files.into_iter().map(|f| ())
    }
    fn handle_update(&mut self, event: EventKV) -> Vec<DeltaKV> {
        
    }
    fn handle_delete(&mut self, event: EventKV) -> Vec<FileKV> {
        let parent_file = self.get_file(event.descriptor);
        let child_file = FileKV::from(parent_file.id, event.value.name.unwrap());

      
        vec![child_file]
    }

    fn get_file(&self, descriptor: WatchDescriptor) -> FileKV {
        let id = self.descriptors.get(&descriptor).unwrap().to_owned();
        let record = self.file_ids_to_records.get(&*id).unwrap().to_owned();
        FileKV { id, record }
    }

}
