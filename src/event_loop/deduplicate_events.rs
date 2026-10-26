use std::sync::Arc;

use anyhow::Result;
use async_walkdir::DirEntry;

use futures::{Stream, stream};
use globset::GlobSet;
use log::warn;
use rustc_hash::{FxHashMap, FxHashSet};
use tokio::pin;
use tokio_stream::StreamExt as _;

use crate::{
    context::Context,
    notify::{
        EventKind,
        event::{Event, EventData},
        event_kind::DeltaKind,
        walk_dir::walk_dir,
    },
    util::{
        arc_path::{ArcPath, IntoArcPath},
        globs::Globs,
    },
};



pub fn deduplicate_events(
    new_events: impl IntoIterator<Item = Event>,
    mut existing_events: FxHashMap<ArcPath, EventData>,
) -> FxHashMap<ArcPath, EventData> {
    for (path, event) in new_events {
        existing_events
            .entry(path)
            .and_modify(|current| *current = merge(*current, event))
            .or_insert(event);
    }
    
    existing_events
}

/// Whether to overwrite the event_kind of the current entry. States marked with a `?` are theoretically impossible.
///
/// | current \ new | Create | Update | Delete |
/// |-------------|--------|--------|--------|
/// | Create      | 1?     | 0      | 1      |
/// | Update      | 1      | 1      | 1      |
/// | Delete      | 1      | 1?     | 1?     |
fn merge(current: EventData, new: EventData) -> EventData {
    match (current.kind.delta(), new.kind.delta()) {
        (DeltaKind::Create, DeltaKind::Update) => EventData {
            kind: EventKind::Create,
            ..new
        },
        _ => EventData { ..new },
    }
}



pub fn find_children(
    dirs: impl IntoIterator<Item = ArcPath>,
    exclude: Arc<Globs>,
) -> impl Stream<Item = DirEntry> {
    let streams = dirs.into_iter().map(|path| walk_dir(path, exclude.clone()));

    stream::select_all(streams).filter_map(|res| match res {
        Ok(entry) => Some(entry),
        Err(err) => {
            warn!("failed to walk children: {err}");
            None
        }
    })
}

async fn entry_into_create_event(entry: DirEntry) -> Event {
    let path = entry.path().into_arc_path();
    let metadata = entry.metadata().await.unwrap();
    let event = EventData {
        kind: EventKind::Create,
        is_dir: metadata.is_dir(),
        timestamp: metadata.modified().unwrap().into(),
    };
    (path, event)
}
