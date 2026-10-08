#[path = "keyspaces_macro.rs"]
mod keyspaces_macro;
use std::{marker::PhantomData, path::Path, sync::Arc};

use camino::Utf8Path;
use keyspaces_macro::keyspaces;
use tokio::io::{self, AsyncWriteExt, BufReader};
use async_compression::{self as compression, Level, tokio::{bufread::ZstdDecoder, write::{GzipEncoder, ZstdEncoder}}};

keyspaces! {
    FsState, "fs", Arc<Utf8Path> => bool;
}

pub trait Ks {
    const NAME: &'static str;
    type Key: serde::Serialize + serde::de::DeserializeOwned;
    type Value: serde::Serialize + serde::de::DeserializeOwned;
}

#[derive(Clone)]
pub struct Keyspace<T:Ks> {
    inner: fjall::Keyspace,
    _marker: PhantomData<T>,
}



impl<K: Ks> Keyspace<K> {
    const BYTES_COMPRESSION: Level = Level::Fastest;
    
    pub fn new(inner: fjall::Keyspace) -> Self {
          Self { inner, _marker: PhantomData }
      }
  
      pub fn inner(&self) -> &fjall::Keyspace {
          &self.inner
      }
  
      fn encode_key(key: &K::Key) -> Vec<u8> {
          bincode::serialize(key).expect("key serialization is infallible for well-formed types")
      }
  
      fn decode_value(bytes: fjall::Slice) -> K::Value {
          bincode::deserialize(&bytes).expect("stored bytes must match K::Value's shape")
      }
  
      pub fn insert(&self, key: &K::Key, value: &K::Value) -> fjall::Result<()> {
          let k = Self::encode_key(key);
          let v = bincode::serialize(value).expect("value serialization is infallible for well-formed types");
          self.inner.insert(k, v)
      }
  
      pub fn get(&self, key: &K::Key) -> fjall::Result<Option<K::Value>> {
          Ok(self.inner.get(Self::encode_key(key))?.map(Self::decode_value))
      }
  
      pub fn remove(&self, key: &K::Key) -> fjall::Result<()> {
          self.inner.remove(Self::encode_key(key))
      }
  
      pub fn contains_key(&self, key: &K::Key) -> fjall::Result<bool> {
          self.inner.contains_key(Self::encode_key(key))
      }
  
      pub fn len(&self) -> fjall::Result<usize> {
          self.inner.len()
      }

      pub fn path(&self) -> &Path {
          self.inner.path()
      }

      pub fn disk_space(&self) -> usize {
          self.inner.disk_space() as usize
      }


      /// I'm writing this before testing. If brute force doesn't work, we can insert each kv pair into a buffer and serialize that. Fjall has a native method to populate a keyspace from a stream of kv pairs
      pub async fn compress(&self) -> io::Result<Vec<u8>> {
          let buf = Vec::with_capacity(self.disk_space());
          let zstd = ZstdEncoder::with_quality(buf, Level::Precise(19));
          let mut tar = tokio_tar::Builder::new(zstd);
          tar.append_dir_all(".", self.path()).await?;
          
          let mut zstd = tar.into_inner().await?;
          zstd.shutdown().await?;
          Ok(zstd.into_inner())
      }
      pub async fn decompress(buf: &[u8], dest: &Path) -> io::Result<()> {
          let buf_reader = BufReader::new(buf.as_ref());
          let zstd = ZstdDecoder::new(buf_reader);
          tokio_tar::Archive::new(zstd).unpack(dest).await
      }
}


