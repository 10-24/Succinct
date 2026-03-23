use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use futures::future::try_join_all;

use crate::{database::local_writer::LocalWriter, delta::{Delta, DeltaKind}, state::{file_id::{FileId, FileIdOrd}, state::State}};


impl State {
    
    pub(crate) async fn ensure_files_exist(&self, local_writer: &mut LocalWriter<'_>,deltas: &BTreeMap<FileIdOrd, Delta>, timestamp: DateTime<Utc>) -> sqlx::Result<()> {
        for (id, delta) in deltas.iter() {
            local_writer.ensure_file_exists(id, delta.file_name, timestamp, delta.parent_id).await?;
        }
        Ok(())
    }
    
    pub(crate) async fn populate_sparse_tree(&self, deltas: BTreeMap<FileIdOrd, Delta>) -> sqlx::Result<()> {
        let implicit_deltas = deltas.iter().map(|(id, delta)| self.get_implict_deltas(**id, delta));
        let implicit_deltas = try_join_all(implicit_deltas).await?.into_iter().flatten();
        // let implicit_delta_depths =
        let mut populated_deltas = BTreeMap::new();
        
        todo!()
    }
    
    async fn get_implict_deltas(&self, id: FileId, delta: &Delta) -> sqlx::Result<BTreeMap<FileIdOrd,>> {
        let ancestors = self.local_reader.get_file_ancestors(id).await?.into_iter();
        
        let mut implicit = ancestors.map(implicit_update(delta.ord));

        if delta.kind != DeltaKind::Delete {
            let descendants:Vec<_> = self.local_reader.get_file_descendants(id).await?;
            implicit.chain(descendants.into_iter().map(implicit_delete(delta.ord)));
        }
        Ok(implicit)
    }

    fn has_newer_version(deltas: &FxHashMap<FileId, Delta_>, entry: (FileId, &Delta_)) -> bool {
        deltas.get(&entry.0).is_some_and(|d| entry.1.ord <= d.ord)
    }
}
fn implicit_update(ord: u16) -> impl Fn(FileId) -> (FileId, Delta) {
    move |ancestor_id| {
        (
            ancestor_id,
            Delta {
                ord,
                kind: DeltaKind::Update,
            },
        )
    }
}

fn implicit_delete(ord: u16) -> impl Fn(FileId) -> (FileId, Delta_) {
    move |ancestor_id| {
        (
            ancestor_id,
            Delta_ {
                ord,
                kind: DeltaKind::Delete,
            },
        )
    }
}
