

use crate::util::BlockNum;

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
}
