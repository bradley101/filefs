

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

use std::{fs::{ File, OpenOptions }, io::{Seek, SeekFrom, Write}, os::unix::fs::FileExt};
use std::collections::LinkedList;
use super::inode::{ Inode, INODE_SIZE, MAX_FILE_NAME_SIZE};

struct ffs {
    opened_file: Option<File>,
    name: String,
    size: u32,
    root_inode: Inode,
    cwd: Inode,
    free_inodes: LinkedList<u16>
}

trait Path {
    fn byte_array(&self) -> &[u8];
}

impl Path for String {
    fn byte_array(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Path for &str {
    fn byte_array(&self) -> &[u8] {
        return self.as_bytes()
    }
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
                            .create(true)
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

        self.cwd = self.root_inode.clone();

        Ok(())
    }

    fn init_free_inodes_list(&mut self) {
        self.free_inodes.clear();
        
        // Not including the root inode number "0" in the free inode list
        for inode_number in 1..self.get_max_inode_count() {
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
        let mut root_inode = Inode::default();
        let root_name_bytes = "/".as_bytes();
        root_inode.name[..root_name_bytes.len().min(MAX_FILE_NAME_SIZE)].copy_from_slice(&root_name_bytes[..]);
        root_inode.file_type = 1;
        match self.persist_inode(&root_inode) {
            Ok(_) => {
                self.root_inode = root_inode;
                Ok(())
            },
            Err(err) => Err(err)
        }
        
    }

    fn persist_inode(&mut self, inode: &Inode) -> Result<(), String> {
        let inode_offset = INODE_STARTING_POS + (INODE_SIZE * inode.inode_number as usize);

        let inode_bytes= inode.serialize();
        let of = self.opened_file.as_mut().unwrap();

        match of.write_all_at(&inode_bytes, inode_offset as u64) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string())
        }
    }

    fn touch<T: Path>(&mut self, name: T) -> Result<(), String> {
        let mut inode = Inode::new(name.byte_array());

        if self.free_inodes.is_empty() {
            return Err(String::from("No free inodes available"));
        }

        let mut cwd = self.cwd.clone();
        inode.inode_number = self.free_inodes.pop_front().unwrap();
        inode.parent = self.cwd.inode_number;
        cwd.childrens[self.get_children_count(&cwd) as usize] = inode.inode_number;

        if let Err(err) = self.persist_inode(&inode) {
            return Err(err);
        }

        if let Err(err) = self.persist_inode(&cwd) {
            return Err(err);
        }
            
        self.cwd = cwd;
        Ok(())
    }

    fn ls<T: Path>(&mut self, name: T) -> Result<(), String> {
        Ok(())
    }

    fn get_children_count(&self, wd: &Inode) -> usize {
        wd.childrens.iter().take_while(|&&child| child != 0).count()
    }

    fn get_cwd(&self) -> Inode {
        self.cwd.clone()
    }

    fn flush_to_file(&mut self) -> Result<(), std::io::Error> {
        self.opened_file.as_mut().unwrap().flush()
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;

    #[test]
    fn test_fs() {
        let fs = ffs::new(
            "test_fs.dat".to_string(),
            1024 * 1024 * 10);
        assert!(fs.is_some());
        let fs = fs.unwrap();
        remove_file(fs.name).unwrap();
    }

    #[test]
    fn test_fs_touch() {
        let fs = ffs::new(
            "test_fs_touch.dat".to_string(),
            1024 * 1024 * 10);
        assert!(fs.is_some());
        let mut fs = fs.unwrap();

        let x1 = fs.touch("file1");
        assert!(x1.is_ok());

        let x2 = fs.touch("file2");
        assert!(x2.is_ok());

        let x3 = fs.touch("file3");
        assert!(x3.is_ok());

        let cwd = fs.get_cwd();

        assert!(fs.get_children_count(&cwd) == 3);
        // remove_file(fs.name).unwrap();
    }

}


     