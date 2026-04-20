#!/usr/bin/env cargo

---
[dependencies]
rustc-hash = "2.1.1"
---
// cargo +nightly -Zscript scripts/script.rs
use std::hash::{Hash, Hasher};
use rustc_hash::FxHasher;

fn main() {

    
    let mut hasher = FxHasher::default();
    "dog".hash(&mut hasher);
    dbg!(hasher.finish());
    
    let mut hasher = FxHasher::default();
    "dog".hash(&mut hasher);
    "dog".hash(&mut hasher);
    dbg!(hasher.finish());
    
    
    let mut hasher = FxHasher::default();
    "dog".hash(&mut hasher);
    "dog".hash(&mut hasher);
    "dog".hash(&mut hasher);
    dbg!(hasher.finish());
}