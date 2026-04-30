use std::{io::{self, Read}, ops::Deref, os::unix::fs::MetadataExt};

use compio::{buf::bytes::Bytes, driver::BorrowedBuffer, fs, io::AsyncReadManagedAt};
use futures::{
    SinkExt, StreamExt, TryFutureExt, future::{JoinAll, join_all}, stream::FuturesUnordered
};
use opendal::{Buffer, options::WriteOptions};

use crate::{
    db::tables::file::FileId,
    path::{AbsPath, RelPath},
    state::remote_drive::remote_drive::RemoteDrive,
};

impl RemoteDrive {
    pub async fn upload_all(
        &mut self,
        mut paths: impl Iterator<Item = UpdatePath>,
    ) -> Result<(), DriveErr> {
        let mut uploads = FuturesUnordered::new();
        uploads.extend(paths.by_ref().take(4).map(|p| self.upload(p)));

        while let Some(upload_res) = uploads.next().await {
            upload_res?;
            if let Some(p) = paths.next() {
                uploads.push(self.upload(p));
            }
        }

        Ok(())
    }

    async fn upload(&self, path: UpdatePath) -> Result<(),DriveErr> {
        let write_options = WriteOptions {
            concurrent: Self::CONCURRENCY,
            ..WriteOptions::default()
        };

        let file_buf = self
            .read_local(&path.abs)
            .await
            .map_err(DriveErr::Local)?;
        let mut writer = self
            .conn
            .writer_options(&path.rel, write_options)
            .await
            .map_err(DriveErr::Remote)?;
        writer.write_from(file_buf.as_slice()).await;
        Ok(())
    }

    async fn read_local(&self, local_path: &AbsPath) -> io::Result<BorrowedBuffer> {
        let file = fs::File::open(local_path).await?;
        let len = file.metadata().await?.size() as usize;
        file.read_managed_at(&self.buf_pool, len, 0).await
    }
}

pub enum DriveErr {
    Local(io::Error),
    Remote(opendal::Error),
}


pub struct UpdatePath {
    pub id: FileId,
    pub rel: RelPath,
    pub abs: AbsPath,
}
