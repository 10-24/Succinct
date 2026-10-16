use std::sync::Arc;

use anyhow::Result;
use futures::stream;
use globset::GlobSet;
use rustc_hash::FxHashMap;
use tokio::io;
use tokio_stream::StreamExt as _;

use crate::{
    context::ContextInner,
    notify::{
        EventKind,
        event::{Event, EventData},
        walk_dir::walk_dir,
    },
    util::arc_path::{ArcPath, IntoArcPath},
};

pub struct FilteredEvents {
    pub new_dirs: Vec<ArcPath>,
    pub events: FxHashMap<ArcPath, EventData>,
}

pub async fn filter_events(
    ctx: &ContextInner,
    all_events: impl Iterator<Item = Event>,
) -> Result<FilteredEvents> {
    let sync_dir = ctx.config.sync_dir.as_std_path();
    
    let mut events = FxHashMap::default();

    
    for (event_path, event_data) in all_events {
        let relpath = event_path.strip_prefix(sync_dir).unwrap();
        if ctx.exclude.is_match(relpath) {
            continue;
        }
    
        events
            .entry(event_path.into_arc_path())
            .and_modify(|current| *current = merge_events(*current, event_data))
            .or_insert(event_data);
    }

    let mut new_dirs = filter_new_dirs(events.iter());
    if new_dirs.is_empty() {
        return Ok(FilteredEvents {
            new_dirs,
            events,
        })
    }
    
    let missed_events = fetched_missed_events(&new_dirs, &ctx.exclude).await?;
    let new_missed_dirs = filter_new_dirs(missed_events.iter().map(to_refs));
    new_dirs.extend(new_missed_dirs);
    events.extend(missed_events);
    

    Ok(FilteredEvents { new_dirs, events })
}

async fn fetched_missed_events(
    dirs: &[ArcPath],
    exclude: &Arc<GlobSet>,
) -> io::Result<Vec<(ArcPath, EventData)>> {
    let streams = dirs.iter().map(|d| walk_dir(d.clone(), exclude.clone()));

    let events = stream::select_all(streams).filter_map(Result::ok)
        .then(|entry| async move {
            let metadata = entry.metadata().await.unwrap();
            let data = EventData {
                kind: EventKind::Create,
                timestamp: metadata.modified().unwrap().into(),
                is_dir: metadata.is_dir(),
            };
            (entry.path().into_arc_path(), data)
        })
        .collect::<Vec<_>>()
        .await;

    Ok(events)
}

/// Whether to overwrite the event_kind of the current entry. States marked with a `?` are theoretically impossible.
///
/// | current \ new | Create | Update | Delete |
/// |-------------|--------|--------|--------|
/// | Create      | 1?     | 0      | 1      |
/// | Update      | 1      | 1      | 1      |
/// | Delete      | 1      | 1?     | 1?     |
fn merge_events(current: EventData, new: EventData) -> EventData {
    match (current.kind, new.kind) {
        (EventKind::Create, EventKind::Update) => EventData {
            kind: EventKind::Create,
            ..new
        },
        _ => EventData { ..new },
    }
}


fn filter_new_dirs<'a>(events: impl Iterator<Item=(&'a ArcPath, &'a EventData)>) -> Vec<ArcPath> {
   events.filter_map(|(path,event_data)| {
       if event_data.is_dir && event_data.kind.is_create() {
           return Some(path.clone())
       }
       None
   }).collect()
}

fn to_refs<K,V>(tuple: &(K,V)) -> (&K,&V) {
    (&tuple.0, &tuple.1)
}