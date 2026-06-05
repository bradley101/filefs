

use std::io::Error;

use crate::{fs_metadata::fs_metadata, medium::types::byte_compatible, util::BlockNum};

use super::block_data_types::BlockType;

#[derive(Default)]
pub struct Block {
    pub block_number: BlockNum,
    pub data: Vec<u8>,
    pub block_type: BlockType,
}

impl Block {
    #[inline(always)]
    pub fn get_block_number(&self) -> BlockNum {
        self.block_number
    }

    #[inline]
    pub fn get_block_type(&self) -> BlockType {
        self.block_type
    }

    pub fn persist<M: byte_compatible>(&self, metadata: &mut fs_metadata<M>) -> Result<(), Error> {
        let offset = self.block_number as u64 * metadata.super_block_get_block_size() as u64;
        metadata.medium.borrow_mut().write_all(offset, metadata.super_block_get_block_size(), self.data.as_slice())
    }
}
