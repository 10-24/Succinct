use std::{
    collections::VecDeque,
    hash::{Hash, Hasher},
    mem,
    sync::Arc,
    time::{Duration, SystemTime},
};

use rustc_hash::FxHasher;
use tokio::sync::Mutex;

struct Entry<T> {
    hash: u64,
    expiration: SystemTime,
    item: T,
}

struct EphemeralItems<T: Hash>(Vec<Entry<T>>);

impl<T: Hash + Eq> EphemeralItems<T> {
    pub fn insert(&mut self, item: T, lifetime: Duration) {
        let new_entry = Entry {
            hash: Self::hash(&item),
            expiration: SystemTime::now() + lifetime,
            item,
        };
        self.0.push(new_entry);
    }

    pub fn extract_matches<'a, I>(&mut self, targets: I) -> Vec<T>
    where
        I: IntoIterator<Item = &'a T>,
        T: 'a,
    {
        let now = SystemTime::now();
        let mut targets = targets
            .into_iter()
            .map(|item| (Self::hash(&item), item))
            .collect::<Vec<_>>();

        let mut removed_entries = Vec::new();

        let mut i = 0;
        while i < self.0.len() {
            if self.0[i].expiration <= now {
                self.0.swap_remove(i);
                continue;
            }

            let candidate = (self.0[i].hash, &self.0[i].item);
            if extract(&mut targets, |t| *t == candidate).is_some() {
                let entry = self.0.swap_remove(i);
                removed_entries.push(entry.item);
                continue;
            };

            i += 1;
        }
        removed_entries
    }

    fn hash(item: &T) -> u64 {
        let mut state = FxHasher::default();
        item.hash(&mut state);
        state.finish()
    }
}

fn extract<T: Eq>(vec: &mut Vec<T>, predicate: impl Fn(&T) -> bool) -> Option<T> {
    let index = vec.iter().position(predicate)?;
    Some(vec.swap_remove(index))
}

#[derive(Clone)]
pub struct EphemeralSender<T: Hash + Eq> {
    items: Arc<Mutex<EphemeralItems<T>>>,
    lifetime: Duration,
}

impl<T: Hash + Eq> EphemeralSender<T> {

    pub async fn insert(&self, new_item: T) {
        let mut guard = self.items.lock().await;
        guard.insert(new_item, self.lifetime);
    }
}

pub struct EphemeralReceiver<T: Hash + Eq>(Arc<Mutex<EphemeralItems<T>>>);

impl<T: Hash + Eq> EphemeralReceiver<T> {
    pub async fn extract_matches<'a, I>(&mut self, targets: I) -> Vec<T>
    where
        I: IntoIterator<Item = &'a T>,
        T: 'a,
    {
        let mut guard = self.0.lock().await;
        guard.extract_matches(targets)
    }
}

pub fn ephemeral_channel<T: Hash + Eq>(
    lifetime: Duration,
) -> (EphemeralSender<T>, EphemeralReceiver<T>) {
    let items = EphemeralItems(Vec::new());
    let items = Arc::from(Mutex::from(items));

    let rx = EphemeralReceiver(items.clone());
    let tx = EphemeralSender { lifetime, items };
    (tx, rx)
}
