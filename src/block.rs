

const BLOCK_SIZE: usize = 4 << 10;
const BLOCK_DATA_SIZE: usize = BLOCK_SIZE - 8;

use bitvec::prelude::*;

pub struct SuperBlock {
    version: u16,
    total_inodes: u16,
    total_blocks: u16,
    free_inodes: u16,
    free_blocks: u16,
    inode_size_log: u8,
    block_size_log: u8,
}

pub struct InodeBitmap {
    bitmap: BitVec<u8>
}

pub struct BlockBitmap {
    bitmap: BitVec<u8>
}

pub struct Block {
    pub block_number: u32,
    pub data: [u8; BLOCK_DATA_SIZE],
    pub next_block_number: u32
}

