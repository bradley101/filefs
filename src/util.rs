
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
