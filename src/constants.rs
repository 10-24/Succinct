use std::time::Duration;

use const_format::concatcp;


pub const DEFAULT_DEBOUNCE_DURATION: Duration = Duration::from_secs(12);
pub const DEFAULT_BUCKET_NAME: &str = "sync";
pub const DEFAULT_BUCKET_REGION: &str = "auto";
pub const APP_NAME:&str = env!("CARGO_PKG_NAME");
pub const CONFIG_FILE_NAME: &str = concatcp!(APP_NAME, ".toml");
pub const DB_FILE_NAME: &str = concatcp!(APP_NAME, ".db");

pub const REQUIRED_EXCLUDE_GLOBS: &[&str] = &[

];


#[derive(strum::IntoStaticStr)]
#[allow(non_camel_case_types)]
pub enum EnvVar {
    SUCCINCT_DIR,
}
impl EnvVar {
    pub fn read(self) -> Option<String> {
        std::env::var(self.name()).ok()
    }

    pub fn name(self) -> &'static str {
        self.into()
    }
}



