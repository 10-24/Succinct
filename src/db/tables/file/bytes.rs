use redb::{Value, TypeName};
use crate::state::file::File;

impl Value for &File {
    type SelfType<'a> = &'a File where Self: 'a;

    type AsBytes<'a> = &'a [u8] where Self: 'a;

    fn fixed_width() -> Option<usize> {
        Some(File::SIZE)
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a 
    {
        // Using bytemuck for a safe zero-copy cast
        bytemuck::from_bytes(data)
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b 
    {
        bytemuck::bytes_of(*value)
    }

    fn type_name() -> TypeName {
        TypeName::new("File")
    }
}