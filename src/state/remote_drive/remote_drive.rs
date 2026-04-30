use std::collections::VecDeque;

use compio::{buf::bytes::Bytes, io::AsyncReadExt, runtime::BufferPool};
use futures::{SinkExt, stream::FuturesUnordered};
use opendal::{Buffer, Operator, options::{ReadOptions, WriteOptions}};

use crate::{path::{AbsPath, RelPath, fs}, state::remote_drive::save::DriveErr};

pub struct RemoteDrive {
    pub(super) conn: Operator,
    pub(super) buf_pool: BufferPool,
}

impl RemoteDrive {
    pub const CONCURRENCY: usize = 4;
    
    pub(crate) async fn delete(&self, path: &RelPath) -> Result<(),DriveErr> {
        self.conn.delete(path).await.map_err(DriveErr::Remote)
    }

    

    pub async fn create_dir(&self, path: &RelPath) -> anyhow::Result<()> {
        self.conn.create_dir(path).await?;
        Ok(())
    }
}
