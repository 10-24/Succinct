use futures::{SinkExt, StreamExt};
use opendal::{Buffer, Operator};
use sqlx::{PgConnection, SqliteConnection};
use tokio::{fs, io};
use tokio_util::{bytes::Bytes, io::ReaderStream};

use crate::path::{AbsPath, Local, Remote};

pub(crate) async fn delete(
    remote_drive: &Operator,
    remote_path: &AbsPath<Remote>,
) -> anyhow::Result<()> {
    remote_drive.delete(remote_path).await?;
    Ok(())
}

pub(crate) async fn save(
    remote_drive: &Operator,
    local_path: &AbsPath<Local>,
    remote_path: &AbsPath<Remote>,
) -> anyhow::Result<()> {
    let file_content = fs::File::open(local_path.as_ref()).await?;
    let upload_stream =
        ReaderStream::with_capacity(file_content, 65536).map(convert_to_upload_stream);

    
    let mut upload_sink = remote_drive
        .writer_with(remote_path)
        .chunk(65536)
        .await?
        .into_sink();

    upload_sink
        .send_all(&mut upload_stream.boxed())
        .await?; // Its typically considered bad taste to send your stream into someone's sink
    upload_sink.close().await?;

    Ok(())
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
