use std::{cell::RefCell, io::Error, rc::Rc};

use crate::{core::{block_bitmap::BlockBitmap, inode::Inode, inode_bitmap::InodeBitmap, super_block::SuperBlock}, medium::types::byte_compatible};

pub struct fs_metadata<T: byte_compatible> {
    super_block: SuperBlock,
    inode_bitmap: InodeBitmap,
    block_bitmap: BlockBitmap,
    medium: Rc<RefCell<T>>
}

impl <T: byte_compatible> fs_metadata<T> {
    
    pub fn create_new(medium: Rc<RefCell<T>>, fs_size: u32, block_size: u32, bytes_per_inode: u32) -> Result<Self, Error>
    {   
        let super_block = SuperBlock::create_new(fs_size, block_size, bytes_per_inode);
        super_block.persist(medium.borrow_mut())?;
        
        let inode_bitmap = InodeBitmap::new(super_block.get_total_inodes());
        inode_bitmap.persist(medium.borrow_mut(), &super_block)?;
        
        let mut block_bitmap = BlockBitmap::new(super_block.get_total_blocks());

        // set the bitmap in the bitmap blocks for the above structures
        block_bitmap.set(1);
        (0..super_block.get_inode_bitmap_block_count())
            .for_each(|b|
                block_bitmap.set(b + 1));
        (0..super_block.get_block_bitmap_block_count())
            .for_each(|b|
                block_bitmap.set(1 + super_block.get_inode_bitmap_block_count() + b));

        block_bitmap.persist(medium.borrow_mut(), &super_block)?;
        
        Ok(Self {
            super_block,
            inode_bitmap,
            block_bitmap,
            medium
        })
    }

    pub fn fetch(medium: Rc<RefCell<T>>) -> Result<Self, Error>
    {
        let super_block = SuperBlock::deserialize(medium.borrow_mut())?;
        let inode_bitmap = InodeBitmap::fetch(medium.borrow_mut(), &super_block)?;
        let block_bitmap = BlockBitmap::fetch(medium.borrow_mut(), &super_block)?;
        
        Ok(Self {
            super_block,
            inode_bitmap,
            block_bitmap,
            medium
        })
    }

    fn persist_super_block(&mut self) -> Result<(), std::io::Error> {
        self.super_block.persist(self.medium.borrow_mut())
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
        self.inode_bitmap.persist(self.medium.borrow_mut(), &self.super_block)
    }

    pub fn persist_inode(&mut self, inode: &Inode) -> Result<(), std::io::Error> {
        inode.persist(self.medium.borrow_mut(), &self.super_block)
    }

    pub fn set_inode_in_bitmap(&mut self, inode: u16) {
        self.inode_bitmap.set(inode as usize);
    }

    fn persist_block_bitmap(&mut self) -> Result<(), std::io::Error> {
        self.block_bitmap.persist(self.medium.borrow_mut(), &self.super_block)
    }

    pub fn is_inode_bitmap_full(&self) -> bool {
        self.inode_bitmap.is_full()
    }

    pub fn inode_find_first_free(&self) -> Option<usize> {
        self.inode_bitmap.find_first_free()
    }
}

