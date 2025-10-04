

use std::{cmp::max, fs::File, io::{Seek, SeekFrom, Write}, os::unix::fs::FileExt};
use bitvec::prelude::*;

use super::{block_data_types::BlockDataType, inode::{INODE_BITMAP_STARTING_BLOCK_NUMBER, INODE_SIZE}};

pub const SUPER_BLOCK_FILE_OFFSET: u64 = 0;
pub const SUPER_BLOCK_SIZE: usize = 1 << 8;


#[derive(Default)]
pub struct Block {
    pub block_number: u16,
    pub data: Vec<u8>,
    pub block_type: BlockDataType,
}
