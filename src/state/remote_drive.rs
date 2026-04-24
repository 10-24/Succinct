use futures::{SinkExt, StreamExt};
use opendal::{Buffer, Operator};
use tokio::{fs, io};
use tokio_util::{bytes::Bytes, io::ReaderStream};

use crate::path::{AbsPath, RelPath};

pub struct RemoteDrive {
    pub drive: Operator,
    root: AbsPath<Remote>,
}

impl RemoteDrive {
    pub fn into_remote_path(&self, path: &RelPath) -> AbsPath<Remote> {
        self.root.join(path)
    }

    pub(crate) async fn delete(&self, path: &RelPath) -> anyhow::Result<()> {
        let remote_path = self.into_remote_path(path);
        self.drive.delete(remote_path.as_ref()).await?;
        Ok(())
    }

    pub(crate) async fn save_file(
        &self,
        local_path: &AbsPath,
        path: &RelPath,
    ) -> anyhow::Result<()> {
        let remote_path = self.into_remote_path(path);
        let file_content = fs::File::open(local_path.as_ref()).await?;

        let upload_stream =
            ReaderStream::with_capacity(file_content, 65536).map(convert_to_upload_stream);

        let mut upload_sink = self
            .drive
            .writer_with(remote_path.as_ref())
            .chunk(65536)
            .await?
            .into_sink();

        upload_sink.send_all(&mut upload_stream.boxed()).await?; // Its generally considered bad taste to send your stream into someone's sink
        upload_sink.close().await?;

        Ok(())
    }
    
    pub async fn create_dir(&self, path: &RelPath) -> anyhow::Result<()> {
        let remote_path = self.into_remote_path(path);
        self.drive.create_dir(remote_path.as_ref()).await?;
        Ok(())
    }
}

fn convert_to_upload_stream(chunk: Result<Bytes, io::Error>) -> opendal::Result<opendal::Buffer> {
    chunk.map(Buffer::from).map_err(into_opendal_err)
}

fn into_opendal_err(e: io::Error) -> opendal::Error {
    opendal::Error::new(
        opendal::ErrorKind::Unexpected,
        format!("Reader failed: {:?}", e),
    )
}
