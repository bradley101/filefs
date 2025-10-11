/*
    this file represents the structure of a directory entry in the filesystem
    it contains, the corresponding Inode, and helper functions
*/

use crate::core::inode::Inode;

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
}