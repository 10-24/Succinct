use std::time::Duration;

use const_format::concatcp;
use object_store::aws::AmazonS3ConfigKey;


pub const DEFAULT_DEBOUNCE_DURATION: Duration = Duration::from_secs(12);
pub const APP_NAME:&str = env!("CARGO_PKG_NAME");
pub const CONFIG_FILE_NAME: &str = concatcp!(APP_NAME, ".toml");
pub const DB_FILE_NAME: &str = concatcp!(APP_NAME, ".db");

pub const REQUIRED_EXCLUDE_GLOBS: &[&str] = &[

];


type RemoteConfigOption = (AmazonS3ConfigKey,&'static str);
pub const DEFAULT_REMOTE_CONFIG: &[RemoteConfigOption] = &[
    (AmazonS3ConfigKey::Bucket, "sync"),
    (AmazonS3ConfigKey::Region, "auto"),
];



