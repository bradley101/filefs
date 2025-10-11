
pub const NUM_RELEASED_VERSIONS: usize = 1;
pub const VALID_FS_VERSIONS: [[u8; 3]; NUM_RELEASED_VERSIONS]  = [
    [0, 0, 1]
];
pub const CURRENT_FS_VERSION_IDX: usize = 0;

pub fn get_latest_version() -> [u8; 3] {
    VALID_FS_VERSIONS[CURRENT_FS_VERSION_IDX]
}

pub const MAX_FILE_NAME_SIZE: usize = 64;
pub const MAX_CHILDREN_COUNT: usize = 64;

pub const INODE_SIZE: usize = 256;
pub const INODE_BITMAP_STARTING_BLOCK_NUMBER: usize = 1;

pub const SUPER_BLOCK_FILE_OFFSET: u64 = 0;
pub const SUPER_BLOCK_SIZE: usize = 1 << 8;

pub trait Path {
    fn to_le_bytes(&self) -> &[u8];
    fn to_String(&self) -> String;
}

impl Path for String {
    fn to_le_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
    fn to_String(&self) -> String {
        self.clone()
    }
}

impl Path for &str {
    fn to_le_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
    fn to_String(&self) -> String {
        self.to_string()
    }
}

impl Path for [u8] {
    fn to_le_bytes(&self) -> &[u8] {
        self
    }
    fn to_String(&self) -> String {
        String::from_utf8(self.to_vec()).unwrap_or_default()
    }
}

impl Path for &[u8] {
    fn to_le_bytes(&self) -> &[u8] {
        self
    }
    fn to_String(&self) -> String {
        String::from_utf8(self.to_vec()).unwrap_or_default()
    }
}


