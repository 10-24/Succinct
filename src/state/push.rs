use std::fs::Metadata;

use futures::future::try_join_all;
use sqlx::PgExecutor;
use tokio::{fs, try_join};

use crate::{
    database::{local_writer::LocalWriter, remote_writer::RemoteWriter},
    state::{file::File, file_id::FileId, remote_drive, state::State},
};

impl State {
    pub async fn clear_queue(
        &mut self,
        local_writer: &mut LocalWriter<'_>,
        remote_writer: &mut RemoteWriter<'_>,
    ) -> anyhow::Result<()>
    {
        
        let deleted_files = self.local_reader.get_delete_queue().await?;
        let delete_futs = deleted_files.iter().map(|f| self.delete_from_drive(&f.file_id));
        try_join_all(delete_futs).await?;
        for delete in deleted_files {
            try_join!(
                remote_writer.delete(delete.file_id),
                local_writer.dequeue_update(delete.file_id)
            )?;
        }
        
        let updated_files = self.local_reader.get_update_queue().await?;
        let save_futs = updated_files.iter().map(|f| self.save_file_to_drive(f));
        try_join_all(save_futs).await?;
        for updated_file in updated_files {
            try_join!(
                remote_writer.insert_file(&updated_file),
                local_writer.dequeue_update(updated_file.id)
            )?;
        }
        
        Ok(())
    }

    async fn delete_from_drive(&self, id: &FileId) -> anyhow::Result<()> {
        let path = self.local_reader.get_file_path(*id).await?.unwrap();
        self.remote_drive.delete(&path).await?;
        Ok(())
    }

    async fn save_file_to_drive(&self, updated_file: &File) -> anyhow::Result<()> {
        let path = self
            .local_reader
            .get_file_path(updated_file.id)
            .await?
            .unwrap();
        let local_path = self.local_root.join(&path);

        let metadata = fs::metadata(local_path.as_ref()).await?;
        if !metadata.is_dir() {
            self.remote_drive.save_file(&local_path, &path).await?;
            return Ok(());
        }

        if updated_file.is_new() {
            // Could cause errors if a file becomes a folder
            self.remote_drive.create_dir(&path).await?;
        }
        return Ok(());
    }
}

// pub struct Biconn {
//     local: SqliteConnection,
//     remote: PgConnection,
// }

// #[derive(Debug)]
// enum Error {
//     RemoteDb(sqlx::Error),
//     LocalDb(sqlx::Error),
//     RemoteDrive(opendal::Error),
//     LocalDrive(tokio::io::Error),
// }

// type Result<T> = anyhow::Result<T, Error>;
