
const INODE_SIZE: usize = 256;
const USABLE_INODE_SIZE: usize = 4 
                                + 64 
                                + 4;


/*
    For now we only support files in our filesystem,
    Support for directories will be added later.
*/

pub struct Inode {
    pub inode_number: u32,
    pub name: [u8; 64],
    pub starting_block_number: u32,
    reserved: [u8; (INODE_SIZE - USABLE_INODE_SIZE) as usize],
}

/*
    As this is a file based virtual filesystem,
    users may need to store temporary files,
    so the filesystem size may be very less,
    so that said, we have to support very small file sizes,
    hence very small inodes, and very small data blocks.
    This will not be a thin-client like FS.


    So we have to come up with a table to mention how many 
    inodes we can store depending on the filesystem size.

    The table is as follows:-

    File System Size     | No of Inodes  | Size of Inodes
    ---------------------|---------------|-----------------
          <=  1 MB       |      8        |      2K
          <=  10 MB      |      64       |      16K
          >= 100 MB      |      512      |      128K


*/

impl Inode {
    pub fn new(name: [u8; 64]) -> Inode {
        Inode {
            inode_number: 0,
            name: name,
            starting_block_number: 0,
            reserved: [0; (INODE_SIZE - USABLE_INODE_SIZE)]
        }
    }
}