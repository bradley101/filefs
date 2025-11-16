
use std::cell::RefMut;

use bitvec::prelude::*;

use super::{block::Block, block_data_types::BlockDataType, super_block::SuperBlock};
use crate::{medium::types::byte_compatible, util::INODE_BITMAP_STARTING_BLOCK_NUMBER};

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

    pub fn persist<T: byte_compatible>(&self, medium: RefMut<'_, T>, super_block_ref: &SuperBlock) -> std::io::Result<()> {
        let blocks = self.serialize(super_block_ref);

        for block in blocks {
            let block_offset = block.block_number as u64 * super_block_ref.get_block_size() as u64;
            let tmp_res = medium.write_all(block_offset, block.data.len(), block.data.as_slice());
            if tmp_res.is_err() {
                return Err(tmp_res.err().unwrap());
            }
        }

        Ok(())
    }

    /* 
        First get the number of inode bitmap blocks from the super block,
        then fetch that many blocks from the inode bitmap starting block offset,
        then pass the vec to the deserialize function to generate a InodeBitmap
    */
    pub fn fetch<T: byte_compatible>(medium: RefMut<'_, T>, super_block_ref: &SuperBlock) -> std::io::Result<Self> {
        let total_inode_bitmap_blocks = super_block_ref.get_inode_bitmap_block_count();
        let mut blocks: Vec<Block> = Vec::with_capacity(total_inode_bitmap_blocks);
        let mut start = INODE_BITMAP_STARTING_BLOCK_NUMBER as u64 * super_block_ref.get_block_size() as u64;

        for i in 0..total_inode_bitmap_blocks {
            let block = Block::default();
            let mut buffer = vec![0_u8; super_block_ref.get_block_size()];
            let tmp_res = medium.read_all(start, buffer.len(), buffer.as_mut_slice());
            if tmp_res.is_err() {
                return Err(tmp_res.err().unwrap());
            }
            blocks.push(Block {
                block_number: (i + INODE_BITMAP_STARTING_BLOCK_NUMBER) as u16,
                data: buffer,
                block_type: BlockDataType::InodeBitmap,
            });
            start += super_block_ref.get_block_size() as u64;
        }
        
        Ok(Self::deserialize(blocks, super_block_ref))
    }

    fn deserialize(blocks: Vec<Block>, super_block_ref: &SuperBlock) -> Self {
        let total_inodes = super_block_ref.get_total_inodes();
        let mut bitmap = bitvec![u8, Lsb0; 0; total_inodes];
        bitmap.fill(false);

        let mut current_index = 0;
        for block in blocks[..blocks.len() - 1].iter() {
            let data = &block.data;
            bitmap.as_raw_mut_slice()[current_index..current_index + (data.len() >> 3)]
                .copy_from_slice(&data);
            current_index += data.len();
        }
        {
            // Handle the last block separately to avoid overrun
            let last_block = &blocks[blocks.len() - 1];
            let data = &last_block.data;
            let remaining_bits = total_inodes - current_index * 8;
            let bytes_to_copy = (remaining_bits + 7) / 8; // Round up to the nearest byte
            bitmap.as_raw_mut_slice()[current_index..current_index + bytes_to_copy]
                .copy_from_slice(&data[..bytes_to_copy]);
        }
        Self { bitmap }
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
                block_type: BlockDataType::InodeBitmap,
            });
        }

        blocks
    }

    // pub fn deserialize(file: &mut File, block_size: usize) 

    pub fn set(&mut self, inode_num: usize) {
        assert!(inode_num < self.bitmap.len());
        self.bitmap.set(inode_num, true);
    }

    pub fn get(&self, inode_num: usize) -> bool {
        assert!(inode_num < self.bitmap.len());
        *self.bitmap.get(inode_num).unwrap()
    }

    pub fn is_full(&self) -> bool {
        self.bitmap.all()
    }

    pub fn find_first_free(&self) -> Option<usize> {
        self.bitmap.first_zero()
    }
}
