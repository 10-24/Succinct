// TODO: Reimplement with appropriate database driver when remote sync is needed.
// Previously used sqlx/PostgreSQL. All sqlx dependencies have been removed from the project.

use crate::state::{file::File, file_id::FileId};

pub struct RemoteWriter;

impl RemoteWriter {
    pub async fn insert_file(&mut self, _entry: &File) -> anyhow::Result<()> {
        todo!("Reimplement remote writer")
    }

    pub async fn delete(&mut self, _id: FileId) -> anyhow::Result<()> {
        todo!("Reimplement remote writer")
    }
}
