

use core::num;
use std::{fs::File, io::Write};

use bitvec::prelude::*;
use crate::inode::INODE_SIZE;

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
    inode_bitmap_block_count: u8,
    block_bitmap_block_count: u8,
    inode_start_block: u16,
    total_inode_blocks: u16,
}

#[derive(Clone, Default)]
pub struct BlockBitmap {
    bitmap: BitVec<u8>
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Block {
    pub block_number: u16,
    pub data: Vec<u8>,
}

impl SuperBlock {
    pub fn create_new(fs_size: u32, block_size: u32, bytes_per_inode: u32) -> Self {
        let ti = (fs_size / bytes_per_inode) as u16 ;
        let tb = (fs_size / block_size) as u16;
        let inode_block_count = (ti as usize * INODE_SIZE) / block_size as usize;
        let inode_bitmap_block_count = ti / 8 / block_size as u16;
        let block_bitmap_block_count = tb / 8 / block_size as u16;

        Self {
            version: super::util::get_latest_version(),
            total_inodes: ti,
            total_blocks: tb,
            free_inodes: ti,
            free_blocks: tb,
            block_size_log: block_size.ilog2() as u8,
            inode_size_log: INODE_SIZE.ilog2() as u8,
            inode_bitmap_block_count: inode_bitmap_block_count as u8,
            block_bitmap_block_count: block_bitmap_block_count as u8,
            inode_start_block: inode_bitmap_block_count + block_bitmap_block_count + 1, // 1 for superblock
            total_inode_blocks: inode_block_count as u16,
        }
    }

    pub fn persist(&self, file: &mut File) -> std::io::Result<()> {
        // use serde to serialize and write this superblock in the file
        let buffer = self.serialize();
        file.write_all(buffer.data.as_slice())
    }

    #[inline(always)]
    pub fn get_total_inodes(&self) -> usize {
        self.total_inodes as usize
    }

    #[inline(always)]
    pub fn get_total_blocks(&self) -> usize {
        self.total_blocks as usize
    }

    #[inline(always)]
    pub fn get_block_size(&self) -> usize {
        1 << self.block_size_log
    }

    fn serialize(&self) -> Block {
        let mut buffer: Vec<u8> = Vec::new();
        // serialize all the fields of the superblock into buffer
        buffer.extend_from_slice(&self.version);
        buffer.extend_from_slice(&self.total_inodes.to_le_bytes());
        buffer.extend_from_slice(&self.total_blocks.to_le_bytes());
        buffer.extend_from_slice(&self.free_inodes.to_le_bytes());
        buffer.extend_from_slice(&self.free_blocks.to_le_bytes());
        buffer.push(self.inode_size_log);
        buffer.push(self.block_size_log);
        buffer.push(self.inode_bitmap_block_count);
        buffer.push(self.block_bitmap_block_count);
        buffer.extend_from_slice(&self.inode_start_block.to_le_bytes());
        buffer.extend_from_slice(&self.total_inode_blocks.to_le_bytes());

        Block {
            block_number: 0, // Superblock is always at block number 0
            data: buffer,
        }
    }
}

impl BlockBitmap {
    pub fn new(num_blocks: usize) -> Self {
        let mut bitmap = bitvec![u8, Lsb0; 0; num_blocks];
        bitmap.fill(false);
        Self {
            bitmap: bitmap
        }
    }

    fn serialize(&self, super_block_ref: &SuperBlock) -> Vec<Block> {
        let mut blocks: Vec<Block> = Vec::new();
        let total_bitmap_blocks = super_block_ref.block_bitmap_block_count as usize;
        let bitmap_vec = self.bitmap.as_raw_slice();

        for i in 0..total_bitmap_blocks {
            let start = i * super_block_ref.get_block_size();
            let end = start + super_block_ref.get_block_size();
            let data = if end > bitmap_vec.len() {
                &bitmap_vec[start..]
            } else {
                &bitmap_vec[start..end]
            };
            blocks.push(Block {
                block_number: i as u16,
                data: data.to_vec(),
            });
        }        

        blocks
    }

    fn serialize_to_vec(&self) -> Vec<u8> {
        self.bitmap.as_raw_slice().to_vec()
    }
}
