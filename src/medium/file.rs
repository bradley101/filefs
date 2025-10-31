use std::fs::{OpenOptions, File};
use std::io::Error;
use std::os::unix::fs::FileExt;
use crate::{medium::types::byte_compatible, util::Path};


pub struct file_medium {
    file: File
}

impl file_medium {
    pub fn new<T: Path>(path: T) -> Self {
        let file = file_medium::create_file_obj(path, false);
        Self { file }
    }

    pub fn load<T: Path>(path: T) -> Self {
        let file = file_medium::create_file_obj(path, true);
        Self { file }
    }

    fn create_file_obj<T: Path>(path: T, existing: bool) -> File {
        let file = OpenOptions::new()
                .create(!existing)
                .read(true)
                .write(true)
                .open(path.to_String());

        if file.is_err() {
            panic!("{}", if existing { "Cannot load the file" }
                   else { "Cannot create the file" });
        }

        file.unwrap()
    }
}

impl byte_compatible for file_medium {
    fn read_all(&self, offset: u64, len: usize, buffer: &mut [u8]) -> Result<(), Error>
    {
        self.file.read_exact_at(buffer, offset)
    }

    fn write_all(&self, offset: u64, len: usize, buffer: &[u8]) -> Result<(), Error>
    {
        self.file.write_all_at(buffer, offset)
    }
}