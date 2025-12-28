use crate::{core::block::Block, util::InodeNum};

struct DirectoryBlockEntry {
    inode: InodeNum,
} 

pub struct DirectoryBlockView {
    block: Block,
}

impl DirectoryBlockView {
    pub fn new(block: Block) -> Self {
        Self { block }
    }
}
