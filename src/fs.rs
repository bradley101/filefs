



const FS_DESCRIPTOR_SIZE: usize = 128;
const FS_USED_SIZE: usize = 2
                            + BYTES_REQUIRED_TO_REPRESENT_MAX_INODE_COUNT
                            + 3;

const INODE_STARTING_POS: usize = FS_DESCRIPTOR_SIZE;

const MAX_SUPPORTED_INODE_COUNT: u16 = 512;
const BYTES_REQUIRED_TO_REPRESENT_MAX_INODE_COUNT: usize = MAX_SUPPORTED_INODE_COUNT as usize / 8;

use std::{fs::{ File, OpenOptions }, io::{Seek, SeekFrom, Write}, os::unix::fs::FileExt};

use crate::{block::{self, BlockBitmap, SuperBlock, SUPER_BLOCK_FILE_OFFSET}, inode::InodeBitmap};

use super::inode::{ Inode, INODE_SIZE, MAX_CHILDREN_COUNT, MAX_FILE_NAME_SIZE, FileType };
use bitvec::prelude::*;

struct ffs {
    super_block: SuperBlock,
    underlying_file: Option<File>,
    cwd: Inode,
    inode_bitmap: InodeBitmap,
    block_bitmap: BlockBitmap,
}

impl Default for ffs {
    fn default() -> Self {
        ffs {
            super_block: SuperBlock::default(),
            cwd: Inode::default(),
            underlying_file: None,
            inode_bitmap: InodeBitmap::default(),
            block_bitmap: BlockBitmap::default(),
        }
    }
}

impl Drop for ffs {
    fn drop(&mut self) {
        // let _ = self.flush_to_file();
    }
}

impl ffs {
    pub fn load(file_name: String) -> Option<Self> {
        let mut inst = ffs::default();

        match inst.init(false, &file_name, 0, 0, 0) {
            Ok(f) => {
                Some(inst)
            },
            Err(_) => None
        }
    }

    pub fn new(file_name: String, size: u32, block_size: u32, bytes_per_inode: u32) -> Option<Self> {
        let mut inst = ffs::default();

        match inst.init(true, &file_name, size, block_size, bytes_per_inode) {
            Ok(f) => {
                Some(inst)
            },
            Err(_) => None
        }
    }

    fn init
        (&mut self, new: bool, file_name: &String, size: u32, block_size: u32, bytes_per_inode: u32)
        -> Result<(), std::io::Error>
    {
        if new {
            return self.create_new(file_name, size, block_size, bytes_per_inode)
        } else {
            return self.load_existing(file_name);
        }
        // panic!("Unsupported operation: ffs::init called with new = false. This is not implemented yet.");
    }

    fn load_existing(&mut self, file_name: &String) -> Result<(), std::io::Error>
    {
        let ff = OpenOptions::new()
                        .create(false)
                        .read(true)
                        .write(true)
                        .open(file_name.clone());
        
        if ff.is_err() {
            return Err(ff.err().unwrap());
        }

        self.underlying_file = Some(ff.unwrap());

        if let Err(err) = self.load_super_block() {
            return Err(err);
        }

        self.load_root_inode();

        Ok(())
    }

    fn create_new
        (&mut self, file_name: &String, size: u32, block_size: u32, bytes_per_inode: u32)
        -> Result<(), std::io::Error>
    {
        let ff = OpenOptions::new()
                            .create(true)
                            .read(true)
                            .write(true)
                            .open(file_name.clone());
        if ff.is_err() {
            return Err(ff.err().unwrap());
        }

        self.underlying_file = Some(ff.unwrap());

        if let Err(err) = self.create_new_super_block(size, block_size, bytes_per_inode) {
            return Err(err);
        }

        self.create_root_inode();
        // self.init_free_inodes_list();

        // self.cwd = self.root_inode.clone();

        Ok(())
    }

    fn load_super_block(&mut self) -> Result<(), std::io::Error>
    {
        let tmp_res = self.fetch_super_block();
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }
        self.super_block = tmp_res.unwrap();

        Ok(())
    }


    fn create_new_super_block
        (&mut self, fs_size: u32, block_size: u32, bytes_per_inode: u32)
        -> Result<(), std::io::Error>
    {
        let super_block = SuperBlock::create_new(fs_size, block_size, bytes_per_inode);
        let tmp_res = self.persist_super_block(&super_block);
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }
        self.super_block = super_block;

        // Create the Inode Bitmap
        let inode_bitmap = InodeBitmap::new(self.super_block.get_total_inodes());
        let tmp_res = self.persist_inode_bitmap(&inode_bitmap);
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }
        self.inode_bitmap = inode_bitmap;
        
        let block_bitmap = BlockBitmap::new(self.super_block.get_total_blocks());
        let tmp_res = self.persist_block_bitmap(&block_bitmap);
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }
        self.block_bitmap = block_bitmap;

        Ok(())
    }

    fn fetch_super_block(&mut self) -> Result<SuperBlock, std::io::Error> {
        let f = self.underlying_file.as_mut().unwrap();

        let tmp_res = f.seek(SeekFrom::Start(SUPER_BLOCK_FILE_OFFSET));
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        SuperBlock::deserialize(f, self.super_block.get_block_size())
    }

    fn persist_super_block(&mut self, super_block: &SuperBlock) -> Result<(), std::io::Error> {
        let f = self.underlying_file.as_mut().unwrap();
        
        let tmp_res = f.seek(SeekFrom::Start(SUPER_BLOCK_FILE_OFFSET));
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        super_block.persist(f)
    }

    // fn fetch_inode_bitmap(&mut self) -> Result<InodeBitmap, std::io::Error> {
    //     let f = self.underlying_file.as_mut().unwrap();
    //     InodeBitmap::deserialize(f, &self.super_block)
    // }

    fn persist_inode_bitmap(&mut self, inode_bitmap: &InodeBitmap) -> Result<(), std::io::Error> {
        let f = self.underlying_file.as_mut().unwrap();
        inode_bitmap.persist(f, &self.super_block)
    }

    fn persist_block_bitmap(&mut self, block_bitmap: &BlockBitmap) -> Result<(), std::io::Error> {
        let f = self.underlying_file.as_mut().unwrap();
        block_bitmap.persist(f, &self.super_block)
    }

    fn load_root_inode(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
    
    fn create_root_inode(&mut self) -> Result<(), std::io::Error> {
        let f = self.underlying_file.as_mut().unwrap();
        let mut root_inode = Inode::default();
        root_inode.name = String::from("/");
        root_inode.inode_number = 0;
        root_inode.file_type = FileType::Directory;
        
        let tmp_res = root_inode.persist(f, &self.super_block);
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        self.inode_bitmap.allocate_inode(0);
        self.inode_bitmap.persist(f, &self.super_block)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;

    #[test]
    fn test_new_fs() {
        const TEST_FS_SIZE: u32 = 10 * (1 << 20); // 10 MB
        const BLOCK_SIZE: u32 = 4 * (1 << 10); // 4 KB
        const BYTES_PER_INODE: u32 = 1 << 12; // 4096 bytes per inode
        let FILE_NAME = "test_fs.dat".to_string();

        let fs = ffs::new(
            FILE_NAME,
            TEST_FS_SIZE,
            BLOCK_SIZE,
            BYTES_PER_INODE,
        );
        assert!(fs.is_some());
        let fs = fs.unwrap();
    }

    #[test]
    fn test_existing_fs() {
        let FILE_NAME = "test_fs.dat".to_string();

        let fs = ffs::load(FILE_NAME);
        assert!(fs.is_some());
        let fs = fs.unwrap();
    }
}


     