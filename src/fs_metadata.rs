use crate::{core::{block_bitmap::BlockBitmap, inode_bitmap::InodeBitmap, super_block::SuperBlock}};

#[derive(Default)]
struct fs_metadata {
    super_block: SuperBlock,
    inode_bitmap: InodeBitmap,
    block_bitmap: BlockBitmap,
}

impl <'a> fs_metadata {

}

