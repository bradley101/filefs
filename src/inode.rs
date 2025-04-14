use std::char::MAX;

pub const MAX_FILE_NAME_SIZE: usize = 64;
pub const MAX_CHILDREN_COUNT: usize = 64;

pub const INODE_SIZE: usize = 256;
pub const USABLE_INODE_SIZE: usize = 2 
                                + 2
                                + MAX_FILE_NAME_SIZE 
                                + 2
                                + 1
                                + 2
                                + (2 * MAX_CHILDREN_COUNT);


/*
    For now we only support files in our filesystem,
    Support for directories will be added later.
*/


#[derive(Copy, Clone)]
pub struct Inode {
    pub inode_number: u16,                          // inode number of the file
    pub parent: u16,                                // inode number of the parent
    pub name: [u8; MAX_FILE_NAME_SIZE],                             // name of the file
    pub starting_block_number: u16,                 // starting block number of the file
    pub file_type: u8,                              // 0 for file, 1 for directory
    pub file_size: u16,                             // size of the file in bytes
    pub childrens: [u16; MAX_CHILDREN_COUNT],                       // array of inode numbers of the children
    reserved: [u8; (INODE_SIZE - USABLE_INODE_SIZE) as usize],
}

impl Default for Inode {
    fn default() -> Self {
        Inode {
            inode_number: 0,
            parent: 0,
            name: [0; MAX_FILE_NAME_SIZE],
            starting_block_number: 0,
            file_type: 0,
            file_size: 0,
            childrens: [0; MAX_CHILDREN_COUNT],
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
    pub fn new(name: &[u8]) -> Inode {
        let mut inode = Inode::default();
        inode.name[..name.len().min(MAX_FILE_NAME_SIZE)].copy_from_slice(name);
        inode
    }

    pub fn serialize(&self) -> [u8; INODE_SIZE] {
        let mut serialized_inode: [u8; INODE_SIZE] = [0; INODE_SIZE];
        serialized_inode[0..2].copy_from_slice(&self.inode_number.to_le_bytes());
        serialized_inode[2..4].copy_from_slice(&self.parent.to_le_bytes());
        serialized_inode[4..68].copy_from_slice(&self.name);
        serialized_inode[68..70].copy_from_slice(&self.starting_block_number.to_le_bytes());
        serialized_inode[70] = self.file_type;
        serialized_inode[71..73].copy_from_slice(&self.file_size.to_le_bytes());
        
        for i in 0..MAX_CHILDREN_COUNT {
            serialized_inode[73 + (i * 2)..75 + (i * 2)].copy_from_slice(&self.childrens[i].to_le_bytes());
        }
        
        serialized_inode[USABLE_INODE_SIZE..USABLE_INODE_SIZE + self.reserved.len()].copy_from_slice(&self.reserved);
        serialized_inode
    }
}
