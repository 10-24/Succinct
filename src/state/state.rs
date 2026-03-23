
use crate::database::file_id::FileId;

use crate::state::file_id::FileIdOrd;
use crate::{
    database::{QueuedDeletion, local_reader::LocalReader, local_writer::LocalWriter},
    delta::{Delta, DeltaKind},
    path::{AbsPath, Local, Remote},
    state::file::{File, },
};
use chrono::{DateTime, Utc};
use futures::future::try_join_all;
use opendal::Operator;
use rustc_hash::FxHashMap;
use sqlx::{Connection, PgConnection, PgPool, SqlitePool};
use std::collections::BTreeMap;
use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap},
};
use tokio::try_join;

pub struct State {
    pub(crate) write_pool: SqlitePool,
    pub(crate) local_reader: LocalReader,
    pub(crate) remote_db: PgPool,
    pub(crate) remote_drive: Operator,
    pub(crate) local_root: AbsPath<Local>,
    pub(crate) remote_root: AbsPath<Remote>,
}

impl State {
    
    
    /// File and Folder creations cannot be implicit (Eg. If a parent and child file/folder are created, both must be included)
    pub async fn push_deltas(&mut self, mut deltas: BTreeMap<FileIdOrd,Delta>) -> sqlx::Result<()> {
        let timestamp = Utc::now();
        let mut txn = self.write_pool.begin().await?.into();
        self.ensure_files_exist(&mut txn, &deltas, timestamp);
        self.populate_sparse_tree(deltas).await?;
        
        while let Some(delta) = deltas.pop_last() {

            match delta.kind {
                DeltaKind::Update => {
                    self.handle_update(&mut txn, delta, timestamp).await?;
                }
                DeltaKind::Delete => {
                    self.handle_delete(&mut txn, delta, timestamp).await?;
                }
            }
        }

        txn.commit().await?;
        Ok(())
    }

    async fn handle_update(
        &self,
        local_writer: &mut LocalWriter<'_>,
        delta: Delta,
        timestamp: DateTime<Utc>,
    ) -> sqlx::Result<()> {
        let file_id = delta.file_id;

        let child_hashes = self.local_reader.get_file_child_hashes(file_id).await?;
        let file = File::new(delta.file_name, delta.file_id, timestamp, child_hashes);

        local_writer.modify_file(file.to_owned()).await?;
        local_writer
            .enqueue_update(QueuedUpdate {
                file_id,
                depth: delta.depth,
            })
            .await?;

        Ok(())
    }

    async fn handle_delete(
        &self,
        local_writer: &mut LocalWriter<'_>,
        delta: Delta,
        deleted_at: DateTime<Utc>,
    ) -> sqlx::Result<()> {
        let file_id = delta.file_id;

        let descendants: Vec<FileId> = self.local_reader.get_file_descendants(file_id).await?;
        let subtree = descendants.into_iter().chain(Some(file_id));

        for id in subtree {
            let record = QueuedDeletion {
                file_id: id,
                deleted_at,
            };

            local_writer.delete_file(record.file_id).await?;
            local_writer.enqueue_delete(record).await?;
        }

        Ok(())
    } 
}

fn floor_mul(x: usize, y: f32) -> usize {
    let x = x as f32;
    let prod = x * y;
    prod as usize
}


struct FullTrace {
    id: FileId,
    ancestors: Vec<FileId>,
    descendants: Vec<FileId>,
}

