use std::io::Error;


pub enum medium {

}

pub trait byte_compatible {
    fn read_all(&self, offset: u64, len: usize, buffer: &mut [u8]) -> Result<(), Error>;
    fn write_all(&self, offset: u64, len: usize, bufffer: &[u8]) -> Result<(), Error>;
}