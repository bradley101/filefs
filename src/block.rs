

const BLOCK_SIZE: usize = 4 * 1024;
const BLOCK_DATA_SIZE: usize = BLOCK_SIZE - 8;

/*
    We will use a linked list kind of approach while
    implementing the data blocks.
    
    This way we can eliminate the fragmentation problem.
    
    However this method has its own cons, but its okay 
    for a temp file system.
*/

pub struct Block {
    pub block_number: u32,
    pub data: [u8; BLOCK_DATA_SIZE],
    pub next_block_number: u32
}

