
use std::cell::RefCell;
use std::rc::Rc;

use crate::core::inode::FileType;
use crate::entity::directory::Directory;
use crate::fs_metadata::fs_metadata;
use crate::medium::types::byte_compatible;

pub struct ffs<T: byte_compatible> {
    metadata: fs_metadata<T>,
    medium: Rc<RefCell<T>>,
    cwd: Directory,
}

impl <T: byte_compatible> ffs<T> {
    pub fn load(medium: T) -> Result<Self, std::io::Error> {
        let medium = Rc::new(RefCell::new(medium));
        let metadata = fs_metadata::fetch(medium.clone())?;
        let cwd = Directory::load(0, &metadata, medium.borrow_mut())?;

        Ok(Self { metadata, medium, cwd })
    }

    pub fn new(medium: T, size: u32, block_size: u32, bytes_per_inode: u32) -> Result<Self, std::io::Error> {
        let medium = Rc::new(RefCell::new(medium));
        let mut metadata = fs_metadata::create_new(medium.clone(),
                                                               size,
                                                               block_size,
                                                               bytes_per_inode)?;
        let cwd = Directory::new("/",
                                            None,
                                            &mut metadata)?;

        Ok(Self { metadata, medium, cwd })
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
        let medium = crate::medium::file::file_medium::new("test_fs.dat");

        let fs = ffs::new(
            medium,
            TEST_FS_SIZE,
            BLOCK_SIZE,
            BYTES_PER_INODE,
        );
        assert!(fs.is_ok());
    }

    #[test]
    fn test_existing_fs() {
        let medium = crate::medium::file::file_medium::new("test_fs.dat");
        let fs = ffs::load(medium);
        assert!(fs.is_ok());
    }
}
