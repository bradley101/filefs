
use std::fs::{ File, OpenOptions };
use crate::core::block_bitmap::BlockBitmap;
use crate::core::super_block::SuperBlock;
use crate::core::inode::{ Inode, FileType };
use crate::core::inode_bitmap::InodeBitmap;

struct ffs {
    super_block: SuperBlock,
    underlying_file: Option<File>,
    cwd: Inode,
    root: Inode,
    inode_bitmap: InodeBitmap,
    block_bitmap: BlockBitmap,
}

impl Default for ffs {
    fn default() -> Self {
        ffs {
            super_block: SuperBlock::default(),
            underlying_file: None,
            cwd: Inode::default(),
            root: Inode::default(),
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

        if let Err(err) = self.fetch_super_block() {
            return Err(err);
        }

        if let Err(err) = self.load_root_inode() {
            return Err(err);
        }

        self.cwd = self.root.clone();
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

        if let Err(err) = self.create_root_inode() {
            return Err(err);
        }

        self.cwd = self.root.clone();
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
        let f = self.underlying_file.as_mut().unwrap();

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
        let f = self.underlying_file.as_mut().unwrap();
        self.super_block.persist(f)
    }

    fn persist_inode_bitmap(&mut self) -> Result<(), std::io::Error> {
        let f = self.underlying_file.as_mut().unwrap();
        self.inode_bitmap.persist(f, &self.super_block)
    }

    fn persist_block_bitmap(&mut self) -> Result<(), std::io::Error> {
        let f = self.underlying_file.as_mut().unwrap();
        self.block_bitmap.persist(f, &self.super_block)
    }

    fn load_root_inode(&mut self) -> Result<(), std::io::Error> {
        let f = self.underlying_file.as_mut().unwrap();
        
        let tmp_res = Inode::load(f, 0, &self.super_block);
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        self.root = tmp_res.unwrap();
        Ok(())
    }
    
    fn create_root_inode(&mut self) -> Result<(), std::io::Error> {
        let f = self.underlying_file.as_mut().unwrap();
        self.root = Inode::default();
        self.root.name = String::from("/");
        self.root.inode_number = 0;
        self.root.file_type = FileType::Directory;
        
        let tmp_res = self.root.persist(f, &self.super_block);
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        self.inode_bitmap.set(0);
        self.inode_bitmap.persist(f, &self.super_block)
    }

    fn create_new_file(&mut self, file_name: &String, file_type: FileType) -> Result<(), std::io::Error> {
        let new_inode = Inode::create_new(&self.cwd,
                                                                file_name,
                                                                FileType::File,
                                                                &self.super_block,
                                                                &self.inode_bitmap);
        if new_inode.is_err() {
            return Err(new_inode.err().unwrap());
        }

        let new_inode = new_inode.unwrap();
        let f = self.underlying_file.as_mut().unwrap();

        // TODO - make this whole operation atomic
        let tmp_res = new_inode.persist(f, &self.super_block);
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        self.inode_bitmap.set(new_inode.inode_number as usize);
        let tmp_res = self.persist_inode_bitmap();
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        Ok(())
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

        let fs = ffs::new(
            file_name,
            TEST_FS_SIZE,
            BLOCK_SIZE,
            BYTES_PER_INODE,
        );
        assert!(fs.is_some());
    }

    #[test]
    fn test_existing_fs() {
        let file_name = "test_fs.dat".to_string();
        let fs = ffs::load(file_name);
        assert!(fs.is_some());
    }
}
