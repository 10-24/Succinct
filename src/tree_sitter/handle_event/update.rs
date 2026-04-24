use crate::{delta::{Delta, DeltaData, DeltaKV}, tree_sitter::{event::EventKV, tree_sitter::TreeSitter}};

impl TreeSitter {
    pub fn handle_update(&self,event:EventKV,index:u16) -> Option<DeltaKV> {
        let id = self.get_id(&event)?;
        let delta = Delta {
            index,
            data: DeltaData::Update,
        };
        Some((id,delta))
    }
}