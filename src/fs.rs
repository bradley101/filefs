

const FS_DESCRIPTOR_SIZE: usize = 32;
const FS_USED_SIZE: usize = 3;

struct FsDescriptor {
    pub version: [u8; 3],
    reserved: [u8; (FS_DESCRIPTOR_SIZE - FS_USED_SIZE)]
}

const NUM_RELEASED_VERSIONS: usize = 1;
const VALID_FS_VERSIONS: [[u8; 3]; NUM_RELEASED_VERSIONS]  = [
    [0, 0, 1]
];