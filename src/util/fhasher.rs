use std::hash::{Hash, Hasher};

use rustc_hash::{FxHasher};

pub struct FHasher(FxHasher);

impl FHasher {
    pub fn new() -> Self {
        Self(FxHasher::default())
    }
    
    pub fn hash(mut self, value: impl Hash) -> Self {
        value.hash(&mut self.0);
        self
    }

    pub fn finish(self) -> u64 {
        self.0.finish()
    }
}
