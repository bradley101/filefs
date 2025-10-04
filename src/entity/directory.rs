/*
    this file represents the structure of a directory entry in the filesystem
    it contains, the corresponding Inode, and helper functions
*/

use crate::core::inode::Inode;

struct Directory {
    inode: Inode,
}