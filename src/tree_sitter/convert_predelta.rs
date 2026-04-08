use crate::{delta::{Delta, DeltaKind}, path::{AbsPath, Local}, state::file_id::FileIdOrd, tree_sitter::{subscribe::WalkNode, tree_sitter::{FileKV, PredeltaKV, TreeSitter, WatchDescriptor}}};

impl TreeSitter {
    
    pub(crate) async fn convert_predelta(&mut self, predelta: PredeltaKV, index: u16) -> impl Iterator<Item = (FileIdOrd,Delta)> {
        let kind = predelta.value.kind;
        let files = match kind {
            DeltaKind::Update => vec![self.get_file(predelta.descriptor).to_owned()],
            DeltaKind::Create => self.handle_create(predelta).await,
            DeltaKind::Delete => self.handle_delete(predelta),
        };
        files.into_iter().map(FileKV::into_delta_fn(kind,index))
    }

    async fn handle_create(&mut self, predelta: PredeltaKV) -> Vec<FileKV> {
        let parent_file = self.get_file(predelta.descriptor);
        let child_file = FileKV::from(parent_file.id, predelta.value.name.unwrap());

        let parent_path = self
            .db
            .get_file_path(*parent_file.id)
            .await
            .unwrap()
            .unwrap();
        let child_path = self.root.join(&parent_path.child(&child_file.record.name));

        if child_path.is_ignored(predelta.value.is_dir, &self.ignore) {
            return vec![];
        }
        
        self.add_new_subtree(child_path, child_file, predelta.value.is_dir).await
    }

    fn handle_delete(&mut self, predelta: PredeltaKV) -> Vec<FileKV> {
        let parent_file = self.get_file(predelta.descriptor);
        let child_file = FileKV::from(parent_file.id, predelta.value.name.unwrap());

        self.file_ids_to_records.remove(&*child_file.id); // Leaks the descriptor

        vec![child_file]
    }

    fn get_file(&self, descriptor: WatchDescriptor) -> FileKV {
        let id = self
            .descriptors_to_file_ids
            .get(&descriptor)
            .unwrap()
            .to_owned();
        let record = self.file_ids_to_records.get(&*id).unwrap().to_owned();
        FileKV { id, record }
    }
    
    async fn add_new_subtree(&mut self, abs_path: AbsPath<Local>, file:FileKV, is_dir:bool) -> Vec<FileKV> {
        let node = WalkNode {
            abs_path,
            is_dir,
            file:file.clone(),
        };
        
        let watch_res = self
            .subscribe_subtree(node)
            .await;
        if let Ok(files) = watch_res {
            return files;
        };
        eprintln!("Failed to watch entry: {:?}", watch_res);
        return vec![file];
    }
}