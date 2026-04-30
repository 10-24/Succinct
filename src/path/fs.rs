use std::{io::{self, Cursor}, os::unix::fs::MetadataExt};

use compio::{fs::File, io::{AsyncReadExt}};
use futures::FutureExt;

use crate::{path::AbsPath};


pub async fn read_to_string(path: &AbsPath) -> io::Result<String> {
    let mut f = read_file(path).await?;
    
    let size = f.get_ref().metadata().await?.size() as usize;
    let buf = String::with_capacity(size);
    let res = f.read_to_string(buf).await;
    _ = res.0?;
    Ok(res.1)
}
   

pub async fn read_file(path: &AbsPath) -> io::Result<Cursor<File>>{
    read_file_raw(path).await.map(Cursor::new)
}

pub async fn read_file_raw(path: &AbsPath) -> io::Result<File> {
    let mut open_opts = compio::fs::OpenOptions::new();
    let f = open_opts.read(true).open(path).await?;
    Ok(f)
}


pub fn panic_required_file<T>(path: &AbsPath) -> impl FnOnce(io::Error) -> T {
    move |err: io::Error| {
        match err.kind() {
            io::ErrorKind::NotFound => panic!("{}", format_file_not_found_error(path)),
            _ => panic!("Error: Failed to read file: {path}\n Error: {err}",),
        }
    }
}

pub fn format_file_not_found_error(path:&AbsPath) -> String {
    format!("Error: Required File not found; Are sure {path} exists?",
    )
}