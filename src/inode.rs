
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

pub const INODE_BITMAP_STARTING_BLOCK_NUMBER: usize = 2;

use std::{io::{Seek, SeekFrom, Write}, os::unix::fs::FileExt};

use bitvec::prelude::*;
use crate::block::{Block, SuperBlock};

use super::block::BlockBitmap;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum FileType {
    #[default]
    File = 0_u8,
    Directory = 1
}

#[derive(Clone, Default)]
pub struct Inode {
    pub inode_number: u16,
    pub parent: u16,
    pub name: String,
    pub data_blocks: [u16; 32],
    pub block_bitmap: BlockBitmap,
    pub file_type: FileType,
    pub file_size: u32,
}

impl Inode {
    pub fn persist(&self, file: &mut std::fs::File, super_block_ref: &SuperBlock) -> std::io::Result<()> {
        let buffer = self.serialize();
        file.write_all_at(buffer.as_slice(),
            super_block_ref.get_inode_start_block() as u64 + (INODE_SIZE as u64 * self.inode_number as u64))
    }

    fn serialize(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::with_capacity(INODE_SIZE);

        buffer.extend_from_slice(&self.inode_number.to_le_bytes());
        buffer.extend_from_slice(&self.parent.to_le_bytes());

        let name_bytes = self.name.as_bytes();
        let mut name_buffer = vec![0_u8; MAX_FILE_NAME_SIZE];
        name_buffer[..name_bytes.len().min(MAX_FILE_NAME_SIZE)].copy_from_slice(&name_bytes[..name_bytes.len().min(MAX_FILE_NAME_SIZE)]);
        buffer.extend_from_slice(&name_buffer);

        for &block in &self.data_blocks {
            buffer.extend_from_slice(&block.to_le_bytes());
        }

        buffer.extend_from_slice(self.block_bitmap.serialize_to_vec().as_slice());

        buffer.push(self.file_type as u8);
        buffer.extend_from_slice(&self.file_size.to_le_bytes());


        buffer.resize(INODE_SIZE, 0); // Ensure the buffer is exactly INODE_SIZE
        buffer
    }
}

#[derive(Clone, Default)]
pub struct InodeBitmap {
    bitmap: BitVec<u8>
}

impl InodeBitmap {
    pub fn new(num_inodes: usize) -> Self {
        let mut bitmap = bitvec![u8, Lsb0; 0; num_inodes];
        bitmap.fill(false);
        Self {
            bitmap
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

    fn serialize(&self, super_block_ref: &SuperBlock) -> Vec<Block> {
        let mut blocks: Vec<Block> = Vec::new();
        let total_inode_bitmap_blocks = super_block_ref.get_inode_bitmap_block_count();
        let bitmap_vec = self.bitmap.as_raw_slice();

        for i in 0..total_inode_bitmap_blocks {
            let start = i * super_block_ref.get_block_size();
            let end = start + super_block_ref.get_block_size();
            let data = if end > bitmap_vec.len() {
                &bitmap_vec[start..]
            } else {
                &bitmap_vec[start..end]
            };
            blocks.push(Block {
                block_number: (i + INODE_BITMAP_STARTING_BLOCK_NUMBER) as u16,
                data: data.to_vec(),
            });
        }
        

        blocks

    }

    // pub fn deserialize(file: &mut File, block_size: usize) 

    pub fn allocate_inode(&mut self, inode_num: u16) {
        self.bitmap.set(inode_num as usize, true);
    }
}


