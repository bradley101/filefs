use crate::{core::block::Block, fs_metadata::fs_metadata, medium::types::byte_compatible, util::InodeNum};
use std::mem::size_of;


const ENTRY_IN_USE: u8 = 0x01;

#[derive(Default)]
struct DirectoryBlockEntry {
    inode: InodeNum,
    flags: u8,
} 

pub struct DirectoryBlockView {
    block: Block,
}

impl DirectoryBlockEntry {
    pub const SIZE: usize = size_of::<InodeNum>() + size_of::<u8>();

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = vec![0_u8; Self::SIZE];
        buffer[0..size_of::<InodeNum>()].copy_from_slice(self.inode.to_le_bytes().as_slice());
        buffer[size_of::<InodeNum>()..Self::SIZE].copy_from_slice(self.inode.to_le_bytes().as_slice());
        buffer
    }
}

impl DirectoryBlockView {
    pub fn new(block: Block) -> Self {
        Self { block }
    }

    fn format<M: byte_compatible>(&mut self, metadata: &mut fs_metadata<M>) -> Result<(), std::io::Error> {
        self.block.data.clear();
        self.block.persist(metadata)
    }

    // pub fn serialize(&self) -> Vec<u8> {
    //
    // }
}
