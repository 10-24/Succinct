use std::{sync::Arc, time::Duration};

use anyhow::Context;

use crate::{
    constants::{config_default},
    context::config::raw::RawConfig,
    util::{
        arc_path::{ArcPath, IntoArcPath},
        globs::{Glob, Globs},
    },
};

#[derive(Debug, derive_getters::Getters)]
pub struct Config {
    exclude: Arc<Globs>,
    debounce: Duration,
    ignore_deletions: bool,
    remote: RemoteConfig,
}

#[derive(Debug, derive_getters::Getters)]
pub struct RemoteConfig {
    endpoint: Box<str>,
    access_key_id: Box<str>,
    secret_access_key: Box<str>,
    max_retries: u16,
    timeout: Duration,
    region: Box<str>,
    bucket: Box<str>,
}

impl Config {
    
    pub fn from_toml(toml: &str) -> anyhow::Result<(ArcPath, Self)> {
        let raw = toml::from_str::<RawConfig>(toml)?;
        let sync_dir = raw.sync_dir.into_arc_path();

        let config = Self {
            exclude: Self::create_exclude(sync_dir.clone(), raw.exclude)?.into(),
            debounce: raw.debounce_seconds.map(Duration::from_secs).unwrap_or(config_default::DEBOUNCE),
            ignore_deletions: raw.ignore_deletions.unwrap_or(config_default::IGNORE_DELETIONS),
            remote: RemoteConfig {
                endpoint: raw.remote.endpoint,
                access_key_id: raw.remote.access_key_id,
                secret_access_key: raw.remote.secret_access_key,
                timeout: raw.remote.timeout_seconds.map(Duration::from_secs).unwrap_or(config_default::remote::CONNECT_TIMEOUT),
                region: raw.remote.region.unwrap_or_else(owned(config_default::remote::REGION)),
                bucket: raw.remote.bucket.unwrap_or_else(owned(config_default::remote::BUCKET)),
            },
        };

        Ok((sync_dir, config))
    }

    fn create_exclude(sync_dir: ArcPath, user_globs: Vec<Glob>) -> anyhow::Result<Globs> {
        let required_exclude_globs = config_default::REQUIRED_EXCLUDE_GLOBS
            .into_iter()
            .map(Glob::try_from)
            .map(Result::unwrap);

        let globs = required_exclude_globs.chain(user_globs);
        Globs::new(sync_dir, globs).with_context(|| "failed to build excluded glob patterns")
    }
}


fn owned<I, O: From<I>>(i: I) -> impl FnOnce() -> O {
    move || O::from(i)
}