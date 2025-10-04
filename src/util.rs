use std::io::Write;

use crate::{block::{Block, SuperBlock}, inode::InodeBitmap};


pub trait Path {
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

pub const NUM_RELEASED_VERSIONS: usize = 1;
pub const VALID_FS_VERSIONS: [[u8; 3]; NUM_RELEASED_VERSIONS]  = [
    [0, 0, 1]
];
pub const CURRENT_FS_VERSION_IDX: usize = 0;

pub fn get_latest_version() -> [u8; 3] {
    VALID_FS_VERSIONS[CURRENT_FS_VERSION_IDX]
}

pub trait Persist {
    fn persist(&self, file: &mut std::fs::File) -> std::io::Result<()>;
}

impl Persist for SuperBlock {
    fn persist(&self, file: &mut std::fs::File) -> std::io::Result<()> {
        // use serde to serialize and write this superblock in the file
        let serialized = bincode::serialize(self).expect("Failed to serialize SuperBlock");
        file.write_all(serialized.as_slice())
    }
}

impl Persist for Block {
    fn persist(&self, file: &mut std::fs::File) -> std::io::Result<()> {
        // use serde to serialize and write this block in the file
        let serialized = bincode::serialize(self).expect("Failed to serialize Block");
        file.write_all(serialized.as_slice())
    }
}

// impl From<SuperBlock> for Block {
//     fn from(super_block: SuperBlock) -> Self {
//         let data = bincode::serialize(&super_block)
//             .expect("Failed to serialize SuperBlock into Block");
//         Block { data }
//     }
// }

// impl TryFrom<Block> for InodeBitmap {
//     type Error = std::io::Error;

//     fn try_from(block: Block) -> Result<Self, Self::Error> {
//         let bitmap: InodeBitmap = bincode::deserialize(&block.data)
//             .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to deserialize InodeBitmap"))?;
//         Ok(bitmap)
//     }
// }


