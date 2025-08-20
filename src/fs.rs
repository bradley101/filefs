



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

// macro_rules! iter_children {
//     ($fs:expr) => {
//         $fs.cwd.childrens
//             .iter()
//             .filter(|child_inode| {   
//                 **child_inode > 0
//             })
//             .map(|child_inode| {
//                 $fs.fetch_inode(*child_inode).ok().unwrap()
//             })
//     };
// }

impl Drop for ffs {
    fn drop(&mut self) {
        // let _ = self.flush_to_file();
    }
}

impl ffs {
    pub fn new(file_name: String, size: u32, block_size: u32, bytes_per_inode: u32, new: bool) -> Option<Self> {
        let mut inst = ffs::default();

        match inst.init(new, &file_name, size, block_size, bytes_per_inode) {
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

        }
        panic!("Unsupported operation: ffs::init called with new = false. This is not implemented yet.");
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

    // fn init_free_inodes_list(&mut self) {
    //     self.free_inodes.clear();
        
    //     // Not including the root inode number "0" in the free inode list
    //     for inode_number in 1..self.get_max_inode_count() {
    //         self.free_inodes.push_back(inode_number);
    //     }
    // }

    fn load_super_block(&mut self) -> Result<(), std::io::Error>
    {
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

        
    }

    fn persist_super_block(&mut self, super_block: &SuperBlock) -> Result<(), std::io::Error> {
        let f = self.underlying_file.as_mut().unwrap();
        
        let tmp_res = f.seek(SeekFrom::Start(SUPER_BLOCK_FILE_OFFSET));
        if tmp_res.is_err() {
            return Err(tmp_res.err().unwrap());
        }

        super_block.persist(f)
    }

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

/*

    fn get_inode_offset(inode_number: u16) -> usize {
        INODE_STARTING_POS + (INODE_SIZE * inode_number as usize)
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
        if self.free_inodes.is_empty() || self.get_children_count(&self.cwd) == MAX_CHILDREN_COUNT {
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
 */
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;

    #[test]
    fn test_fs() {
        const TEST_FS_SIZE: u32 = 10 * (1 << 20); // 10 MB
        const BLOCK_SIZE: u32 = 4 * (1 << 10); // 4 KB
        const BYTES_PER_INODE: u32 = 1 << 12; // 4096 bytes per inode
        let FILE_NAME = "test_fs.dat".to_string();

        let fs = ffs::new(
            FILE_NAME,
            TEST_FS_SIZE,
            BLOCK_SIZE,
            BYTES_PER_INODE,
            true
        );
        assert!(fs.is_some());
        let fs = fs.unwrap();
    }

    /*
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
     */
}


     