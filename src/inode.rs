
pub const INODE_SIZE: usize = 256;
pub const USABLE_INODE_SIZE: usize = 2 
                                + 64 
                                + 2
                                + 1
                                + 2
                                + (2 * 64);


/*
    For now we only support files in our filesystem,
    Support for directories will be added later.
*/

pub struct Inode {
    pub inode_number: u16,                          // inode number of the file
    pub name: [u8; 64],                             // name of the file
    pub starting_block_number: u16,                 // starting block number of the file
    pub file_type: u8,                              // 0 for file, 1 for directory
    pub file_size: u16,                             // size of the file in bytes
    pub childrens: [u16; 64],                       // array of inode numbers of the children
    reserved: [u8; (INODE_SIZE - USABLE_INODE_SIZE) as usize],
}

impl Default for Inode {
    fn default() -> Self {
        Inode {
            inode_number: 0,
            name: [0; 64],
            starting_block_number: 0,
            file_type: 0,
            file_size: 0,
            childrens: [0; 64],
            reserved: [0; (INODE_SIZE - USABLE_INODE_SIZE) as usize],
        }
    }
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
            file_type: 0,
            file_size: 0,
            childrens: [0; 64],
            reserved: [0; (INODE_SIZE - USABLE_INODE_SIZE)]
        }
    }

    pub fn serialize(&self) -> [u8; INODE_SIZE] {
        let mut serialized_inode: [u8; INODE_SIZE] = [0; INODE_SIZE];
        serialized_inode[0..2].copy_from_slice(&self.inode_number.to_le_bytes());
        serialized_inode[2..66].copy_from_slice(&self.name);
        serialized_inode[66..68].copy_from_slice(&self.starting_block_number.to_le_bytes());
        serialized_inode[68] = self.file_type;
        serialized_inode[69..71].copy_from_slice(&self.file_size.to_le_bytes());
        
        for i in 0..64 {
            serialized_inode[71 + (i * 2)..73 + (i * 2)].copy_from_slice(&self.childrens[i].to_le_bytes());
        }
        
        serialized_inode[135..].copy_from_slice(&self.reserved);
        serialized_inode
    }
}
