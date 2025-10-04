

use super::block_data_types::BlockDataType;

#[derive(Default)]
pub struct Block {
    pub block_number: u16,
    pub data: Vec<u8>,
    pub block_type: BlockDataType,
}
