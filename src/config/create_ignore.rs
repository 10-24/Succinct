use crate::{config::{
    IGNORE_FILE_NAME,
    config::{Config, panic_required_file},
}, path::{AbsPath, Local}};
use anyhow::anyhow;
use colored::*;
use globset::{Glob, GlobSet, GlobSetBuilder};
use tokio::{fs, io::AsyncBufReadExt};

impl Config {
    pub async fn create_ignore(&self) -> anyhow::Result<GlobSet> {
        let path = self.local.root_path.child(IGNORE_FILE_NAME);
        let file = fs::File::open(path.as_ref())
            .await
            .unwrap_or_else(panic_required_file(&path));
        let mut file = tokio::io::BufReader::new(file).lines();
        
        let mut builder = GlobSetBuilder::new();
        while let Some(line) = file.next_line().await? {
            let glob = Glob::new(&line).map_err(parse_err(&line, &path))?;
            builder.add(glob);
        }
        Ok(builder.build()?)
    }
}

fn parse_err(line: &str, path: &AbsPath<Local>) -> impl FnOnce(globset::Error) -> anyhow::Error {
    move |e: globset::Error| {
        anyhow!(
            "
            Failed to parse {IGNORE_FILE_NAME}
            -->  {}
            Error: {e}
            Path: {path}
            ",
            line.underline(),
        )
    }
}