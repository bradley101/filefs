
use std::{io::{Seek, SeekFrom, Write}, os::unix::fs::FileExt};

use bitvec::prelude::*;

use super::{block::Block, block_data_types::BlockDataType, super_block::SuperBlock};
use crate::util::INODE_BITMAP_STARTING_BLOCK_NUMBER;

#[derive(Debug, Clone, Default)]
pub struct BlockBitmap {
    bitmap: BitVec<u8>
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
                block_type: BlockDataType::BlockBitmap,
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
        let total_bitmap_blocks = super_block_ref.get_block_bitmap_block_count() as usize;
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
                block_type: BlockDataType::BlockBitmap,
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
