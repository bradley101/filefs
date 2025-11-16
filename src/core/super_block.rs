use std::{cell::RefMut, cmp::max};

use super::{block::Block, block_data_types::BlockDataType};

use crate::{medium::types::byte_compatible, util::{
    INODE_SIZE, SUPER_BLOCK_FILE_OFFSET, SUPER_BLOCK_SIZE
}};

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


impl SuperBlock {
    pub fn create_new(fs_size: u32, block_size: u32, bytes_per_inode: u32) -> Self {
        let ti = (fs_size / bytes_per_inode) as u16 ;
        let tb = (fs_size / block_size) as u16;
        let inode_block_count = (ti as usize * INODE_SIZE) / block_size as usize;
        let inode_bitmap_block_count = max(1, ti / 8 / block_size as u16);
        let block_bitmap_block_count = max(1, tb / 8 / block_size as u16);

        Self {
            version: crate::util::get_latest_version(),
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

    pub fn persist<T: byte_compatible>(&self, medium: RefMut<'_, T>) -> std::io::Result<()> {
        let buffer = self.serialize();
        medium.write_all(SUPER_BLOCK_FILE_OFFSET, buffer.data.len(), buffer.data.as_slice())
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
            block_type: BlockDataType::SuperBlock,
        }
    }

    pub fn deserialize<T: byte_compatible>(file: RefMut<'_, T>) -> Result<SuperBlock, std::io::Error> {
        let mut block = Block::default();
        block.data.resize(SUPER_BLOCK_SIZE, 0);

        let tmp_res =
            file.read_all(SUPER_BLOCK_FILE_OFFSET, block.data.len(), block.data.as_mut_slice());
        
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
