use sqlx::{PgConnection, SqliteConnection};

use crate::{
    path::{AbsPath, Path},
    state::{
        file::File,
        local::{self, QueuedDeletion, QueuedUpdate},
        remote,
        state::State,
    },
};

impl State {
    async fn clear_queue(&mut self, conn: &mut Biconn) -> Result<()> {
        let mut result: Result<()> = Ok(());

        let deletion_records = local::get_delete_queue(&mut conn.local)
            .await
            .map_err(Error::LocalDb)?;
        for deletion in deletion_records {
            let push_result = self
                .process_queued_deletion(&mut conn.local, &mut conn.remote, deletion)
                .await;
            result = result.and(push_result);
        }

        let update_records = local::get_update_queue(&mut conn.local)
            .await
            .map_err(Error::LocalDb)?;
        for update in update_records {
            let update_result = self
                .process_queued_update(&mut conn.local, &mut conn.remote, update)
                .await;
            result = result.and(update_result);
        }

        result
    }

    async fn process_queued_update(
        &mut self,
        conn_local: &mut SqliteConnection,
        remote: &mut PgConnection,
        update: File,
    ) -> Result<()> {
        let file_id = update.id;
        let path = self
            .get_abs_path(conn_local, file_id)
            .await
            .map_err(Error::LocalDb)?;

        self.save_to_drive(&path).await?;

        remote::insert_file(remote, update)
            .await
            .map_err(Error::RemoteDb)?;

        local::remove_from_update_queue(conn_local, file_id)
            .await
            .expect("Failed to remove update queue");
        Ok(())
    }

    async fn process_queued_deletion(
        &self,
        local_conn: &mut SqliteConnection,
        remote: &mut PgConnection,
        deletion: QueuedDeletion,
    ) -> Result<()> {
        let file_rel_path = local::get_file_path(local_conn, deletion.file_id)
            .await
            .map_err(Error::LocalDb)?;

        self.remote_drive
            .delete(file_rel_path.as_ref())
            .await
            .map_err(Error::RemoteDrive);

        remote::delete_file(remote, deletion)
            .await
            .map_err(Error::RemoteDb)?;

        local::remove_from_delete_queue(local_conn, deletion.file_id)
            .await
            .expect("Failed to remove delete queue");
        Ok(())
    }

    async fn save_to_drive(&self, file_abs_path: &AbsPath) -> Result<()> {
        let file_rel_path = file_abs_path.as_relative(&self.local_root);
        let metadata: std::fs::Metadata = tokio::fs::metadata(file_abs_path.as_ref())
            .await
            .map_err(Error::LocalDrive)?;
        if metadata.is_dir() {
            return self
                .remote_drive
                .create_dir(file_rel_path)
                .await
                .map_err(Error::RemoteDrive);
        }

        let file_content: Vec<u8> = tokio::fs::read(file_abs_path.as_ref())
            .await
            .map_err(Error::LocalDrive)?;
        self.remote_drive
            .write(file_rel_path, file_content)
            .await
            .map_err(Error::RemoteDrive)?;
        Ok(())
    }

    pub async fn get_abs_path(
        &self,
        conn_local: &mut SqliteConnection,
        file_id: i64,
    ) -> sqlx::Result<AbsPath> {
        let rel_path = &local::get_file_path(conn_local, file_id).await?;
        Ok(self.local_root.join(rel_path))
    }
}

pub struct Biconn {
    local: SqliteConnection,
    remote: PgConnection,
}

#[derive(Debug)]
enum Error {
    RemoteDb(sqlx::Error),
    LocalDb(sqlx::Error),
    RemoteDrive(opendal::Error),
    LocalDrive(tokio::io::Error),
}

type Result<T> = anyhow::Result<T, Error>;
