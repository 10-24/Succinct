use std::marker::PhantomData;

use derive_more::{From, Into};
use sqlx::{PgConnection, PgExecutor, PgPool, PgTransaction, postgres::{PgConnectOptions, PgPoolOptions}};


use crate::{config::DATABASE_WRITE_CONNECTIONS, state::{file::File, file_id::FileId}};

#[derive(From,Into)]
pub struct RemoteWriter<'q> {
    pub txn: PgTransaction<'q>,
}


impl<'q> RemoteWriter<'q>{
    pub async fn init(options: PgConnectOptions) -> PgPool {
        PgPoolOptions::new()
            .max_connections(DATABASE_WRITE_CONNECTIONS)
            .connect_with(options)
            .await
            .unwrap()
    }
    
    pub async fn insert_file(&mut self, entry: &File) -> sqlx::Result<()> {
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
        .bind(entry.name.to_string())
        .bind(entry.hash)
        .bind(entry.modified_at)
        .bind(entry.parent_id)
        .execute(&mut *self.txn)
        .await?;
    
        Ok(())
    }

    pub async fn delete(&mut self, id:FileId) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM files WHERE id = $1")
            .bind(id.0)
            .execute(&mut *self.txn)
            .await?;
        Ok(())
    }
}


