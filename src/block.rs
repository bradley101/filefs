

use std::{cmp::max, fs::File, io::{Seek, SeekFrom, Write}, os::unix::fs::FileExt};

use bitvec::prelude::*;
use crate::inode::INODE_SIZE;

use super::inode::INODE_BITMAP_STARTING_BLOCK_NUMBER;

pub const SUPER_BLOCK_FILE_OFFSET: u64 = 0;
pub const SUPER_BLOCK_SIZE: usize = 1 << 8;

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
        let inode_bitmap_block_count = max(1, ti / 8 / block_size as u16);
        let block_bitmap_block_count = max(1, tb / 8 / block_size as u16);

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
        let buffer = self.serialize();
        file.write_all_at(buffer.data.as_slice(), SUPER_BLOCK_FILE_OFFSET)
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

    #[inline(always)]
    pub fn get_inode_bitmap_block_count(&self) -> usize {
        self.inode_bitmap_block_count as usize
    }

    #[inline(always)]
    pub fn get_inode_start_block(&self) -> usize {
        self.inode_start_block as usize
    }

    pub fn get_block_bitmap_block_count(&self) -> usize {
        self.block_bitmap_block_count as usize
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
        // buffer.resize(self.get_block_size(), 0);

        Block {
            block_number: 0, // Superblock is always at block number 0
            data: buffer,
        }
    }

    pub fn deserialize(file: &mut File) -> Result<SuperBlock, std::io::Error> {
        let mut block = Block::default();
        block.data.resize(SUPER_BLOCK_SIZE, 0);

        let tmp_res =
            file.read_exact_at(block.data.as_mut_slice(), SUPER_BLOCK_FILE_OFFSET);
        
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        SuperBlock::deserialize_block(block)
    }

    fn deserialize_block(block: Block) -> Result<SuperBlock, std::io::Error> {
        let bytes = block.data.as_slice();
        
        Ok(SuperBlock {
            version: [bytes[0], bytes[1], bytes[2]],
            total_inodes: u16::from_le_bytes([bytes[3], bytes[4]]),
            total_blocks: u16::from_le_bytes([bytes[5], bytes[6]]),
            free_inodes: u16::from_le_bytes([bytes[7], bytes[8]]),
            free_blocks: u16::from_le_bytes([bytes[9], bytes[10]]),
            inode_size_log: bytes[11],
            block_size_log: bytes[12],
            inode_bitmap_block_count: bytes[13],
            block_bitmap_block_count: bytes[14],
            inode_start_block: u16::from_le_bytes([bytes[15], bytes[16]]),
            total_inode_blocks: u16::from_le_bytes([bytes[17], bytes[18]]),
        })
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

    pub fn persist(&self, file: &mut std::fs::File, super_block_ref: &SuperBlock) -> std::io::Result<()> {
        let blocks = self.serialize(super_block_ref);

        let tmp_res = 
            file.seek(SeekFrom::Start(blocks[0].block_number as u64 * super_block_ref.get_block_size() as u64));
        
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        for block in blocks {
            let tmp_res = file.write_all(&block.data);
            if tmp_res.is_err() {
                return Err(tmp_res.err().unwrap());
            }
        }

        Ok(())
    }

    pub fn fetch(file: &mut std::fs::File, super_block_ref: &SuperBlock) -> std::io::Result<Self> {
        let total_block_bitmap_blocks = super_block_ref.get_block_bitmap_block_count();
        let mut blocks: Vec<Block> = Vec::with_capacity(total_block_bitmap_blocks);
        let mut start = ((1 + super_block_ref.get_inode_bitmap_block_count()) * super_block_ref.get_block_size()) as u64;

        for i in 0..total_block_bitmap_blocks {
            let block = Block::default();
            let mut buffer = vec![0_u8; super_block_ref.get_block_size()];
            let tmp_res = file.read_exact_at(buffer.as_mut_slice(), start);
            if tmp_res.is_err() {
                return Err(tmp_res.err().unwrap());
            }
            blocks.push(Block {
                block_number: (INODE_BITMAP_STARTING_BLOCK_NUMBER + super_block_ref.get_inode_bitmap_block_count() + i) as u16,
                data: buffer,
            });
            start += super_block_ref.get_block_size() as u64;
        }
        
        Ok(Self::deserialize(blocks, super_block_ref))
    }

    fn deserialize(blocks: Vec<Block>, super_block_ref: &SuperBlock) -> Self {
        let total_blocks = super_block_ref.get_total_blocks();
        let mut bitmap = bitvec![u8, Lsb0; 0; total_blocks];
        bitmap.fill(false);

        let mut current_index = 0;
        for block in blocks[..blocks.len() - 1].iter() {
            let data = &block.data;
            let len = data.len();
            bitmap.as_raw_mut_slice()[current_index..current_index + (len >> 3)]
                .copy_from_slice(&data);
            current_index += len;
        }
        {
            // Handle the last block separately to avoid overrun
            let last_block = &blocks[blocks.len() - 1];
            let data = &last_block.data;
            let remaining_bits = total_blocks - current_index * 8;
            let bytes_to_copy = (remaining_bits + 7) / 8; // Round up to the nearest byte
            bitmap.as_raw_mut_slice()[current_index..current_index + bytes_to_copy]
                .copy_from_slice(&data[..bytes_to_copy]);
        }
        Self { bitmap }
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
                block_number: (1 + super_block_ref.get_inode_bitmap_block_count() + i) as u16,
                data: data.to_vec(),
            });
        }        

        blocks
    }

    pub fn serialize_to_vec(&self) -> Vec<u8> {
        self.bitmap.as_raw_slice().to_vec()
    }

    pub fn set(&mut self, block_number: usize) {
        self.bitmap.set(block_number, true);
    }
}
