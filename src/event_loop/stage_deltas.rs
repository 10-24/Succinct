
use redb::WriteTransaction;
use rustc_hash::FxHashMap;

use crate::{context::ContextInner, database::{table::{Entry, StagedDeltas}, table_trait::Txn as _}, notify::event::EventData, util::arc_path::ArcPath};




pub fn stage_deltas(ctx: &ContextInner, events: FxHashMap<ArcPath, EventData>,) {
    let tx = ctx.db.staged_deltas.begin_write().unwrap();
    {
        let mut table = tx.open::<StagedDeltas>().unwrap();
        for (e_path, e_data) in events {
            let rel_path = e_path.strip_prefix(ctx.config.sync_dir.as_str()).unwrap().as_str();
    
            let new_entry = Entry {
                modified_at: e_data.timestamp.timestamp_millis(),
                is_deleted: e_data.kind.is_delete(),
            };
            
            table.insert(rel_path,new_entry).unwrap();
        }
    }
    tx.commit().unwrap()
}
