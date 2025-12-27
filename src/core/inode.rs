
use std::cell::RefMut;

use crate::fs_metadata::fs_metadata;
use crate::medium::types::byte_compatible;
use crate::util::{BlockNum, FileSize, INODE_SIZE, InodeNum, MAX_CHILDREN_COUNT, Path};

use super::block_bitmap::BlockBitmap;
use super::super_block::SuperBlock;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileType {
    #[default]
    File = 0_u8,
    Directory = 1
}

#[derive(Debug, Clone)]
pub struct Inode {
    pub inode_number: InodeNum,
    pub parent: InodeNum,
    pub name: String,
    pub data_blocks: [BlockNum; MAX_CHILDREN_COUNT],
    pub block_bitmap: BlockBitmap,
    pub file_type: FileType,
    pub file_size: FileSize,
}

impl Inode {
    pub fn create_new<T: Path, M: byte_compatible>
        (parent: InodeNum,
         name: T,
         file_type: FileType,
         metadata: &mut fs_metadata<M>) -> Result<Self, std::io::Error>
    {
        let name = name.to_String();
        if name.len() > crate::util::MAX_FILE_NAME_SIZE {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "File name too long"));
        }

        if metadata.is_inode_bitmap_full() {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "No free inodes available"));
        }

        let inode_number = metadata.inode_find_first_free().expect("No free inodes available") as u16;
        let new_inode = Self {
            inode_number,
            parent,
            name,
            data_blocks: [0; MAX_CHILDREN_COUNT],
            block_bitmap: BlockBitmap::new(MAX_CHILDREN_COUNT as usize),
            file_type,
            file_size: 0,
        };
        metadata.set_inode_in_bitmap(inode_number);
        metadata.persist_inode_bitmap()?;
        metadata.persist_inode(&new_inode)?;

        Ok(new_inode)
    }
    pub fn persist<T: byte_compatible>(&self, medium: RefMut<'_, T>, super_block_ref: &SuperBlock) -> std::io::Result<()> {
        let buffer = self.serialize();
        let inode_offset = 
            super_block_ref.get_inode_start_block() as u64 * super_block_ref.get_block_size() as u64
            + (INODE_SIZE as u64 * self.inode_number as u64);
        medium.write_all(inode_offset, buffer.len(), buffer.as_slice())
    }

    fn serialize(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::with_capacity(INODE_SIZE);

        buffer.extend_from_slice(&self.inode_number.to_le_bytes());
        buffer.extend_from_slice(&self.parent.to_le_bytes());

        let name_bytes = self.name.as_bytes();
        let mut name_buffer = vec![0_u8; crate::util::MAX_FILE_NAME_SIZE];
        name_buffer[..name_bytes.len().min(crate::util::MAX_FILE_NAME_SIZE)].copy_from_slice(&name_bytes[..name_bytes.len().min(crate::util::MAX_FILE_NAME_SIZE)]);
        buffer.extend_from_slice(&name_buffer);

        for &block in &self.data_blocks {
            buffer.extend_from_slice(&block.to_le_bytes());
        }

        buffer.extend_from_slice(self.block_bitmap.serialize_to_vec().as_slice());

        buffer.push(self.file_type as u8);
        buffer.extend_from_slice(&self.file_size.to_le_bytes());


        buffer.resize(INODE_SIZE, 0); // Ensure the buffer is exactly INODE_SIZE
        buffer
    }

    pub fn get_free_location_in_non_zero_block<M: byte_compatible>(
        &self,
        metadata: &fs_metadata<M>) -> Option<(usize, usize)>
    {
        
    }
}


