use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use derive_more::Deref;

use crate::{
    db::{
        tables::{
            CHILDREN, ChildrenTable, ChildrenTableMut, FILES, FilesTable, FilesTableMut,
            file::{File, FileIdOrd},
            timestamp::Timestamp,
        },
        writer::DbWriter,
    },
    delta::{Delta, DeltaData, DeltaKind},
    state::{file::File, file_id::FileIdOrd, state::State},
    tables_mut,
};

impl State {
    pub(crate) fn instantiate_files_and_complete_tree(
        &self,
        deltas: BTreeMap<FileIdOrd, Delta>,
        now: Timestamp,
        files_table: &mut FilesTableMut,
        children_table: &mut ChildrenTableMut,
    ) -> CompleteDeltas {
        self.instantiate_new_files(&deltas, now, files_table, children_table);
        self.complete_tree(deltas)
    }

    fn instantiate_new_files(
        &self,
        deltas: &BTreeMap<FileIdOrd, Delta>,
        now: Timestamp,
        files_table: &mut FilesTableMut,
        children_table: &mut ChildrenTableMut,
    ) {
        for (id, delta) in deltas.iter() {
            let DeltaData::Create(file_info) = delta.data else {
                continue;
            };
            let file = File::empty(*id, file_info, now);
            files_table.insert(**id, file).unwrap();
            children_table.insert(file_info.parent_id, **id).unwrap();
        }
    }

    pub(crate) fn complete_tree(
        &self,
        explicit_deltas: BTreeMap<FileIdOrd, Delta>,
    ) -> CompleteDeltas {
        let mut complete_deltas = CompleteDeltas(BTreeMap::new());

        for (id, explicit_delta) in explicit_deltas.iter() {
            let explicit_delta = DeltaValue::from(explicit_delta);
            complete_deltas.try_insert(*id, explicit_delta);

            for (id, implicit_delta) in self.get_implicit_deltas(*id, explicit_delta) {
                complete_deltas.try_insert(id, implicit_delta);
            }
        }

        complete_deltas
    }

    fn get_implicit_deltas(
        &self,
        id: FileIdOrd,
        delta: DeltaValue,
    ) -> Vec<(FileIdOrd, DeltaValue)> {
        let ancestors = self
            .local_db
            .get_file_ancestors(*id)
            .into_iter()
            .map(into_implicit_update(delta.index));
        if DeltaKind::Delete != delta.kind {
            return ancestors.collect();
        };

        let deleted_descendants = self
            .local_db
            .get_file_descendants(id)
            .into_iter()
            .map(into_implicit_delete(delta.index));
        ancestors.chain(deleted_descendants).collect()
    }
}

fn into_implicit_update(index: u16) -> impl Fn(FileIdOrd) -> (FileIdOrd, DeltaValue) {
    move |ancestor_id| {
        let delta = DeltaValue {
            index,
            kind: DeltaKind::Update,
        };
        (ancestor_id, delta)
    }
}

fn into_implicit_delete(index: u16) -> impl Fn(FileIdOrd) -> (FileIdOrd, DeltaValue) {
    move |ancestor_id| {
        let delta = DeltaValue {
            index,
            kind: DeltaKind::Delete,
        };
        (ancestor_id, delta)
    }
}

/// Proof: All new files have been initialized & all implicit updates have been included.
#[derive(Debug, Deref)]
pub struct CompleteDeltas(BTreeMap<FileIdOrd, DeltaValue>);

impl CompleteDeltas {
    fn try_insert(&mut self, id: FileIdOrd, delta: DeltaValue) {
        self.0
            .entry(id)
            .and_modify(|prev_entry| {
                if delta.index > prev_entry.index {
                    *prev_entry = delta;
                }
            })
            .or_insert(delta);
    }
}
#[derive(Debug, Clone, Copy)]
pub struct DeltaValue {
    pub kind: DeltaKind,
    pub index: u16,
}

impl DeltaValue {
    pub fn from(delta: &Delta) -> Self {
        Self {
            kind: delta.data.kind(),
            index: delta.index,
        }
    }
}

pub type DeltaKV = (FileIdOrd, DeltaValue);
type FullDeltaKV = (FileIdOrd, Delta);
