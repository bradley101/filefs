

const NUM_RELEASED_VERSIONS: usize = 1;
const VALID_FS_VERSIONS: [[u8; 3]; NUM_RELEASED_VERSIONS]  = [
    [0, 0, 1]
];
const CURRENT_FS_VERSION_IDX: usize = 0;

const FS_DESCRIPTOR_SIZE: usize = 128;
const FS_USED_SIZE: usize = 2
                            + BYTES_REQUIRED_TO_REPRESENT_MAX_INODE_COUNT
                            + 3;

const INODE_STARTING_POS: usize = FS_DESCRIPTOR_SIZE;

const MAX_SUPPORTED_INODE_COUNT: u16 = 512;
const BYTES_REQUIRED_TO_REPRESENT_MAX_INODE_COUNT: usize = MAX_SUPPORTED_INODE_COUNT as usize / 8;

struct FsDescriptor {
    pub total_inodes: u16,
    pub free_inode_list_idx: [u8; MAX_SUPPORTED_INODE_COUNT as usize / 8],
    pub version: [u8; 3],
    reserved: [u8; FS_DESCRIPTOR_SIZE - FS_USED_SIZE]
}

impl Default for FsDescriptor {
    fn default() -> Self {
        FsDescriptor {
            total_inodes: 0,
            free_inode_list_idx: [0; BYTES_REQUIRED_TO_REPRESENT_MAX_INODE_COUNT],
            version: [0; 3],
            reserved: [0; FS_DESCRIPTOR_SIZE - FS_USED_SIZE]
        }
    }
}

use std::{fs::{ File, OpenOptions }, io::{Seek, SeekFrom, Write}, os::unix::fs::FileExt};
use std::collections::LinkedList;

use super::inode::{ Inode, INODE_SIZE, MAX_CHILDREN_COUNT, MAX_FILE_NAME_SIZE, FileTypes };

struct ffs {
    opened_file: Option<File>,
    name: String,
    size: u32,
    root_inode: Inode,
    cwd: Inode,
    free_inodes: LinkedList<u16>,
    free_blocks: LinkedList<u16>
}

trait Path {
    fn byte_array(&self) -> &[u8];
    fn string(&self) -> String;
}

impl Path for String {
    fn byte_array(&self) -> &[u8] {
        self.as_bytes()
    }

    fn string(&self) -> String {
        self.to_string()
    }
}

impl Path for &str {
    fn byte_array(&self) -> &[u8] {
        self.as_bytes()
    }

    fn string(&self) -> String {
        String::from(*self)
    }
}

impl Path for &[u8] {
    fn byte_array(&self) -> &[u8] {
        *self
    }

    fn string(&self) -> String {
        String::from_utf8_lossy(*self).trim_end_matches('\0').to_string()
    }
}

struct DirItem {
    name: String,
    item_type: u8,
    size: u16
}

impl DirItem {
    pub fn from(inode: &Inode) -> Self {
        Self {
            name: String::from_utf8_lossy(&inode.name).trim_end_matches('\0').to_string(),
            item_type: inode.file_type,
            size: inode.file_size
        }
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
            free_inodes: LinkedList::new(),
            free_blocks: LinkedList::new()
        }
    }
}

macro_rules! iter_children {
    ($fs:expr) => {
        $fs.cwd.childrens
            .iter()
            .filter(|child_inode| {   
                **child_inode > 0
            })
            .map(|child_inode| {
                $fs.fetch_inode(*child_inode).ok().unwrap()
            })
    };
}

impl Drop for ffs {
    fn drop(&mut self) {
        let _ = self.flush_to_file();
    }
}

impl ffs {
    pub fn new(file_name: String, size: u32, new: bool) -> Option<Self> {
        let mut inst = ffs::default();
        inst.name = file_name;
        inst.size = size;

        match inst.init(new) {
            Ok(f) => {
                Some(inst)
            },
            Err(_) => None
        }
    }

    fn init(&mut self, new: bool) -> Result<(), String> {
        if new {
            self.create_new()
        } else {
            Err(String::from("Currently not supporting"))
        }
    }

    fn create_new(&mut self) -> Result<(), String> {
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

    fn read_fs_desc(&self) -> Result<FsDescriptor, String> {
        let f = self.opened_file.as_ref().unwrap();
        
        let mut desc = FsDescriptor::default();
        let desc_bytes = unsafe {
            std::slice::from_raw_parts_mut(
                &mut desc as *mut FsDescriptor as *mut u8, FS_DESCRIPTOR_SIZE)
        };
        
        match f.read_exact_at(desc_bytes, 0) {
            Ok(_) => Ok(desc),
            Err(err) => Err(String::from("error in reading fs descriptor"))
        }
    }

    fn write_fs_desc(&mut self) -> Result<(), String> {
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
            MAX_SUPPORTED_INODE_COUNT
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

    fn get_inode_offset(inode_number: u16) -> usize {
        INODE_STARTING_POS + (INODE_SIZE * inode_number as usize)
    }

    fn persist_inode(&mut self, inode: &Inode) -> Result<(), String> {
        let inode_offset = ffs::get_inode_offset(inode.inode_number);

        let inode_bytes= inode.serialize();
        let of = self.opened_file.as_mut().unwrap();

        match of.write_all_at(&inode_bytes, inode_offset as u64) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string())
        }
    }

