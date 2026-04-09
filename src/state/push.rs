use futures::future::try_join_all;
use tokio::{fs, try_join};

use crate::{
    database::remote_writer::RemoteWriter,
    state::{file::File, file_id::FileId, state::State},
};

impl State {
    pub async fn clear_queue(&mut self, remote_writer: &mut RemoteWriter) -> anyhow::Result<()> {
        let deleted_files = self.local_reader.get_delete_queue();
        let delete_futs = deleted_files
            .iter()
            .map(|f| self.delete_from_drive(&f.file_id));
        try_join_all(delete_futs).await?;
        for delete in deleted_files {
            try_join!(remote_writer.delete(delete.file_id), async {
                self.local_writer.dequeue_update(delete.file_id);
                Ok(())
            })?;
        }

        let updated_files = self.local_reader.get_update_queue();
        let save_futs = updated_files.iter().map(|f| self.save_file_to_drive(f));
        try_join_all(save_futs).await?;
        for updated_file in updated_files {
            try_join!(remote_writer.insert_file(&updated_file), async {
                self.local_writer.dequeue_update(updated_file.id);
                Ok(())
            })?;
        }

        Ok(())
    }

    async fn delete_from_drive(&self, id: &FileId) -> anyhow::Result<()> {
        let path = self.local_reader.get_file_path(*id).unwrap();
        self.remote_drive.delete(&path).await?;
        Ok(())
    }

    async fn save_file_to_drive(&self, updated_file: &File) -> anyhow::Result<()> {
        let path = self.local_reader.get_file_path(updated_file.id).unwrap();
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
