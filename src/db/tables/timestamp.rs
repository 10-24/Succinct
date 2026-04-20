use bytemuck::{Pod, Zeroable};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy,PartialEq, Eq,PartialOrd, Ord,Zeroable,Deserialize,Serialize,Pod,Default,Hash)]
#[repr(C)]
pub struct Timestamp (i64);
impl Timestamp {
    pub fn now() -> Self {
        let timestamp_sec = Utc::now().timestamp();
        Self(timestamp_sec)
    }
    pub fn into_datetime(self) -> chrono::DateTime<Utc> {
        DateTime::from_timestamp_secs(self.0 as i64).unwrap()
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.into_datetime().fmt(f)
    }
}

impl std::fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.into_datetime().fmt(f)
    }
}