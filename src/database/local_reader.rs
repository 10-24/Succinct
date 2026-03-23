use compact_str::{CompactString};
use sqlx::{Executor, Postgres, SqliteConnection, SqlitePool, sqlite::{SqliteConnectOptions, SqlitePoolOptions}};

use chrono::{DateTime, Utc};

use crate::{
    config::{DATABASE_READ_CONNECTIONS, ROOT_ID, ROOT_PARENT_ID}, database::QueuedDeletion, path::RelPath, state::file::{File, FileId}
};
#[derive(Debug)]
pub struct LocalReader {
    pool: SqlitePool,
}

impl LocalReader {
    
    pub async fn init(options: SqliteConnectOptions) -> Self {
        let pool = SqlitePoolOptions::new()
                .max_connections(DATABASE_READ_CONNECTIONS) 
                .connect_with(options)
                .await.unwrap();
        Self {
            pool,
        }
    }
    
    pub async fn get_file_child_hashes(
        &self,
        parent_id: FileId,
    ) -> sqlx::Result<impl Iterator<Item = i32>> {
        let hashes = sqlx::query_scalar!("SELECT hash FROM file WHERE parent_id = ?", parent_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(hashes.into_iter().map(|h| h as i32))
    }
    
    pub async fn get_file(&self, id: FileId) -> sqlx::Result<Option<File>> {
        sqlx::query_as!(
            File,
            r#"
            SELECT
                id,
                name,
                hash as "hash: i32",
                modified_at as "modified_at: DateTime<Utc>",
                parent_id
            FROM file
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
    }
    
    /// BROKEN: should return all descendants, not just immediate children
    pub async fn get_file_descendants(
        &self,
        parent_id: FileId,
    ) -> sqlx::Result<Vec<FileId>> {
        let ids = sqlx::query_scalar!("SELECT id FROM file WHERE parent_id = ?", parent_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(ids.into_iter().map(FileId::from).collect())
    }
    
    pub async fn get_file_children_hashes(
        &self,
        parent_id: FileId,
    ) -> sqlx::Result<impl Iterator<Item = i32>> {
        let hashes = sqlx::query_scalar!("SELECT hash FROM file WHERE parent_id = ?", parent_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(hashes.into_iter().map(|h| h as i32))
    }
    
    pub async fn get_update_queue(&self) -> sqlx::Result<Vec<File>> {
        sqlx::query_as!(
            File,
            r#"
            SELECT
                file.id,
                file.name,
                file.hash as "hash: i32",
                file.modified_at as "modified_at: DateTime<Utc>",
                file.parent_id
            FROM queued_update
            INNER JOIN file ON queued_update.file_id = file.id 
            ORDER BY queued_update.depth ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
    }
    
    
    pub async fn get_delete_queue(&self) -> sqlx::Result<Vec<QueuedDeletion>> {
        sqlx::query_as!(
            QueuedDeletion,
            r#"
            SELECT
                file_id,
                deleted_at as "deleted_at: DateTime<Utc>"
            FROM queued_delete
            "#
        )
        .fetch_all(&self.pool)
        .await
    }
    
    
    
    pub async fn get_file_components(
        &self,
        file_id: FileId,
    ) -> sqlx::Result<Vec<CompactString>> {
        sqlx::query_scalar!(
            r#"
                WITH RECURSIVE file_path AS (
                    SELECT id, name, parent_id FROM file WHERE id = ?
                    UNION ALL
                    SELECT f.id, f.name, f.parent_id
                    FROM file f
                    JOIN file_path fp ON f.id = fp.parent_id
                    WHERE fp.id != ?
                )
                SELECT name as "name: CompactString" FROM file_path
                "#,
            file_id,
            ROOT_PARENT_ID,
        )
        .fetch_all(&self.pool)
        .await
    }
    
    pub async fn get_file_depth(
        &self,
        file_id: FileId,
    ) -> sqlx::Result<u16> {
        sqlx::query_scalar!(
            r#"
                WITH RECURSIVE file_path AS (
                    SELECT id, name, parent_id FROM file WHERE id = ?
                    UNION ALL
                    SELECT f.id, f.name, f.parent_id
                    FROM file f
                    JOIN file_path fp ON f.id = fp.parent_id
                    WHERE fp.id != ?
                )
                SELECT COUNT(*) FROM file_path
                "#,
            file_id,
            ROOT_PARENT_ID,
        )
        .fetch_one(&self.pool)
        .await.map(|count| count as u16)
    }
    
    pub async fn get_file_path(&self, file_id: FileId) -> sqlx::Result<Option<RelPath>> {
        let components = self.get_file_components(file_id).await?;
        if components.is_empty() {
            return Ok(None);
        }
        Ok(Some(RelPath::from(components.join("/"))))
    }
    
    pub async fn get_file_ancestors(&self, file_id: FileId) -> sqlx::Result<impl Iterator<Item = FileId>> {
        let components = self.get_file_components(file_id).await?;
        
        let ancestors = components.into_iter().scan(ROOT_PARENT_ID, |last_id,component| {
            *last_id = last_id.child(&component);
            Some(*last_id)
        });
        Ok(ancestors)
    }
    
    
    pub async fn get_file_parent_id(
        &self,
        file_id: FileId,
    ) -> sqlx::Result<Option<FileId>> {
        if file_id == ROOT_ID {
            return Ok(None);
        }
        let opt = sqlx::query_scalar!("SELECT parent_id FROM file WHERE id = ?", file_id)
            .fetch_optional(&self.pool)
            .await?.map(FileId::from);
        Ok(opt)
    }

}
