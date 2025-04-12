

const NUM_RELEASED_VERSIONS: usize = 1;
const VALID_FS_VERSIONS: [[u8; 3]; NUM_RELEASED_VERSIONS]  = [
    [0, 0, 1]
];
const CURRENT_FS_VERSION_IDX: usize = 0;

const FS_DESCRIPTOR_SIZE: usize = 32;
const FS_USED_SIZE: usize = 3;


#[derive(Default)]
struct FsDescriptor {
    pub version: [u8; 3],
    reserved: [u8; (FS_DESCRIPTOR_SIZE - FS_USED_SIZE)]
}

use std::{fs::{ File, OpenOptions }, io::{Seek, SeekFrom, Write}};

#[derive(Default)]
struct ffs {
    opened_file: Option<File>,
    name: String,
    size: u32
}

impl ffs {
    pub fn new(file_name: String, size: u32) -> Option<Self> {
        let mut inst = ffs::default();
        inst.name = file_name;
        inst.size = size;

        match inst.init() {
            Ok(f) => {
                inst.opened_file = Some(f);
                Some(inst)
            },
            Err(_) => None
        }

    }

    fn init(&self) -> Result<File, String> {
        let ff = OpenOptions::new()
                            .create_new(true)
                            .read(true)
                            .write(true)
                            .open(self.name.clone());
        
        if ff.is_ok() {
            let mut f = ff.unwrap();
            match self.write_fs_desc(&mut f) {
                Ok(_) => Ok(f),
                Err(err) => Err(err)
            }
        } else {
            Err(ff.err().unwrap().to_string())
        }

    }

    fn write_fs_desc(&self, f: &mut File) -> Result<(), String>{
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

}


     