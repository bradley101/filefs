/*
    this file represents the structure of a directory entry in the filesystem
    it contains, the corresponding Inode, and helper functions
*/

use std::cell::RefMut;

use crate::core::inode::{FileType, Inode};
use crate::fs_metadata::fs_metadata;
use crate::medium::types::byte_compatible;
use crate::util::Path;

use super::file::File;

pub struct Directory {
    inode: Inode,
}

impl Directory {
    pub fn new<T: Path, M: byte_compatible>(
        name: T,
        parent: Option<&Directory>,
        metadata: &mut fs_metadata<M>
    ) -> Result<Self, std::io::Error> {
        let inode = Inode::create_new(
            if parent.is_none() { 0 } else { parent.unwrap().get_inode_number() },
            name,
            FileType::Directory,
            metadata)?;
        
        if parent.is_some() {
            // let parent = parent.unwrap();
            // let mut block_bitmap = &parent.inode.block_bitmap;
        }

        Ok(Self { inode })
    }

    pub fn load<M: byte_compatible>(
        inode_num: u16,
        metadata: &fs_metadata<M>,
        medium: RefMut<'_, M>) -> Result<Self, std::io::Error>
    {
        Ok(Self {
            inode: Inode::load(
                medium,
                inode_num,
                metadata)?
        })
    }

    pub fn get_inode_number(&self) -> u16 {
        self.inode.inode_number
    }

    pub fn create_new_directory<T: Path, M: byte_compatible>(
        &mut self,
        name: T,
        metadata: &mut fs_metadata<M>
    ) -> Result<Self, std::io::Error> {
        Directory::new(name, Some(&self), metadata)
    }

    pub fn create_new_file<T: Path, M: byte_compatible>(
        &mut self,
        name: T,
        metadata: &mut fs_metadata<M>
    ) -> Result<File, std::io::Error> {
        File::new(name, &self, metadata)
    }

    pub fn get_first_empty_block(&self) -> Option<usize> {
        self.inode.block_bitmap.get_first_free_block()
    }

    // pub fn get_available_bit_in_non_full_block<M: byte_compatible>(
    //     &self,
    //     metadata: &mut fs_metadata<M>) -> Option<(usize, usize)> {
    //
    //     let inode_block_bitmap = &self.inode.block_bitmap;
    //     for inode_block_bitmap.stringify!
    //     None
    // }



}
