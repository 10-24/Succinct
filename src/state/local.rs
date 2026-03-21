use compact_str::{CompactString, CompactStringExt};
use sqlx::{Executor, Postgres, Sqlite, SqliteConnection};

use chrono::{DateTime, Utc};

use crate::{path::Path, state::file::File};

pub async fn insert_file(exec: &mut SqliteConnection, entry: File) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT OR REPLACE INTO file (id, name, hash, modified_at, parent_id) VALUES (?, ?, ?, ?, ?)",
        entry.id,
        entry.name,
        entry.hash,
        entry.modified_at,
        entry.parent_id
    )
    .execute(exec)
    .await?;

    Ok(())
}

pub async fn remove(exec: &mut SqliteConnection, id: i64) -> sqlx::Result<()> {
    sqlx::query!("DELETE FROM file WHERE id = ?", id)
        .execute(exec)
        .await?;
    Ok(())
}

pub async fn remove_remote(
    exec: impl Executor<'_, Database = Postgres>,
    id: i64,
) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM file WHERE id = $1")
        .bind(id)
        .execute(exec)
        .await?;
    Ok(())
}

pub async fn get_file(exec: &mut SqliteConnection, id: i64) -> sqlx::Result<Option<File>> {
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
    .fetch_optional(exec)
    .await
}

pub async fn get_file_child_hashes(
    exec: &mut SqliteConnection,
    parent_id: i64,
) -> sqlx::Result<Vec<i64>> {
    sqlx::query_scalar!("SELECT hash FROM file WHERE parent_id = ?", parent_id)
        .fetch_all(exec)
        .await
}

pub async fn get_file_descendant_ids(
    exec: &mut SqliteConnection,
    parent_id: i64,
) -> sqlx::Result<Vec<i64>> {
    sqlx::query_scalar!("SELECT id FROM file WHERE parent_id = ?", parent_id)
        .fetch_all(exec)
        .await
}

pub async fn get_file_children_hashes(
    exec: &mut SqliteConnection,
    parent_id: i64,
) -> sqlx::Result<impl Iterator<Item = i32>> {
    let hashes = sqlx::query_scalar!("SELECT hash FROM file WHERE parent_id = ?", parent_id)
        .fetch_all(exec)
        .await?;
    Ok(hashes.into_iter().map(|h| h as i32))
}

pub async fn delete_file(exec: &mut SqliteConnection, id: i64) -> sqlx::Result<()> {
    let _ = sqlx::query!("DELETE FROM file WHERE id = ?", id)
        .execute(exec)
        .await;
    Ok(())
}

pub async fn enqueue_update(exec: &mut SqliteConnection, update: QueuedUpdate) -> sqlx::Result<()> {
    sqlx::query!(
        "DELETE FROM queued_delete where file_id = ?",
        update.file_id
    )
    .execute(&mut *exec)
    .await?;

    sqlx::query!(
        "INSERT OR REPLACE INTO queued_update (file_id) VALUES (?)",
        update.file_id,
    )
    .execute(exec)
    .await?;

    Ok(())
}

pub async fn enqueue_delete(
    exec: &mut SqliteConnection,
    deletion: QueuedDeletion,
) -> sqlx::Result<()> {
    let _ = sqlx::query!(
        "DELETE FROM queued_update where file_id = ?",
        deletion.file_id
    )
    .execute(&mut *exec)
    .await;

    sqlx::query!(
        "INSERT OR REPLACE INTO queued_delete (file_id, deleted_at) VALUES (?, ?)",
        deletion.file_id,
        deletion.deleted_at,
    )
    .execute(exec)
    .await?;

    Ok(())
}

pub async fn get_update_queue(exec: &mut SqliteConnection) -> sqlx::Result<Vec<File>> {
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
        "#
    )
    .fetch_all(exec)
    .await
}

pub async fn get_delete_queue(exec: &mut SqliteConnection) -> sqlx::Result<Vec<QueuedDeletion>> {
    sqlx::query_as!(
        QueuedDeletion,
        r#"
        SELECT
            file_id,
            deleted_at as "deleted_at: DateTime<Utc>"
        FROM queued_delete
        "#
    )
    .fetch_all(exec)
    .await
}

pub async fn remove_from_update_queue(exec: &mut SqliteConnection, id: i64) -> sqlx::Result<()> {
    sqlx::query!("DELETE FROM queued_update WHERE file_id = ?", id)
        .execute(exec)
        .await?;

    Ok(())
}

pub async fn remove_from_delete_queue(exec: &mut SqliteConnection, id: i64) -> sqlx::Result<()> {
    sqlx::query!("DELETE FROM queued_delete WHERE file_id = ?", id)
        .execute(exec)
        .await?;

    Ok(())
}

pub async fn get_file_path(exec: &mut SqliteConnection, file_id: i64) -> sqlx::Result<Path> {
    let components: Vec<CompactString> = sqlx::query_scalar!(
        r#"
            WITH RECURSIVE file_path AS (
                SELECT id, name, parent_id FROM file WHERE id = ?
                UNION ALL
                SELECT f.id, f.name, f.parent_id
                FROM file f
                JOIN file_path fp ON f.id = fp.parent_id
            )
            SELECT name as "name: CompactString" FROM file_path
            "#,
            file_id
    )
    .fetch_all(exec)
    .await?;

  
    Ok(Path::from(components.join("/")))
}

#[derive(Debug)]
pub struct QueuedUpdate {
    pub file_id: i64,
    // pub updated_at: DateTime<Utc>,
}

#[derive(Debug,Clone, Copy)]
pub struct QueuedDeletion {
    pub file_id: i64,
    pub deleted_at: DateTime<Utc>,
}
