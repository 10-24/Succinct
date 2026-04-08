
use crate::database::QueuedUpdate;
use crate::database::file_id::FileId;

use crate::state::file_id::FileIdOrd;
use crate::state::prepare_deltas::DeltaKV;
use crate::state::remote_drive::RemoteDrive;
use crate::{
    database::{QueuedDeletion, local_reader::LocalReader, local_writer::LocalWriter},
    delta::{Delta, DeltaKind},
    path::{AbsPath, Local},
    state::file::{File, },
};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, SqlitePool};
use std::collections::BTreeMap;


pub struct State {
    pub(crate) write_pool: SqlitePool,
    pub(crate) local_reader: LocalReader,
    // pub(crate) remote_db: PgPool,
    pub(crate) remote_drive: opendal::Operator,
    pub(crate) local_root: AbsPath<Local>,
}

impl State {
    pub fn new(write_pool: SqlitePool, local_reader: LocalReader, remote_drive: opendal::Operator, local_root: AbsPath<Local>) -> Self {
        Self {
            write_pool,
            local_reader,
            remote_drive,
            local_root,
        }
    }
    
    /// File and Folder creations cannot be implicit (Eg. If a parent and child file/folder are created, both must be included)
    pub async fn push_deltas(&mut self, deltas: BTreeMap<FileIdOrd,Delta>) -> sqlx::Result<()> {
        let timestamp = Utc::now();
        
        let mut init_files_txn = self.write_pool.begin().await?.into();
        self.ensure_files_exist(&mut init_files_txn, &deltas, timestamp).await?;
        init_files_txn.txn.commit().await?;
        
        let mut deltas = self.complete_tree(deltas).await?;
        let mut update_files_txn = self.write_pool.begin().await?.into();
        while let Some(delta) = deltas.pop_last() {
            match delta.1.kind {
                DeltaKind::Update | DeltaKind::Create => self.handle_modify(&mut update_files_txn, delta, timestamp).await?,
                DeltaKind::Delete => self.handle_delete(&mut update_files_txn, delta, timestamp).await?,
            }
        }
        update_files_txn.txn.commit().await
    }
    

    /// Handles both Update and Create
    async fn handle_modify(
        &self,
        local_writer: &mut LocalWriter<'_>,
        delta: DeltaKV,
        modified_at: DateTime<Utc>,
    ) -> sqlx::Result<()> {
        let file_id = delta.0.value;

        let child_hashes = self.local_reader.get_file_child_hashes(file_id).await?;
        let file_hash = File::calculate_hash(file_id, modified_at, child_hashes);
        local_writer.update_file(file_id, file_hash, modified_at).await;
        
        let queued_update = QueuedUpdate {
            file_id,
            depth: delta.0.depth,
        };
        local_writer
            .enqueue_update(queued_update)
            .await?;

        Ok(())
    }

    async fn handle_delete(
        &self,
        local_writer: &mut LocalWriter<'_>,
        delta: DeltaKV,
        deleted_at: DateTime<Utc>,
    ) -> sqlx::Result<()> {
        let file_id = delta.0.value;

        let descendants = self.local_reader.get_file_descendants(file_id).await?;
        let subtree = descendants.into_iter().chain(Some(delta.0));

        for file_id in subtree {
            local_writer.delete_file(*file_id).await?;
            
            let queued_deletion = QueuedDeletion {
                deleted_at,
                file_id: *file_id,
            };
            local_writer.enqueue_delete(queued_deletion).await?;
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

