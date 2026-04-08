use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use futures::future::try_join_all;

use crate::{
    database::local_writer::LocalWriter,
    delta::{Delta, DeltaKind},
    state::{
        file::File,
        file_id::{FileId, FileIdOrd},
        state::State,
    },
};

impl State {
    pub(crate) async fn ensure_files_exist(
        &self,
        local_writer: &mut LocalWriter<'_>,
        deltas: &BTreeMap<FileIdOrd, Delta>,
        timestamp: DateTime<Utc>,
    ) -> sqlx::Result<()> {
        for (id, delta) in deltas.iter() { 
            let file = File::empty(delta.file.name.to_owned(), id.depth, timestamp, delta.file.parent_id);
            local_writer.ensure_file_exists(file).await?;
        }
        Ok(())
    }

    pub(crate) async fn complete_tree(
        &self,
        explicit_deltas: BTreeMap<FileIdOrd, Delta>,
    ) -> sqlx::Result<FinalDeltas> {
        let implicit_deltas = explicit_deltas
            .iter()
            .map(|(id, delta)| self.get_implict_deltas(*id, delta));
        let implicit_deltas = try_join_all(implicit_deltas).await?.into_iter().flatten();
        
        let explicit_deltas = explicit_deltas
            .into_iter()
            .map(|(id, delta)| (id, DeltaValue::from(&delta)));

        let mut deltas = BTreeMap::new();

        for delta in explicit_deltas.chain(implicit_deltas) {
            try_insert(&mut deltas, delta);
        }
        Ok(deltas)
    }

    async fn get_implict_deltas(
        &self,
        id: FileIdOrd,
        file_id: &FileIdOrd,
        delta: &Delta,
    ) -> sqlx::Result<Vec<(FileIdOrd, DeltaValue)>> {
        let ancestors = self.local_reader.get_file_ancestors(*id).await?.ok_or(sqlx::Error::RowNotFound)?;
        let updated_ancestors = ancestors.map(implicit_update(file_id.ord));
       
        if delta.1.kind != DeltaKind::Delete {
            return Ok(updated_ancestors.collect());
        }
        let deleted_descendants = self.local_reader.get_file_descendants(*id).await?.into_iter().map(implicit_delete(delta.ord));
        
        Ok(updated_ancestors.chain(deleted_descendants).collect())
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
            ord: delta.ord,
        }
    }
}

pub type DeltaKV = (FileIdOrd, DeltaValue);
type FullDeltaKV = (FileIdOrd, Delta);
