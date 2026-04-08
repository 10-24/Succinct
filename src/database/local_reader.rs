use std::iter;

use compact_str::CompactString;
use futures::future::try_join_all;
use sqlx::{
    Executor, Postgres, SqliteConnection, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

use chrono::{DateTime, Utc};
use tokio::{join, try_join};

use crate::{
    config::{DATABASE_READ_CONNECTIONS, INTERNAL_ROOT_NAME, ROOT_ID, ROOT_PARENT_ID},
    database::QueuedDeletion,
    path::RelPath,
    state::{
        file::File,
        file_id::{FileId, FileIdOrd},
    },
};
#[derive(Debug,Clone)]
pub struct LocalReader {
    pool: SqlitePool,
}

impl LocalReader {
    pub async fn init(options: SqliteConnectOptions) -> Self {
        let pool = SqlitePoolOptions::new()
            .max_connections(DATABASE_READ_CONNECTIONS)
            .connect_with(options)
            .await
            .unwrap();
        Self { pool }
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
                parent_id,
                depth as "depth: u16",
                created_at as "created_at: DateTime<Utc>"
            FROM file
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_file_descendants(&self, parent_id: FileId) -> sqlx::Result<Vec<FileIdOrd>> {
        sqlx::query_as!(
            FileIdOrd,
            r#"
                WITH RECURSIVE descendants AS (
                    SELECT id, depth FROM file WHERE id = ?
                    UNION ALL
                    SELECT f.id, f.depth
                    FROM file f
                    JOIN descendants d ON f.parent_id = d.id
                )
                SELECT
                    id as "value: i64",
                    COALESCE(depth, 0) as "depth: u16"
                FROM descendants
                WHERE id != ?
                "#,
            parent_id,
            parent_id,
        )
        .fetch_all(&self.pool)
        .await
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
                file.parent_id,
                file.depth as "depth: u16",
                file.created_at as "created_at: DateTime<Utc>"
            FROM queued_update
            INNER JOIN file ON queued_update.file_id = file.id
            ORDER BY file.depth ASC
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

    pub async fn file_exists(&self, file_id: FileId) -> sqlx::Result<bool> {
        let exists =
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM file WHERE id = ?)", file_id,)
                .fetch_one(&self.pool)
                .await?;

        Ok(exists == 1)
    }

    pub async fn get_path_components(
        &self,
        file_id: FileId,
    ) -> sqlx::Result<Option<Vec<CompactString>>> {
        let mut components = vec![INTERNAL_ROOT_NAME.into()];

        if file_id.is_root() {
            return Ok(Some(components));
        }

        let non_root_components: Vec<CompactString> = sqlx::query_scalar!(
            r#"
                WITH RECURSIVE file_path AS (
                    SELECT id, name, parent_id FROM file WHERE id = ?
                    UNION ALL
                    SELECT f.id, f.name, f.parent_id
                    FROM file f
                    JOIN file_path fp ON f.id = fp.parent_id
                    WHERE f.depth != 0
                )
                SELECT name as "name: CompactString" FROM file_path
                "#,
            file_id,
        )
        .fetch_all(&self.pool)
        .await?;

        if non_root_components.is_empty() {
            return Ok(None);
        }
        components.extend(non_root_components);
        Ok(Some(components))
    }

    pub async fn get_file_path(&self, file_id: FileId) -> sqlx::Result<Option<RelPath>> {
        if let Some(components) = self.get_path_components(file_id).await? {
            let path = RelPath::new(components.join("/"));
            return Ok(Some(path));
        };
        Ok(None)
    }

    pub async fn get_file_ancestors(
        &self,
        file_id: FileId,
    ) -> sqlx::Result<Option<impl Iterator<Item = FileIdOrd>>> {
        let Some(components) = self.get_path_components(file_id).await? else {
            return Ok(None);
        };

        let ancestor_ids = components
            .into_iter()
            .scan(ROOT_PARENT_ID, |last_id, component| {
                *last_id = last_id.child(&component);
                Some(*last_id)
            }); 
        Ok(Some(ancestor_ids))
    }

    pub async fn get_file_parent_id(&self, file_id: FileId) -> sqlx::Result<Option<FileId>> {
        if file_id == *ROOT_ID {
            return Ok(None);
        }
        let opt = sqlx::query_scalar!("SELECT parent_id FROM file WHERE id = ?", file_id)
            .fetch_optional(&self.pool)
            .await?
            .map(FileId::from);
        Ok(opt)
    }
}
