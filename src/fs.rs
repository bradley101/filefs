

const NUM_RELEASED_VERSIONS: usize = 1;
const VALID_FS_VERSIONS: [[u8; 3]; NUM_RELEASED_VERSIONS]  = [
    [0, 0, 1]
];
const CURRENT_FS_VERSION_IDX: usize = 0;

const FS_DESCRIPTOR_SIZE: usize = 32;
const FS_USED_SIZE: usize = 3;

const INODE_STARTING_POS: usize = FS_DESCRIPTOR_SIZE;

#[derive(Default)]
struct FsDescriptor {
    pub version: [u8; 3],
    reserved: [u8; (FS_DESCRIPTOR_SIZE - FS_USED_SIZE)]
}

use std::{fs::{ File, OpenOptions }, io::{Seek, SeekFrom, Write}};
use std::collections::LinkedList;
use super::inode::{ Inode, INODE_SIZE };

struct ffs {
    opened_file: Option<File>,
    name: String,
    size: u32,
    root_inode: Inode,
    cwd: Inode,
    free_inodes: LinkedList<u16>
}

impl Default for ffs {
    fn default() -> Self {
        ffs {
            opened_file: None,
            name: String::new(),
            size: 0,
            root_inode: Inode::default(),
            cwd: Inode::default(),
            free_inodes: LinkedList::new()
        }
    }
}

impl ffs {
    pub fn new(file_name: String, size: u32) -> Option<Self> {
        let mut inst = ffs::default();
        inst.name = file_name;
        inst.size = size;

        match inst.init() {
            Ok(f) => {
                Some(inst)
            },
            Err(_) => None
        }

    }

    fn init(&mut self) -> Result<(), String> {
        let ff = OpenOptions::new()
                            .truncate(true)
                            .create_new(true)
                            .read(true)
                            .write(true)
                            .open(self.name.clone());
        if ff.is_err() {
            return Err(ff.err().unwrap().to_string());
        }

        self.opened_file = Some(ff.unwrap());

        match self.write_fs_desc() {
            Err(err) => return Err(err),
            _ => {}
        }

        self.create_root_inode();
        self.init_free_inodes_list();
        Ok(())
    }

    fn init_free_inodes_list(&mut self) {
        let MAX_INODE_COUNT: u16 = self.get_max_inode_count();
        let TOTAL_INODE_SIZE: usize = (MAX_INODE_COUNT as usize) * INODE_SIZE as usize;
        self.free_inodes.clear();
        
        // Not including the root inode number "0" in the free inode list
        for inode_number in 1..MAX_INODE_COUNT {
            self.free_inodes.push_back(inode_number);
        }
    }

    fn write_fs_desc(&mut self) -> Result<(), String>{
        let f = self.opened_file.as_mut().unwrap();

        if f.seek(SeekFrom::Start(0)).is_err() {
            return Err(String::from("Unable to seek to start"));
        }

        let mut fs_desc = FsDescriptor::default();
        fs_desc.version = VALID_FS_VERSIONS[CURRENT_FS_VERSION_IDX].clone();

        let bytes = unsafe {
            std::slice::from_raw_parts(
                &fs_desc as *const FsDescriptor as *const u8,
                std::mem::size_of::<FsDescriptor>())
        };

        match f.write_all(&bytes) {
            Ok(_) => {
                match f.write_all(&vec![0u8; self.size as usize - bytes.len()]) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(err.to_string())
                }
            },
            Err(err) => Err(err.to_string())
        }
    }

    fn get_max_inode_count(&self) -> u16 {
        if self.size <= 1024 * 1024 {
            8u16
        } else if self.size <= 10 * 1024 * 1024 {
            64u16
        } else {
            512u16
        }
    }

    fn create_root_inode(&mut self) -> Result<(), String> {
        self.root_inode = Inode::default();
        {
            let mut ri = &mut self.root_inode;
            ri.name.copy_from_slice("/".as_bytes());
            ri.file_type = 1;
        }
        self.persist_inode(&self.root_inode)
    }

    fn persist_inode(&mut self, inode: &Inode) -> Result<(), String> {
        let inode_offset = INODE_STARTING_POS + (INODE_SIZE * inode.inode_number as usize);
        match self.opened_file.as_mut().unwrap().write_all(&inode.serialize()) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string())
        }

    }

}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::fs::remove_file;

    #[test]
    fn test_fs() {
        let fs = ffs::new(
            "test_fs.dat".to_string(),
            1024 * 1024 * 10);
        assert!(fs.is_some());
        let fs = fs.unwrap();
        // remove_file(fs.name).unwrap();
    }
}


     