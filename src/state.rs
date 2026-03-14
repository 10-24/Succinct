use std::{
    collections::BTreeSet,
    hash::{Hash, Hasher},
};

use anyhow::Result;
use futures::future::join_all;
use rustc_hash::FxHasher;
use sqlx::{Database, PgPool, Pool, Postgres, Sqlite, SqlitePool};
use chrono::{Utc,DateTime};
use crate::{
    delta::{Delta, DeltaKind},
    entry::{Entry},
    path::Path,
};

struct State {
    remote: Pool<Postgres>,
    local: Pool<Sqlite>,
}
impl State {
    
    pub async fn new() -> Result<Self> {
        Ok(Self {
            local:  SqlitePool::connect(env!("DATABASE_URL")).await?,
            remote: PgPool::connect(env!("REMOTE_DATABASE_URL")).await?,
        })
    }

    pub async fn push_deltas(&mut self, mut deltas: BTreeSet<Delta>) -> Result<()> {
        // Todo: Assert databases are synced
        let timestamp = Utc::now();

        while let Some(delta) = deltas.pop_last() {
            deltas.extend(create_parent_delta(&delta));
            
            match delta.kind {
                DeltaKind::Update => self.handle_update(&delta.path, timestamp).await?,
                DeltaKind::Delete => self.handle_delete(Entry::get_id(&delta.path)).await?,
            }
        }
        Ok(())
    }

    async fn handle_update(&self, conn: &mut <DB as Database>::Connection, path: &Path, timestamp: DateTime<Utc>) -> Result<()> {
        let entry_id = Entry::get_id(path);
        
        let mut hasher = FxHasher::default();
        let entry_children_hashes = self.get_child_hashes(entry_id).await?;
        for child_hash in entry_children_hashes {
            child_hash.hash(&mut hasher);
        }

        let entry = Entry::new(path, timestamp, hasher);
        self.insert(entry).await?;

        Ok(())
    }
    
    async fn handle_delete(&self, parent_id: i64) -> Result<()> {
        let child_ids: Vec<i64> =sqlx::query_scalar!("SELECT id FROM files WHERE parent_id = ?", parent_id)
                .fetch_all(&self.local)
                .await?;
        join_all(child_ids.into_iter().map(|child_id|self.handle_delete(child_id))).await.into_iter().collect()
    }
    
    async fn hash_children(&self, entry_id: i64, state: &mut FxHasher) -> Result<()> {
        let hashes: Vec<i64> =
            sqlx::query_scalar!("SELECT hash FROM files WHERE parent_id = ?", entry_id)
                .fetch_all(&self.local)
                .await?;

        for hash in hashes {
            state.write_i64(hash);
        }
        Ok(())
    }

    pub async fn insert(&self, entry: Entry) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT OR REPLACE INTO files (id, name, hash, modified_at, parent_id)
            VALUES (?, ?, ?, ?, ?)
            ",
            entry.id,
            entry.name,
            entry.hash,
            entry.modified_at,
            entry.parent_id
        )
        .execute(&self.local)
        .await?;

        Ok(())
    }

    pub async fn remove(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM files WHERE id = ?", id)
            .execute(&self.local)
            .await?;
        Ok(())
    }

    pub async fn get(&self, id: i64) -> Result<Option<Entry>, sqlx::Error> {
        sqlx::query_as!(
            Entry,
            r#"
            SELECT 
                id, 
                name, 
                hash as "hash: i32", 
                modified_at as "modified_at: DateTime<Utc>", 
                parent_id 
            FROM files
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.local)
        .await
    }

    pub async fn get_child_hashes(&self, parent_id: i64) -> Result<Vec<i64>, sqlx::Error> {
        sqlx::query_scalar!("SELECT hash FROM files WHERE parent_id = ?", parent_id)
            .fetch_all(&self.local)
            .await
    }
}

fn create_parent_delta(delta: &Delta) -> Option<Delta> {
    let parent = delta.path.parent()?;
    Some(Delta {
        path: parent,
        kind: DeltaKind::Update,
        depth: delta.depth - 1,
    })
}

fn floor_mul(x: usize, y: f32) -> usize {
    let x = x as f32;
    let prod = x * y;
    prod as usize
}
