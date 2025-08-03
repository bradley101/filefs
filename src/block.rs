

use core::num;
use std::{fs::File, io::Write};

use bitvec::prelude::*;
use super::inode::InodeBitmap;

pub const SUPER_BLOCK_FILE_OFFSET: u64 = 0;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct SuperBlock {
    version: [u8; 3],
    total_inodes: u16,
    total_blocks: u16,
    free_inodes: u16,
    free_blocks: u16,
    inode_size_log: u8,
    block_size_log: u8,
    // For Now - We will use block 2 for inode bitmap and block 3 for block bitmap
    // inode_bitmap_block_count: u8, // This field is not used in the current implementation
    // block_bitmap_block_count: u8, // This field is not used in the current implementation
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct BlockBitmap {
    bitmap: BitVec<u8>
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Block {
    pub data: Vec<u8>,
}

impl SuperBlock {
    pub fn create_new(fs_size: u32, block_size: u32, bytes_per_inode: u32) -> Self {
        let ti = (fs_size / bytes_per_inode) as u16 ;
        let tb = (fs_size / block_size) as u16;

        Self {
            version: super::util::get_latest_version(),
            total_inodes: ti,
            total_blocks: tb,
            free_inodes: ti,
            free_blocks: tb,
            block_size_log: block_size.ilog2() as u8,
            inode_size_log: 8,
        }
    }

    // pub fn persist(&self, file: &mut File) -> std::io::Result<()> {
    //     // use serde to serialize and write this superblock in the file
    //     let serialized = bincode::serialize(self).expect("Failed to serialize SuperBlock");
    //     file.write_all(serialized.as_slice())
    // }

    pub fn get_total_inodes(&self) -> usize {
        self.total_inodes as usize
    }
}

impl BlockBitmap {
    pub fn new(num_blocks: usize) -> Self {
        let mut bitmap = bitvec![u8, Lsb0; 0; num_blocks];
        bitmap.fill(false);
        Self {
            bitmap
        }
    }
}
