use std::{borrow::Cow, cmp::Ordering};

use camino::{Utf8Path, Utf8PathBuf};
use redb::{Key, TypeName, Value};
use ref_cast::RefCast;

#[derive(Debug, RefCast)]
#[repr(transparent)]
pub struct ObjPath(str);

impl ObjPath {

    /// Returns None If base is not a prefix of self 
    pub fn without_prefix<'a>(base: &Utf8Path, path: &'a Utf8Path) -> Option<&'a Self>{
   
        let path_str = path.strip_prefix(base).ok()?.as_str().trim_end_matches('/');
        
        let object_path = Self::ref_cast(path_str);
        
        Some(object_path)
    }

    pub fn with_prefix(&self, base: impl Into<Utf8PathBuf>) -> Utf8PathBuf {
        let mut base = base.into();
        let suffix: &Utf8Path = self.into();
        base.push(suffix);
        base
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }

}

impl<'a> From<&'a ObjPath> for &'a Utf8Path {
    fn from(value: &'a ObjPath) -> Self {
        Utf8Path::new(value.as_str())
    }
}



impl Value for &ObjPath {
    type SelfType<'a>
        = &'a str
    where
        Self: 'a;
    type AsBytes<'a>
        = &'a str
    where
        Self: 'a;


    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> &'a str
    where
        Self: 'a,
    {
        core::str::from_utf8(data).unwrap()
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> &'a str
    where
        Self: 'b,
    {
        value
    }

    fn type_name() -> TypeName {
        TypeName::new("&ObjectPath")
    }
}

impl Key for &ObjPath {
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        let str1 = Self::from_bytes(data1);
        let str2 = Self::from_bytes(data2);
        str1.cmp(str2)
    }

    fn separator<'a>(left: &'a [u8], right: &'a [u8]) -> Cow<'a, [u8]> {
        debug_assert!(left < right);
        // `str` orders as its bytes, but `compare()` deserializes, so the cut must keep it valid
        let common_bytes = left.iter().zip(right).take_while(|(x, y)| x == y).count();
        let separator_len = round_up_to_char_boundary(right, common_bytes + 1);
        if separator_len < left.len() && separator_len < right.len() {
            Cow::Borrowed(&right[..separator_len])
        } else {
            Cow::Borrowed(left)
        }
    }

    // The empty string is valid UTF-8, and sorts below every other one
    fn min_encoded_key() -> Option<Cow<'static, [u8]>> {
        Some(Cow::Borrowed(&[]))
    }
}

// `index` advanced to the next character boundary, or the end. Only a character's trailing bytes
// match `10xx_xxxx`, so this needs no deserialization, unlike `str::is_char_boundary()`.
fn round_up_to_char_boundary(utf8: &[u8], mut index: usize) -> usize {
    while index < utf8.len() && utf8[index] & 0b1100_0000 == 0b1000_0000 {
        index += 1;
    }
    index
}
