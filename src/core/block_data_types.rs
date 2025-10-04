/*
    This file defines the kind of data that can be stored in a block pointed by inodes.
    Possible data types are:
        a. Super Block (for the filesystem)
        b. Inode Bitmap
        c. Block Bitmap
        d. Inode Data (for inodes)
        e. User Data (for files)
        f. Indirect Block Pointers (for both files and directories)
        g. Children Inode Numbers (for directories)
*/

#[derive(Default)]
pub enum BlockDataType {
    SuperBlock,
    InodeBitmap,
    BlockBitmap,
    InodeData,
    UserData,
    IndirectBlockPointers,
    ChildrenInodeNumbers,
    #[default]
    Other
}

