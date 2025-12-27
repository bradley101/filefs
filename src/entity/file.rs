use std::io::Error;

use crate::{core::inode::{FileType, Inode}, entity::directory::Directory, fs_metadata::fs_metadata, medium::types::byte_compatible, util::Path};

pub struct File {
    inode: Inode
}

impl File {
    pub fn new<T: Path, M: byte_compatible>(
        name: T,
        parent: &Directory,
        metadata: &mut fs_metadata<M>) -> Result<Self, Error>
    {
        let empty_block = match parent.get_first_empty_block() {
            Some(v) => v,
            None => return Err(std::io::Error::new(
                               std::io::ErrorKind::Other,
                               "No free blocks available in directory"))
        };
        

        let inode = Inode::create_new(
            parent.get_inode_number(),
            name,
            FileType::File,
            metadata)?;
        
        


    }
}
