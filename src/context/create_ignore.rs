use anyhow::{bail, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};



pub async fn create_exclude(globs: impl Iterator<Item = &str>) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for glob_str in globs {
        match Glob::new(glob_str) {
            Ok(glob) => builder.add(glob),
            Err(e) => bail!("Failed to parse excluded glob '{glob_str}': {e}"),
        };
    }
    Ok(builder.build()?)
}

