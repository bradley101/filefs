
use std::fs::{ File, OpenOptions };
use crate::core::block_bitmap::BlockBitmap;
use crate::core::super_block::SuperBlock;
use crate::core::inode::{ Inode, FileType };
use crate::core::inode_bitmap::InodeBitmap;
use crate::entity::directory::Directory;
use crate::util::Path;
use crate::medium::types::byte_compatible;

pub struct ffs<T: byte_compatible> {
    super_block: SuperBlock,
    medium: Option<T>,
    cwd: Directory,
    inode_bitmap: InodeBitmap,
    block_bitmap: BlockBitmap,
}

impl <T: byte_compatible> Default for ffs<T> {
    fn default() -> Self {
        ffs {
            super_block: SuperBlock::default(),
            medium: None,
            cwd: Directory::default(),
            inode_bitmap: InodeBitmap::default(),
            block_bitmap: BlockBitmap::default(),
        }
    }
}

impl <T: byte_compatible> ffs<T> {
    pub fn load(medium: T) -> Option<Self> {
        let mut inst = ffs::default();

        match inst.init(false, medium, 0, 0, 0) {
            Ok(f) => {
                Some(inst)
            },
            Err(_) => None
        }
    }

    pub fn new(medium: T, size: u32, block_size: u32, bytes_per_inode: u32) -> Option<Self> {
        let mut inst = ffs::default();

        match inst.init(true, medium, size, block_size, bytes_per_inode) {
            Ok(f) => {
                Some(inst)
            },
            Err(_) => None
        }
    }

    fn init
        (&mut self, new: bool, medium: T, size: u32, block_size: u32, bytes_per_inode: u32)
        -> Result<(), std::io::Error>
    {
        if new {
            return self.create_new(medium, size, block_size, bytes_per_inode)
        } else {
            return self.load_existing(medium);
        }
    }

    fn load_existing(&mut self, medium: T) -> Result<(), std::io::Error>
    {
        self.medium = Some(medium);

        if let Err(err) = self.fetch_super_block() {
            return Err(err);
        }

        if let Err(err) = self.load_root_directory() {
            return Err(err);
        }

        Ok(())
    }

    fn create_new
        (&mut self, medium: T, size: u32, block_size: u32, bytes_per_inode: u32)
        -> Result<(), std::io::Error>
    {
        self.medium = Some(medium);

        if let Err(err) = self.create_new_super_block(size, block_size, bytes_per_inode) {
            return Err(err);
        }

        if let Err(err) = self.create_root_directory() {
            return Err(err);
        }

        Ok(())
    }

    fn create_root_directory(&mut self) -> Result<(), std::io::Error> {
         // Create the root directory
         let tmp_res = Directory::create_new(
            FileType::Directory,
            "/",
            None,
            &self.super_block,
            &mut self.inode_bitmap,
            self.medium.as_mut().unwrap());
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        self.cwd = tmp_res.unwrap();
        Ok(())
    }

    fn load_root_directory(&mut self) -> Result<(), std::io::Error> {
        let tmp_res = Directory::load(0, &self.super_block, self.medium.as_mut().unwrap());
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        self.cwd = tmp_res.unwrap();
        Ok(())
    }

    fn create_new_super_block
        (&mut self, fs_size: u32, block_size: u32, bytes_per_inode: u32)
        -> Result<(), std::io::Error>
    {
        self.super_block = SuperBlock::create_new(fs_size, block_size, bytes_per_inode);
        let tmp_res = self.persist_super_block();
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        // Create the Inode Bitmap
        self.inode_bitmap = InodeBitmap::new(self.super_block.get_total_inodes());
        let tmp_res = self.persist_inode_bitmap();
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }
        
        self.block_bitmap = BlockBitmap::new(self.super_block.get_total_blocks());
        let tmp_res = self.persist_block_bitmap();
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        // set the bitmap in the bitmap blocks for the above structures
        self.block_bitmap.set(1);
        (0..self.super_block.get_inode_bitmap_block_count())
            .for_each(|b|
                self.block_bitmap.set(b + 1));
        (0..self.super_block.get_block_bitmap_block_count())
            .for_each(|b|
                self.block_bitmap.set(1 + self.super_block.get_inode_bitmap_block_count() + b));

        let tmp_res = self.persist_block_bitmap();
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        Ok(())
    }

    fn fetch_super_block(&mut self) -> Result<(), std::io::Error> {
        let f = self.medium.as_mut().unwrap();

        let tmp_res = SuperBlock::deserialize(f);
        if tmp_res.is_err(){
            return Err(tmp_res.err().unwrap());
        }
        self.super_block = tmp_res.unwrap();

        // Fetch the inode bitmap
        let tmp_res = InodeBitmap::fetch(f, &self.super_block);
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }
        self.inode_bitmap = tmp_res.unwrap();

        // Fetch the block bitmap
        let tmp_res = BlockBitmap::fetch(f, &self.super_block);
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }
        self.block_bitmap = tmp_res.unwrap();
        Ok(())
    }

    fn persist_super_block(&mut self) -> Result<(), std::io::Error> {
        let f = self.medium.as_mut().unwrap();
        self.super_block.persist(f)
    }

    fn persist_inode_bitmap(&mut self) -> Result<(), std::io::Error> {
        let f = self.medium.as_mut().unwrap();
        self.inode_bitmap.persist(f, &self.super_block)
    }

    fn persist_block_bitmap(&mut self) -> Result<(), std::io::Error> {
        let f = self.medium.as_mut().unwrap();
        self.block_bitmap.persist(f, &self.super_block)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_fs() {
        const TEST_FS_SIZE: u32 = 10 * (1 << 20); // 10 MB
        const BLOCK_SIZE: u32 = 4 * (1 << 10); // 4 KB
        const BYTES_PER_INODE: u32 = 1 << 12; // 4096 bytes per inode
        let file_name = "test_fs.dat".to_string();
        let medium = crate::medium::file::file_medium::new(file_name);

        let fs = ffs::new(
            medium,
            TEST_FS_SIZE,
            BLOCK_SIZE,
            BYTES_PER_INODE,
        );
        assert!(fs.is_some());
    }

    #[test]
    fn test_existing_fs() {
        let file_name = "test_fs.dat".to_string();
        let medium = crate::medium::file::file_medium::new(file_name);
        let fs = ffs::load(medium);
        assert!(fs.is_some());
    }
}
