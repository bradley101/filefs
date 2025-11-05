use std::io::Error;

use crate::{core::{block_bitmap::BlockBitmap, inode::Inode, inode_bitmap::InodeBitmap, super_block::SuperBlock}, medium::types::byte_compatible};

pub struct fs_metadata<'a, T: byte_compatible> {
    super_block: SuperBlock,
    inode_bitmap: InodeBitmap,
    block_bitmap: BlockBitmap,
    medium: Option<&'a T>
}

impl <'a, T: byte_compatible> Default for fs_metadata<'a, T> {
    fn default() -> Self {
        Self {
            super_block: SuperBlock::default(),
            inode_bitmap: InodeBitmap::default(),
            block_bitmap: BlockBitmap::default(),
            medium: None
        }
    }
}

impl <'a, T: byte_compatible> fs_metadata<'a, T> {
    
    pub fn create_new(medium: &'a T, fs_size: u32, block_size: u32, bytes_per_inode: u32) -> Result<Self, Error>
    {
        let mut md = fs_metadata::<'a, T>::default();
        md.medium = Some(medium);
        
        md.super_block = SuperBlock::create_new(fs_size, block_size, bytes_per_inode);
        let tmp_res = md.super_block.persist(md.medium.as_mut().unwrap());
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        // Create the Inode Bitmap
        let inode_bitmap = InodeBitmap::new(md.super_block.get_total_inodes());
        let tmp_res = inode_bitmap.persist(md.medium.as_mut().unwrap(), &md.super_block);
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }
        
        md.block_bitmap = BlockBitmap::new(md.super_block.get_total_blocks());
        let tmp_res = md.block_bitmap.persist(md.medium.as_mut().unwrap(), &md.super_block);
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        // set the bitmap in the bitmap blocks for the above structures
        md.block_bitmap.set(1);
        (0..md.super_block.get_inode_bitmap_block_count())
            .for_each(|b|
                md.block_bitmap.set(b + 1));
        (0..md.super_block.get_block_bitmap_block_count())
            .for_each(|b|
                md.block_bitmap.set(1 + md.super_block.get_inode_bitmap_block_count() + b));

        let tmp_res = md.block_bitmap.persist(md.medium.as_mut().unwrap(), &md.super_block);
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        Ok(md)
    }

    pub fn fetch(medium: &'a T) -> Result<Self, Error>
    {
        let mut md = fs_metadata::<'a, T>::default();
        md.medium = Some(medium);
        let md_mut_ref = md.medium.as_mut().unwrap();

        md.super_block = SuperBlock::deserialize(md_mut_ref)?;
        md.inode_bitmap = InodeBitmap::fetch(md_mut_ref, &md.super_block)?;
        md.block_bitmap = BlockBitmap::fetch(md_mut_ref, &md.super_block)?;

        Ok(md)
    }

    fn persist_super_block(&mut self) -> Result<(), std::io::Error> {
        self.super_block.persist(self.medium.as_mut().unwrap())
    }

    pub fn super_block_get_total_blocks(&self) -> usize {
        self.super_block.get_total_blocks()
    }

    pub fn super_block_get_inode_start_block(&self) -> usize {
        self.super_block.get_inode_start_block()
    }

    pub fn super_block_get_block_size(&self) -> usize {
        self.super_block.get_block_size()
    }

    pub fn persist_inode_bitmap(&mut self) -> Result<(), std::io::Error> {
        self.inode_bitmap.persist(self.medium.as_mut().unwrap(), &self.super_block)
    }

    pub fn persist_inode(&mut self, inode: &Inode) -> Result<(), std::io::Error> {
        inode.persist(self.medium.as_mut().unwrap(), &self.super_block)
    }

    pub fn set_inode_in_bitmap(&mut self, inode: u16) {
        self.inode_bitmap.set(inode as usize);
    }

    fn persist_block_bitmap(&mut self) -> Result<(), std::io::Error> {
        self.block_bitmap.persist(self.medium.as_mut().unwrap(), &self.super_block)
    }

    pub fn is_inode_bitmap_full(&self) -> bool {
        self.inode_bitmap.is_full()
    }

    pub fn inode_find_first_free(&self) -> Option<usize> {
        self.inode_bitmap.find_first_free()
    }
}

