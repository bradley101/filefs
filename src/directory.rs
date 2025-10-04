/*
    this file represents the structure of a directory entry in the filesystem
    it contains, the corresponding Inode, and helper functions
*/

struct Directory {
    inode: inode::Inode,
}