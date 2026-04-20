use std::sync::Arc;

use globset::GlobSet;
use inotify::{Inotify, Watches};
use rustc_hash::FxHashMap;
use tokio::sync::mpsc;

use crate::{
    db::Db,
    delta::{DeltaReceiver, DeltaSender,},
    path::{AbsPath, Local},
};
use crate::db::tables::file::FileId;

pub struct TreeSitter {
    pub(crate) inotify_watch_list: Watches,
    pub(crate) descriptors: FxHashMap<i32, FileId>,
    pub(crate) root: AbsPath<Local>,
    pub(crate) ignore:Arc<GlobSet>,
    pub(crate) db: Db,
    pub(crate) output_tx: DeltaSender,
}

impl TreeSitter {
    pub fn start(root: AbsPath<Local>, ignore: GlobSet, db: Db) -> DeltaReceiver {
        let inotify = Inotify::init().unwrap();
        
        let (output_tx, output_rx) = mpsc::channel(4);
        let mut tree_sitter = Box::from(Self {
            descriptors: FxHashMap::default(),
            inotify_watch_list: inotify.watches(),
            root: root.clone(),
            ignore,
            output_tx,
            db,
        });

        tokio::spawn(async move {
            tree_sitter.subscribe_subtree().await;
            tree_sitter.watch(inotify).await;
        });
        output_rx
    }
}


