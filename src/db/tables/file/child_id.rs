use crate::db::tables::file::{FileId, FileName};

impl FileId {
    
    pub const ROOT: FileId = Self::compute_root();
    
    /// Max number of top bits inherited from the parent. See [FileId::random_shift]
    /// On average a child will inherit 6.0078125 bits
    const OLD_BITS:u32 = u8::BITS as u32;
    /// Min number of bottom bits unique to all children.
    const YOUNG_BITS:u32 = (u64::BITS - u8::BITS) as u32;
    

    pub fn child(&self, name: &FileName) -> Self {
        let delta = self.hash_combine(name);
        let mutations = delta >> Self::random_shift(delta as u8);
        Self(self.0 ^ mutations)
    }
    
    #[inline]
    /// https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function
    fn hash_combine(&self, name: &FileName) -> u64 {
        let name_bytes = name.as_bytes().iter();
        let self_bytes = self.0.to_ne_bytes();
        let self_bytes = self_bytes.iter();
        
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in name_bytes.chain(self_bytes) {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash ^= hash >> 33;
        hash = hash.wrapping_mul(0xff51afd7ed558ccd);
        hash ^= hash >> 33;
        
        hash
    }

    #[inline]
    const fn random_shift(n: u8) -> u32 {
        Self::OLD_BITS - n.trailing_ones()
    }
    
    /// Creates a hash-friendly midpoint
    const fn compute_root() -> FileId {
        let old = 2_u64.pow(Self::OLD_BITS - 1);
        let young = 0x7FFFFFFFFF8D;
        let root = (old << Self::YOUNG_BITS) | young;
        
        Self(root)
    }
}
