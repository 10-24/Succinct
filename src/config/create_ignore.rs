use crate::{config::{
    IGNORE_FILE_NAME,
    config::{Config, panic_required_file},
}, path::{AbsPath, fs}};
use anyhow::anyhow;
use colored::*;
use globset::{Glob, GlobSet, GlobSetBuilder};


impl Config {
    pub async fn create_ignore(&self) -> anyhow::Result<GlobSet> {
        let path = self.local.root_path.child(IGNORE_FILE_NAME);
        let file = fs::read_to_string(&path).await.unwrap_or_else(panic_required_file(&path));

        
        let mut builder = GlobSetBuilder::new();
        for line in file.lines() {
            let glob = Glob::new(&line).map_err(parse_err(&line, &path))?;
            builder.add(glob);
        }
        Ok(builder.build()?)
    }
}

fn parse_err(line: &str, path: &AbsPath) -> impl FnOnce(globset::Error) -> anyhow::Error {
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