use sqlx::{Executor, PgConnection, Postgres};

use chrono::{DateTime, Utc};

use crate::state::file::File;

pub async fn insert_file(exec: &mut PgConnection, entry: File) -> sqlx::Result<()> {
    sqlx::query(
        "
        INSERT INTO users (id, name, hash, modified_at, parent_id)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id)
        DO UPDATE SET
            hash = EXCLUDED.hash,
            modified_at = EXCLUDED.modified_at;
    ",
    )
    .bind(entry.id)
    .bind(entry.name)
    .bind(entry.hash)
    .bind(entry.modified_at)
    .bind(entry.parent_id)
    .execute(exec)
    .await?;

    Ok(())
}


pub async fn get_file(exec: &mut PgConnection, id: i64) -> sqlx::Result<Option<File>> {
    sqlx::query_as::<_, File>(
        r#"
        SELECT
            id,
            name,
            hash as "hash: i32",
            modified_at as "modified_at: DateTime<Utc>",
            parent_id
        FROM files
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(exec)
    .await
}

pub async fn get_file_descendant_ids(
    exec: &mut PgConnection,
    parent_id: i64,
) -> sqlx::Result<Vec<i64>> {
    sqlx::query_scalar("SELECT id FROM files WHERE parent_id = $1")
        .bind(parent_id)
        .fetch_all(exec)
        .await
}

pub async fn get_file_children_hashes(
    exec: &mut PgConnection,
    parent_id: i64,
) -> sqlx::Result<impl Iterator<Item = i32>> {
    let hashes = sqlx::query_scalar("SELECT hash FROM files WHERE parent_id = $1")
        .bind(parent_id)
        .fetch_all(exec)
        .await?;
    Ok(hashes.into_iter())
}

pub async fn delete_file(exec: &mut PgConnection, id: i64, modified_before: DateTime<Utc>) -> sqlx::Result<()> {
    let _ = sqlx::query("DELETE FROM files WHERE id = $1 AND modified_at <= $2")
        .bind(id)
        .bind(modified_before)
        .execute(exec)
        .await;
    Ok(())
}

