use std::{
    error::Error,
    os::fd::{OwnedFd},
};

use camino::Utf8Path;
use futures::Stream;

use nix::{
    fcntl::{AT_FDCWD, AtFlags},
    unistd::linkat,
};
use tokio::{
    fs::{self, File},
    io::{self, AsyncWriteExt as _, BufWriter},
};
use tokio_stream::StreamExt as _;
use tokio_util::bytes::Bytes;

use crate::util::arc_path::{ArcPath, IntoArcPath};

struct EmptyTempFile {
    file: File,
    dest: ArcPath,
}

impl EmptyTempFile {
    pub async fn create(dest: impl IntoArcPath) -> io::Result<Self> {
        let dest = dest.into_arc_path();
        let file = Self::open_file(dest.parent().unwrap()).await?;
        Ok(Self { dest, file })
    }

    async fn open_file(parent: &Utf8Path) -> io::Result<File> {
        fs::OpenOptions::new()
            .write(true)
            .read(true)
            .mode(0o600)
            .open(parent)
            .await
    }

    pub async fn write_chunks<S, E>(mut self, mut stream: S) -> anyhow::Result<TempFile>
    where
        S: Stream<Item = Result<Bytes, E>> + Unpin,
        E: Into<anyhow::Error> + Sync + Send + Error + 'static,
    {
        let mut writer = BufWriter::with_capacity(64 * 1024, &mut self.file);

        while let Some(chunk_res) = stream.next().await {
            let chunk = chunk_res?;
            writer.write_all(&chunk).await?;
        }
        writer.flush().await?;
        let fd = self.file.into_std().await.into();
        Ok(TempFile {
            fd,
            dest: self.dest,
        })
    }
}

pub struct TempFile {
    fd: OwnedFd,
    dest: ArcPath,
}

impl TempFile {
    pub async fn create(dest: impl IntoArcPath) -> io::Result<EmptyTempFile> {
        EmptyTempFile::create(dest).await
    }

    pub fn replace(self) -> io::Result<()> {
        linkat(
            &self.fd,
            "",
            AT_FDCWD,
            self.dest.as_std_path(),
            AtFlags::AT_EMPTY_PATH, // flags: Tells kernel to operate directly on olddirfd
        )
        .map_err(Into::into)
    }

    pub fn destination(&self) -> &ArcPath {
        &self.dest
    }
}
