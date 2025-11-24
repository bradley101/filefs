use std::io::Error;

use crate::{core::inode::{FileType, Inode}, entity::directory::Directory, fs_metadata::fs_metadata, medium::types::byte_compatible, util::Path};

pub struct file {
    inode: Inode
}

impl file {
    pub fn new<T: Path, M: byte_compatible>(
        name: T,
        parent: &Directory,
        metadata: &mut fs_metadata<M>) -> Result<Self, Error>
    {
        Ok(Self {
            inode: Inode::create_new(
                parent.get_inode_number(),
                name,
                FileType::File,
                metadata)?
        })
    }
}