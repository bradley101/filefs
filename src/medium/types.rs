use std::io::Error;


pub enum medium {

}

pub trait byte_compatible {
    fn read_all() -> Result<Vec<u8>, Error>;
    fn write_all() -> Result<(), Error>;
}