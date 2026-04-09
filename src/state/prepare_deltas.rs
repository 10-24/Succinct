use std::collections::BTreeMap;

use chrono::{DateTime, Utc};

use crate::{
    database::local_writer::DbWriter,
    delta::{Delta, DeltaKind},
    state::{file::File, file_id::FileIdOrd, state::State},
};

impl State {
    pub(crate) fn ensure_files_exist(
        &self,
        deltas: &BTreeMap<FileIdOrd, Delta>,
        timestamp: DateTime<Utc>,
    ) {
        for (id, delta) in deltas.iter() {
            let file = File::empty(
                delta.file.name.to_owned(),
                id.depth,
                timestamp,
                delta.file.parent_id,
            );
            self.local_writer.ensure_file_exists(file);
        }
    }

    pub(crate) fn complete_tree(&self, explicit_deltas: BTreeMap<FileIdOrd, Delta>) -> FinalDeltas {
        let implicit_deltas: Vec<Vec<(FileIdOrd, DeltaValue)>> = explicit_deltas
            .iter()
            .map(|(id, delta)| self.get_implicit_deltas(*id, delta))
            .collect();

        let explicit_deltas = explicit_deltas
            .into_iter()
            .map(|(id, delta)| (id, DeltaValue::from(&delta)));

        let mut deltas = BTreeMap::new();

        for delta in explicit_deltas.chain(implicit_deltas.into_iter().flatten()) {
            try_insert(&mut deltas, delta);
        }

        deltas
    }

    fn get_implicit_deltas(&self, id: FileIdOrd, delta: &Delta) -> Vec<(FileIdOrd, DeltaValue)> {
        let Some(ancestors) = self.local_reader.get_file_ancestors(*id) else {
            return Vec::new();
        };

        let updated_ancestors = ancestors.into_iter().map(implicit_update(delta.index));

        if delta.kind != DeltaKind::Delete {
            return updated_ancestors.collect();
        }

        let deleted_descendants = self
            .local_reader
            .get_file_descendants(*id)
            .into_iter()
            .map(implicit_delete(delta.index));

        updated_ancestors.chain(deleted_descendants).collect()
    }
}

fn implicit_update(ord: u16) -> impl Fn(FileIdOrd) -> (FileIdOrd, DeltaValue) {
    move |ancestor_id| {
        (
            ancestor_id,
            DeltaValue {
                ord,
                kind: DeltaKind::Update,
            },
        )
    }
}

fn implicit_delete(ord: u16) -> impl Fn(FileIdOrd) -> (FileIdOrd, DeltaValue) {
    move |ancestor_id| {
        (
            ancestor_id,
            DeltaValue {
                ord,
                kind: DeltaKind::Delete,
            },
        )
    }
}

fn try_insert(deltas: &mut FinalDeltas, delta: DeltaKV) {
    let prev_entry = deltas.entry(delta.0);
    prev_entry
        .and_modify(|prev_entry| {
            if delta.1.ord > prev_entry.ord {
                *prev_entry = delta.1;
            }
        })
        .or_insert(delta.1);
}

pub type FinalDeltas = BTreeMap<FileIdOrd, DeltaValue>;

#[derive(Debug, Clone, Copy)]
pub struct DeltaValue {
    pub kind: DeltaKind,
    pub ord: u16,
}

impl DeltaValue {
    pub fn from(delta: &Delta) -> Self {
        Self {
            kind: delta.kind,
            ord: delta.index,
        }
    }
}

pub type DeltaKV = (FileIdOrd, DeltaValue);
type FullDeltaKV = (FileIdOrd, Delta);
