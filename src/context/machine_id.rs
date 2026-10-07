use serde::{Deserialize, Serialize};
use tokio::{fs, io};
use xxhash_rust::xxh32::xxh32;



#[derive(Debug,Serialize,Deserialize,Clone, Copy)]
pub struct MachineId(u32);
impl MachineId {
    const PATH: &'static str = "/etc/machine-id";
    const SEED: u32 = 1024;

    pub async fn read() -> io::Result<Self> {
        fs::read_to_string(Self::PATH).await.map(Self::from_str)
    }

    pub fn from_str(id: impl AsRef<str>) -> Self {
        let id = id.as_ref().trim();
        assert!(!id.is_empty());
        let hash = xxh32(id.as_bytes(), Self::SEED);
        Self(hash)
    }
}