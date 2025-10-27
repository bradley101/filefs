/*
    this file represents the structure of a directory entry in the filesystem
    it contains, the corresponding Inode, and helper functions
*/

use crate::core::inode::{FileType, Inode};
use crate::core::inode_bitmap::InodeBitmap;
use crate::core::super_block::SuperBlock;
use crate::util::Path;

#[derive(Debug, Clone, Default)]
pub struct Directory {
    inode: Inode,
}

impl Directory {
    pub fn new(inode: Inode) -> Self {
        Self { inode  }
    }

    pub fn get_inode_number(&self) -> u16 {
        self.inode.inode_number
    }

    pub fn create_new<T: Path>(
        ftype: FileType,
        name: T,
        parent: &Directory,
        super_block_ref: &SuperBlock,
        inode_bitmap_ref: &mut InodeBitmap
    ) -> Result<Self, std::io::Error> {
        let inode = Inode::create_new(
            parent.get_inode_number(),
            name,
            ftype,
            super_block_ref,
            inode_bitmap_ref);
        if inode.is_err() {
            return Err(inode.err().unwrap());
        }

        Ok(Self { inode : inode.unwrap() })
    }
}