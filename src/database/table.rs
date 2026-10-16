use crate::database::table_trait::TableDef;


#[derive(Debug,  Clone, Copy, redb_derive::Value)]
pub struct Entry {
    modified_at: i64,
    is_deleted: bool
}

impl Entry {
    pub fn new(modified_at: i64, is_deleted: bool) -> Self {
        Self { modified_at, is_deleted }
    }

    pub fn is_newer(&self, other:&Self ) -> bool {
        self.modified_at > other.modified_at
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }
}

pub struct StagedDeltas;
impl TableDef for StagedDeltas {
    const NAME:&str = "staged_deltas";
    type Key = &'static str;
    type Value = Entry;
}

pub struct State;
impl TableDef for State {
    const NAME:&str = "state";
    type Key = &'static str;
    type Value = Entry;
}