use crate::state::file::File;

pub type FileBytes = [u8; File::SIZE];

impl File {
    pub fn from_bytes(bytes: &FileBytes) -> &Self {
        bytemuck::from_bytes(bytes)
    }
    pub fn as_bytes(&self) -> &FileBytes {
        bytemuck::bytes_of(self).try_into().unwrap()
    }
}