    fn fetch_inode(&self, inode_number: u16) -> Result<Inode, String> {
        let inode_offset = ffs::get_inode_offset(inode_number);

        let mut inode = Inode::default();
        let inode_bytes = unsafe {
            std::slice::from_raw_parts_mut(
                &mut inode as *mut Inode as *mut u8,
                std::mem::size_of::<Inode>())
        };

        if let Err(err) = self.opened_file.as_ref().unwrap()
            .read_exact_at(inode_bytes, inode_offset as u64) {
            return Err(err.to_string());
        }

        Ok(inode)
    }

    fn check_if_already_exists(&self, name_bytes: &[u8]) -> Option<Inode> {
        iter_children!(&self)
            .find(|child| {
                String::from_utf8_lossy(child.name.as_slice())
                    .trim_end_matches('\0').to_string().as_bytes() == name_bytes
            })
    }

    fn create_item<T: Path>(&mut self, name: T, file_type: FileTypes) -> Result<(), String> {
        if self.free_inodes.is_empty() && self.get_children_count(&self.cwd) == MAX_CHILDREN_COUNT {
            return Err(String::from("no space left"));
        }
        
        {
            let existing_inode = self.check_if_already_exists(name.byte_array());
            if existing_inode.is_some() {
                return Err(String::from("file already exists"));
            }
        }
        
        let mut inode = Inode::new(name.byte_array());
        let mut cwd = self.cwd.clone();
        inode.inode_number = self.free_inodes.pop_front().unwrap();
        inode.parent = self.cwd.inode_number;
        inode.file_type = file_type as u8;
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

    fn mkdir<T: Path>(&mut self, dir_name: T) -> Result<(), String> {
        self.create_item(dir_name, FileTypes::Directory)
    }


    fn touch<T: Path>(&mut self, name: T) -> Result<(), String> {
        self.create_item(name, FileTypes::File)
    }

    fn cd<T: Path>(&mut self, name: T) -> Result<(), String> {
        if name.byte_array() == "..".as_bytes() {
            self.cwd = self.fetch_inode(self.cwd.parent)?;
            return Ok(());
        }

        let existing_inode = self.check_if_already_exists(name.byte_array());
        if existing_inode.is_none() {
            return Err(format!("{} not found", name.string()))
        }

        self.cwd = existing_inode.unwrap();
        Ok(())
    }

    fn ls(&self) -> Result<Vec<DirItem>, String> {
        // TODO -  Currently not supporting names
        let mut items: Vec<DirItem> = Vec::with_capacity(MAX_CHILDREN_COUNT);
        
        iter_children!(&self)
            .map(|inode| {
                DirItem::from(&inode)
            })
            .for_each(|item| {
                items.push(item);
            });
        
        Ok(items)
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
            1024 * 1024 * 10,
            true
        );
        assert!(fs.is_some());
        let fs = fs.unwrap();
    }

    #[test]
    fn test_fs_touch() {
        let fs = ffs::new(
            "test_fs_touch.dat".to_string(),
            1024 * 1024 * 10,
            true
        );
        assert!(fs.is_some());
        let mut fs = fs.unwrap();

        let x1 = fs.touch("file1");
        assert!(x1.is_ok());

        let x2 = fs.touch("file2");
        assert!(x2.is_ok());

        let x3 = fs.touch("file3");
        assert!(x3.is_ok());

        let x4 = fs.touch("file3");
        assert!(x4.as_ref().is_err());

        println!("Error is -> {}", x4.err().unwrap());

        let cwd = fs.get_cwd();

        let I: usize = std::mem::size_of::<Inode>();

        assert!(fs.get_children_count(&cwd) == 3);
    }

    #[test]
    fn test_ls() {
        let fs = ffs::new(
            "test_ls.dat".to_string(),
            1024 * 1024 * 10, 
            true);
        assert!(fs.is_some());
        let mut fs = fs.unwrap();
        
        for i in 1..64 {
            fs.touch(format!("file{}", i));
        }

        let items = fs.ls().ok().unwrap();

        assert!(items.len() == 63)
    }

    #[test]
    fn test_mkdir_cd() {
        let fs = ffs::new(
            "test_mkdir_cd.dat".to_string(),
            1024 * 1024 * 10, true);
        assert!(fs.is_some());
        let mut fs = fs.unwrap();

        assert_eq!(fs.get_cwd().name.as_slice().string(), "/");
        fs.mkdir("dir1");

        let x = fs.ls().ok().unwrap();
        assert_eq!(x.len(), 1);
        assert_eq!(x[0].name, "dir1");
        
        fs.cd("dir1");
        assert_eq!(fs.get_cwd().name.as_slice().string(), "dir1");

        let x = fs.ls().ok().unwrap();
        assert_eq!(x.len(), 0);

        fs.cd("..");
        let x = fs.ls().ok().unwrap();
        assert_eq!(x.len(), 1);
        assert_eq!(x[0].name, "dir1");
    }

}


     