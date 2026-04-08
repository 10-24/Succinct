use ignore::gitignore::{Gitignore, GitignoreBuilder};
use tokio::{fs, io::AsyncBufReadExt};
use colored::*;
use crate::{ config::{ IGNORE_FILE_NAME, config::{Config, panic_required_file}}};

impl Config {
    pub async fn create_ignore(&self) -> Gitignore {
        let ignore_path = self.local.root_path.join(IGNORE_FILE_NAME);
        let ignore_file = fs::File::open(ignore_path.as_ref()).await.unwrap_or_else(panic_required_file(ignore_path));
        
        let mut ignore_file_lines = tokio::io::BufReader::new(ignore_file).lines();
        let mut ignore_builder = GitignoreBuilder::new(self.local.root_path.as_ref());
        while let Some(line) = ignore_file_lines.next_line().await.unwrap() {
            if let Err(e) = ignore_builder.add_line(None,&line) {
                panic!("
                    Failed to parse {IGNORE_FILE_NAME}
                    >    {}
                    Path: 
                    Error: {e}
                ", line.red().bold())
            }
        }
        let (ignore, build_err) = ignore_builder.build_global();
        if let Some(build_err) = build_err {
            panic!("Failed to build ignore: {}", build_err);
        }
        ignore
    }
}