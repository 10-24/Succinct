use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};
use std::ops::RangeInclusive;
use std::simd::prelude::*;
use std::{array, u64};

use crate::db::tables::file::{FileId, FileName};

impl FileId {
    /// Inclusive
    const RANGE: RangeInclusive<u8> = 2..=6;
    const RANGE_WIDTH: u8 = Self::mask_offset(*FileId::RANGE.end()) + *FileId::RANGE.end();
    const MASKS: Simd<u64, 8> = u64x8::from_array([
        Self::create_mask(0 + *Self::RANGE.start()),
        Self::create_mask(1 + *Self::RANGE.start()),
        Self::create_mask(2 + *Self::RANGE.start()),
        Self::create_mask(3 + *Self::RANGE.start()),
        Self::create_mask(4 + *Self::RANGE.start()),
        0,
        0,
        0,
    ]);
    const OFFSETS: Simd<u64, 8> = u64x8::from_array([
        Self::mask_offset(0 + *Self::RANGE.start()) as u64,
        Self::mask_offset(1 + *Self::RANGE.start()) as u64,
        Self::mask_offset(2 + *Self::RANGE.start()) as u64,
        Self::mask_offset(3 + *Self::RANGE.start()) as u64,
        Self::mask_offset(4 + *Self::RANGE.start()) as u64,
        0,
        0,
        0,
    ]);
    pub fn child(&self, name: &FileName) -> Self {
        let mut hasher = FxHasher::default();
        name.as_str().hash(&mut hasher);
        let hash = hasher.finish();

        let components = (u64x8::splat(hash) & Self::MASKS) >> Self::OFFSETS;
        let (a, b, c) = Self::deduplicate(components);
        let mutated_indecies = join4([a,b,c, u64x8::default()]);
        let mutation = (u64x32::splat(1) << mutated_indecies).reduce_xor();
        Self(self.0 ^ mutation)
    }

    #[inline]
    const fn create_mask(n: u8) -> u64 {
        let offset = Self::mask_offset(n);
        let mask = mask(n) << offset;
        duplicate(mask)
    }

    #[inline]
    const fn mask_offset(n: u8) -> u8 {
        const fn trianglar(n: u8) -> u8 {
            (n * (n + 1)) / 2
        }
        trianglar(n - 1) - trianglar(*Self::RANGE.start() - 1)
    }

    #[inline]
    const fn duplicate(n: u32) -> u64 {
        let n = n as u64;

        let a = n << (0 * Self::RANGE_WIDTH);
        let b = n << (1 * Self::RANGE_WIDTH);
        let c = n << (2 * Self::RANGE_WIDTH);
        a | b | c
    }

    #[inline]
    fn deduplicate(n: Array) -> (Array,Array,Array) {
        const U20_MASK: Array = u64x8::splat(u64::MAX >> (64 - 20));
        const RANGE_WIDTH: u64 = FileId::RANGE_WIDTH as u64;
        const OFFSET: (Array, Array, Array) = (
            u64x8::splat(0 * RANGE_WIDTH),
            u64x8::splat(1 * RANGE_WIDTH),
            u64x8::splat(2 * RANGE_WIDTH),
        );
        
        let a = (n >> OFFSET.0) & U20_MASK;
        let b = (n >> OFFSET.1) & U20_MASK;
        let c = (n >> OFFSET.2) & U20_MASK;
        (a, b, c)
    }
}

#[inline]
fn mask(digits: u8) -> u32 {
    u32::MAX >> (32 - digits)
}

fn join4(arrays:[Array;4]) -> Array32 {
    let mut result = [u64::default();32];
    
    for (i, a) in arrays.into_iter().enumerate() {
        let start = i * 8;
        let end = start + 8;
        result[start..end].copy_from_slice(a.as_array());
    }
    u64x32::from_array(result)
}
type Array = Simd<u64, 8>;
type Array32 = Simd<u64, 32>;