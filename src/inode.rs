
pub const MAX_FILE_NAME_SIZE: usize = 64;
pub const MAX_CHILDREN_COUNT: usize = 64;

pub const INODE_SIZE: usize = 256;
pub const USABLE_INODE_SIZE: usize = 2 
                                + 2
                                + MAX_FILE_NAME_SIZE 
                                + 2
                                + 1
                                + 2
                                + (2 * MAX_CHILDREN_COUNT);

use bitvec::prelude::*;
use super::block::BlockBitmap;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum FileType {
    #[default]
    File = 0_u8,
    Directory = 1
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct Inode {
    pub inode_number: u16,
    pub parent: u16,
    pub name: String,
    pub data_blocks: [u16; 32],
    pub block_bitmap: BlockBitmap,
    pub file_type: FileType,
    pub file_size: u32,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct InodeBitmap {
    bitmap: BitVec<u8>
}

impl InodeBitmap {
    pub fn new(num_inodes: usize) -> Self {
        let mut bitmap = BitVec::with_capacity(num_inodes);
        bitmap.fill(true);
        Self {
            bitmap
        }
    }

    pub fn persist(&self, file: &mut std::fs::File) -> std::io::Result<()> {
        // Serialize the bitmap and write it to the file
        let serialized = bincode::serialize(self).expect("Failed to serialize InodeBitmap");
        file.write_all(serialized.as_slice())
    }

    fn serialize(&self) -> Vec<u8> {
        let buffer: Vec<u8> = Vec::new();
        buffer.extend_from_slice(self.bitmap.);
    }
}


