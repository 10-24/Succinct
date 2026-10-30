use anyhow::{Context as _, Result};
use camino::Utf8Path;
use derive_more::Deref;
use globset::{Glob, GlobSet, GlobSetBuilder};
use object_store::{
    RetryConfig,
    aws::{AmazonS3, AmazonS3Builder},
};
use std::{path::Path, sync::Arc};

use crate::{
    context::{
        app_paths::{AppPaths, AppPathsBuilder}, config::{Config, RemoteConfig, config::{Config, RemoteConfig}}, machine_id::MachineId,
    }, database::db::Db, util::{arc_path::{ArcPath, IntoArcPath}, globs::Globs},
};
use tokio::{fs, io, join, try_join};
pub mod app_paths;
pub mod config;
pub mod machine_id;



#[derive(Clone, Deref)]
pub struct Context(Arc<ContextFields>);
struct ContextFields {
    pub config: Config,
    pub machine_id: MachineId,
    pub exclude: Arc<Globs>,
    pub object_store: AmazonS3,
    pub paths: AppPaths,
    pub db: Databases,
}

impl Context {
    pub async fn load() -> Result<Context> {
        
        let paths_builder = AppPathsBuilder::new();
        let config_path = paths_builder.config();
        let ((sync_dir, config), machine_id) = try_join!(Self::read_config(config_path), Self::read_machine_id())?;
    
        let paths = paths_builder.build(sync_dir).await;
        let object_store = Self::create_object_store(&config.remote)?;

        let exclude = Globs::new(config.sync_dir.clone(),&config.exclude)?.into();
        let databases = Self::create_databases(&paths).await?;
        
        let fields = ContextFields {
            config,
            machine_id,
            exclude,
            object_store,
            paths,
            db: databases
        };
        let context = Self(Arc::from(fields));
        
        Ok(context)
    }

    async fn read_config(path: &Path) -> Result<(ArcPath, Config)> {
        let toml = fs::read_to_string(path).await?;
        Config::from_toml(&toml).with_context(|| format!("Failed to deserialize config ({})",path.to_string_lossy()))
    }


    fn create_object_store(config: &RemoteConfig) -> object_store::Result<AmazonS3> {
        let http_client_opts = object_store::ClientOptions::new()
            .with_connect_timeout(*config.timeout());

   
        AmazonS3Builder::new()
            .with_client_options(http_client_opts)
            .with_retry(RetryConfig {
                max_retries: config.max_retries() as usize,
                ..Default::default()
            })
            .build()
    }

    async fn create_databases(paths: &AppPaths) -> io::Result<Databases> {
        let (staged_deltas_db, state_db) = try_join!(
            Db::new(paths.staged_deltas_db()),
            Db::new(paths.state_db())
        )?;

        Ok(Databases {
            staged_deltas: staged_deltas_db,
            state: state_db,
        })
    }

    async fn read_machine_id() -> Result<MachineId> {
        Ok(MachineId::read().await?)
    }
}

pub struct Databases {
    pub staged_deltas: Db,
    pub state: Db,
}
