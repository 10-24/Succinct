use std::ffi::OsString;

use compact_str::CompactString;
use ignore::gitignore::Gitignore;
use inotify::{Event, EventMask, Inotify, Watches};
use rustc_hash::FxHashMap;
use tokio::sync::mpsc;

use crate::{
    database::local_reader::LocalReader,
    delta::{Delta, DeltaKind, DeltaReceiver, DeltaSender, FileRecord},
    path::{AbsPath, Local},
    state::file_id::{FileId, FileIdOrd},
};

pub struct TreeSitter {
    pub(crate) inotify_watch_list: Watches,
    pub(crate) descriptors_to_file_ids: FxHashMap<i32, FileIdOrd>,
    pub(crate) file_ids_to_records: FxHashMap<FileId, FileRecord>,
    pub(crate) root: AbsPath<Local>,
    pub(crate) ignore: Gitignore,
    pub(crate) db: LocalReader,
    pub(crate) output_tx: DeltaSender,
}

impl TreeSitter {
    pub fn start(root: AbsPath<Local>, ignore: Gitignore, db: LocalReader) -> DeltaReceiver {
        let inotify = Inotify::init().unwrap();

        let (output_tx, output_rx) = mpsc::channel(4);
        let mut tree_sitter = Self {
            descriptors_to_file_ids: FxHashMap::default(),
            file_ids_to_records: FxHashMap::default(),
            inotify_watch_list: inotify.watches(),
            root: root.clone(),
            ignore,
            output_tx,
            db,
        };
        
        tokio::spawn(async move {
            tree_sitter.subscribe_root(root).await;
            tree_sitter.watch(inotify).await;
        });
        output_rx
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FileKV {
    pub id: FileIdOrd,
    pub record: FileRecord,
}

impl FileKV {
    pub fn from(parent_id: FileIdOrd, name: CompactString) -> Self {
        let id = parent_id.child(&name);
        let record = FileRecord {
            parent_id: *parent_id,
            name,
        };
        Self { id, record }
    }
    pub fn into_delta_fn(kind: DeltaKind, index:u16) -> impl Fn(Self) -> (FileIdOrd, Delta) {
        move |file| {
            let delta = Delta {
                file: file.record,
                kind,
                index,
            };
            (file.id, delta)
        }
    }
}


