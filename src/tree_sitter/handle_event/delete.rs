use crate::{db::tables::file::FileIdOrd, delta::{Delta, DeltaData, DeltaKV}, tree_sitter::{event::EventKV, tree_sitter::TreeSitter}};

impl TreeSitter {
    pub fn handle_delete(&mut self,event: EventKV,index:u16) -> Vec<DeltaKV> {
        
        let Some(id) = self.get_id(&event) else {
            return vec![];
        };
        
        let Some(is_dir) = self.db.is_dir(*id) else {
            return vec![]; // Skip if file doesn't exist
        };
         
        let mut result = vec![delete_delta(id, index)];
        if !is_dir {
            return result;
        }
        
        let descendant_deltas = self.db.get_file_descendants(id).into_iter().map(|d_id| delete_delta(d_id, index));
        result.extend(descendant_deltas);
        result
    }
}
fn delete_delta(id:FileIdOrd,index:u16) -> DeltaKV {
    let delta = Delta{index, data: DeltaData::Delete };
    (id,delta)
}