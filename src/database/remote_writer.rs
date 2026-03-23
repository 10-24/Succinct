use std::marker::PhantomData;

use sqlx::{PgConnection, PgExecutor, PgPool, postgres::{PgConnectOptions, PgPoolOptions}};


use crate::{config::DATABASE_WRITE_CONNECTIONS, state::file::{File, FileId}};

pub struct RemoteWriter<'q, E: PgExecutor<'q>> {
    pub exec: E,
    _marker: PhantomData<&'q ()>,
}

impl<'q, E> RemoteWriter<'q, E>
where
    E: PgExecutor<'q>,
    for<'e> &'e E: PgExecutor<'e>,
{
    pub async fn init(options: PgConnectOptions) -> PgPool {
        PgPoolOptions::new()
            .max_connections(DATABASE_WRITE_CONNECTIONS)
            .connect_with(options)
            .await
            .unwrap()
    }
    
    pub async fn insert_file(&mut self, entry: File) -> sqlx::Result<()> {
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
        .execute(&self.exec)
        .await?;
    
        Ok(())
    }

}


