use crate::{
    delta::{Delta, DeltaKind},
    path::{AbsPath, Path},
    state::{
        file::File,
        local::{self, QueuedDeletion, QueuedUpdate},
    },
};
use chrono::{DateTime, Utc};
use opendal::{Operator, services::Gdrive};
use sqlx::{Connection, PgConnection, Sqlite, SqliteConnection, Transaction};
use std::{cell::RefCell, collections::BTreeSet};

pub struct State {
    pub(crate) local_root: AbsPath,
    local_db: RefCell<SqliteConnection>,
    remote_db: RefCell<PgConnection>,

    pub(crate) remote_drive: Operator,
}

impl State {
    // pub async fn new() -> Result<Self> {
    //     // Ok(Self {
    //         local: SqlitePool::connect(env!("DATABASE_URL")).await?,
    //         remote: PgPool::connect(env!("REMOTE_DATABASE_URL")).await?,
    //     })
    // }

    pub async fn push_deltas(&mut self, mut deltas: BTreeSet<Delta>) -> sqlx::Result<()> {
        let timestamp = Utc::now();
        let mut local_db = self.local_db.borrow_mut();
        let mut txn = local_db.begin().await?;

        while let Some(delta) = deltas.pop_last() {
            try_insert_parent(&mut deltas, &delta);

            match delta.kind {
                DeltaKind::Update => {
                    self.handle_update(&mut txn, &delta.path, timestamp).await?;
                }
                DeltaKind::Delete => {
                    self.handle_delete(&mut txn, &delta.path, timestamp).await?;
                }
            }
        }

        txn.commit().await?;
        Ok(())
    }

    async fn handle_update<'a>(
        &self,
        txn: &mut Transaction<'_, Sqlite>,
        path: &Path,
        timestamp: DateTime<Utc>,
    ) -> sqlx::Result<()> {
        let file_id = File::get_id(path);

        let child_hashes = local::get_file_children_hashes(txn, file_id).await?;
        let file = File::new(path, timestamp, child_hashes);
        local::insert_file(txn, file.to_owned()).await?;
        local::enqueue_update(txn, QueuedUpdate { file_id }).await?;

        Ok(())
    }

    async fn handle_delete(
        &self,
        txn: &mut Transaction<'_, Sqlite>,
        path: &Path,
        deleted_at: DateTime<Utc>,
    ) -> sqlx::Result<()> {
        let root_id = File::get_id(path);
        let descendants: Vec<i64> = local::get_file_descendant_ids(txn, root_id).await?;
        let subtree = descendants.into_iter().chain(Some(root_id));

        for id in subtree {
            let record = QueuedDeletion {
                file_id: id,
                deleted_at,
            };
            local::delete_file(txn, record.file_id).await?;
            local::enqueue_delete(txn, record).await?;
        }

        Ok(())
    }
}

/// Attempts to create an implicit delta and avoids overwriting
fn try_insert_parent(deltas: &mut BTreeSet<Delta>,delta: &Delta)  {
    
    let Some(parent_path) = delta.path.parent() else {
        return
    };
    
    let parent_delta = Delta {
        path: parent_path,
        kind: DeltaKind::Update,
        depth: delta.depth - 1,
    };
    
    if deltas.contains(&parent_delta) {
        return
    }
    
    deltas.insert(parent_delta);
    
}

fn floor_mul(x: usize, y: f32) -> usize {
    let x = x as f32;
    let prod = x * y;
    prod as usize
}

