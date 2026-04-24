use rustc_hash::FxHashSet;

use crate::{
    db::tables::file::{FileIdOrd, FileName, info::FileInfo},
    delta::{Delta, DeltaData, DeltaKV},
    path::AbsPath,
    tree_sitter::{event::EventKV, subscribe::WalkNode, tree_sitter::TreeSitter},
};

impl TreeSitter {
    
    pub async fn handle_create(&mut self, event: EventKV, index: u16) -> Vec<DeltaKV> {
        let Some((id, info, path)) = self.try_create_child(&event) else {
            return vec![];
        };
        self.subscribe(*id, path.to_owned()).await.unwrap();
        
        let data = DeltaData::Create(info);
        let mut result = vec![(id, Delta{data,index})];
        if info.is_dir {
            return result;
        }

        let descendants = self
            .subscribe_descendants(path, id, &FxHashSet::default())
            .await
            .unwrap()
            .into_iter()
            .map(|(id,file)| {
                let data = DeltaData::Create(file);
                (id, Delta { data, index })
            });
        result.extend(descendants);
        result
    }

    fn try_create_child(&mut self, event: &EventKV) -> Option<(FileIdOrd, FileInfo, AbsPath)> {
        let parent_id = *self.descriptors.get(&event.descriptor).unwrap();
        let parent_rel_path = self.db.get_file_path(parent_id);
        let parent_id = parent_id.into_ord(parent_rel_path.depth());
        let name = FileName::from_os_str(&event.name.unwrap())?.to_owned();
        let path = parent_rel_path.child(&name);

        if self.ignore.is_match(path.as_str()) {
            return None;
        }

        let id = parent_id.child(&name);
        let info = FileInfo {
            name,
            parent_id: *parent_id,
            is_dir: event.value.is_dir,
        };
        let path = self.root.join(&path);
        Some((id, info, path))
    }

}
