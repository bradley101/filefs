/*
    this file represents the structure of a directory entry in the filesystem
    it contains, the corresponding Inode, and helper functions
*/

use std::cell::RefMut;

use crate::core::inode::{FileType, Inode};
use crate::fs_metadata::fs_metadata;
use crate::medium::types::byte_compatible;
use crate::util::Path;

#[derive(Default)]
pub struct Directory {
    inode: Inode,
}

impl Directory {
    pub fn get_inode_number(&self) -> u16 {
        self.inode.inode_number
    }

    pub fn create_new<T: Path, M: byte_compatible>(
        ftype: FileType,
        name: T,
        parent: Option<&Directory>,
        metadata: &mut fs_metadata<M>
    ) -> Result<Self, std::io::Error> {
        Ok(Self { 
            inode: Inode::create_new(
                if parent.is_none() { 0 } else { parent.unwrap().get_inode_number() },
                name,
                ftype,
                metadata)?
        })
    }

    pub fn load<M: byte_compatible>(
        inode_num: u16,
        metadata: &fs_metadata<M>,
        medium: RefMut<'_, M>) -> Result<Self, std::io::Error>
    {
        Ok(Self {
            inode: Inode::load(
                medium,
                0,
                metadata)?
        })
    }

}