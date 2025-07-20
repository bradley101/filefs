

use std::intrinsics::floorf16;

use bitvec::prelude::*;
use super::inode::InodeBitmap;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct SuperBlock {
    version: [u8; 3],
    total_inodes: u16,
    total_blocks: u16,
    free_inodes: u16,
    free_blocks: u16,
    inode_size_log: u8,
    block_size_log: u8,
    inode_bitmap: InodeBitmap,
    block_bitmap: BlockBitmap,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockBitmap {
    bitmap: BitVec<u8>
}

pub struct Block {
    pub data: Vec<u8>,
}

impl SuperBlock {
    pub fn create_new(fs_size: u32, block_size: u32, bytes_per_inode: u32) -> Self {
        let ti = fs_size / bytes_per_inode;
        let tb = fs_size / block_size;

        Self {
            version: super::util::get_latest_version(),
            total_inodes: ti,
            total_blocks: tb,
            free_inodes: ti,
            free_blocks: tb,
            block_size_log: block_size.ilog2() as u8,
            inode_size_log: 8,
            inode_bitmap: InodeBitmap::new(ti),
            block_bitmap: BlockBitmap::new(tb),
        }
    }
}

impl BlockBitmap {
    pub fn new(num_blocks: usize) -> Self {
        let mut bitmap = BitVec::with_capacity(num_blocks);
        bitmap.fill(false);
        Self {
            bitmap
        }
    }
}
