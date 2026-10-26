use std::{io, sync::Arc};

use async_walkdir::DirEntry;
use futures::{Stream, stream};
use log::warn;
use tokio_stream::StreamExt as _;

use crate::{context::Context, notify::{EventKind, event::{Event, EventData}, walk_dir::walk_dir}, util::{arc_path::{ArcPath, IntoArcPath}, globs::Globs}};

pub async fn find_missed_child_create_events(
    events: impl IntoIterator<Item = (&ArcPath, &EventData)>,
    exclude: Arc<Globs>,
) -> Vec<Event> {
    let new_dir_events = events
        .into_iter()
        .filter(|(_, event)| event.kind.delta().is_create() && event.is_dir)
        .map(|(path, _)| path.clone());

    let new_children = find_children(new_dir_events, exclude);
    new_children.then(entry_into_create_event).collect().await
}


pub fn find_children(
    dirs: impl IntoIterator<Item = ArcPath>,
    exclude: Arc<Globs>,
) -> impl Stream<Item = DirEntry> {
    let streams = dirs.into_iter().map(|path| walk_dir(path, exclude.clone()));

    stream::select_all(streams).filter_map(|res| res.inspect_err(|err| warn!("failed to walk children: {err}")).ok())
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
