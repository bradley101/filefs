

use super::block_data_types::BlockDataType;

pub trait BlockLike {
    pub fn get_block_type(&self) -> BlockDataType;
    pub fn get_block_number(&self) -> u16;
}

#[derive(Default)]
pub struct Block {
    pub block_number: u16,
    pub data: Vec<u8>,
    pub block_type: BlockDataType,
}
