use std::fs::Metadata;

use sqlx::PgExecutor;
use tokio::fs;

use crate::{
    database::remote_writer::RemoteWriter,
    state::{file::File, remote_drive, state::State},
};

impl State {
    pub async fn clear_queue<'a, E>(
        &mut self,
        local_writer: &mut crate::database::local_writer::LocalWriter<'_>,
        remote_writer: &mut RemoteWriter<'a, E>,
    ) -> anyhow::Result<()>
    where
        E: PgExecutor<'a>,
        for<'e> &'e E: PgExecutor<'e>,
    {
        let deletion_records = self.local_reader.get_delete_queue().await?;
        for deletion in deletion_records {
            let path = self
                .local_reader
                .get_file_path(deletion.file_id)
                .await?
                .unwrap();
            let remote_path = self.remote_root.join(&path);

            remote_drive::delete(&self.remote_drive, &remote_path).await?;
            // TODO: Remove from delete queue
        }

        let updated_files = self.local_reader.get_update_queue().await?;
        for updated_file in updated_files {
            self.process_queued_update(&self.local_reader, remote_writer, updated_file)
                .await?;
            // TODO: Remove from update queue
        }

        Ok(())
    }

    async fn process_queued_update<'a, E>(
        &mut self,
        local_reader: &crate::database::local_reader::LocalReader,
        remote_writer: &mut RemoteWriter<'a, E>,
        updated_file: File,
    ) -> anyhow::Result<()>
    where
        E: PgExecutor<'a>,
        for<'e> &'e E: PgExecutor<'e>,
    {
        let file_id = updated_file.id;

        let path = local_reader.get_file_path(file_id).await?.unwrap();
        let abs_path = self.local_root.join(&path);
        let rel_path = abs_path.as_relative(&self.local_root);

        let metadata: Metadata = fs::metadata(abs_path.as_ref()).await?;
        if metadata.is_dir() {
            return Ok(self.remote_drive.create_dir(rel_path).await?);
        }

        let file_content: Vec<u8> = tokio::fs::read(abs_path.as_ref()).await?;

        self.remote_drive.write(rel_path, file_content).await?;

        remote_writer.insert_file(updated_file).await?;
        Ok(())
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
