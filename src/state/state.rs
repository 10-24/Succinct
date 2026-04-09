use crate::database::QueuedUpdate;
use crate::database::file_id::FileId;

use crate::state::file_id::FileIdOrd;
use crate::state::prepare_deltas::DeltaKV;
use crate::state::remote_drive::RemoteDrive;
use crate::{
    database::{QueuedDeletion, local_reader::Db, local_writer::DbWriter},
    delta::{Delta, DeltaKind},
    path::{AbsPath, Local},
    state::file::File,
};
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

pub struct State {
    pub(crate) local_writer: DbWriter,
    pub(crate) local_reader: Db,
    pub(crate) remote_drive: opendal::Operator,
    pub(crate) local_root: AbsPath<Local>,
}

impl State {
    pub fn new(
        local_writer: DbWriter,
        local_reader: Db,
        remote_drive: opendal::Operator,
        local_root: AbsPath<Local>,
    ) -> Self {
        Self {
            local_writer,
            local_reader,
            remote_drive,
            local_root,
        }
    }

    /// File and Folder creations cannot be implicit (Eg. If a parent and child file/folder are created, both must be included)
    pub async fn push_deltas(&mut self, deltas: BTreeMap<FileIdOrd, Delta>) {
        let timestamp = Utc::now();

        self.ensure_files_exist(&deltas, timestamp);

        let mut deltas = self.complete_tree(deltas);
        while let Some(delta) = deltas.pop_last() {
            match delta.1.kind {
                DeltaKind::Update | DeltaKind::Create => self.handle_modify(delta, timestamp),
                DeltaKind::Delete => self.handle_delete(delta, timestamp),
            }
        }
    }

    /// Handles both Update and Create
    fn handle_modify(&self, delta: DeltaKV, modified_at: DateTime<Utc>) {
        let file_id = delta.0.value;

        let child_hashes = self.local_reader.get_file_child_hashes(file_id);
        let file_hash = File::calculate_hash(file_id, modified_at, child_hashes.into_iter());
        self.local_writer
            .update_file(file_id, file_hash, modified_at);

        let queued_update = QueuedUpdate {
            file_id,
            depth: delta.0.depth,
        };
        self.local_writer.enqueue_update(queued_update);
    }

    fn handle_delete(&self, delta: DeltaKV, deleted_at: DateTime<Utc>) {
        let file_id = delta.0.value;

        let descendants = self.local_reader.get_file_descendants(file_id);
        let subtree = descendants.into_iter().chain(Some(delta.0));

        for file_id in subtree {
            self.local_writer.delete_file(*file_id);

            let queued_deletion = QueuedDeletion {
                deleted_at,
                file_id: *file_id,
            };
            self.local_writer.enqueue_delete(queued_deletion);
        }
    }
}

fn floor_mul(x: usize, y: f32) -> usize {
    let x = x as f32;
    let prod = x * y;
    prod as usize
}

struct FullTrace {
    id: FileId,
    ancestors: Vec<FileId>,
    descendants: Vec<FileId>,
}
