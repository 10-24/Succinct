use derive_more::{From, Into};
use sqlx::{SqliteExecutor, SqlitePool, SqliteTransaction, sqlite::{SqliteConnectOptions, SqlitePoolOptions}};
use std::marker::PhantomData;
use crate::{config::DATABASE_WRITE_CONNECTIONS, database::{QueuedDeletion, QueuedUpdate}, state::file::{File, FileId}};
use compact_str::CompactString;
use chrono::DateTime;
use chrono::Utc;

#[derive(From,Into)]
pub struct LocalWriter<'q> {
    pub txn: SqliteTransaction<'q>,
}

impl<'a> LocalWriter<'a> {
    pub async fn init(options: SqliteConnectOptions) -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(DATABASE_WRITE_CONNECTIONS)
            .connect_with(options)
            .await
            .unwrap()
    }
    
    pub async fn remove(&mut self, id: FileId) -> sqlx::Result<()> {
        sqlx::query!("DELETE FROM file WHERE id = ?", id)
            .execute(&mut *self.txn)
            .await?;
        Ok(())
    }
    
    pub async fn delete_file(&mut self, id: FileId) -> sqlx::Result<()> {
        let _ = sqlx::query!("DELETE FROM file WHERE id = ?", id)
            .execute(&mut *self.txn)
            .await;
        Ok(())
    }
    
    pub async fn enqueue_update(&mut self, update: QueuedUpdate) -> sqlx::Result<()> {
        sqlx::query!(
            "DELETE FROM queued_delete where file_id = ?",
            update.file_id
        )
        .execute(&mut *self.txn)
        .await?;
    
        sqlx::query!(
            "INSERT OR REPLACE INTO queued_update (file_id) VALUES (?)",
            update.file_id,
        )
        .execute(&mut *self.txn)
        .await?;
    
        Ok(())
    }
    
    pub async fn enqueue_delete(
        &mut self,
        deletion: QueuedDeletion,
    ) -> sqlx::Result<()> {
        let _ = sqlx::query!(
            "DELETE FROM queued_update where file_id = ?",
            deletion.file_id
        )
        .execute(&mut *self.txn)
        .await;
    
        sqlx::query!(
            "INSERT OR REPLACE INTO queued_delete (file_id, deleted_at) VALUES (?, ?)",
            deletion.file_id,
            deletion.deleted_at,
        )
        .execute(&mut *self.txn)
        .await?;
    
        Ok(())
    }
    
    
    
    pub async fn remove_from_update_queue(&mut self, id: FileId) -> sqlx::Result<()> {
        sqlx::query!("DELETE FROM queued_update WHERE file_id = ?", id)
            .execute(&mut *self.txn)
            .await?;
    
        Ok(())
    }
    
    pub async fn remove_from_delete_queue(&mut self, id: FileId) -> sqlx::Result<()> {
        sqlx::query!("DELETE FROM queued_delete WHERE file_id = ?", id)
            .execute(&mut *self.txn)
            .await?;
    
        Ok(())
    }
    
    pub async fn modify_file(&mut self, id: FileId, hash: i32, modified_at: DateTime<Utc>) -> sqlx::Result<()> {
        sqlx::query!(
            "UPDATE file SET hash = ?, modified_at = ? WHERE id = ?",
            hash,
            modified_at,
            id
        )
        .execute(&mut *self.txn)
        .await?;
    
        Ok(())
    }
    
    pub async fn ensure_file_exists(&mut self, id: FileId, name: impl Into<CompactString>, modified_at: DateTime<Utc>, parent_id: FileId) -> sqlx::Result<()> {
        let file = File {
            id,
            name: name.into(),
            parent_id: id,
            hash: 0,
            modified_at,
        };
        sqlx::query!(
            "INSERT OR IGNORE INTO file (id, name, hash, modified_at, parent_id) VALUES (?, ?, ?, ?, ?)",
            file.id,
            file.name,
            file.hash,
            file.modified_at,
            file.parent_id
        )
        .execute(&mut *self.txn)
        .await?;
        
        Ok(())
    }
}
