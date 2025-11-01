/*
    this file represents the structure of a directory entry in the filesystem
    it contains, the corresponding Inode, and helper functions
*/

use crate::core::inode::{FileType, Inode};
use crate::core::inode_bitmap::InodeBitmap;
use crate::core::super_block::SuperBlock;
use crate::medium::types::byte_compatible;
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

    pub fn create_new<T: Path, M: byte_compatible>(
        ftype: FileType,
        name: T,
        parent: Option<&Directory>,
        super_block_ref: &SuperBlock,
        inode_bitmap_ref: &mut InodeBitmap,
        medium: &mut M
    ) -> Result<Self, std::io::Error> {
        let inode = Inode::create_new(
            if parent.is_none() { 0 } else { parent.unwrap().get_inode_number() },
            name,
            ftype,
            super_block_ref,
            inode_bitmap_ref);
        if inode.is_err() {
            return Err(inode.err().unwrap());
        }

        let inode = inode.unwrap();
        inode_bitmap_ref.set(inode.inode_number as usize);

        let tmp_res = inode.persist(medium, super_block_ref);
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        let tmp_res = inode_bitmap_ref.persist(medium, super_block_ref);
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        Ok(Self { inode  })
    }

    pub fn load<M: byte_compatible>(
        inode_num: u16,
        super_block_ref: &SuperBlock,
        medium: &mut M) -> Result<Self, std::io::Error>
    {
        let tmp_res = Inode::load(
            medium,
            0,
            super_block_ref);
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        Ok(Directory::new(tmp_res.unwrap()))
    }

}