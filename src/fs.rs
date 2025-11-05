
use std::cell::RefCell;

use crate::core::inode::FileType;
use crate::entity::directory::Directory;
use crate::fs_metadata::fs_metadata;
use crate::medium::types::byte_compatible;

pub struct ffs<'a, T: byte_compatible> {
    metadata: fs_metadata<'a, T>,
    medium: RefCell<Option<T>>,
    cwd: Directory,
}

impl <'a, T: byte_compatible> Default for ffs<'a, T> {
    fn default() -> Self {
        ffs {
            metadata: fs_metadata::default(),
            medium: RefCell::new(None),
            cwd: Directory::default(),
        }
    }
}

impl <'a, T: byte_compatible> ffs<'a, T> {
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
        (&'a mut self, new: bool, medium: T, size: u32, block_size: u32, bytes_per_inode: u32)
        -> Result<(), std::io::Error>
    {
        self.medium = RefCell::new(Some(medium));
        if new {
            return self.create_new(size, block_size, bytes_per_inode)
        }
        self.load_existing()
    }

    fn load_existing(&'a mut self) -> Result<(), std::io::Error>
    {
        self.metadata = fs_metadata::fetch(&*self.medium.borrow_mut().as_mut().unwrap())?;
        self.cwd = Directory::load(
            0,
            &self.metadata,
            self.medium.as_mut().unwrap()
        )?;
        Ok(())
    }

    fn create_new
        (&'a mut self, size: u32, block_size: u32, bytes_per_inode: u32)
        -> Result<(), std::io::Error>
    {
        self.metadata = fs_metadata::create_new(
                            self.medium.as_ref().unwrap(),
                            size,
                            block_size,
                            bytes_per_inode)?;
        self.cwd = Directory::create_new(
                    FileType::Directory,
                    "/",
                    None,
                    &mut self.metadata)?;
        Ok(())
    }

    fn load_root_directory(&mut self) -> Result<(), std::io::Error> {
        self.cwd = Directory::load(
            0,
            &self.metadata,
            self.medium.as_mut().unwrap()
        )?;

        Ok(())
    }

    fn fetch_metadata(&'a mut self) -> Result<(), std::io::Error> {
        let f = self.medium.as_ref().unwrap();
        self.metadata = fs_metadata::fetch(f)?;
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
